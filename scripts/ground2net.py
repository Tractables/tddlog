#!/usr/bin/env python3
"""Translate a ground ProbLog program, as `tddlog --ground` prints it, into
a Bayesian network of Boolean variables and into Dice.

    tddlog --ground prog.pl > prog.ground.pl
    ground2net.py prog.ground.pl OUT --format uai|hugin|dice|dicemap

writes OUT.uai and OUT.uai.evid (a UAI BAYES file and its evidence), OUT.net
and OUT.inst (Hugin, for Ace), OUT.dice (one Dice program returning every
query as a tuple), or OUT__qJ.dice (one Dice program per query) listed in
OUT.dicemap, and for every format OUT.json: the network's variables in
order and, for each query of the ground program, the variable and the
value whose marginal answers it. This is how the systems that read
networks or Dice programs are given a program that is not a network or a
Dice program.

The ground program's probabilities sit on independent facts: `--ground`
has already turned every annotated disjunction and every probabilistic rule
into switch facts and plain rules. Each fact becomes a root, and each
derived atom the disjunction of its rules' bodies, a body the conjunction
of its literals, built as an and-inverter graph: conjunctions of signals,
a signal a node or its negation, with constants folded and equal
conjunctions shared. In the network a conjunction of more than two signals
is a chain of conjunctions of two, so that no table has more than three
variables, and each table is deterministic; in Dice it is one `&&`.

A recursive component (atoms that depend on each other) is unrolled into
stages: stage 0 has every atom of the component false, and stage s applies
the rules once to stage s - 1, the atoms outside the component at their
final values. Within a world each stage before the least fixpoint adds an
atom, so the stage numbered by the component's size is the least model, and
the translation is exact; it stops earlier where a stage's formulas are
the previous stage's. A component with negation inside it has a
well-founded rather than a least model; tddlog computes it
(`src/tp/aft.rs`), and this translation refuses it (exit status 3), as it
does a network of more than LIMIT conjunctions.

The ground program leaves out the weight tddlog factors out of the facts
it simplifies (`GroundProgram.weight`); it is a constant factor of every
world, so it cancels in every conditional marginal asked here.
"""
import argparse
import itertools
import json
import os
import sys
from decimal import Decimal

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import source_tree  # noqa: E402,F401  (the source tree's converters)
import gen_bn  # noqa: E402
from lpad2problog import split_top  # noqa: E402

REFUSED = 3
LIMIT = 2_000_000
TRUE, FALSE = 1, -1   # node 1 is the constant true


class Refused(Exception):
    """A program this translation cannot express exactly."""


def literals(body):
    """The literals of a body as tddlog prints it, `a, \\+b`, as (atom,
    positive)."""
    if '(' in body or "'" in body or '"' in body:
        parts = split_top(body, ',')
    else:
        parts = body.split(',')
    out = []
    for p in parts:
        p = p.strip()
        out.append((p[2:].strip(), False) if p.startswith('\\+') else (p, True))
    return out


def parse(text):
    """(facts {atom: p}, rules {head: [body]}, evidence [(atom, value)],
    queries [(key, atom, positive)]) of a ground program, one clause a
    line as `GroundProgram`'s Display writes them."""
    facts, rules, evidence, queries = {}, {}, [], []
    for line in text.splitlines():
        c = line.strip()
        if not c or c.startswith('%'):
            continue
        if not c.endswith('.'):
            raise ValueError(f'not a clause of a ground program: {c}')
        c = c[:-1]
        if c.startswith('query(') and c.endswith(')'):
            q = c[6:-1].strip()
            positive = not q.startswith('\\+')
            queries.append((''.join(q.split()), q if positive else q[2:].strip(), positive))
        elif c.startswith('evidence(') and c.endswith(')'):
            atom, _, value = c[9:-1].rpartition(',')
            if value.strip() not in ('true', 'false'):
                raise ValueError(f'not an observation: {c}')
            evidence.append((atom.strip(), value.strip() == 'true'))
        else:
            head, neck, body = c.partition(' :- ')
            prob, colons, atom = head.partition('::')
            if colons and not neck:
                if atom in facts:
                    raise ValueError(f'a fact stated twice: {c}')
                facts[atom] = float(prob)
            else:
                rules.setdefault(head, []).append(literals(body) if neck else [])
    return facts, rules, evidence, queries


class Graph:
    """An and-inverter graph: node 1 the constant true, every other node a
    fact (`('fact', p)`) or a conjunction of signals (`('and', inputs)`), a
    signal +k or -k for node k or its negation."""

    def __init__(self):
        self.nodes = [None, ('true',)]
        self.ands = {}

    def fact(self, p):
        self.nodes.append(('fact', p))
        return len(self.nodes) - 1

    def conj(self, inputs):
        kept = set()
        for s in inputs:
            if s == FALSE:
                return FALSE
            if s != TRUE:
                kept.add(s)
        if any(-s in kept for s in kept):
            return FALSE
        if not kept:
            return TRUE
        if len(kept) == 1:
            return kept.pop()
        key = tuple(sorted(kept))
        k = self.ands.get(key)
        if k is None:
            if len(self.ands) >= LIMIT:
                raise Refused(f'the network would have more than {LIMIT} conjunctions')
            self.nodes.append(('and', key))
            k = self.ands[key] = len(self.nodes) - 1
        return k

    def disj(self, inputs):
        return -self.conj([-s for s in inputs])


def components(rules):
    """The strongly connected components of the derived atoms, each after
    the components it depends on (Tarjan's algorithm, without recursion)."""
    deps = {h: [a for body in bodies for a, _ in body if a in rules] for h, bodies in rules.items()}
    index, low, stack, on, out = {}, {}, [], set(), []
    for root in rules:
        if root in index:
            continue
        work = [(root, 0)]
        while work:
            v, i = work.pop()
            if i == 0:
                index[v] = low[v] = len(index)
                stack.append(v)
                on.add(v)
            if i < len(deps[v]):
                work.append((v, i + 1))
                w = deps[v][i]
                if w not in index:
                    work.append((w, 0))
                elif w in on:
                    low[v] = min(low[v], index[w])
                continue
            if low[v] == index[v]:
                comp = []
                while True:
                    w = stack.pop()
                    on.discard(w)
                    comp.append(w)
                    if w == v:
                        break
                out.append(comp)
            if work:
                u = work[-1][0]
                low[u] = min(low[u], low[v])
    return out


def build(text):
    """(graph, observations [(node, value)], queries [(key, signal)]) of a
    ground program."""
    facts, rules, evidence, queries = parse(text)
    g = Graph()
    sig = {a: g.fact(p) for a, p in facts.items()}

    def lit(atom, positive, stage=None):
        s = stage[atom] if stage is not None and atom in stage else sig.get(atom, FALSE)
        return s if positive else -s

    def define(h, stage=None):
        bodies = [g.conj([lit(a, pos, stage) for a, pos in body]) for body in rules[h]]
        return g.disj(bodies + ([sig[h]] if h in facts else []))

    for comp in components(rules):
        members = set(comp)
        if len(comp) == 1 and not any(a == comp[0] for body in rules[comp[0]] for a, _ in body):
            sig[comp[0]] = define(comp[0])
            continue
        for h in comp:
            for body in rules[h]:
                for a, pos in body:
                    if not pos and a in members:
                        raise Refused(f'negation through recursion ({h} and {a}): the program has a '
                                      'well-founded model, not a least one')
        stage = {h: FALSE for h in comp}
        for _ in range(len(comp)):
            nxt = {h: define(h, stage) for h in comp}
            if nxt == stage:
                break
            stage = nxt
        sig.update(stage)
    asked = [(key, sig.get(a, FALSE) if pos else -sig.get(a, FALSE)) for key, a, pos in queries]
    # each observed node once, with the value its observations give it; an
    # observation that contradicts another or a constant leaves the constant
    # true observed false, which no world satisfies
    need = {}
    for a, v in evidence:
        s = sig.get(a, FALSE)
        k, value = abs(s), (s > 0) == v
        if k == TRUE and value:
            continue
        if k == TRUE or need.get(k, value) != value:
            return g, [(TRUE, False)], asked
        need[k] = value
    return g, list(need.items()), asked


def cone(g, signals):
    """The nodes the signals depend on, in the graph's order."""
    seen, stack = set(), [abs(s) for s in signals]
    while stack:
        k = stack.pop()
        if k in seen:
            continue
        seen.add(k)
        if g.nodes[k][0] == 'and':
            stack += [abs(s) for s in g.nodes[k][1]]
    return sorted(seen)


VALUES = ['false', 'true']


def network(g, observed, asked):
    """The network in gen_bn.py's form, (order, variables, tables), its
    observations [(variable, value)], and each query's (variable, value)."""
    order, variables, tables = [], {}, {}

    def add(name, parents, rows):
        order.append(name)
        variables[name] = VALUES
        tables[name] = (parents, rows)

    def ref(s):
        return f'n{abs(s)}', VALUES[s > 0]

    for k in cone(g, [s for s, _ in observed] + [s for _, s in asked]):
        node = g.nodes[k]
        if node[0] == 'true':
            add(f'n{k}', [], {(): [0.0, 1.0]})
        elif node[0] == 'fact':
            add(f'n{k}', [], {(): fact_row(node[1])})
        else:
            # a chain of conjunctions of two: n<k>_<j> and, last, n<k>
            cur = ref(node[1][0])
            for j, s in enumerate(node[1][1:], 1):
                nxt = ref(s)
                name = f'n{k}' if j == len(node[1]) - 1 else f'n{k}_{j}'
                rows = {(u, w): [0.0, 1.0] if (u, w) == (cur[1], nxt[1]) else [1.0, 0.0]
                        for u, w in itertools.product(VALUES, VALUES)}
                add(name, [cur[0], nxt[0]], rows)
                cur = (name, 'true')
    evidence = [(f'n{abs(s)}', VALUES[(s > 0) == v]) for s, v in observed]
    return (order, variables, tables), evidence, {key: ref(s) for key, s in asked}


def fact_row(p):
    """A fact's row, [1 - p, p], as two doubles whose decimals sum to one.
    gen_bn.py's writers check each row's sum in the decimals its entries
    mean (`gen_bn.meant_decimal`: a double within two units in its last
    place of a decimal of at most 12 digits means that decimal). Near a
    short decimal, the larger entry of [1.0 - p, p] can be within two of
    its units of that decimal while the smaller is more than two of its
    own, smaller, units from the complement, and the row means a sum
    that rounding does not explain (1 + 2.6e-16 on bb_GPA_w19, 1 + 3e-16
    on uwcse2). So the larger entry is the decimal it means, and the smaller
    one minus that decimal, which moves p by at most three units in the
    last place of the larger."""
    big = gen_bn.meant_decimal(max(p, 1.0 - p))
    small = float(1 - big)
    return [float(big), small] if p < 0.5 else [small, float(big)]


def decimal(p):
    """A probability as Dice reads it: a decimal without an exponent."""
    return format(Decimal(repr(float(p))), 'f')


def dice(g, observed, results):
    """A Dice program returning the signals `results` (nested pairs, in
    order) given the observations, defining only the nodes they depend on."""
    def ref(s):
        if abs(s) == TRUE:
            return 'true' if s > 0 else 'false'
        return f'n{s}' if s > 0 else f'(!n{-s})'
    out = []
    for k in cone(g, [s for s, _ in observed] + list(results)):
        node = g.nodes[k]
        if node[0] == 'fact':
            out.append(f'let n{k} = flip {decimal(node[1])} in')
        elif node[0] == 'and':
            out.append(f"let n{k} = {' && '.join(ref(s) for s in node[1])} in")
    for j, (s, v) in enumerate(observed):
        out.append(f'let obs{j} = observe ({ref(s if v else -s)}) in')
    result = ref(results[-1])
    for s in reversed(results[:-1]):
        result = f'({ref(s)}, {result})'
    out.append(result)
    return '\n'.join(out) + '\n'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('ground', help='the ground program, as tddlog --ground prints it')
    ap.add_argument('out', help='the stem of the files written')
    ap.add_argument('--format', choices=['uai', 'hugin', 'dice', 'dicemap'], required=True)
    a = ap.parse_args()
    try:
        with open(a.ground) as f:
            g, observed, asked = build(f.read())
    except Refused as x:
        print(f'ground2net.py: {x}', file=sys.stderr)
        sys.exit(REFUSED)
    if not asked:
        sys.exit('ground2net.py: the program asks no query')
    net, evidence, cells = network(g, observed, asked)
    with open(a.out + '.json', 'w') as f:
        json.dump({'order': net[0], 'queries': cells}, f)
    if a.format == 'uai':
        model, evid = gen_bn.write_uai(net, evidence)
        with open(a.out + '.uai', 'w') as f:
            f.write(model)
        with open(a.out + '.uai.evid', 'w') as f:
            f.write(evid)
    elif a.format == 'hugin':
        with open(a.out + '.net', 'w') as f:
            f.write(gen_bn.write_hugin(net))
        with open(a.out + '.inst', 'w') as f:
            f.write(gen_bn.write_inst(evidence))
    elif a.format == 'dice':
        with open(a.out + '.dice', 'w') as f:
            f.write(dice(g, observed, [s for _, s in asked]))
    else:
        stem = os.path.basename(a.out)
        with open(a.out + '.dicemap', 'w') as m:
            for j, (key, s) in enumerate(asked):
                with open(f'{a.out}__q{j}.dice', 'w') as f:
                    f.write(dice(g, observed, [s]))
                m.write(f'{stem}__q{j}.dice bool {key}\n')


if __name__ == '__main__':
    main()
