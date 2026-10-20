#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into SGCL, the language of Genfer (Zaiser, Murawski and Ong, NeurIPS 2023;
github.com/fzaiser/genfer), which infers with generating functions.

The program is lowered by dice_ir.py. SGCL is imperative over natural
numbers: a Boolean is a variable that is 0 or 1, an int(w) one from 0 to
2^w - 1. `flip p` is `x ~ Bernoulli(p)`, `discrete` `Categorical`,
`uniform(w, lo, hi)` `UniformDisc(lo, hi)`. A Boolean expression is an
SGCL event (`x = 1 && y = 0`, `x < y`) where it is read once, by an `if`
or an `observe` (as Genfer's own programs write them), and a variable set
by `if EVENT { x := 1; } else { x := 0; }` where it is read more. SGCL's
assignments are affine (`x := 2*y + 1`, `x += y`, `x -= 3`): + and a
product with a constant are assignments, reduced modulo 2^w by a chain of
`if x >= k*2^w { x -= k*2^w; }`; -, a product of two variables and >>
split on the values of one operand. `observe e` is `observe EVENT;`, a
conditional an `if`, `iterate(f, x, k)` `loop k { ... }` over the
function's inlined body.

Genfer's time grows exponentially with the number of program variables
(its README), so the variables are registers: a value lives in one from
its definition to its last read, after which the register is set to 0
(which marginalizes it) and reused.

Genfer returns the distribution of one variable, so the program's value
is asked one component at a time, as dice2psi.py does: `dice2genfer.py
program.dice K` writes the program whose value is the K-th component
(counted from 0, left to right through nested tuples), with only the
statements it and the observations need; `--components` prints each
component's type (`bool` or `int W`). Run it with `--limit 2^W` (2 for a
Boolean) for the masses of all its values. With `--joint` a value of N
Booleans is one component, an int N whose most significant bit is the
first Boolean: the joint distribution a circuit's `joint` program asks.
A negation is written `(!(E))`, since Genfer reads a `!` only where no
space precedes it.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions.

Usage: dice2genfer.py program.dice K [--joint] > program.sgcl
       dice2genfer.py program.dice --components [--joint]
"""
import argparse
import heapq
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice_ir import (BOOL, BOOLEAN, Const, Var, args_of, decimal, deep, leaves, lower, prune, refuse,  # noqa: E402
                    size, uses)

EVENT_ARGS = {'not', 'and', 'or'}   # operators whose operands are read as events


def event_reads(stmts, count):
    """Adds to count every read of a variable as an event: an observation,
    the condition of an if or an ite, an operand of !, && and ||."""
    for s in stmts:
        if s[0] == 'observe':
            xs = [s[1]]
        elif s[0] == 'if':
            xs = [s[1]]
            event_reads(s[2], count)
            event_reads(s[3], count)
        elif s[0] == 'loop':
            xs = []
            event_reads(s[4], count)
        elif s[0] == 'op' and s[2] in EVENT_ARGS:
            xs = s[3]
        elif s[0] == 'op' and s[2] == 'ite':
            xs = s[3][:1]
        else:
            xs = []
        for x in xs:
            if isinstance(x, Var):
                count[x] = count.get(x, 0) + 1
    return count


def ifelse(cases, last):
    """`if E1 { B1 } else if ... else { last }` for cases (E, B), or last
    alone when there are none."""
    return ' else '.join([f'if {e} {{ {b} }}' for e, b in cases] + [f'{{ {last} }}']) if cases else last


class Emitter:
    def __init__(self, stmts, result):
        self.defs = {}
        self.collect(stmts)
        n = uses(stmts, {})
        if isinstance(result, Var):
            n[result] = n.get(result, 0) + 1
        ev = event_reads(stmts, {})
        # A Boolean operation read once, as an event, is written as one.
        self.inline = {v for v, s in self.defs.items()
                       if s[0] == 'op' and s[2] in BOOLEAN and n.get(v) == 1 and ev.get(v) == 1}
        self.first, self.last = {}, {}
        self.pos = 0
        self.walk(stmts)
        if isinstance(result, Var):
            self.last[result] = float('inf')
        self.allocate()
        self.result = result

    def collect(self, stmts):
        for s in stmts:
            if s[0] in ('sample', 'op'):
                self.defs[s[1]] = s
            elif s[0] == 'if':
                self.collect(s[2])
                self.collect(s[3])
            elif s[0] == 'loop':
                self.collect(s[4])

    # --- live ranges -----------------------------------------------------------

    def reads(self, xs):
        """The variables that values xs read, an inlined operation read
        through its operands."""
        out = []
        for x in xs:
            if isinstance(x, Var):
                if x in self.inline:
                    out += self.reads(args_of(self.defs[x]))
                else:
                    out.append(x)
        return out

    def use(self, xs):
        for x in self.reads(xs):
            self.last[x] = max(self.last.get(x, self.pos), self.pos)

    def define(self, v):
        self.first.setdefault(v, self.pos)
        self.last.setdefault(v, self.pos)

    def walk(self, stmts):
        """Positions in program order: a value's live range runs from its
        first definition to its last read."""
        for s in stmts:
            kind = s[0]
            if kind == 'op' and s[1] in self.inline:
                continue
            if kind in ('sample', 'op', 'observe'):
                self.use(args_of(s))
                if kind != 'observe':
                    self.define(s[1])
                self.pos += 1
            elif kind == 'if':
                _, c, then, other, outs = s
                self.use([c])
                self.pos += 1
                for branch, side in ((then, 1), (other, 2)):
                    self.walk(branch)
                    for o in outs:
                        self.use([o[side]])
                        self.define(o[0])
                    self.pos += 1
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                self.use(inits)
                for v in states:
                    self.define(v)
                self.pos += 1
                start = self.pos
                self.walk(body)
                self.use(nexts)
                # the states, and what the body reads from before it, live
                # through the loop
                for v in states:
                    self.last[v] = max(self.last[v], self.pos)
                for v, p in self.first.items():
                    if p < start <= self.last[v]:
                        self.last[v] = max(self.last[v], self.pos)
                self.pos += 1

    def allocate(self):
        """Registers by linear scan: a register is free for a value that
        starts after the last read of the one it held."""
        self.reg, self.kills = {}, {}
        busy, free, count = [], [], 0
        for v in sorted(self.first, key=lambda v: (self.first[v], v.n)):
            start = self.first[v]
            while busy and busy[0][0] < start:
                heapq.heappush(free, heapq.heappop(busy)[1])
            if free:
                r = heapq.heappop(free)
            else:
                r, count = count, count + 1
            self.reg[v] = f'x{r}'
            heapq.heappush(busy, (self.last[v], r))
            if self.last[v] != float('inf'):
                self.kills.setdefault(self.last[v], []).append(self.reg[v])
        self.registers = count

    # --- code ----------------------------------------------------------------

    def val(self, x):
        return str(x.k) if isinstance(x, Const) else self.reg[x]

    def event(self, x):
        """x, a Boolean, as an SGCL event."""
        if isinstance(x, Const):
            return '0 = 0' if x.k else '0 = 1'
        if x in self.inline:
            s = self.defs[x]
            return self.op_event(s[2], s[3])
        return f'{self.reg[x]} = 1'

    def chain(self, op, xs):
        """The operands of a chain of an associative operator, inlined
        operations of the same operator flattened."""
        out = []
        for x in xs:
            if isinstance(x, Var) and x in self.inline and self.defs[x][2] == op:
                out += self.chain(op, self.defs[x][3])
            else:
                out.append(x)
        return out

    def negation(self, x):
        """Whether x's event prints as a negation, parenthesized already."""
        return isinstance(x, Var) and x in self.inline and self.defs[x][2] == 'not'

    def op_event(self, op, args):
        if op == 'not':
            return f'(!({self.event(args[0])}))'
        if op in ('and', 'or'):
            sep = ' && ' if op == 'and' else ' || '
            return sep.join(self.event(x) if self.negation(x) else f'({self.event(x)})'
                            for x in self.chain(op, args))
        a, b = (self.val(x) for x in args)
        return {'xor': f'{a} != {b}', 'iff': f'{a} = {b}', 'eq': f'{a} = {b}', 'ne': f'{a} != {b}',
                'lt': f'{a} < {b}', 'le': f'{a} <= {b}'}[op]

    def cases(self, x, body):
        """`if x = v { body(v) } else if ...` over the values x takes."""
        vs = x.support
        return ifelse([(f'{self.val(x)} = {v}', body(v)) for v in vs[:-1]], body(vs[-1]))

    def reduce(self, r, top, m):
        """r, at most top, modulo m."""
        if top < m:
            return []
        return [' else '.join(f'if {r} >= {q * m} {{ {r} -= {q * m}; }}' for q in range(top // m, 0, -1))]

    def times(self, r, c, a):
        if c == 0:
            return f'{r} := 0;'
        return f'{r} := {self.val(a)};' if c == 1 else f'{r} := {c}*{self.val(a)};'

    def op_code(self, v, op, args):
        r = self.reg[v]
        if op in BOOLEAN:
            return [f'if {self.op_event(op, args)} {{ {r} := 1; }} else {{ {r} := 0; }}']
        if op == 'copy':
            return [f'{r} := {self.val(args[0])};']
        if op == 'ite':
            c, a, b = args
            return [f'if {self.event(c)} {{ {r} := {self.val(a)}; }} else {{ {r} := {self.val(b)}; }}']
        m = size(v.t)
        a = args[0]
        if op == 'shl':
            c = 2 ** args[1]
            return [self.times(r, c, a)] + self.reduce(r, c * max(a.support), m)
        if op == 'shr':
            k = args[1]
            groups = {}
            for x in a.support:
                groups.setdefault(x >> k, []).append(x)
            keys = sorted(groups)
            return [ifelse([(f'{self.val(a)} in [{", ".join(map(str, groups[j]))}]', f'{r} := {j};')
                            for j in keys[:-1]], f'{r} := {keys[-1]};')]
        b = args[1]
        if op == 'add':
            if isinstance(a, Const):
                a, b = b, a
            top = max(a.support) + max(b.support)
            if isinstance(b, Const):
                return [f'{r} := {self.val(a)} + {b.k};'] + self.reduce(r, top, m)
            return [f'{r} := {self.val(a)};', f'{r} += {self.val(b)};'] + self.reduce(r, top, m)
        if op == 'sub':
            if isinstance(b, Const):
                d = (m - b.k) % m
                return [f'{r} := {self.val(a)} + {d};'] + self.reduce(r, max(a.support) + d, m)
            if isinstance(a, Const):
                return [self.cases(b, lambda y: f'{r} := {(a.k - y) % m};')]

            def shifted(y):
                d = (m - y) % m
                return f'{r} := {self.val(a)};' if d == 0 else f'{r} := {self.val(a)} + {d};'
            return [self.cases(b, shifted)] + self.reduce(r, max(a.support) + max((m - y) % m for y in b.support), m)
        if op == 'mul':
            if isinstance(a, Const) or (isinstance(b, Var) and len(a.support) < len(b.support)):
                a, b = b, a
            top = max(a.support) * max(b.support)
            if isinstance(b, Const):
                return [self.times(r, b.k, a)] + self.reduce(r, top, m)
            return [self.cases(b, lambda y: self.times(r, y, a))] + self.reduce(r, top, m)
        raise ValueError(f'unknown operator {op}')

    def emit(self, stmts, out, depth):
        pad = '    ' * depth
        for s in stmts:
            kind = s[0]
            if kind == 'op' and s[1] in self.inline:
                continue
            if kind == 'sample':
                _, v, dist, p = s
                if dist == 'flip':
                    code = [f'{self.reg[v]} ~ Bernoulli({decimal(p)});']
                elif dist == 'discrete':
                    code = [f'{self.reg[v]} ~ Categorical({", ".join(decimal(q) for q in p)});']
                else:
                    code = [f'{self.reg[v]} ~ UniformDisc({p[0]}, {p[1]});']
                out += [pad + c for c in code]
                self.step(out, pad)
            elif kind == 'op':
                out += [pad + c for c in self.op_code(s[1], s[2], s[3])]
                self.step(out, pad)
            elif kind == 'observe':
                out.append(f'{pad}observe {self.event(s[1])};')
                self.step(out, pad)
            elif kind == 'if':
                _, c, then, other, outs = s
                out.append(f'{pad}if {self.event(c)} {{')
                self.step(out, pad)
                for branch, side in ((then, 1), (other, 2)):
                    if side == 2:
                        out.append(f'{pad}}} else {{')
                    self.emit(branch, out, depth + 1)
                    out += [f'{pad}    {self.reg[o[0]]} := {self.val(o[side])};' for o in outs]
                    self.step(out, pad + '    ')
                out.append(f'{pad}}}')
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                out += [f'{pad}{self.reg[v]} := {self.val(x)};' for v, x in zip(states, inits)]
                self.step(out, pad)
                out.append(f'{pad}loop {k} {{')
                self.emit(body, out, depth + 1)
                out += [f'{pad}    {self.reg[v]} := {self.val(x)};' for v, x in zip(states, nexts)
                        if not (isinstance(x, Var) and self.reg.get(x) == self.reg[v])]
                kills = self.kills.get(self.pos, [])
                self.pos += 1
                out.append(f'{pad}}}')
                # the values that end at the loop's end are set to 0 after it
                out += [f'{pad}{r} := 0;' for r in kills]

    def step(self, out, pad):
        """Past one position: the registers whose values end there are set
        to 0, which marginalizes them."""
        out += [f'{pad}{r} := 0;' for r in self.kills.get(self.pos, [])]
        self.pos += 1

    def program(self, stmts):
        out = []
        self.pos = 0
        self.emit(stmts, out, 0)
        if isinstance(self.result, Const):
            out += [f'ret := {self.result.k};', 'return ret']
        else:
            out.append(f'return {self.reg[self.result]}')
        return out


def translate(text, k, source='a Dice program', joint=False):
    """The SGCL program that returns component k of a Dice program's value,
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
        e = Emitter(kept, result)
        head = [f'# {source}, translated by tddlog\'s scripts/dice2genfer.py, component {k} '
                f'({"bool" if types[k] == BOOL else f"int({types[k][1]})"}); {e.registers} variables.']
        return '\n'.join(head + e.program(kept)) + '\n', types
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
