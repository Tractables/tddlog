#!/usr/bin/env python3
"""Generate a smokers social-network program (Fierens et al. 2015 style).

usage: gen_smokers.py N [--friends-per F] [--seed S] [--queries smokes|asthma|all] [--symmetric]
N people, each with F random friends (directed), probabilistic stress and
influence, asthma as a probabilistic consequence of smoking. --symmetric
makes friendship mutual: each random pair is written in both directions,
friend(X,Y) and friend(Y,X), once each.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--friends-per', type=int, default=2)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--queries', default='smokes', choices=['smokes', 'asthma', 'all'])
ap.add_argument('--symmetric', action='store_true')
a = ap.parse_args()
rng = random.Random(a.seed)
print(f'% smokers: {a.n} people, {a.friends_per} friends each, seed {a.seed}'
      + (', symmetric' if a.symmetric else ''))
print('0.3::stress(X) :- person(X).')
print('0.2::influences(X,Y) :- friend(X,Y).')
print('smokes(X) :- stress(X).')
print('smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).')
print('0.4::asthma(X) :- smokes(X).')
for i in range(1, a.n + 1):
    print(f'person(p{i}).')
written = set()
for i in range(1, a.n + 1):
    # the others are 1..n without i; sampling their indices draws what sampling
    # their list would, without building it (linear in n, not quadratic)
    for k in rng.sample(range(a.n - 1), min(a.friends_per, a.n - 1)):
        j = k + 1 if k + 1 < i else k + 2
        for x, y in ((i, j), (j, i)) if a.symmetric else ((i, j),):
            if (x, y) not in written:
                written.add((x, y))
                print(f'friend(p{x},p{y}).')
if a.queries in ('smokes', 'all'):
    print('query(smokes(X)) :- person(X).')
if a.queries in ('asthma', 'all'):
    print('query(asthma(X)) :- person(X).')
