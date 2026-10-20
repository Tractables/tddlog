#!/usr/bin/env python3
"""Generate a smokers social-network program (Fierens et al. 2015 style).

usage: gen_smokers.py N [--friends-per F] [--seed S] [--queries smokes|asthma|all]
N people, each with F random friends (directed), probabilistic stress and
influence, asthma as a probabilistic consequence of smoking.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--friends-per', type=int, default=2)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--queries', default='smokes', choices=['smokes', 'asthma', 'all'])
a = ap.parse_args()
rng = random.Random(a.seed)
print(f'% smokers: {a.n} people, {a.friends_per} friends each, seed {a.seed}')
print('0.3::stress(X) :- person(X).')
print('0.2::influences(X,Y) :- friend(X,Y).')
print('smokes(X) :- stress(X).')
print('smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).')
print('0.4::asthma(X) :- smokes(X).')
for i in range(1, a.n + 1):
    print(f'person(p{i}).')
for i in range(1, a.n + 1):
    others = [j for j in range(1, a.n + 1) if j != i]
    for j in rng.sample(others, min(a.friends_per, len(others))):
        print(f'friend(p{i},p{j}).')
if a.queries in ('smokes', 'all'):
    print('query(smokes(X)) :- person(X).')
if a.queries in ('asthma', 'all'):
    print('query(asthma(X)) :- person(X).')
