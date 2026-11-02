#!/usr/bin/env python3
"""An N x N grid with probabilistic edges (right and down) and the reachability
of its far corner from its near one.

usage: gen_grid.py N [--seed S] [--prob P] [--evidence] --template
       gen_grid.py N [--seed S] [--prob P] [--evidence] --arguments
       gen_grid.py N [--seed S] [--prob P] --data KEY

The family is one program and its data (bench.py's FOLD): --template prints
the program, which consults the grid (its place $1) and asks for the
probability of a path from n0_0 to the node $2, the far corner; with
--evidence it observes a path from n0_0 to the node $3, the grid's centre
(a program of its own); --arguments prints $2 (and $3); --data prints the
grid (the same with or without --evidence), KEY being only its name: each edge `P::edge(a,b).` with P fixed or
random in [0.1, 0.9].
"""
import argparse
import random

TEMPLATE = """% reachability in a probabilistic grid: path/2 is the transitive closure of edge/2, and the
% corner $2 is queried from n0_0. $1 is the grid's edges, a file of facts `P::edge(A,B).`.
:- consult($1).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
query(path(n0_0,$2)).
"""
EVIDENCE = """evidence(path(n0_0,$3),true).
"""


def node(i, j):
    return f'n{i}_{j}'


def grid(n, seed, prob):
    rng = random.Random(seed)

    def p():
        return prob if prob is not None else round(rng.uniform(0.1, 0.9), 2)

    lines = [f'% grid {n}x{n}, seed {seed}']
    for i in range(n):
        for j in range(n):
            if j + 1 < n:
                lines.append(f'{p()}::edge({node(i, j)},{node(i, j + 1)}).')
            if i + 1 < n:
                lines.append(f'{p()}::edge({node(i, j)},{node(i + 1, j)}).')
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--prob', type=float, default=None)
    ap.add_argument('--evidence', action='store_true', help='observe a path from the origin to the grid centre')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true')
    mode.add_argument('--arguments', action='store_true')
    mode.add_argument('--data', metavar='KEY')
    a = ap.parse_args()
    n = a.n
    if a.evidence and n < 3:
        ap.error('--evidence needs a grid of side at least 3')
    if a.template:
        print(TEMPLATE + (EVIDENCE if a.evidence else ''), end='')
    elif a.arguments:
        print(node(n - 1, n - 1))
        if a.evidence:
            print(node(n // 2, n // 2))
    else:
        print(grid(n, a.seed, a.prob), end='')


if __name__ == '__main__':
    main()
