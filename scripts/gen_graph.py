#!/usr/bin/env python3
"""Generate a probabilistic graph reachability program.

usage: gen_graph.py N M [--seed S] [--prob P] [--queries all|one|from1] [--undirected]
Nodes 1..N, M random edges (no self loops, no duplicates) each `P::edge(i,j).`,
`path/2` transitive closure, queries as selected.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('m', type=int)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--prob', type=float, default=None, help='fixed edge probability (default: random in [0.1,0.9])')
ap.add_argument('--queries', default='from1', choices=['all', 'one', 'from1'])
ap.add_argument('--undirected', action='store_true')
a = ap.parse_args()
rng = random.Random(a.seed)
edges = set()
while len(edges) < a.m:
    i, j = rng.randrange(1, a.n + 1), rng.randrange(1, a.n + 1)
    if i == j:
        continue
    if a.undirected and (j, i) in edges:
        continue
    edges.add((i, j))
print(f'% random graph: {a.n} nodes, {a.m} edges, seed {a.seed}')
for (i, j) in sorted(edges):
    p = a.prob if a.prob is not None else round(rng.uniform(0.1, 0.9), 2)
    print(f'{p}::edge({i},{j}).')
if a.undirected:
    print('conn(X,Y) :- edge(X,Y).')
    print('conn(X,Y) :- edge(Y,X).')
    print('path(X,Y) :- conn(X,Y).')
    print('path(X,Y) :- conn(X,Z), path(Z,Y).')
else:
    print('path(X,Y) :- edge(X,Y).')
    print('path(X,Y) :- edge(X,Z), path(Z,Y).')
if a.queries == 'all':
    print('query(path(X,Y)).')
elif a.queries == 'one':
    print(f'query(path(1,{a.n})).')
else:
    for j in range(2, a.n + 1):
        print(f'query(path(1,{j})).')
