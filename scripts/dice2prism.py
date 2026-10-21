#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a discrete-time Markov chain in the PRISM language, which the model
checkers PRISM (Kwiatkowska, Norman and Parker, CAV 2011) and Storm (Hensel,
Junges, Katoen, Quatmann and Volk, STTT 2022) read, with the reachability
properties that give the distribution of the program's value.

The program is lowered by dice_ir.py: functions inlined, tuples resolved,
constants folded. A discrete program without unbounded loops is a chain
that walks through its computation: one module, a program counter `pc`,
and the program's values as the module's variables, a Boolean a `bool`, an
int(w) a number in [0..2^w-1] whose +, - and * are taken modulo 2^w as
Dice's (`mod`), << a product and >> the `floor` of a quotient. A `flip`,
`discrete` or `uniform` is a probabilistic choice of a command, and the
pure operations after it are computed in the same command, the choice's
outcome substituted in each of its updates, up to the next choice; so the
chain has a step only where something random happens (and where an
expression would grow beyond 64 operations, whose operands are then
stored). Where a condition (of an `if`, or an `observe`) is known from a
choice's outcome, each outcome jumps to its side; where it is a function
of the state, the command is split into one guarded by each case (`[] pc=3
& (x = 2) -> ...`, a network's conditional table a command per row)
before the choice, and after it such a condition starts the next step, so
that the cases of two conditions do not multiply.
`observe e` sends the states where e is false to an absorbing state, and
the end of the program is an absorbing state labelled "done", where only
the value is kept. `iterate(f, x, k)` is the inlined body of f as a loop
with a counter, which has the states of the unrolled program in k times
fewer commands. The probabilities are written exactly: a decimal as the
shortest one that reads back as the program's double, the outcomes of a
choice summing to one in rationals (Storm's exact engine checks it).

What the checkers' time grows with is the number of states, so a value is
in the state only while it is live: a variable is reset to false or 0 after
the last read of its value, and states that differ only in dead values are
one. The variables are registers, used in turn by values whose live ranges
do not meet. As one writes a chain by hand, a sample is taken just before
its first reader and an observation is checked right after what it reads
is computed (the order of independent samples and of conditioning does not
change the distribution).

The answer to value v of the component asked is P(F "done" & r=v) / P(F
"done") for the variable r that holds it at the end: a property for each
value the component can take (for a Boolean only true) and one, `obs`, for
P(F "done"), left out where nothing is observed (every run is then done). The
properties are unbounded reachability; the chain is acyclic but for its
two absorbing states. dice2prism.py writes the properties to the file
`--props` names (one a line, named `"obs"` and `"vN"`: PRISM refuses a
property named as a label) and lists them in the model's header.

The value is asked one component at a time, as dice2psi.py does:
`dice2prism.py program.dice K` writes the model whose value is the K-th
component (counted from 0, left to right through nested tuples), with only
the statements it and the observations need; `--components` prints each
component's type (`bool` or `int W`). With `--joint` a value of N Booleans
is one component, an int(N) whose most significant bit is the first
Boolean: the joint distribution a circuit's `joint` program asks.

Supported: what dice2problog.py supports except `/`, `%`, `::`, lists, and
recursive functions (the chain has no stack), and integers beyond PRISM's
32 bits. A program that is not supported exits with status 3.

Usage: dice2prism.py program.dice K [--joint] [--props FILE] > program.prism
       dice2prism.py program.dice --components [--joint]
"""
import argparse
import heapq
import os
import re
import sys
from decimal import Decimal, localcontext
from fractions import Fraction

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice2problog import bits_for  # noqa: E402
from dice_ir import BOOL, Const, Lower, Var, deep, leaves, lower, prune, refuse, size  # noqa: E402

EXPRESSION = 64       # the largest expression a command computes before its operands are stored
GROUPS = 256          # the most commands, one per guard, that a step is split into
INT_MAX = 2 ** 31 - 1  # PRISM's integers are Java's
DONE = -1             # the position of the end, absorbing
ATOM = re.compile(r'[A-Za-z_][A-Za-z0-9_]*|[0-9]+')


# --- the program, in the order the chain takes it ---------------------------

def reads(s):
    """The variables statement s reads, in its branches and body too."""
    kind = s[0]
    if kind == 'op':
        return {x for x in s[3] if isinstance(x, Var)}
    if kind == 'observe':
        return {s[1]} if isinstance(s[1], Var) else set()
    if kind == 'if':
        out = {s[1]} | {x for o in s[4] for x in o[1:] if isinstance(x, Var)}
        for t in s[2] + s[3]:
            out |= reads(t)
        return out
    if kind == 'loop':
        out = {x for x in list(s[3]) + list(s[5]) if isinstance(x, Var)}
        for t in s[4]:
            out |= reads(t)
        return out
    return set()


def defines(s):
    """The variables statement s defines for the statements after it."""
    if s[0] in ('sample', 'op'):
        return (s[1],)
    if s[0] == 'if':
        return tuple(o[0] for o in s[4])
    if s[0] == 'loop':
        return tuple(s[2])
    return ()


def reorder(stmts):
    """A block's statements, and its branches' and loop bodies', with each
    observation right after the statement that defines what it reads and
    each sample just before its first reader in the block (at the block's
    end if none reads it); the others keep their order."""
    stmts = [('if', s[1], reorder(s[2]), reorder(s[3]), s[4]) if s[0] == 'if' else
             ('loop', s[1], s[2], s[3], reorder(s[4]), s[5]) if s[0] == 'loop' else s for s in stmts]
    front, body, after, where = [], [], {}, {}
    for s in stmts:
        if s[0] == 'observe':
            j = where.get(s[1])
            (front if j is None else after.setdefault(j, [])).append(s)
        else:
            for v in defines(s):
                where[v] = len(body)
            body.append(s)
    order = list(front)
    for j, s in enumerate(body):
        order.append(s)
        order += after.get(j, [])
    first = {}
    for j, s in enumerate(order):
        if s[0] != 'sample':
            for v in reads(s):
                first.setdefault(v, j)
    before, last = {}, []
    for s in order:
        if s[0] == 'sample':
            j = first.get(s[1])
            (last if j is None else before.setdefault(j, [])).append(s)
    out = []
    for j, s in enumerate(order):
        if s[0] != 'sample':
            out += before.get(j, [])
            out.append(s)
    return out + last


class Flat:
    """The statements as a list of instructions with forward jumps and loop
    back edges: ('sample', v, dist, p), ('op', v, OP, args), ('observe', a),
    ('branch', c, then, else), ('jump', to) and ('end',). The outs of an if
    are copies at the end of each side; a loop of k > 1 rounds counts them
    in a variable of its own and branches back to the body's start."""

    def __init__(self, stmts, result):
        self.code, self.result, self.counters = [], result, 0
        self.emit(stmts)
        self.code.append(('end',))

    def counter(self, t, support):
        self.counters += 1
        return Var(10 ** 9 + self.counters, t, support)

    def emit(self, stmts):
        code = self.code
        for s in stmts:
            kind = s[0]
            if kind == 'if':
                _, c, then, other, outs = s
                at = len(code)
                code.append(None)
                self.emit(then)
                code += [('op', v, 'copy', [a]) for v, a, _ in outs]
                jump = len(code)
                code.append(None)
                self.emit(other)
                code += [('op', v, 'copy', [b]) for v, _, b in outs]
                code[at] = ('branch', c, at + 1, jump + 1)
                code[jump] = ('jump', len(code))
            elif kind == 'loop':
                _, k, states, inits, body, nexts = s
                code += [('op', v, 'copy', [x]) for v, x in zip(states, inits)]
                if k == 1:
                    self.emit(body)
                    code += [('op', v, 'copy', [x]) for v, x in zip(states, nexts)]
                    continue
                t = ('int', bits_for(k + 1))
                i, more = self.counter(t, range(k + 1)), self.counter(BOOL, (0, 1))
                code.append(('op', i, 'copy', [Const(t, 0)]))
                head = len(code)
                self.emit(body)
                code += [('op', v, 'copy', [x]) for v, x in zip(states, nexts)]
                code.append(('op', i, 'add', [i, Const(t, 1)]))
                code.append(('op', more, 'lt', [i, Const(t, k)]))
                code.append(('branch', more, head, len(code) + 1))
            else:
                code.append(s)

    def liveness(self):
        """The variables live before each instruction, a tuple each: a
        backward pass, repeated while a loop's back edge saw a set that has
        grown since."""
        code, n = self.code, len(self.code)
        live = [()] * n
        end = (self.result,) if isinstance(self.result, Var) else ()
        while True:
            seen, cur = {}, set()
            for i in range(n - 1, -1, -1):
                s = code[i]
                kind = s[0]
                if kind == 'end':
                    cur = set(end)
                elif kind == 'jump':
                    cur = set(live[s[1]])
                elif kind == 'branch':
                    for t in s[2:]:
                        if t <= i:
                            seen[t] = live[t]
                    cur = set(live[s[2]]) | set(live[s[3]])
                    cur.add(s[1])
                else:
                    if kind in ('sample', 'op'):
                        cur.discard(s[1])
                    if kind == 'op':
                        cur.update(x for x in s[3] if isinstance(x, Var))
                    elif kind == 'observe' and isinstance(s[1], Var):
                        cur.add(s[1])
                live[i] = tuple(cur)
            if all(set(v) == set(live[t]) for t, v in seen.items()):
                return live


def outcomes(s):
    """The values of a sample and their probabilities, exact: a decimal as
    the shortest one that reads back as the program's double, the outcomes
    of a discrete summing to one (the largest takes up the rounding)."""
    _, v, dist, p = s
    if dist == 'flip':
        f = Fraction(repr(float(p)))
        return [(1, f), (0, 1 - f)]
    if dist == 'uniform':
        lo, hi = p
        return [(k, Fraction(1, hi - lo)) for k in range(lo, hi)]
    fs = [Fraction(repr(float(q))) for q in p]
    gap = 1 - sum(fs)
    if abs(gap) > Fraction(1, 10 ** 4):
        raise ValueError(f'discrete probabilities that sum to {float(1 - gap)}')
    fs[max(range(len(fs)), key=lambda k: fs[k])] += gap
    return [(k, f) for k, f in enumerate(fs) if f > 0]


def probability(f):
    """A probability as PRISM and Storm read it exactly: a decimal, or a
    fraction of two integers where it has no finite decimal."""
    n, d = f.numerator, f.denominator
    m = d
    for q in (2, 5):
        while m % q == 0:
            m //= q
    if m == 1:
        with localcontext() as c:
            c.prec = 4000
            return format(Decimal(n) / Decimal(d), 'f')
    if d > INT_MAX:
        raise NotImplementedError(f'the probability {f} has no decimal and a denominator beyond PRISM\'s integers')
    return f'{n}/{d}'


# --- the chain's commands ----------------------------------------------------

class Fold(Lower):
    """dice_ir's operations folded where an operand decides them, each made
    once (equal operations on equal operands are one value), with their
    definitions: the values a command computes from its state."""

    def __init__(self):
        super().__init__()
        self.n = 2 * 10 ** 9
        self.memo, self.defs = {}, {}

    def op(self, out, op, t, args):
        key = (op, t, tuple(args))
        v = self.memo.get(key)
        if v is None:
            x = args[0]
            if op == 'not' and isinstance(x, Var) and self.defs.get(x, ('',))[0] == 'not':
                v = self.defs[x][1][0]
            else:
                mark = len(out)
                v = super().op(out, op, t, args)
                for s in out[mark:]:
                    self.defs[s[1]] = (s[2], s[3])
            self.memo[key] = v
        return v

    def __call__(self, op, t, args):
        return self.op([], op, t, args)


class Path:
    """The outcomes of a step that are at one instruction: their
    probabilities and values ({Var: value}; a variable not there is the
    state's). It has sampled once it has made the step's choice, and moved
    once it has run an instruction."""
    __slots__ = ('pos', 'branches', 'sampled', 'moved')

    def __init__(self, pos, branches, sampled, moved):
        self.pos, self.branches, self.sampled, self.moved = pos, branches, sampled, moved

    def copy(self):
        return Path(self.pos, [(p, dict(env)) for p, env in self.branches], self.sampled, self.moved)


class Group:
    """A command: the paths of the states its guard ({condition: truth})
    holds in, the mass that failed an observation, and the outcomes that
    ended, (probability, the position they end at, the values live there)."""
    __slots__ = ('guard', 'paths', 'fail', 'out')

    def __init__(self, guard, paths, fail=Fraction(0), out=()):
        self.guard, self.paths, self.fail, self.out = guard, paths, fail, list(out)

    def copy(self):
        return Group(dict(self.guard), [p.copy() for p in self.paths], self.fail, self.out)


class Chain:
    def __init__(self, flat):
        self.code, self.result = flat.code, flat.result
        self.live = flat.liveness()
        self.fold = Fold()
        self.sizes, self.texts = {}, {}
        self.steps = {}
        todo = [0]
        while todo:
            t = todo.pop()
            if t not in self.steps:
                self.steps[t] = self.step_commands(t)
                todo += [u for g in self.steps[t] for _, u, _ in g.out if u != DONE and u not in self.steps]
        self.observes = any(s[0] == 'observe' for s in self.code)
        self.allocate()

    def live_at(self, t):
        if t == DONE:
            return (self.result,) if isinstance(self.result, Var) else ()
        return self.live[t]

    # --- one step: the commands from a position ---------------------------

    def step_commands(self, start):
        """The commands of the step at position start: the program run from
        there on the state's values, a choice made at most once, each path
        ended where it meets a second choice, the end, a jump back or a
        limit; a condition that the state decides splits the command."""
        stack = [Group({}, [Path(start, [(Fraction(1), {})], False, False)])]
        done = []
        while stack:
            g = stack.pop()
            room = len(done) + len(stack) + 2 <= GROUPS
            c = self.run(g, room)
            if c is None:
                done.append(g)
                continue
            h = g.copy()
            h.guard[c] = True
            g.guard[c] = False
            stack += [g, h]
        return done

    def run(self, g, room):
        """Runs g's paths until they end, or returns a condition to split g on."""
        k = 0
        while k < len(g.paths):
            path = g.paths[k]
            if path.pos is None:
                k += 1
                continue
            c = self.step(g, path, room)
            if c is not None:
                return c
        return None

    def val(self, env, x):
        return env.get(x, x) if isinstance(x, Var) else x

    def known(self, g, x):
        """x's truth under g's guard, None where the guard does not decide it."""
        neg = False
        while True:
            if isinstance(x, Const):
                return bool(x.k) != neg
            if x in g.guard:
                return g.guard[x] != neg
            d = self.fold.defs.get(x)
            if d is None or d[0] != 'not':
                return None
            x, neg = d[1][0], not neg

    def atom(self, x):
        while self.fold.defs.get(x, ('',))[0] == 'not':
            x = self.fold.defs[x][1][0]
        return x

    def size(self, x):
        """The number of operations in x's expression, written as a tree."""
        if not isinstance(x, Var) or x not in self.fold.defs:
            return 1
        s = self.sizes.get(x)
        if s is None:
            s = self.sizes[x] = 1 + sum(self.size(a) for a in self.fold.defs[x][1] if not isinstance(a, int))
        return s

    def finish(self, g, path, target):
        # the live values the path assigned; the others keep their registers
        live = self.live_at(target)
        g.out += [(p, target, {v: env[v] for v in live if v in env}) for p, env in path.branches]
        path.pos, path.branches = None, []

    def step(self, g, path, room):
        """Runs path's instruction: None once it is run or the path has
        ended, else the condition to split g on."""
        i = path.pos
        s = self.code[i]
        kind = s[0]
        if kind == 'end':
            self.finish(g, path, DONE)
            return None
        if kind == 'jump':
            path.pos = s[1]
            return None
        if kind == 'sample':
            if path.sampled:
                self.finish(g, path, i)
                return None
            (_, env), = path.branches
            v = s[1]
            path.branches = [(q, {**env, v: Const(v.t, k)}) for k, q in outcomes(s)]
            path.sampled = path.moved = True
            path.pos = i + 1
            return None
        if kind == 'op':
            _, v, op, args = s
            xs = [self.fold(op, v.t, [self.val(env, a) for a in args]) for _, env in path.branches]
            if path.moved and any(self.size(x) > EXPRESSION for x in xs):
                self.finish(g, path, i)
                return None
            for (_, env), x in zip(path.branches, xs):
                env[v] = x
            path.moved = True
            path.pos = i + 1
            return None
        # an observation or a branch: its condition in each outcome
        conds = [self.val(env, s[1]) for _, env in path.branches]
        truth = [self.known(g, c) for c in conds]
        if None in truth:
            # a condition of the state splits the command before the choice
            # (a table's rows); after it, the condition starts the next step,
            # so that the cases of two conditions do not multiply
            if path.sampled or (path.moved and not room):
                self.finish(g, path, i)
                return None
            return self.atom(conds[truth.index(None)])
        path.moved = True
        if kind == 'observe':
            g.fail += sum(p for (p, _), t in zip(path.branches, truth) if not t)
            path.branches = [b for b, t in zip(path.branches, truth) if t]
            path.pos = i + 1 if path.branches else None
            return None
        yes = [b for b, t in zip(path.branches, truth) if t]
        no = [b for b, t in zip(path.branches, truth) if not t]
        if yes and no:
            other = Path(s[3], no, path.sampled, True)
            g.paths.append(other)
            self.back(g, other, i)
            path.branches, path.pos = yes, s[2]
        else:
            path.pos = s[2] if yes else s[3]
        self.back(g, path, i)
        return None

    def back(self, g, path, i):
        """A path that jumps back, to the start of a loop's body, ends there."""
        if path.pos is not None and path.pos <= i:
            self.finish(g, path, path.pos)

    # --- registers ------------------------------------------------------------

    def allocate(self):
        """A register for each value live at a step, by a linear scan over
        the steps in program order: two values share one where their ranges
        from the first to the last step they are live at do not meet."""
        order = sorted(self.steps)
        self.pc = {t: k for k, t in enumerate(order)}
        self.pc[DONE] = len(order)
        self.fail = len(order) + 1
        first, last = {}, {}
        for t in order + [DONE]:
            k = self.pc[t]
            for v in self.live_at(t):
                first.setdefault(v, k)
                last[v] = k
        self.regs, self.reg = [], {}
        free, busy = {}, []
        for v in sorted(first, key=lambda v: (first[v], v.n)):
            while busy and busy[0][0] < first[v]:
                _, r = heapq.heappop(busy)
                heapq.heappush(free.setdefault(self.regs[r], []), r)
            if free.get(v.t):
                r = heapq.heappop(free[v.t])
            else:
                if v.t != BOOL and v.t[1] > 30:
                    raise NotImplementedError(f'an int({v.t[1]}), beyond PRISM\'s 32-bit integers')
                r = len(self.regs)
                self.regs.append(v.t)
            self.reg[v] = r
            heapq.heappush(busy, (last[v], r))
        self.contents = {}

    def content(self, t):
        """{register: the value it holds} at position t."""
        c = self.contents.get(t)
        if c is None:
            c = self.contents[t] = {self.reg[v]: v for v in self.live_at(t)}
        return c

    # --- text -----------------------------------------------------------------

    def expr(self, x):
        if isinstance(x, Const):
            return ('true' if x.k else 'false') if x.t == BOOL else str(x.k)
        d = self.fold.defs.get(x)
        if d is None:
            return f'x{self.reg[x]}'
        s = self.texts.get(x)
        if s is None:
            s = self.texts[x] = self.op_text(x, *d)
        return s

    def neg(self, e):
        return f'!{e}' if ATOM.fullmatch(e) or e.startswith('(') else f'!({e})'

    def op_text(self, v, op, args):
        a = [None if isinstance(x, int) else self.expr(x) for x in args]
        if op == 'not':
            return self.neg(a[0])
        if op == 'copy':
            return a[0]
        if op == 'ite':
            return f'({a[0]} ? {a[1]} : {a[2]})'
        if op in ('and', 'or', 'iff', 'xor', 'eq', 'ne', 'lt', 'le'):
            # Booleans compared with = and != (Storm's parser has no <=>)
            sym = {'and': '&', 'or': '|', 'iff': '=', 'xor': '!=', 'eq': '=', 'ne': '!=', 'lt': '<', 'le': '<='}[op]
            return f'({a[0]} {sym} {a[1]})'
        m = size(v.t)
        x = args[0]
        if op == 'shr':
            return f'floor({a[0]} / {2 ** args[1]})'
        if op == 'shl':
            y, b = Const(v.t, 2 ** args[1]), str(2 ** args[1])
        else:
            y, b = args[1], a[1]
        # PRISM checks each update in every state its guard admits, reachable
        # or not, so mod is left out only where no value of the operands'
        # ranges wraps: a variable's is [0..m-1], not its reachable support
        if op == 'sub':
            if isinstance(x, Const) and isinstance(y, Const) and x.k >= y.k:
                return f'({a[0]} - {b})'
            return f'mod({a[0]} - {b} + {m}, {m})'
        sym = '+' if op == 'add' else '*'
        top = max(x.support) + max(y.support) if op == 'add' else max(x.support) * max(y.support)
        if top > INT_MAX:
            raise NotImplementedError(f'a product up to {top}, beyond PRISM\'s 32-bit integers')
        hx, hy = (z.k if isinstance(z, Const) else m - 1 for z in (x, y))
        return f'({a[0]} {sym} {b})' if (hx + hy if op == 'add' else hx * hy) < m else f'mod({a[0]} {sym} {b}, {m})'

    def update(self, start, target, values):
        cur, nxt = self.content(start), self.content(target)
        parts = []
        for r in sorted(cur.keys() | nxt.keys()):
            w = nxt.get(r)
            if w is None:
                parts.append(f"(x{r}'={self.reset(r)})")
            else:
                e = self.expr(values.get(w, w))
                if e != f'x{r}':
                    parts.append(f"(x{r}'={e})")
        return '&'.join(parts + [f"(pc'={self.pc[target]})"])

    def reset(self, r):
        return 'false' if self.regs[r] == BOOL else '0'

    def commands(self, start):
        failed = '&'.join([f"(x{r}'={self.reset(r)})" for r in sorted(self.content(start))] + [f"(pc'={self.fail})"])
        out = []
        for g in self.steps[start]:
            ups = {}
            for p, target, values in g.out:
                u = self.update(start, target, values)
                ups[u] = ups.get(u, 0) + p
            if g.fail:
                ups[failed] = ups.get(failed, 0) + g.fail
            assert sum(ups.values()) == 1, (start, sum(ups.values()))
            guard = ''.join(f' & {self.expr(c) if t else self.neg(self.expr(c))}' for c, t in g.guard.items())
            if len(ups) == 1:
                rhs = next(iter(ups))
            else:
                rhs = ' + '.join(f'{probability(p)}:{u}' for u, p in ups.items())
            out.append(f'  [] pc={self.pc[start]}{guard} -> {rhs};')
        return out

    def properties(self):
        """(name, property): P(done) where something is observed, and for each
        value of the result the probability that the run is done with it."""
        out = [('obs', 'P=? [F "done"]')] if self.observes else []
        r = self.result
        if isinstance(r, Const):
            if r.t != BOOL or r.k:
                out.append((f'v{r.k}', 'P=? [F "done"]'))
        elif r.t == BOOL:
            if 1 in r.support:
                out.append(('v1', f'P=? [F ("done" & x{self.reg[r]})]'))
        else:
            out += [(f'v{k}', f'P=? [F ("done" & x{self.reg[r]}={k})]') for k in r.support]
        return out

    def model(self, head):
        top = self.fail if self.observes else self.pc[DONE]
        lines = [f'// {line}' for line in head] + ['dtmc', '', 'module program', f'  pc : [0..{top}] init 0;']
        for r, t in enumerate(self.regs):
            lines.append(f'  x{r} : bool init false;' if t == BOOL else f'  x{r} : [0..{size(t) - 1}] init 0;')
        for t in sorted(self.steps):
            lines += self.commands(t)
        lines.append(f'  [] pc={self.pc[DONE]} -> true;')
        if self.observes:
            lines.append(f'  [] pc={self.fail} -> true;')
        lines += ['endmodule', '', f'label "done" = pc={self.pc[DONE]};']
        return '\n'.join(lines) + '\n'


def translate(text, k, source='a Dice program', joint=False):
    """(the PRISM model of component k of a Dice program's value, its
    properties as the text of a properties file, the types of all the
    components: `BOOL` or ('int', w))."""
    def run():
        stmts, value = lower(text, joint)
        comps = leaves(value)
        types = [x.t for x in comps]
        if k is None:
            return None, None, types
        if not 0 <= k < len(comps):
            raise ValueError(f'component {k} of {len(comps)}')
        result = comps[k]
        kept = reorder(prune(stmts, {result} if isinstance(result, Var) else set()))
        chain = Chain(Flat(kept, result))
        props = chain.properties()
        commands = sum(len(g) for g in chain.steps.values())
        head = [f'{source}, translated by tddlog\'s scripts/dice2prism.py, component {k} '
                f'({"bool" if types[k] == BOOL else f"int({types[k][1]})"}): {len(chain.steps)} steps, '
                f'{len(chain.regs)} registers, {commands} commands.'] + \
            [f'"{name}": {prop}' for name, prop in props]
        return chain.model(head), ''.join(f'"{name}": {prop};\n' for name, prop in props), types
    return deep(run)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('program')
    ap.add_argument('component', nargs='?', type=int)
    ap.add_argument('--components', action='store_true', help="print each component's type instead")
    ap.add_argument('--joint', action='store_true', help='the value, a tuple of Booleans, as one int whose bits '
                    'they are (the most significant first): their joint distribution')
    ap.add_argument('--props', help='write the properties to this file')
    a = ap.parse_args()
    if a.components == (a.component is not None):
        raise SystemExit('give a component K, or --components')
    if a.props and a.components:
        raise SystemExit('--props writes the properties of a component K')
    with open(a.program) as f:
        text = f.read()
    try:
        model, props, types = translate(text, a.component, os.path.basename(a.program), a.joint)
    except NotImplementedError as err:
        refuse(err)
    if a.components:
        for t in types:
            print('bool' if t == BOOL else f'int {t[1]}')
        return
    if a.props:
        with open(a.props, 'w') as f:
            f.write(props)
    sys.stdout.write(model)


if __name__ == '__main__':
    main()
