#!/usr/bin/env python3
"""ProbLog2's probabilistic grid (Fierens et al., TPLP 15(3), 2015, Section
9.1 p. 31 and 9.2.1 p. 32): nodes n_X_Y of an N x N grid, directed edges of
probability 0.5 from each node to its right, lower and lower-right
neighbours, and the probability of a path from n_I_I to n_N_N, I = N - D.
The paper's grid is 16 x 16 with D from 1 to 15 (its Fig. 5: ProbLog2 at
about 110 s for D = 10, beyond its limit after).

usage: gen_pgrid.py N --template
       gen_pgrid.py N D --arguments

--template prints the program of the N x N grid, whose start node is its
place $1 (tddlog reads it with the node as its argument: `tddlog
pgrid16.pl n_6_6`); --arguments prints the argument of distance D, the
node n_I_I.
"""
import argparse


def template(n):
    lines = [f'% probabilistic grid {n}x{n} (Fierens et al., TPLP 2015), written by gen_pgrid.py {n} --template:',
             f'% path($1,n_{n}_{n}), from the node $1, the program\'s argument.']
    for x in range(1, n):
        for y in range(1, n + 1):
            lines.append(f'0.5::edge(n_{x}_{y},n_{x + 1}_{y}).')
    for x in range(1, n + 1):
        for y in range(1, n):
            lines.append(f'0.5::edge(n_{x}_{y},n_{x}_{y + 1}).')
    for x in range(1, n):
        for y in range(1, n):
            lines.append(f'0.5::edge(n_{x}_{y},n_{x + 1}_{y + 1}).')
    lines += ['path(X,Y) :- edge(X,Y).', 'path(X,Y) :- edge(X,Z), path(Z,Y).', f'query(path($1,n_{n}_{n})).']
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    ap.add_argument('d', type=int, nargs='?')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help='the program of the grid, its start node the place $1')
    mode.add_argument('--arguments', action='store_true', help="distance D's argument")
    a = ap.parse_args()
    if a.n < 2:
        ap.error('N is the side of the grid: 2 or more')
    if a.template:
        if a.d is not None:
            ap.error('--template prints the program of every distance: D gives its argument')
        print(template(a.n), end='')
        return
    if a.d is None or not 1 <= a.n - a.d < a.n:
        ap.error(f'--arguments needs a distance D from 1 to {a.n - 1}')
    i = a.n - a.d
    print(f'n_{i}_{i}')


if __name__ == '__main__':
    main()
