#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into Pluck's language (Bowers, Lew, Tenenbaum, Solar-Lezama and Mansinghka,
"Stochastic Lazy Knowledge Compilation for Inference in Discrete
Probabilistic Programs", PLDI 2025), which compiles a lazy functional
program's random choices into BDDs.

The program is lowered by dice_ir.py. Pluck's values are algebraic data,
and each variable is written as a Pluck program writes a value of its kind:

- a Boolean is `(True)` or `(False)`, and so is a variable of two values
  (a network's variable of two states), true for the larger value;
- a variable of k values is a constructor of the type `value`, `(V0)` to
  `(Vk-1)` named after the value, which a `match` reads, as Pluck's own
  programs define and read their categories;
- an operation on such variables is a case split: `x + y` is a `match` on
  x whose cases are `match`es on y, each giving the sum's constructor, and
  an `if` on a test of a variable against constants is a `match` on that
  variable, so that a network's table is one `match` on each parent;
- a number with too many values to split on (an operation of more than
  CASES cases, decided once for all the variables that arithmetic, `if`
  and loops connect) is a binary integer, its bits Booleans, as Pluck's
  standard library writes its fixed-width integers (`int1` to `int4`) and
  Dice compiles one: `+`, `-` and `*` modulo 2^w are ripple-carry adders
  and shift-and-add, `<` and `==` compare the bits, a shift moves them.

`flip p` is `(flip p)`. A `discrete` or `uniform` draw is the decision tree
of flips that Pluck's `discrete` macro expands it into, each flip given the
values above it, written out because the macro rounds a flip's probability
to ten decimals. A Boolean operation is `and`, `or`, `not` or `iff` of
Pluck's standard library, or an `if`. An operation read more than once, or
read by a `match`, is a `let` binding, which Pluck evaluates once (call by
need); another is written where it is read. A branch's statements are bindings that only the
branch reads (Pluck evaluates a binding where it is first read), and an
observation in a branch holds where the branch is taken.
`iterate(f, x, k)` is a recursive function over Pluck's natural numbers
(through its Y combinator, as the paper's ladder and diamond networks
iterate), its state the tuple of the loop's variables as nested `Pair`s,
with the conjunction of the body's observations in it; the operations of
the body that read nothing of the state are computed once, outside it.

The program is one function, `dice_model`, which returns the conjunction
of the observations and the components of the program's value (counted
from 0, left to right through nested tuples): a Boolean, an int's
constructor or, for a binary int, the list of its bits, the most
significant first. A query `cK` asks the distribution of component K,
conditioned on the observations (`Posterior`) or not (`Marginal`), one
query per component in one file: Pluck evaluates lazily, so a query forces
only what its component and the observations read. `--components` prints
each component's type (`bool` or `int W`). With `--joint` a value of N
Booleans is one component, an int N whose most significant bit is the
first Boolean: the joint distribution a circuit's `joint` program asks
(bench.py's dicemap). scripts/pluck_dice.jl runs the file. A probability
is written as a decimal without an exponent, which is how Pluck reads a
number.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions (refused, exit 3).

Usage: dice2pluck.py program.dice [--joint] > program.pluck
       dice2pluck.py program.dice --components [--joint]
"""
import argparse
from decimal import Decimal
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice2problog import CHAIN  # noqa: E402
from dice_ir import BOOL, Const, apply, deep, joint_bits, leaves, lower, prune, refuse  # noqa: E402

# The most cases a case split may have: past it, the variables of the
# operation are binary integers.
CASES = 1024


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
    """A value of the translated program (a Boolean, a constructor or a
    loop's tuple): op on args, in the scope where it is bound or written."""
    __slots__ = ('n', 'op', 'args', 'scope', 'refs', 'cross', 'chained', 'name')

    def __init__(self, n, op, args, scope):
        self.n, self.op, self.args, self.scope = n, op, args, scope
        self.refs, self.cross, self.chained, self.name = 0, False, False, f'v{n}'


class Lit:
    """The constructor `(Vk)` of the value k (one object for each k)."""
    __slots__ = ('k',)

    def __init__(self, k):
        self.k = k


class Num:
    """An int(w) of the translated program, which takes the values
    `support`, in one of the forms of `Translator.shape`: 'k' the value x,
    'b' a Boolean x, true for the larger of its two values, 'c' x a
    constructor (a Lit or a node), 'bits' x the list of its w bits, the
    least significant first."""
    __slots__ = ('w', 'support', 'form', 'x')

    def __init__(self, w, support, form, x):
        self.w, self.support, self.form, self.x = w, tuple(support), form, x


def tuple_paths(n):
    """The path (a list of 'fst'/'snd') to each of n leaves of a balanced
    tree of Pairs."""
    if n == 1:
        return [[]]
    h = n // 2
    return [['fst'] + p for p in tuple_paths(h)] + [['snd'] + p for p in tuple_paths(n - h)]


def categoricals(stmts):
    """For each int variable, whether it is split on (else binary): the
    variables that arithmetic, `ite`, `if` and loops connect are one class,
    split on where none of its operations has more than CASES cases."""
    parent, worst = {}, {}

    def find(v):
        root = v
        while parent.get(root, root) is not root:
            root = parent[root]
        while v is not root:
            parent[v], v = root, parent[v]
        return root

    def join(xs, cases):
        xs = [x for x in xs if not isinstance(x, Const) and x.t != BOOL]
        if not xs:
            return
        roots = {find(x) for x in xs}
        top = roots.pop()
        for r in roots:
            parent[r] = top
            worst[top] = max(worst.get(top, 0), worst.pop(r, 0))
        worst[top] = max([worst.get(top, 0), cases] + [len(x.support) for x in xs])

    def walk(ss):
        for s in ss:
            kind = s[0]
            if kind == 'op':
                v, op, args = s[1], s[2], s[3]
                xs = args[:1] if op in ('shl', 'shr') else args[1:] if op == 'ite' else args
                join([v] + list(xs), 0 if op in ('ite', 'copy') else math.prod(len(x.support) for x in xs))
            elif kind == 'sample':
                join([s[1]], 0)
            elif kind == 'if':
                walk(s[2])
                walk(s[3])
                for v, a, b in s[4]:
                    join([v, a, b], 0)
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                walk(body)
                for v, a, b in zip(states, inits, nexts):
                    join([v, a, b], 0)
    walk(stmts)
    return lambda v: worst.get(find(v), len(v.support)) <= CASES


class Translator:
    def __init__(self):
        self.n = 0
        self.scope = Scope(None)
        self.top = self.scope
        self.scopes = [self.scope]
        self.lits = {}
        self.split_on = None

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

    def lit(self, k):
        if k not in self.lits:
            self.lits[k] = Lit(k)
        return self.lits[k]

    def flip(self, p):
        if p <= 0:
            return False
        if p >= 1:
            return True
        return self.node('flip', p, fresh=True)

    @staticmethod
    def tests(x):
        """Whether x is a case split into Booleans (a test of a variable
        against constants)."""
        return isinstance(x, Node) and x.op == 'case' and all(isinstance(r, bool) for r in x.args[2:])

    def merge(self, x, y, f):
        """f of two tests of one variable, a test of it; else None."""
        if self.tests(x) and self.tests(y) and x.args[0] is y.args[0] and x.args[1] == y.args[1]:
            return self.case(x.args[0], x.args[1], [f(a, b) for a, b in zip(x.args[2:], y.args[2:])])
        return None

    def NOT(self, x):
        if isinstance(x, bool):
            return not x
        if x.op == 'not':
            return x.args[0]
        if self.tests(x):
            return self.case(x.args[0], x.args[1], [not r for r in x.args[2:]])
        return self.node('not', x)

    def AND(self, x, y):
        if x is False or y is False:
            return False
        if x is True:
            return y
        if y is True or x is y:
            return x
        m = self.merge(x, y, lambda a, b: a and b)
        if m is not None:
            return m
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
        m = self.merge(x, y, lambda a, b: a or b)
        if m is not None:
            return m
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
        m = self.merge(x, y, lambda a, b: a == b)
        if m is not None:
            return m
        if x.n > y.n:
            x, y = y, x
        return self.node('iff', x, y)

    def XOR(self, x, y):
        return self.NOT(self.IFF(x, y))

    def ITE(self, c, x, y):
        """`(if c x y)`, x and y Booleans or constructors; on a test of a
        variable, a `match` on the variable."""
        if isinstance(c, bool):
            return x if c else y
        if x is y:
            return x
        if self.tests(c):
            p, vals = c.args[0], c.args[1]
            return self.case(p, vals, [self.at(x if r else y, p, a) for a, r in zip(vals, c.args[2:])])
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

    def case(self, p, vals, results):
        """`(match p ...)`: results[i] where p is the constructor of
        vals[i]."""
        if all(r is results[0] for r in results):
            return results[0]
        if all(isinstance(r, Lit) and r.k == a for r, a in zip(results, vals)):
            return p
        return self.node('case', p, tuple(vals), *results)

    @staticmethod
    def at(x, p, a):
        """x where p is the constructor of a (a case split on p, its case)."""
        if isinstance(x, Node) and x.op == 'case' and x.args[0] is p and a in x.args[1]:
            return x.args[2 + x.args[1].index(a)]
        return x

    def all(self, xs):
        """The conjunction of xs, a balanced tree."""
        xs = [x for x in xs if x is not True]
        if not xs:
            return True
        if len(xs) == 1:
            return xs[0]
        h = len(xs) // 2
        return self.AND(self.all(xs[:h]), self.all(xs[h:]))

    # --- integers: their forms and case splits -----------------------------------

    def shape(self, v):
        """The form of int variable v: 'k' a constant, 'b' a Boolean (two
        values), 'c' a constructor (more), 'bits' a binary integer."""
        if len(v.support) == 1:
            return 'k'
        if not self.split_on(v):
            return 'bits'
        return 'b' if len(v.support) == 2 else 'c'

    def leaf(self, form, support):
        """The value a of a variable of the form and support, written."""
        if form == 'c':
            return self.lit
        hi = support[-1]
        return lambda a: a == hi

    def split(self, x, f):
        """f(a) where x has the value a, a case split on x."""
        if x.form == 'k':
            return f(x.x)
        if x.form == 'b':
            lo, hi = x.support
            return self.ITE(x.x, f(hi), f(lo))
        if x.form == 'c':
            if isinstance(x.x, Lit):
                return f(x.x.k)
            return self.case(x.x, x.support, [f(a) for a in x.support])
        raise ValueError('a case split on a binary integer')   # categoricals keeps them apart

    def constructor(self, x):
        """The constructor of x's value."""
        return x.x if x.form == 'c' else self.split(x, self.lit)

    def lift(self, xs, f):
        """f of the values of the Nums xs, case splits on them in turn."""
        def go(i, vals):
            return f(*vals) if i == len(xs) else self.split(xs[i], lambda a: go(i + 1, vals + [a]))
        return go(0, [])

    def bits(self, x):
        if x.form == 'bits':
            return x.x
        if x.form == 'k':
            return [bool(x.x >> i & 1) for i in range(x.w)]
        return [self.split(x, lambda a, i=i: bool(a >> i & 1)) for i in range(x.w)]

    def convert(self, x, v):
        """x in the form of variable v."""
        form, w = self.shape(v), v.t[1]
        if form == 'k':
            return Num(w, v.support, 'k', v.support[0])
        if form == 'bits':
            return Num(w, v.support, 'bits', self.bits(x))
        if form == 'c':   # v's values include x's
            return Num(w, v.support, 'c', self.constructor(x))
        if x.form == 'b' and x.support == v.support:
            return x
        return Num(w, v.support, form, self.split(x, self.leaf(form, v.support)))

    def choose(self, c, v, a, b):
        """v = a if c else b."""
        a, b = self.convert(a, v), self.convert(b, v)
        if a.form == 'k':
            return a
        if a.form == 'bits':
            return Num(a.w, v.support, 'bits', [self.ITE(c, x, y) for x, y in zip(a.x, b.x)])
        return Num(a.w, v.support, a.form, self.ITE(c, a.x, b.x))

    # --- binary integers, the least significant bit first --------------------------

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

    def binary(self, op, args, xs):
        """OP on binary integers (xs, their bits): the result's bits, or
        [the Boolean] of a comparison."""
        if op in ('shl', 'shr'):
            a, k = xs[0], args[1]
            w = len(a)
            return [False] * min(k, w) + a[:max(w - k, 0)] if op == 'shl' else a[k:] + [False] * min(k, w)
        a, b = xs
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

    # --- draws --------------------------------------------------------------------

    def draw(self, vals, mass, leaf):
        """One of vals (sorted) with the masses `mass`, the decision tree of
        flips of Pluck's `discrete`: the upper half against the lower."""
        if len(vals) == 1:
            return leaf(vals[0])
        h = len(vals) // 2
        lo, hi = vals[:h], vals[h:]
        up = math.fsum(mass[a] for a in hi)
        return self.ITE(self.flip(up / (up + math.fsum(mass[a] for a in lo))),
                        self.draw(hi, mass, leaf), self.draw(lo, mass, leaf))

    def sample(self, v, dist, p):
        if dist == 'flip':
            return [self.flip(p)]
        w = v.t[1]
        if dist == 'uniform':
            lo, hi = p
            p = [0.0] * lo + [1.0 / (hi - lo)] * (hi - lo)
        p = [float(q) for q in p] + [0.0] * (2 ** w - len(p))
        form = self.shape(v)
        if form == 'k':
            return Num(w, v.support, 'k', v.support[0])
        if form == 'bits':
            return Num(w, v.support, 'bits', self.bits_drawn(p, w))
        return Num(w, v.support, form, self.draw(list(v.support), dict(enumerate(p)), self.leaf(form, v.support)))

    def bits_drawn(self, m, w):
        """The w bits of a value drawn with masses m, the most significant
        first, each a flip given the bits above it."""
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
        """[the Boolean] of a Boolean, the Num of an int."""
        if isinstance(x, Const):
            return [x.k == 1] if x.t == BOOL else Num(x.t[1], (x.k,), 'k', x.k)
        return env[x]

    def op(self, v, op, args, env):
        if op in ('eq', 'ne', 'lt', 'le') and args[0].t == BOOL or op not in ('eq', 'ne', 'lt', 'le') and v.t == BOOL:
            return self.boolean(op, [self.val(x, env) for x in args])
        if op == 'ite':
            return self.choose(self.val(args[0], env)[0], v, self.val(args[1], env), self.val(args[2], env))
        if op == 'copy':
            return self.convert(self.val(args[0], env), v)
        xs = [self.val(x, env) for x in (args[:1] if op in ('shl', 'shr') else args)]
        if any(x.form == 'bits' for x in xs) or v.t != BOOL and self.shape(v) == 'bits':
            out = self.binary(op, args, [self.bits(x) for x in xs])
            return out if v.t == BOOL else Num(v.t[1], v.support, 'bits', out)
        t = args[0].t if op in ('eq', 'ne', 'lt', 'le', 'add', 'sub', 'mul') else v.t
        extra = list(args[1:]) if op in ('shl', 'shr') else []
        if v.t == BOOL:
            return [self.lift(xs, lambda *a: apply(op, t, list(a)) == 1)]
        form = self.shape(v)
        leaf = self.leaf(form, v.support)
        return Num(v.t[1], v.support, form, self.lift(xs, lambda *a: leaf(apply(op, t, list(a) + extra))))

    def boolean(self, op, xs):
        if op == 'copy':
            return xs[0]
        if op == 'not':
            return [self.NOT(xs[0][0])]
        if op == 'ite':
            return [self.ITE(xs[0][0], xs[1][0], xs[2][0])]
        a, b = xs[0][0], xs[1][0]
        if op in ('and', 'or', 'xor', 'iff'):
            return [{'and': self.AND, 'or': self.OR, 'xor': self.XOR, 'iff': self.IFF}[op](a, b)]
        if op in ('eq', 'ne'):
            eq = self.IFF(a, b)
            return [eq if op == 'eq' else self.NOT(eq)]
        if op == 'lt':
            return [self.AND(self.NOT(a), b)]
        if op == 'le':
            return [self.OR(self.NOT(a), b)]
        raise ValueError(f'unknown operator {op}')

    def stmts(self, ss, env, obs):
        """Translates statements ss, their variables' values into env and
        their observations into obs."""
        for s in ss:
            kind = s[0]
            if kind == 'sample':
                env[s[1]] = self.sample(s[1], s[2], s[3])
            elif kind == 'op':
                env[s[1]] = self.op(s[1], s[2], s[3], env)
            elif kind == 'observe':
                obs.append(self.val(s[1], env)[0])
            elif kind == 'if':
                _, c, then, other, outs = s
                c = self.val(c, env)[0]
                ot, oe = [], []
                self.stmts(then, env, ot)
                self.stmts(other, env, oe)
                for v, a, b in outs:
                    a, b = self.val(a, env), self.val(b, env)
                    env[v] = [self.ITE(c, a[0], b[0])] if v.t == BOOL else self.choose(c, v, a, b)
                if ot or oe:
                    obs.append(self.ITE(c, self.all(ot), self.all(oe)))
            elif kind == 'loop':
                self.loop(s, env, obs)
            else:
                raise ValueError(f'unknown statement {kind}')

    def slots(self, x, v):
        """What the tuple of a loop's state holds of variable v, from x."""
        if v.t == BOOL:
            return x
        y = self.convert(x, v)
        return [] if y.form == 'k' else y.x if y.form == 'bits' else [y.x]

    def unslot(self, v, it):
        """Variable v from the values of the tuple's iterator it."""
        if v.t == BOOL:
            return [next(it)]
        form, w = self.shape(v), v.t[1]
        if form == 'k':
            return Num(w, v.support, 'k', v.support[0])
        return Num(w, v.support, form, [next(it) for _ in range(w)] if form == 'bits' else next(it))

    def loop(self, s, env, obs):
        _, k, states, inits, body, nexts = s
        start = [x for v, a in zip(states, inits) for x in self.slots(self.val(a, env), v)]
        observes = has_observe(body)
        width = len(start) + (1 if observes else 0)
        if width == 0:
            return
        paths = tuple_paths(width)
        outer = self.scope
        self.scope = inner = Scope(outer)
        self.scopes.append(inner)
        param = self.node('param', fresh=True)
        it = iter([self.node('proj', param, tuple(p), fresh=True) for p in paths])
        for v in states:
            env[v] = self.unslot(v, it)
        body_obs = []
        self.stmts(body, env, body_obs)
        result = [x for v, a in zip(states, nexts) for x in self.slots(self.val(a, env), v)]
        if observes:
            result.append(self.AND(next(it), self.all(body_obs)))
            start.append(True)
        self.scope = outer
        lp = self.node('loop', k, param, inner, tuple(start), tuple(result), fresh=True)
        out = iter([self.node('proj', lp, tuple(p), fresh=True) for p in paths])
        for v in states:
            env[v] = self.unslot(v, out)
        if observes:
            obs.append(next(out))

    # --- text ---------------------------------------------------------------------

    def count(self, roots):
        """Which nodes are bindings: those the program reads more than once
        or from an inner scope (a loop's body or start), and the variables
        a `match` reads, named as they are in Pluck's programs."""
        stack = [(x, self.top, False) for x in roots]
        seen = set()
        while stack:
            x, frm, cross = stack.pop()
            if not isinstance(x, Node):
                continue
            x.refs += 1
            if cross or x.scope is not frm:
                x.cross = True
            if x.n in seen:
                continue
            seen.add(x.n)
            if x.op == 'loop':
                _, param, inner, start, result = x.args
                stack += [(y, x.scope, True) for y in start] + [(y, inner, False) for y in result]
            elif x.op == 'case':
                stack += [(x.args[0], x.scope, True)] + [(y, x.scope, False) for y in x.args[2:]]
            else:
                stack += [(y, x.scope, False) for y in (x.args[:1] if x.op == 'proj' else x.args)]
        # a chain of `and` or of `or` written in place nests as deep as it is
        # long (a ground conjunction of a million literals): every CHAIN of
        # its operations, one is a binding
        run = {}
        for scope in self.scopes:
            for v in scope.nodes:
                if v.op in ('and', 'or'):
                    run[v.n] = 1 + max((run[a.n] for a in v.args if isinstance(a, Node) and a.op == v.op
                                        and not self.bound(a)), default=0)
                    v.chained = run[v.n] >= CHAIN

    def bound(self, v):
        return v.refs > 0 and (v.op in ('param', 'loop') or v.refs > 1 or v.cross or v.chained)

    def code(self, x):
        """x where it is read: a binding's name, else its expression."""
        if isinstance(x, bool):
            return '(True)' if x else '(False)'
        if isinstance(x, Lit):
            return f'(V{x.k})'
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
        if op == 'case':
            cases = ' '.join(f'V{k} => {self.code(r)}' for k, r in zip(a[1], a[2:]))
            return f'(match {self.code(a[0])} {cases})'
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
        self.split_on = categoricals(stmts)
        env, obs = {}, []
        self.stmts(stmts, env, obs)
        values, types, forms = [], [], []
        for c in comps:
            if joint:
                values.append([self.val(x, env)[0] for x in c])
                types.append(('int', len(c)))
                forms.append('bits')
            elif c[0].t == BOOL:
                values.append(self.val(c[0], env))
                types.append(BOOL)
                forms.append('bool')
            else:
                x = self.val(c[0], env)
                values.append(x.x[::-1] if x.form == 'bits' else [self.constructor(x)])
                types.append(c[0].t)
                forms.append(x.form)
        ev = self.all(obs)
        roots = [ev] + [x for b in values for x in b]
        self.count(roots)
        codes = []
        for f, b in zip(forms, values):
            if f != 'bits':
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
        head = ''
        if self.lits:
            head = '(define-type value ' + ' '.join(f'(V{k})' for k in sorted(self.lits)) + ')\n\n'
        program = head + f'(define (dice_model)\n  {model})\n\n' + '\n'.join(queries) + '\n'
        return program, types, forms


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
    program, types, forms = deep(lambda: Translator().translate(text, joint))
    what = {'bool': 'bool', 'bits': 'int({}), its bits'}
    head = (f';; {source}, translated by tddlog\'s scripts/dice2pluck.py: query cK is component K of its value ('
            + ', '.join(what.get(f, 'int({}), its constructor V0 ...').format(t[1] if t != BOOL else '')
                        for t, f in zip(types, forms)) + ').\n')
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
