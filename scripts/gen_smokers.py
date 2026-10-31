#!/usr/bin/env python3
"""Generate a smokers social-network program (Fierens et al. 2015 style).

usage: gen_smokers.py N [--friends-per F] [--seed S] [--queries smokes|asthma|all] [--symmetric [--nested]]
N people, each with F random friends (directed), probabilistic stress and
influence, asthma as a probabilistic consequence of smoking. --symmetric
makes friendship mutual: each random pair is written in both directions,
friend(X,Y) and friend(Y,X), once each.

--nested (with --symmetric) makes the sizes of one seed nested: person i
draws min(F, i - 1) friends among persons 1 to i - 1, from a random stream
of its own (seeded by the seed and i), so that the program of N + 1 people
is the program of N people plus person N + 1 and that person's
friendships. The graph has 3N - 6 edges for F = 3 and N >= 3, about the
average degree of the plain symmetric graph of the same size, whose sizes
are unrelated graphs.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--friends-per', type=int, default=2)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--queries', default='smokes', choices=['smokes', 'asthma', 'all'])
ap.add_argument('--symmetric', action='store_true')
ap.add_argument('--nested', action='store_true')
a = ap.parse_args()
if a.nested and not a.symmetric:
    ap.error('--nested needs --symmetric')
rng = random.Random(a.seed)
print(f'% smokers: {a.n} people, {a.friends_per} friends each, seed {a.seed}'
      + (', symmetric' if a.symmetric else '') + (', nested' if a.nested else ''))
print('0.3::stress(X) :- person(X).')
print('0.2::influences(X,Y) :- friend(X,Y).')
print('smokes(X) :- stress(X).')
print('smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).')
print('0.4::asthma(X) :- smokes(X).')
for i in range(1, a.n + 1):
    print(f'person(p{i}).')
written = set()
for i in range(1, a.n + 1):
    if a.nested:
        # the people before i, each pair new: written once each way, in i's order
        for k in random.Random(f'{a.seed}/{i}').sample(range(i - 1), min(a.friends_per, i - 1)):
            print(f'friend(p{i},p{k + 1}).')
            print(f'friend(p{k + 1},p{i}).')
        continue
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
