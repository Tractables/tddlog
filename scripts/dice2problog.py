#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a propositional ProbLog program.

The translator evaluates the program symbolically: every `let` binds a value
that is a Boolean literal, an integer, or a tuple of values. An integer of
type int(w) is w bits, as in Dice, each a literal. Every `flip` becomes a
fresh probabilistic fact; a `discrete` and a `uniform` become flips bit by
bit as Dice draws them (`choice`); Boolean operators and conditionals
become rules over fresh atoms. Integer operations work on the bits: + and
- an adder, * shifted additions, == a conjunction, < a comparator, a
conditional a choice of bits. The values an integer can take are
tracked where their pairs are few, so that a comparison every pair
decides is a constant. An `observe` inside a conditional branch holds
only when the branch is taken, so it becomes an evidence atom that is
true whenever the path condition is false. A chain `if c0 then e0 else
if c1 then e1 ... else en` is merged once: branch i is guarded by one
atom, that c_i holds and no earlier condition did, and values equal in
several branches are merged under the disjunction of their guards.
Within a branch, a variable the guard compares with a constant takes
only the values the guard allows (under a disjunction, the values either
side allows): a constant where it allows one, and the bits on which the
values agree are constants, so a loop over a guarded-command step
(pmc1.dice) carries the values its state reaches rather than every value
of its type. Operations on the same operands are made once. Functions are inlined at each call. A loop
`iterate(f, init, k)` is written as a ProbLog user writes one, recursion
over a step index that the grounder unrolls: f's body once, its atoms
indexed by the step T (`step(T)`, 0 to k - 1), each flip a probabilistic
fact of its step, and the state before step T the atoms s(T) of the
state's Booleans and bits, s(0) the initial state's and s(T + 1) the
body's at step T; the body's observations hold at every step, `failed`
where one does not and the evidence that it is false (see `iterate`). A function that calls itself is inlined up to the
recursion limit (Dice's default, 10): a call past it is unreachable, an
observation of false on its path, and returns its declared type's default.
The program's return value is queried: every value of an integer (the
conjunction of its bits), every component of a tuple.

Supported: let, if-then-else, flip, discrete, uniform, int(w, k), true,
false, &&, ||, !, ^, <=>, ==, !=, <, <=, >, >=, +, -, *, << and >> by a
constant, tuples with fst/snd, observe, fun definitions and calls,
iterate, recursion. Operator precedence is Dice's (lib/Parser.mly).
Not supported: lists, /, %, nth_bit, sample.

Usage: dice2problog.py program.dice > program.pl

The harness's copy: tddlog reads Dice programs itself since this script
left the source tree (`tddlog --program program.dice` prints the same
program, byte for byte), and the benchmark harness writes the ProbLog
program of a Dice program with it. The translators of Dice programs for
the other systems (dice_ir.py, dice2alea.py, dice2prism.py, dice2psi.py,
dice2roulette.py) import this script's parser from here, as does the
exact evaluation of tests/test_dice2problog.py.
"""
import math
import re
import sys
import threading
from fractions import Fraction

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
        # A function's parameter types, by name (None where not declared).
        self.param_types = {}

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
        params, types = [], []
        while self.peek() != ('op', ')'):
            params.append(self.take('id')[1])
            types.append(None)
            if self.peek() == ('op', ':'):   # type annotation, not checked
                self.take()
                types[-1] = self.type_()
            if self.peek() == ('op', ','):
                self.take()
        self.take('op', ')')
        self.param_types[name] = types
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
# an Int; ('t', left, right) a tuple. In a loop's body an atom of the step
# is `name(T)` (STEP), and the state after step k `name(k)`.
TRUE, FALSE = ('b', 'true'), ('b', 'false')
STEP = '(T)'


def at_step(lit):
    """Whether a literal is of the step of a loop's body."""
    return lit.endswith(STEP)


class Int:
    """An integer of `width` bits, modulo 2^width: bits[k] is the literal of
    its bit k, least significant first ('true' or 'false' where it is
    constant). `support` holds every value it can take (it may hold more),
    or is None where that set was not worked out. Its query atoms are
    `name(v)`, where it has a name, else a fresh atom from `hint`; `size`
    values from 0 are queried when it is the program's result."""

    def __init__(self, width, bits, support, name=None, hint=None, size=None):
        self.width = width
        self.mod = 1 << width
        self.bits = list(bits)
        self.support = None if support is None else sorted(support)
        self.members = None if support is None else set(self.support)
        self.name = name
        self.hint = hint
        self.size = size if size is not None else self.support[-1] + 1 if self.support else self.mod

    def allows(self, v):
        return 0 <= v < self.mod and (self.members is None or v in self.members)

    def const(self):
        """The value when the integer is a constant, else None."""
        if all(b in ('true', 'false') for b in self.bits):
            return sum(1 << k for k, b in enumerate(self.bits) if b == 'true')
        return None


# The most pairs of values whose results an integer operation lists, to
# know the values the result can take.
SUPPORT_PAIRS = 1 << 16


def shape(value):
    """'b', 'i' or 't'."""
    return 'i' if isinstance(value, Int) else value[0]


def flat(value):
    """A value's Booleans and integers, depth first."""
    return flat(value[1]) + flat(value[2]) if shape(value) == 't' else [value]


def flat_names(value, base):
    """A name for each of flat(value): base, and base_1 and base_2 for a
    tuple's components."""
    return flat_names(value[1], base + '_1') + flat_names(value[2], base + '_2') if shape(value) == 't' else [base]


def rebuild(value, parts):
    """A value of value's shape from the list flat(value) would give."""
    parts = iter(parts)

    def build(v):
        return ('t', build(v[1]), build(v[2])) if shape(v) == 't' else next(parts)
    return build(value)


def bits_of(part):
    """The literals of a Boolean or an integer."""
    return part.bits if isinstance(part, Int) else [part[1]]


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
        # In a loop's body: the step predicate, the body's observations and
        # its flips; the loops' step literals.
        self.loop = None
        self.observed = []
        self.flips = set()
        self.steps = set()

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
        # A loop's rule reads the step's atoms first and their negations
        # after them, as a ProbLog programmer writes it: the grounder then
        # binds the step from the atoms the round before added. A rule to
        # the next step (its head at T1) reads `step(T), T1 is T + 1` first,
        # so that a top-down grounder (ProbLog's) calls the step's atoms
        # with T bound; an atom called at an unbound step can reach its own
        # negation, which ProbLog stops at as a negative cycle.
        if not head.endswith('(T1)'):
            first = [b for b in body if at_step(b) and not b.startswith('\\+') and b not in self.steps]
            body = first + [b for b in body if b not in first]
        if at_step(head) and not any(at_step(b) and not b.startswith('\\+') for b in body):
            body = [self.loop + STEP] + body      # the step binds T
        self.lines.append(f'{head} :- {", ".join(body)}.' if body else f'{head}.')

    def live(self, bodies):
        """The conjunctions `bodies` without constants and repeats: 'true'
        or 'false' where their disjunction is constant, the one literal of a
        single one-literal body, else the list of bodies left."""
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
        return live

    def define(self, head, bodies):
        """The literal for the disjunction of conjunctions `bodies`: a
        constant or one literal where `live` folds them to it, else `head`
        defined by one rule per body."""
        live = self.live(bodies)
        if isinstance(live, str):
            return live
        if self.loop and any(at_step(b) for body in live for b in body):
            head += STEP
        for body in live:
            self.rule(head, body)
        return head

    def gate(self, name, bodies):
        """define() under a fresh atom from `name`, made only where the
        bodies do not fold; the same bodies give the same atom."""
        live = self.live(bodies)
        if isinstance(live, str):
            return live
        return self.memo(('gate', tuple(map(tuple, live))), lambda: self.define(self.fresh(name), live))

    def xor(self, a, b, name):
        a, b = sorted((a, b))
        return self.gate(name, [[a, self.neg(b)], [self.neg(a), b]])

    def flip(self, p, name):
        a = self.fresh(name)
        if self.loop:
            self.lines.append(f'{p!r}::{a}{STEP} :- {self.loop}{STEP}.')
            self.flips.add(a + STEP)
            return a + STEP
        self.lines.append(f'{p!r}::{a}.')
        return a

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
        width = mod.bit_length() - 1
        return self.memo(('k', k, mod),
                         lambda: Int(width, ['true' if k >> i & 1 else 'false' for i in range(width)], [k]))

    def choice(self, ps, mod, name):
        """A categorical over the values 0..len(ps)-1 in bits, as Dice
        encodes it (gen_discrete in lib/Passes.ml): from the most
        significant bit down, bit k is a flip whose probability is the
        mass of the values with bit k set among those that agree on the
        bits above it, over the mass of those values. One flip serves
        every setting of the bits above where that ratio is the same (a
        uniform over a power of two takes one flip a bit), a ratio of 0 or
        1 is a constant, and the settings that serve a flip are read as a
        decision on the bits above, settings of no mass left out."""
        a = self.fresh(name)
        width = mod.bit_length() - 1
        masses = [Fraction(p) for p in ps] + [Fraction(0)] * (mod - len(ps))
        bits = ['false'] * width
        for k in reversed(range(width)):
            ratio = {}                  # a setting u of the bits above k: the ratio
            for u in range(mod >> (k + 1)):
                block = masses[u << (k + 1):(u + 1) << (k + 1)]
                total = sum(block)
                if total:
                    ratio[u] = float(sum(block[1 << k:]) / total)
            flips = {}
            for r in ratio.values():
                if r not in flips:
                    flips[r] = 'false' if r == 0 else 'true' if r == 1 else self.flip(r, f'{a}_f{k}')

            def paths(lo, size, r):
                """The conjunctions of literals of the bits above k that
                hold for the settings in [lo, lo + size) of ratio r."""
                mine = [u for u in range(lo, lo + size) if ratio.get(u) == r]
                if not mine:
                    return []
                if len(mine) == sum(u in ratio for u in range(lo, lo + size)):
                    return [[]]
                half = size // 2
                j = k + half.bit_length()       # the bit that splits the range
                return [[self.neg(bits[j])] + t for t in paths(lo, half, r)] + \
                    [[bits[j]] + t for t in paths(lo + half, half, r)]
            bits[k] = self.gate(f'{a}_b{k}', [t + [f] for r, f in flips.items() for t in paths(0, mod >> (k + 1), r)])
        return Int(width, bits, [v for v, p in enumerate(ps) if p > 0], a, size=len(ps))

    def same_mod(self, x, y, op):
        if x.mod != y.mod:
            raise TypeError(f'{op}: integers of {x.mod} and {y.mod} values')
        return x.mod

    def results(self, x, y, f):
        """The values f takes on the values of x and y, where both are known
        and their pairs not too many; else None."""
        if x.support is None or y.support is None or len(x.support) * len(y.support) > SUPPORT_PAIRS:
            return None
        return {f(a, b) for a in x.support for b in y.support}

    def add_bits(self, xs, ys, carry, name):
        """The bits of xs + ys + carry modulo 2^len(xs), the carry rippling
        from bit 0."""
        out = []
        for k, (x, y) in enumerate(zip(xs, ys)):
            h = self.xor(x, y, name)
            out.append(self.xor(h, carry, name))
            if k + 1 < len(xs):
                carry = self.gate(name, [[x, y], [carry, h]])
        return out

    def int_op(self, op, x, y, name):
        """x op y for + - and *, modulo 2^width: an adder; x - y as x + ~y +
        1; x * y as the sum of x shifted by each bit of y that is set. Its
        atoms are NAME_b...; NAME is left to its query atoms."""
        mod = self.same_mod(x, y, op)
        hint, name = name, name + '_b'
        f = {'+': lambda a, b: a + b, '-': lambda a, b: a - b, '*': lambda a, b: a * b}[op]

        def make():
            support = self.results(x, y, lambda a, b: f(a, b) % mod)
            if support is not None and len(support) == 1:
                return self.constant(support.pop(), mod)
            if op == '+':
                bits = self.add_bits(x.bits, y.bits, 'false', name)
            elif op == '-':
                bits = self.add_bits(x.bits, [self.neg(b) for b in y.bits], 'true', name)
            else:
                bits = ['false'] * x.width
                for i, yi in enumerate(y.bits):
                    if yi != 'false':
                        part = ['false'] * i + [self.conj(yi, xj, name) for xj in x.bits[:x.width - i]]
                        bits = self.add_bits(bits, part, 'false', name)
            return Int(x.width, bits, support, hint=hint)
        return self.memo((op, id(x), id(y)), make, x, y)

    def shift(self, op, x, k, name):
        def make():
            if op == '<<':
                bits = (['false'] * k + x.bits)[:x.width]
                support = None if x.support is None else {(a << k) % x.mod for a in x.support}
            else:
                bits = (x.bits[k:] + ['false'] * k)[:x.width]
                support = None if x.support is None else {a >> k for a in x.support}
            return Int(x.width, bits, support, hint=name)
        return self.memo((op, id(x), k), make, x)

    def compare(self, op, x, y, name):
        """The Boolean x op y for integers x and y: == bit by bit, < by a
        comparator from bit 0 up (x < y where its bit is below y's, or equal
        and x < y below it); a constant where every pair of their values
        gives the same."""
        self.same_mod(x, y, op)
        if op == '!=':
            return ('b', self.neg(self.compare('==', x, y, name)[1]))
        if op in ('>', '>='):
            return self.compare(MIRROR[op], y, x, name)
        if op == '<=':
            return ('b', self.neg(self.compare('<', y, x, name)[1]))

        def make():
            outcomes = self.results(x, y, COMPARE[op])
            if outcomes is not None and len(outcomes) == 1:
                return TRUE if outcomes.pop() else FALSE
            same = [self.neg(self.xor(a, b, name + '_b')) for a, b in zip(x.bits, y.bits)]
            if op == '==':
                return ('b', self.gate(name, [same]))
            lt = 'false'
            for a, b, eq in zip(x.bits, y.bits, same):
                lt = self.gate(name, [[self.neg(a), b], [eq, lt]])
            return ('b', lt)
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
            return ('b', self.flip(p, name))
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
            return self.shift(e[1], self.integer(self.eval(e[2], env, path, name), name), e[3], name)
        if kind == 'bin':
            return self.binary(e[1], self.eval(e[2], env, path, name), self.eval(e[3], env, path, name), name)
        if kind == 'observe':
            c = self.literal(self.eval(e[1], env, path, name), name)
            # Holds when the branch is not taken or the condition is true;
            # in a loop's body, at each step.
            a = self.fresh('obs') + (STEP if self.loop else '')
            self.rule(a, path + [c])
            for i, p in enumerate(path):
                self.rule(a, path[:i] + [self.neg(p)])
            if not path and c in ('true', 'false'):
                if c == 'false':
                    self.lines.append('evidence(false_observation, true).')
                return TRUE
            (self.observed if self.loop else self.evidence).append(a)
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
            return self.iterate(e[1], self.eval(e[2], env, path, name), e[3], path, name)
        raise ValueError(f'unknown expression {kind}')

    def iterate(self, fname, init, k, path, name):
        """`iterate(fname, init, k)` as recursion over a step index, which
        the grounder unrolls: fname's body once, under `step(T)` for T from
        0 to k - 1, its flips probabilistic facts of the step and its atoms
        of the step where they read one (`name(T)`). Each literal of the
        state (a Boolean, a bit of an integer) is an atom s of its own,
        s(0) the initial state's literal and s(T1), T1 = T + 1, the body's
        at step T, and the value is the state at k; a literal the step
        keeps (the body returns the initial state's literal where the
        state holds it, as the key of caesar-iter) is no atom but itself.
        Which literals are kept, and the values the state's integers take
        (for the comparisons and the conditions that they decide), are a
        fixpoint over the step: the initial state's, and what the body
        makes of them, until no evaluation of the body adds to them (the
        evaluations before the last are undone). The step's observations
        hold at every step: `failed` holds where one does not at some
        step, and the evidence is that it is false."""
        if k == 0:
            return init
        if self.loop:
            raise NotImplementedError('iterate inside a loop')
        start = flat(init)
        kept = [[True] * len(bits_of(p)) for p in start]
        values = [p.support if isinstance(p, Int) else None for p in start]
        state = [[None] * len(bits_of(p)) for p in start]
        bases = flat_names(init, fname)
        hints = list(bases)
        loop = None

        def at(i, index):
            """Part i of the state at `index` (STEP, or the last step)."""
            p = start[i]
            if all(kept[i]):
                return p
            ls = [lit if kept[i][j] else f'{state[i][j]}{index}' for j, lit in enumerate(bits_of(p))]
            return Int(p.width, ls, values[i], hint=hints[i]) if isinstance(p, Int) else ('b', ls[0])

        while True:
            for i, p in enumerate(start):
                for j in range(len(state[i])):
                    if not kept[i][j] and state[i][j] is None:
                        state[i][j] = self.fresh(bases[i] if shape(p) == 'b' else f'{bases[i]}_b{j}')
            loop = loop or self.fresh('step')
            self.steps.add(loop + STEP)
            mark = self.mark()
            ends = self.body(fname, rebuild(init, [at(i, STEP) for i in range(len(start))]), path, name, loop)
            self.undo(mark)
            grew = False
            for i, (p, q) in enumerate(zip(start, ends)):
                for j, (a, b) in enumerate(zip(bits_of(p), bits_of(q))):
                    if kept[i][j] and a != b:
                        kept[i][j], grew = False, True
                if isinstance(p, Int) and values[i] is not None:
                    more = None if q.support is None else sorted(set(values[i]) | set(q.support))
                    if more != values[i]:
                        values[i], grew = more, True
            if not grew:
                break
        for i, p in enumerate(start):
            for j, lit in enumerate(bits_of(p)):
                if not kept[i][j]:
                    self.rule(f'{state[i][j]}(0)', [lit])
        self.lines.append(f'{loop}(0).')
        if k > 1:
            self.lines.append(f'{loop}(T1) :- {loop}(T), T < {k - 1}, T1 is T + 1.')
        ends = self.body(fname, rebuild(init, [at(i, STEP) for i in range(len(start))]), path, name, loop)
        after = [f'{loop}(T)', 'T1 is T + 1']
        for i, q in enumerate(ends):
            if isinstance(q, Int):
                hints[i] = q.name or q.hint or hints[i]
            for j, lit in enumerate(bits_of(q)):
                if not kept[i][j]:
                    self.rule(f'{state[i][j]}(T1)', after + [lit])
        if self.observed:
            failed = self.fresh('failed')
            for o in self.observed:
                self.rule(failed, [f'{loop}(T)', self.neg(o)])
            self.evidence.append(self.neg(failed))
        return rebuild(init, [at(i, f'({k})') for i in range(len(start))])

    def body(self, fname, state, path, name, loop):
        """The parts of fname(state) as a loop's step: its atoms of the
        step, its observations in `observed`."""
        self.loop, self.observed = loop, []
        try:
            out = self.call(fname, [state], path, name)
        finally:
            self.loop = None
        ends = flat(out)
        if len(ends) != len(flat(state)) or any(shape(p) != shape(q) or (isinstance(p, Int) and p.mod != q.mod)
                                                for p, q in zip(flat(state), ends)):
            raise TypeError(f'{fname}: the loop step returns a value of another type than its state')
        return ends

    def mark(self):
        return (len(self.lines), len(self.evidence), dict(self.counter), set(self.taken), dict(self.memos),
                set(self.flips))

    def undo(self, mark):
        """Back to what the translation was at `mark`."""
        lines, evidence, self.counter, self.taken, self.memos, self.flips = mark
        del self.lines[lines:]
        del self.evidence[evidence:]

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
        if op in MIRROR and l[0] == 'var' and r[0] == 'int' and isinstance(env.get(l[1]), Int) and \
                env[l[1]].support is not None:
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
            if x.support is None:
                continue
            support = [a for a in x.support if a in allowed]
            if not support:
                return None
            if len(support) == 1:
                env[var] = self.constant(support[0], x.mod)
            elif len(support) < len(x.support):
                # The bits on which the values left agree are constants.
                bits = [b if len({a >> k & 1 for a in support}) > 1 else 'true' if support[0] >> k & 1 else 'false'
                        for k, b in enumerate(x.bits)]
                env[var] = self.memo(('restrict', id(x), tuple(support)),
                                     lambda: Int(x.width, bits, support, x.name, x.hint, x.size), x)
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
        for v, _ in parts:
            self.same_mod(value, v, 'if')

        def make():
            support = None if any(v.support is None for v, _ in parts) else \
                set().union(*(v.members for v, _ in parts))
            bits = []
            for k in range(value.width):
                lits = {v.bits[k] for v, _ in parts}
                bits.append(lits.pop() if len(lits) == 1 else
                            self.gate(name + '_b', [[g, v.bits[k]] for v, g in parts]))
            return Int(value.width, bits, support, hint=name, size=max(v.size for v, _ in parts))
        return self.memo(('merge', tuple((g, id(v)) for v, g in parts)), make, *(v for v, _ in parts))

    def disj(self, guards, name):
        if len(guards) == 1:
            return guards[0]
        return self.memo(('or', tuple(guards)), lambda: self.define(self.fresh(name), [[g] for g in guards]))

    def queries(self, v, name):
        if isinstance(v, Int):
            # Each value's atom: the conjunction of its bits' literals.
            q = v.name or self.fresh(v.hint or name)
            for k in range(v.size):
                atom = f'{q}({k})'
                if v.allows(k):
                    self.rule(atom, [b if k >> i & 1 else self.neg(b) for i, b in enumerate(v.bits)])
                self.lines.append(f'query({atom}).')
        elif v[0] == 'b':
            if v[1] in ('true', 'false') or v[1].endswith(')'):
                # A constant, or an atom of a loop's last step: an atom of
                # its own, as a Boolean's query has no argument.
                name = self.fresh(name)
                self.lines.append(f'{name} :- {"fail" if v[1] == "false" else v[1]}.')
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
            self.lines.append(f'evidence({self.neg(a)}, false).' if a.startswith('\\+') else f'evidence({a}, true).')
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
    """The translation of a Dice program. Its evaluation recurses as deep
    as its chains of lets and ifs: it runs on a thread with a large
    stack."""
    return deep(lambda: Translator().translate(text))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.stdout.write(translate(open(sys.argv[1]).read()))
