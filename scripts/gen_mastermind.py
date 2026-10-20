#!/usr/bin/env python3
"""Generate a Mastermind belief-state program.

usage: gen_mastermind.py C P R [--seed S]

The game of Chavira, Darwiche and Jaeger's relational Bayesian networks
(IJAR 2006), as the probabilistic logic program of Vlasselaer et al.
(TP-compilation, IJAR 2016, Table 3, there unrolled over 8 rounds): a hidden
code of P pegs, each of C colours uniformly; R random guesses, each answered
with the number of black pegs (right colour in the right place) and white
pegs (right colour in the wrong place) for a code drawn from the seed. The
feedback is evidence, and the colour of every peg is queried. Black pegs and
the count of each colour in the code are partial sums over the pegs; white
pegs are the sum over colours of the smaller of the two counts, less the
black ones.
"""
import argparse
import math
import random

ap = argparse.ArgumentParser()
ap.add_argument('c', type=int)
ap.add_argument('p', type=int)
ap.add_argument('r', type=int)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if a.c < 2 or a.p < 1 or a.r < 1:
    ap.error('need at least two colours, a peg and a guess')
rng = random.Random(a.seed)
C, P, R = a.c, a.p, a.r
code = [rng.randrange(C) for _ in range(P)]
guesses = [[rng.randrange(C) for _ in range(P)] for _ in range(R)]

print(f'% mastermind: {C} colours, {P} pegs, {R} guesses, seed {a.seed}')
for k in range(1, P + 1):
    # Rounded down: ProbLog refuses an annotated disjunction whose
    # probabilities sum to more than one.
    print('; '.join(f'{math.floor(10**12 / C) / 10**12:.12f}::code({k},c{c})' for c in range(C)) + '.')
for k in range(1, P + 1):
    print(f'peg({k - 1},{k}).')
for c in range(C):
    print(f'colour({c},c{c}).')
for g, guess in enumerate(guesses):
    print(f'round(g{g}).')
    for k, c in enumerate(guess, 1):
        print(f'guess(g{g},{k},c{c}).')
    for c in range(C):
        print(f'guessed(g{g},c{c},{guess.count(c)}).')
print('black(G,0,0) :- round(G).')
print('black(G,K,B) :- peg(J,K), black(G,J,B0), guess(G,K,X), code(K,X), B is B0+1.')
print('black(G,K,B) :- peg(J,K), black(G,J,B), guess(G,K,X), code(K,Y), X \\= Y.')
print('count(X,0,0) :- colour(_,X).')
print('count(X,K,N) :- peg(J,K), count(X,J,N0), code(K,X), N is N0+1.')
print('count(X,K,N) :- peg(J,K), count(X,J,N), code(K,Y), X \\= Y.')
print(f'common(G,X,N) :- guessed(G,X,M), count(X,{P},N), N =< M.')
print(f'common(G,X,M) :- guessed(G,X,M), count(X,{P},N), N > M.')
print('matched(G,0,0) :- round(G).')
print('matched(G,I,T) :- colour(J,X), I is J+1, matched(G,J,T0), common(G,X,M), T is T0+M.')
print(f'white(G,W) :- matched(G,{C},T), black(G,{P},B), W is T-B.')
for g, guess in enumerate(guesses):
    black = sum(x == y for x, y in zip(code, guess))
    common = sum(min(code.count(c), guess.count(c)) for c in range(C))
    print(f'evidence(black(g{g},{P},{black})).')
    print(f'evidence(white(g{g},{common - black})).')
for k in range(1, P + 1):
    for c in range(C):
        print(f'query(code({k},c{c})).')
