#!/usr/bin/env python3
"""Generate an intractable query with a self-join on a random graph.

usage: gen_rgraph.py N [--seed S]

A graph query of Fink, Huang, Olteanu, "Anytime approximation in
probabilistic databases" (VLDB J. 22(6), 2013, Section 5, "Experiments with
intractable positive graph queries"), after Olteanu, Huang, Koch (ICDE
2010): an undirected random graph on N nodes is a tuple-independent
probabilistic database whose possible worlds are the subgraphs of the
N-clique, one tuple e(x,y), x < y, per edge. The Boolean query is the
paper's G_P2, whether the graph has a path of two edges, a self-join of
e. Its complement is that the edges present form a matching, and counting
the matchings of a graph is #P-complete (Valiant 1979), so the query is
#P-hard; lit_exact.py answers it by dynamic programming over the subsets
of nodes, exponential in N.

The paper gives every edge one probability P_e (0.7 down to 0.01). Here
each edge's probability is drawn from [0.05, 0.95] and scaled by
2.5/N^1.5, so that about one path is expected whatever N and the answer
stays away from 0 and 1; the probabilities do not change the lineage.
The paper's other queries are left out: the triangle has no exact method
past 8 nodes but enumeration, where its lineage is small, and the path of
three edges and the two degrees of separation are variants of this query
and of the standard suite's graph family.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if a.n < 3:
    ap.error('need N >= 3')
rng = random.Random(a.seed)
N = a.n
scale = 2.5 / N ** 1.5

print(f'% a path of two edges in a random graph: the {N}-clique, every edge a probabilistic tuple, seed {a.seed}')
for x in range(N):
    for y in range(x + 1, N):
        print(f'{round(rng.uniform(0.05, 0.95) * scale, 4)}::e({x},{y}).')
print('adj(X,Y) :- e(X,Y).')
print('adj(X,Y) :- e(Y,X).')
print('path2 :- adj(Y,X), adj(Y,Z), X < Z.')
print('query(path2).')
