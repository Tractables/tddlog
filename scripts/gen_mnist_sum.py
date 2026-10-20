#!/usr/bin/env python3
"""Generate an MNIST sum-N program: the distribution of the sum of N digits.

usage: gen_mnist_sum.py N [--seed S] [--peak P]

The neurosymbolic task of DeepProbLog's single-digit addition (Manhaeve et
al., NeurIPS 2018) grown to N digits as in Scallop's sum2/sum3/sum4 (Li,
Huang, Naik, PLDI 2023, where DeepProbLog times out on sum4). Each digit
image is an annotated disjunction over 0..9 that stands in for the
classifier's softmax: mass P on a true digit drawn from the seed, the rest
spread at random, in units of 1e-4. The sum is accumulated left to right in
partial sums, and every value 0..9N is queried, the distribution a training
step asks for.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--peak', type=float, default=0.7)
a = ap.parse_args()
if a.n < 1 or not 0 <= a.peak <= 1:
    ap.error('N must be positive and the peak in [0, 1]')
rng = random.Random(a.seed)


def softmax(true):
    """Ten probabilities in units of 1e-4 summing to one: the noise first,
    rounded down, and the rest on the true digit."""
    w = [rng.random() for _ in range(10)]
    units = [int((1 - a.peak) * 10000 * x / sum(w)) for x in w]
    units[true] += 10000 - sum(units)
    return units


truth = [rng.randrange(10) for _ in range(a.n)]
print(f'% MNIST sum-{a.n}: {a.n} digits, peak {a.peak}, seed {a.seed}, true digits {"".join(map(str, truth))}')
for i, t in enumerate(truth, 1):
    print('; '.join(f'{u / 10000:.4f}::digit({i},{d})' for d, u in enumerate(softmax(t))) + '.')
for i in range(2, a.n + 1):
    print(f'next({i - 1},{i}).')
print('partial(1,S) :- digit(1,S).')
print('partial(K,S) :- next(J,K), partial(J,S0), digit(K,D), S is S0+D.')
print(f'sum(S) :- partial({a.n},S).')
for s in range(9 * a.n + 1):
    print(f'query(sum({s})).')
