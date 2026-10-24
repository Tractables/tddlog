#!/usr/bin/env python3
"""A Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020) as
code over finite variables, for the translators into imperative,
sum-product and lazy functional languages (dice2genfer.py, dice2sppl.py,
dice2pluck.py).

The program is read by dice2problog.py's parser and lowered to a list of
statements over variables (`Var`), each a Boolean (0 or 1) or an int(w)
(0 to 2^w - 1), defined once:

    ('sample', v, 'flip', p)            v ~ Bernoulli(p)
    ('sample', v, 'discrete', ps)       v ~ Categorical(ps)
    ('sample', v, 'uniform', (lo, hi))  v uniform on lo .. hi - 1
    ('op', v, OP, args)                 v = OP(args), pure
    ('observe', a)                      a holds
    ('if', c, then, else, outs)         the branches, each a list; after
                                        it, v = a or b for (v, a, b) in outs
    ('loop', k, states, inits, body, nexts)
                                        states = inits, then k times: body,
                                        states = nexts

An argument is a `Var` or a `Const`. The pure operators are `not`, `and`,
`or`, `xor`, `iff`, `eq`, `ne`, `lt`, `le`, `add`, `sub`, `mul` (modulo
2^w, as Dice's), `shl` and `shr` (by a constant, the second argument a
Python int), `ite` (c, a, b) and `copy`. Each variable carries the values
it can take (`support`), and a variable with one value is that constant:
operations on constants are folded. A tuple is a `Tup` of values, resolved
statically; a function is inlined at each call (no recursion), and
`iterate(f, x, k)` is a loop of k inlined calls. A flip inside a branch
stays in it, as does an observe.

`prune` drops what a set of variables and the observations do not need.
"""
from decimal import Decimal
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
from dice2problog import Parser, bits_for, calls, deep, discrete_probs, tokenize  # noqa: E402

BOOL = ('bool',)


class Var:
    __slots__ = ('n', 't', 'support')

    def __init__(self, n, t, support):
        self.n, self.t, self.support = n, t, tuple(sorted(support))

    def __repr__(self):
        return f'v{self.n}'


class Const:
    __slots__ = ('t', 'k')

    def __init__(self, t, k):
        self.t, self.k = t, k

    @property
    def support(self):
        return (self.k,)

    def __eq__(self, other):
        return isinstance(other, Const) and (self.t, self.k) == (other.t, other.k)

    def __hash__(self):
        return hash((self.t, self.k))

    def __repr__(self):
        return f'c{self.k}'


class Tup:
    __slots__ = ('a', 'b')

    def __init__(self, a, b):
        self.a, self.b = a, b

    @property
    def t(self):
        return ('tuple', self.a.t, self.b.t)


TRUE, FALSE = Const(BOOL, 1), Const(BOOL, 0)


def show(t):
    if t[0] == 'bool':
        return 'bool'
    if t[0] == 'int':
        return f'int({t[1]})'
    return f'({show(t[1])}, {show(t[2])})'


def same(a, b, what):
    if a != b:
        raise TypeError(f'{what}: {show(a)} and {show(b)}')
    return a


def size(t):
    return 2 if t == BOOL else 2 ** t[1]


def leaves(v):
    """The components of a value, left to right through nested tuples."""
    return leaves(v.a) + leaves(v.b) if isinstance(v, Tup) else [v]


def apply(op, t, xs):
    """OP on the numbers xs, the result of type t (an operand's type for the
    integer operators)."""
    m = size(t)
    a = xs[0]
    b = xs[1] if len(xs) > 1 else None
    return {'not': lambda: 1 - a, 'and': lambda: a & b, 'or': lambda: a | b, 'xor': lambda: a ^ b,
            'iff': lambda: 1 - (a ^ b), 'eq': lambda: int(a == b), 'ne': lambda: int(a != b),
            'lt': lambda: int(a < b), 'le': lambda: int(a <= b), 'add': lambda: (a + b) % m,
            'sub': lambda: (a - b) % m, 'mul': lambda: (a * b) % m, 'shl': lambda: (a << b) % m,
            'shr': lambda: a >> b, 'ite': lambda: b if a else xs[2], 'copy': lambda: a}[op]()


BOOLEAN = {'not', 'and', 'or', 'xor', 'iff', 'eq', 'ne', 'lt', 'le'}


def args_of(s):
    """The variables a statement reads (of an if: its condition and the
    sources of its outs; of a loop: its inits and nexts)."""
    kind = s[0]
    if kind == 'op':
        xs = s[3][:1] if s[2] in ('shl', 'shr') else s[3]
    elif kind == 'observe':
        xs = [s[1]]
    elif kind == 'if':
        xs = [s[1]] + [x for _, a, b in s[4] for x in (a, b)]
    elif kind == 'loop':
        xs = list(s[3]) + list(s[5])
    else:
        xs = []
    return [x for x in xs if isinstance(x, Var)]


class Lower:
    def __init__(self):
        self.n = 0
        self.funs = {}

    def var(self, t, support):
        self.n += 1
        return Var(self.n, t, support)

    # --- pure operations, folded where an operand decides ------------------

    def op(self, out, op, t, args):
        """The value of OP(args) (t its type), a statement in out where it
        is not a constant."""
        if op in ('shl', 'shr'):
            x, k = args
            if isinstance(x, Const):
                return Const(t, apply(op, t, [x.k, k]))
            if op == 'shl' and 2 ** k >= size(t) or op == 'shr' and k >= t[1]:
                return Const(t, 0)
            if k == 0:
                return x
            support = {apply(op, t, [a, k]) for a in x.support}
        else:
            if all(isinstance(a, Const) for a in args):
                return Const(t, apply(op, args[0].t if op not in ('ite',) else t, [a.k for a in args]))
            folded = self.fold(out, op, t, args)
            if folded is not None:
                return folded
            st = args[0].t if op in ('eq', 'ne', 'lt', 'le', 'add', 'sub', 'mul') else t
            sizes = math.prod(len(a.support) for a in args)
            if sizes <= 1 << 16:
                support = set()
                for xs in product([a.support for a in args]):
                    support.add(apply(op, st, xs))
            else:
                support = range(size(t))
        if len(support) == 1:
            return Const(t, next(iter(support)))
        v = self.var(t, support)
        out.append(('op', v, op, list(args)))
        return v

    def fold(self, out, op, t, args):
        """OP(args) where a constant operand or equal operands decide it
        without a new variable, else None."""
        if op == 'not':
            return None
        if op == 'ite':
            c, a, b = args
            if isinstance(c, Const):
                return a if c.k else b
            if a is b or a == b:
                return a
            if t == BOOL:
                if a == TRUE and b == FALSE:
                    return c
                if a == FALSE and b == TRUE:
                    return self.op(out, 'not', BOOL, [c])
                if a == FALSE:
                    return self.op(out, 'and', BOOL, [self.op(out, 'not', BOOL, [c]), b])
                if b == FALSE:
                    return self.op(out, 'and', BOOL, [c, a])
                if a == TRUE:
                    return self.op(out, 'or', BOOL, [c, b])
                if b == TRUE:
                    return self.op(out, 'or', BOOL, [self.op(out, 'not', BOOL, [c]), a])
            return None
        if op == 'copy':
            return args[0]
        a, b = args
        if a is b:
            if op in ('and', 'or'):
                return a
            if op in ('xor', 'ne', 'lt'):
                return FALSE
            if op in ('iff', 'eq', 'le'):
                return TRUE
            if op == 'sub':
                return Const(t, 0)
        if op in ('and', 'or', 'xor', 'iff'):
            if isinstance(a, Const):
                a, b = b, a
            if isinstance(b, Const):
                if op == 'and':
                    return a if b.k else FALSE
                if op == 'or':
                    return TRUE if b.k else a
                flip = (op == 'xor') == bool(b.k)
                return self.op(out, 'not', BOOL, [a]) if flip else a
            return None
        if op in ('add', 'sub') and isinstance(b, Const) and b.k == 0:
            return a
        if op == 'add' and isinstance(a, Const) and a.k == 0:
            return b
        if op == 'mul':
            if isinstance(a, Const):
                a, b = b, a
            if isinstance(b, Const):
                if b.k == 0:
                    return Const(t, 0)
                if b.k == 1:
                    return a
        return None

    # --- expressions --------------------------------------------------------

    def value(self, e, env, out):
        """The value of expression e under env (Dice names to values), its
        statements appended to out."""
        if e[0] == 'let':
            env = dict(env)
            while e[0] == 'let':
                env[e[1]] = self.value(e[2], env, out)
                e = e[3]
        kind = e[0]
        if kind == 'var':
            if e[1] not in env:
                raise NameError(f'unbound variable {e[1]}')
            return env[e[1]]
        if kind == 'bool':
            return TRUE if e[1] else FALSE
        if kind == 'flip':
            p = e[1]
            if p <= 0:
                return FALSE
            if p >= 1:
                return TRUE
            v = self.var(BOOL, (0, 1))
            out.append(('sample', v, 'flip', p))
            return v
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            t = ('int', bits_for(len(ps)))
            support = [k for k, p in enumerate(ps) if p > 0]
            if len(support) == 1:
                return Const(t, support[0])
            v = self.var(t, support)
            out.append(('sample', v, 'discrete', ps))
            return v
        if kind == 'uniform':
            width, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** width:
                raise ValueError(f'uniform({width}, {lo}, {hi}): empty or out of range')
            t = ('int', width)
            if hi - lo == 1:
                return Const(t, lo)
            v = self.var(t, range(lo, hi))
            out.append(('sample', v, 'uniform', (lo, hi)))
            return v
        if kind == 'int':
            width, k = e[1:]
            if not 0 <= k < 2 ** width:
                raise ValueError(f'integer constant {k} out of range for {2 ** width} values')
            return Const(('int', width), k)
        if kind == 'tuple':
            return Tup(self.value(e[1], env, out), self.value(e[2], env, out))
        if kind in ('fst', 'snd'):
            v = self.value(e[1], env, out)
            if not isinstance(v, Tup):
                raise TypeError(f'{kind} of {show(v.t)}')
            return v.a if kind == 'fst' else v.b
        if kind == 'not':
            v = self.value(e[1], env, out)
            same(v.t, BOOL, '!')
            return self.op(out, 'not', BOOL, [v])
        if kind == 'shift':
            v = self.value(e[2], env, out)
            if v.t[0] != 'int':
                raise TypeError(f'{e[1]} of {show(v.t)}')
            return self.op(out, 'shl' if e[1] == '<<' else 'shr', v.t, [v, e[3]])
        if kind == 'bin':
            a = self.value(e[2], env, out)
            b = self.value(e[3], env, out)
            return self.binary(out, e[1], a, b)
        if kind == 'observe':
            v = self.value(e[1], env, out)
            same(v.t, BOOL, 'observe')
            if v != TRUE:
                out.append(('observe', v))
            return TRUE
        if kind == 'if':
            return self.ite(e, env, out)
        if kind == 'call':
            params, types, body = self.signature(e[1])
            args = [self.value(x, env, out) for x in e[2]]
            if len(params) != len(args):
                raise TypeError(f'{e[1]}: {len(params)} parameters, {len(args)} arguments')
            for p, a in zip(types, args):
                if p is not None:
                    same(p, a.t, e[1])
            return self.value(body, dict(zip(params, args)), out)
        if kind == 'iterate':
            return self.iterate(e, env, out)
        raise ValueError(f'unknown expression {kind}')

    def binary(self, out, op, a, b):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op in ('&&', '||'):
            same(a.t, BOOL, op)
            same(b.t, BOOL, op)
            return self.op(out, 'and' if op == '&&' else 'or', BOOL, [a, b])
        if a.t == BOOL and op in ('^', '<=>', '==', '!='):
            same(b.t, BOOL, op)
            return self.op(out, 'xor' if op in ('^', '!=') else 'iff', BOOL, [a, b])
        if a.t[0] != 'int':
            raise TypeError(f'{op} on {show(a.t)}')
        same(a.t, b.t, op)
        if op in ('+', '-', '*'):
            return self.op(out, {'+': 'add', '-': 'sub', '*': 'mul'}[op], a.t, [a, b])
        if op in ('>', '>='):
            a, b, op = b, a, {'>': '<', '>=': '<='}[op]
        if op in ('==', '!=', '<', '<='):
            return self.op(out, {'==': 'eq', '!=': 'ne', '<': 'lt', '<=': 'le'}[op], BOOL, [a, b])
        raise ValueError(f'unknown operator {op}')

    def join(self, c, a, b, out, then, other):
        """The value that is a where c holds and b where not: equal
        components are shared, the others an out of the if (or an `ite`
        where neither branch has statements)."""
        same(a.t, b.t, 'if')
        if isinstance(a, Tup):
            return Tup(self.join(c, a.a, b.a, out, then, other), self.join(c, a.b, b.b, out, then, other))
        if a is b or a == b:
            return a
        if then is None:
            return self.op(out, 'ite', a.t, [c, a, b])
        v = self.var(a.t, set(a.support) | set(b.support))
        then.append((v, a, b))
        return v

    def ite(self, e, env, out):
        c = self.value(e[1], env, out)
        same(c.t, BOOL, 'if')
        if isinstance(c, Const):
            return self.value(e[2] if c.k else e[3], env, out)
        then, other = [], []
        a = self.value(e[2], env, then)
        b = self.value(e[3], env, other)
        if not then and not other:
            return self.join(c, a, b, out, None, None)
        outs = []
        v = self.join(c, a, b, out, outs, None)
        out.append(('if', c, then, other, outs))
        return v

    def iterate(self, e, env, out):
        fname, init, k = e[1], self.value(e[2], env, out), e[3]
        params, types, body = self.signature(fname)
        if len(params) != 1:
            raise TypeError(f'iterate: {fname} takes {len(params)} arguments')
        if types[0] is not None:
            same(types[0], init.t, f'iterate({fname})')
        if k == 0:
            return init
        comps = leaves(init)
        # The states' supports: the values the loop reaches, a fixpoint.
        supports = [set(c.support) for c in comps]
        while True:
            states = [self.var(c.t, s) for c, s in zip(comps, supports)]
            state = rebuild(init, iter(states))
            loop = []
            result = self.value(body, {params[0]: state}, loop)
            same(result.t, init.t, f'iterate({fname})')
            nexts = leaves(result)
            grown = [s | set(x.support) for s, x in zip(supports, nexts)]
            if grown == supports:
                break
            supports = grown
        # A next that is another state is copied first: the states update
        # one by one.
        for i, x in enumerate(nexts):
            if isinstance(x, Var) and any(x is s for j, s in enumerate(states) if j != i):
                v = self.var(x.t, x.support)
                loop.append(('op', v, 'copy', [x]))
                nexts[i] = v
        out.append(('loop', k, states, comps, loop, nexts))
        return rebuild(init, iter(states))

    def signature(self, fname):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        return self.funs[fname]

    def program(self, text):
        parser = Parser(tokenize(text))
        funs, body = parser.program()
        names = [f[0] for f in funs]
        for i, (fname, params, ret, fbody) in enumerate(funs):
            for other in names[i:]:
                if calls(fbody, other):
                    raise NotImplementedError(f'{fname} calls {other}: recursion is not supported'
                                              if other == fname else f'{fname} calls {other}, defined after it')
            self.funs[fname] = (params, parser.param_types[fname], fbody)
        out = []
        value = self.value(body, {}, out)
        return out, value


def rebuild(like, it):
    """A value shaped as `like` with the leaves from iterator it."""
    if isinstance(like, Tup):
        a = rebuild(like.a, it)
        return Tup(a, rebuild(like.b, it))
    return next(it)


def product(supports):
    if not supports:
        yield ()
        return
    for x in supports[0]:
        for rest in product(supports[1:]):
            yield (x,) + rest


def lower(text, joint=False):
    """(statements, value) of a Dice program. With joint, the value, a
    tuple of N Booleans (nested pairs, the most significant first), is the
    int(N) whose bits they are: their joint distribution as one component,
    as the Dice programs of an Alea.jl circuit (bench.py's `joint` lines of
    a dicemap) are asked."""
    def run():
        low = Lower()
        stmts, value = low.program(text)
        if joint:
            bits = joint_bits(value)
            t = ('int', len(bits))
            value = Const(t, 0)
            for i, b in enumerate(bits):
                term = low.op(stmts, 'ite', t, [b, Const(t, 1 << (len(bits) - 1 - i)), Const(t, 0)])
                value = low.op(stmts, 'add', t, [value, term])
        return stmts, value
    return deep(run)


def joint_bits(value):
    """The Booleans of a value asked jointly: a tuple of them, the most
    significant first."""
    bits = leaves(value)
    if any(b.t != BOOL for b in bits):
        raise TypeError(f'joint: a value of Booleans, not {show(value.t)}')
    return bits


def prune(stmts, live):
    """The statements that the variables of `live` (a set, needed after
    stmts) and the observations need; `live` becomes the set needed before
    them."""
    kept = []
    for s in reversed(stmts):
        kind = s[0]
        if kind in ('sample', 'op'):
            if s[1] in live:
                live.discard(s[1])
                live.update(args_of(s))
                kept.append(s)
        elif kind == 'observe':
            live.update(args_of(s))
            kept.append(s)
        elif kind == 'if':
            _, c, then, other, outs = s
            outs = [o for o in outs if o[0] in live]
            after = live - {o[0] for o in outs}
            lt = after | {a for _, a, _ in outs if isinstance(a, Var)}
            le = after | {b for _, _, b in outs if isinstance(b, Var)}
            then = prune(then, lt)
            other = prune(other, le)
            if then or other or outs:
                live.clear()
                live.update(lt | le)
                if isinstance(c, Var):
                    live.add(c)
                kept.append(('if', c, then, other, outs))
        elif kind == 'loop':
            _, k, states, inits, body, nexts = s
            need = {i for i, v in enumerate(states) if v in live}
            while True:
                inner = {nexts[i] for i in need if isinstance(nexts[i], Var)}
                pruned = prune(body, inner)
                grown = need | {i for i, v in enumerate(states) if v in inner}
                if grown == need:
                    break
                need = grown
            if pruned or need:
                keep = sorted(need)
                live.difference_update(states)
                live.update(inits[i] for i in keep if isinstance(inits[i], Var))
                kept.append(('loop', k, [states[i] for i in keep], [inits[i] for i in keep], pruned,
                             [nexts[i] for i in keep]))
    kept.reverse()
    return kept


def uses(stmts, count):
    """Adds to count (Var: number) every read of a variable in stmts."""
    for s in stmts:
        for x in args_of(s):
            count[x] = count.get(x, 0) + 1
        if s[0] == 'if':
            uses(s[2], count)
            uses(s[3], count)
        elif s[0] == 'loop':
            uses(s[4], count)
    return count


def decimal(p):
    """A probability as the shortest decimal that reads back as the double,
    without an exponent (Genfer reads none), rounded to 19 places (Genfer
    reads a decimal's digits into a 64-bit integer)."""
    d = Decimal(repr(float(p)))
    if d.as_tuple().exponent < -19:
        d = d.quantize(Decimal(10) ** -19)
    return format(d, 'f')


REFUSED = 3   # bench.py's REFUSED


def refuse(err):
    """Exits with REFUSED: the target language cannot express the program."""
    print(f'cannot express the program: {err}', file=sys.stderr)
    sys.exit(REFUSED)
