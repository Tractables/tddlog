#!/usr/bin/env python3
"""Exact answers for the TPC-H programs of gen_tpch.py, the reference to check
tddlog against where ProbLog does not finish.

usage: tpch_exact.py program.pl [--precision N]

Reads the tuples of the program's probabilistic facts and evaluates each
query directly, as the SQL of its TPC-H query reads (joins and selections
over the tuples, written here query by query rather than read from the
program's rules), into its lineage: a monotone DNF over the tuples, one
clause per way the query is satisfied. The probability of the lineage is
computed by the decomposition of Olteanu, Huang and Koch's d-trees (ICDE
2010): variable-disjoint parts are independent (P = 1 - prod(1 - P_i)), and
a tuple in every clause factors out (P = p * P(rest)). A part where
neither applies is a core, answered by variable elimination on the factor
graph whose factors say that no clause holds (lit_exact.partition, as for
the h0 queries of gen_pdb.py), after the tuples of one clause alone are
folded into that clause's weight. On a hierarchical query's lineage
(read-once on TPC-H data) the first two steps suffice and the computation
is linear; on q2, q7, q20 and q21 the elimination is exponential in the
width of the bipartite core between suppliers and parts or orders.

Prints `q:\tp` per query, in the order of the query facts, as tddlog does.
"""
import argparse
import math
import os
import re
import sys
import threading

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lit_exact  # noqa: E402

FACT = re.compile(r'^([0-9.eE+-]+)::(\w+)\((.*)\)\.\s*$')


def parse(text):
    """The probabilistic facts by relation, each tuple (id, fields), and the
    probability of each id; fields that are integers are ints."""
    rels, probs = {}, []
    for line in text.splitlines():
        m = FACT.match(line)
        if not m:
            continue
        fields = tuple(int(x) if re.fullmatch(r'-?\d+', x) else x for x in m.group(3).split(','))
        rels.setdefault(m.group(2), []).append((len(probs), fields))
        probs.append(float(m.group(1)))
    return rels, probs


def queries(text):
    return [q for q in re.findall(r'^query\((\w+)\)\.\s*$', text, re.M)]


def index(rel, key):
    """The tuples of a relation by the field at position `key`."""
    out = {}
    for t, f in rel:
        out.setdefault(f[key], []).append((t, f))
    return out


# Field positions, as gen_tpch.py writes the relations.
L_ORDER, L_PART, L_SUPP, L_QTY, L_DISC, L_FLAG, L_SHIP, L_COMMIT, L_RECEIPT, L_INSTR, L_MODE = \
    0, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12
P_NAME, P_BRAND, P_TYPE1, P_TYPE2, P_TYPE3, P_SIZE, P_CONT1, P_CONT2 = 1, 3, 4, 5, 6, 7, 8, 9


def lineage(name, db):
    """The clauses of query `name` over the database `db`."""
    rel = lambda r: db.get(r, [])  # noqa: E731
    nations = {f[0]: (t, f[1], f[2]) for t, f in rel('nation')}
    regions = {f[0]: (t, f[1]) for t, f in rel('region')}
    out = []
    if name == 'q1':
        out = [(t,) for t, f in rel('lineitem') if f[L_SHIP] <= 19980902]
    elif name == 'q2':
        supp = {f[0]: (t, f[1]) for t, f in rel('supplier')}
        ps = index(rel('partsupp'), 0)
        for tp, p in rel('part'):
            if p[P_SIZE] != 15 or p[P_TYPE3] != 'brass':
                continue
            for tps, (_, s) in ps.get(p[0], []):
                ts, n = supp[s]
                tn, _, r = nations[n]
                tr, rname = regions[r]
                if rname == 'europe':
                    out.append((tp, tps, ts, tn, tr))
    elif name in ('q3', 'q10', 'q18'):
        cust = {f[0]: (t, f) for t, f in rel('customer')}
        items = index(rel('lineitem'), L_ORDER)
        for to, o in rel('orders'):
            tc, c = cust[o[1]]
            if name == 'q3' and not (c[2] == 'building' and o[3] < 19950315):
                continue
            if name == 'q10' and not 19931001 <= o[3] < 19940101:
                continue
            for tl, l in items.get(o[0], []):
                if name == 'q3' and not l[L_SHIP] > 19950315:
                    continue
                if name == 'q10' and l[L_FLAG] != 'r':
                    continue
                out.append((tc, to, tl) + ((nations[c[1]][0],) if name == 'q10' else ()))
    elif name in ('q4', 'q12'):
        items = index(rel('lineitem'), L_ORDER)
        for to, o in rel('orders'):
            if name == 'q4' and not 19930701 <= o[3] < 19931001:
                continue
            for tl, l in items.get(o[0], []):
                if name == 'q4' and l[L_COMMIT] < l[L_RECEIPT]:
                    out.append((to, tl))
                if name == 'q12' and l[L_MODE] in ('mail', 'ship') and l[L_COMMIT] < l[L_RECEIPT] \
                        and l[L_SHIP] < l[L_COMMIT] and 19940101 <= l[L_RECEIPT] < 19950101:
                    out.append((to, tl))
    elif name == 'q6':
        out = [(t,) for t, f in rel('lineitem')
               if 19940101 <= f[L_SHIP] < 19950101 and 5 <= f[L_DISC] <= 7 and f[L_QTY] < 24]
    elif name in ('q7', 'q21'):
        supp = {f[0]: (t, f[1]) for t, f in rel('supplier')}
        orders = {f[0]: (t, f) for t, f in rel('orders')}
        cust = {f[0]: (t, f) for t, f in rel('customer')}
        for tl, l in rel('lineitem'):
            ts, n1 = supp[l[L_SUPP]]
            tn1, name1, _ = nations[n1]
            to, o = orders[l[L_ORDER]]
            if name == 'q21':
                if name1 == 'saudi_arabia' and l[L_RECEIPT] > l[L_COMMIT] and o[2] == 'f':
                    out.append((ts, tn1, tl, to))
                continue
            if not 19950101 <= l[L_SHIP] <= 19961231:
                continue
            tc, c = cust[o[1]]
            tn2, name2, _ = nations[c[1]]
            if (name1, name2) in (('france', 'germany'), ('germany', 'france')):
                out.append((ts, tn1, tl, to, tc, tn2))
    elif name in ('q11', 'q20'):
        supp = {f[0]: (t, f[1]) for t, f in rel('supplier')}
        part = {f[0]: (t, f) for t, f in rel('part')}
        for tps, (p, s) in rel('partsupp'):
            ts, n = supp[s]
            tn, nname, _ = nations[n]
            if name == 'q11' and nname == 'germany':
                out.append((tps, ts, tn))
            if name == 'q20' and nname == 'canada':
                tp, pf = part[p]
                if pf[P_NAME] == 'forest':
                    out.append((ts, tn, tps, tp))
    elif name in ('q14', 'q17', 'q19'):
        part = {f[0]: (t, f) for t, f in rel('part')}
        q19 = {('brand12', 'sm'): ({'case', 'box', 'pack', 'pkg'}, 1, 11, 5),
               ('brand23', 'med'): ({'bag', 'box', 'pkg', 'pack'}, 10, 20, 10),
               ('brand34', 'lg'): ({'case', 'box', 'pack', 'pkg'}, 20, 30, 15)}
        for tl, l in rel('lineitem'):
            tp, p = part[l[L_PART]]
            if name == 'q14' and 19950901 <= l[L_SHIP] < 19951001:
                out.append((tl, tp))
            if name == 'q17' and p[P_BRAND] == 'brand23' and (p[P_CONT1], p[P_CONT2]) == ('med', 'box'):
                out.append((tl, tp))
            if name == 'q19' and l[L_INSTR] == 'deliver_in_person' and l[L_MODE] in ('air', 'air reg'):
                case = q19.get((p[P_BRAND], p[P_CONT1]))
                if case and p[P_CONT2] in case[0] and case[1] <= l[L_QTY] <= case[2] and 1 <= p[P_SIZE] <= case[3]:
                    out.append((tl, tp))
    elif name == 'q15':
        supp = {f[0]: t for t, f in rel('supplier')}
        out = [(supp[f[L_SUPP]], t) for t, f in rel('lineitem') if 19960101 <= f[L_SHIP] < 19960401]
    elif name == 'q16':
        part = {f[0]: (t, f) for t, f in rel('part')}
        sizes = {49, 14, 23, 45, 19, 3, 36, 9}
        for tps, (p, _) in rel('partsupp'):
            tp, pf = part[p]
            if pf[P_BRAND] != 'brand45' and (pf[P_TYPE1], pf[P_TYPE2]) == ('medium', 'polished') \
                    and pf[P_SIZE] in sizes:
                out.append((tps, tp))
    else:
        raise SystemExit(f'tpch_exact: no query {name}')
    return [frozenset(c) for c in out]


def components(clauses):
    """The clauses grouped by the connected components of their variables."""
    parent = {}

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    for c in clauses:
        it = iter(c)
        first = next(it)
        r = find(parent.setdefault(first, first))
        for v in it:
            s = find(parent.setdefault(v, v))
            if s != r:
                parent[s] = r
    groups = {}
    for c in clauses:
        groups.setdefault(find(next(iter(c))), []).append(c)
    return list(groups.values())


def probability(clauses, p):
    """P of the monotone DNF `clauses` (frozensets of tuple ids) where tuple t
    holds with probability p[t], independently."""

    def solve(cs):
        if not cs:
            return 0.0
        if any(not c for c in cs):
            return 1.0
        fail = 1.0
        for part in components(cs):
            fail *= 1.0 - connected(part)
        return 1.0 - fail

    def connected(cs):
        if len(cs) == 1:
            return math.prod(p[v] for v in cs[0])
        common = frozenset.intersection(*cs)
        if common:
            return math.prod(p[v] for v in common) * solve(list({c - common for c in cs}))
        return core(cs)

    def core(cs):
        """1 - Z, Z the partition function of the factor graph whose factors
        say that no clause holds: P(no clause) is the sum over the tuples
        of the product of their weights and of 1 - [the clause holds]. A
        tuple of one clause alone is summed out first, which leaves the
        clause holding with the product of those tuples' probabilities, q,
        on the rest of its tuples; clauses left over the same tuples are
        one, with 1 - (1 - q1)(1 - q2). The rest is variable elimination
        (lit_exact.partition)."""
        weighted = {c: 1.0 for c in cs}
        while True:
            count = {}
            for c in weighted:
                for v in c:
                    count[v] = count.get(v, 0) + 1
            folded = {}
            for c, q in weighted.items():
                own = frozenset(v for v in c if count[v] == 1)
                rest = c - own
                q *= math.prod(p[v] for v in own)
                folded[rest] = 1.0 - (1.0 - folded.get(rest, 0.0)) * (1.0 - q)
            if len(folded) == len(weighted) and all(c in weighted for c in folded):
                break
            weighted = folded
        constant = 1.0 - weighted.pop(frozenset(), 0.0)
        tuples = sorted({v for c in weighted for v in c})
        factors = [([v], np.array([1.0 - p[v], p[v]])) for v in tuples]
        for c, q in weighted.items():
            scope = sorted(c)
            t = np.ones([2] * len(scope))
            t[(1,) * len(scope)] = 1.0 - q
            factors.append((scope, t))
        return 1.0 - constant * lit_exact.partition(factors, {v: 2 for v in tuples})

    return solve(list(set(clauses)))


def answers(text):
    rels, p = parse(text)
    return {q: probability(lineage(q, rels), p) for q in queries(text)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    a = ap.parse_args()
    text = open(a.program).read()
    result = {}
    # The decomposition recurses once per nesting of a lineage; a deep stack.
    sys.setrecursionlimit(1_000_000)
    threading.stack_size(1 << 29)
    t = threading.Thread(target=lambda: result.update(answers(text)))
    t.start()
    t.join()
    for q in queries(text):
        if q not in result:
            sys.exit(f'tpch_exact: no answer for {q}')
        print(f'{q}:\t{result[q]:.{a.precision}f}')


if __name__ == '__main__':
    main()
