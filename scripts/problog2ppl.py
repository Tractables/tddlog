#!/usr/bin/env python3
"""Write a ProbLog program of the Datalog fragment in a probabilistic
functional language, first order as it is written: Alea.jl, Roulette,
WebPPL, Psi, Pluck or SPPL, over the program's facts, its rules as loops
(in Pluck, recursions over lists; in SPPL, Python's comprehensions); or in
Dice, which has no relations, its recursions over a step as `iterate`s of
a function.

usage: problog2ppl.py LANG PROGRAM OUT FACTS   writes the program in LANG to
                                               OUT and its facts to FACTS,
                                               and prints the queries, one a
                                               line, in the order the
                                               program answers them
       problog2ppl.py psi PROGRAM OUT          the same, the facts in the
       problog2ppl.py pluck PROGRAM OUT        program (Psi, Pluck, SPPL
       problog2ppl.py sppl PROGRAM OUT         and Dice read no data file
       problog2ppl.py dice PROGRAM OUT         of symbols)
       problog2ppl.py dicemap PROGRAM OUT      a Dice program a query,
                                               STEM__qJ.dice beside OUT
                                               (STEM.dicemap), which lists
                                               them, as ground2net.py's
                                               --format dicemap does

LANG is alea (an Alea.jl program in the form of data/alea, which
alea_bench.jl runs, and whose circuit it writes for tddlog), roulette (a
module of Roulette's `#lang roulette/example/disrupt`), webppl (a WebPPL
program), psi (a Psi program, `psi --dp`), pluck (a Pluck program,
which pluck_dice.jl runs, its queries cK), sppl (an SPPL program, which
sppl_dice.py runs, its queries the events of queries_()) or dice (a Dice
program returning its queries as nested pairs). PROGRAM is a
ProbLog program as bench.py hands the systems that take no arguments (its
consults inlined), read by problog2souffle.py's parse.

The translation keeps the program's structure. A relation is a table from
its tuples of constants to whether each holds: a coin of its own for each
probabilistic fact, drawn once, and true for a certain one; a predicate's
rules add to it, and a tuple that several instances give holds where one
does. The facts are a data file that the program reads, as a database is
loaded, so that a family's program is one text whatever its size: a line
`relation<TAB>probability<TAB>argument...` a fact, the relation named
NAME/ARITY. A rule is a loop over the relations its body joins, in the
order the body is written. An atom whose arguments are all bound is looked
up, an atom with some bound is read through an index of the relation on
those positions (built once the relation is complete), and an atom with
none is scanned. A comparison or `is` filters or binds as soon as its
variables are bound, and a negation reads the relation's tuple. The
predicates are evaluated in the order of their dependencies. A recursive
component (predicates that depend on each other) is evaluated in stages
from no tuple, each stage applying its rules to the stage before, until a
stage adds no tuple and the stages number the component's tuples. In every
world each stage before the least model adds a tuple that holds, so that
stage is the least model, as ProbLog's semantics has it.

A probabilistic rule p::H :- B draws its coin once per ground instance of
the clause, every variable bound (problog2souffle.py's `named`), as ProbLog
does. An annotated disjunction p1::H1; ...; pn::Hn :- B draws one choice per
ground instance, the k-th head chosen with probability pk and none of them
with the rest (`choose`); its heads are evaluated together. In a recursive
component the coins and choices are kept by instance, so that every stage
reads the same ones.

The languages differ in how a tuple's holding is held:
- Alea.jl and Roulette compute symbolically: a holding is a Boolean of the
  coins (Alea's Dist{Bool}, Rosette's symbolic Boolean), and `|`, `&` and
  `!` build it; the queries are asked once the relations are built.
- WebPPL enumerates worlds: a holding is a function of no arguments that
  says whether the tuple holds in the world being enumerated, memoized
  (`mem`) within it, and a conjunction or disjunction asks its operands in
  turn and stops at the first that decides it, so that a world draws only
  the coins its queries and evidence read.
- Psi has arrays and loops but no dictionaries or symbols: a relation is a
  column of integers for each argument and a column of Booleans (`flip`s
  and their `&&` and `||`), a tuple is looked up by a scan, a symbol is a
  number past every integer of the program, and the facts are arrays in
  the program. Its value is the array of the queries' holdings, asked one
  entry a run, since Psi prints the joint distribution of what it returns.
- Pluck is lazy and functional, with algebraic data and recursion and no
  loops or tables: a relation is a list of (tuple, holding) pairs, a tuple
  a constructor of its arity, a symbol or an integer only compared for
  equality (a key) a constructor of its own, an integer only compared, by
  order too, its rank among the integers it is compared with, and one in
  arithmetic one of Pluck's unary naturals, up to Pluck.LIMIT
  (Pluck.kinds); a rule is a
  `concat_map` over the relations its body joins, a tuple is looked up by
  a scan, and a recursive component's stages are a recursion that carries
  the coins of its probabilistic rules by instance. Pluck compiles each
  query (`cK`) lazily into a BDD, given the evidence.
- SPPL's host language is Python, whose values a program may compute
  before its first sample; its probabilistic part samples each variable
  once, of arrays of them, and branches on events of the variables (`if`)
  in loops over a range (`for`). The facts and the joins of the rules are
  Python constants: a rule's matches a list comprehension over the
  relations its body joins, a recursive component's tuples a fixpoint of
  its rules over the tuples that hold in some world, and the tuples the
  queries and the evidence read a fixpoint too. Each probabilistic fact
  and instance of a probabilistic rule samples a coin, and each instance
  of a tuple that is read a variable, 1 where its body (its coin and the
  tuples it reads) or the instance before it of the same tuple holds, the
  last of a tuple's instances the tuple's holding; a recursive component's
  tuples go in cycles, the tuples that read each other, each cycle in as
  many stages as it has tuples, each stage reading the one before. SPPL
  compiles the program into a sum-product expression, conditioned on the
  evidence, and sppl_dice.py asks it each query's event.
- Dice has Booleans, tuples, functions and `iterate`, and no relations,
  data, loops over data or recursion. Its program is written over the
  tuples that hold in some world, computed by the translation (Ground): a
  tuple is a let of its fact's flip or of its instances, an instance its
  coin (a flip, an annotated disjunction's discrete) and its matches. A
  recursive component whose tuples are in layers, an integer argument of
  each relation, each layer read only by itself and the next, is a
  recursion over a step (an HMM's time, a loop written as recursion): a
  function from one layer's tuples to the next's (Chain), its rules the
  instances of a layer with the step left out, a coin of the layer's own
  a flip in it, `iterate`d between the layers the rest of the program
  reads (called at each layer where the layers differ, the difference its
  arguments). Another recursive component, a recursion through data
  (reachability over a graph), is unrolled in stages, as ground2net.py
  unrolls a ground program's (its `fixpoint`). The program returns its
  queries and observes its evidence.

A clause whose head has a variable that its callers bind and its body does
not (ProbLog grounds from the queries: `weather(sun,T) <- T>0, Tprev is
T-1, weather(sun,Tprev).` asked weather(sun,10)) is evaluated for the
arguments it is called with, a relation of them that its body reads first
(`demand`, magic sets); a tuple of it holds in every world. An anonymous
variable in a head is such a variable (`list(N, _) :- elements(N),
ends(N).`), named as no other variable of its clause is.

A program the translation cannot be given exits with status 3 (bench.py's
REFUSED) and the reason; a reason that begins `translator:` is this
translator's limit, not the language's (a query or evidence that is not
ground, a negation through recursion, whose well-founded model the
languages could compute and this translation does not).
"""
from decimal import Decimal
import itertools
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import problog2souffle as p2s  # noqa: E402
from problog2souffle import Refused, REFUSED  # noqa: E402
import ground2net as g2n  # noqa: E402
from ground2net import TRUE, FALSE  # noqa: E402
from dice2problog import bits_for, deep  # noqa: E402

GROUND = ('int', 'sym', 'num')
# a comparison's kind: term (in)equality, arithmetic (in)equality, an order
TESTS = {'=': 'eq', '==': 'eq', '\\=': 'neq', '\\==': 'neq', '=:=': 'aeq', '=\\=': 'aneq',
         '<': '<', '>': '>', '=<': '<=', '>=': '>='}
# the names the back ends give their own variables, which a relation's name must not take
OWN = re.compile(r'[xate]\d+|coins\d+|chosen|instances_.*|size|rels?(_.*)?|(next|stage|stages)_.*|.*_(facts|by(_\d+)*)')


def key(atom):
    return (atom[0], len(atom[1]))


def show_term(t):
    """A term as ProbLog writes it."""
    if t[0] == 'var':
        return t[1]
    if t[0] == 'anon':
        return '_'
    if t[0] == 'sym':
        return p2s.atom_text((t[1], []))
    if t[0] in ('int', 'num'):
        return str(t[1])
    if t[0] == 'neg':
        return f'-{show_term(t[1])}'
    if t[0] == 'op':
        return f'{show_term(t[2])} {t[1]} {show_term(t[3])}'
    raise Refused(f'translator: the term {t}')


def show_atom(a):
    return a[0] + (f'({",".join(show_term(t) for t in a[1])})' if a[1] else '')


def show_lit(lit):
    if lit[0] == 'pos':
        return show_atom(lit[1])
    if lit[0] == 'not':
        return '\\+ ' + show_atom(lit[1])
    if lit[0] == 'cmp':
        return f'{show_term(lit[2])} {lit[1]} {show_term(lit[3])}'
    if lit[0] == 'is':
        return f'{show_term(lit[1])} is {show_term(lit[2])}'
    if lit[0] == 'range':
        return f'between({show_term(lit[2])},{show_term(lit[3])},{show_term(lit[1])})'
    raise Refused(f'translator: the literal {lit}')


def term_vars(t):
    return set(p2s.variables(t, {}))


# --- the program: its relations, the plans of its rules, its strata ------------

class Atom:
    """A body atom: relation `pred`, its arguments' roles in order, each
    ('const', term), ('match', var) a variable bound before, ('bind', var)
    one it binds, ('same', var) one it binds at an earlier position, or
    ('anon',). `bound`: the positions (from 0) of constants and matches;
    `probe` where every position is one (the tuple is looked up); else `by`,
    the positions of the index it is read through, empty for a scan."""
    def __init__(self, pred, roles, negated=False):
        self.pred, self.roles, self.negated = pred, roles, negated
        self.bound = [i for i, r in enumerate(roles) if r[0] in ('const', 'match')]
        self.probe = len(self.bound) == len(roles)
        self.by = [] if self.probe else self.bound

    def indexed(self):
        """Whether the atom is read through an index: a positive one with a
        bound position and not all, a negated one not all bound (with none
        bound, through the index of no position)."""
        return not self.probe and (self.negated or bool(self.by))


class Rule:
    """A clause: its heads [(relation, arguments)], their probabilities
    (None for a certain rule, [p] for a probabilistic one, [p1, ..., pn] for
    an annotated disjunction), its body as `steps` (Atom, ('test', kind, a,
    b), ('let', var, expr), ('range', var, lo, hi)), its variables, its text,
    its place among the rules, and whether it is a rule of the demand
    (`demand`), whose instances hold in every world."""
    def __init__(self, heads, probs, steps, variables, text, number, demand=False):
        self.heads, self.probs, self.steps = heads, probs, steps
        self.variables, self.text, self.number, self.demand = variables, text, number, demand


class Program:
    """preds: {key: its facts {args: probability}}; rules: [Rule]; strata:
    the components in the order of their dependencies, each (keys, whether
    recursive); queries: ground atoms; evidence: (ground atom, value)."""


def plan(heads, body, preds):
    """The steps of a clause's body: the literals in the order written, each
    comparison, `is` and negation taken as soon as its variables are bound."""
    bound, steps, todo = set(), [], list(body)
    what = f'a clause of {heads[0][0]}/{len(heads[0][1])}'

    def binding(lit):
        """(X, T) where lit is X = T (or T = X), X a variable not yet bound
        and T bound: it binds X."""
        if lit[0] == 'cmp' and lit[1] == '=':
            for x, y in ((lit[2], lit[3]), (lit[3], lit[2])):
                if x[0] == 'var' and x[1] not in bound and term_vars(y) <= bound:
                    return x, y
        return None

    def ready(lit):
        if lit[0] == 'cmp':
            return binding(lit) is not None or term_vars(lit[2]) | term_vars(lit[3]) <= bound
        if lit[0] == 'is':
            return term_vars(lit[2]) <= bound
        if lit[0] == 'not':
            return {t[1] for t in lit[1][1] if t[0] == 'var'} <= bound
        if lit[0] == 'range':
            return term_vars(lit[2]) | term_vars(lit[3]) <= bound
        return False

    while todo:
        i = next((k for k, lit in enumerate(todo) if lit[0] != 'pos' and ready(lit)), None)
        if i is None:
            i = next((k for k, lit in enumerate(todo) if lit[0] == 'pos'), None)
        if i is None:
            raise Refused(f'a literal of {what} whose variables no atom binds: {show_lit(todo[0])}')
        lit = todo.pop(i)
        if lit[0] in ('pos', 'not'):
            k = key(lit[1])
            if k not in preds:
                raise Refused(f'{k[0]}/{k[1]}, which no clause defines (ProbLog stops on an unknown clause)')
            roles, seen = [], set()
            for t in lit[1][1]:
                if t[0] in GROUND:
                    roles.append(('const', t))
                elif t[0] == 'anon':
                    roles.append(('anon',))
                elif t[0] == 'var' and t[1] in bound:
                    roles.append(('match', t[1]))
                elif t[0] == 'var' and t[1] in seen:
                    roles.append(('same', t[1]))
                elif t[0] == 'var' and lit[0] == 'pos':
                    roles.append(('bind', t[1]))
                    seen.add(t[1])
                else:
                    raise Refused('arithmetic as an argument of an atom (a compound term in ProbLog)')
            steps.append(Atom(k, roles, lit[0] == 'not'))
            bound |= seen
        elif lit[0] == 'cmp':
            kind = TESTS[lit[1]]
            if kind in ('eq', 'neq') and any(t[0] in ('op', 'neg') for t in (lit[2], lit[3])):
                raise Refused('translator: term (in)equality on arithmetic (a compound term in ProbLog)')
            b = binding(lit)
            if b is not None:
                steps.append(('let', b[0][1], b[1]))
                bound.add(b[0][1])
            else:
                steps.append(('test', kind, lit[2], lit[3]))
        elif lit[0] == 'is':
            x, e = lit[1], lit[2]
            if x[0] == 'var' and x[1] not in bound:
                steps.append(('let', x[1], e))
                bound.add(x[1])
            else:   # X is E with X bound: E's value unifies with X
                steps.append(('test', 'eq', x, e))
        elif lit[0] == 'range':
            _, x, lo, hi = lit
            steps.append(('range', x[1], lo, hi))
            bound.add(x[1])
    for _, args in heads:
        for t in args:
            if t[0] in ('op', 'neg'):
                raise Refused('arithmetic in the head of a clause (a compound term in ProbLog)')
            if t[0] == 'var' and t[1] not in bound:
                raise Refused(f'the variable {t[1]} of {what} that its body does not bind')
    return steps


def strongly_connected(deps):
    """The strongly connected components of deps (node -> the nodes it
    depends on), each sorted, every component after those it depends on
    (Tarjan's order), without recursion."""
    index, low, stack, on, out = {}, {}, [], set(), []
    for root in sorted(deps):
        if root in index:
            continue
        index[root] = low[root] = len(index)
        stack.append(root)
        on.add(root)
        work = [(root, iter(sorted(deps[root])))]
        while work:
            v, it = work[-1]
            w = next(it, None)
            if w is None:
                work.pop()
                if work:
                    low[work[-1][0]] = min(low[work[-1][0]], low[v])
                if low[v] == index[v]:
                    comp = []
                    while True:
                        x = stack.pop()
                        on.discard(x)
                        comp.append(x)
                        if x == v:
                            break
                    out.append(sorted(comp))
            elif w not in index:
                index[w] = low[w] = len(index)
                stack.append(w)
                on.add(w)
                work.append((w, iter(sorted(deps[w]))))
            elif w in on:
                low[v] = min(low[v], index[w])
    return out


def bound_by(body):
    """The variables that a body binds by itself, as `plan` binds them."""
    bound, grown = set(), True
    while grown:
        grown = False
        for lit in body:
            new = set()
            if lit[0] == 'pos':
                new = set().union(*map(term_vars, lit[1][1]))
            elif lit[0] == 'is' and lit[1][0] == 'var' and term_vars(lit[2]) <= bound:
                new = {lit[1][1]}
            elif lit[0] == 'cmp' and lit[1] == '=':
                new = {x[1] for x, y in ((lit[2], lit[3]), (lit[3], lit[2])) if x[0] == 'var' and term_vars(y) <= bound}
            elif lit[0] == 'range' and term_vars(lit[2]) | term_vars(lit[3]) <= bound:
                new = {lit[1][1]}
            if not new <= bound:
                bound |= new
                grown = True
    return bound


DEMAND = 'demand_'


def demand(clauses, queries, evidence):
    """The clauses, where some head variable of a clause is bound by its
    callers and not by its body (ProbLog grounds from the queries, which
    bind it: `weather(sun,T) <- T>0, Tprev is T-1, weather(sun,Tprev).`
    asked weather(sun,10)), evaluated for the arguments they are called
    with (magic sets). Such a clause n reads a relation `demand_n` of the
    values of those variables, first in its body; a ground query or piece
    of evidence of one of its heads' predicates is a tuple of it, and so is
    every call of one in a body, the literals before the call (but its
    negations) its rule's body. A tuple of demand_n holds whatever the
    coins (Rule.demand): it is an argument a call may ask for, the rule's
    instance for it holding as the clause's does, so that the answers are
    ProbLog's. A head variable that the body binds only after a call, which
    a test before the call reads (`step(I,K,D) :- K < D, K1 is K+1,
    step(I,K1,D).`, D bound by the recursive call), is demanded too, so
    that the call's demand reads the test."""
    unsafe = {}
    for n, (heads, _, body) in enumerate(clauses):
        free = dict.fromkeys(v for h in heads for t in h[1] for v in sorted(term_vars(t)) if v not in bound_by(body))
        if free:
            unsafe[n] = list(free)
    while unsafe:
        names = {f'{DEMAND}{n + 1}' for n in unsafe}
        if any(h[0] in names for heads, _, _ in clauses for h in heads):
            raise Refused(f'translator: a predicate named {DEMAND}N, the name of a relation of the demand')

        def guard(n):
            return ('pos', (f'{DEMAND}{n + 1}', [('var', v) for v in unsafe[n]]))
        calls = {}   # a predicate: (n, the positions of unsafe[n]'s variables in its head)
        for n, free in unsafe.items():
            for h in clauses[n][0]:
                at = {}
                for i, t in enumerate(h[1]):
                    if t[0] == 'var':
                        at.setdefault(t[1], i)
                if not set(free) <= set(at):
                    raise Refused(f'translator: a variable of an annotated disjunction of {h[0]}/{len(h[1])} '
                                  'that is bound by its callers and not in each head')
                # an annotated disjunction's heads of one predicate call it once
                entry = (n, [at[v] for v in free])
                if entry not in calls.setdefault(key(h), []):
                    calls[key(h)].append(entry)
        out, more = [], {}
        for atom in list(queries) + [a for a, _ in evidence]:
            for n, at in calls.get(key(atom), []):
                out.append(([(f'{DEMAND}{n + 1}', [atom[1][i] for i in at])], None, []))
        for m, (heads, _, body) in enumerate(clauses):
            before = [guard(m)] if m in unsafe else []
            mine = {v for h in heads for t in h[1] for v in term_vars(t)}
            for lit in body:
                for n, at in calls.get(key(lit[1]), []) if lit[0] in ('pos', 'not') else []:
                    args = [lit[1][1][i] for i in at]
                    if any(t[0] == 'anon' for t in args):
                        raise Refused(f'translator: a call of {lit[1][0]}/{len(lit[1][1])} that leaves an argument '
                                      'its clause needs unbound')
                    out.append(([(f'{DEMAND}{n + 1}', args)], None, list(before)))
                    tested = set().union(*(term_vars(t) for x in before if x[0] != 'pos' for t in x[1:]
                                           if isinstance(t, tuple)))
                    more.setdefault(m, set()).update((tested - bound_by(before)) & mine)
                if lit[0] != 'not':
                    before.append(lit)
        more = {m: sorted(vs) for m, vs in more.items() if vs}
        if not more:
            return [(heads, probs, [guard(n)] + body if n in unsafe else body)
                    for n, (heads, probs, body) in enumerate(clauses)] + out, names
        for m, vs in more.items():
            unsafe[m] = unsafe.get(m, []) + vs
    return clauses, set()


def named_heads(clause):
    """The clause, each anonymous variable of its heads a variable of its
    own, named as no variable of the clause is (`list(N, _) :- elements(N),
    ends(N).`), so that `demand` evaluates it for the arguments its callers
    ask, as ProbLog grounds it."""
    heads, probs, body = clause
    taken = set(p2s.clause_variables(heads, body))
    names = (f'Anon{k}' for k in itertools.count(1) if f'Anon{k}' not in taken)
    return [(name, [('var', next(names)) if t[0] == 'anon' else t for t in args])
            for name, args in heads], probs, body


def read(text):
    """The Program of a ProbLog text."""
    facts, rules, ads_fact, ads_rule, queries, evidence = p2s.parse(text)
    rules, ads_rule = p2s.builtins(facts, rules, ads_fact, ads_rule, queries, evidence,
                                   lambda lo, hi, x, body, k: [('range', x, lo, hi)])
    for atom in queries:
        if any(t[0] not in GROUND for t in atom[1]):
            raise Refused(f'translator: a query that is not ground: {show_atom(atom)}')
    for atom, _ in evidence:
        if any(t[0] not in GROUND for t in atom[1]):
            raise Refused(f'translator: evidence that is not ground: {show_atom(atom)}')
    # every clause as its heads, their probabilities and its body
    clauses = []
    for prob, head, body in rules:
        if prob != 0:
            clauses.append(([head], None if prob is None or prob >= 1 else [prob], body))
    for alts, body in ads_fact + ads_rule:
        alts = [(p, h) for p, h in alts if p > 0]
        if alts:
            clauses.append(([h for _, h in alts], [p for p, _ in alts], body))
    clauses, demanded = demand([named_heads(c) for c in clauses], queries, evidence)
    preds = dict(facts)
    for heads, _, _ in clauses:
        for h in heads:
            preds.setdefault(key(h), {})
    for k in [key(a) for a in queries] + [key(a) for a, _ in evidence]:
        if k not in preds:
            raise Refused(f'{k[0]}/{k[1]}, which no clause defines (ProbLog stops on an unknown clause)')
    # the heads of an annotated disjunction are evaluated together, as one node
    node = {k: k for k in preds}

    def find(k):
        while node[k] != k:
            k = node[k]
        return k
    for heads, _, _ in clauses:
        for h in heads[1:]:
            a, b = find(key(heads[0])), find(key(h))
            if a != b:
                node[b] = a
    deps = {k: set() for k in preds}
    for heads, _, body in clauses:
        for h in heads:
            deps[key(h)].update(key(lit[1]) for lit in body if lit[0] in ('pos', 'not'))
    together = {}
    for k in preds:
        together.setdefault(find(k), set()).add(k)
    # only what the queries and the evidence read, as ProbLog grounds them
    need = p2s.reach({k: deps[k] | together[find(k)] for k in preds},
                     [key(a) for a in queries] + [key(a) for a, _ in evidence])
    prog = Program()
    prog.preds = {k: rows for k, rows in preds.items() if k in need}
    prog.rules = []
    for heads, probs, body in clauses:
        if key(heads[0]) not in need:
            continue
        text = ('; '.join((f'{p2s.fmt_prob(p)}::' if p is not None else '') + show_atom(h)
                          for p, h in zip(probs or [None], heads))
                + (f' :- {", ".join(show_lit(lit) for lit in body)}' if body else '') + '.')
        steps = plan(heads, body, prog.preds)
        prog.rules.append(Rule([(key(h), h[1]) for h in heads], probs, steps, p2s.clause_variables(heads, body),
                               text, len(prog.rules), heads[0][0] in demanded))
    graph = {}
    for k in need:
        graph.setdefault(find(k), set()).update(find(d) for d in deps[k] if d in need)
    prog.strata = []
    for comp in strongly_connected(graph):
        members = sorted(k for k in need if find(k) in comp)
        recursive = len(comp) > 1 or comp[0] in graph[comp[0]]
        for r in prog.rules:
            if recursive and r.heads[0][0] in members:
                for s in r.steps:
                    if isinstance(s, Atom) and s.negated and s.pred in members:
                        raise Refused('translator: a negation through recursion '
                                      f'({r.heads[0][0][0]}/{r.heads[0][0][1]} and {s.pred[0]}/{s.pred[1]})')
        prog.strata.append((members, recursive))
    prog.queries = queries
    prog.evidence = evidence
    return prog


NUMERIC = re.compile(r'[-+]?(\d|\.\d|inf|nan)', re.I)


def data_lines(prog):
    """The data file: a line `relation<TAB>probability<TAB>argument...` a
    fact of probability above 0, the relation NAME/ARITY. A symbol that the
    languages could read as a number, or that holds a tab or a line break,
    is refused (the file could not say which it is)."""
    out = []
    for (name, n), rows in prog.preds.items():
        rel = f'{name}/{n}'
        for args, p in rows.items():
            if p <= 0:
                continue
            fields = [rel, p2s.fmt_prob(p)]
            for t in args:
                if t[0] == 'sym':
                    s = t[1]
                    if not s or NUMERIC.match(s) or s[0] == '#' or s != s.strip() or re.search(r'[\t\n\r]', s):
                        raise Refused(f'translator: the symbol {s!r}, which a data file cannot tell from a number')
                    fields.append(s)
                else:
                    fields.append(repr(float(t[1])) if t[0] == 'num' else str(t[1]))
            out.append('\t'.join(fields))
    return out


# --- what the back ends share --------------------------------------------------

def idents(keys, reserved, own=OWN):
    """{key: an identifier}: the predicate's name, `_ARITY` appended where
    the name has two arities, `p_` before a name that is no identifier, `_`
    appended where the language reserves it or the program's own variables
    could take it (`own`)."""
    arities = {}
    for name, n in keys:
        arities.setdefault(name, set()).add(n)
    out = {}
    for name, n in sorted(keys):
        base = name if re.fullmatch(r'[a-z][A-Za-z0-9_]*', name) else 'p_' + re.sub(r'\W', '_', name)
        if len(arities[name]) > 1:
            base = f'{base}_{n}'
        while base in reserved or own.fullmatch(base) or base in out.values():
            base += '_'
        out[(name, n)] = base
    return out


def var_names(variables, reserved):
    """{ProbLog variable: its name in the program}: the same (a ProbLog
    variable starts with a capital or `_`, the program's own names with a
    small letter), `_` appended where the language reserves it."""
    out = {}
    for v in variables:
        name = v
        while name in reserved:
            name += '_'
        out[v] = name
    return out


class Backend:
    """A language's program: the relations' names (`rel`), the loops of a
    rule (`rule`), the strata in order (`program`)."""

    def __init__(self, prog, source):
        self.prog, self.source = prog, source
        self.rel = idents(prog.preds, self.RESERVED)
        self.built = set()   # the indexes of relations of earlier strata, each built once

    def term(self, t, names):
        if t[0] == 'var':
            return names[t[1]]
        if t[0] in GROUND:
            return self.const(t)
        if t[0] == 'neg':
            return self.neg(self.term(t[1], names))
        if t[0] == 'op':
            if t[1] == '/':
                raise Refused('translator: the division / (a float in ProbLog)')
            return self.arith(t[1], self.term(t[2], names), self.term(t[3], names))
        raise Refused(f'translator: the term {t}')

    def role_value(self, role, names):
        return self.term(role[1], names) if role[0] == 'const' else names[role[1]]

    def index_atoms(self, rules, keys):
        """The atoms of `rules` read through an index of a relation of `keys`,
        one a relation and its positions."""
        seen, out = set(), []
        for r in rules:
            for s in r.steps:
                if isinstance(s, Atom) and s.indexed() and s.pred in keys and (s.pred, tuple(s.by)) not in seen:
                    seen.add((s.pred, tuple(s.by)))
                    out.append(s)
        return out

    def earlier_indexes(self, rules, members):
        """The index atoms of rules over relations of earlier strata whose
        index no earlier stratum has built."""
        out = []
        for s in self.index_atoms(rules, {k for k in self.prog.preds if k not in members}):
            if (s.pred, tuple(s.by)) not in self.built:
                self.built.add((s.pred, tuple(s.by)))
                out.append(s)
        return out

    def index_name(self, s):
        return self.rel[s.pred] + self.SEP + 'by' + ''.join(self.SEP + str(i + 1) for i in s.by)

    def coin_rules(self, rules):
        return [r for r in rules if r.probs]


# --- Alea.jl -------------------------------------------------------------------

class Julia(Backend):
    """The Alea.jl program (the module's docstring)."""
    SEP = '_'
    RESERVED = set('''baremodule begin break catch const continue do else elseif end export false finally for
function global if import let local macro module quote return struct true try using while abstract mutable
primitive type where in isa Relation NONE add index holds holds_any probe choose constant facts program
model observe QUERY FACTS flip pr db stage before queries evidence Alea Dict Tuple Any Bool Int Float64
Symbol div mod get length copy Pair Vector nothing missing'''.split())
    TESTS = {'eq': '===', 'neq': '!==', 'aeq': '==', 'aneq': '!=', '<': '<', '>': '>', '<=': '<=', '>=': '>='}

    def const(self, t):
        if t[0] == 'int':
            return str(t[1])
        if t[0] == 'num':
            return repr(float(t[1]))
        if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', t[1]) and t[1] not in self.RESERVED:
            return ':' + t[1]
        return 'Symbol("' + t[1].replace('\\', '\\\\').replace('"', '\\"').replace('$', '\\$') + '")'

    def neg(self, a):
        return f'-{a}'

    def arith(self, op, a, b):
        return f'div({a}, {b})' if op == '//' else f'mod({a}, {b})' if op == 'mod' else f'({a} {op} {b})'

    def tuple(self, items):
        return '()' if not items else f'({items[0]},)' if len(items) == 1 else f'({", ".join(items)})'

    def rule(self, r, out, ind, write, coins):
        """The loop nest of rule r, adding to the relations `write` names."""
        names = var_names(r.variables, self.RESERVED)
        out.append(f'{ind}# {r.text}')
        vals, opened, fresh = [], [], [0]
        state = {'depth': ind, 'loop': False}

        def local(hint):
            fresh[0] += 1
            return f'{hint}{fresh[0]}'

        def block(line, loop):
            out.append(state['depth'] + line)
            opened.append(state['depth'])
            state['depth'] += '    '
            state['loop'] |= loop

        def guard(cond):
            if state['loop']:
                out.append(f'{state["depth"]}{cond} || continue')
            else:
                block(f'if {cond}', False)
        for s in r.steps:
            if isinstance(s, Atom):
                rel = self.rel[s.pred]
                k = self.tuple([self.role_value(s.roles[i], names) for i in s.bound])
                if s.negated:
                    vals.append(f'!holds({rel}, {k})' if s.probe else f'!holds_any({self.index_name(s)}, {k})')
                    continue
                v = local('x')
                vals.append(v)
                if s.probe:
                    block(f'for {v} in probe({rel}, {k})', True)
                    continue
                pattern, tests = [], []
                for role in s.roles:
                    if role[0] == 'bind':
                        pattern.append(names[role[1]])
                    elif role[0] == 'same':
                        c = local('a')
                        pattern.append(c)
                        tests.append(f'{c} === {names[role[1]]}')
                    else:
                        pattern.append('_')
                src = f'get({self.index_name(s)}, {k}, NONE)' if s.by else rel
                block(f'for ({self.tuple(pattern)}, {v}) in {src}', True)
                for test in tests:
                    guard(test)
            elif s[0] == 'test':
                guard(f'{self.term(s[2], names)} {self.TESTS[s[1]]} {self.term(s[3], names)}')
            elif s[0] == 'let':
                out.append(f'{state["depth"]}{names[s[1]]} = {self.term(s[2], names)}')
            else:
                block(f'for {names[s[1]]} in {self.term(s[2], names)}:{self.term(s[3], names)}', True)
        depth = state['depth']
        inst = self.tuple([names[v] for v in r.variables])
        heads = [(write[h], self.tuple([self.term(t, names) for t in args])) for h, args in r.heads]
        if r.probs is None:
            out.append(f'{depth}add!({heads[0][0]}, {heads[0][1]}, {" & ".join([] if r.demand else vals) or "true"})')
        elif len(r.probs) == 1:
            coin = f'flip({p2s.fmt_prob(r.probs[0])})'
            if coins:
                coin = f'get!(() -> {coin}, {coins}, {inst})'
            out.append(f'{depth}add!({heads[0][0]}, {heads[0][1]}, {" & ".join(vals + [coin])})')
        else:
            choice = f'choose({", ".join(p2s.fmt_prob(p) for p in r.probs)})'
            if coins:
                choice = f'get!(() -> {choice}, {coins}, {inst})'
            out.append(f'{depth}chosen = {choice}')
            for i, (rel, t) in enumerate(heads):
                out.append(f'{depth}add!({rel}, {t}, {" & ".join(vals + [f"chosen[{i + 1}]"])})')
        for d in reversed(opened):
            out.append(f'{d}end')

    def indexes(self, atoms, ind, out):
        for s in atoms:
            out.append(f'{ind}{self.index_name(s)} = index({self.rel[s.pred]}{"".join(f", {i + 1}" for i in s.by)})')

    def program(self):
        prog, rel = self.prog, self.rel
        ind, out = '    ', ['function program(db)']
        for k in prog.preds:
            if prog.preds[k]:
                out.append(f'{ind}{rel[k]}_facts = get(db, "{k[0]}/{k[1]}", Relation())')
        out.append('')
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            self.indexes(self.earlier_indexes(rules, members), ind, out)
            if not recursive:
                for k in members:
                    start = (f'copy({rel[k]}_facts)' if rules else f'{rel[k]}_facts') if prog.preds[k] else 'Relation()'
                    out.append(f'{ind}{rel[k]} = {start}')
                for r in rules:
                    self.rule(r, out, ind, rel, None)
                out.append('')
                continue
            out.append(f'{ind}# {", ".join(f"{k[0]}/{k[1]}" for k in members)}: recursive; the rules applied in '
                       'stages from no tuple, each')
            out.append(f'{ind}# reading the stage before, until a stage adds no tuple and the stages number the tuples')
            coins = {}
            for r in self.coin_rules(rules):
                coins[r.number] = f'coins{r.number + 1}'
                out.append(f'{ind}{coins[r.number]} = Dict{{Tuple,Any}}()   # by instance, of `{r.text}`')
            for k in members:
                out.append(f'{ind}{rel[k]} = Relation()')
            size = ' + '.join(f'length({rel[k]})' for k in members)
            out.append(f'{ind}stage, before = 0, -1')
            out.append(f'{ind}while stage < {size} || {size} != before')
            ind2 = ind + '    '
            out.append(f'{ind2}before = {size}')
            self.indexes(self.index_atoms(rules, set(members)), ind2, out)
            nxt = {k: f'next_{rel[k]}' for k in members}
            for k in members:
                out.append(f'{ind2}{nxt[k]} = ' + (f'copy({rel[k]}_facts)' if prog.preds[k] else 'Relation()'))
            for r in rules:
                self.rule(r, out, ind2, {**rel, **nxt}, coins.get(r.number))
            out.append(f'{ind2}{", ".join(rel[k] for k in members)} = {", ".join(nxt[k] for k in members)}')
            out.append(f'{ind2}stage += 1')
            out.append(f'{ind}end')
            out.append('')

        def holds(a):
            return f'holds({rel[key(a)]}, {self.tuple([self.const(t) for t in a[1]])})'
        out.append(f'{ind}queries = {self.tuple([holds(a) for a in prog.queries])}')
        ev = ' & '.join(('' if v else '!') + holds(a) for a, v in prog.evidence)
        out.append(f'{ind}evidence = {ev or "true"}')
        out.append(f'{ind}queries, evidence')
        out.append('end')
        return '\n'.join(out)

    def text(self):
        return JULIA_PRELUDE.replace('{source}', self.source) + self.program() + '\n' + JULIA_MODEL


JULIA_PRELUDE = '''\
# {source}: a ProbLog program written in Julia for Alea.jl by tddlog's
# scripts/problog2ppl.py, first order as it is written. A relation is a
# dictionary from its tuples to whether each holds, a Dist{Bool} of the coins
# or a Bool; each probabilistic fact is a coin of its own (`flip`), read from
# the facts file FACTS; each rule is a loop over the relations its body
# joins. `model()` is the queries' holdings, given the evidence (`observe`),
# in the form of tddlog's data/alea programs: Alea.jl answers them with `pr`,
# and its tddlog backend writes their circuit for tddlog.
#
#   julia --project=ALEA alea_bench.jl run --file FACTS FACTS_PATH problog.jl
using Alea

const QUERY = :pr
const FACTS = ""   # the facts file (alea_bench.jl --file FACTS PATH)

const Relation = Dict{Tuple,Any}
const NONE = Pair{Tuple,Any}[]

# the tuple t of r holds also where x does
add!(r::Relation, t::Tuple, x) = (r[t] = haskey(r, t) ? r[t] | x : x; nothing)
# whether the tuple t of r holds
holds(r::Relation, t::Tuple) = get(r, t, false)
# whether t holds, if it is a tuple of r: none or one value to loop over
probe(r::Relation, t::Tuple) = haskey(r, t) ? (r[t],) : ()
# the tuples of r and whether each holds, by their values at positions ps
function index(r::Relation, ps...)
    ix = Dict{Tuple,Vector{Pair{Tuple,Any}}}()
    for (t, x) in r
        push!(get!(() -> Pair{Tuple,Any}[], ix, map(p -> t[p], ps)), t => x)
    end
    ix
end
# whether a tuple of the index's key k holds
holds_any(ix, k) = reduce(|, (x for (_, x) in get(ix, k, NONE)); init = false)
# the choice of an annotated disjunction p1::h1; ...; pn::hn: whether each
# head is the one chosen, the k-th with probability pk and none with the rest
function choose(ps...)
    chosen, none, left = Any[], true, 1.0
    for p in ps
        q = left > 0 ? min(p / left, 1.0) : 1.0
        c = q >= 1.0 ? true : flip(q)
        push!(chosen, none & c)
        none, left = none & !c, left - p
    end
    chosen
end

constant(s) = something(tryparse(Int, s), tryparse(Float64, s), Some(Symbol(s)))
# the relations of the facts file: a line `relation probability argument...`
# a fact, a coin of its own where its probability is below one
function facts(path)
    db = Dict{String,Relation}()
    for line in eachline(path)
        f = split(line, '\\t')
        p = parse(Float64, f[2])
        add!(get!(Relation, db, f[1]), Tuple(constant.(f[3:end])), p >= 1 ? true : flip(p))
    end
    db
end

'''

JULIA_MODEL = '''
# The queries' holdings, the evidence observed: Alea's `@alea` keeps the
# observation, and the relations are built outside it, as plain Julia.
function model()
    qs, ev = program(facts(FACTS))
    ev === true && return qs
    @alea begin
        observe(ev)
        qs
    end
end
'''


# --- Roulette ------------------------------------------------------------------

class Racket(Backend):
    """The Roulette module (the module's docstring)."""
    SEP = '-'
    RESERVED = set('''define lambda let if cond else and or not when unless for list vector hash add holds
holds-any probe index choose constant facts distribution probability real report statement flip observe stage
before db queries evidence dists car cdr first second rest length match values void null any'''.split())
    TESTS = {'eq': 'equal?', 'neq': 'equal?', 'aeq': '=', 'aneq': '=', '<': '<', '>': '>', '<=': '<=', '>=': '>='}

    def __init__(self, prog, source):
        super().__init__(prog, source)
        self.rel = {k: v.replace('_', '-') for k, v in self.rel.items()}

    def const(self, t):
        if t[0] == 'int':
            return str(t[1])
        if t[0] == 'num':
            return repr(float(t[1]))
        if re.fullmatch(r'[a-z][A-Za-z0-9_]*', t[1]):
            return "'" + t[1]
        return "'|" + t[1].replace('\\', '\\\\').replace('|', '\\|') + "|"

    def neg(self, a):
        return f'(- {a})'

    def arith(self, op, a, b):
        return f'({ {"//": "quotient", "mod": "modulo"}.get(op, op) } {a} {b})'

    def test(self, kind, a, b):
        t = f'({self.TESTS[kind]} {a} {b})'
        return f'(not {t})' if kind in ('neq', 'aneq') else t

    def conj(self, vals):
        return vals[0] if len(vals) == 1 else f'(&& {" ".join(vals)})' if vals else '#t'

    def rule(self, r, out, ind, write, coins):
        names = var_names(r.variables, self.RESERVED)
        out.append(f'{ind};; {r.text}')
        vals, opened, fresh = [], [0], [0]
        state = {'depth': ind}

        def local(hint):
            fresh[0] += 1
            return f'{hint}{fresh[0]}'

        def form(text):
            out.append(state['depth'] + text)
            opened[0] += 1
            state['depth'] += '  '
        for s in r.steps:
            if isinstance(s, Atom):
                rel = self.rel[s.pred]
                k = f'(list {" ".join(self.role_value(s.roles[i], names) for i in s.bound)})'.replace('(list )', '(list)')
                if s.negated:
                    vals.append(f'(! (holds {rel} {k}))' if s.probe else f'(! (holds-any {self.index_name(s)} {k}))')
                    continue
                v = local('x')
                vals.append(v)
                if s.probe:
                    form(f'(for ([{v} (in-list (probe {rel} {k}))])')
                    continue
                pattern, tests = [], []
                for role in s.roles:
                    if role[0] == 'bind':
                        pattern.append(names[role[1]])
                    elif role[0] == 'same':
                        c = local('a')
                        pattern.append(c)
                        tests.append(f'(equal? {c} {names[role[1]]})')
                    else:
                        pattern.append('_')
                t = local('t')
                src = f'(in-list (hash-ref {self.index_name(s)} {k} null))' if s.by else f'(in-hash-pairs {rel})'
                form(f'(for ([{t} {src}])')
                out.append(f'{state["depth"]}(match-define (cons (list {" ".join(pattern)}) {v}) {t})')
                for test in tests:
                    form(f'(when {test}')
            elif s[0] == 'test':
                form(f'(when {self.test(s[1], self.term(s[2], names), self.term(s[3], names))}')
            elif s[0] == 'let':
                form(f'(let ([{names[s[1]]} {self.term(s[2], names)}])')
            else:
                form(f'(for ([{names[s[1]]} (in-range {self.term(s[2], names)} (+ {self.term(s[3], names)} 1))])')
        inst = f'(list {" ".join(names[v] for v in r.variables)})'

        def tup(args):
            return f'(list {" ".join(self.term(t, names) for t in args)})'.replace('(list )', '(list)')
        heads = [(write[h], tup(args)) for h, args in r.heads]
        if r.probs is None:
            lines = [f'(add! {heads[0][0]} {heads[0][1]} {self.conj([] if r.demand else vals)})']
        elif len(r.probs) == 1:
            coin = f'(flip {p2s.fmt_prob(r.probs[0])})'
            if coins:
                coin = f'(hash-ref! {coins} {inst} (λ () {coin}))'
            lines = [f'(add! {heads[0][0]} {heads[0][1]} {self.conj(vals + [coin])})']
        else:
            choice = f'(choose {" ".join(p2s.fmt_prob(p) for p in r.probs)})'
            if coins:
                choice = f'(hash-ref! {coins} {inst} (λ () {choice}))'
            form(f'(let ([chosen {choice}])')
            lines = [f'(add! {rel} {t} {self.conj(vals + [f"(list-ref chosen {i})"])})'
                     for i, (rel, t) in enumerate(heads)]
        for line in lines[:-1]:
            out.append(state['depth'] + line)
        out.append(state['depth'] + lines[-1] + ')' * opened[0])

    def indexes(self, atoms, ind, out):
        for s in atoms:
            out.append(f'{ind}(define {self.index_name(s)} (index {self.rel[s.pred]}{"".join(f" {i}" for i in s.by)}))')

    def program(self):
        prog, rel = self.prog, self.rel
        out = ['(define db (facts (vector-ref (current-command-line-arguments) 0)))']
        for k in prog.preds:
            if prog.preds[k]:
                out.append(f'(define {rel[k]}-facts (hash-ref db "{k[0]}/{k[1]}" (make-hash)))')
        out.append('')
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            self.indexes(self.earlier_indexes(rules, members), '', out)
            if not recursive:
                for k in members:
                    start = ((f'(hash-copy {rel[k]}-facts)' if rules else f'{rel[k]}-facts') if prog.preds[k]
                             else '(make-hash)')
                    out.append(f'(define {rel[k]} {start})')
                if rules:
                    out.append('(statement')
                    for r in rules:
                        self.rule(r, out, '  ', rel, None)
                    out[-1] += ')'
                out.append('')
                continue
            out.append(f';; {", ".join(f"{k[0]}/{k[1]}" for k in members)}: recursive; the rules applied in stages '
                       'from no tuple, each')
            out.append(';; reading the stage before, until a stage adds no tuple and the stages number the tuples')
            coins = {}
            for r in self.coin_rules(rules):
                coins[r.number] = f'coins{r.number + 1}'
                out.append(f'(define {coins[r.number]} (make-hash))   ; by instance, of `{r.text}`')
            params = ' '.join(rel[k] for k in members)
            fn = f'stage-{rel[members[0]]}'
            out.append(f'(define ({fn} {params})')
            nxt = {k: f'next-{rel[k]}' for k in members}
            for k in members:
                out.append(f'  (define {nxt[k]} ' + (f'(hash-copy {rel[k]}-facts))' if prog.preds[k] else '(make-hash))'))
            self.indexes(self.index_atoms(rules, set(members)), '  ', out)
            for r in rules:
                self.rule(r, out, '  ', {**rel, **nxt}, coins.get(r.number))
            out.append(f'  (list {" ".join(nxt[k] for k in members)}))')
            out.append(f'(define-values ({params})')
            out.append(f'  (let loop ([rels (list {" ".join("(make-hash)" for _ in members)})] [stage 0] [before -1])')
            out.append('    (define size (for/sum ([r (in-list rels)]) (hash-count r)))')
            out.append('    (if (or (< stage size) (not (= size before)))')
            out.append(f'        (loop (apply {fn} rels) (+ stage 1) size)')
            out.append('        (apply values rels))))')
            out.append('')

        def holds(a):
            return f'(holds {rel[key(a)]} (list {" ".join(self.const(t) for t in a[1])}))'.replace('(list )', '(list)')
        ev = [holds(a) if v else f'(! {holds(a)})' for a, v in prog.evidence]
        if ev:
            out.append(f'(statement (observe! {self.conj(ev)}))')
        out.append('(define dists')
        out.append('  (list ' + '\n        '.join(f'(distribution {holds(a)})' for a in prog.queries) + '))')
        out.append('(statement (report dists))')
        return '\n'.join(out)

    def text(self):
        return RACKET_PRELUDE.replace('{source}', self.source) + self.program() + '\n'


RACKET_PRELUDE = '''\
#lang roulette/example/disrupt
;; {source}: a ProbLog program written for Roulette by tddlog's
;; scripts/problog2ppl.py, first order as it is written. A relation is a hash
;; from its tuples (lists of constants) to whether each holds, a Boolean of the
;; coins; each probabilistic fact is a coin of its own (`flip`), read from the
;; facts file named on the command line; each rule is a loop over the
;; relations its body joins. Each query's distribution is Roulette's, at the
;; module's level (`display-pmf`), given the evidence; the program prints
;; `roulette K true P` and `roulette K false P` for the K-th query.

(require racket/port racket/string racket/match)

;; A statement of the program: a definition, which the language does not
;; run as a query of its own (it runs each expression of the module as one).
(define-syntax-rule (statement e ...) (define-values () (begin e ... (values))))

;; the tuple t of r holds also where x does
(define (add! r t x)
  (hash-set! r t (if (hash-has-key? r t) (|| (hash-ref r t) x) x)))
;; whether the tuple t of r holds
(define (holds r t) (hash-ref r t #f))
;; whether t holds, if it is a tuple of r: none or one value to loop over
(define (probe r t) (if (hash-has-key? r t) (list (hash-ref r t)) null))
;; the tuples of r and whether each holds, (tuple . holding) each, by their
;; values at positions ps (from 0)
(define (index r . ps)
  (define ix (make-hash))
  (for ([(t x) (in-hash r)])
    (hash-update! ix (for/list ([p (in-list ps)]) (list-ref t p)) (λ (es) (cons (cons t x) es)) null))
  ix)
;; whether a tuple of the index's key k holds
(define (holds-any ix k)
  (for/fold ([some #f]) ([e (in-list (hash-ref ix k null))]) (|| some (cdr e))))
;; the choice of an annotated disjunction p1::h1; ...; pn::hn: whether each
;; head is the one chosen, the k-th with probability pk and none with the rest
(define (choose . ps)
  (let loop ([ps ps] [left 1.0] [none #t])
    (if (null? ps)
        null
        (let* ([q (if (> left 0) (min 1.0 (/ (car ps) left)) 1.0)]
               [c (if (>= q 1.0) #t (flip q))])
          (cons (&& none c) (loop (cdr ps) (- left (car ps)) (&& none (! c))))))))

(define (constant s) (or (string->number s) (string->symbol s)))
;; the relations of the facts file: a line `relation probability argument...`
;; a fact, a coin of its own where its probability is below one
(define (facts path)
  (define db (make-hash))
  (for ([line (in-lines (open-input-file path))])
    (define f (string-split line "\\t" #:trim? #f))
    (define p (string->number (cadr f)))
    (add! (hash-ref! db (car f) make-hash) (map constant (cddr f)) (if (>= p 1) #t (flip p))))
  db)

;; The distribution of v, a list of (value . probability): Roulette's query
;; at the module's level (`display-pmf`), read back from the table it
;; prints, or from the one value it prints alone.
(define (distribution v)
  (define text (with-output-to-string (λ () (display-pmf v))))
  (define (datum s)
    (define d (read (open-input-string s)))
    (if (and (pair? d) (eq? (car d) 'quote)) (cadr d) d))
  (define rows
    (for/list ([line (in-list (string-split text "\\n"))]
               #:when (regexp-match? #rx"[│|]" line))
      (filter (λ (s) (not (string=? s ""))) (map string-trim (regexp-split #rx"[│|]" line)))))
  (if (null? rows)
      (list (cons (datum text) 1))
      (for/list ([r (in-list rows)]
                 #:unless (equal? r '("Value" "Probability")))
        (cons (datum (car r)) (string->number (cadr r))))))
(define (real p) (if (exact? p) (exact->inexact p) p))
(define (probability dist v)
  (define e (assoc v dist))
  (if e (cdr e) 0))
(define (report dists)
  (for ([dist (in-list dists)] [k (in-naturals 1)])
    (printf "roulette ~a true ~a\\n" k (real (probability dist #t)))
    (printf "roulette ~a false ~a\\n" k (real (probability dist #f)))))

'''


# --- WebPPL --------------------------------------------------------------------

class WebPPL(Backend):
    """The WebPPL program (the module's docstring)."""
    SEP = '_'
    RESERVED = set('''break case catch class const continue debugger default delete do else export extends false
finally for function if import in instanceof new null return super switch this throw true try typeof var void
while with yield let static enum await implements package protected interface private public arguments eval
flip mem map filter reduce any all and or not yes no coin holds holdsAny probe index relation relationOfFacts
choose keyOf lpFacts model Infer condition categorical sum only Math JSON Object Array undefined NaN
Infinity'''.split())
    TESTS = {'eq': '===', 'neq': '!==', 'aeq': '===', 'aneq': '!==', '<': '<', '>': '>', '<=': '<=', '>=': '>='}

    def const(self, t):
        if t[0] == 'int':
            return str(t[1])
        if t[0] == 'num':
            return repr(float(t[1]))
        return "'" + t[1].replace('\\', '\\\\').replace("'", "\\'") + "'"

    def neg(self, a):
        return f'-{a}'

    def arith(self, op, a, b):
        if op == '//':
            return f'Math.trunc({a} / {b})'
        if op == 'mod':
            return f'((({a}) % ({b}) + ({b})) % ({b}))'
        return f'({a} {op} {b})'

    def conj(self, vals):
        return vals[0] if len(vals) == 1 else f'and([{", ".join(vals)}])' if vals else 'yes'

    def guarded(self, cond, lines, ind):
        """lines where cond holds; no instance where it does not."""
        return ([f'{ind}if ({cond}) {{'] + ['  ' + line for line in lines]
                + [f'{ind}}} else {{', f'{ind}  return [];', f'{ind}}}'])

    def rule(self, r, out, ind, coins):
        """The instances of rule r, an expression: a list of [head relation,
        tuple, holding] built by nested maps over the relations its body
        joins."""
        names = var_names(r.variables, self.RESERVED)
        fresh = [0]

        def local(hint):
            fresh[0] += 1
            return f'{hint}{fresh[0]}'

        def instance(h, args, holding):
            return f"['{self.rel[h]}', [{', '.join(self.term(t, names) for t in args)}], {holding}]"

        def nest(steps, vals, ind):
            if not steps:
                inst = f'[{", ".join(names[v] for v in r.variables)}]'
                (h, args), = r.heads[:1]
                if r.probs is None:
                    return [f'{ind}return [{instance(h, args, self.conj([] if r.demand else vals))}];']
                if len(r.probs) == 1:
                    coin = f'{coins}({inst})' if coins else f'coin({p2s.fmt_prob(r.probs[0])})'
                    return [f'{ind}return [{instance(h, args, self.conj(vals + [coin]))}];']
                choice = f'{coins}({inst})' if coins else f'choose([{", ".join(p2s.fmt_prob(p) for p in r.probs)}])'
                rows = [instance(h, args, self.conj(vals + [f'chosen[{i}]'])) for i, (h, args) in enumerate(r.heads)]
                return [f'{ind}var chosen = {choice};', f'{ind}return [', ',\n'.join(f'{ind}  {x}' for x in rows),
                        f'{ind}];']
            s, rest = steps[0], steps[1:]
            if isinstance(s, Atom):
                rel = self.rel[s.pred]
                k = f'[{", ".join(self.role_value(s.roles[i], names) for i in s.bound)}]'
                if s.negated:
                    v = f'not(holds({rel}, {k}))' if s.probe else f'not(holdsAny({self.index_name(s)}, {k}))'
                    return nest(rest, vals + [v], ind)
                e = local('e')
                src = (f'probe({rel}, {k})' if s.probe else
                       f'({self.index_name(s)}[keyOf({k})] || [])' if s.by else f'_.values({rel})')
                lines, tests = [f'{ind}return _.flatten(map(function({e}) {{'], []
                for i, role in enumerate(s.roles):
                    if role[0] == 'bind':
                        lines.append(f'{ind}  var {names[role[1]]} = {e}.t[{i}];')
                    elif role[0] == 'same':
                        tests.append(f'{e}.t[{i}] === {names[role[1]]}')
                inner = nest(rest, vals + [f'{e}.v'], ind + '  ')
                lines += self.guarded(' && '.join(tests), inner, ind + '  ') if tests else inner
                lines.append(f'{ind}}}, {src}));')
                return lines
            if s[0] == 'test':
                cond = f'{self.term(s[2], names)} {self.TESTS[s[1]]} {self.term(s[3], names)}'
                return self.guarded(cond, nest(rest, vals, ind), ind)
            if s[0] == 'let':
                return [f'{ind}var {names[s[1]]} = {self.term(s[2], names)};'] + nest(rest, vals, ind)
            lo, hi = self.term(s[2], names), self.term(s[3], names)
            return ([f'{ind}return _.flatten(map(function({names[s[1]]}) {{'] + nest(rest, vals, ind + '  ')
                    + [f'{ind}}}, _.range({lo}, {hi} + 1)));'])
        out.append(f'{ind}// {r.text}')
        out.append(f'{ind}(function() {{')
        out.extend(nest(r.steps, [], ind + '  '))
        out.append(f'{ind}}})()')

    def relations(self, members, rules, out, ind, coins, write):
        """The relations of members (`write` names them): each its facts and
        the instances of the rules whose heads it holds."""
        prog, rel = self.prog, self.rel
        parts = []
        for r in rules:
            part = []
            self.rule(r, part, ind + '  ', coins.get(r.number))
            parts.append('\n'.join(part))
        name = f'instances_{rel[members[0]]}'
        out.append(f'{ind}var {name} = [].concat(')
        out.append(',\n'.join(parts))
        out.append(f'{ind});')
        for k in members:
            base = f'{rel[k]}_facts' if prog.preds[k] else '{}'
            mine = name if len(members) == 1 else f"only('{rel[k]}', {name})"
            out.append(f'{ind}var {write(k)} = relation({base}, {mine});')

    def indexes(self, atoms, ind, out):
        for s in atoms:
            out.append(f'{ind}var {self.index_name(s)} = index({self.rel[s.pred]}, {s.by});')

    def program(self):
        prog, rel = self.prog, self.rel
        out = []
        for k in prog.preds:
            if prog.preds[k]:
                out.append(f"var {rel[k]}_facts = relationOfFacts(lpFacts[{self.const(('sym', f'{k[0]}/{k[1]}'))}] || []);")
        out.append('')
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            self.indexes(self.earlier_indexes(rules, members), '', out)
            if not recursive:
                if rules:
                    self.relations(members, rules, out, '', {}, lambda k: rel[k])
                else:
                    for k in members:
                        out.append(f'var {rel[k]} = ' + (f'{rel[k]}_facts;' if prog.preds[k] else '{};'))
                out.append('')
                continue
            out.append(f'// {", ".join(f"{k[0]}/{k[1]}" for k in members)}: recursive; the rules applied in stages '
                       'from no tuple, each')
            out.append('// reading the stage before, until a stage adds no tuple and the stages number the tuples')
            coins = {}
            for r in self.coin_rules(rules):
                coins[r.number] = f'coins{r.number + 1}'
                draw = (f'coin({p2s.fmt_prob(r.probs[0])})' if len(r.probs) == 1 else
                        f'choose([{", ".join(p2s.fmt_prob(p) for p in r.probs)}])')
                out.append(f'var {coins[r.number]} = mem(function(instance) {{ return {draw}; }});'
                           f'   // by instance, of `{r.text}`')
            fn, loop = f'stage_{rel[members[0]]}', f'stages_{rel[members[0]]}'
            out.append(f'var {fn} = function(rels) {{')
            for i, k in enumerate(members):
                out.append(f'  var {rel[k]} = rels[{i}];')
            self.indexes(self.index_atoms(rules, set(members)), '  ', out)
            self.relations(members, rules, out, '  ', coins, lambda k: f'next_{rel[k]}')
            out.append(f'  return [{", ".join("next_" + rel[k] for k in members)}];')
            out.append('};')
            out.append(f'var {loop} = function(rels, stage, before) {{')
            out.append('  var size = sum(map(function(r) { return _.size(r); }, rels));')
            out.append(f'  return stage < size || size !== before ? {loop}({fn}(rels), stage + 1, size) : rels;')
            out.append('};')
            out.append(f'var rels_{rel[members[0]]} = {loop}([{", ".join("{}" for _ in members)}], 0, -1);')
            for i, k in enumerate(members):
                out.append(f'var {rel[k]} = rels_{rel[members[0]]}[{i}];')
            out.append('')

        def holds(a):
            return f'holds({rel[key(a)]}, [{", ".join(self.const(t) for t in a[1])}])'
        for i in range(len(prog.queries)):
            out.append(f'// component {i} bool')
        out.append('var model = function() {')
        for a, v in prog.evidence:
            out.append(f'  condition({"" if v else "!"}{holds(a)}());')
        out.append('  return [' + ',\n          '.join(holds(a) + '()' for a in prog.queries) + '];')
        out.append('};')
        out.append("Infer({method: 'enumerate'}, model);")
        return '\n'.join(out)

    def text(self):
        return WEBPPL_PRELUDE.replace('{source}', self.source) + self.program() + '\n'


WEBPPL_PRELUDE = '''\
// {source}: a ProbLog program written in WebPPL by tddlog's
// scripts/problog2ppl.py, first order as it is written. A relation is an
// object from a tuple's key to {t: the tuple, v: whether it holds}, v a
// function of no arguments that says whether the tuple holds in the world
// being enumerated, memoized within it; each probabilistic fact is a coin of
// its own (`flip`), drawn where a world first reads it; a rule's instances
// are maps over the relations its body joins. The facts are lpFacts, which
// the runner reads from the facts file: {relation: [[argument ...,
// probability], ...]}. The queries are enumerated exactly, given the
// evidence.

var yes = function() { return true; };
var no = function() { return false; };
// a coin of probability p, drawn once in a world
var coin = function(p) { return mem(function() { return flip(p); }); };
var and = function(xs) { return function() { return all(function(x) { return x(); }, xs); }; };
var not = function(x) { return function() { return !x(); }; };
var keyOf = function(t) { return JSON.stringify(t); };
// whether the tuple t of r holds
var holds = function(r, t) { var e = r[keyOf(t)]; return e ? e.v : no; };
// the entry of t, if it is a tuple of r: none or one to map over
var probe = function(r, t) { var e = r[keyOf(t)]; return e ? [e] : []; };
// the entries of r by their values at positions ps (from 0)
var index = function(r, ps) {
  return _.groupBy(map(function(e) { return {k: keyOf(_.at(e.t, ps)), t: e.t, v: e.v}; }, _.values(r)), 'k');
};
// whether a tuple of the index's key k holds
var holdsAny = function(ix, k) {
  var es = ix[keyOf(k)] || [];
  return function() { return any(function(e) { return e.v(); }, es); };
};
// the choice of an annotated disjunction p1::h1; ...; pn::hn: whether each
// head is the one chosen, the k-th with probability pk and none with the rest
var choose = function(ps) {
  var rest = 1 - sum(ps);
  var vs = _.range(ps.length + (rest > 0 ? 1 : 0));
  var c = mem(function() { return categorical({ps: rest > 0 ? ps.concat([rest]) : ps, vs: vs}); });
  return map(function(k) { return function() { return c() === k; }; }, _.range(ps.length));
};
// the instances, [relation, tuple, holding] each, of relation `name`
var only = function(name, instances) { return filter(function(i) { return i[0] === name; }, instances); };
// the relation of base and instances: a tuple holds where its fact or one
// of its instances does
var relation = function(base, instances) {
  var entries = map(function(e) { return {k: keyOf(e.t), t: e.t, v: e.v}; }, _.values(base)).concat(
    map(function(i) { return {k: keyOf(i[1]), t: i[1], v: i[2]}; }, instances));
  return mapObject(function(k, es) {
    var vs = map(function(e) { return e.v; }, es);
    return {t: es[0].t, v: mem(function() { return any(function(x) { return x(); }, vs); })};
  }, _.groupBy(entries, 'k'));
};
// a relation of facts, [argument ..., probability] each
var relationOfFacts = function(rows) {
  return _.fromPairs(map(function(row) {
    var t = row.slice(0, row.length - 1), p = row[row.length - 1];
    return [keyOf(t), {t: t, v: p >= 1 ? yes : coin(p)}];
  }, rows));
};

'''

# --- Psi -----------------------------------------------------------------------

class Psi(Backend):
    """The Psi program (the module's docstring): Psi has arrays, `for` and
    `while` loops and no dictionaries or symbols, so a relation is a column
    of integers for each argument and a column of Booleans for whether each
    tuple holds, looked up by a scan; a symbol is a number of its own (the
    program's comment lists them), the facts are arrays in the program, and
    its value is the array of the queries' holdings, which the harness asks
    one entry a run (`return queries[K];`), as Psi's own programs are asked."""
    SEP = '_'
    RESERVED = set('''def if else for in while return observe assert flip true false array repeat cobserve
categorical uniformInt bernoulli gauss uniform exponential beta gamma laplace cauchy pareto studentT weibull
rayleigh poisson binomial geometric exp log sin cos abs floor ceil sqrt main import readCSV infer sampleFrom
expectation Expectation Marginal Distribution length lambda forget with dat queries evidence stage before
string i'''.split())
    # the names of the program's own variables (`fresh`), which a relation's name must not take
    OWN = re.compile(OWN.pattern + r'|[vfkcgqn]\d+|(stage|before)\d+')
    TESTS = {'eq': '==', 'neq': '!=', 'aeq': '==', 'aneq': '!=', '<': '<', '>': '>', '<=': '<=', '>=': '>='}
    RETURN = 'return queries;'

    def __init__(self, prog, source):
        super().__init__(prog, source)
        self.rel = idents(prog.preds, self.RESERVED, self.OWN)
        syms, ints = set(), [0]

        def scan(t):
            if t[0] == 'sym':
                syms.add(t[1])
            elif t[0] == 'int':
                ints.append(abs(t[1]))
            elif t[0] == 'num':
                raise Refused('translator: a number that is not an integer, which a Psi column of integers '
                              f'cannot hold: {t[1]}')
            elif t[0] in ('neg', 'op'):
                for u in t[1:]:
                    if isinstance(u, tuple):
                        scan(u)
        for rows in prog.preds.values():
            for args in rows:
                for t in args:
                    scan(t)
        for r in prog.rules:
            for _, args in r.heads:
                for t in args:
                    scan(t)
            for s in r.steps:
                if isinstance(s, Atom):
                    for role in s.roles:
                        if role[0] == 'const':
                            scan(role[1])
                else:
                    for t in s[2:]:
                        if isinstance(t, tuple):
                            scan(t)
        for a in list(prog.queries) + [a for a, _ in prog.evidence]:
            for t in a[1]:
                scan(t)
        # a symbol's number lies past every integer the program has, so that no term equality mistakes one
        base = max(10 ** 6, 10 ** len(str(max(ints))) * 10)
        self.sym = {s: base + k for k, s in enumerate(sorted(syms))}

    def const(self, t):
        if t[0] == 'int':
            return str(t[1])
        if t[0] == 'sym':
            return str(self.sym[t[1]])
        raise Refused(f'translator: the constant {t}')

    def neg(self, a):
        return f'-{a}'

    def arith(self, op, a, b):
        if op in ('//', 'mod'):
            raise Refused(f'translator: the integer operation {op} in Psi')
        return f'({a} {op} {b})'

    def col(self, rel, i):
        return f'{rel}_{i + 1}'

    def match(self, rel, k, pairs):
        return ' && '.join(f'{self.col(rel, i)}[{k}] == {v}' for i, v in pairs) or 'true'

    def add(self, out, ind, rel, args, value, fresh):
        """The tuple args of rel holds also where value does."""
        v, f, k = fresh('v'), fresh('f'), fresh('k')
        out.append(f'{ind}{v} := {value};')
        out.append(f'{ind}{f} := false;')
        out.append(f'{ind}for {k} in [0..{rel}_h.length) {{')
        out.append(f'{ind}  if {self.match(rel, k, list(enumerate(args)))} {{ {rel}_h[{k}] = {rel}_h[{k}] || {v}; '
                   f'{f} = true; }}')
        out.append(f'{ind}}}')
        grow = ''.join(f' {self.col(rel, i)} = {self.col(rel, i)} ~ [{a}];' for i, a in enumerate(args))
        out.append(f'{ind}if !{f} {{{grow} {rel}_h = {rel}_h ~ [{v}]; }}')

    def holds(self, out, ind, rel, args, name):
        """name: whether the tuple args of rel holds."""
        k = name + '_k'
        out.append(f'{ind}{name} := false;')
        out.append(f'{ind}for {k} in [0..{rel}_h.length) {{ if {self.match(rel, k, list(enumerate(args)))} '
                   f'{{ {name} = {rel}_h[{k}]; }} }}')

    def coin(self, out, ind, table, inst, draw, kind, fresh):
        """c: the draw of the instance inst, kept in table across stages."""
        c, got, k = fresh('c'), fresh('g'), fresh('k')
        out.append(f'{ind}{c} := {"false" if kind == "𝔹" else "0"};')
        out.append(f'{ind}{got} := false;')
        out.append(f'{ind}for {k} in [0..{table}_c.length) {{ if {self.match(table, k, list(enumerate(inst)))} '
                   f'{{ {c} = {table}_c[{k}]; {got} = true; }} }}')
        grow = ''.join(f' {self.col(table, i)} = {self.col(table, i)} ~ [{a}];' for i, a in enumerate(inst))
        out.append(f'{ind}if !{got} {{ {c} = {draw};{grow} {table}_c = {table}_c ~ [{c}]; }}')
        return c

    def categorical(self, probs):
        ps = [p2s.fmt_prob(p) for p in probs]
        if sum(probs) < 1 - 1e-12:
            ps.append('1 - ' + ' - '.join(ps))
        return f'categorical([{", ".join(ps)}])'

    def rule(self, r, out, ind, write, coins, fresh):
        names = var_names(r.variables, self.RESERVED)
        out.append(f'{ind}// {r.text}')
        vals, closes, depth = [], [], [ind]

        def open_(line):
            out.append(f'{depth[0]}{line} {{')
            closes.append(depth[0])
            depth[0] += '  '
        for s in r.steps:
            if isinstance(s, Atom):
                rel = self.rel[s.pred]
                bound = [(i, self.role_value(s.roles[i], names)) for i in s.bound]
                if s.negated:   # whether a tuple of the bound values holds (any one, where a position is anonymous)
                    n, k = fresh('n'), fresh('k')
                    out.append(f'{depth[0]}{n} := false;')
                    out.append(f'{depth[0]}for {k} in [0..{rel}_h.length) {{ if {self.match(rel, k, bound)} '
                               f'{{ {n} = {n} || {rel}_h[{k}]; }} }}')
                    vals.append(f'!{n}')
                    continue
                k = fresh('k')
                open_(f'for {k} in [0..{rel}_h.length)')
                first = {}
                for i, role in enumerate(s.roles):
                    if role[0] == 'bind':
                        first[role[1]] = i
                same = [(i, f'{self.col(rel, first[role[1]])}[{k}]') for i, role in enumerate(s.roles)
                        if role[0] == 'same']
                if bound or same:
                    open_(f'if {self.match(rel, k, bound + same)}')
                for v, i in first.items():
                    out.append(f'{depth[0]}{names[v]} := {self.col(rel, i)}[{k}];')
                x = fresh('x')
                out.append(f'{depth[0]}{x} := {rel}_h[{k}];')
                vals.append(x)
            elif s[0] == 'test':
                open_(f'if {self.term(s[2], names)} {self.TESTS[s[1]]} {self.term(s[3], names)}')
            elif s[0] == 'let':
                if not closes:   # a block of the rule's own, so that its variable is the rule's alone
                    open_('if true')
                out.append(f'{depth[0]}{names[s[1]]} := {self.term(s[2], names)};')
            else:
                open_(f'for {names[s[1]]} in [{self.term(s[2], names)}..{self.term(s[3], names)} + 1)')
        d = depth[0]
        body = [] if r.demand else vals
        inst = [names[v] for v in r.variables]
        heads = [(write[h], [self.term(t, names) for t in args]) for h, args in r.heads]
        if r.probs is None:
            self.add(out, d, heads[0][0], heads[0][1], ' && '.join(body) or 'true', fresh)
        elif len(r.probs) == 1:
            draw = f'flip({p2s.fmt_prob(r.probs[0])})'
            if coins:
                c = self.coin(out, d, coins, inst, draw, '𝔹', fresh)
            else:
                c = fresh('c')
                out.append(f'{d}{c} := {draw};')
            self.add(out, d, heads[0][0], heads[0][1], ' && '.join(body + [c]), fresh)
        else:
            draw = self.categorical(r.probs)
            if coins:
                c = self.coin(out, d, coins, inst, draw, 'ℕ', fresh)
            else:
                c = fresh('c')
                out.append(f'{d}{c} := {draw};')
            for i, (rel, args) in enumerate(heads):
                self.add(out, d, rel, args, ' && '.join(body + [f'{c} == {i}']), fresh)
        for c in reversed(closes):
            out.append(f'{c}}}')

    def empty(self, rel, arity, out, ind, decl=':='):
        for i in range(arity):
            out.append(f'{ind}{self.col(rel, i)} {decl} []: ℤ[];')
        out.append(f'{ind}{rel}_h {decl} []: 𝔹[];')

    def program(self):
        prog, rel = self.prog, self.rel
        count = [0]

        def fresh(hint):
            count[0] += 1
            return f'{hint}{count[0]}'
        ind, out = '  ', ['def main() {']
        if self.sym:
            shown = sorted(self.sym.items(), key=lambda kv: kv[1])
            out.append(f'{ind}// the symbols as numbers: ' + ', '.join(f'{s} {n}' for s, n in shown[:40])
                       + (f', ... ({len(shown)} in all)' if len(shown) > 40 else ''))
        # the facts: a column of each argument, their probabilities, and whether each holds
        for k, rows in prog.preds.items():
            if not rows:
                continue
            name, arity = f'{rel[k]}_facts', k[1]
            rows = [(args, p) for args, p in rows.items() if p > 0]
            out.append(f'{ind}// {k[0]}/{k[1]}: {len(rows)} facts')
            for i in range(arity):
                out.append(f'{ind}{self.col(name, i)} := [{", ".join(self.const(a[i]) for a, _ in rows)}]: ℤ[];')
            out.append(f'{ind}{name}_p := [{", ".join(p2s.fmt_prob(p) for _, p in rows)}]: ℝ[];')
            out.append(f'{ind}{name}_h := array({name}_p.length, false);')
            out.append(f'{ind}for i in [0..{name}_p.length) {{ if {name}_p[i] < 1 {{ {name}_h[i] = flip({name}_p[i]); }} '
                       f'else {{ {name}_h[i] = true; }} }}')
        out.append('')

        def start(k, prefix, decl=':='):
            """prefix's columns as k's facts, or empty."""
            if prog.preds[k]:
                for i in range(k[1]):
                    out.append(f'{ind2}{self.col(prefix, i)} {decl} {self.col(rel[k] + "_facts", i)};')
                out.append(f'{ind2}{prefix}_h {decl} {rel[k]}_facts_h;')
            else:
                self.empty(prefix, k[1], out, ind2, decl)
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            if not recursive:
                ind2 = ind
                for k in members:
                    start(k, rel[k])
                for r in rules:
                    self.rule(r, out, ind, rel, None, fresh)
                out.append('')
                continue
            out.append(f'{ind}// {", ".join(f"{k[0]}/{k[1]}" for k in members)}: recursive; the rules applied in '
                       'stages from no tuple,')
            out.append(f'{ind}// each reading the stage before, until a stage adds no tuple and the stages number '
                       'the tuples')
            coins = {}
            for r in self.coin_rules(rules):
                coins[r.number] = f'coins{r.number + 1}'
                out.append(f'{ind}// the draws of `{r.text}`, by instance')
                for i in range(len(r.variables)):
                    out.append(f'{ind}{self.col(coins[r.number], i)} := []: ℤ[];')
                out.append(f'{ind}{coins[r.number]}_c := []: {"𝔹" if len(r.probs) == 1 else "ℕ"}[];')
            for k in members:
                self.empty(rel[k], k[1], out, ind)
            size = ' + '.join(f'{rel[k]}_h.length' for k in members)
            stage, before = fresh('stage'), fresh('before')
            out.append(f'{ind}{stage} := 0;')
            out.append(f'{ind}{before} := -1;')
            out.append(f'{ind}while {stage} < {size} || {size} != {before} {{')
            ind2 = ind + '  '
            out.append(f'{ind2}{before} = {size};')
            nxt = {k: f'next_{rel[k]}' for k in members}
            for k in members:
                start(k, nxt[k])
            for r in rules:
                self.rule(r, out, ind2, {**rel, **nxt}, coins.get(r.number), fresh)
            for k in members:
                for i in range(k[1]):
                    out.append(f'{ind2}{self.col(rel[k], i)} = {self.col(nxt[k], i)};')
                out.append(f'{ind2}{rel[k]}_h = {nxt[k]}_h;')
            out.append(f'{ind2}{stage} = {stage} + 1;')
            out.append(f'{ind}}}')
            out.append('')
        qs = []
        for a in prog.queries:
            q = fresh('q')
            self.holds(out, ind, rel[key(a)], [self.const(t) for t in a[1]], q)
            qs.append(q)
        ev = []
        for a, v in prog.evidence:
            e = fresh('e')
            self.holds(out, ind, rel[key(a)], [self.const(t) for t in a[1]], e)
            ev.append(e if v else f'!{e}')
        if ev:
            out.append(f'{ind}observe({" && ".join(ev)});')
        out.append(f'{ind}queries := [{", ".join(qs)}];')
        out.append(f'{ind}{self.RETURN}')
        out.append('}')
        return '\n'.join(out)

    def text(self):
        return PSI_PRELUDE.replace('{source}', self.source) + self.program() + '\n'


PSI_PRELUDE = '''\
// {source}: a ProbLog program written in Psi by tddlog's scripts/problog2ppl.py,
// first order as it is written. A relation is a column of integers for each
// argument and a column of Booleans for whether each tuple holds, a symbol a
// number of its own; each probabilistic fact is a coin of its own (`flip`), the
// facts arrays in the program; each rule is a loop over the relations its body
// joins, a tuple looked up by a scan. The value is the array of the queries'
// holdings, given the evidence (`observe`): one is asked a run, its `return`
// made `return queries[K];`.
//
//   psi --dp PROGRAM.psi
'''


# --- Pluck ---------------------------------------------------------------------

class Pluck(Backend):
    """The Pluck program (the module's docstring): Pluck is a lazy functional
    language of algebraic data, with recursion and no loops, tables or
    files, so a relation is a list of its tuples, each `(Pair TUPLE
    HOLDING)`, a tuple a constructor of its arity (`(T2 a b)`); a symbol is
    a constructor of its own, an integer one of Pluck's naturals; a rule is
    a `concat_map` over the relations its body joins, a recursive component
    a recursion over its stages. Its queries are Pluck's, `cK` the K-th."""
    RESERVED = set('''define define-type lambda fn let if match query Marginal Posterior PosteriorSamples flip
uniform discrete and or not iff inc dec mod geom randnat randnatlist car cdr cdr_safe isempty fold map mapi filter
filteri length range index append append_one list=? take given suspendible-list=? constructor=? constructors_equal
fst snd Y O S Unit True False Nil Cons Pair Sure Chance insert union holds any_holds concat_map between draw
draw_all size_of coin_of attach program found table2 more body rec size before rels stage'''.split())
    # the names of the program's own variables and functions, which a relation's name must not take
    OWN = re.compile(OWN.pattern + r'|[eatrvqk]\d+|rc\d+|(stage|rels|coins|rule)\d+|eq_.*|c_.*|.*_base|ev|m')
    # the largest integer written, as Pluck's unary naturals hold it: (S (S ... (O)))
    LIMIT = 10000
    ARITH = {'+': '+', '-': '-', '*': '*', 'mod': 'mod'}

    def __init__(self, prog, source):
        super().__init__(prog, source)
        self.rel = idents(prog.preds, self.RESERVED, self.OWN)
        self.kinds(prog)

    def kinds(self, prog):
        """How each relation's position and each rule's variable holds its
        values, by the class of the positions the program unifies or
        compares with it (`mode`), and the names of the constructors
        (`sym`). A class that meets arithmetic (an operation or a range)
        holds Pluck's naturals, the integers' values (`nat`); one whose
        integers are only compared, by order too, holds the ranks of its
        integers among them, from 0 (`rank`), which keep every comparison,
        as a user writes a day number in place of yyyymmdd; any other holds
        constructors (`con`): its symbols, and its integers, which the
        program compares only for equality (keys), each a constructor of its
        own, as a user writes a symbol. Pluck compares a constructor by
        `constructor=?` and a natural by its value (`==`); a class holding
        symbols and naturals is refused."""
        parent, kind, fresh, consts = {}, {}, [0], []

        def find(x):
            parent.setdefault(x, x)
            while parent[x] != x:
                parent[x] = parent[parent[x]]
                x = parent[x]
            return x

        def union(a, b):
            a, b = find(a), find(b)
            if a != b:
                parent[a] = b
                kind.setdefault(b, set()).update(kind.pop(a, set()))

        def mark(x, k):
            kind.setdefault(find(x), set()).add(k)

        def node(t, r):
            if t[0] == 'var':
                return ('var', r, t[1])
            fresh[0] += 1
            x = ('term', fresh[0])
            if t[0] in ('sym', 'int'):
                mark(x, t[0])
                consts.append((x, t))
            elif t[0] == 'num':
                raise Refused('translator: a number that is not an integer, which Pluck\'s naturals do not hold')
            elif t[0] == 'neg':
                raise Refused('translator: a negation of a number, which Pluck\'s naturals do not hold')
            elif t[0] == 'op':
                if t[1] not in self.ARITH:
                    raise Refused(f'translator: the integer operation {t[1]} in Pluck')
                mark(x, 'arith')
                for u in t[2:]:
                    mark(node(u, r), 'arith')
            return x
        for k, rows in prog.preds.items():
            for args in rows:
                for i, t in enumerate(args):
                    union(('col', k, i), node(t, None))
        self.test_at = {}   # (rule, step): a node of the class of a test's two sides
        for r in prog.rules:
            n = r.number
            for h, args in r.heads:
                for i, t in enumerate(args):
                    union(('col', h, i), node(t, n))
            for j, s in enumerate(r.steps):
                if isinstance(s, Atom):
                    for i, role in enumerate(s.roles):
                        if role[0] == 'const':
                            union(('col', s.pred, i), node(role[1], n))
                        elif role[0] != 'anon':
                            union(('col', s.pred, i), ('var', n, role[1]))
                elif s[0] == 'test':
                    a, b = node(s[2], n), node(s[3], n)
                    union(a, b)
                    self.test_at[n, j] = a
                    if s[1] not in ('eq', 'neq'):   # an order or an arithmetic comparison (=:=)
                        mark(a, 'order')
                elif s[0] == 'let':
                    union(('var', n, s[1]), node(s[2], n))
                else:
                    mark(('var', n, s[1]), 'arith')
                    mark(node(s[2], n), 'arith')
                    mark(node(s[3], n), 'arith')
        for a in list(prog.queries) + [a for a, _ in prog.evidence]:
            for i, t in enumerate(a[1]):
                union(('col', key(a), i), node(t, None))

        def mode(x):
            marks = kind.get(find(x), set())
            return 'nat' if 'arith' in marks else 'rank' if 'order' in marks else 'con'
        for x in list(parent):
            if 'sym' in kind.get(find(x), set()) and mode(x) != 'con':
                what = f'{x[1][0]}/{x[1][1]}' if x[0] == 'col' else 'a variable'
                raise Refused(f'translator: a position of {what} that holds symbols and is compared by order or '
                              "used in arithmetic, which Pluck's values do not mix")
        domains, cons = {}, set()
        for x, t in consts:
            if t[0] == 'sym' or mode(x) == 'con':
                cons.add(t)
            elif mode(x) == 'rank':
                domains.setdefault(find(x), set()).add(t[1])
            elif t[1] < 0:
                raise Refused('translator: a negative integer in arithmetic, which Pluck\'s naturals do not hold')
            elif t[1] > self.LIMIT:
                raise Refused(f'translator: an integer above {self.LIMIT} in arithmetic, which Pluck holds in '
                              'unary, (S (S ... (O)))')
        self.ranks = {c: {v: i for i, v in enumerate(sorted(vs))} for c, vs in domains.items()}
        if any(len(r) > self.LIMIT + 1 for r in self.ranks.values()):
            raise Refused(f'translator: more than {self.LIMIT + 1} integers compared with each other, whose ranks '
                          'Pluck holds in unary, (S (S ... (O)))')
        self.sym, taken = {}, set()
        for t in sorted(cons):
            base = 'c_' + re.sub(r'\W', '_', str(t[1]))
            name, k = base, 1
            while name in taken:
                k += 1
                name = f'{base}_{k}'
            taken.add(name)
            self.sym[t] = name
        self.class_of, self.mode = find, mode
        self.kind_of = lambda x: 'sym' if mode(x) == 'con' else 'int'

    def term(self, t, names, at=None):
        """A term at the position of node `at` of `kinds` (None: in arithmetic)."""
        return self.const(t, at) if t[0] in GROUND else super().term(t, names)

    def const(self, t, at=None):
        """A constant at the position of node `at` of `kinds` (None: in
        arithmetic): an integer its value, its rank or its constructor, as
        the position's mode says, and a symbol its constructor."""
        if t[0] == 'int' and (at is None or self.mode(at) == 'nat'):
            return str(t[1])
        if t[0] == 'int' and self.mode(at) == 'rank':
            return str(self.ranks[self.class_of(at)][t[1]])
        if t in self.sym:
            return f'({self.sym[t]})'
        raise Refused(f'translator: the constant {t}')

    def arith(self, op, a, b):
        return f'({self.ARITH[op]} {a} {b})'

    def eq(self, kind, a, b):
        return f'({"==" if kind == "int" else "constructor=?"} {a} {b})'

    def conj(self, vals):
        out = '(True)' if not vals else vals[-1]
        for v in reversed(vals[:-1]):
            out = f'(and {v} {out})'
        return out

    def tup(self, items):
        return f'(T{len(items)}{"".join(" " + x for x in items)})'

    def eq_def(self, name, kinds):
        """A function comparing two tuples of `kinds`."""
        n = len(kinds)
        if not n:
            return f'(define ({name} a b) (True))'
        a = ' '.join(f'a{i + 1}' for i in range(n))
        b = ' '.join(f'b{i + 1}' for i in range(n))
        return (f'(define ({name} a b) (match a T{n} {a} => (match b T{n} {b} => '
                f'{self.conj([self.eq(k, f"a{i + 1}", f"b{i + 1}") for i, k in enumerate(kinds)])})))')

    def choice(self, probs):
        """The choice of an annotated disjunction: the index of the head
        chosen, the K-th with probability pK, len(probs) for none, as a
        chain of flips."""
        rest, out = 1.0, []
        for i, p in enumerate(probs):
            last = i == len(probs) - 1 and sum(probs) >= 1 - 1e-12
            if last:
                out.append(str(i))
                break
            q = min(1.0, p / rest) if rest > 0 else 1.0
            out.append(f'(if (flip {literal(q)}) {i}')
            rest -= p
        else:
            out.append(str(len(probs)))
        return ' '.join(out) + ')' * (len(out) - 1)

    def body(self, r, names, out_rel, leaf, fresh):
        """The expression of rule r's body: the list of the values `leaf`
        gives each instance, from the holdings of its atoms."""
        n = r.number

        def go(i, vals):
            if i == len(r.steps):
                return leaf(vals)
            s = r.steps[i]
            if isinstance(s, Atom):
                rel = out_rel(s.pred)
                pats, conds = [], []
                for j, role in enumerate(s.roles):
                    k = self.kind_of(('col', s.pred, j))
                    if role[0] == 'bind':
                        pats.append(names[role[1]])
                        continue
                    a = fresh('a')
                    pats.append(a)
                    if role[0] == 'const':
                        conds.append(self.eq(k, a, self.const(role[1], ('col', s.pred, j))))
                    elif role[0] in ('match', 'same'):
                        conds.append(self.eq(k, a, names[role[1]]))
                arity = len(s.roles)
                if s.negated:
                    t = fresh('t')
                    test = f'(match {t} T{arity}{"".join(" " + p for p in pats)} => {self.conj(conds)})'
                    return go(i + 1, vals + [f'(not (any_holds (lambda {t} -> {test}) {rel}))'])
                e = fresh('e')
                inner = go(i + 1, vals + [f'(snd {e})'])
                if conds:
                    inner = f'(if {self.conj(conds)} {inner} (Nil))'
                return (f'(concat_map (lambda {e} -> (match (fst {e}) T{arity}{"".join(" " + p for p in pats)} => '
                        f'{inner})) {rel})')
            if s[0] == 'test':
                at = self.test_at[n, i]
                kind = self.kind_of(at)
                a, b = self.term(s[2], names, at), self.term(s[3], names, at)
                if s[1] in ('eq', 'neq'):
                    cond = self.eq(kind, a, b)
                elif s[1] in ('aeq', 'aneq'):
                    cond = f'(== {a} {b})'
                else:
                    cond = f'({self.TESTS[s[1]]} {a} {b})'
                if s[1] in ('neq', 'aneq'):
                    cond = f'(not {cond})'
                return f'(if {cond} {go(i + 1, vals)} (Nil))'
            if s[0] == 'let':
                return f'(let ({names[s[1]]} {self.term(s[2], names, ("var", n, s[1]))}) {go(i + 1, vals)})'
            return (f'(concat_map (lambda {names[s[1]]} -> {go(i + 1, vals)}) '
                    f'(between {self.term(s[2], names)} {self.term(s[3], names)}))')
        return go(0, [])

    TESTS = {'<': '<', '>': '>', '<=': '<=', '>=': '>='}

    def program(self):
        prog, rel = self.prog, self.rel
        count = [0]

        def fresh(hint):
            count[0] += 1
            return f'{hint}{count[0]}'
        out = []
        if self.sym:
            out.append('(define-type sym' + ''.join(f' ({c})' for c in self.sym.values()) + ')')
        arities = sorted({k[1] for k in prog.preds} | {len(r.variables) for r in self.coin_rules(prog.rules)})
        for n in arities:
            out.append(f'(define-type tuple{n} (T{n}{"".join(f" a{i + 1}" for i in range(n))}))')
        for k in prog.preds:
            out.append(self.eq_def(f'eq_{rel[k]}', [self.kind_of(('col', k, i)) for i in range(k[1])]))
        recursive_rules = {r.number for members, recursive in prog.strata if recursive
                           for r in prog.rules if r.heads[0][0] in members}
        for r in self.coin_rules(prog.rules):
            if r.number in recursive_rules:
                out.append(self.eq_def(f'eq_k{r.number + 1}', [self.kind_of(('var', r.number, v)) for v in r.variables]))
        out.append('')
        for k, rows in prog.preds.items():
            rows = [(args, p) for args, p in rows.items() if p > 0]
            if not rows:
                continue
            out.append(f';; {k[0]}/{k[1]}: {len(rows)} facts')
            items = [f'(Pair {self.tup([self.const(t, ("col", k, i)) for i, t in enumerate(args)])} '
                     f'{"(Sure)" if p >= 1 else f"(Chance {literal(p)})"})' for args, p in rows]
            out.append(f'(define {rel[k]}_facts')
            out.append('  ' + ''.join(f'(Cons {x}\n  ' for x in items) + '(Nil)' + ')' * len(items) + ')')
        out.append('')
        binds = []   # the program's let, in order

        def bind(name, expr):
            binds.append(f'({name} {expr})')
        for k in prog.preds:
            if prog.preds[k] and any(p > 0 for p in prog.preds[k].values()):
                bind(f'{rel[k]}_base', f'(draw_all {rel[k]}_facts)')

        def base(k):
            return f'{rel[k]}_base' if prog.preds[k] and any(p > 0 for p in prog.preds[k].values()) else '(Nil)'

        def rule_list(r, names, read):
            """The list of r's instances, each the list of its heads (Pair TUPLE HOLDING)."""
            heads = [self.tup([self.term(t, names, ('col', h, i)) for i, t in enumerate(args)]) for h, args in r.heads]

            def leaf(vals):
                body = self.conj([] if r.demand else vals)
                if r.probs is None:
                    return f'(Cons (Cons (Pair {heads[0]} {body}) (Nil)) (Nil))'
                if len(r.probs) == 1:
                    return f'(Cons (Cons (Pair {heads[0]} (and {body} (flip {literal(r.probs[0])}))) (Nil)) (Nil))'
                c = fresh('k')
                items = ''.join(f'(Cons (Pair {h} (and {body} (== {c} {i}))) ' for i, h in enumerate(heads))
                return f'(let ({c} {self.choice(r.probs)}) (Cons {items}(Nil){")" * len(heads)} (Nil)))'
            return self.body(r, names, read, leaf, fresh)

        def instances(r, names, read):
            """The instances of probabilistic rule r in a recursive component, each (Pair KEY (Pair BODY HEADS))."""
            heads = [self.tup([self.term(t, names, ('col', h, i)) for i, t in enumerate(args)]) for h, args in r.heads]
            keyt = self.tup([names[v] for v in r.variables])
            hs = ''.join(f'(Cons {h} ' for h in heads) + '(Nil)' + ')' * len(heads)
            return self.body(r, names, read, lambda vals: f'(Cons (Pair {keyt} (Pair {self.conj(vals)} {hs})) (Nil))',
                             fresh)

        def adds(k, lists):
            """The tuples the rules give relation k: (rule list, head index) each."""
            parts = [f'(map (lambda x -> (index {i} x)) {name})' if not entries else name for name, i, entries in lists]
            out = '(Nil)'
            for p in reversed(parts):
                out = p if out == '(Nil)' else f'(append {p} {out})'
            return out
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            if not recursive:
                lists = {k: [] for k in members}
                for r in rules:
                    names = {v: 'v' + re.sub(r'\W', '_', v) for v in r.variables}
                    name = f'rule{r.number + 1}'
                    bind(name, rule_list(r, names, lambda p: rel[p]))
                    for i, (h, _) in enumerate(r.heads):
                        lists[h].append((name, i, False))
                for k in members:
                    bind(rel[k], f'(union eq_{rel[k]} {base(k)} {adds(k, lists[k])})' if lists[k] else base(k))
                continue
            # a recursive component: the stages, from no tuple, each reading the stage before
            st, loop = fresh('stage'), fresh('rels')
            coins = [r for r in self.coin_rules(rules)]
            inner = []
            for i, k in enumerate(members):
                inner.append(f'({rel[k]} (index {i} (fst rels)))')
            for i, r in enumerate(coins):
                inner.append(f'(coins{r.number + 1} (index {i} (snd rels)))')
            lists = {k: [] for k in members}
            for r in rules:
                names = {v: 'v' + re.sub(r'\W', '_', v) for v in r.variables}
                if r.probs is None:
                    name = f'rule{r.number + 1}'
                    inner.append(f'({name} {rule_list(r, names, lambda p: rel[p])})')
                    for i, (h, _) in enumerate(r.heads):
                        lists[h].append((name, i, False))
                    continue
                name = f'rc{r.number + 1}'
                draw = (f'(lambda -> (flip {literal(r.probs[0])}))' if len(r.probs) == 1 else
                        f'(lambda -> {self.choice(r.probs)})')
                pick = '(lambda c i -> c)' if len(r.probs) == 1 else '(lambda c i -> (== c i))'
                inner.append(f'({name} (attach eq_k{r.number + 1} {draw} {pick} coins{r.number + 1} '
                             f'{instances(r, names, lambda p: rel[p])}))')
                if len({h for h, _ in r.heads}) > 1:
                    raise Refused('translator: an annotated disjunction of several predicates in a recursion, in Pluck')
                lists[r.heads[0][0]].append((f'(snd {name})', 0, True))
            nexts = [f'(union eq_{rel[k]} {base(k)} {adds(k, lists[k])})' for k in members]
            value = (f'(Pair {self.list_of(nexts)} '
                     f'{self.list_of([f"(fst rc{r.number + 1})" for r in coins])})')
            bind(st, '(lambda rels -> (let (' + '\n        '.join(inner) + ')\n      ' + value + '))')
            start = f'(Pair {self.list_of(["(Nil)"] * len(members))} {self.list_of(["(Nil)"] * len(coins))})'
            bind(loop, f'((Y (lambda rec rels stage before -> (let (size (size_of (fst rels))) '
                       f'(if (or (< stage size) (not (== size before))) (rec ({st} rels) (S stage) size) rels)))) '
                       f'{start} 0 1)')
            for i, k in enumerate(members):
                bind(rel[k], f'(index {i} (fst {loop}))')

        def holds(a):
            args = self.tup([self.const(t, ('col', key(a), i)) for i, t in enumerate(a[1])])
            return f'(holds eq_{rel[key(a)]} {rel[key(a)]} {args})'
        qs = []
        for a in prog.queries:
            q = fresh('q')
            bind(q, holds(a))
            qs.append(q)
        ev = [holds(a) if v else f'(not {holds(a)})' for a, v in prog.evidence]
        out.append(';; the evidence and the queries')
        out.append('(define (program)')
        out.append('  (let (' + '\n        '.join(binds) + ')')
        out.append(f'    (Pair {self.conj(ev)} {self.list_of(qs)})))')
        out.append('')
        for i in range(len(qs)):
            ask = f'(Posterior (index {i} (snd m)) (fst m))' if ev else f'(Marginal (index {i} (snd m)))'
            out.append(f'(query c{i} (let ((m (program))) {ask}))')
        return '\n'.join(out)

    def list_of(self, items):
        return ''.join(f'(Cons {x} ' for x in items) + '(Nil)' + ')' * len(items)

    def text(self):
        return PLUCK_PRELUDE.replace('{source}', self.source) + self.program() + '\n'


def literal(p):
    """A probability as Pluck reads it: digits and a point, no exponent."""
    s = format(Decimal(repr(float(p))), 'f')
    return s if '.' in s else s + '.0'


PLUCK_PRELUDE = '''\
;; {source}: a ProbLog program written in Pluck by tddlog's scripts/problog2ppl.py,
;; first order as it is written. A relation is a list of its tuples, each (Pair
;; TUPLE HOLDING), a symbol a constructor of its own and an integer a natural;
;; each probabilistic fact is a coin of its own (`flip`), drawn once; each rule a
;; concat_map over the relations its body joins, a tuple that several instances
;; give holding where one does (`union`). Query cK asks the K-th query, given the
;; evidence.

(define-type weight (Sure) (Chance p))
(define (draw w) (match w Sure => (True) Chance p => (flip p)))
(define (draw_all fs) (map (lambda f -> (Pair (fst f) (draw (snd f)))) fs))
;; rel with the tuple t, holding where v does too
(define (insert eq t v rel)
  (match rel
    Nil => (Cons (Pair t v) (Nil))
    Cons e rest => (if (eq (fst e) t) (Cons (Pair t (or (snd e) v)) rest) (Cons e (insert eq t v rest)))))
(define (union eq rel xs)
  (match xs
    Nil => rel
    Cons x rest => (union eq (insert eq (fst x) (snd x) rel) rest)))
;; whether the tuple t of rel holds; whether one of the tuples that p accepts does
(define (holds eq rel t)
  (match rel
    Nil => (False)
    Cons e rest => (if (eq (fst e) t) (snd e) (holds eq rest t))))
(define (any_holds p rel)
  (match rel
    Nil => (False)
    Cons e rest => (if (p (fst e)) (or (snd e) (any_holds p rest)) (any_holds p rest))))
(define (concat_map f xs)
  (match xs
    Nil => (Nil)
    Cons x rest => (append (f x) (concat_map f rest))))
(define (between lo hi) (if (> lo hi) (Nil) (Cons lo (between (S lo) hi))))
(define (size_of rels) (fold (lambda acc r -> (+ acc (length r))) 0 rels))
;; the coins of a probabilistic rule in a recursion, kept by instance across the stages: the
;; instances xs, each (Pair KEY (Pair BODY HEADS)), with the coin of each KEY in table (a new
;; one drawn where it has none): (Pair TABLE' the heads, each (Pair TUPLE HOLDING))
(define (coin_of eq k table)
  (match table
    Nil => (Pair (False) (False))
    Cons e rest => (if (eq (fst e) k) (Pair (True) (snd e)) (coin_of eq k rest))))
(define (attach eq draw pick table xs)
  (match xs
    Nil => (Pair table (Nil))
    Cons x rest =>
    (let ((found (coin_of eq (fst x) table))
          (c (if (fst found) (snd found) (draw)))
          (table2 (if (fst found) table (Cons (Pair (fst x) c) table)))
          (more (attach eq draw pick table2 rest))
          (body (fst (snd x))))
      (Pair (fst more) (append (mapi (lambda h i -> (Pair h (and body (pick c i)))) (snd (snd x))) (snd more))))))

'''

# --- SPPL ----------------------------------------------------------------------

class SPPL(Backend):
    """The SPPL program (the module's docstring): SPPL's host language is
    Python, whose values a program may compute before its first sample, and
    its probabilistic part is samples, `if` on events of the variables and
    `for` over a range, over arrays of variables, each sampled once. So the
    facts and the joins of the rules are Python constants: each rule's
    matches, a list comprehension over the relations its body joins, each
    (instance, heads, the tuples its body reads); a recursive component's
    tuples, a fixpoint of its rules over the tuples that hold in some world;
    the tuples the queries and the evidence read, and those they read in turn.
    Then a coin is sampled for each probabilistic fact and each instance of a
    probabilistic rule (`discrete`), and each instance of a tuple that is
    read, whose body holds or an earlier instance of its tuple does, sets a
    variable of its own (`atomic`), the last of a tuple's instances its
    holding: an `if` on the event of its body, its coin and the tuples it
    reads, in a `for` over its component's steps. A component's tuples go in
    cycles, the tuples that read each other, each cycle after those it reads,
    in as many stages as it has tuples, each stage reading the one before; a
    tuple that reads no tuple of its own cycle takes one stage. The queries are
    events, which sppl_dice.py asks of the model (`queries_`), given the
    evidence (`condition`)."""
    SEP = '_'
    RESERVED = {'None', 'True', 'False'}
    OWN = re.compile(OWN.pattern + r'|rule\d+|comp\d+')
    TESTS = {'eq': '==', 'neq': '!=', 'aeq': '==', 'aneq': '!=', '<': '<', '>': '>', '<=': '<=', '>=': '>='}

    def __init__(self, prog, source):
        super().__init__(prog, source)
        self.rel = idents(prog.preds, self.RESERVED, self.OWN)
        self.demand = {h for r in prog.rules if r.demand for h, _ in r.heads}

    def const(self, t):
        if t[0] == 'int':
            return str(t[1])
        if t[0] == 'num':
            return repr(float(t[1]))
        # SPPL's compiler reads `~=` as a sample wherever it is written
        return repr(t[1]).replace('~=', '~\\x3d')

    def neg(self, a):
        return f'(-{a})'

    def arith(self, op, a, b):
        return f'idiv_({a}, {b})' if op == '//' else f'({a} % {b})' if op == 'mod' else f'({a} {op} {b})'

    def tup(self, items):
        return '()' if not items else f'({items[0]},)' if len(items) == 1 else f'({", ".join(items)})'

    # SPPL's compiler writes each binary operation back in a parenthesis of its own, and Python's parser
    # refuses more than 200 nested: a sum of lists past CHAIN of them is cat_ of their list (cora's 559 rules
    # of sametitle/2)
    CHAIN = 100

    def chain(self, parts):
        """The lists PARTS concatenated."""
        return ' + '.join(parts) if len(parts) <= self.CHAIN else f'cat_([{", ".join(parts)}])'

    def matches(self, r, inside):
        """The list comprehension of rule r's matches, each (instance, heads,
        refs): the instance its variables' values, the heads (relation,
        tuple), the refs (relation, index of the tuple, 1 where it holds and
        0 where it does not, 1 where the relation is of the component
        `inside`, read at the stage before). A recursive component's
        relations are the lists RELATION_r, and their indexes are built once
        for each application of the rule."""
        names = var_names(r.variables, self.RESERVED)
        clauses, refs, negs, local, count = [], [], [], {}, [0]

        def fresh(hint):
            count[0] += 1
            return f'{hint}{count[0]}'
        for s in r.steps:
            if not isinstance(s, Atom):
                if s[0] == 'test':
                    clauses.append(f'if {self.term(s[2], names)} {self.TESTS[s[1]]} {self.term(s[3], names)}')
                elif s[0] == 'let':
                    clauses.append(f'for {names[s[1]]} in [{self.term(s[2], names)}]')
                else:
                    clauses.append(f'for {names[s[1]]} in range({self.term(s[2], names)}, '
                                   f'{self.term(s[3], names)} + 1)')
                continue
            q, w = self.rel[s.pred], s.pred in inside
            if s.negated and s.pred in self.demand:
                raise Refused('translator: a negation of a relation of the demand')
            ts = f'{q}_r' if w else f'{q}_t'
            k = self.tup([self.role_value(s.roles[i], names) for i in s.bound])
            j = fresh('j')
            if s.probe:
                ix = f'{q}_ri' if w else f'{q}_i'
                if w:
                    local[ix] = f'index_({ts})'
                src = f'[{ix}.get({k})] if {j} is not None'
            elif s.by or s.negated:
                ix = (f'{q}_rby' + ''.join(f'_{i + 1}' for i in s.by)) if w else self.index_name(s)
                if w:
                    local[ix] = f'by_({ts}, {self.tup([str(i) for i in s.by])})'
                src = f'{ix}.get({k}, [])'
            else:
                src = None
            if s.negated:
                n = fresh('n')
                clauses.append(f"for {n} in [[('{q}', {j}, 0, 0) for {j} in {src}]]")
                negs.append(n)
                continue
            pattern, tests = [], []
            for role in s.roles:
                if role[0] == 'bind':
                    pattern.append(names[role[1]])
                elif role[0] == 'same':
                    a = fresh('a')
                    pattern.append(a)
                    tests.append(f'if {a} == {names[role[1]]}')
                else:
                    pattern.append('_')
            if src is None:
                clauses.append(f'for {j}, {self.tup(pattern)} in enumerate({ts})')
            else:
                clauses.append(f'for {j} in {src}')
                if any(role[0] in ('bind', 'same') for role in s.roles):
                    clauses.append(f'for {self.tup(pattern)} in [{ts}[{j}]]')
            clauses += tests
            if s.pred not in self.demand:
                refs.append(f"('{q}', {j}, 1, {int(w)})")
        if not clauses or not clauses[0].startswith('for '):
            clauses.insert(0, 'for _ in [()]')
        inst = self.tup([names[v] for v in r.variables])
        heads = self.tup([f"('{self.rel[h]}', {self.tup([self.term(t, names) for t in args])})"
                          for h, args in r.heads])
        body = '[]' if r.demand else ' + '.join([f'[{", ".join(refs)}]'] + negs)
        text = f'[({inst}, {heads}, {body})' + ''.join(f'\n    {c}' for c in clauses) + ']'
        if local:
            text = f'(lambda {", ".join(local)}: {text})({", ".join(local.values())})'
        return text

    def wrapped(self, name, items):
        """`name = [items]`, a few to a line."""
        lines, line = [], ''
        for x in items:
            if line and len(line) + len(x) > 100:
                lines.append(line)
                line = ''
            line += x + ', '
        lines.append(line)
        return f'{name} = [' + '\n    '.join(x.rstrip() for x in lines).rstrip(',') + ']'

    def program(self):
        prog, rel = self.prog, self.rel
        out, cmd = [], []
        rules_of, comp_of, need = {}, {}, {}
        for r in prog.rules:
            for h, _ in r.heads:
                if r not in rules_of.setdefault(h, []):
                    rules_of[h].append(r)
        for n, (members, _) in enumerate(prog.strata):
            for k in members:
                comp_of[k] = n
        for r in prog.rules:   # the indexes a relation's readers in other components take
            home = comp_of[r.heads[0][0]]
            for s in r.steps:
                if isinstance(s, Atom) and s.indexed() and not (prog.strata[home][1] and comp_of[s.pred] == home):
                    need.setdefault(s.pred, {})[self.index_name(s)] = s.by
        blocks = []   # the blocks of coins, in order: NAME_c0 the first's index, NAME_cd their distributions

        def coins(name, dists):
            prev = blocks[-1] if blocks else None
            out.append(f'{name}_c0 = ' + (f'{prev}_c0 + len({prev}_cd)' if prev else '0'))
            out.append(f'{name}_cd = {dists}')
            blocks.append(name)
        facts = {k: [(a, p) for a, p in rows.items() if p > 0] for k, rows in prog.preds.items()}
        for k, rows in facts.items():
            if not rows:
                continue
            q = rel[k]
            out.append(f'# {k[0]}/{k[1]}: {len(rows)} facts, each a tuple and its probability')
            out.append(self.wrapped(f'{q}_ft', [self.tup([self.const(t) for t in a]) for a, _ in rows]))
            out.append(self.wrapped(f'{q}_fp', [p2s.fmt_prob(p) for _, p in rows]))
            coins(q, f'[{{1: p_, 0: 1 - p_}} for p_ in {q}_fp if p_ < 1]')
            out.append(f'{q}_fc = coins_of_({q}_fp, {q}_c0)')
        out.append('')
        at, arrays, comps = {}, [], []   # comps: each component of derived relations, its name and its relations
        for n, (members, recursive) in enumerate(prog.strata):
            rules = [r for r in prog.rules if r.heads[0][0] in members]

            def tuples(k, made, before=()):
                """The tuples of relation k: `before`, its facts and the heads of its rules' matches `made`."""
                parts = list(before) + ([f'{rel[k]}_ft'] if facts[k] else [])
                if rules_of.get(k):
                    parts.append(f"heads_('{rel[k]}', {self.chain([made(r) for r in rules_of[k]])})")
                    return f'dedup_({self.chain(parts)})'
                return parts[0] if parts else '[]'
            if not recursive:
                for r in rules:
                    out.append(f'# rule {r.number + 1}: {r.text}')
                    out.append(f'rule{r.number + 1}_m = {self.matches(r, set())}')
                for k in members:
                    out.append(f'{rel[k]}_t = {tuples(k, lambda r: f"rule{r.number + 1}_m")}')
            else:
                out.append(f'# {", ".join(f"{k[0]}/{k[1]}" for k in members)}: recursive; the tuples that hold in '
                           'some world, a fixpoint of the rules')
                lists = [f'{rel[k]}_r' for k in members]
                for r in rules:
                    out.append(f'# rule {r.number + 1}: {r.text}')
                    out.append(f'rule{r.number + 1}_g = lambda {", ".join(lists)}: {self.matches(r, set(members))}')
                # each step keeps the tuples before it first, so that a tuple's index stays
                steps = [tuples(k, lambda r: f'rule{r.number + 1}_g(*ts_)', [f'ts_[{i}]'])
                         for i, k in enumerate(members)]
                out.append(f'comp{n + 1}_t = fix_(lambda ts_: {self.tup(steps)}, {self.tup(["[]"] * len(members))})')
                for i, k in enumerate(members):
                    out.append(f'{rel[k]}_t = comp{n + 1}_t[{i}]')
                for r in rules:
                    out.append(f'rule{r.number + 1}_m = rule{r.number + 1}_g(*comp{n + 1}_t)')
            for k in members:
                out.append(f'{rel[k]}_i = index_({rel[k]}_t)')
                for ix, by in need.get(k, {}).items():
                    out.append(f'{ix} = by_({rel[k]}_t, {self.tup([str(i) for i in by])})')
            for r in self.coin_rules(rules):
                m = f'rule{r.number + 1}'
                out.append(f'{m}_k = index_(dedup_([m_[0] for m_ in {m}_m]))')
                ps = [p2s.fmt_prob(p) for p in r.probs]
                dist = [f'{i + 1}: {p}' for i, p in enumerate(ps)]
                if sum(r.probs) < 1 - 1e-12:
                    dist.append(f'0: 1 - ({" + ".join(ps)})')
                coins(m, f'[{{{", ".join(dist)}}}] * len({m}_k)')
            derived = [k for k in members if rules_of.get(k) and k not in self.demand]
            for k in derived:
                q = rel[k]
                parts = [f'facts_({q}_i, {q}_ft, {q}_fc)'] if facts[k] else []
                for r in rules_of[k]:
                    m = f'rule{r.number + 1}'
                    parts.append(f"inst_('{q}', {q}_i, {m}_m, " + (f'{m}_k, {m}_c0)' if r.probs else 'None, 0)'))
                out.append(f'{q}_inst = {self.chain(parts)}')
                out.append(f'{q}_of = by_tuple_({q}_inst, len({q}_t))')
            out.append('')
            for k in members:
                q = rel[k]
                if k in self.demand:
                    continue
                if not rules_of.get(k):
                    at[k] = f"fact_({q}_fc[j_], v_)" if facts[k] else 'None'
                    continue
                c = f'comp{n + 1}'
                at[k] = f"value_({c}_acc, {c}_at, {c}_end, ('{q}', j_), v_, s_)"
            if derived:
                c = f'comp{n + 1}'
                comps.append((c, '{' + ', '.join(f"'{rel[k]}'" for k in derived) + '}'))
                arrays.append(f'{c}_acc = array(len({c}_steps))')
                what = ', '.join(f'{k[0]}/{k[1]}' for k in derived)
                cmd += [f'# {what}: an instance holds where its body or an earlier instance of its tuple does'
                        + (', a cycle of tuples in as many stages as it has tuples' if recursive else ''),
                        f'for k_ in range(len({c}_steps)):',
                        f'    if {c}_ev(k_):',
                        f'        {c}_acc[k_] ~= atomic(loc=1)',
                        '    else:',
                        f'        {c}_acc[k_] ~= atomic(loc=0)']
        # the event of a relation's tuple j holding (v_ 1) or not (0), read at stage s_ (None: the last)
        out.append('at_ = {' + ''.join(f"\n    '{rel[k]}': lambda j_, v_, s_: {e}," for k, e in at.items()
                                      if e != 'None') + '}')
        out.append('ix_ = {' + ', '.join(f"'{rel[k]}': {rel[k]}_i" for k in at) + '}')
        out.append('coin_d_ = ' + (self.chain([f'{b}_cd' for b in blocks]) or '[]'))
        rels = [rel[k] for k in at if rules_of.get(k)]   # the derived relations
        out.append('insts_ = {' + ', '.join(f"'{q}': {q}_inst" for q in rels) + '}')
        out.append('of_ = {' + ', '.join(f"'{q}': {q}_of" for q in rels) + '}')

        def holds(a, v=True):
            return f"holds_('{rel[key(a)]}', {self.tup([self.const(t) for t in a[1]])}, {int(v)})"

        def asked(a):
            return f"('{rel[key(a)]}', {self.tup([self.const(t) for t in a[1]])})"
        seeds = [asked(a) for a in prog.queries] + [asked(a) for a, _ in prog.evidence]
        out.append('# the tuples the queries and the evidence read, and those their instances read in turn')
        out.append(self.wrapped('asked_', seeds))
        out.append('used_ = reach_([(q_, ix_[q_][t_]) for q_, t_ in asked_ if t_ in ix_.get(q_, {})], lambda w_: True)')
        for c, names in comps:
            out.append(f'{c}_steps = steps_({names})')
            out.append(f'{c}_at = at_of_({c}_steps)')
            out.append(f'{c}_end = ends_({c}_steps)')
            out.append(f'{c}_prev = prev_({c}_steps)')
            out.append(f'{c}_ev = lambda k_: ev_(step_({c}_steps, {c}_end, k_), {c}_prev[k_], lambda i_: {c}_acc[i_])')
        out.append('queries_ = lambda: [' + ', '.join(holds(a) for a in prog.queries) + ']')
        if prog.evidence:
            out.append('evidence_ = lambda: all_([' + ', '.join(holds(a, v) for a, v in prog.evidence) + '])')
        out.append('')
        out.append('coins_ = array(len(coin_d_))')
        out += arrays
        out.append('')
        out.append('true_ ~= atomic(loc=1)')
        out.append('for n_ in range(len(coin_d_)):')
        out.append('    coins_[n_] ~= discrete(coin_d_[n_])')
        out += cmd
        if prog.evidence:
            out.append('condition(evidence_())')
        return '\n'.join(out)

    def text(self):
        comps = ''.join(f'# component {k} bool query{k}\n' for k in range(len(self.prog.queries)))
        return (SPPL_PRELUDE.replace('{source}', self.source).replace('{components}', comps)
                + self.program() + '\n')


SPPL_PRELUDE = '''\
# {source}: a ProbLog program written in SPPL by tddlog's scripts/problog2ppl.py,
# first order as it is written. The facts and the joins of the rules are Python
# constants, computed before the first sample: each rule's matches a list
# comprehension over the relations its body joins, a recursive component's
# tuples a fixpoint of its rules, and the tuples that the queries and the
# evidence read. Then each probabilistic fact and each instance of a
# probabilistic rule samples a coin, and each instance of a tuple that is read
# sets a variable of its own, in a `for` over its component's instances: 1
# where its body holds or an earlier instance of its tuple does, the last of a
# tuple's instances its holding. A cycle of tuples that read each other does so
# in as many stages as it has tuples, each stage reading the one before. The
# queries are the events of queries_(), asked of the model given the evidence:
#
#   sppl_dice.py PROGRAM.sppl
#
# queries
{components}
from functools import reduce
from itertools import accumulate

yes_ = lambda: true_ << {1}
no_ = lambda: true_ << {0}
all_ = lambda es_: reduce(lambda a_, b_: a_ & b_, es_) if es_ else yes_()
fix_ = lambda g_, x_: (lambda y_: x_ if y_ == x_ else fix_(g_, y_))(g_(x_))
dedup_ = lambda xs_: list(dict.fromkeys(xs_))
cat_ = lambda ls_: [x_ for l_ in ls_ for x_ in l_]
index_ = lambda ts_: {t_: j_ for j_, t_ in enumerate(ts_)}
by_ = lambda ts_, ps_: reduce(lambda d_, j_: d_.setdefault(tuple(ts_[j_][p_] for p_ in ps_), []).append(j_) or d_,
                              range(len(ts_)), {})
idiv_ = lambda a_, b_: abs(a_) // abs(b_) * (1 if (a_ < 0) == (b_ < 0) else -1)
# the heads of relation q_ that matches ms_ give; a coin's index for each probability below 1
heads_ = lambda q_, ms_: [h_ for m_ in ms_ for r_, h_ in m_[1] if r_ == q_]
coins_of_ = lambda ps_, c0_: [None if p_ >= 1 else c0_ + n_
                              for p_, n_ in zip(ps_, accumulate([0] + [int(x_ < 1) for x_ in ps_]))]
# a relation's instances, each (its tuple's index, its coin and value or None, its refs); their indexes by tuple
facts_ = lambda i_, ts_, cs_: [(i_[t_], None if c_ is None else (c_, 1), []) for t_, c_ in zip(ts_, cs_)]
inst_ = lambda q_, i_, ms_, ks_, c0_: [(i_[h_], None if ks_ is None else (c0_ + ks_[m_[0]], n_ + 1), m_[2])
                                      for m_ in ms_ for n_, (r_, h_) in enumerate(m_[1]) if r_ == q_]
by_tuple_ = lambda es_, n_: (lambda d_: [d_.get((j_,), []) for j_ in range(n_)])(by_(es_, (0,)))
# the tuples (relation, index) that a tuple's instances read; those that tuples us_ read in turn, where ok_
reads_ = lambda u_: ([(r_, k_) for m_ in of_[u_[0]][u_[1]] for r_, k_, _, _ in insts_[u_[0]][m_][2]]
                     if u_[0] in of_ else [])
reach_ = lambda us_, ok_: fix_(lambda s_: s_ | {w_ for u_ in s_ for w_ in reads_(u_) if ok_(w_)}, frozenset(us_))
# a component's steps, each (relation, instance, stage, cycle), over the tuples of its relations c_ that the
# queries and the evidence read: its cycles of tuples (the tuples that reach the same ones), each after the
# cycles it reads (which reach fewer), in as many stages as it has tuples, each stage reading the one before
cycles_ = lambda rs_: sorted(reduce(lambda d_, u_: d_.setdefault(rs_[u_], []).append(u_) or d_, sorted(rs_),
                                    {}).values(), key=lambda us_: (len(rs_[us_[0]]), us_[0]))
steps_ = lambda c_: [(u_[0], m_, s_, g_) for g_, us_ in enumerate(cycles_(
                         {u_: reach_([u_], lambda w_: w_[0] in c_) for u_ in used_ if u_[0] in c_}))
                     for s_ in range(1, len(us_) + 1) for u_ in us_ for m_ in of_[u_[0]][u_[1]]]
# a step's tuple and stage; the step of a tuple's last instance at each stage; a tuple's last stage and its
# cycle; the step before each, where it is an earlier instance of its tuple at its stage
key_ = lambda x_: (x_[0], insts_[x_[0]][x_[1]][0], x_[2])
at_of_ = lambda st_: {key_(x_): k_ for k_, x_ in enumerate(st_)}
ends_ = lambda st_: {key_(x_)[:2]: (x_[2], x_[3]) for x_ in st_}
prev_ = lambda st_: [k_ - 1 if k_ and key_(st_[k_ - 1]) == key_(st_[k_]) else None for k_ in range(len(st_))]
# events: a fact's; a tuple's at stage s_ (None its last, 0 before the first); a step's coin and body, which
# reads the tuples of its own cycle at the stage before; the step's or the one before it
fact_ = lambda c_, v_: (yes_() if v_ else no_()) if c_ is None else coins_[c_] << {v_}
value_ = lambda a_, at_s_, end_, u_, v_, s_: (lambda t_: (no_() if v_ else yes_()) if t_ == 0
                                              else a_[at_s_[u_ + (t_,)]] << {v_})(end_[u_][0] if s_ is None else s_)
coin_ = lambda c_: [] if c_ is None else [coins_[c_[0]] << {c_[1]}]
step_ = lambda st_, end_, k_: (lambda q_, m_, s_, g_: all_(coin_(insts_[q_][m_][1]) + [
    at_[r_](j_, v_, s_ - 1 if w_ and end_[(r_, j_)][1] == g_ else None)
    for r_, j_, v_, w_ in insts_[q_][m_][2]]))(*st_[k_])
ev_ = lambda e_, p_, acc_: e_ if p_ is None else e_ | (acc_(p_) << {1})
holds_ = lambda q_, t_, v_: (lambda j_: (no_() if v_ else yes_()) if j_ is None else at_[q_](j_, v_, None))(
    ix_[q_].get(t_))

'''


# --- Dice ----------------------------------------------------------------------

DICE_KEYWORDS = set('''let in if then else fun flip true false fst snd observe discrete int uniform iterate bool
list sample nth_bit head tail length binomial unit repeat return match with not sum'''.split())
# the names of the program's own values: a node's nK, an observation's obsK, the state s, a function's
# arguments aK, and the functions f_RELATION
DICE_OWN = re.compile(r'[a-z]\d*|[a-z]+\d+|f_.*')
NODE_NAME = re.compile(r'[a-z]\d*|[a-z]+\d+')
# a slot's argument at its step, and at the step before (Chain)
STEP, PREV = ('step', 0), ('step', -1)


def value_of(t, env):
    """The value of term t under the binding env, ('int', n), ('num', x) or
    ('sym', s), as ProbLog's `is` computes it."""
    if t[0] == 'var':
        return env[t[1]]
    if t[0] in GROUND:
        return t
    if t[0] not in ('neg', 'op'):
        raise Refused(f'translator: the term {t}')
    args = [value_of(a, env) for a in (t[1:] if t[0] == 'neg' else t[2:])]
    for a in args:
        if a[0] == 'sym':
            raise Refused(f'arithmetic on the symbol {a[1]} (a type error in ProbLog)')
    if t[0] == 'neg':
        return (args[0][0], -args[0][1])
    op, (a, b) = t[1], args
    if op == '/':
        raise Refused('translator: the division / (a float in ProbLog)')
    if op in ('//', 'mod'):
        if a[0] != 'int' or b[0] != 'int':
            raise Refused(f'{op} of a number that is no integer (a type error in ProbLog)')
        if b[1] == 0:
            raise Refused(f'{op} by zero (an error in ProbLog)')
        if op == 'mod':
            return ('int', a[1] % b[1])
        q = abs(a[1]) // abs(b[1])
        return ('int', q if (a[1] < 0) == (b[1] < 0) else -q)
    v = {'+': a[1] + b[1], '-': a[1] - b[1], '*': a[1] * b[1]}[op]
    return ('int', v) if a[0] == b[0] == 'int' else ('num', float(v))


def test_holds(kind, a, b):
    """Whether the comparison of kind `kind` (TESTS) holds of values a and b."""
    if kind == 'eq':
        return a == b
    if kind == 'neq':
        return a != b
    for x in (a, b):
        if x[0] == 'sym':
            raise Refused(f'an arithmetic comparison of the symbol {x[1]} (a type error in ProbLog)')
    x, y = a[1], b[1]
    return {'aeq': x == y, 'aneq': x != y, '<': x < y, '>': x > y, '<=': x <= y, '>=': x >= y}[kind]


class Instance:
    """A ground instance of a rule: the rule, its binding (the values of its
    variables, in order), its heads [(relation, tuple)], and its body's
    matches {(pos, neg): None}, each the tuples the match reads as (the
    place of its atom in the body, (relation, tuple)): pos those that must
    hold, neg those that must not (a negated atom's tuples; where its
    arguments are all bound, its tuple, possible or not). The instance
    holds where its coin does and one match does."""
    __slots__ = ('rule', 'binding', 'heads', 'alts')

    def __init__(self, rule, binding, heads):
        self.rule, self.binding, self.heads, self.alts = rule, binding, heads, {}


class Ground:
    """The tuples of the program's relations that hold in some world
    (`tuples`), and its rules' instances over them (`insts`), the strata
    in order, a recursive one to its fixpoint; the instances of each tuple
    (`of`), those that read it (`readers`), the tuples that hold in every
    world (`certain`: a certain fact, a tuple of the demand, the head of a
    certain rule's instance that reads only such tuples), and the tuples
    the queries and the evidence read (`asked`)."""

    def __init__(self, prog):
        self.prog = prog
        self.demand = {h for r in prog.rules if r.demand for h, _ in r.heads}
        self.tuples = {k: {} for k in prog.preds}
        self.indexes = {k: {} for k in prog.preds}
        self.insts = {}
        for k, rows in prog.preds.items():
            for args, p in rows.items():
                if p > 0:
                    self.add((k, tuple(args)))
        order = []
        for members, recursive in prog.strata:
            rules = [r for r in prog.rules if r.heads[0][0] in members]
            while any([self.instantiate(r) for r in rules]) and recursive:
                pass
            order.append((rules, recursive))
        # (a tuple of the demand holds in every world: its instances are not read)
        self.of, self.readers, by_rule = {}, {}, {}
        for inst in self.insts.values():
            by_rule.setdefault(inst.rule.number, []).append(inst)
            for x in dict.fromkeys(inst.heads):
                self.of.setdefault(x, []).append(inst)
            for x in dict.fromkeys(y for pos, neg in inst.alts for _, y in pos + neg):
                if not inst.rule.demand:
                    self.readers.setdefault(x, []).append(inst)
        self.certain = {(k, tuple(args)) for k, rows in prog.preds.items() for args, p in rows.items() if p >= 1}
        self.certain |= {(k, t) for k in self.demand for t in self.tuples[k]}
        for rules, recursive in order:
            insts = [inst for r in rules for inst in by_rule.get(r.number, ())]
            grown = True
            while grown:
                grown = False
                for inst in insts:
                    if inst.rule.probs or all(h in self.certain for h in inst.heads):
                        continue
                    if any(all(y in self.certain for _, y in pos) and not any(self.exists(y) for _, y in neg)
                           for pos, neg in inst.alts):
                        self.certain.update(inst.heads)
                        grown = recursive
        self.asked = {(key(a), tuple(a[1])) for a in prog.queries} | {(key(a), tuple(a[1])) for a, _ in prog.evidence}

    def exists(self, x):
        return x[1] in self.tuples[x[0]]

    def add(self, x):
        k, t = x
        if t in self.tuples[k]:
            return False
        self.tuples[k][t] = None
        for by, ix in self.indexes[k].items():
            ix.setdefault(tuple(t[j] for j in by), []).append(t)
        return True

    def lookup(self, k, by, vals):
        """The tuples of relation k whose arguments at positions `by` are `vals`."""
        if len(by) == k[1]:
            return [vals] if vals in self.tuples[k] else []
        if not by:
            return list(self.tuples[k])
        ix = self.indexes[k].get(tuple(by))
        if ix is None:
            ix = self.indexes[k][tuple(by)] = {}
            for t in self.tuples[k]:
                ix.setdefault(tuple(t[j] for j in by), []).append(t)
        return list(ix.get(vals, ()))

    def instantiate(self, r):
        """Adds rule r's instances over the tuples so far, and their heads;
        whether a tuple was added."""
        grown = False
        for env, pos, neg in self.matches(r):
            binding = tuple(env[v] for v in r.variables)
            inst = self.insts.get((r.number, binding))
            if inst is None:
                heads = [(h, tuple(env[t[1]] if t[0] == 'var' else t for t in args)) for h, args in r.heads]
                inst = self.insts[(r.number, binding)] = Instance(r, binding, heads)
                for x in heads:
                    grown = self.add(x) or grown
            inst.alts[(pos, neg)] = None
        return grown

    def matches(self, r):
        """Each (binding, pos, neg) of a match of rule r's body (Instance)."""
        out, steps = [], r.steps

        def walk(i, env, pos, neg):
            while i < len(steps) and not isinstance(steps[i], Atom):
                s = steps[i]
                if s[0] == 'test':
                    if not test_holds(s[1], value_of(s[2], env), value_of(s[3], env)):
                        return
                elif s[0] == 'let':
                    env = {**env, s[1]: value_of(s[2], env)}
                else:
                    lo, hi = value_of(s[2], env), value_of(s[3], env)
                    if lo[0] != 'int' or hi[0] != 'int':
                        raise Refused('between/3 of a number that is no integer (a type error in ProbLog)')
                    for v in range(lo[1], hi[1] + 1):
                        walk(i + 1, {**env, s[1]: ('int', v)}, pos, neg)
                    return
                i += 1
            if i == len(steps):
                out.append((env, pos, neg))
                return
            s = steps[i]
            vals = tuple(s.roles[j][1] if s.roles[j][0] == 'const' else env[s.roles[j][1]] for j in s.bound)
            if s.negated:
                if s.pred in self.demand:
                    raise Refused('translator: a negation of a relation of the demand')
                found = [vals] if s.probe else self.lookup(s.pred, s.bound, vals)
                walk(i + 1, env, pos, neg + tuple((i, (s.pred, t)) for t in found))
                return
            for t in self.lookup(s.pred, s.bound, vals):
                e = dict(env)
                if all(e.setdefault(role[1], t[j]) == t[j] for j, role in enumerate(s.roles)
                       if role[0] in ('bind', 'same')):
                    walk(i + 1, e, pos if s.pred in self.demand else pos + ((i, (s.pred, t)),), neg)
        walk(0, {}, (), ())
        return out


class DiceGraph(g2n.Graph):
    """ground2net.py's and-inverter graph, with Dice's other values as
    nodes: ('fact', p) a flip; ('choice', [p...]) a discrete; ('pick', c,
    i) whether choice c is i; ('input', name) a function's argument;
    ('fst', v), ('snd', v), ('tuple', signals); ('call', f, state, args)
    and ('iterate', f, state, n). A node of a key (a fact's, a coin's) is
    made once."""

    def __init__(self):
        super().__init__()
        self.keyed = {}

    def node(self, node, key=None):
        key = node if key is None else key
        k = self.keyed.get(key)
        if k is None:
            self.nodes.append(node)
            k = self.keyed[key] = len(self.nodes) - 1
        return k


def children(node):
    kind = node[0]
    if kind in ('and', 'tuple'):
        return [abs(s) for s in node[1]]
    if kind in ('pick', 'fst', 'snd'):
        return [node[1]]
    if kind == 'call':
        return [abs(node[2])] + [abs(a) for a in node[3]]
    if kind == 'iterate':
        return [abs(node[2])]
    return []


def nested(items):
    """Values as Dice's nested pairs, (a, (b, c))."""
    out = items[-1]
    for x in reversed(items[:-1]):
        out = f'({x}, {out})'
    return out


def dice_type(n):
    """The type of n Booleans as nested pairs."""
    return nested(['bool'] * n)


def project(g, v, i, n):
    """The i-th of the n Booleans of nested pairs v."""
    if n == 1:
        return v
    for _ in range(min(i, n - 1)):
        v = g.node(('snd', v))
    return v if i == n - 1 else g.node(('fst', v))


def choice_probs(probs):
    """An annotated disjunction's choice as Dice's discrete: its heads'
    probabilities, and none of them with the rest, as decimals."""
    ps = [Decimal(g2n.decimal(p)) for p in probs]
    rest = 1 - sum(ps)
    if rest < Decimal('-1e-4'):   # Dice's tolerance
        raise Refused(f'an annotated disjunction whose probabilities sum to {sum(ps)}')
    return [format(p, 'f') for p in ps + ([rest] if rest > 0 else [])]


def coin_node(g, key, probs):
    """A probabilistic rule instance's coin, a flip, or an annotated
    disjunction's choice, a discrete."""
    if len(probs) == 1:
        return g.node(('fact', probs[0]), key)
    return g.node(('choice', choice_probs(probs)), key)


def instance_signal(g, probs, heads, head, alts, read, coin):
    """The signal that an instance gives `head`, of its rule's probabilities
    `probs` and its heads: its coin (where an annotated disjunction's, a
    choice of `head`) and one of its matches, each (pos, neg) of refs that
    read() reads."""
    body = g.disj([g.conj([read(y) for y in pos] + [-read(y) for y in neg]) for pos, neg in alts])
    if not probs or body == FALSE or (len(probs) == 1 and probs[0] >= 1):
        return body
    c = coin()
    if len(probs) == 1:
        return g.conj([c, body])
    return g.conj([g.disj([g.node(('pick', c, i)) for i, h in enumerate(heads) if h == head]), body])


class Scope:
    """A Dice scope, the program's body or a function's: its graph, and its
    nodes' names, a tuple's where one names it (`hint`), nK otherwise. A
    conjunction of negations is written as the disjunction it negates."""

    def __init__(self, fnames):
        self.g, self.names, self.taken, self.fnames = DiceGraph(), {}, set(), fnames

    def inverted(self, k):
        node = self.g.nodes[k]
        return node[0] == 'and' and all(s < 0 for s in node[1])

    def hint(self, s, text):
        """Names signal s's node after text where the node's value is s's."""
        k = abs(s)
        if k == TRUE or k in self.names or self.g.nodes[k][0] in ('input', 'tuple') or (s > 0) == self.inverted(k):
            return
        base = re.sub(r'[^A-Za-z0-9_]', '_', text)[:48]
        name, n = base, 1
        while name in self.taken or name in DICE_KEYWORDS or name in self.fnames or NODE_NAME.fullmatch(name):
            n += 1
            name = f'{base}_v{n}'
        self.taken.add(name)
        self.names[k] = name

    def ref(self, s):
        k = abs(s)
        if k == TRUE:
            return 'true' if s > 0 else 'false'
        name = self.names.get(k, f'n{k}')
        return name if (s > 0) != self.inverted(k) else f'(!{name})'

    def expr(self, k):
        node = self.g.nodes[k]
        kind = node[0]
        if kind == 'fact':
            return f'flip {g2n.decimal(node[1])}'
        if kind == 'choice':
            return f'discrete({", ".join(node[1])})'
        if kind == 'pick':
            return f'{self.ref(node[1])} == int({bits_for(len(self.g.nodes[node[1]][1]))}, {node[2]})'
        if kind == 'and':
            if self.inverted(k):
                return ' || '.join(self.ref(-s) for s in node[1])
            return ' && '.join(self.ref(s) for s in node[1])
        if kind in ('fst', 'snd'):
            return f'{kind} {self.ref(node[1])}'
        if kind == 'tuple':
            return nested([self.ref(s) for s in node[1]])
        if kind == 'call':
            return f'{node[1]}({", ".join([self.ref(node[2])] + [self.ref(a) for a in node[3]])})'
        if kind == 'iterate':
            return f'iterate({node[1]}, {self.ref(node[2])}, {node[3]})'
        raise AssertionError(node)

    def lets(self, roots):
        """The lets of the nodes the signals `roots` read, in order, and those nodes."""
        seen, stack = set(), [abs(s) for s in roots]
        while stack:
            k = stack.pop()
            if k not in seen:
                seen.add(k)
                stack += children(self.g.nodes[k])
        cone = sorted(seen)
        return [f'let {self.names.get(k, f"n{k}")} = {self.expr(k)} in' for k in cone
                if self.g.nodes[k][0] not in ('true', 'input')], cone


class Abstract:
    """A rule's instances at the layers of a Chain with the step left out:
    its key (the rule's number and its binding, STEP or PREV for a variable
    that is the layer or the layer before at every layer), its rule, its
    heads (slots), its matches at each layer it has (`at`) and at any
    (`alts`), each (pos, neg) of refs: ('same', slot) a slot of the layer,
    ('prev', slot) one of the layer before, ('ext', tuple) a tuple of
    another component, STEP and PREV in place of the layer."""

    def __init__(self, key, rule, heads):
        self.key, self.rule, self.heads, self.at, self.alts = key, rule, heads, {}, {}


def layering(gr, pos, insts):
    """((the number of layers, the reads within a layer negated), the layers
    in order, {layer: the layer before}) of a recursive component's
    instances `insts` where its relations' steps are at positions `pos`, or
    None: each instance's heads in one layer, reading the component's
    tuples of that layer or of the one before, the layers one sequence."""
    pred, succ, within = {}, {}, 0
    for inst in insts:
        layer = inst.heads[0][1][pos[inst.heads[0][0]]][1]
        if any(h[1][pos[h[0]]][1] != layer for h in inst.heads[1:]):
            return None
        for refs, _ in inst.alts:
            for _, y in refs:
                if y[0] in pos:
                    m = y[1][pos[y[0]]][1]
                    if m == layer:
                        within += 1
                    elif pred.setdefault(layer, m) != m or succ.setdefault(m, layer) != layer:
                        return None
    layers = {t[p][1] for k, p in pos.items() for t in gr.tuples[k]}
    starts = [layer for layer in layers if layer not in pred]
    if len(starts) != 1:
        return None
    order = starts
    while order[-1] in succ and len(order) <= len(layers):
        order.append(succ[order[-1]])
    if len(order) != len(layers) or set(order) != layers:
        return None
    return (len(order), -within), order, pred


def find_chain(dice, members):
    """(The Chain of recursive component `members` or None, the steps'
    positions of its best layering where it has one but no Chain fits, so
    that the program says so), the Chain: an argument of
    each relation that is an integer in every tuple, its step, the layers
    one sequence (`layering`), whose function (Chain.plan) takes the fewest
    arguments, then has the most layers and the fewest reads within a
    layer. (A sum's argument is in layers too, mnist_sum's partial(K, S),
    each sum read by itself and the one before; but each instance reads the
    sum of a value, so that every one is an argument of a function over
    it.)"""
    gr = dice.gr
    live = [k for k in members if gr.tuples[k]]
    insts = [inst for inst in gr.insts.values() if inst.heads[0][0] in members]
    cands = []
    for k in live:
        cands.append([j for j in range(k[1]) if all(t[j][0] == 'int' for t in gr.tuples[k])])
        if not cands[-1]:
            return None, None
    found = []
    for combo in itertools.islice(itertools.product(*cands), 256):
        pos = dict(zip(live, combo))
        layers = layering(gr, pos, insts)
        if layers is not None and len(layers[1]) >= 2:
            found.append((layers[0], pos, layers[1], layers[2]))
    found.sort(key=lambda f: f[0], reverse=True)
    best = None
    for score, pos, order, pred in found[:8]:
        chain = Chain(dice, live, pos, order, pred, insts)
        if chain.chosen is not None and (best is None or len(chain.chosen['params']) < len(best.chosen['params'])):
            best = chain
    return best, found[0][1] if found else None


class Chain:
    """A recursion over a step: a recursive component whose tuples are in
    layers, the values of an integer argument of each of its relations, each
    layer read by itself and by the next alone. The layers from the first
    (s 0) or the second (s 1) on are computed by one function of the layer
    before, `f_RELATION`, whose state is the slots (tuples with STEP in
    place of their layer) that the next layer or the rest of the program
    reads. Its rules are the instances of a layer with the step left out
    (Abstract), the same at each layer, and a coin of a layer's own (a
    probabilistic fact, or a rule instance that reads only certain tuples,
    that nothing else reads, of one probability at every layer) is a flip
    in the function, drawn at each call. What differs between the layers is
    an argument of the function: a tuple of another component that is
    shared or asked; a fact whose probability differs; an instance that a
    layer does not have and could otherwise hold there (a guard). Without
    arguments, the layers are `iterate`s of the function, each ending where
    the rest of the program reads a layer; with them, a call a layer."""

    def __init__(self, dice, members, pos, order, pred, insts):
        self.dice, self.gr = dice, dice.gr
        self.members, self.pos, self.order, self.pred, self.insts = members, pos, order, pred, insts
        self.fname, self.text, self.states = None, None, None
        plans = [p for p in (self.plan(0), self.plan(1)) if p is not None]
        self.chosen = min(plans, key=lambda p: (len(p['params']), p['s'])) if plans else None

    def layer(self, x):
        return x[1][self.pos[x[0]]][1]

    def slot(self, x):
        p = self.pos[x[0]]
        return (x[0], tuple(STEP if j == p else t for j, t in enumerate(x[1])))

    def conc(self, a, layer):
        """Slot or abstract tuple a at a layer: None where it reads the layer
        before and the layer has none."""
        if PREV in a[1] and layer not in self.pred:
            return None
        return (a[0], tuple(('int', layer) if t == STEP else ('int', self.pred[layer]) if t == PREV else t
                            for t in a[1]))

    def plan(self, s):
        """The function from layer order[s] on: its abstract instances, its
        state, the classes of its other reads, its arguments and the layers
        the rest of the program reads (its breaks), or None where a layer's
        instances are not those of the function."""
        gr, pred = self.gr, self.pred
        F = self.order[s:]
        if len(F) < 2:
            return None
        at = {layer: i for i, layer in enumerate(F)}
        by_rule = {}
        for inst in self.insts:
            layer = self.layer(inst.heads[0])
            if layer in at:
                by_rule.setdefault(inst.rule.number, []).append((layer, inst))
        inside = {id(inst) for lst in by_rule.values() for _, inst in lst}
        abstract = {}
        for n, lst in by_rule.items():
            r = lst[0][1].rule
            kind = {}
            for j in range(len(r.variables)):
                if all(inst.binding[j] == ('int', layer) for layer, inst in lst):
                    kind[j] = STEP
                elif all(layer in pred and inst.binding[j] == ('int', pred[layer]) for layer, inst in lst):
                    kind[j] = PREV
            var_at = {v: j for j, v in enumerate(r.variables)}

            def ref(a, y, layer):
                if y[0] in self.pos:
                    return ('same' if self.layer(y) == layer else 'prev', self.slot(y))
                roles = r.steps[a].roles
                return ('ext', (y[0], tuple(kind.get(var_at[roles[j][1]], t) if roles[j][0] in ('match', 'bind', 'same')
                                            else t for j, t in enumerate(y[1]))))
            for layer, inst in lst:
                key = (n, tuple(kind.get(j, b) for j, b in enumerate(inst.binding)))
                heads = tuple(self.slot(h) for h in inst.heads)
                A = abstract.get(key)
                if A is None:
                    A = abstract[key] = Abstract(key, r, heads)
                elif A.heads != heads:
                    return None
                A.at[layer] = {(tuple(ref(a, y, layer) for a, y in pos), tuple(ref(a, y, layer) for a, y in neg)): None
                               for pos, neg in inst.alts}
        produce = {}
        for A in abstract.values():
            for layer in F:
                A.alts.update(A.at.get(layer, {}))
            for h in dict.fromkeys(A.heads):
                produce.setdefault(h, []).append(A)
        # the slots the rest of the program reads, and the layers where it reads them
        outside, breaks = {}, set()
        for k in self.members:
            for t in gr.tuples[k]:
                x = (k, t)
                if self.layer(x) in at and (x in gr.asked or any(id(i) not in inside for i in gr.readers.get(x, ()))):
                    outside[self.slot(x)] = None
                    breaks.add(at[self.layer(x)])
        if not breaks:
            return None
        # the slots the function computes, and those its state carries: those read outside or by the next layer
        needed, state, work = dict(outside), dict(outside), list(outside)
        while work:
            for A in produce.get(work.pop(), ()):
                for refs, _ in A.alts:
                    for kind_, v in refs:
                        if kind_ == 'prev':
                            state[v] = None
                        if kind_ != 'ext' and v not in needed:
                            needed[v] = None
                            work.append(v)
        used = list({id(A): A for v in needed for A in produce.get(v, ())}.values())
        # each tuple of another component that the function reads: the layers whose instances read it,
        # and its class: true, false, a coin of the layer's own, or an argument
        uses, seen = {}, {}
        for A in used:
            for layer, alts in A.at.items():
                for refs, negs in alts:
                    for kind_, v in refs + negs:
                        if kind_ == 'ext':
                            uses.setdefault(v, {})[layer] = None
                            seen.setdefault(layer, {}).setdefault(self.conc(v, layer), {})[v] = None
        cls = {}
        for e, layers in uses.items():
            xs = [self.conc(e, layer) for layer in layers]
            if all(x in gr.certain for x in xs):
                cls[e] = TRUE
            elif not any(gr.exists(x) for x in xs):
                cls[e] = FALSE
            else:
                ps = set()
                for layer, x in zip(layers, xs):
                    p = self.dice.lone(x)
                    if (p is None or x in gr.asked or len(seen[layer][x]) > 1 or (STEP not in e[1] and PREV not in e[1])
                            or any(id(i) not in inside or self.layer(i.heads[0]) != layer
                                   for i in gr.readers.get(x, ()))):
                        ps = None
                        break
                    ps.add(p)
                cls[e] = ('flip', ps.pop()) if ps and len(ps) == 1 else 'arg'

        def possible(refs, layer):
            """Whether a match's refs `refs` may all hold at a layer in the function."""
            for kind_, v in refs:
                if kind_ == 'ext' and cls[v] != 'arg':   # true, false or a flip
                    if cls[v] == FALSE:
                        return False
                    continue
                if kind_ == 'same':
                    x = self.conc(v, layer)
                elif kind_ == 'prev':
                    x = self.conc(v, pred[layer]) if layer in pred else None
                else:   # an argument, false at a layer whose instances do not read it
                    x = self.conc(v, layer) if layer in uses[v] else None
                if x is None or not gr.exists(x):
                    return False
            return True
        guards = []
        for A in used:
            for layer in F:
                if layer in A.at:
                    if any(alt not in A.at[layer] and possible(alt[0], layer) for alt in A.alts):
                        return None
                elif any(possible(alt[0], layer) for alt in A.alts):
                    guards.append(A)
                    break
        facts = {}
        for v in needed:
            ps = [self.dice.prog.preds[v[0]].get(self.conc(v, layer)[1], 0) for layer in F]
            if any(p > 0 for p in ps):
                facts[v] = TRUE if all(p >= 1 for p in ps) else ('flip', ps[0]) if len(set(ps)) == 1 else 'arg'
        params = ([('guard', A) for A in guards] + [('ext', e) for e, c in cls.items() if c == 'arg']
                  + [('fact', v) for v, c in facts.items() if c == 'arg'])
        state = sorted(state)
        return {'s': s, 'F': F, 'at': at, 'state': state, 'index': {v: i for i, v in enumerate(state)},
                'needed': needed, 'produce': produce, 'cls': cls, 'uses': uses, 'facts': facts, 'params': params,
                'guards': {A.key for A in guards}, 'breaks': sorted(breaks)}

    def function(self):
        """The function's text: from the state of the layer before (`s`) and
        the arguments (aK), the state of the layer."""
        if self.text is not None:
            return self.text
        p, dice = self.chosen, self.dice
        sc = Scope(dice.fnames)
        g = sc.g
        n = len(p['state'])
        s_in = g.node(('input', 's'))
        sc.names[s_in] = 's'
        args = {}
        for i, (kind_, v) in enumerate(p['params']):
            args[(kind_, v.key if kind_ == 'guard' else v)] = a = g.node(('input', f'a{i + 1}'))
            sc.names[a] = f'a{i + 1}'

        def name(v, prev=False):
            return dice.tuple_name((v[0], tuple(PREV if prev and t == STEP else t for t in v[1])))

        def ref(r_, read):
            kind_, v = r_
            if kind_ == 'same':
                return read(v)
            if kind_ == 'prev':
                x = project(g, s_in, p['index'][v], n)
                sc.hint(x, name(v, True))
                return x
            c = p['cls'][v]
            if c in (TRUE, FALSE):
                return c
            x = args[('ext', v)] if c == 'arg' else g.node(('fact', c[1]), ('ext', v))
            sc.hint(x, name(v))
            return x

        def compute(v, read):
            c = p['facts'].get(v)
            parts = [FALSE if c is None else TRUE if c == TRUE else args[('fact', v)] if c == 'arg'
                     else g.node(('fact', c[1]), ('fact', v))]
            for A in p['produce'].get(v, ()):
                sig = instance_signal(g, A.rule.probs, A.heads, v, A.alts, lambda r_: ref(r_, read),
                                      lambda A=A: coin_node(g, ('coin', A.key), A.rule.probs))
                if A.key in p['guards']:
                    sig = g.conj([args[('guard', A.key)], sig])
                parts.append(sig)
            return g.disj(parts)
        # the slots that read each other within a layer, in stages
        deps = {v: {w for A in p['produce'].get(v, ()) for refs, _ in A.alts for kind_, w in refs if kind_ == 'same'}
                for v in p['needed']}
        comp_of = {}
        for comp in strongly_connected(deps):
            if len(comp) > 1 or comp[0] in deps[comp[0]]:
                for v in comp:
                    comp_of[v] = comp
        memo = {}

        def slot(v):
            if v not in memo:
                if v in comp_of:
                    stage = g2n.fixpoint(comp_of[v], lambda h, st: compute(h, lambda y: st[y] if y in st else slot(y)))
                else:
                    stage = {v: compute(v, slot)}
                for h, x in stage.items():
                    memo[h] = x
                    sc.hint(x, name(h))
            return memo[v]
        outs = [slot(v) for v in p['state']]
        lets, _ = sc.lets(outs)
        head = ', '.join([f's: {dice_type(n)}'] + [f'a{i + 1}: bool' for i in range(len(p['params']))])
        self.text = (f'fun {self.fname}({head}) {{\n' + ''.join(f'  {line}\n' for line in lets)
                     + f'  {nested([sc.ref(o) for o in outs])}\n}}\n')
        return self.text

    def args(self, layer):
        """The function's arguments at a layer."""
        p, dice = self.chosen, self.dice
        out = []
        for kind_, v in p['params']:
            if kind_ == 'guard':
                out.append(TRUE if layer in v.at else FALSE)
            elif kind_ == 'ext':
                out.append(dice.define(self.conc(v, layer)) if layer in p['uses'][v] else FALSE)
            else:
                out.append(dice.fact(self.conc(v, layer)))
        return out

    def build(self):
        """The calls of the function in the program's body, from the first
        layer to the last the rest of the program reads, a state at each
        layer it reads (each break)."""
        p, dice = self.chosen, self.dice
        g = dice.main.g
        F, n = p['F'], len(p['state'])
        init = [FALSE] * n if p['s'] == 0 else [dice.define(self.conc(v, self.order[0])) for v in p['state']]
        cur = init[0] if n == 1 else g.node(('tuple', tuple(init)))
        self.states, i = {}, 0
        while i <= p['breaks'][-1]:
            j = i if p['params'] else next(b for b in p['breaks'] if b >= i)
            call = ('call', self.fname, cur, tuple(self.args(F[i]))) if j == i else ('iterate', self.fname, cur, j - i + 1)
            cur = self.states[j] = g.node(call, ('call', self.fname, i))
            i = j + 1

    def read(self, x):
        """The signal of member tuple x of a layer of the function, which the
        rest of the program reads."""
        if self.states is None:
            self.build()
        p = self.chosen
        st = self.states[p['at'][self.layer(x)]]
        return project(self.dice.main.g, st, p['index'][self.slot(x)], len(p['state']))


class Dice(Backend):
    """The Dice program (the module's docstring): Dice has Booleans, tuples,
    functions and `iterate`, and no relations, data or recursion, so the
    program is written over the tuples that hold in some world (Ground). A
    tuple is a let of its fact's flip or its instances, an instance its coin
    (a flip, or an annotated disjunction's discrete) and its matches. A
    recursion over a step is a function from one layer's tuples to the
    next's (Chain), and a recursion through data is unrolled in stages, as
    ground2net.py unrolls it (`fixpoint`). The program returns its queries
    as a tuple and observes its evidence."""
    RESERVED = DICE_KEYWORDS

    def __init__(self, prog, source):
        super().__init__(prog, source)
        self.rel = idents(prog.preds, DICE_KEYWORDS, DICE_OWN)
        self.gr = Ground(prog)
        self.fnames = set()
        self.main = Scope(self.fnames)
        self.memo, self.cover, self.comp, self.chains, self.notes = {}, {}, {}, [], []
        for members, recursive in prog.strata:
            members = [k for k in members if k not in self.gr.demand]
            if not recursive or not members:
                continue
            what = ', '.join(f'{k[0]}/{k[1]}' for k in members)
            staged = [(k, t) for k in members for t in self.gr.tuples[k]]
            if not staged:
                self.notes.append(f'{what}: a recursion none of whose tuples holds in any world')
                continue
            chain, layered = find_chain(self, members)
            if chain is None and layered is None:
                self.notes.append(f'{what}: a recursion through data, its tuples unrolled in stages')
            elif chain is None:
                steps = ', '.join(f'{k[0]}/{k[1]} argument {p + 1}' for k, p in layered.items())
                self.notes.append(f'{what}: a recursion over a step ({steps}) that no function of the step '
                                  'writes, its tuples unrolled in stages')
            else:
                fname = f'f_{self.rel[chain.members[0]]}'
                while fname in self.fnames:
                    fname += '_'
                chain.fname = fname
                self.fnames.add(fname)
                self.chains.append(chain)
                p = chain.chosen
                for x in staged:
                    if chain.layer(x) in p['at']:
                        self.cover[x] = chain
                staged = [x for x in staged if x not in self.cover]
                steps = ', '.join(f'{k[0]}/{k[1]} argument {chain.pos[k] + 1}' for k in chain.members)
                how = (f'a call a step with {len(p["params"])} arguments' if p['params']
                       else 'iterated between the steps the program reads')
                self.notes.append(f'{what}: a recursion over a step ({steps}), {fname} from step {p["F"][0]}, {how}')
            deps = {x: set() for x in staged}
            for x in staged:
                for inst in self.gr.of.get(x, ()):
                    deps[x].update(y for refs, _ in inst.alts for _, y in refs if y in deps)
            for comp in strongly_connected(deps):
                if len(comp) > 1 or comp[0] in deps[comp[0]]:
                    for x in comp:
                        self.comp[x] = comp

    def tuple_name(self, x):
        parts = [self.rel[x[0]]]
        for t in x[1]:
            if t == STEP:
                parts.append('t')
            elif t == PREV:
                parts.append('tp')
            elif t[0] == 'int':
                parts.append(f'm{-t[1]}' if t[1] < 0 else str(t[1]))
            elif t[0] == 'num':
                parts.append(repr(float(t[1])).replace('-', 'm').replace('.', 'p'))
            else:
                parts.append(t[1])
        return '_'.join(parts)

    def fact(self, x):
        p = self.prog.preds[x[0]].get(x[1], 0)
        return FALSE if p <= 0 else TRUE if p >= 1 else self.main.g.node(('fact', p), ('fact', x))

    def lone(self, x):
        """The probability of tuple x where it is a coin of its own: a fact
        that no rule gives, or the head of one instance of a probabilistic
        rule of one head, with no fact, whose match reads only certain
        tuples; else None."""
        p = self.prog.preds[x[0]].get(x[1], 0)
        insts = self.gr.of.get(x, [])
        if not insts:
            return p if 0 < p < 1 else None
        if p > 0 or len(insts) != 1:
            return None
        r = insts[0].rule
        if not r.probs or len(r.probs) != 1 or len(insts[0].heads) != 1:
            return None
        if any(all(y in self.gr.certain for _, y in pos) and not any(self.gr.exists(y) for _, y in neg)
               for pos, neg in insts[0].alts):
            return r.probs[0]
        return None

    def concrete(self, x, read):
        """Tuple x's signal: its fact or one of its instances."""
        g = self.main.g
        parts = [self.fact(x)]
        for inst in self.gr.of.get(x, ()):
            parts.append(instance_signal(
                g, inst.rule.probs, inst.heads, x, [([y for _, y in pos], [y for _, y in neg]) for pos, neg in inst.alts],
                read, lambda inst=inst: coin_node(g, ('coin', inst.rule.number, inst.binding), inst.rule.probs)))
        return g.disj(parts)

    def define(self, x):
        """Tuple x's signal in the program's body."""
        s = self.memo.get(x)
        if s is not None:
            return s
        if x in self.gr.certain:
            s = TRUE
        elif not self.gr.exists(x):
            s = FALSE
        elif x in self.cover:
            s = self.cover[x].read(x)
        elif x in self.comp:
            stage = g2n.fixpoint(self.comp[x], lambda h, st: self.concrete(h, lambda y: st[y] if y in st else self.define(y)))
            for h, v in stage.items():
                self.memo[h] = v
                self.main.hint(v, self.tuple_name(h))
            return stage[x]
        else:
            s = self.concrete(x, self.define)
        self.memo[x] = s
        self.main.hint(s, self.tuple_name(x))
        return s

    def program(self, results, observed):
        """The program returning the signals `results` given `observed` [(signal, value)]."""
        sc = self.main
        lets, cone = sc.lets([s for s, _ in observed] + results)
        called = {sc.g.nodes[k][1] for k in cone if sc.g.nodes[k][0] in ('call', 'iterate')}
        out = [DICE_PRELUDE.replace('{source}', self.source) + ''.join(f'// {note}\n' for note in self.notes)]
        out += [chain.function() for chain in self.chains if chain.fname in called]
        out += [line + '\n' for line in lets]
        out += [f'let obs{j} = observe ({sc.ref(s if v else -s)}) in\n' for j, (s, v) in enumerate(observed)]
        out.append(nested([sc.ref(s) for s in results]) + '\n')
        return ''.join(out)

    def signals(self):
        if not self.prog.queries:
            raise Refused('translator: a program that asks no query')
        observed = [(self.define((key(a), tuple(a[1]))), v) for a, v in self.prog.evidence]
        return [self.define((key(a), tuple(a[1]))) for a in self.prog.queries], observed

    def text(self):
        results, observed = self.signals()
        return self.program(results, observed)

    def each(self):
        """A program a query: [(the query as bench.py keys it, the program)]."""
        results, observed = self.signals()
        return [(p2s.atom_text(a), self.program([s], observed)) for a, s in zip(self.prog.queries, results)]


DICE_PRELUDE = '''\
// {source}: a ProbLog program written in Dice by tddlog's scripts/problog2ppl.py,
// over the tuples of its relations that hold in some world. A tuple is a let:
// its fact's flip, or one of its rule instances, an instance its coin (a flip,
// or an annotated disjunction's discrete) and one of its matches. A recursion
// over a step (an argument that is the layer of each tuple, each layer read by
// itself and the next) is a function from the state of one layer to the next,
// iterated, its rules the instances of a step with the step left out; a
// recursion through data is unrolled in stages, each applying the rules to the
// stage before. The program returns its queries and observes its evidence.
'''

LANGS = {'alea': Julia, 'roulette': Racket, 'webppl': WebPPL, 'psi': Psi, 'pluck': Pluck, 'sppl': SPPL, 'dice': Dice}
# the languages whose programs hold their facts themselves: no facts file
INLINE = {'psi', 'pluck', 'sppl', 'dice'}


def translate(lang, text, source='a ProbLog program'):
    """(the program in LANG, the lines of its facts file (none for a
    language of INLINE), the queries as tddlog prints them)."""
    prog = read(text)
    text = LANGS[lang](prog, source).text()
    return text, [] if lang in INLINE else data_lines(prog), [p2s.atom_text(a) for a in prog.queries]


def dicemap(path, out):
    """Writes a Dice program a query of the program at `path`, beside OUT
    (STEM.dicemap) as STEM__qJ.dice, and OUT, a line `STEM__qJ.dice bool
    QUERY` each, as bench.py's run_dice_circuit reads them; the queries."""
    prog = read(open(path).read())
    stem = out[:-len('.dicemap')] if out.endswith('.dicemap') else out
    lines = []
    for j, (q, text) in enumerate(Dice(prog, os.path.basename(path)).each()):
        with open(f'{stem}__q{j}.dice', 'w') as f:
            f.write(text)
        lines.append(f'{os.path.basename(stem)}__q{j}.dice bool {q}\n')
    with open(out, 'w') as f:
        f.write(''.join(lines))
    return [p2s.atom_text(a) for a in prog.queries]


def main(argv):
    if argv[:1] == ['dicemap'] and len(argv) == 3:
        queries = dicemap(argv[1], argv[2])
    else:
        if not argv or argv[0] not in LANGS or len(argv) != (3 if argv[0] in INLINE else 4):
            sys.exit(__doc__)
        lang, path, out = argv[:3]
        text, rows, queries = translate(lang, open(path).read(), os.path.basename(path))
        if lang not in INLINE:
            with open(argv[3], 'w') as f:
                f.write(''.join(line + '\n' for line in rows))
        with open(out, 'w') as f:
            f.write(text)
    sys.stdout.write(''.join(q + '\n' for q in queries))


if __name__ == '__main__':
    try:
        # Dice's translation recurses as deep as the program's chains of tuples
        deep(main, sys.argv[1:])
    except (Refused, g2n.Refused) as x:
        print(f'problog2ppl.py: {x}', file=sys.stderr)
        sys.exit(REFUSED)
