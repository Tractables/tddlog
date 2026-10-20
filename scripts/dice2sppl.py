#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a program of SPPL (Saad, Rinard and Mansinghka, PLDI 2021;
github.com/probsys/sppl), whose compiler builds a sum-product expression.

The program is lowered by dice_ir.py. In SPPL a variable is defined once,
by a sample (`x ~= bernoulli(p=0.3)`, `discrete({0: 0.5, 1: 0.5})`,
`randint(low=0, high=10)`, `atomic(loc=k)` for a constant) or as a
transform of one other variable (`x ~= y`); an `if` conditions on an
event, which is a set of values of one variable (`y << {1, 2}`) or a
conjunction of such (`(a << {1}) & (b << {0})`); `condition(EVENT)`
observes one. A Boolean is 0 or 1. A Boolean expression that is a
conjunction (or the negation of one) of such sets is written as the event
where it is read once by an `if` or an `observe`, else as
`if EVENT: x ~= atomic(loc=1) else: x ~= atomic(loc=0)`. An operation of
two variables (+, -, *, comparisons, ^) has no event or transform in SPPL,
so it is a switch on the values of one operand and, in each case, on the
sets of values of the other that give each result (`atomic(loc=r)`). A
Dice conditional is an `if` whose branches define the same variables (a
variable one branch does not need is `atomic(loc=0)` there), a chain of
`else if` an `elif` chain; `iterate(f, x, k)` is unrolled (SPPL's own
`for` unrolls too).

The program's value is asked by sppl_dice.py, which compiles the program
once and asks `prob` for each value of each component, each a single
marginal; a comment line `# component K ...` names each component's
variable (or constant) and the values it can take.
With `--joint` (a tuple of Booleans asked jointly, as a circuit's `joint`
program is) a `# joint` line asks sppl_dice.py for the probability of
each assignment of the components, as the int whose bits they are (the
first the most significant): SPPL has no sum of two variables, so the
int itself would be a switch on its 2^N values.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions.

Usage: dice2sppl.py program.dice [--joint] > program.sppl
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice_ir import (BOOL, BOOLEAN, Const, Var, apply, args_of, decimal, deep, joint_bits, leaves,  # noqa: E402
                     lower, prune, refuse, uses)

EVENT_ARGS = {'not', 'and', 'or'}


def event_reads(stmts, count):
    for s in stmts:
        xs = []
        if s[0] == 'observe':
            xs = [s[1]]
        elif s[0] == 'if':
            xs = [s[1]]
            event_reads(s[2], count)
            event_reads(s[3], count)
        elif s[0] == 'loop':
            event_reads(s[4], count)
        elif s[0] == 'op' and s[2] in EVENT_ARGS:
            xs = s[3]
        elif s[0] == 'op' and s[2] == 'ite':
            xs = s[3][:1]
        for x in xs:
            if isinstance(x, Var):
                count[x] = count.get(x, 0) + 1
    return count


def literal_op(s):
    """Whether operation s is a set of values of one variable: a comparison
    with a constant."""
    return s[2] in ('eq', 'ne', 'lt', 'le') and sum(isinstance(x, Const) for x in s[3]) == 1


class Emitter:
    def __init__(self, stmts, inline):
        self.inline = inline
        self.env = {}       # Var: its SPPL name, or a Const
        # inlined Var: (positive, {name: values}) of this unrolling, until
        # its one reader takes it (and may change its lits)
        self.events = {}
        self.dummies = 0
        self.supports = {}  # SPPL name: the values it takes
        self.holds = set()  # the observations inside branches, 1 where they hold
        # inlined events whose readers cannot take their form: to be variables
        # (the lines emitted with them are discarded)
        self.unformable = set()

    # --- events --------------------------------------------------------------
    # An event is (positive, lits): the conjunction of `name << values` for
    # lits, or (positive false) its negation.

    def val(self, x):
        if isinstance(x, Const):
            return x
        return self.env[x]

    def event(self, x):
        if isinstance(x, Const):
            return (bool(x.k), {})
        if x in self.inline:
            return self.events.pop(x)
        return (True, {self.env[x]: frozenset([1])})

    def support(self, name):
        return self.supports[name]

    def form(self, positive, e, x):
        """Event e as a conjunction (positive) or a negated one."""
        pol, lits = e
        if pol == positive:
            return lits
        if not lits:
            raise ValueError('a constant operand of && or ||')   # folded by dice_ir
        if len(lits) == 1:
            (name, values), = lits.items()
            return {name: frozenset(self.support(name)) - values}
        self.unformable.add(x)
        return lits

    def op_event(self, v, op, args):
        """The event of a Boolean operation (v unformable if it has none)."""
        if op == 'not':
            pol, lits = self.event(args[0])
            return (not pol, lits)
        if op in ('and', 'or'):
            positive = op == 'and'
            lits = {}
            for x in args:
                more = self.form(positive, self.event(x), x)
                if len(more) > len(lits):   # the smaller merged into the larger
                    lits, more = more, lits
                for name, values in more.items():
                    lits[name] = lits[name] & values if name in lits else values
            return (positive, lits)
        if op in ('eq', 'ne', 'lt', 'le') and sum(isinstance(x, Const) for x in args) == 1:
            x = next(a for a in args if isinstance(a, Var))
            name = self.env[x]
            values = frozenset(k for k in x.support
                               if apply(op, x.t, [k if a is x else a.k for a in args]))
            return (True, {name: values})
        self.unformable.add(v)
        return (True, {})

    def render(self, lits):
        if not lits:
            return 'True'
        terms = [f'({name} << {{{", ".join(map(str, sorted(values)))}}})' for name, values in sorted(lits.items())]
        # SPPL's compiler writes each & in parentheses, and Python refuses
        # more than 200 nested: a balanced tree nests them log-deep
        while len(terms) > 2:
            terms = [f'({terms[i]} & {terms[i + 1]})' if i + 1 < len(terms) else terms[i]
                     for i in range(0, len(terms), 2)]
        return ' & '.join(terms)

    def test(self, e):
        pol, lits = e
        return self.render(lits) if pol else f'~({self.render(lits)})'

    # --- statements ----------------------------------------------------------

    def name(self, v, suffix):
        n = f'v{v.n}{suffix}'
        self.env[v] = n
        self.supports[n] = v.support
        return n

    def assign(self, name, x):
        """name ~= the value x (a constant or a variable)."""
        x = self.val(x)
        return f'{name} ~= atomic(loc={x.k})' if isinstance(x, Const) else f'{name} ~= {x}'

    def table(self, v, op, args, suffix):
        """v = op(args) by a switch on the values of the variable operands."""
        n = self.name(v, suffix)
        xs = [a for a in args if isinstance(a, Var)]
        if op in ('add', 'sub', 'mul', 'eq', 'ne', 'lt', 'le'):
            t = args[0].t
        else:
            t = v.t

        def f(vals):
            it = iter(vals)
            return apply(op, t, [next(it) if isinstance(a, Var) else (a if isinstance(a, int) else a.k)
                                 for a in args])
        if len(xs) == 1:
            return self.switch(xs[0], lambda k: f([k]), n)
        x, y = sorted(xs, key=lambda a: len(a.support))
        swap = x is not xs[0]
        cases = [(frozenset([a]), self.switch(y, lambda b, a=a: f([b, a] if swap else [a, b]), n))
                 for a in x.support]
        return self.chain(self.env[x], cases)

    def switch(self, x, f, n):
        """n ~= f(x) by the sets of x's values with each result."""
        groups = {}
        for k in x.support:
            groups.setdefault(f(k), []).append(k)
        if len(groups) == 1:
            return [f'{n} ~= atomic(loc={next(iter(groups))})']
        return self.chain(self.env[x], [(frozenset(ks), [f'{n} ~= atomic(loc={r})']) for r, ks in groups.items()])

    def chain(self, name, cases):
        """if name << s1: ... elif ...: ... else: ..., or the one case's
        body (SPPL's if needs an else)."""
        if len(cases) == 1:
            return cases[0][1]
        out = []
        for i, (values, body) in enumerate(cases):
            if i == 0:
                out.append(f'if {self.render({name: values})}:')
            elif i < len(cases) - 1:
                out.append(f'elif {self.render({name: values})}:')
            else:
                out.append('else:')
            out += ['    ' + line for line in body]
        return out

    def emit(self, stmts, suffix, depth):
        """(lines, names defined), depth the number of enclosing ifs."""
        out, names = [], []
        for s in stmts:
            kind = s[0]
            if kind == 'sample':
                _, v, dist, p = s
                n = self.name(v, suffix)
                names.append(n)
                if dist == 'flip':
                    out.append(f'{n} ~= bernoulli(p={decimal(p)})')
                elif dist == 'discrete':
                    terms = ', '.join(f'{k}: {decimal(q)}' for k, q in enumerate(p) if q > 0)
                    out.append(f'{n} ~= discrete({{{terms}}})')
                else:
                    out.append(f'{n} ~= randint(low={p[0]}, high={p[1]})')
            elif kind == 'op':
                _, v, op, args = s
                if op == 'copy':
                    self.env[v] = self.val(args[0])
                    continue
                if v in self.inline:
                    self.events[v] = self.op_event(v, op, args)
                    continue
                if op == 'ite':
                    c, a, b = args
                    n = self.name(v, suffix)
                    names.append(n)
                    pol, lits = self.event(c)
                    if not pol:
                        a, b = b, a
                    out += [f'if {self.render(lits)}:', '    ' + self.assign(n, a), 'else:', '    ' + self.assign(n, b)]
                    continue
                if op in ('not', 'and', 'or') or op in BOOLEAN and literal_op(s):
                    pol, lits = self.op_event(v, op, args)
                    n = self.name(v, suffix)
                    names.append(n)
                    one, zero = (1, 0) if pol else (0, 1)
                    out += [f'if {self.render(lits)}:', f'    {n} ~= atomic(loc={one})',
                            'else:', f'    {n} ~= atomic(loc={zero})']
                    continue
                out += self.table(v, op, args, suffix)
                names.append(self.env[v])
            elif kind == 'observe':
                if depth == 0:
                    out.append(f'condition({self.test(self.event(s[1]))})')
                    continue
                # SPPL's condition in a branch would renormalize the branch:
                # the observation is a variable, 1 where it holds and on the
                # paths that do not take the branch, conditioned on after
                # the outermost if
                self.dummies += 1
                n = f'ok{self.dummies}'
                self.supports[n] = (0, 1)
                self.holds.add(n)
                pol, lits = self.event(s[1])
                one, zero = (1, 0) if pol else (0, 1)
                out += [f'if {self.render(lits)}:', f'    {n} ~= atomic(loc={one})',
                        'else:', f'    {n} ~= atomic(loc={zero})']
                names.append(n)
            elif kind == 'if':
                lines, defined = self.emit_if(s, suffix, depth)
                out += lines
                names += defined
                if depth == 0:
                    out += [f'condition({n} << {{1}})' for n in defined if n in self.holds]
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                current = [self.val(x) for x in inits]
                for i in range(k + 1):
                    for j, (v, x) in enumerate(zip(states, current)):
                        if isinstance(x, Const):   # a state's operations read a variable
                            n = self.name(v, f'{suffix}_{i}s{j}')
                            out.append(f'{n} ~= atomic(loc={x.k})')
                            names.append(n)
                        else:
                            self.env[v] = x
                    if i == k:
                        break
                    lines, defined = self.emit(body, f'{suffix}_{i}', depth)
                    out += lines
                    names += defined
                    current = [self.val(x) for x in nexts]
        return out, names

    def branches(self, s, values, suffix, depth):
        """The branches of if s and of the ifs chained in its else, as
        (event, lines, names), each ending with values[v] (of the outs of
        the chain's first if) assigned to v."""
        _, c, then, other, outs = s
        lines, names = self.emit(then, suffix, depth + 1)
        here = {o[0]: o[1] for o in outs}
        there = {o[0]: o[2] for o in outs}

        def resolve(x, side):
            while isinstance(x, Var) and x in side:
                x = side[x]
            return x
        first_values = {v: resolve(x, here) for v, x in values.items()}
        else_values = {v: resolve(x, there) for v, x in values.items()}
        lines = lines + [self.assign(self.env[v], x) for v, x in first_values.items()]
        out = [(self.event(c), lines, names)]
        # an else that is an if, after the operations its condition inlines
        if other and other[-1][0] == 'if' and all(t[0] == 'op' and t[1] in self.inline for t in other[:-1]):
            self.emit(other[:-1], suffix, depth + 1)   # their events, no lines
            return out + self.branches(other[-1], else_values, suffix, depth)
        lines, names = self.emit(other, suffix, depth + 1)
        lines = lines + [self.assign(self.env[v], x) for v, x in else_values.items()]
        return out + [(None, lines, names)]

    def emit_if(self, s, suffix, depth):
        outs = s[4]
        names = [self.name(o[0], suffix) for o in outs]
        bs = self.branches(s, {o[0]: o[0] for o in outs}, suffix, depth)
        if len(bs) == 2 and not bs[0][0][0]:
            # if not C: A else: B is if C: B else: A
            (e, a, an), (_, b, bn) = bs
            bs = [((True, e[1]), b, bn), (None, a, an)]
        defined = set(names)
        for _, _, ns in bs:
            defined.update(ns)
        out = []
        for i, (e, lines, ns) in enumerate(bs):
            if not lines:
                self.dummies += 1
                d = f'skip{self.dummies}'
                self.supports[d] = (0,)
                lines = [f'{d} ~= atomic(loc=0)']
                ns = ns + [d]
                defined.add(d)
            bs[i] = (e, lines, ns)
        for i, (e, lines, ns) in enumerate(bs):
            pad = sorted(defined - set(ns) - set(names))
            body = lines + [f'{n} ~= atomic(loc={int(n in self.holds)})' for n in pad]
            if e is None:
                out.append('else:')
            else:
                out.append(f'{"if" if i == 0 else "elif"} {self.test(e)}:')
            out += ['    ' + line for line in body]
        return out, sorted(defined)


def inlined(stmts, value):
    """The Boolean operations read once, as an event."""
    n = uses(stmts, {})
    for x in leaves(value):
        if isinstance(x, Var):
            n[x] = n.get(x, 0) + 1
    ev = event_reads(stmts, {})
    defs = {}

    def collect(ss):
        for s in ss:
            if s[0] == 'op':
                defs[s[1]] = s
            elif s[0] == 'if':
                collect(s[2])
                collect(s[3])
            elif s[0] == 'loop':
                collect(s[4])
    collect(stmts)
    return {v for v, s in defs.items()
            if (s[2] in EVENT_ARGS or literal_op(s)) and n.get(v) == 1 and ev.get(v) == 1}


def translate(text, source='a Dice program', joint=False):
    """The SPPL program of a Dice program, with `# component` lines."""
    def run():
        stmts, value = lower(text)
        comps = joint_bits(value) if joint else leaves(value)
        kept = prune(stmts, {x for x in comps if isinstance(x, Var)})
        inline = inlined(kept, value)
        while True:
            e = Emitter(kept, inline)
            lines, _ = e.emit(kept, '', 0)
            if not e.unformable:
                break
            assert e.unformable <= inline
            inline = inline - e.unformable
        head = [f'# {source}, translated by tddlog\'s scripts/dice2sppl.py.']
        if joint:
            head.append('# joint')
        for k, x in enumerate(comps):
            ty = 'bool' if x.t == BOOL else f'int {x.t[1]}'
            got = e.val(x)
            if isinstance(got, Const):
                head.append(f'# component {k} {ty} const {got.k}')
            else:
                head.append(f'# component {k} {ty} {got} {",".join(map(str, e.supports[got]))}')
        if not lines:
            lines = ['skip0 ~= atomic(loc=0)']
        return '\n'.join(head + lines) + '\n'
    return deep(run)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('program')
    ap.add_argument('--joint', action='store_true', help='the value, a tuple of Booleans, as one int whose bits '
                    'they are (the most significant first): their joint distribution')
    a = ap.parse_args()
    with open(a.program) as f:
        text = f.read()
    try:
        sys.stdout.write(translate(text, os.path.basename(a.program), a.joint))
    except NotImplementedError as err:
        refuse(err)


if __name__ == '__main__':
    main()
