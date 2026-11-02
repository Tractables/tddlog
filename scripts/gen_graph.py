#!/usr/bin/env python3
"""A random probabilistic graph and the reachability of its nodes from node 1.

usage: gen_graph.py N M [--seed S] [--prob P] --template
       gen_graph.py N M [--seed S] [--prob P] --arguments
       gen_graph.py N M [--seed S] [--prob P] --data KEY

The family is one program and its data (bench.py's FOLD): --template prints
the program, which consults the graph (its place $1) and asks for the
probability of a path from node 1 to each node up to $2; --arguments prints
the argument $2, N; --data prints the graph, KEY (the name bench.py gives
the file) being only its name: N nodes 1..N, M random edges (no self loops,
no duplicates), each `P::edge(i,j).` with P fixed or random in [0.1, 0.9].
"""
import argparse
import random

TEMPLATE = """% reachability from node 1 in a probabilistic graph: path/2 is the transitive closure of
% edge/2, and each node from 2 to $2 is queried. $1 is the graph's edges, a file of facts
% `P::edge(I,J).`.
:- consult($1).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
query(path(1,Y)) :- between(2,$2,Y).
"""


def graph(n, m, seed, prob):
    rng = random.Random(seed)
    edges = set()
    while len(edges) < m:
        i, j = rng.randrange(1, n + 1), rng.randrange(1, n + 1)
        if i != j:
            edges.add((i, j))
    lines = [f'% random graph: {n} nodes, {m} edges, seed {seed}']
    for (i, j) in sorted(edges):
        p = prob if prob is not None else round(rng.uniform(0.1, 0.9), 2)
        lines.append(f'{p}::edge({i},{j}).')
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    ap.add_argument('m', type=int)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--prob', type=float, default=None, help='fixed edge probability (default: random in [0.1,0.9])')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true')
    mode.add_argument('--arguments', action='store_true')
    mode.add_argument('--data', metavar='KEY')
    a = ap.parse_args()
    if a.template:
        print(TEMPLATE, end='')
    elif a.arguments:
        print(a.n)
    else:
        print(graph(a.n, a.m, a.seed, a.prob), end='')


if __name__ == '__main__':
    main()
