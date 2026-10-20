#!/usr/bin/env python3
"""Generate a two-layer noisy-or diagnosis program with observed findings.

usage: gen_noisyor.py D F [--links L] [--seed S]

The BN2O networks of the UAI 2008 exact inference evaluation, after QMR-DT
(Shwe et al., Methods of Information in Medicine 1991): D diseases with
priors in [0.05, 0.3], F findings, each caused by L distinct random
diseases through noisy-or links of strength [0.2, 0.9] and by a leak of
[0.01, 0.05]. Every finding is observed, with its value in a world sampled
from the model; the positive findings couple their causes, which is what
makes the posterior hard. The posterior of every disease is queried.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('d', type=int)
ap.add_argument('f', type=int)
ap.add_argument('--links', type=int, default=3)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if not 1 <= a.links <= a.d or a.f < 1:
    ap.error('need 1 <= L <= D and at least one finding')
rng = random.Random(a.seed)


def uniform(lo, hi):
    return round(rng.uniform(lo, hi), 3)


prior = [uniform(0.05, 0.3) for _ in range(a.d)]
leak = [uniform(0.01, 0.05) for _ in range(a.f)]
causes = [{i: uniform(0.2, 0.9) for i in sorted(rng.sample(range(a.d), a.links))} for _ in range(a.f)]
ill = [rng.random() < p for p in prior]
present = [rng.random() < leak[j] or any(ill[i] and rng.random() < p for i, p in causes[j].items())
           for j in range(a.f)]

print(f'% noisy-or diagnosis: {a.d} diseases, {a.f} findings, {a.links} causes each, seed {a.seed}')
for i, p in enumerate(prior):
    print(f'{p}::disease(d{i}).')
for j in range(a.f):
    print(f'{leak[j]}::leak(f{j}).')
    for i, p in causes[j].items():
        print(f'{p}::cause(d{i},f{j}).')
print('finding(F) :- leak(F).')
print('finding(F) :- disease(D), cause(D,F).')
for j in range(a.f):
    print(f'evidence(finding(f{j}),{"true" if present[j] else "false"}).')
for i in range(a.d):
    print(f'query(disease(d{i})).')
