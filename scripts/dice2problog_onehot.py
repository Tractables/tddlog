#!/usr/bin/env python3
# tddlog's scripts/dice2problog.py as it was before it held integers in
# bits: integers one-hot. docs/benchmarks.md times tddlog on its
# translations beside the bits ones (variant auto+onehot).
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a propositional ProbLog program.

The translator evaluates the program symbolically: every `let` binds a value
that is a Boolean literal, an integer, or a tuple of values. An integer of
type int(w) is a one-hot family of literals, one per value modulo 2^w,
exactly one true; a family's own atoms are `v(k)`. Every `flip`, `discrete`
and `uniform` becomes a fresh probabilistic fact or annotated disjunction;
Boolean operators and conditionals become rules over fresh atoms. An
integer operation on two families becomes rules over the pairs of values
(`s(v) :- x(a), y(b)` for every a + b = v), and an operation with a
constant relabels the family without new atoms where it is one to one.
A family's atoms are made when something reads them, so `observe c == x +
y` with c constant writes only the pairs that sum to c. An `observe` inside
a conditional branch holds only when the branch is taken, so it becomes an
evidence atom that is true whenever the path condition is false. A chain
`if c0 then e0 else if c1 then e1 ... else en` is merged once: branch i is
guarded by one atom, that c_i holds and no earlier condition did, and
values equal in several branches are merged under the disjunction of their
guards. Within a branch, a variable the guard compares with a constant
takes only the values the guard allows (a constant where it allows one;
under a disjunction, the values either side allows), so a loop over a guarded-command step (pmc1.dice) carries the values its
state reaches rather than every value of its type. Operations on the same
operands are made once. Functions are inlined at each call; `iterate(f, init, k)`
applies f k times by inlining, as Dice does. A function that calls itself is inlined up to the
recursion limit (Dice's default, 10): a call past it is unreachable, an
observation of false on its path, and returns its declared type's default.
The program's return value is queried: every value of an integer, every
component of a tuple.

Supported: let, if-then-else, flip, discrete, uniform, int(w, k), true,
false, &&, ||, !, ^, <=>, ==, !=, <, <=, >, >=, +, -, *, << and >> by a
constant, tuples with fst/snd, observe, fun definitions and calls,
iterate, recursion. Operator precedence is Dice's (lib/Parser.mly).
Not supported: lists, /, %, nth_bit, sample.

Usage: dice2problog.py program.dice > program.pl
"""
import math
import re
import sys
import threading

TOKEN = re.compile(r'\s*(?:(//[^\n]*|\(\*.*?\*\))|([A-Za-z_][A-Za-z0-9_\']*)'
                   r'|([0-9]*\.[0-9]+(?:[eE][-+]?[0-9]+)?|[0-9]+\.?[0-9]*(?:[eE][-+]?[0-9]+)?)'
                   r'|(<=>|==|!=|<=|>=|<<|>>|&&|\|\||::|->|[-+*/%<>^(){}\[\],=!:]))', re.S)

# Dice's binary operators and their precedence, low to high (lib/Parser.mly);
# all associate to the left. `!` sits between && and <=>.
BINARY = {'||': 1, '&&': 2, '<=>': 4, '^': 5, '<': 6, '<=': 6, '>': 6, '>=': 6, '!=': 6,
          '+': 8, '-': 8, '==': 8, '<<': 8, '>>': 8, '*': 9, '/': 9, '%': 9}
NOT_LEVEL = 3
UNSUPPORTED = {'sample', 'head', 'tail', 'length', 'nth_bit'}
RECURSION_LIMIT = 10   # Dice's default (lib/Passes.ml)


def tokenize(text):
    pos, out = 0, []
    while pos < len(text):
        m = TOKEN.match(text, pos)
        if not m or m.end() == pos:
            if text[pos:].strip() == '':
                break
            raise SyntaxError(f'cannot tokenize at {text[pos:pos + 30]!r}')
        pos = m.end()
        if m.group(1):
            continue
        if m.group(2):
            out.append(('id', m.group(2)))
        elif m.group(3):
            out.append(('num', m.group(3)))
        else:
            out.append(('op', m.group(4)))
    return out


class Parser:
    """Recursive descent for Dice's expression language.

    Grammar:
      expr   := 'let' id '=' expr 'in' expr | 'if' expr 'then' expr 'else' expr
              | 'observe' expr | '!' expr | expr binop expr | atom
      atom   := 'flip' num | 'flip' '(' num ')' | 'discrete' '(' nums ')'
              | 'int' '(' num ',' num ')' | 'uniform' '(' num ',' num ',' num ')'
              | 'iterate' '(' id ',' expr ',' num ')'
              | 'fst' atom | 'snd' atom | 'true' | 'false' | id ['(' args ')']
              | '(' expr [',' expr] ')'
    `let`, `if` and `observe` extend as far right as they can, as Dice's
    parser resolves those conflicts by shifting. Dice's `fst e` and `snd e`
    also take the longest e, which a well-typed program never uses (no
    binary operator returns a tuple), so here they take an atom.
    """

    def __init__(self, tokens):
        self.toks = tokens
        self.i = 0

    def peek(self, k=0):
        return self.toks[self.i + k] if self.i + k < len(self.toks) else ('eof', None)

    def take(self, kind=None, value=None):
        t = self.peek()
        if (kind and t[0] != kind) or (value and t[1] != value):
            raise SyntaxError(f'expected {value or kind}, got {t} at token {self.i}')
        self.i += 1
        return t

    def number(self):
        return self.take('num')[1]

    def integer(self):
        s = self.number()
        if not s.isdigit():
            raise SyntaxError(f'expected an integer, got {s} at token {self.i}')
        return int(s)

    def program(self):
        funs = []
        while self.peek() == ('id', 'fun'):
            funs.append(self.fun())
        body = self.expr()
        if self.peek()[0] != 'eof':
            raise SyntaxError(f'trailing tokens from {self.peek()}')
        return funs, body

    def fun(self):
        self.take('id', 'fun')
        name = self.take('id')[1]
        self.take('op', '(')
        params = []
        while self.peek() != ('op', ')'):
            params.append(self.take('id')[1])
            if self.peek() == ('op', ':'):   # type annotation, not checked
                self.take()
                self.type_()
            if self.peek() == ('op', ','):
                self.take()
        self.take('op', ')')
        ret = None
        if self.peek() == ('op', ':'):
            self.take()
            ret = self.type_()
        self.take('op', '{')
        body = self.expr()
        self.take('op', '}')
        return name, params, ret, body

    def type_(self):
        """bool, int(w), (t, t) or list(t), as ('bool',), ('int', w),
        ('tuple', t, t), ('list', t)."""
        t = self.take()
        if t == ('id', 'bool'):
            return ('bool',)
        if t in (('id', 'int'), ('id', 'list')):
            self.take('op', '(')
            inner = self.integer() if t[1] == 'int' else self.type_()
            self.take('op', ')')
            return (t[1], inner)
        if t == ('op', '('):
            a = self.type_()
            self.take('op', ',')
            b = self.type_()
            self.take('op', ')')
            return ('tuple', a, b)
        raise SyntaxError(f'unexpected token {t} in a type at {self.i}')

    def expr(self, level=0):
        e = self.prefix()
        while True:
            t = self.peek()
            prec = BINARY.get(t[1]) if t[0] == 'op' else None
            if prec is None or prec < level:
                return e
            self.take()
            if t[1] in ('<<', '>>'):
                e = ('shift', t[1], e, self.integer())
            else:
                e = ('bin', t[1], e, self.expr(prec + 1))

    def prefix(self):
        t = self.peek()
        if t == ('id', 'let'):
            # A chain of lets is parsed in a loop: programs have thousands.
            binds = []
            while self.peek() == ('id', 'let'):
                self.take()
                name = self.take('id')[1]
                self.take('op', '=')
                value = self.expr()
                self.take('id', 'in')
                binds.append((name, value))
            e = self.expr()
            for name, value in reversed(binds):
                e = ('let', name, value, e)
            return e
        if t == ('id', 'if'):
            self.take()
            c = self.expr()
            self.take('id', 'then')
            a = self.expr()
            self.take('id', 'else')
            b = self.expr()
            return ('if', c, a, b)
        if t == ('id', 'observe'):
            self.take()
            return ('observe', self.expr())
        if t == ('op', '!'):
            self.take()
            return ('not', self.expr(NOT_LEVEL + 1))
        return self.atom()

    def args(self, *kinds):
        self.take('op', '(')
        out = []
        for k, kind in enumerate(kinds):
            if k:
                self.take('op', ',')
            out.append(kind())
        self.take('op', ')')
        return out

    def atom(self):
        t = self.take()
        if t == ('id', 'flip'):
            if self.peek() == ('op', '('):
                return ('flip', float(self.args(self.number)[0]))
            return ('flip', float(self.number()))
        if t == ('id', 'discrete'):
            self.take('op', '(')
            ps = [float(self.number())]
            while self.peek() == ('op', ','):
                self.take()
                ps.append(float(self.number()))
            self.take('op', ')')
            return ('discrete', ps)
        if t == ('id', 'int'):
            return ('int', *self.args(self.integer, self.integer))
        if t == ('id', 'uniform'):
            return ('uniform', *self.args(self.integer, self.integer, self.integer))
        if t == ('id', 'iterate'):
            return ('iterate', *self.args(lambda: self.take('id')[1], self.expr, self.integer))
        if t in (('id', 'fst'), ('id', 'snd')):
            return (t[1], self.atom())
        if t == ('id', 'true'):
            return ('bool', True)
        if t == ('id', 'false'):
            return ('bool', False)
        if t[0] == 'id' and t[1] in UNSUPPORTED:
            raise NotImplementedError(f'{t[1]} is not supported')
        if t[0] == 'id':
            if self.peek() == ('op', '('):
                self.take()
                args = []
                while self.peek() != ('op', ')'):
                    args.append(self.expr())
                    if self.peek() == ('op', ','):
                        self.take()
                self.take('op', ')')
                return ('call', t[1], args)
            return ('var', t[1])
        if t == ('op', '('):
            e = self.expr()
            if self.peek() == ('op', ','):
                self.take()
                f = self.expr()
                self.take('op', ')')
                return ('tuple', e, f)
            self.take('op', ')')
            return e
        if t == ('op', '['):
            raise NotImplementedError('lists are not supported')
        raise SyntaxError(f'unexpected token {t} at {self.i}')


# Values: ('b', literal) with literal 'atom', '\\+atom', 'true' or 'false';
# an Int; ('t', left, right) a tuple.
TRUE, FALSE = ('b', 'true'), ('b', 'false')


class Int:
    """An integer modulo `mod`: lit(v) is the literal that holds exactly when
    the value is v, 'false' for a value outside `support`. `make(v)` gives a
    literal not yet made; it is called once per value, when first read.
    The family's own atoms, where it has them, are `name(v)`; `size` values
    from 0 are queried when the integer is the program's result."""

    def __init__(self, mod, support, make, name=None, size=None):
        self.mod = mod
        self.support = sorted(support)
        self.members = set(support)
        self.make = make
        self.name = name
        self.size = size if size is not None else (self.support[-1] + 1 if self.support else 1)
        self.lits = {}

    def lit(self, v):
        if v not in self.members:
            return 'false'
        if v not in self.lits:
            self.lits[v] = self.make(v)
        return self.lits[v]

    def const(self):
        """The value when the integer is a constant, else None."""
        if len(self.support) == 1 and self.lit(self.support[0]) == 'true':
            return self.support[0]
        return None


def shape(value):
    """'b', 'i' or 't'."""
    return 'i' if isinstance(value, Int) else value[0]


COMPARE = {'==': lambda a, b: a == b, '!=': lambda a, b: a != b, '<': lambda a, b: a < b,
           '<=': lambda a, b: a <= b, '>': lambda a, b: a > b, '>=': lambda a, b: a >= b}
MIRROR = {'==': '==', '!=': '!=', '<': '>', '<=': '>=', '>': '<', '>=': '<='}


def meet(a, b):
    """The facts of two conditions that both hold; None if they conflict."""
    if a is None or b is None:
        return None
    out = dict(a)
    for var, allowed in b.items():
        out[var] = out[var] & allowed if var in out else allowed
        if not out[var]:
            return None
    return out


def join(a, b):
    """The facts of a condition that holds where either of two does: the
    union of what both say of a variable; None if neither can hold."""
    if a is None or b is None:
        return b if a is None else a
    return {var: a[var] | b[var] for var in a.keys() & b.keys()}


def bits_for(n):
    """Dice's int width for n values (num_binary_digits (n - 1))."""
    return max(1, (n - 1).bit_length())


def discrete_probs(ps):
    """The probabilities of `discrete(ps)` as translated: a list whose sum
    is off one by rounding in the source, by at most 1e-4 (the tolerance
    of Dice's type check, Passes.ml), is divided by its sum, as Dice does:
    each bit of its encoding is a flip whose probability is a ratio of
    sums of the list. Translated as written, a sum over one is an error
    in ProbLog, and one under one leaves the variable without a value in
    some worlds."""
    # `math.fsum`, not `sum`: CPython 3.12 sums floats with compensation and
    # older versions do not, so which side of this test a row falls on would
    # depend on the Python that ran it.
    total = math.fsum(ps)
    if total != 1 and abs(total - 1) <= 1e-4:
        return [p / total for p in ps]
    return list(ps)


class Translator:
    def __init__(self):
        self.lines = []
        self.counter = {}
        self.funs = {}
        self.evidence = []
        self.active = {}
        self.memos = {}
        self.taken = set()

    def fresh(self, base):
        """A new atom name from `base`: base, base_1, base_2, ..., the first
        not taken (base_1 may be another base's)."""
        base = re.sub(r'[^a-z0-9_]', '_', base.lower()) or 'v'
        if not base[0].isalpha():
            base = 'v_' + base
        n = self.counter.get(base, 0)
        while (name := base if n == 0 else f'{base}_{n}') in self.taken:
            n += 1
        self.counter[base] = n + 1
        self.taken.add(name)
        return name

    def rule(self, head, body):
        body = [b for b in body if b != 'true']
        if 'false' in body:
            return
        self.lines.append(f'{head} :- {", ".join(body)}.' if body else f'{head}.')

    def define(self, head, bodies):
        """The literal for the disjunction of conjunctions `bodies`: 'true' or
        'false' when they are constant, the one literal of a single one-literal
        body, else `head` defined by one rule per body."""
        live = []
        for body in bodies:
            body = list(dict.fromkeys(b for b in body if b != 'true'))
            if 'false' in body or any(self.neg(b) in body for b in body):
                continue
            if not body:
                return 'true'
            live.append(body)
        if not live:
            return 'false'
        if len(live) == 1 and len(live[0]) == 1:
            return live[0][0]
        for body in live:
            self.rule(head, body)
        return head

    def neg(self, lit):
        if lit == 'true':
            return 'false'
        if lit == 'false':
            return 'true'
        return lit[2:] if lit.startswith('\\+') else '\\+' + lit

    def literal(self, value, name):
        if shape(value) != 'b':
            raise TypeError(f'{name}: Boolean expected')
        return value[1]

    def integer(self, value, name):
        if not isinstance(value, Int):
            raise TypeError(f'{name}: integer expected')
        return value

    def memo(self, key, make, *keep):
        """make() once per key; `keep` holds the objects whose ids are in
        the key, so that no id is reused while the key stands."""
        if key not in self.memos:
            self.memos[key] = (make(), keep)
        return self.memos[key][0]

    def constant(self, k, mod):
        if not 0 <= k < mod:
            raise ValueError(f'integer constant {k} out of range for {mod} values')
        return self.memo(('k', k, mod), lambda: Int(mod, [k], lambda v: 'true'))

    def choice(self, ps, mod, name):
        """An annotated disjunction over the values 0..len(ps)-1."""
        a = self.fresh(name)
        heads = [f'{p!r}::{a}({k})' for k, p in enumerate(ps) if p > 0]
        if len(heads) == 1 and ps[max(range(len(ps)), key=ps.__getitem__)] >= 1:
            k = ps.index(max(ps))
            self.lines.append(f'{a}({k}).')
        else:
            self.lines.append('; '.join(heads) + '.')
        return Int(mod, [k for k, p in enumerate(ps) if p > 0], lambda v: f'{a}({v})', a, len(ps))

    def family(self, mod, support, bodies, name, size=None):
        """The integer whose value v holds when one of the conjunctions
        bodies(v) does; its atoms, `name(v)`, are made when read."""
        a = self.fresh(name)
        return Int(mod, support, lambda v: self.define(f'{a}({v})', bodies(v)), a, size)

    def same_mod(self, x, y, op):
        if x.mod != y.mod:
            raise TypeError(f'{op}: integers of {x.mod} and {y.mod} values')
        return x.mod

    def map_int(self, x, key, f, name):
        """The integer f(x) for f a function of values, named by `key`; a
        one-to-one f relabels x's literals and makes no atoms."""
        def make():
            buckets = {}
            for a in x.support:
                buckets.setdefault(f(a) % x.mod, []).append(a)
            if all(len(b) == 1 for b in buckets.values()):
                back = {v: b[0] for v, b in buckets.items()}
                return Int(x.mod, list(back), lambda v: x.lit(back[v]))
            return self.family(x.mod, list(buckets), lambda v: [[x.lit(a)] for a in buckets[v]], name)
        return self.memo(('map', id(x), key), make, x)

    def int_op(self, op, x, y, name):
        mod = self.same_mod(x, y, op)
        f = {'+': lambda a, b: a + b, '-': lambda a, b: a - b, '*': lambda a, b: a * b}[op]
        kx, ky = x.const(), y.const()
        if ky is not None:
            return self.map_int(x, (op, 'right', ky), lambda a: f(a, ky), name)
        if kx is not None:
            return self.map_int(y, (op, 'left', kx), lambda b: f(kx, b), name)

        def make():
            pairs = {}
            for a in x.support:
                for b in y.support:
                    pairs.setdefault(f(a, b) % mod, []).append((a, b))
            return self.family(mod, list(pairs), lambda v: [[x.lit(a), y.lit(b)] for a, b in pairs[v]], name)
        return self.memo((op, id(x), id(y)), make, x, y)

    def compare(self, op, x, y, name):
        """The Boolean x op y for integers x and y."""
        self.same_mod(x, y, op)
        if op == '!=':
            return ('b', self.neg(self.compare('==', x, y, name)[1]))
        test = COMPARE[op]

        def make():
            kx, ky = x.const(), y.const()
            if ky is not None:
                bodies = [[x.lit(a)] for a in x.support if test(a, ky)]
            elif kx is not None:
                bodies = [[y.lit(b)] for b in y.support if test(kx, b)]
            elif op == '==':
                bodies = [[x.lit(a), y.lit(a)] for a in x.support if a in y.members]
            else:
                bodies = [[x.lit(a), y.lit(b)] for a in x.support for b in y.support if test(a, b)]
            if len(bodies) == 1 and len(bodies[0]) == 1:
                return ('b', bodies[0][0])
            return ('b', self.define(self.fresh(name), bodies))
        return self.memo((op, id(x), id(y)), make, x, y)

    def eval(self, e, env, path, name):
        """Translate expression e under variable environment env and path
        condition path (a list of literals that hold when this expression is
        evaluated); name is the hint for atoms it creates."""
        while e[0] == 'let':
            env = {**env, e[1]: self.eval(e[2], env, path, e[1])}
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
            a = self.fresh(name)
            self.lines.append(f'{p!r}::{a}.')
            return ('b', a)
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            return self.choice(ps, 2 ** bits_for(len(ps)), name)
        if kind == 'uniform':
            width, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** width:
                raise ValueError(f'uniform({width}, {lo}, {hi}): empty or out of range')
            return self.choice([0.0] * lo + [1 / (hi - lo)] * (hi - lo), 2 ** width, name)
        if kind == 'int':
            return self.constant(e[2], 2 ** e[1])
        if kind == 'tuple':
            return ('t', self.eval(e[1], env, path, name + '_1'), self.eval(e[2], env, path, name + '_2'))
        if kind in ('fst', 'snd'):
            v = self.eval(e[1], env, path, name)
            if shape(v) != 't':
                raise TypeError(f'{name}: tuple expected')
            return v[1] if kind == 'fst' else v[2]
        if kind == 'not':
            return ('b', self.neg(self.literal(self.eval(e[1], env, path, name), name)))
        if kind == 'shift':
            x = self.integer(self.eval(e[2], env, path, name), name)
            k = e[3]
            return self.map_int(x, (e[1], k), (lambda a: a << k) if e[1] == '<<' else (lambda a: a >> k), name)
        if kind == 'bin':
            return self.binary(e[1], self.eval(e[2], env, path, name), self.eval(e[3], env, path, name), name)
        if kind == 'observe':
            c = self.literal(self.eval(e[1], env, path, name), name)
            a = self.fresh('obs')
            # Holds when the branch is not taken or the condition is true.
            self.rule(a, path + [c])
            for i, p in enumerate(path):
                self.rule(a, path[:i] + [self.neg(p)])
            if not path and c in ('true', 'false'):
                if c == 'false':
                    self.lines.append('evidence(false_observation, true).')
                return TRUE
            self.evidence.append(a)
            return TRUE
        if kind == 'if':
            # A chain `if c0 then e0 else if c1 then e1 ... else en` is taken
            # as one: branch i is guarded by g_i, that c_i holds and no
            # earlier condition did, an atom shared by everything the
            # branch computes, and the branches' values are merged once.
            # Within a branch, a variable the guard compares with a
            # constant takes only the values the guard allows (a constant
            # where it allows one): what the branch computes is read only
            # under its guard.
            branches, reach, known = [], 'true', {}
            while True:
                at = self.refine(env, known)
                c = self.literal(self.eval(e[1], at, self.within(path, reach), name + '_c'), name)
                guard = self.conj(reach, c, name + '_g')
                inside = self.refine(at, self.facts(e[1], at, True))
                if guard != 'false' and inside is not None:
                    branches.append((guard, self.eval(e[2], inside, self.within(path, guard), name + '_then')))
                reach = self.conj(reach, self.neg(c), name + '_g')
                known = meet(known, self.facts(e[1], at, False))
                e = e[3]
                if reach == 'false' or known is None:
                    break
                if e[0] != 'if':
                    branches.append((reach, self.eval(e, self.refine(env, known), self.within(path, reach),
                                                      name + '_else')))
                    break
            return self.merge(branches, name)
        if kind == 'call':
            params = self.funs[e[1]][0] if e[1] in self.funs else range(len(e[2]))
            args = [self.eval(x, env, path, f'{e[1]}_{p}') for p, x in zip(params, e[2])]
            return self.call(e[1], args, path, name)
        if kind == 'iterate':
            v = self.eval(e[2], env, path, name)
            for _ in range(e[3]):
                v = self.call(e[1], [v], path, name)
            return v
        raise ValueError(f'unknown expression {kind}')

    def binary(self, op, l, r, name):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op in ('&&', '||'):
            l, r = self.literal(l, name), self.literal(r, name)
            if op == '&&':
                return ('b', self.conj(l, r, name))
            if 'true' in (l, r):
                return TRUE
            if l == 'false' or l == r:
                return ('b', r)
            if r == 'false':
                return ('b', l)
            return ('b', self.memo(('or', l, r), lambda: self.define(self.fresh(name), [[l], [r]])))
        if op in ('^', '<=>') or (op in ('==', '!=') and shape(l) == 'b'):
            # Boolean equality and its negation; Dice reads == and != on
            # integers only, the translator on Booleans too.
            lit = self.equal(self.literal(l, name), self.literal(r, name), name)
            return ('b', self.neg(lit) if op in ('^', '!=') else lit)
        if 't' in (shape(l), shape(r)):
            raise TypeError(f'{name}: {op} on a tuple')
        x, y = self.integer(l, name), self.integer(r, name)
        if op in ('+', '-', '*'):
            return self.int_op(op, x, y, name)
        return self.compare(op, x, y, name)

    def equal(self, l, r, name):
        """The literal for l == r on Boolean literals."""
        if l in ('true', 'false'):
            return r if l == 'true' else self.neg(r)
        if r in ('true', 'false'):
            return l if r == 'true' else self.neg(l)
        return self.define(self.fresh(name), [[l, r], [self.neg(l), self.neg(r)]])

    def call(self, fname, args, path, name):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        params, ret, body, recursive = self.funs[fname]
        if len(params) != len(args):
            raise TypeError(f'{fname}: {len(params)} parameters, {len(args)} arguments')
        depth = self.active.get(fname, 0)
        if recursive and depth > RECURSION_LIMIT:
            # Dice's last copy of a recursive function: `observe false` on
            # this path, and the return type's default value.
            self.eval(('observe', ('bool', False)), {}, path, name)
            return self.default(ret, fname)
        self.active[fname] = depth + 1
        try:
            return self.eval(body, dict(zip(params, args)), path, name)
        finally:
            self.active[fname] = depth

    def default(self, t, fname):
        if t is None:
            raise TypeError(f'{fname}: a recursive function needs a return type')
        if t[0] == 'bool':
            return FALSE
        if t[0] == 'int':
            return self.constant(0, 2 ** t[1])
        if t[0] == 'tuple':
            return ('t', self.default(t[1], fname), self.default(t[2], fname))
        raise NotImplementedError(f'{fname}: default value of {t[0]}')

    def facts(self, e, env, positive):
        """What condition e, or its negation when not positive, says of
        variables compared with constants: a map from variable to the values
        it allows (a set of integers; {True} or {False} for a Boolean)."""
        if e[0] == 'not':
            return self.facts(e[1], env, not positive)
        if e[0] == 'var' and e[1] in env and shape(env[e[1]]) == 'b':
            return {e[1]: {positive}}
        if e[0] != 'bin':
            return {}
        op, l, r = e[1:]
        if (op == '&&' and positive) or (op == '||' and not positive):
            return meet(self.facts(l, env, positive), self.facts(r, env, positive))
        if op in ('&&', '||'):
            return join(self.facts(l, env, positive), self.facts(r, env, positive))
        if op in MIRROR and l[0] == 'int' and r[0] == 'var':
            op, l, r = MIRROR[op], r, l
        if op in MIRROR and l[0] == 'var' and r[0] == 'int' and isinstance(env.get(l[1]), Int):
            x, k = env[l[1]], r[2]
            return {l[1]: {a for a in x.support if COMPARE[op](a, k) == positive}}
        return {}

    def refine(self, env, known):
        """env with each variable in `known` restricted to the values it
        allows; None if one allows none of its values."""
        if known is None:
            return None
        env = dict(env)
        for var, allowed in known.items():
            x = env[var]
            if not isinstance(x, Int):
                lits = {True: 'true', False: 'false'}
                left = [b for b in (True, False) if b in allowed and x[1] != lits[not b]]
                if not left:
                    return None
                if len(left) == 1:
                    env[var] = TRUE if left[0] else FALSE
                continue
            support = [a for a in x.support if a in allowed]
            if not support:
                return None
            if len(support) == 1:
                env[var] = self.constant(support[0], x.mod)
            elif len(support) < len(x.support):
                env[var] = self.memo(('restrict', id(x), tuple(support)),
                                     lambda: Int(x.mod, support, x.lit, x.name, x.size), x)
        return env

    def within(self, path, guard):
        return path if guard == 'true' else path + [guard]

    def conj(self, a, b, name):
        if 'false' in (a, b):
            return 'false'
        if a == 'true' or a == b:
            return b
        if b == 'true':
            return a
        return self.memo(('and', a, b), lambda: self.define(self.fresh(name), [[a, b]]))

    def merge(self, branches, name):
        """The value of a conditional whose branch values are `branches`,
        pairs (guard, value) whose guards are exclusive and exhaustive where
        the conditional is evaluated. Equal values are merged first, under
        the disjunction of their guards."""
        value = branches[0][1]
        if any(shape(v) != shape(value) for _, v in branches):
            raise TypeError(f'{name}: branches of different shape')
        if shape(value) == 't':
            return ('t', self.merge([(g, v[1]) for g, v in branches], name + '_1'),
                    self.merge([(g, v[2]) for g, v in branches], name + '_2'))
        groups = {}
        for g, v in branches:
            groups.setdefault(v[1] if shape(v) == 'b' else id(v), (v, []))[1].append(g)
        if len(groups) == 1:
            return value
        # The guards are exclusive and exhaustive, so the value with the
        # most of them is guarded by the negation of the others' where
        # those are fewer.
        groups = sorted(groups.values(), key=lambda vg: -len(vg[1]))
        rest = [g for _, gs in groups[1:] for g in gs]
        first = self.neg(self.disj(rest, name + '_g')) if len(rest) < len(groups[0][1]) else \
            self.disj(groups[0][1], name + '_g')
        parts = [(groups[0][0], first)] + [(v, self.disj(gs, name + '_g')) for v, gs in groups[1:]]
        if shape(value) == 'b':
            return ('b', self.define(self.fresh(name), [[g, v[1]] for v, g in parts]))
        mod = value.mod
        for v, _ in parts:
            self.same_mod(value, v, 'if')

        def make():
            support = sorted(set().union(*(v.members for v, _ in parts)))
            return self.family(mod, support, lambda k: [[g, v.lit(k)] for v, g in parts if k in v.members],
                               name, max(v.size for v, _ in parts))
        return self.memo(('merge', tuple((g, id(v)) for v, g in parts)), make, *(v for v, _ in parts))

    def disj(self, guards, name):
        if len(guards) == 1:
            return guards[0]
        return self.memo(('or', tuple(guards)), lambda: self.define(self.fresh(name), [[g] for g in guards]))

    def queries(self, v, name):
        if isinstance(v, Int):
            q = v.name or self.fresh(name)
            for k in range(v.size):
                lit, atom = v.lit(k), f'{q}({k})'
                if lit != atom:
                    self.rule(atom, [lit])
                self.lines.append(f'query({atom}).')
        elif v[0] == 'b':
            if v[1] in ('true', 'false'):
                name = self.fresh(name)
                self.lines.append(f'{name} :- {v[1]}.' if v[1] == 'true' else f'{name} :- fail.')
                self.lines.append(f'query({name}).')
            else:
                self.lines.append(f'query({v[1]}).')
        else:
            self.queries(v[1], name + '_1')
            self.queries(v[2], name + '_2')

    def translate(self, text):
        funs, body = Parser(tokenize(text)).program()
        for name, params, ret, fbody in funs:
            self.funs[name] = (params, ret, fbody, calls(fbody, name))
        result = self.eval(body, {}, [], 'result')
        for a in self.evidence:
            self.lines.append(f'evidence({a}, true).')
        self.queries(result, 'result')
        return '\n'.join(self.lines) + '\n'


def calls(e, fname):
    """Whether expression e calls function fname."""
    stack = [e]
    while stack:
        e = stack.pop()
        if isinstance(e, tuple):
            if e[0] in ('call', 'iterate') and e[1] == fname:
                return True
            stack.extend(x for x in e[1:] if isinstance(x, (tuple, list)))
        elif isinstance(e, list):
            stack.extend(e)
    return False


def deep(fn, *args):
    """fn(*args) on a thread with a large stack and recursion limit: the
    parser's trees are as deep as a program's chains of lets and ifs."""
    out = []

    def run():
        try:
            out.append(fn(*args))
        except BaseException as err:   # re-raised on the caller's thread
            out.append(err)

    sys.setrecursionlimit(1_000_000)
    threading.stack_size(1 << 29)
    t = threading.Thread(target=run)
    t.start()
    t.join()
    if isinstance(out[0], BaseException):
        raise out[0]
    return out[0]


def translate(text):
    """The translation of a Dice program. Loops unroll and the families'
    atoms are made on first read, which recurses as deep as the loop is
    long: it runs on a thread with a large stack."""
    return deep(lambda: Translator().translate(text))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.stdout.write(translate(open(sys.argv[1]).read()))
