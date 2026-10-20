#!/usr/bin/env python3
"""Estimate a reachability program's answers from sampled graphs.

usage: sample_paths.py PROGRAM [--samples N] [--seed S] [--against ANSWERS]

PROGRAM holds probabilistic edge facts `p::edge(u,v).`, the two rules of
`path/2` as the transitive closure of `edge/2`, and queries
`query(path(s,t)).`, as the graph and grid families of the benchmarks
(gen_graph.py, gen_grid.py) do; anything else is refused. Prints each
query's estimate and its standard error. With --against, reads printed
answers (`atom: p` lines, as tddlog and ProbLog print them, or a
results file of the baselines branch's bench.py, whose tddlog record of
the program's instance is read) and prints how many standard errors the
farthest answer lies from its estimate.

This checks the answers of instances no exact system finishes. It is not
exact: an answer within a few standard errors of its estimate is
consistent with it, nothing more.
"""
import argparse
import json
import re
import sys
from pathlib import Path

import numpy as np

EDGE = re.compile(r'([0-9.eE+-]+)::edge\((\w+),(\w+)\)\.$')
QUERY = re.compile(r'query\(path\((\w+),(\w+)\)\)\.$')
RULES = {'path(X,Y) :- edge(X,Y).', 'path(X,Y) :- edge(X,Z), path(Z,Y).'}


def read(path):
    """The edges (p, u, v) and the queries (s, t) of a reachability program."""
    edges, queries, rules = [], [], set()
    for n, raw in enumerate(Path(path).read_text().splitlines(), 1):
        line = raw.split('%', 1)[0].strip()
        if not line:
            continue
        if m := EDGE.match(line):
            edges.append((float(m.group(1)), m.group(2), m.group(3)))
        elif m := QUERY.match(line):
            queries.append((m.group(1), m.group(2)))
        elif line in RULES:
            rules.add(line)
        else:
            sys.exit(f'{path}:{n}: not a reachability program: {raw}')
    if rules != RULES:
        sys.exit(f'{path}: path/2 is not the transitive closure of edge/2')
    return edges, queries


def estimate(edges, queries, samples, seed):
    """Per query, the fraction of sampled graphs with a path, and its
    standard error. Each node's reachability from a source is a vector
    over the samples, widened edge by edge until nothing changes."""
    rng = np.random.default_rng(seed)
    present = [(u, v, rng.random(samples) < p) for p, u, v in edges]
    nodes = {u for _, u, _ in edges} | {v for _, _, v in edges}
    out = {}
    for s in sorted({s for s, _ in queries}):
        reach = {n: np.zeros(samples, dtype=bool) for n in nodes}
        changed = True
        while changed:
            changed = False
            for u, v, e in present:
                wider = reach[v] | (e if u == s else reach[u] & e)
                if not np.array_equal(wider, reach[v]):
                    reach[v] = wider
                    changed = True
        for q, t in queries:
            if q == s:
                p = float(reach[t].mean()) if t in reach else 0.0
                out[f'path({s},{t})'] = (p, (p * (1 - p) / samples) ** 0.5)
    return out


def answers(path, instance):
    if path.endswith('.jsonl'):
        for line in Path(path).read_text().splitlines():
            r = json.loads(line)
            if r['instance'] == instance and r['system'] == 'tddlog':
                return r['answers']
        sys.exit(f'{path}: no tddlog record of {instance}')
    return {a.strip(): float(p) for a, p in (line.rsplit(':', 1) for line in Path(path).read_text().splitlines() if ':' in line)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--samples', type=int, default=1_000_000)
    ap.add_argument('--seed', type=int, default=7)
    ap.add_argument('--against')
    a = ap.parse_args()
    edges, queries = read(a.program)
    est = estimate(edges, queries, a.samples, a.seed)
    if a.against is None:
        for q, (p, se) in est.items():
            print(f'{q}:\t{p:.6f}\t{se:.6f}')
        return
    given = answers(a.against, Path(a.program).stem)
    missing = sorted(set(est) - set(given))
    if missing:
        sys.exit(f'{a.against}: no answer for {missing}')
    z = max(abs(given[q] - p) / max(se, 1e-12) for q, (p, se) in est.items())
    print(f'{Path(a.program).stem}: {len(est)} answers, {a.samples} samples, the farthest {z:.2f} standard errors from its estimate')


if __name__ == '__main__':
    main()
