#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into Pluck's language (Bowers, Lew, Tenenbaum, Solar-Lezama and Mansinghka,
"Stochastic Lazy Knowledge Compilation for Inference in Discrete
Probabilistic Programs", PLDI 2025), which compiles a lazy functional
program's random choices into BDDs.

The program is lowered by dice_ir.py. Pluck's values are algebraic data:
a Boolean is `(True)` or `(False)`, and its natural numbers are unary
(`(S (S (O)))`), with no integers of a fixed width. An int(w) is therefore
w Booleans, its bits, as Dice compiles one: `+`, `-` and `*` modulo 2^w are
ripple-carry adders and shift-and-add, `<` and `==` compare the bits, a
shift moves them. `flip p` is `(flip p)`; `discrete` and `uniform` draw the
bits one after the other, each a flip whose probability depends on the bits
drawn before it (2^w - 1 flips, as Pluck's own `discrete` macro expands into
a decision tree of flips). A Boolean operation is `and`, `or`, `not` or
`iff` of Pluck's standard library, or an `if`. An operation read more than
once is a `let` binding, which Pluck evaluates once (call by need); one read
once is written where it is read. A branch's statements are bindings that
only the branch reads (Pluck evaluates a binding where it is first read),
and an observation in a branch holds where the branch is taken.
`iterate(f, x, k)` is a recursive function over Pluck's natural numbers
(through its Y combinator, as the paper's ladder and diamond networks
iterate), its state the tuple of the loop's bits as nested `Pair`s, with
the conjunction of the body's observations in it; the operations of the
body that read nothing of the state are computed once, outside it.

The program is one function, `dice_model`, which returns the conjunction
of the observations and the components of the program's value (counted
from 0, left to right through nested tuples), a Boolean or, for an int,
the list of its bits, the most significant first. A query `cK` asks the
distribution of component K, conditioned on the observations (`Posterior`)
or not (`Marginal`), one query per component in one file: Pluck evaluates
lazily, so a query forces only what its component and the observations
read. `--components` prints each component's type (`bool` or `int W`).
With `--joint` a value of N Booleans is one component, an int N whose most
significant bit is the first Boolean: the joint distribution a circuit's
`joint` program asks (bench.py's dicemap). scripts/pluck_dice.jl runs the
file. A probability is written as a decimal without an exponent, which is
how Pluck reads a number.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions (refused, exit 3).

Usage: dice2pluck.py program.dice [--joint] > program.pluck
       dice2pluck.py program.dice --components [--joint]
"""
import argparse
from decimal import Decimal
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice_ir import BOOL, Const, deep, joint_bits, leaves, lower, prune, refuse  # noqa: E402


def literal(p):
    """A probability as Pluck reads it: digits and a point, no exponent."""
    s = format(Decimal(repr(float(p))), 'f')
    return s if '.' in s else s + '.0'


class Scope:
    """The bindings of a `let`: the model's, or a loop body's."""
    __slots__ = ('parent', 'depth', 'nodes', 'cache')

    def __init__(self, parent):
        self.parent, self.depth = parent, (parent.depth + 1 if parent else 0)
        self.nodes, self.cache = [], {}


class Node:
    """A Boolean of the translated program (or a loop's tuple): op on args,
    in the scope where it is bound or written."""
    __slots__ = ('n', 'op', 'args', 'scope', 'refs', 'cross', 'name')

    def __init__(self, n, op, args, scope):
        self.n, self.op, self.args, self.scope = n, op, args, scope
        self.refs, self.cross, self.name = 0, False, f'v{n}'


def tuple_paths(n):
    """The path (a list of 'fst'/'snd') to each of n leaves of a balanced
    tree of Pairs."""
    if n == 1:
        return [[]]
    h = n // 2
    return [['fst'] + p for p in tuple_paths(h)] + [['snd'] + p for p in tuple_paths(n - h)]


class Translator:
    def __init__(self):
        self.n = 0
        self.scope = Scope(None)
        self.top = self.scope
        self.scopes = [self.scope]

    # --- nodes, folded where an operand decides -------------------------------

    def node(self, op, *args, fresh=False):
        """op(args), hash-consed. A deterministic operation goes to the
        innermost scope of its operands, so that a loop body computes once
        what does not depend on its state."""
        scope = self.scope
        if not fresh:
            inner = [a.scope for a in args if isinstance(a, Node)]
            scope = max(inner, key=lambda s: s.depth) if inner else self.top
            key = (op,) + tuple(a.n if isinstance(a, Node) else ('k', a) for a in args)
            if key in scope.cache:
                return scope.cache[key]
        self.n += 1
        v = Node(self.n, op, args, scope)
        scope.nodes.append(v)
        if not fresh:
            scope.cache[key] = v
        return v

    def flip(self, p):
        if p <= 0:
            return False
        if p >= 1:
            return True
        return self.node('flip', p, fresh=True)

    def NOT(self, x):
        if isinstance(x, bool):
            return not x
        if x.op == 'not':
            return x.args[0]
        return self.node('not', x)

    def AND(self, x, y):
        if x is False or y is False:
            return False
        if x is True:
            return y
        if y is True or x is y:
            return x
        if x.n > y.n:
            x, y = y, x
        return self.node('and', x, y)

    def OR(self, x, y):
        if x is True or y is True:
            return True
        if x is False:
            return y
        if y is False or x is y:
            return x
        if x.n > y.n:
            x, y = y, x
        return self.node('or', x, y)

    def IFF(self, x, y):
        if isinstance(x, bool):
            x, y = y, x
        if isinstance(y, bool):
            return x if y else self.NOT(x)
        if x is y:
            return True
        if x.n > y.n:
            x, y = y, x
        return self.node('iff', x, y)

    def XOR(self, x, y):
        return self.NOT(self.IFF(x, y))

    def ITE(self, c, x, y):
        if isinstance(c, bool):
            return x if c else y
        if x is y:
            return x
        if isinstance(x, bool) and isinstance(y, bool):
            return c if x else self.NOT(c)
        if x is False:
            return self.AND(self.NOT(c), y)
        if y is False:
            return self.AND(c, x)
        if x is True:
            return self.OR(c, y)
        if y is True:
            return self.OR(self.NOT(c), x)
        return self.node('ite', c, x, y)

    def all(self, xs):
        """The conjunction of xs, a balanced tree."""
        xs = [x for x in xs if x is not True]
        if not xs:
            return True
        if len(xs) == 1:
            return xs[0]
        h = len(xs) // 2
        return self.AND(self.all(xs[:h]), self.all(xs[h:]))

    # --- integers as bits, the least significant first -------------------------

    def add(self, a, b, carry):
        out = []
        for x, y in zip(a, b):
            t = self.XOR(x, y)
            out.append(self.XOR(t, carry))
            carry = self.OR(self.AND(x, y), self.AND(carry, t))
        return out

    def less(self, a, b):
        r = False
        for x, y in zip(a, b):
            r = self.ITE(self.IFF(x, y), r, y)
        return r

    def mul(self, a, b):
        w = len(a)
        acc = [False] * w
        for i, y in enumerate(b):
            if y is False:
                continue
            acc = self.add(acc, [False] * i + [self.AND(y, x) for x in a[:w - i]], False)
        return acc

    def categorical(self, ps, w):
        """The w bits of a value drawn with masses ps, the most significant
        first, each a flip given the bits above it."""
        m = [float(p) for p in ps] + [0.0] * (2 ** w - len(ps))
        bits = [None] * w

        def tree(i, j, prefix):
            if i == j:
                base = prefix << (j + 1)
                tot = sum(m[base:base + 2 ** (j + 1)])
                one = sum(m[base + 2 ** j:base + 2 ** (j + 1)])
                if tot <= 0 or one <= 0:
                    return False
                return True if one >= tot else self.flip(one / tot)
            return self.ITE(bits[i], tree(i - 1, j, 2 * prefix + 1), tree(i - 1, j, 2 * prefix))
        for j in reversed(range(w)):
            bits[j] = tree(w - 1, j, 0)
        return bits

    # --- statements ---------------------------------------------------------------

    def val(self, x, env):
        if isinstance(x, Const):
            return [x.k == 1] if x.t == BOOL else [bool(x.k >> i & 1) for i in range(x.t[1])]
        return env[x]

    def sample(self, v, dist, p):
        if dist == 'flip':
            return [self.flip(p)]
        w = v.t[1]
        if dist == 'uniform':
            lo, hi = p
            p = [0.0] * lo + [1.0 / (hi - lo)] * (hi - lo)
        return self.categorical(p, w)

    def op(self, op, args, env):
        if op in ('shl', 'shr'):
            a, k = self.val(args[0], env), args[1]
            w = len(a)
            return [False] * min(k, w) + a[:max(w - k, 0)] if op == 'shl' else a[k:] + [False] * min(k, w)
        xs = [self.val(x, env) for x in args]
        if op == 'copy':
            return xs[0]
        if op == 'not':
            return [self.NOT(xs[0][0])]
        if op == 'ite':
            c = xs[0][0]
            return [self.ITE(c, x, y) for x, y in zip(xs[1], xs[2])]
        a, b = xs
        if op in ('and', 'or', 'xor', 'iff'):
            return [{'and': self.AND, 'or': self.OR, 'xor': self.XOR, 'iff': self.IFF}[op](a[0], b[0])]
        if op in ('eq', 'ne'):
            eq = self.all([self.IFF(x, y) for x, y in zip(a, b)])
            return [eq if op == 'eq' else self.NOT(eq)]
        if op == 'lt':
            return [self.less(a, b)]
        if op == 'le':
            return [self.NOT(self.less(b, a))]
        if op == 'add':
            return self.add(a, b, False)
        if op == 'sub':
            return self.add(a, [self.NOT(y) for y in b], True)
        if op == 'mul':
            return self.mul(a, b)
        raise ValueError(f'unknown operator {op}')

    def stmts(self, ss, env, obs):
        """Translates statements ss, their variables' bits into env and
        their observations into obs."""
        for s in ss:
            kind = s[0]
            if kind == 'sample':
                env[s[1]] = self.sample(s[1], s[2], s[3])
            elif kind == 'op':
                env[s[1]] = self.op(s[2], s[3], env)
            elif kind == 'observe':
                obs.append(self.val(s[1], env)[0])
            elif kind == 'if':
                _, c, then, other, outs = s
                c = self.val(c, env)[0]
                ot, oe = [], []
                self.stmts(then, env, ot)
                self.stmts(other, env, oe)
                for v, a, b in outs:
                    env[v] = [self.ITE(c, x, y) for x, y in zip(self.val(a, env), self.val(b, env))]
                if ot or oe:
                    obs.append(self.ITE(c, self.all(ot), self.all(oe)))
            elif kind == 'loop':
                self.loop(s, env, obs)
            else:
                raise ValueError(f'unknown statement {kind}')

    def loop(self, s, env, obs):
        _, k, states, inits, body, nexts = s
        init = [self.val(x, env) for x in inits]
        observes = has_observe(body)
        width = sum(len(b) for b in init) + (1 if observes else 0)
        if width == 0:
            return
        paths = tuple_paths(width)
        outer = self.scope
        self.scope = inner = Scope(outer)
        self.scopes.append(inner)
        param = self.node('param', fresh=True)
        bits = [self.node('proj', param, tuple(p), fresh=True) for p in paths]
        it = iter(bits)
        for v, b in zip(states, init):
            env[v] = [next(it) for _ in b]
        body_obs = []
        self.stmts(body, env, body_obs)
        result = [x for v in nexts for x in self.val(v, env)]
        if observes:
            result.append(self.AND(bits[-1], self.all(body_obs)))
        self.scope = outer
        start = [x for b in init for x in b] + ([True] if observes else [])
        lp = self.node('loop', k, param, inner, tuple(start), tuple(result), fresh=True)
        out = iter([self.node('proj', lp, tuple(p), fresh=True) for p in paths])
        for v, b in zip(states, init):
            env[v] = [next(out) for _ in b]
        if observes:
            obs.append(next(out))

    # --- text ---------------------------------------------------------------------

    def count(self, roots):
        """Which nodes are bindings: those read more than once or from an
        inner scope (a loop's body or start)."""
        def ref(x, frm, cross=False):
            if isinstance(x, Node):
                x.refs += 1
                if cross or x.scope is not frm:
                    x.cross = True
        for scope in self.scopes:
            for v in scope.nodes:
                if v.op == 'loop':
                    _, param, inner, start, result = v.args
                    for x in start:
                        ref(x, scope, cross=True)
                    for x in result:
                        ref(x, inner)
                elif v.op == 'proj':
                    ref(v.args[0], scope)
                else:
                    for x in v.args:
                        ref(x, scope)
        for x in roots:
            ref(x, self.top)

    def bound(self, v):
        return v.op in ('param', 'loop') or v.refs > 1 or v.cross

    def code(self, x):
        """x where it is read: a binding's name, else its expression."""
        if isinstance(x, bool):
            return '(True)' if x else '(False)'
        if self.bound(x):
            return x.name
        return self.expr(x)

    def expr(self, v):
        op, a = v.op, v.args
        if op == 'flip':
            return f'(flip {literal(a[0])})'
        if op == 'not':
            return f'(not {self.code(a[0])})'
        if op in ('and', 'or', 'iff'):
            return f'({op} {self.code(a[0])} {self.code(a[1])})'
        if op == 'ite':
            return f'(if {self.code(a[0])} {self.code(a[1])} {self.code(a[2])})'
        if op == 'proj':
            src = a[0].name
            for step in a[1]:
                src = f'({step} {src})'
            return src
        if op == 'loop':
            k, param, inner, start, result = a
            rec, n, m = f'rec{v.n}', f'n{v.n}', f'm{v.n}'
            body = self.let(inner, self.tup([self.code(x) for x in result]), [(param.name, f'({rec} {m})')])
            return (f'((Y (lambda {rec} {n} -> (match {n} O => {self.tup([self.code(x) for x in start])} '
                    f'S {m} => {body}))) {k})')
        raise ValueError(f'unknown node {op}')

    def tup(self, codes):
        if len(codes) == 1:
            return codes[0]
        h = len(codes) // 2
        return f'(Pair {self.tup(codes[:h])} {self.tup(codes[h:])})'

    def let(self, scope, body, first=()):
        binds = list(first) + [(v.name, self.expr(v)) for v in scope.nodes
                               if self.bound(v) and v.op != 'param']
        if not binds:
            return body
        lines = '\n'.join(f'    ({name} {e})' for name, e in binds)
        return f'(let (\n{lines})\n  {body})'

    def translate(self, text, joint):
        stmts, value = lower(text)
        comps = [joint_bits(value)] if joint else [[x] for x in leaves(value)]
        live = {x for c in comps for x in c if not isinstance(x, Const)}
        stmts = prune(stmts, live)
        env, obs = {}, []
        self.stmts(stmts, env, obs)
        values, types = [], []
        for c in comps:
            if joint:
                values.append([self.val(x, env)[0] for x in c])
                types.append(('int', len(c)))
            else:
                bits = self.val(c[0], env)
                values.append(bits if c[0].t == BOOL else bits[::-1])
                types.append(c[0].t)
        ev = self.all(obs)
        roots = [ev] + [x for b in values for x in b]
        self.count(roots)
        codes = []
        for t, b in zip(types, values):
            if t == BOOL:
                codes.append(self.code(b[0]))
            else:
                lst = '(Nil)'
                for x in reversed(b):
                    lst = f'(Cons {self.code(x)} {lst})'
                codes.append(lst)
        model = self.let(self.top, f'(Pair {self.code(ev)} {self.tup(codes)})')
        paths = tuple_paths(len(codes))
        queries = []
        for k, p in enumerate(paths):
            comp = 'dice_m'
            for step in ['snd'] + p:
                comp = f'({step} {comp})'
            ask = f'(Posterior {comp} (fst dice_m))' if ev is not True else f'(Marginal {comp})'
            queries.append(f'(query c{k} (let ((dice_m (dice_model))) {ask}))')
        program = f'(define (dice_model)\n  {model})\n\n' + '\n'.join(queries) + '\n'
        return program, types


def has_observe(stmts):
    for s in stmts:
        if s[0] == 'observe':
            return True
        if s[0] == 'if' and (has_observe(s[2]) or has_observe(s[3])):
            return True
        if s[0] == 'loop' and has_observe(s[4]):
            return True
    return False


def translate(text, source='a Dice program', joint=False):
    """(the Pluck program, the types of the components of the value)."""
    program, types = deep(lambda: Translator().translate(text, joint))
    head = (f';; {source}, translated by tddlog\'s scripts/dice2pluck.py: query cK is component K of its '
            f'value ({", ".join("bool" if t == BOOL else f"int({t[1]}), its bits" for t in types)}).\n')
    return head + program, types


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('program')
    ap.add_argument('--components', action='store_true', help="print each component's type instead")
    ap.add_argument('--joint', action='store_true', help='the value, a tuple of Booleans, as one int whose bits '
                    'they are (the most significant first): their joint distribution')
    a = ap.parse_args()
    with open(a.program) as f:
        text = f.read()
    try:
        program, types = translate(text, os.path.basename(a.program), a.joint)
    except NotImplementedError as err:
        refuse(err)
    if a.components:
        for t in types:
            print('bool' if t == BOOL else f'int {t[1]}')
    else:
        sys.stdout.write(program)


if __name__ == '__main__':
    main()
