#!/usr/bin/env python3
"""Generate an N x N grid with probabilistic edges (right and down, plus
optional left/up) and a corner-to-corner path query.

usage: gen_grid.py N [--seed S] [--prob P] [--bidirectional]
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--prob', type=float, default=None)
ap.add_argument('--bidirectional', action='store_true')
ap.add_argument('--evidence', action='store_true', help='observe a path from the origin to the grid centre')
a = ap.parse_args()
rng = random.Random(a.seed)
n = a.n


def node(i, j):
    return f'n{i}_{j}'


def p():
    return a.prob if a.prob is not None else round(rng.uniform(0.1, 0.9), 2)


print(f'% grid {n}x{n}, seed {a.seed}')
for i in range(n):
    for j in range(n):
        if j + 1 < n:
            print(f'{p()}::edge({node(i, j)},{node(i, j + 1)}).')
            if a.bidirectional:
                print(f'{p()}::edge({node(i, j + 1)},{node(i, j)}).')
        if i + 1 < n:
            print(f'{p()}::edge({node(i, j)},{node(i + 1, j)}).')
            if a.bidirectional:
                print(f'{p()}::edge({node(i + 1, j)},{node(i, j)}).')
print('path(X,Y) :- edge(X,Y).')
print('path(X,Y) :- edge(X,Z), path(Z,Y).')
print(f'query(path({node(0, 0)},{node(n - 1, n - 1)})).')
if a.evidence:
    if n < 3:
        ap.error('--evidence needs a grid of side at least 3')
    print(f'evidence(path({node(0, 0)},{node(n // 2, n // 2)}),true).')
