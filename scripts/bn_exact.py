#!/usr/bin/env python3
"""Exact marginals of a Bayesian-network program by clique-tree calibration,
an answer to check tddlog against where ProbLog does not finish.

Reads the ProbLog programs `gen_bn.py` writes, and nothing else: every clause
is one row of a table,

    p1::x(v1); ...; pk::x(vk) :- y1(u1), ..., ym(um).
    x(v) :- y1(u1), ..., ym(um).

with one atom per parent in the body, plus `evidence(x(v)).` and
`query(x(v)).` facts. Under ProbLog's semantics a variable none of whose rows
fires has no value, and a row whose probabilities sum to less than one leaves
the rest to no value. Such a variable gets a state `None`, in which every
child is `None` too, so the answers are ProbLog's even for programs whose
tables are not distributions.

The variables are eliminated in min-weight order after the evidence has cut
their domains; each elimination makes a cluster, whose message goes to the
cluster that eliminates its first remaining variable. One pass up and one down
gives every cluster its joint with the evidence, and each queried variable is
read from the cluster that eliminated it.

The products are each divided by their largest entry, and the logarithms
of what they divided out are added up the tree: the clusters without a
parent then give the probability of the evidence, which
`--evidence-probability` prints after the queries as tddlog does, `P(e):`,
the probability in scientific notation and `log10` of it.

Usage: bn_exact.py program.pl [--precision N] [--evidence-probability]
Prints `x(v):\tp` per query, in the order of the query facts, as tddlog does.
"""
import argparse
import math
import re
import sys

import numpy as np

ATOM = re.compile(r'([a-z][a-zA-Z0-9_]*)\(([a-zA-Z0-9_]+)\)')


def parse(text):
    rows = {}        # variable -> list of (parent assignment, {value: p})
    values = {}      # variable -> values in order of appearance
    evidence, queries = [], []
    # A comment runs to the end of its line: gen_bn.py's first line for a
    # Markov network ends in a number, not a clause's period.
    text = re.sub(r'%.*', '', text)
    for clause in re.split(r'\.\s*\n', text):
        clause = clause.strip().rstrip('.')
        if not clause:
            continue
        m = re.fullmatch(r'(evidence|query)\((.*)\)', clause)
        if m:
            a = ATOM.fullmatch(m.group(2).strip())
            if not a:
                sys.exit(f'bn_exact: not an atom: {clause}')
            (evidence if m.group(1) == 'evidence' else queries).append((a.group(1), a.group(2)))
            continue
        head, _, body = clause.partition(':-')
        heads = {}
        var = None
        for h in head.split(';'):
            h = h.strip()
            p, _, atom = h.rpartition('::')
            a = ATOM.fullmatch(atom.strip())
            if not a or (var is not None and a.group(1) != var):
                sys.exit(f'bn_exact: not a table row: {clause}')
            var = a.group(1)
            heads[a.group(2)] = float(p) if p else 1.0
        parents = tuple((a.group(1), a.group(2)) for a in ATOM.finditer(body))
        rows.setdefault(var, []).append((parents, heads))
        vs = values.setdefault(var, [])
        vs.extend(v for v in heads if v not in vs)
    return rows, values, evidence, queries


def factors(rows, values):
    """One table per variable over its parents and itself, the states of each
    variable its values and, where it can have none, `None` last."""
    parents = {}
    for x, rs in rows.items():
        names = [p for p, _ in rs[0][0]]
        for ps, _ in rs:
            if [p for p, _ in ps] != names:
                sys.exit(f'bn_exact: rows of {x} name different parents')
        parents[x] = names
    for x, names in parents.items():
        for p in names:
            if p not in rows:
                sys.exit(f'bn_exact: {x} reads {p}, which no row defines')
    order, done = [], set()

    def visit(x, path):
        if x in done:
            return
        if x in path:
            sys.exit(f'bn_exact: a cycle through {x}')
        for p in parents[x]:
            visit(p, path | {x})
        done.add(x)
        order.append(x)

    for x in rows:
        visit(x, frozenset())
    states, tables = {}, {}
    for x in order:
        index = {tuple(u for _, u in ps): heads for ps, heads in rows[x]}
        if len(index) != len(rows[x]):
            sys.exit(f'bn_exact: two rows of {x} for one parent assignment')
        shape = [len(states[p]) for p in parents[x]]
        vs = values[x]
        table = np.zeros(shape + [len(vs) + 1])
        for key in np.ndindex(*shape):
            u = tuple(states[p][i] for p, i in zip(parents[x], key))
            heads = index.get(u)   # `None` among u matches no row
            if heads is None:
                table[key + (len(vs),)] = 1.0
                continue
            for j, v in enumerate(vs):
                table[key + (j,)] = heads.get(v, 0.0)
            table[key + (len(vs),)] = max(0.0, 1.0 - sum(heads.values()))
        if not table[..., len(vs)].any():
            table = table[..., :len(vs)]
            states[x] = list(vs)
        else:
            states[x] = list(vs) + [None]
        tables[x] = (parents[x] + [x], table)
    return states, tables


# Factors one einsum multiplies: numpy before 2.0 takes at most 32 operands.
BATCH = 24


def absorb(fs):
    """The factors with each one whose variables another's include
    multiplied into that one, which adds no axis to it, and divided by its
    largest entry; and the base-10 logarithm of what the divisions took
    out."""
    out, shift = [], 0.0
    for scope, t in sorted(fs, key=lambda f: -len(f[0])):
        k = next((k for k, (big, _) in enumerate(out) if set(scope) <= set(big)), None)
        if k is None:
            out.append((list(scope), t))
            continue
        big, bt = out[k]
        axes = {v: i for i, v in enumerate(big)}
        aligned = np.einsum(t, [axes[v] for v in scope], sorted(axes[v] for v in scope))
        bt = bt * aligned.reshape([bt.shape[i] if v in scope else 1 for i, v in enumerate(big)])
        top = bt.max()
        if top > 0.0:
            bt, shift = bt / top, shift + math.log10(top)
        out[k] = (big, bt)
    return out, shift


def product(fs, keep, sizes):
    """The product of the factors `fs`, each (scope, table), summed onto the
    variables `keep`, constant along those of them no factor names, up to a
    positive constant: every product is divided by its largest entry, so a
    chain of them neither underflows nor overflows a double, and the
    marginals, normalized at the end, are the same."""
    return scaled_product(fs, keep, sizes)[0]


def scaled_product(fs, keep, sizes):
    """`product`, and the base-10 logarithm `s` of the constant it divided
    out: the product itself is the table times `10^s`. einsum takes labels
    below 52, so they are numbered per call, and at most BATCH factors at a
    time: past that, the factors whose variables another's include are
    multiplied into it first, and then batches of what is left are summed
    onto the variables the rest and `keep` name."""
    shift = 0.0
    if len(fs) > BATCH:
        fs, shift = absorb(fs)
    while len(fs) > BATCH:
        head, rest = fs[:BATCH], fs[BATCH:]
        later = set(keep).union(*(scope for scope, _ in rest))
        scope = list(dict.fromkeys(v for s, _ in head for v in s if v in later))
        t, s = scaled_product(head, scope, sizes)
        fs, shift = [(scope, t)] + rest, shift + s
    label = {}
    operands = []
    for scope, t in fs:
        operands += [t, [label.setdefault(v, len(label)) for v in scope]]
    if len(label) > 52:
        sys.exit(f'bn_exact: a cluster of {len(label)} variables')
    named = [v for v in keep if v in label]
    t = np.einsum(*operands, [label[v] for v in named], optimize=True) if fs else np.array(1.0)
    top = t.max()
    if top > 0.0:
        t, shift = t / top, shift + math.log10(top)
    if len(named) == len(keep):
        return t, shift
    t = t.reshape([sizes[v] if v in label else 1 for v in keep])
    return np.broadcast_to(t, [sizes[v] for v in keep]), shift


def observe(tables, states, observed):
    """The tables with each observed variable fixed at its value, its axis
    dropped."""
    out = []
    for scope, t in tables:
        index = tuple(states[v].index(observed[v]) if v in observed else slice(None) for v in scope)
        out.append(([v for v in scope if v not in observed], t[index]))
    return out


def marginals(states, tables, evidence):
    """P(x | e) for every unobserved variable x, the observations, and
    log10 P(e); or None when P(e) = 0."""
    observed = {}
    for x, v in evidence:
        if x not in states or v not in states[x] or observed.get(x, v) != v:
            return None
        observed[x] = v
    fs = observe([tables[x] for x in states], states, observed)
    kept = []
    # The tables the observations leave constant are factors of P(e).
    log10_pe = 0.0
    for scope, t in fs:
        if scope:
            kept.append((scope, t))
        elif float(t) == 0.0:
            return None
        else:
            log10_pe += math.log10(float(t))
    free = [x for x in states if x not in observed]
    sizes = {x: len(states[x]) for x in free}
    # Elimination order by least cluster size, on the interaction graph,
    # ties broken by the variable's position, so that the order does not
    # depend on the set order of names (which changes with the hash seed).
    rank = {x: i for i, x in enumerate(free)}
    neighbors = {x: set() for x in free}
    for scope, _ in kept:
        for v in scope:
            neighbors[v].update(w for w in scope if w != v)
    elim = []
    remaining = set(free)
    while remaining:
        def weight(x):
            w = len(states[x])
            for y in neighbors[x]:
                w *= len(states[y])
            return w
        x = min(remaining, key=lambda y: (weight(y), rank[y]))
        cluster = neighbors[x] | {x}
        elim.append((x, cluster))
        for y in neighbors[x]:
            neighbors[y] |= neighbors[x] - {y}
            neighbors[y].discard(x)
        remaining.discard(x)
        del neighbors[x]
    # Cluster i eliminates elim[i][0] over the scope elim[i][1]. It takes the
    # factors whose first eliminated variable is its own, and sends its
    # message to the cluster that eliminates the first of its separator.
    position = {x: i for i, (x, _) in enumerate(elim)}
    clusters = [{'scope': sorted(u, key=position.get), 'own': [], 'children': [], 'parent': None, 'sep': []}
                for _, u in elim]
    for scope, t in kept:
        clusters[min(position[v] for v in scope)]['own'].append((scope, t))
    for i, (x, u) in enumerate(elim):
        sep = u - {x}
        if sep:
            j = min(position[v] for v in sep)
            clusters[i]['parent'] = j
            clusters[i]['sep'] = sorted(sep, key=position.get)
            clusters[j]['children'].append(i)
    # Up: a child comes before its parent in the elimination order. A
    # message is its product divided by 10^lifted, which counts what its
    # children's messages were divided by as well.
    up, lifted = {}, {}
    for i, c in enumerate(clusters):
        if c['parent'] is not None:
            t, shift = scaled_product(c['own'] + [up[k] for k in c['children']], c['sep'], sizes)
            up[i] = (c['sep'], t)
            lifted[i] = shift + sum(lifted[k] for k in c['children'])
    # Down, and each variable's marginal from the cluster that eliminates it.
    down = {}
    out = {}
    for i in range(len(clusters) - 1, -1, -1):
        c = clusters[i]
        above = [down[i]] if c['parent'] is not None else []
        m, shift = scaled_product(c['own'] + [up[k] for k in c['children']] + above, [elim[i][0]], sizes)
        total = float(m.sum())
        if total == 0.0:
            return None
        if c['parent'] is None:
            # A root's product is its part's joint with the evidence.
            log10_pe += math.log10(total) + shift + sum(lifted[k] for k in c['children'])
        out[elim[i][0]] = m / total
        for k in c['children']:
            rest = c['own'] + [up[j] for j in c['children'] if j != k] + above
            down[k] = (clusters[k]['sep'], product(rest, clusters[k]['sep'], sizes))
    return out, observed, log10_pe


def evidence_line(log10_pe, digits):
    """tddlog's `--evidence-probability` line, `P(e):<tab>p<tab>log10 l`: the
    probability in scientific notation, its mantissa and exponent read off
    its base-10 logarithm, and the logarithm, each with `digits` after the
    point."""
    if log10_pe == -math.inf:
        return f'P(e):\t{0:.{digits}f}e0\tlog10 -inf'
    exponent = math.floor(log10_pe)
    mantissa = f'{10 ** (log10_pe - exponent):.{digits}f}'
    if mantissa.startswith('10'):
        mantissa, exponent = f'{1:.{digits}f}', exponent + 1
    return f'P(e):\t{mantissa}e{exponent}\tlog10 {log10_pe:.{digits}f}'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    ap.add_argument('--evidence-probability', action='store_true',
                    help="after the queries, P(e) as tddlog's --evidence-probability prints it")
    a = ap.parse_args()
    rows, values, evidence, queries = parse(open(a.program).read())
    states, tables = factors(rows, values)
    res = marginals(states, tables, evidence)
    if res is None:
        print('evidence is inconsistent')
        if a.evidence_probability:
            print(evidence_line(-math.inf, a.precision))
        return
    out, observed, log10_pe = res
    for x, v in queries:
        if x in observed:
            p = float(observed[x] == v)
        elif x not in states or v not in states[x]:
            p = 0.0
        else:
            p = float(out[x][states[x].index(v)])
        print(f'{x}({v}):\t{p:.{a.precision}f}')
    if a.evidence_probability:
        print(evidence_line(log10_pe, a.precision))


if __name__ == '__main__':
    main()
