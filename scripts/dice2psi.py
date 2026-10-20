#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a program of Psi's language (Gehr, Misailovic and Vechev, CAV 2016;
github.com/eth-sri/psi), the exact symbolic inference engine.

The program is read by dice2problog.py's parser. Psi is imperative with
real-valued variables: a Boolean is a Psi number that is 0 or 1 (`flip(p)`
draws one; `!`, `&&`, `||` and `==` act on them), an int(w) a number from 0
to 2^w - 1 whose `+`, `-` and `*` are taken modulo 2^w as Dice's, a tuple
a Psi tuple. `discrete(...)` is `categorical([...])` and `uniform(w, lo,
hi)` `uniformInt(lo, hi - 1)`, so Psi sums over the values where Dice
draws bits; the answer is the same distribution. `observe e` is
`observe(e)`, which Psi renormalizes by. A Dice function is a Psi
function, `iterate(f, x, k)` a `for` loop of k calls.

Psi returns the distribution of one expression, and a tuple's value would
come out as the joint. The program's value is therefore asked one
component at a time, as run_dice does for the networks: `dice2psi.py
program.dice K` writes the program whose value is the K-th component
(counted from 0, left to right through nested tuples; `--components` prints
how many there are and each one's type: `bool` or `int W`). With
`--joint` the value, a tuple of Booleans, is one int(N) whose bits they
are, the first the most significant: the Dice programs of an Alea.jl
circuit's `joint` lines (bench.py's dicemap). A program Psi cannot express
is refused (exit 3).

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions (Psi has no recursion).

Usage: dice2psi.py program.dice K [--joint] > program.psi
       dice2psi.py program.dice --components [--joint]
"""
import argparse
from fractions import Fraction
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice2problog import (Parser, bits_for, calls, deep,  # noqa: E402
                          discrete_probs, tokenize)
from dice_ir import refuse  # noqa: E402

BOOL = ('bool',)
ATOM = re.compile(r'[A-Za-z_][A-Za-z0-9_]*|[0-9]+(\.[0-9]+)?')


def literal(p):
    """A probability as Psi reads it: the double's shortest decimal."""
    return repr(float(p))


def exact(ps):
    """The probabilities of a categorical as rationals that sum to 1 exactly:
    Psi answers with probability-of-error 1 where a categorical's floating
    point entries do not sum to 1 (k/36 for k = 1 .. 8 does not)."""
    fs = [Fraction(p).limit_denominator(10 ** 12) for p in ps]
    total = sum(fs)
    return [f'{(f / total).numerator}/{(f / total).denominator}' for f in fs]


def same(a, b, what):
    if a != b:
        raise TypeError(f'{what}: {show(a)} and {show(b)}')
    return a


def show(t):
    if t[0] == 'bool':
        return 'bool'
    if t[0] == 'int':
        return f'int({t[1]})'
    return f'({show(t[1])}, {show(t[2])})'


class Translator:
    def __init__(self):
        self.n = 0
        self.defs = []
        self.funs = {}       # Dice name: (Psi name, parameter types, return type)
        self.tuples = {}     # tuple name: the codes of its two components (a tuple is built only where it is passed on)
        self.built = set()   # the tuple names that have a statement

    def fresh(self, hint):
        self.n += 1
        return f'v{self.n}_{re.sub(r"[^a-z0-9]", "", hint.lower())[:12]}'

    def value(self, e, env, out, hint):
        """e's value as a variable or a constant, its statements in out."""
        code, t = self.expr(e, env, out, hint)
        if ATOM.fullmatch(code):
            return code, t
        v = self.fresh(hint)
        out.append(f'{v} := {code};')
        return v, t

    def expr(self, e, env, out, hint):
        """Psi code for expression e under env (Dice names to Psi
        variables, with their types) and its type; the statements it needs
        are appended to out. Code returned inline creates no flip and
        observes nothing, so reading it twice is harmless; a flip is a
        statement."""
        if e[0] == 'let':
            env = dict(env)
            while e[0] == 'let':
                env[e[1]] = self.value(e[2], env, out, e[1])
                e = e[3]
        kind = e[0]
        if kind == 'var':
            if e[1] not in env:
                raise NameError(f'unbound variable {e[1]}')
            return env[e[1]]
        if kind == 'bool':
            return ('1' if e[1] else '0'), BOOL
        if kind == 'flip':
            p = e[1]
            if p <= 0:
                return '0', BOOL
            if p >= 1:
                return '1', BOOL
            return self.draw(f'flip({literal(p)})', out, hint), BOOL
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            return self.draw(f'categorical([{", ".join(exact(ps))}])', out, hint), \
                ('int', bits_for(len(ps)))
        if kind == 'uniform':
            width, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** width:
                raise ValueError(f'uniform({width}, {lo}, {hi}): empty or out of range')
            return self.draw(f'uniformInt({lo}, {hi - 1})', out, hint), ('int', width)
        if kind == 'int':
            width, k = e[1:]
            if not 0 <= k < 2 ** width:
                raise ValueError(f'integer constant {k} out of range for {2 ** width} values')
            return str(k), ('int', width)
        if kind == 'tuple':
            a, ta = self.value(e[1], env, out, hint)
            b, tb = self.value(e[2], env, out, hint)
            v = self.fresh(hint)
            self.tuples[v] = (a, b)
            return v, ('tuple', ta, tb)
        if kind in ('fst', 'snd'):
            c, t = self.value(e[1], env, out, hint)
            if t[0] != 'tuple':
                raise TypeError(f'{kind} of {show(t)}')
            i = 0 if kind == 'fst' else 1
            # Psi returns 0 for a component of a tuple of tuples: read it from its parts
            return (self.tuples[c][i] if c in self.tuples else f'{c}[{i}]'), t[1 + i]
        if kind == 'not':
            c, t = self.value(e[1], env, out, hint)
            same(t, BOOL, '!')
            return f'!{c}', BOOL
        if kind == 'shift':
            c, t = self.value(e[2], env, out, hint)
            if t[0] != 'int':
                raise TypeError(f'{e[1]} of {show(t)}')
            k, m = e[3], 2 ** t[1]
            if e[1] == '<<':
                return f'(({c} * {2 ** k}) - {m} * floor({c} * {2 ** k} / {m}))', t
            return f'floor({c} / {2 ** k})', t
        if kind == 'bin':
            a, ta = self.value(e[2], env, out, hint)
            b, tb = self.value(e[3], env, out, hint)
            return self.binary(e[1], a, ta, b, tb)
        if kind == 'observe':
            c, t = self.value(e[1], env, out, hint)
            same(t, BOOL, 'observe')
            out.append(f'observe({c});')
            return '1', BOOL
        if kind == 'if':
            return self.ite(e, env, out, hint)
        if kind == 'call':
            args = [self.value(x, env, out, f'{e[1]}arg') for x in e[2]]
            args = [(self.build(c, out), t) for c, t in args]
            name, params, ret = self.signature(e[1])
            if len(params) != len(args):
                raise TypeError(f'{e[1]}: {len(params)} parameters, {len(args)} arguments')
            for p, (_, t) in zip(params, args):
                same(p, t, e[1])
            return self.draw(f'{name}({", ".join(c for c, _ in args)})', out, hint), ret
        if kind == 'iterate':
            init, t = self.value(e[2], env, out, hint)
            init = self.build(init, out)
            name, params, ret = self.signature(e[1])
            if len(params) != 1:
                raise TypeError(f'iterate: {e[1]} takes {len(params)} arguments')
            same(params[0], t, f'iterate({e[1]})')
            same(ret, t, f'iterate({e[1]})')
            s = self.fresh('state')
            out.append(f'{s} := {init};')
            out.append(f'for _ in [0..{e[3]}) {{ {s} = {name}({s}); }}')
            return s, t
        raise ValueError(f'unknown expression {kind}')

    def build(self, code, out):
        """The statement that makes a tuple a Psi value, for a tuple that is
        passed to a function, a loop or out of a branch; Psi answers 0 (error
        probability 1) for a program with a tuple of tuples that it only
        reads, so a tuple that is only taken apart has none."""
        if code in self.tuples and code not in self.built:
            a, b = self.tuples[code]
            self.build(a, out)
            self.build(b, out)
            out.append(f'{code} := ({a}, {b});')
            self.built.add(code)
        return code

    def draw(self, code, out, hint):
        v = self.fresh(hint)
        out.append(f'{v} := {code};')
        return v

    def binary(self, op, a, ta, b, tb):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op in ('&&', '||'):
            same(ta, BOOL, op)
            same(tb, BOOL, op)
            # Psi crashes on a constant operand of && and ||
            if '0' in (a, b) or '1' in (a, b):
                one, other = (a, b) if a in ('0', '1') else (b, a)
                if (one == '1') == (op == '&&'):
                    return other, BOOL
                return one, BOOL
            return f'({a} {op} {b})', BOOL
        if ta == BOOL and op in ('^', '<=>', '==', '!='):
            same(tb, BOOL, op)
            return (f'({a} != {b})' if op in ('^', '!=') else f'({a} == {b})'), BOOL
        if ta[0] != 'int':
            raise TypeError(f'{op} on {show(ta)}')
        same(ta, tb, op)
        m = 2 ** ta[1]
        if op == '+':
            return f'(({a} + {b}) - {m} * (({a} + {b}) >= {m}))', ta
        if op == '-':
            return f'(({a} - {b}) + {m} * ({a} < {b}))', ta
        if op == '*':
            return f'(({a} * {b}) - {m} * floor({a} * {b} / {m}))', ta
        if op in ('==', '!=', '<', '>', '<=', '>='):
            return f'({a} {op} {b})', BOOL
        raise ValueError(f'unknown operator {op}')

    def ite(self, e, env, out, hint):
        c, t = self.value(e[1], env, out, hint)
        same(t, BOOL, 'if')
        if c == '1':
            return self.expr(e[2], env, out, hint)
        if c == '0':
            return self.expr(e[3], env, out, hint)
        then, other = [], []
        built = set(self.built)   # a tuple a branch builds is not there after it
        a, ta = self.value(e[2], env, then, hint)
        b, tb = self.value(e[3], env, other, hint)
        t = same(ta, tb, 'if')
        a, b = self.build(a, then), self.build(b, other)
        self.built = built
        r = self.fresh(hint)
        out.append(f'{r} := 0;')
        out.append(f'if {c} {{ {" ".join(then)} {r} = {a}; }} else {{ {" ".join(other)} {r} = {b}; }}')
        return r, t

    def signature(self, fname):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        return self.funs[fname]

    def function(self, fname, params, types, body):
        if None in types:
            raise TypeError(f'{fname}: a parameter without a type')
        name = 'f_' + re.sub(r'[^a-z0-9]', '_', fname.lower())
        while any(name == f[0] for f in self.funs.values()):
            name += '_'
        args = [self.fresh(p) for p in params]
        out = []
        value, t = self.value(body, {p: (a, ty) for p, a, ty in zip(params, args, types)}, out, fname)
        value = self.build(value, out)
        self.funs[fname] = (name, types, t)
        self.defs.append(f'def {name}({", ".join(args)}) {{\n  ' + '\n  '.join(out + [f'return {value};']) + '\n}')

    def components(self, t, code):
        """The leaves of a value of type t: (code, type) each."""
        if t[0] == 'tuple':
            a, b = self.tuples[code] if code in self.tuples else (f'{code}[0]', f'{code}[1]')
            return self.components(t[1], a) + self.components(t[2], b)
        return [(code, t)]

    def translate(self, text, k, joint=False):
        """(the Psi program asking component k, the components' types). With
        joint, the value, a tuple of Booleans, is one int(N) whose bits they
        are, the first the most significant: their joint distribution."""
        parser = Parser(tokenize(text))
        funs, body = parser.program()
        names = [f[0] for f in funs]
        for i, (fname, params, ret, fbody) in enumerate(funs):
            for other in names[i:]:
                if calls(fbody, other):
                    raise NotImplementedError(f'{fname} calls {other}: Psi has no recursion'
                                              if other == fname else f'{fname} calls {other}, defined after it')
            self.function(fname, params, parser.param_types[fname], fbody)
        out = []
        value, t = self.value(body, {}, out, 'result')
        comps = self.components(t, value)
        if joint:
            if any(ct != BOOL for _, ct in comps):
                raise TypeError('joint: a value of Booleans')
            n = len(comps)
            comps = [(' + '.join(f'({c})*{2 ** (n - 1 - i)}' for i, (c, _) in enumerate(comps)), ('int', n))]
        types = [ct for _, ct in comps]
        if k is None:
            return None, types
        if not 0 <= k < len(comps):
            raise ValueError(f'component {k} of {len(comps)}')
        main = '\n  '.join(out + [f'return {comps[k][0]};'])
        return '\n\n'.join(self.defs + [f'def main() {{\n  {main}\n}}']) + '\n', types


def translate(text, k, source='a Dice program', joint=False):
    """The Psi program that returns component k of a Dice program's value,
    with the types of all the components (`bool` or ('int', w)); with
    joint, of the int whose bits are the value's Booleans."""
    program, types = deep(lambda: Translator().translate(text, k, joint))
    if program is not None:
        program = f'// {source}, translated by tddlog\'s scripts/dice2psi.py, component {k}.\n' + program
    return program, types


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
