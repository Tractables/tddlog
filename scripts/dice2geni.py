#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into the language of Geni (gennifer; Li and Zhang, "Compiling with
Generating Functions", ICFP 2025), which compiles a program to a
generating function.

The program is lowered by dice_ir.py. Geni's language (its IR, the .gir
files of its artifact) is Dice's Boolean core: `let x = e in e`,
`if b then e else e`, pairs and their projections `(fst t)`, `(snd t)`,
`sample(Bernoulli(p))` for `flip p`, `observe b`, and Boolean expressions
of variables with `!`, `&&` and `||`; its interpreter answers a program
whose value is a Boolean or a tuple of them. An int(w) is therefore
w Booleans, as Dice and the bit-blasting of Geni's own surface language
hold it: `discrete` is a chain of Bernoullis, the most significant bit
first, each conditioned on the bits before it; + and - are ripple-carry
adders modulo 2^w, * a sum of shifted partial products, < the carry of
a - b, == a conjunction of equal bits, shifts a rewiring. A bit that the
variable's values (dice_ir.py's supports) fix is that constant. Each gate
is a variable of its own (`let g = (a && !b) in`); a negation is a
literal. A Dice conditional is an `if` whose branches return, as a tuple,
the bits of its outs that differ between them; `iterate(f, x, k)` is
unrolled.

Geni prints the distribution of the program's value, so the program's
value is asked one component at a time, as dice2genfer.py does:
`dice2geni.py program.dice K` writes the program whose value is the K-th
component (counted from 0, left to right through nested tuples), an
int(w) as the tuple of its bits, the most significant first; `--components`
prints each component's type (`bool` or `int W`). Geni does not normalize
by the observations' probability: the masses of all the values, which it
prints, are divided by their sum.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions.

Usage: dice2geni.py program.dice K [--joint] > program.gir
       dice2geni.py program.dice --components [--joint]
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice_ir import BOOL, Const, Var, decimal, deep, leaves, lower, prune, refuse  # noqa: E402

# a discrete over more values is refused: its chain of Bernoullis has a
# leaf for each value
MAX_DISCRETE_BITS = 16


def neg(a):
    """The negation of an atom: a Python bool, or (name, negated)."""
    return (not a) if isinstance(a, bool) else (a[0], not a[1])


class Emitter:
    def __init__(self):
        self.n = 0
        self.env = {}      # Var: its bits, least significant first, each an atom
        self.lines = []    # the lets of the current scope

    # --- gates ----------------------------------------------------------------

    def lit(self, a):
        if isinstance(a, bool):
            return 'true' if a else 'false'
        return f'!{a[0]}' if a[1] else a[0]

    def expr(self, a):
        """An atom as an expression."""
        return f'({self.lit(a)})' if not isinstance(a, bool) and a[1] else self.lit(a)

    def let(self, e):
        self.n += 1
        name = f'g{self.n}'
        self.lines.append(f'let {name} = {e} in')
        return (name, False)

    def AND(self, a, b):
        if isinstance(a, bool):
            return b if a else False
        if isinstance(b, bool):
            return a if b else False
        if a == b:
            return a
        if a == neg(b):
            return False
        return self.let(f'({self.lit(a)} && {self.lit(b)})')

    def OR(self, a, b):
        return neg(self.AND(neg(a), neg(b)))

    def XOR(self, a, b):
        if isinstance(a, bool):
            return neg(b) if a else b
        if isinstance(b, bool):
            return neg(a) if b else a
        if a == b:
            return False
        if a == neg(b):
            return True
        if a[1] != b[1]:   # !x ^ y is !(x ^ y)
            return neg(self.XOR(neg(a) if a[1] else a, neg(b) if b[1] else b))
        a, b = (a[0], False), (b[0], False)
        return self.let(f'(({self.lit(a)} && {self.lit(neg(b))}) || ({self.lit(neg(a))} && {self.lit(b)}))')

    def ITE(self, c, a, b):
        if isinstance(c, bool):
            return a if c else b
        if a == b:
            return a
        if isinstance(a, bool) or isinstance(b, bool) or a == neg(b) or c[0] in (a[0], b[0]):
            return self.OR(self.AND(c, a), self.AND(neg(c), b))
        if c[1]:
            c, a, b = neg(c), b, a
        return self.let(f'(({self.lit(c)} && {self.lit(a)}) || ({self.lit(neg(c))} && {self.lit(b)}))')

    def MAJ(self, a, b, c):
        """The carry of a + b + c."""
        if isinstance(c, bool):
            return self.OR(a, b) if c else self.AND(a, b)
        if isinstance(a, bool) or isinstance(b, bool) or a == b or a == neg(b):
            return self.OR(self.AND(a, b), self.AND(c, self.OR(a, b)))
        return self.let(f'(({self.lit(a)} && {self.lit(b)}) || ({self.lit(c)} && ({self.lit(a)} || '
                        f'{self.lit(b)})))')

    # --- words ------------------------------------------------------------------

    def bits(self, x):
        if isinstance(x, Const):
            return [bool(x.k)] if x.t == BOOL else [bool(x.k >> i & 1) for i in range(x.t[1])]
        return self.env[x]

    def adder(self, a, b, carry):
        """(a + b + carry) modulo 2^w, and the carry out."""
        out = []
        for x, y in zip(a, b):
            out.append(self.XOR(self.XOR(x, y), carry))
            carry = self.MAJ(x, y, carry)
        return out, carry

    def less(self, a, b):
        """a < b: no carry out of a + ~b + 1."""
        carry = True
        for x, y in zip(a, b):
            carry = self.MAJ(x, neg(y), carry)
        return neg(carry)

    def multiply(self, a, b):
        w = len(a)
        total = [False] * w
        for i, y in enumerate(b):
            if y is False:
                continue
            partial = [False] * i + [self.AND(x, y) for x in a[:w - i]]
            total, _ = self.adder(total, partial, False)
        return total

    def operation(self, v, op, args):
        """The bits of v = op(args)."""
        if op in ('shl', 'shr'):
            x, k = self.bits(args[0]), args[1]
            return [False] * k + x[:len(x) - k] if op == 'shl' else x[k:] + [False] * k
        xs = [self.bits(a) for a in args]
        if op == 'copy':
            return xs[0]
        if op == 'not':
            return [neg(xs[0][0])]
        if op in ('and', 'or', 'xor', 'iff'):
            r = {'and': self.AND, 'or': self.OR, 'xor': self.XOR, 'iff': self.XOR}[op](xs[0][0], xs[1][0])
            return [neg(r) if op == 'iff' else r]
        if op == 'ite':
            c, a, b = xs
            return [self.ITE(c[0], x, y) for x, y in zip(a, b)]
        if op in ('eq', 'ne'):
            r = True
            for x, y in zip(*xs):
                r = self.AND(r, neg(self.XOR(x, y)))
            return [r if op == 'eq' else neg(r)]
        if op == 'lt':
            return [self.less(*xs)]
        if op == 'le':
            return [neg(self.less(xs[1], xs[0]))]
        if op == 'add':
            return self.adder(xs[0], xs[1], False)[0]
        if op == 'sub':
            return self.adder(xs[0], [neg(y) for y in xs[1]], True)[0]
        if op == 'mul':
            return self.multiply(*xs)
        raise ValueError(f'unknown operator {op}')

    def fixed(self, v, bits):
        """bits, with the ones that v's values agree on as constants."""
        if v.t == BOOL:
            return bits
        out = []
        for i, b in enumerate(bits):
            seen = {k >> i & 1 for k in v.support}
            out.append(bool(seen.pop()) if len(seen) == 1 else b)
        return out

    def discrete(self, ps, w):
        """The bits of a categorical over 0 .. 2^w - 1 (ps, padded with 0):
        each bit, the most significant first, a Bernoulli conditioned on the
        bits before it."""
        if w > MAX_DISCRETE_BITS:
            raise NotImplementedError(f'a discrete of {w} bits')
        ps = list(ps) + [0.0] * (2 ** w - len(ps))
        top = []                      # sampled bits, most significant first

        def tree(j, prefix):
            # the expression of bit j (counted from the top) after the bits
            # of prefix (a value of the j bits above)
            if len(prefix) < j:
                above = top[len(prefix)]
                if isinstance(above, bool):
                    return tree(j, prefix + [int(above)])
                a = tree(j, prefix + [1])
                b = tree(j, prefix + [0])
                if a == b:
                    return a
                return f'if {self.lit(top[len(prefix)])} then ({a}) else ({b})'
            span = 2 ** (w - j)
            lo = 0
            for bit in prefix:
                lo = 2 * lo + bit
            lo *= span
            mass = sum(ps[lo:lo + span])
            one = sum(ps[lo + span // 2:lo + span])
            if mass <= 0 or one <= 0:
                return 'false'
            if one >= mass:
                return 'true'
            return f'sample(Bernoulli({decimal(one / mass)}))'
        for j in range(w):
            e = tree(j, [])
            top.append(True if e == 'true' else False if e == 'false' else self.let(e))
        return list(reversed(top))

    # --- statements -------------------------------------------------------------

    def emit(self, stmts):
        for s in stmts:
            kind = s[0]
            if kind == 'sample':
                _, v, dist, p = s
                if dist == 'flip':
                    self.env[v] = [self.let(f'sample(Bernoulli({decimal(p)}))')]
                else:
                    if dist == 'uniform':
                        lo, hi = p
                        p = [0.0] * lo + [1.0 / (hi - lo)] * (hi - lo)
                    self.env[v] = self.fixed(v, self.discrete(p, v.t[1]))
            elif kind == 'op':
                _, v, op, args = s
                self.env[v] = self.fixed(v, self.operation(v, op, args))
            elif kind == 'observe':
                a = self.bits(s[1])[0]
                if a is not True:
                    self.n += 1
                    self.lines.append(f'let o{self.n} = observe {self.lit(a)} in')
            elif kind == 'if':
                self.emit_if(s)
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                current = [self.bits(x) for x in inits]
                for i in range(k + 1):
                    for v, x in zip(states, current):
                        self.env[v] = x
                    if i == k:
                        break
                    self.emit(body)
                    current = [self.bits(x) for x in nexts]

    def branch(self, stmts, sources):
        """The lets of a branch and the bits of its outs' sources."""
        outer = self.lines
        self.lines = []
        self.emit(stmts)
        bits = [self.bits(x) for x in sources]
        lines, self.lines = self.lines, outer
        return lines, bits

    def emit_if(self, s):
        _, c, then, other, outs = s
        cond = self.bits(c)[0]
        a_lines, a_bits = self.branch(then, [o[1] for o in outs])
        b_lines, b_bits = self.branch(other, [o[2] for o in outs])
        # the bits that differ go through the tuple; the others are atoms of
        # the outer scope (or constants), the same in both branches
        if isinstance(cond, bool):   # a condition the gates decided
            lines, bits = (a_lines, a_bits) if cond else (b_lines, b_bits)
            self.lines += lines
            for (v, _, _), xs in zip(outs, bits):
                self.env[v] = self.fixed(v, xs)
            return
        diff, result = [], []
        for (v, _, _), xs, ys in zip(outs, a_bits, b_bits):
            bits = []
            for x, y in zip(xs, ys):
                if x == y:
                    bits.append(x)
                else:
                    bits.append(None)
                    diff.append((x, y))
            result.append((v, bits))
        if not a_lines and not b_lines:
            # no statements in the branches: each differing bit an ite
            got = iter([self.ITE(cond, x, y) for x, y in diff])
        else:
            def body(lines, values):
                tup = values[-1] if values else 'false'
                for e in reversed(values[:-1]):
                    tup = f'({e}, {tup})'
                return '(' + ' '.join(lines + [tup]) + ')'
            then_e = body(a_lines, [self.expr(x) for x, _ in diff])
            else_e = body(b_lines, [self.expr(y) for _, y in diff])
            if isinstance(cond, tuple) and cond[1]:
                cond, then_e, else_e = neg(cond), else_e, then_e
            t = self.let(f'if {self.lit(cond)} then {then_e} else {else_e}')
            if len(diff) <= 1:
                got = iter([t])
            else:
                projections = []
                for i in range(len(diff)):
                    p = t[0]
                    for _ in range(i):
                        p = f'(snd {p})'
                    projections.append(f'(fst {p})' if i < len(diff) - 1 else p)
                got = iter([self.let(p) for p in projections])
        for v, bits in result:
            self.env[v] = self.fixed(v, [b if b is not None else next(got) for b in bits])


def translate(text, k, source='a Dice program', joint=False):
    """The Geni program that returns component k of a Dice program's value,
    with the types of all the components (`BOOL` or ('int', w))."""
    def run():
        stmts, value = lower(text, joint)
        comps = leaves(value)
        types = [x.t for x in comps]
        if k is None:
            return None, types
        if not 0 <= k < len(comps):
            raise ValueError(f'component {k} of {len(comps)}')
        result = comps[k]
        kept = prune(stmts, {result} if isinstance(result, Var) else set())
        e = Emitter()
        e.emit(kept)
        bits = e.bits(result)
        ret = e.expr(bits[0])
        for b in bits[1:]:        # the most significant bit first
            ret = f'({e.expr(b)}, {ret})'
        return '\n'.join(e.lines + [ret]) + '\n', types
    return deep(run)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('program')
    ap.add_argument('component', nargs='?', type=int)
    ap.add_argument('--components', action='store_true', help="print each component's type instead")
    ap.add_argument('--joint', action='store_true', help='the value, a tuple of Booleans, as one int whose bits '
                    'they are (the most significant first): their joint distribution')
    a = ap.parse_args()
    if a.components == (a.component is not None):
        raise SystemExit('give a component K, or --components')
    with open(a.program) as f:
        text = f.read()
    try:
        program, types = translate(text, a.component, os.path.basename(a.program), a.joint)
    except NotImplementedError as err:
        refuse(err)
    if a.components:
        for t in types:
            print('bool' if t == BOOL else f'int {t[1]}')
    else:
        sys.stdout.write(program)


if __name__ == '__main__':
    main()
