#!/usr/bin/env python3
"""Generate Boolean queries over a random tuple-independent database.

usage: gen_pdb.py N [--degree K] [--seed S]

The probabilistic-database benchmark of Dalvi and Suciu (VLDB J. 2007,
JACM 2012): unary relations R and T over a domain of N constants and a
binary relation S in which each constant is related to K others drawn at
random, every tuple an independent probabilistic fact. R and T tuples have
probability in [0.05, 0.95]; S tuples have that times min(1, 4/(KN)), so
that two S tuples are expected whatever the domain and the answers stay
away from 1 as it grows (the probabilities do not change the lineage, whose
structure is what makes h0 and h1 hard). Three queries:

    h0   :- R(x), S(x,y), T(y)               #P-hard, the canonical one
    h1   :- R(x), S(x,y) ; S(x,y), T(y)      #P-hard, H1 of the dichotomy
    safe :- R(x), S(x,y)                     hierarchical, polynomial

The lineage of h0 and h1 is a DNF over the bipartite graph of S that has no
polynomial-size decision-DNNF on dense graphs (Beame, Li, Roy, Suciu, TODS
2017); K keeps the graph sparse so the family grows with N.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--degree', type=int, default=2)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if not 1 <= a.degree <= a.n:
    ap.error('need 1 <= K <= N')
rng = random.Random(a.seed)


def prob(scale=1.0):
    return round(rng.uniform(0.05, 0.95) * scale, 2 if scale == 1.0 else 4)


print(f'% tuple-independent database: domain {a.n}, {a.degree} S-tuples per constant, seed {a.seed}')
for i in range(a.n):
    print(f'{prob()}::r(c{i}).')
    print(f'{prob()}::t(c{i}).')
scale = min(1.0, 4 / (a.degree * a.n))
for i in range(a.n):
    for j in sorted(rng.sample(range(a.n), a.degree)):
        print(f'{prob(scale)}::s(c{i},c{j}).')
print('h0 :- r(X), s(X,Y), t(Y).')
print('h1 :- r(X), s(X,Y).')
print('h1 :- s(X,Y), t(Y).')
print('safe :- r(X), s(X,Y).')
for q in ('h0', 'h1', 'safe'):
    print(f'query({q}).')
