#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a Julia script for Alea.jl (https://github.com/Tractables/Alea.jl), the
Julia successor of Dice, which compiles a program to a CUDD BDD.

The program is read by dice2problog.py's parser and written as Julia over
Alea's distributions: a Boolean is a `Dist{Bool}` (a Julia `Bool` when it
is constant), an int(w) a `DistUInt{w}`, a tuple a Julia tuple. `flip p` is
`flip(p)`, `discrete(ps)` is `discrete(DistUInt{w}, ps)` with Dice's width
w, `uniform(w, lo, hi)` is `uniform(DistUInt{w}, lo, hi)`, `int(w, k)` is
`DistUInt{w}(k)`. `&&`, `||` and `!` are `&`, `|` and `!`, `^` is `xor`,
`<=>`, `==` and `!=` are `prob_equals` and its negation; `<`, `<=`, `>`
and `>=` compare unsigned integers; `+`, `-`, `*` and `<<` are Alea's,
which outside its `@alea` macro wrap modulo 2^w as Dice's do; `>>` shifts
a `DistUInt`'s bits (Alea has no `>>` for it).

Expressions are evaluated in the program's order, so Alea creates the
flips, which order its BDD's variables, in the order Dice does. `if c then
a else b` evaluates both branches and merges them with `ifelse`, except
that a branch whose condition is the constant false is not evaluated (a
literal condition when translating, a Julia `Bool` when running), as in
dice2problog.py. `observe e` conjoins `!path | e` to the evidence, with
`path` the conjunction of the conditions of the branches it is in, which
is what Alea's `@alea` macro does with an `observe`; the macro itself is
not used, since under it Alea's `+`, `-` and `*` raise an error on
overflow rather than wrap. A Dice function is a Julia function of the path
and its arguments, `iterate(f, x, k)` a loop of k calls, and a function
that calls itself is cut at Dice's recursion limit as in dice2problog.py:
the call past it observes false and returns its type's default value.

The program's value is queried in one call of `pr`, given the evidence,
one distribution per component of a returned tuple. The script prints
`alea K VALUE PROBABILITY` for every value of the K-th component (true and
false, or 0 to 2^w - 1), then `alea_seconds T`, the time the process took
to build the program and run `pr`, and `alea_compile_seconds T`, Julia's
compilation within that time. Before it, the script loads Alea and runs a
small program through `pr`, so that the time leaves out Julia's start and
the compilation of Alea's common methods. The work runs on a task with a
1 GiB stack: Alea compiles its expression graph and computes probabilities
by recursion as deep as a loop is long.

The script is a module whose own functions Julia interprets rather than
compiles (`@compiler_options compile=min optimize=0 infer=false`); Alea's
methods compile as usual. Julia's optimizer spends its time on long
straight-line functions: on bn_pathfinder it took 184 of the 191 seconds
of building the program and `pr`, and interpreted the two take 10 s, with
the same answers. A long body (digitRecognition's 7 830 observations, a
large network's tables) is also split into functions of at most about
CHUNK statements, each taking the variables it reads and returning those
read after it, since lowering a function takes time that grows faster
than its length.

Supported: what dice2problog.py supports. Not supported: lists, /, %,
nth_bit, sample.

Usage: dice2alea.py program.dice > program.jl
"""
from collections import Counter
import math
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
from dice2problog import (Parser, RECURSION_LIMIT, bits_for, calls, deep,  # noqa: E402
                          discrete_probs, tokenize)

CHUNK = 400
# Every local variable the translation makes is x<number>_<hint>.
VAR = re.compile(r'\bx\d+_[A-Za-z0-9_]*')
TRIVIAL = re.compile(r'true|false|x\d+_[A-Za-z0-9_]*|DistUInt\{\d+\}\(\d+\)')
# Alea's `discrete` asserts that the probabilities sum to one within
# Julia's default `isapprox` tolerance.
SUM_TOLERANCE = math.sqrt(2.0 ** -52)


def trivial(text):
    """A variable or a constant: evaluating it twice creates nothing."""
    return TRIVIAL.fullmatch(text) is not None


class Line:
    """A statement `lhs = rhs`, or `rhs` alone."""

    def __init__(self, rhs, lhs=None):
        self.rhs, self.lhs = rhs, lhs

    weight = 1

    def blocks(self):
        return []

    def reads(self):
        return Counter(VAR.findall(self.rhs))

    def exposed(self):
        return set(VAR.findall(self.rhs))

    def writes(self):
        return set(VAR.findall(self.lhs)) if self.lhs else set()

    def emit(self, out, indent):
        out.append(indent + (f'{self.lhs} = {self.rhs}' if self.lhs else self.rhs))


class Ite:
    """`if c then a else b`: each branch's statements and value, and the
    variable the merged value goes to. A branch runs unless `c` is the
    constant that rules it out."""

    def __init__(self, cond, then, then_value, other, other_value, result):
        self.cond, self.then, self.then_value = cond, then, then_value
        self.other, self.other_value, self.result = other, other_value, result

    @property
    def weight(self):
        return 1 + weight(self.then) + weight(self.other)

    def blocks(self):
        return [self.then, self.other]

    def reads(self):
        out = Counter(VAR.findall(self.cond))
        for block, value in [(self.then, self.then_value), (self.other, self.other_value)]:
            for s in block:
                out.update(s.reads())
            out.update(VAR.findall(value))
        return out

    def exposed(self):
        return set(VAR.findall(self.cond)) | exposed(self.then + [Line(self.then_value)]) | \
            exposed(self.other + [Line(self.other_value)])

    def writes(self):
        return writes(self.then) | writes(self.other) | {self.result}

    def emit(self, out, indent):
        c, t, e = self.cond, self.then_value, self.other_value
        for block, rule_out in [(self.then, 'false'), (self.other, 'true')]:
            if block:
                out.append(f'{indent}if {c} !== {rule_out}')
                for s in block:
                    s.emit(out, indent + ' ')
                out.append(f'{indent}end')
        out.append(f'{indent}{self.result} = {c} === true ? {t} : {c} === false ? {e} : ifelse({c}, {t}, {e})')


class Loop:
    """`for _ in 1:k; state = call; end`, Dice's `iterate`."""

    def __init__(self, k, state, call):
        self.k, self.state, self.call = k, state, call

    weight = 1

    def blocks(self):
        return []

    def reads(self):
        return Counter(VAR.findall(self.call))

    def exposed(self):
        return set(VAR.findall(self.call))

    def writes(self):
        return {self.state}

    def emit(self, out, indent):
        out.append(f'{indent}for _ in 1:{self.k}')
        out.append(f'{indent} {self.state} = {self.call}')
        out.append(f'{indent}end')


def weight(block):
    return sum(s.weight for s in block)


def writes(block):
    return set().union(*(s.writes() for s in block))


def exposed(block):
    """The variables a block may read before it writes them."""
    written, out = set(), set()
    for s in block:
        out |= s.exposed() - written
        written |= s.writes()
    return out


def number(var):
    return int(re.match(r'x(\d+)_', var).group(1))


def literal(p):
    """A probability as Julia reads the same double."""
    return repr(float(p))


class Translator:
    def __init__(self):
        self.n = 0
        self.chunks = 0
        self.defs = []       # the Julia functions, in order
        self.funs = {}       # Dice name: (Julia name, params, ret, body, recursive)
        self.needs = {}      # Dice name: whether a call needs the caller's path
        self.memo = {}
        # The reads of the function being split, as a Counter, updated in
        # place: `+=` and `-=` drop the counts that are not positive by a
        # pass over the whole Counter each time, which made the translation
        # quadratic in the length of a body (hours on Roulette's programs
        # of 300 000 steps). A count can reach 0, never below; a read is a
        # positive count.
        self.reads = None

    def fresh(self, hint):
        self.n += 1
        return f'x{self.n}_{re.sub(r"[^A-Za-z0-9_]", "_", hint)[:16]}'

    # Whether evaluating an expression observes something, so that a branch
    # it is in must know its path condition.
    def observes(self, e):
        key = id(e)
        if key in self.memo:
            return self.memo[key]
        stack, found = [e], False
        while stack and not found:
            x = stack.pop()
            if isinstance(x, list):
                stack.extend(x)
            elif isinstance(x, tuple) and x:
                if x[0] == 'observe' or (x[0] in ('call', 'iterate') and self.needs.get(x[1], True)):
                    found = True
                stack.extend(y for y in x[1:] if isinstance(y, (tuple, list)))
        self.memo[key] = found
        return found

    def value(self, e, env, path, block, hint):
        """e's value as a variable or a constant, its statements in block."""
        t = self.expr(e, env, path, block, hint)
        if trivial(t):
            return t
        v = self.fresh(hint)
        block.append(Line(t, v))
        return v

    def make(self, text, block, hint):
        """A statement for an expression that creates flips: they are made
        at this point of the program, whatever reads them."""
        v = self.fresh(hint)
        block.append(Line(text, v))
        return v

    def expr(self, e, env, path, block, hint):
        """Julia for expression e under env (Dice names to Julia variables
        and constants), with path the Julia path condition (None where no
        observation can be); its statements are appended to block. An
        expression returned inline creates no flip and observes nothing."""
        if e[0] == 'let':
            env = dict(env)
            while e[0] == 'let':
                env[e[1]] = self.value(e[2], env, path, block, e[1])
                e = e[3]
        kind = e[0]
        if kind == 'var':
            if e[1] not in env:
                raise NameError(f'unbound variable {e[1]}')
            return env[e[1]]
        if kind == 'bool':
            return 'true' if e[1] else 'false'
        if kind == 'flip':
            p = e[1]
            if p <= 0:
                return 'false'
            if p >= 1:
                return 'true'
            return self.make(f'flip({literal(p)})', block, hint)
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            total = math.fsum(ps)
            if not math.isclose(total, 1, rel_tol=SUM_TOLERANCE):
                raise ValueError(f'discrete({", ".join(map(str, e[1]))}) sums to {total}, '
                                 "and Alea's discrete takes a distribution")
            w = bits_for(len(ps))
            return self.make(f'discrete(DistUInt{{{w}}}, [{", ".join(literal(p) for p in ps)}])', block, hint)
        if kind == 'uniform':
            width, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** width:
                raise ValueError(f'uniform({width}, {lo}, {hi}): empty or out of range')
            return self.make(f'uniform(DistUInt{{{width}}}, {lo}, {hi})', block, hint)
        if kind == 'int':
            width, k = e[1:]
            if not 0 <= k < 2 ** width:
                raise ValueError(f'integer constant {k} out of range for {2 ** width} values')
            return f'DistUInt{{{width}}}({k})'
        if kind == 'tuple':
            a = self.expr(e[1], env, path, block, hint)
            b = self.expr(e[2], env, path, block, hint)
            return f'({a}, {b})'
        if kind in ('fst', 'snd'):
            return f'{self.expr(e[1], env, path, block, hint)}[{1 if kind == "fst" else 2}]'
        if kind == 'not':
            t = self.expr(e[1], env, path, block, hint)
            return {'true': 'false', 'false': 'true'}.get(t, f'!({t})')
        if kind == 'shift':
            t = self.expr(e[2], env, path, block, hint)
            return f'({t} << {e[3]})' if e[1] == '<<' else f'shr({t}, {e[3]})'
        if kind == 'bin':
            return self.binary(e[1], self.expr(e[2], env, path, block, hint),
                               self.expr(e[3], env, path, block, hint))
        if kind == 'observe':
            if path is None:
                raise AssertionError('an observation where no path condition was kept')
            c = self.expr(e[1], env, path, block, hint)
            block.append(Line(f'observe!({path}, {c})'))
            return 'true'
        if kind == 'if':
            return self.ite(e, env, path, block, hint)
        if kind == 'call':
            args = [self.expr(x, env, path, block, f'{e[1]}_arg') for x in e[2]]
            return self.make(self.call(e[1], args, path), block, hint)
        if kind == 'iterate':
            state = self.fresh(hint)
            block.append(Line(self.expr(e[2], env, path, block, hint), state))
            block.append(Loop(e[3], state, self.call(e[1], [state], path)))
            return state
        raise ValueError(f'unknown expression {kind}')

    def binary(self, op, a, b):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op == '&&':
            return f'({a} & {b})'
        if op == '||':
            return f'({a} | {b})'
        if op == '^':
            return f'xor({a}, {b})'
        if op in ('<=>', '=='):
            return f'prob_equals({a}, {b})'
        if op == '!=':
            return f'!prob_equals({a}, {b})'
        if op in ('<', '<=', '>', '>=', '+', '-', '*'):
            return f'({a} {op} {b})'
        raise ValueError(f'unknown operator {op}')

    def call(self, fname, args, path):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        jname, params = self.funs[fname][:2]
        if len(params) != len(args):
            raise TypeError(f'{fname}: {len(params)} parameters, {len(args)} arguments')
        if path is None and self.needs[fname]:
            raise AssertionError(f'a call of {fname}, which observes, where no path condition was kept')
        return f'{jname}({", ".join([path or "true"] + args)})'

    def ite(self, e, env, path, block, hint):
        c = self.value(e[1], env, path, block, hint)
        if c == 'true':
            return self.expr(e[2], env, path, block, hint)
        if c == 'false':
            return self.expr(e[3], env, path, block, hint)
        parts = []
        for branch, positive in [(e[2], True), (e[3], False)]:
            inner, inner_path = [], None
            if self.observes(branch):
                cond = c if positive else f'!({c})'
                if path == 'true' and positive:
                    inner_path = c
                else:
                    inner_path = self.fresh('path')
                    inner.append(Line(cond if path == 'true' else f'({path} & {cond})', inner_path))
            parts += [inner, self.value(branch, env, inner_path, inner, hint)]
        result = self.fresh(hint)
        block.append(Ite(c, parts[0], parts[1], parts[2], parts[3], result))
        return result

    # Splitting long bodies into functions.
    def split(self, block):
        """Split `block`, and the blocks in it, into functions of at most
        about CHUNK statements; self.reads holds the reads of the function
        the block is in, kept up to date as statements move out of it."""
        for s in block:
            for inner in s.blocks():
                if weight(inner) > CHUNK:
                    self.split(inner)
        while weight(block) > CHUNK:
            groups, group, w = [], [], 0
            for s in block:
                if group and w + s.weight > CHUNK:
                    groups.append(group)
                    group, w = [], 0
                group.append(s)
                w += s.weight
            groups.append(group)
            block[:] = [g[0] if len(g) == 1 and isinstance(g[0], Line) else self.outline(g) for g in groups]

    def outline(self, group):
        """A function of the statements in group, and the statement that
        calls it."""
        inside = Counter()
        for s in group:
            inside.update(s.reads())
        self.reads.subtract(inside)
        params = sorted(exposed(group), key=number)
        results = sorted((v for v in writes(group) if self.reads[v] > 0), key=number)
        self.chunks += 1
        name = f'chunk_{self.chunks}'
        out = [f'function {name}({", ".join(params)})']
        for s in group:
            s.emit(out, '    ')
        out.append(f'    return ({", ".join(results)},)' if len(results) > 1 else
                   f'    return {results[0] if results else "nothing"}')
        out.append('end')
        self.defs.append('\n'.join(out))
        call = Line(f'{name}({", ".join(params)})',
                    f'({", ".join(results)},)' if len(results) > 1 else (results[0] if results else None))
        self.reads.update(call.reads())
        return call

    def function(self, name, params, body, value):
        """The Julia function `name(params...)`, its body split if long."""
        self.reads = Counter(VAR.findall(value))
        for s in body:
            self.reads.update(s.reads())
        self.split(body)
        out = [f'function {name}({", ".join(params)})']
        for s in body:
            s.emit(out, '    ')
        out.append(f'    return {value}')
        out.append('end')
        self.defs.append('\n'.join(out))

    def default(self, t, fname):
        if t is None:
            raise TypeError(f'{fname}: a recursive function needs a return type')
        if t[0] == 'bool':
            return 'false'
        if t[0] == 'int':
            return f'DistUInt{{{t[1]}}}(0)'
        if t[0] == 'tuple':
            return f'({self.default(t[1], fname)}, {self.default(t[2], fname)})'
        raise NotImplementedError(f'{fname}: default value of {t[0]}')

    def translate(self, text, source):
        funs, body = Parser(tokenize(text)).program()
        taken = set()
        for name, params, ret, fbody in funs:
            jname = 'd_' + re.sub(r'[^A-Za-z0-9_]', '_', name)
            while jname in taken:
                jname += '_'
            taken.add(jname)
            self.funs[name] = (jname, params, ret, fbody, calls(fbody, name))
        # A call needs its caller's path when the function observes, calls
        # one that does, or calls itself (the call past the recursion limit
        # observes false).
        self.needs = {name: f[4] for name, f in self.funs.items()}
        changed = True
        while changed:
            changed = False
            for name, f in self.funs.items():
                if not self.needs[name]:
                    self.memo.clear()
                    if self.observes(f[3]):
                        self.needs[name], changed = True, True
        self.memo.clear()
        for name, (jname, params, ret, fbody, recursive) in self.funs.items():
            path = self.fresh('path')
            args = [self.fresh(p) for p in params]
            block = []
            value = self.value(fbody, dict(zip(params, args)), path, block, name)
            if not recursive:
                self.function(jname, [path] + args, block, value)
                continue
            self.function(jname + '_body', [path] + args, block, value)
            depth = f'DEPTH_{jname}'
            self.defs.append('\n'.join([
                f'const {depth} = Ref(0)',
                f'function {jname}({", ".join([path] + args)})',
                f'    if {depth}[] > {RECURSION_LIMIT}',
                f'        observe!({path}, false)',
                f'        return {self.default(ret, name)}',
                '    end',
                f'    {depth}[] += 1',
                '    try',
                f'        return {jname}_body({", ".join([path] + args)})',
                '    finally',
                f'        {depth}[] -= 1',
                '    end',
                'end']))
        # The program's own path is the constant true.
        block = []
        value = self.value(body, {}, 'true', block, 'result')
        self.function('dice_main', [], block, value)
        return PRELUDE.format(source=source) + '\n\n'.join(self.defs) + '\n' + RUNNER


PRELUDE = '''\
# {source}, translated by tddlog's scripts/dice2alea.py.
module AleaProgram
using Alea
# Julia would spend more time optimizing the long straight-line functions
# below than running them takes: they are interpreted, and Alea's own
# methods compile as usual.
Base.Experimental.@compiler_options compile=min optimize=0 infer=false

const EV = Ref{{AnyBool}}(true)
observe!(path, c) = (EV[] = EV[] & (!path | c); true)

function shr(x::DistUInt{{W}}, n::Integer) where W
    k = min(n, W)
    DistUInt{{W}}(vcat(fill(false, k), x.bits[1:W - k]))
end

components(x::Tuple) = reduce(vcat, map(components, x); init = Any[])
components(x) = Any[x]
width(::DistUInt{{W}}) where W = W

function warmup()
    a = flip(0.7)
    x = discrete(DistUInt{{2}}, [0.1, 0.2, 0.3, 0.4])
    y = uniform(DistUInt{{3}}, 1, 6)
    z = ifelse(a, x + DistUInt{{2}}(1), x - DistUInt{{2}}(1))
    t = ifelse(flip(0.4), (a, z), (!a, x * x))
    e = prob_equals(z, DistUInt{{2}}(2)) | (x < z) | xor(a, flip(0.2))
    pr(flip(0.7))
    pr(a, z, y, t...; evidence = e)
    nothing
end

'''

RUNNER = '''
function report(k, x, d)
    if x isa AnyBool
        println("alea ", k, " true ", d[true])
        println("alea ", k, " false ", d[false])
    else
        for v in 0:(2^width(x) - 1)
            println("alea ", k, " ", v, " ", d[v])
        end
    end
end

function run_program()
    EV[] = true
    stats = @timed begin
        comps = components(dice_main())
        dists = pr(comps...; evidence = EV[])
        (comps, length(comps) == 1 ? [dists] : dists)
    end
    comps, dists = stats.value
    for k in eachindex(comps)
        report(k, comps[k], dists[k])
    end
    println("alea_seconds ", stats.time)
    println("alea_compile_seconds ", get(stats, :compile_time, NaN))
end

warmup()
task = Task(run_program, 1 << 30)
schedule(task)
wait(task)
end
'''


def translate(text, source='a Dice program'):
    """The Alea.jl script for a Dice program; `source` names it in the
    script's first line."""
    return deep(lambda: Translator().translate(text, source))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.stdout.write(translate(open(sys.argv[1]).read(), os.path.basename(sys.argv[1])))
