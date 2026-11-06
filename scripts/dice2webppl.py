#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a WebPPL program (github.com/probmods/webppl; Goodman and Stuhlmüller,
The Design and Implementation of Probabilistic Programming Languages,
dippl.org) that WebPPL answers exactly, with `Infer({method: 'enumerate'},
model)`.

The program is read by dice2problog.py's parser and written as a WebPPL
user writes the model. A Dice function is a WebPPL function, a `let` a
`var`, and the program's body the function `model`, whose value is the
Dice program's value (a tuple an array). A Boolean is a JavaScript
Boolean, `flip p` is `flip(p)`. An int(w) is a number from 0 to 2^w - 1
whose `+`, `-`, `*` and `<<` are taken modulo 2^w, as Dice's are, and `>>`
is the floor of the quotient. `discrete(...)` is `categorical({ps, vs})`
over the values of 0 to n - 1 that have mass, `uniform(w, lo, hi)` is
`randomInteger(hi - lo)` plus lo,
and `observe e` is `condition(e)`. `if c then a else b` is `c ? a : b`.
An `if` whose branches bind variables or observe is an `if` statement in
a function applied in place, so that a branch's observations hold only
when it is taken, as in Dice. Dice evaluates both operands of `&&` and
`||`, and JavaScript the second only when the first does not decide: a
second operand that calls a function that observes is a `var` of its own,
so that its observations hold either way. A chain of `&&` or of `||` of
more than 256 operands (dice2problog.py's CHAIN) is one call of WebPPL's
`all` or `any` over the array of its operands, every one of them
evaluated, as Dice evaluates them: WebPPL's compile of a chain grows faster
than the square of its length (1283 operands, 400 s), and of the call
about as its length (0.6 s). `nth_bit(int(w, i), x)`, bit i of x
from the most significant, is `Math.floor(x / 2^(w-1-i)) % 2 == 1`.
`iterate(f, x, k)` is the
recursive function `iterate`, and a recursive Dice function is a
recursive WebPPL function.

Dice cuts recursion at ten nested calls of a function (its default limit:
the deepest call is unreachable, an observation of false). WebPPL recurses
as deep as the program does, so a program that recurses past the limit is
another model in WebPPL, and its enumeration does not end where the
recursion does not (data/dice/geometric.dice).

The value is asked with one `Infer`, its distribution, from which
webppl_dice.js reads each component's marginal: enumeration's cost is its
executions, which a run for each component would repeat. A comment line
`// component K bool` or `// component K int W` gives each component's
type, the components counted from 0, left to right through nested tuples.
With `--joint` the value, a tuple of Booleans, is one int(N) whose bits
they are, the first the most significant: the Dice programs of an Alea.jl
circuit's `joint` lines (bench.py's dicemap).

Supported: what dice2problog.py supports except `/`, `%` and `::`. Lists
are refused by its parser. Also refused: an int wider than 52 bits, and a
product or a left shift whose exact value can pass 2^53, beyond which a
JavaScript number drops low bits. A program refused is a program WebPPL
cannot be given (exit 3).

Usage: dice2webppl.py program.dice [--joint] > program.wppl
"""
import argparse
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
from dice2problog import CHAIN, Parser, bits_for, deep, discrete_probs, tokenize  # noqa: E402
from dice2psi import BOOL, literal, same, show  # noqa: E402
from dice_ir import refuse  # noqa: E402

# JavaScript's precedence of what a translation writes, high to low; an
# operand of lower precedence than its place needs is parenthesized
ATOM, NOT, MUL, ADD, CMP, EQ, AND, OR, COND = 100, 15, 13, 12, 10, 9, 5, 4, 3
# the names a Dice variable or function is not given: JavaScript's reserved
# words and the names the translation's code reads
RESERVED = set('''break case catch class const continue debugger default delete do else enum export extends false
finally for function if implements import in instanceof interface let new null package private protected public
return static super switch this throw true try typeof var void while with yield await arguments eval undefined NaN
Infinity Math Infer model iterate flip categorical randomInteger condition factor sample globalStore util ad dists
all any console require process Object Array Number Boolean String JSON'''.split())
# JavaScript's numbers hold the integers below 2^53 exactly
EXACT_BITS = 53
# a variable or a component of one, which reading twice evaluates nothing twice
READ = re.compile(r'[A-Za-z_$][\w$]*(\[\d\])*')


class Val:
    """A translated expression: its code, its Dice type, its precedence, the
    parts of a tuple written in place, and whether reading it observes
    (`cond`: it calls a function that observes)."""
    __slots__ = ('code', 't', 'prec', 'parts', 'cond')

    def __init__(self, code, t, prec=ATOM, parts=None, cond=False):
        self.code, self.t, self.prec, self.parts, self.cond = code, t, prec, parts, cond


def width(t):
    """t, refused where an int is wider than a JavaScript number holds."""
    if t is not None and t[0] == 'int' and t[1] > EXACT_BITS - 1:
        raise NotImplementedError(f'int({t[1]}): JavaScript numbers hold integers below 2^{EXACT_BITS} exactly')
    if t is not None and t[0] == 'tuple':
        width(t[1])
        width(t[2])
    return t


def indented(stmts, n):
    pad = ' ' * n
    return [pad + line for s in stmts for line in s.split('\n')]


def observes(e, funs):
    """Whether evaluating e observes: an `observe`, or a call of a function that does."""
    stack = [e]
    while stack:
        e = stack.pop()
        if isinstance(e, tuple) and e:
            if e[0] == 'observe' or (e[0] in ('call', 'iterate') and e[1] in funs and funs[e[1]][3]):
                return True
            stack.extend(x for x in e[1:] if isinstance(x, (tuple, list)))
        elif isinstance(e, list):
            stack.extend(e)
    return False


class Translator:
    def __init__(self):
        self.defs = []
        self.funs = {}            # Dice name: (WebPPL name, parameter types, return type, observes)
        self.globals, self.gnext = set(RESERVED), {}
        self.used, self.next = self.globals, self.gnext
        self.iterate = False      # whether the helper `iterate` is used

    def name(self, base):
        """A fresh WebPPL name in the current scope, from a Dice name."""
        base = base.replace("'", '_')
        if base.startswith('_'):
            base = 'v' + base     # WebPPL's own names begin with _
        if base in RESERVED:
            base += '_'
        n = base
        if n in self.used:
            i = self.next.get(base, 2)
            while f'{base}_{i}' in self.used:
                i += 1
            n = f'{base}_{i}'
            self.next[base] = i + 1
        self.used.add(n)
        return n

    def scope(self):
        """A function's scope: the global names, then its own."""
        self.used, self.next = set(self.globals), {}

    @staticmethod
    def operand(v, need):
        return v.code if v.prec >= need else f'({v.code})'

    def bind(self, v, out, name, hint):
        js = name or self.name(hint)
        out.append(f'var {js} = {v.code};')
        return Val(js, v.t)

    def expr(self, e, env, out, name=None):
        """The translation of expression e under env (Dice names to Vals),
        the statements it needs appended to out. `name`: the variable that a
        value needing a statement of its own (an `if` with statements) is
        bound to, a `let`'s."""
        kind = e[0]
        if kind == 'let':
            env = dict(env)
            while e[0] == 'let':
                x, rhs = e[1], e[2]
                if rhs[0] == 'observe':
                    env[x] = self.expr(rhs, env, out)
                else:
                    js = self.name(x)
                    v = self.expr(rhs, env, out, js)
                    if v.code != js:
                        out.append(f'var {js} = {v.code};')
                    env[x] = Val(js, v.t)
                e = e[3]
            return self.expr(e, env, out, name)
        if kind == 'var':
            if e[1] not in env:
                raise NameError(f'unbound variable {e[1]}')
            return env[e[1]]
        if kind == 'bool':
            return Val('true' if e[1] else 'false', BOOL)
        if kind == 'flip':
            p = e[1]
            if p <= 0 or p >= 1:
                return Val('true' if p >= 1 else 'false', BOOL)
            return Val(f'flip({literal(p)})', BOOL)
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            t = width(('int', bits_for(len(ps))))
            vs = [v for v, p in enumerate(ps) if p > 0]   # the values with mass
            if len(vs) == 1:
                return Val(str(vs[0]), t)
            return Val(f'categorical({{ps: [{", ".join(literal(ps[v]) for v in vs)}], '
                       f'vs: [{", ".join(str(v) for v in vs)}]}})', t)
        if kind == 'uniform':
            w, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** w:
                raise ValueError(f'uniform({w}, {lo}, {hi}): empty or out of range')
            t = width(('int', w))
            return Val(f'randomInteger({hi - lo})', t) if lo == 0 else Val(f'{lo} + randomInteger({hi - lo})', t, ADD)
        if kind == 'int':
            w, k = e[1:]
            if not 0 <= k < 2 ** w:
                raise ValueError(f'integer constant {k} out of range for {2 ** w} values')
            return Val(str(k), width(('int', w)))
        if kind == 'tuple':
            a = self.expr(e[1], env, out)
            b = self.expr(e[2], env, out)
            return Val(f'[{a.code}, {b.code}]', ('tuple', a.t, b.t), parts=(a, b), cond=a.cond or b.cond)
        if kind in ('fst', 'snd'):
            v = self.expr(e[1], env, out)
            if v.t[0] != 'tuple':
                raise TypeError(f'{kind} of {show(v.t)}')
            i = 0 if kind == 'fst' else 1
            if v.parts and not v.parts[1 - i].cond:
                return v.parts[i]   # the other part is not read, and observes nothing
            return Val(f'{self.operand(v, ATOM)}[{i}]', v.t[1 + i], cond=v.cond)
        if kind == 'not':
            v = self.expr(e[1], env, out)
            same(v.t, BOOL, '!')
            return Val(f'!{self.operand(v, NOT)}', BOOL, NOT, cond=v.cond)
        if kind == 'shift':
            v = self.expr(e[2], env, out)
            if v.t[0] != 'int':
                raise TypeError(f'{e[1]} of {show(v.t)}')
            k, w = e[3], v.t[1]
            if e[1] == '<<':
                if w + k > EXACT_BITS:
                    raise NotImplementedError(f'int({w}) << {k} can pass 2^{EXACT_BITS}')
                return Val(f'{self.operand(v, MUL)} * {2 ** k} % {2 ** w}', v.t, MUL, cond=v.cond)
            return Val(f'Math.floor({v.code} / {2 ** k})', v.t, cond=v.cond)
        if kind == 'nth_bit':
            # bit i of an int(w) counted from the most significant, as Dice's
            # nth_bit numbers it: x's bit w - 1 - i
            w, i = e[1], e[2]
            v = self.expr(e[3], env, out)
            if v.t != ('int', w):
                raise TypeError(f'nth_bit(int({w}, {i})) of {show(v.t)}')
            if not 0 <= i < w:
                raise ValueError(f'nth_bit(int({w}, {i})): no bit {i} of {w}')
            k = 2 ** (w - 1 - i)
            x = f'Math.floor({v.code} / {k})' if k > 1 else self.operand(v, MUL)
            return Val(f'{x} % 2 == 1', BOOL, EQ, cond=v.cond)
        if kind == 'bin' and e[1] in ('&&', '||'):
            # a chain of one operator, which the parser nests to the left as
            # deep as it is long, walked down its left side; past CHAIN
            # operands one call of all or any over the array of them, whose
            # elements are all evaluated, so that no observation needs a var
            op, rights = e[1], []
            while e[0] == 'bin' and e[1] == op:
                rights.append(e[3])
                e = e[2]
            if len(rights) < CHAIN:
                v = self.expr(e, env, out)
                for r in reversed(rights):
                    v = self.binary(op, v, self.expr(r, env, out), out)
                return v
            vals = [self.expr(e, env, out)] + [self.expr(r, env, out) for r in reversed(rights)]
            for v in vals:
                same(v.t, BOOL, op)
            f = 'all' if op == '&&' else 'any'
            return Val(f'{f}(function(x) {{ return x; }}, [{", ".join(v.code for v in vals)}])', BOOL,
                       cond=any(v.cond for v in vals))
        if kind == 'bin':
            a = self.expr(e[2], env, out)
            b = self.expr(e[3], env, out)
            return self.binary(e[1], a, b, out)
        if kind == 'observe':
            v = self.expr(e[1], env, out)
            same(v.t, BOOL, 'observe')
            if v.code != 'true':
                out.append(f'condition({v.code});')
            return Val('true', BOOL)
        if kind == 'if':
            return self.ite(e, env, out, name)
        if kind == 'call':
            fname, args = e[1], [self.expr(x, env, out) for x in e[2]]
            js, params, ret, obs = self.signature(fname)
            if len(params) != len(args):
                raise TypeError(f'{fname}: {len(params)} parameters, {len(args)} arguments')
            for p, a in zip(params, args):
                same(p, a.t, fname)
            if ret is None:
                raise TypeError(f'{fname}: a recursive function needs a return type')
            return Val(f'{js}({", ".join(a.code for a in args)})', ret, cond=obs or any(a.cond for a in args))
        if kind == 'iterate':
            fname, init, k = e[1], self.expr(e[2], env, out), e[3]
            js, params, ret, obs = self.signature(fname)
            if len(params) != 1:
                raise TypeError(f'iterate: {fname} takes {len(params)} arguments')
            same(params[0], init.t, f'iterate({fname})')
            same(ret, init.t, f'iterate({fname})')
            self.iterate = True
            return Val(f'iterate({js}, {init.code}, {k})', init.t, cond=obs or init.cond)
        raise ValueError(f'unknown expression {kind}')

    def binary(self, op, a, b, out):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op in ('&&', '||'):
            same(a.t, BOOL, op)
            same(b.t, BOOL, op)
            if b.cond:   # Dice evaluates both operands; JavaScript the second one only where the first does not decide
                b = self.bind(b, out, None, 'v')
            p = AND if op == '&&' else OR
            return Val(f'{self.operand(a, p)} {op} {self.operand(b, p + 1)}', BOOL, p, cond=a.cond)
        cond = a.cond or b.cond
        if a.t == BOOL and op in ('^', '<=>', '==', '!='):
            same(b.t, BOOL, op)
            js = '!=' if op in ('^', '!=') else '=='
            return Val(f'{self.operand(a, EQ)} {js} {self.operand(b, EQ + 1)}', BOOL, EQ, cond=cond)
        if a.t[0] != 'int':
            raise TypeError(f'{op} on {show(a.t)}')
        same(a.t, b.t, op)
        w = a.t[1]
        m = 2 ** w
        if op == '+':
            return Val(f'({self.operand(a, ADD)} + {self.operand(b, ADD + 1)}) % {m}', a.t, MUL, cond=cond)
        if op == '-':
            return Val(f'({self.operand(a, ADD)} - {self.operand(b, ADD + 1)} + {m}) % {m}', a.t, MUL, cond=cond)
        if op == '*':
            if 2 * w > EXACT_BITS:
                raise NotImplementedError(f'a product of int({w})s can pass 2^{EXACT_BITS}')
            return Val(f'{self.operand(a, MUL)} * {self.operand(b, MUL + 1)} % {m}', a.t, MUL, cond=cond)
        if op in ('==', '!='):
            return Val(f'{self.operand(a, EQ)} {op} {self.operand(b, EQ + 1)}', BOOL, EQ, cond=cond)
        if op in ('<', '>', '<=', '>='):
            return Val(f'{self.operand(a, CMP)} {op} {self.operand(b, CMP + 1)}', BOOL, CMP, cond=cond)
        raise ValueError(f'unknown operator {op}')

    def ite(self, e, env, out, name):
        c = self.expr(e[1], env, out)
        same(c.t, BOOL, 'if')
        if c.code in ('true', 'false'):
            return self.expr(e[2] if c.code == 'true' else e[3], env, out, name)
        then, other = [], []
        a = self.expr(e[2], env, then)
        b = self.expr(e[3], env, other)
        t = same(a.t, b.t, 'if')
        if not then and not other:
            # an else-if chain stays flat (?: groups to the right), a line a branch where it is long
            head = f'{self.operand(c, OR)} ? {self.operand(a, OR)}'
            other_code = self.operand(b, COND)
            sep = '\n    : ' if '\n' in other_code or len(head) + len(other_code) > 100 else ' : '
            return Val(head + sep + other_code, t, COND, cond=c.cond or a.cond or b.cond)
        lines = (['(function() {', f'  if ({c.code}) {{'] + indented(then + [f'return {a.code};'], 4)
                 + ['  } else {'] + indented(other + [f'return {b.code};'], 4) + ['  }', '})()'])
        return self.bind(Val('\n'.join(lines), t), out, name, 'r')

    def signature(self, fname):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        return self.funs[fname]

    def function(self, fname, params, types, ret, body):
        if None in types:
            raise TypeError(f'{fname}: a parameter without a type')
        types = [width(t) for t in types]
        self.used, self.next = self.globals, self.gnext
        js = self.name(fname)
        # signed before its body, which may call it
        self.funs[fname] = (js, types, width(ret), observes(body, {**self.funs, fname: (js, types, ret, False)}))
        self.scope()
        args = [self.name(p) for p in params]
        out = []
        v = self.expr(body, {p: Val(a, ty) for p, a, ty in zip(params, args, types)}, out)
        if ret is not None:
            same(ret, v.t, fname)
        self.funs[fname] = (js, types, v.t, self.funs[fname][3])
        self.defs.append('\n'.join([f'var {js} = function({", ".join(args)}) {{']
                                   + indented(out + [f'return {v.code};'], 2) + ['};']))

    def components(self, v, out):
        """The leaves of value v, left to right, as Vals each read once: a
        tuple that is neither written in place nor a variable is bound to
        one first."""
        if v.t[0] != 'tuple':
            return [v]
        if v.parts:
            return self.components(v.parts[0], out) + self.components(v.parts[1], out)
        if not READ.fullmatch(v.code):
            v = self.bind(v, out, None, 'value')
        return self.components(Val(f'{v.code}[0]', v.t[1]), out) + self.components(Val(f'{v.code}[1]', v.t[2]), out)

    def translate(self, text, source, joint):
        parser = Parser(tokenize(text), nth_bit=True)
        funs, body = parser.program()
        for fname, params, ret, fbody in funs:
            self.function(fname, params, parser.param_types[fname], ret, fbody)
        self.scope()
        out = []
        v = self.expr(body, {}, out)
        types = leaves(v.t)
        if joint:
            if any(t != BOOL for t in types):
                raise TypeError('joint: a value of Booleans')
            n = len(types)
            v = Val(' + '.join(f'({self.operand(c, OR)} ? {2 ** (n - 1 - i)} : 0)'
                               for i, c in enumerate(self.components(v, out))), ('int', n), ADD)
            types = [width(('int', n))]
        model = '\n'.join(['var model = function() {'] + indented(out + [f'return {v.code};'], 2) + ['};'])
        helper = ['var iterate = function(f, x, k) {\n  return k == 0 ? x : iterate(f, f(x), k - 1);\n};'] \
            if self.iterate else []
        head = [f"// {source}, translated by tddlog's scripts/dice2webppl.py."] + \
            [f'// component {k} ' + ('bool' if t == BOOL else f'int {t[1]}') for k, t in enumerate(types)]
        return '\n'.join(head) + '\n\n' + '\n\n'.join(helper + self.defs + [model]) + \
            "\n\nInfer({method: 'enumerate'}, model);\n"


def leaves(t):
    """The types of the components of a value of type t, left to right."""
    return leaves(t[1]) + leaves(t[2]) if t[0] == 'tuple' else [t]


def translate(text, source='a Dice program', joint=False):
    """The WebPPL program of a Dice program; with joint, its value, a tuple
    of Booleans, as the int whose bits they are."""
    return deep(lambda: Translator().translate(text, source, joint))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('program')
    ap.add_argument('--joint', action='store_true', help='the value, a tuple of Booleans, as one int whose bits '
                    'they are (the most significant first): their joint distribution')
    a = ap.parse_args()
    with open(a.program) as f:
        text = f.read()
    try:
        program = translate(text, os.path.basename(a.program), a.joint)
    except NotImplementedError as err:
        refuse(err)
    sys.stdout.write(program)


if __name__ == '__main__':
    main()
