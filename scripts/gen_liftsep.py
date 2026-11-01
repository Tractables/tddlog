#!/usr/bin/env python3
"""Generate the safe queries that separate lifted from grounded inference.

usage: gen_liftsep.py N [--seed S]

Beame, Li, Roy, Suciu, "Exact model counting of query expressions:
limitations of propositional methods" (ACM TODS 42(1), 2017; ICDT 2014),
Section 3: over a complete domain of N constants, a tuple-independent
database with unary relations R and T and binary relations S1, S2, S3,
every one of the 2N + 3N^2 tuples present with its own probability, and
the four conjunctive queries of h3:

    h30 :- R(x), S1(x,y)      h31 :- S1(x,y), S2(x,y)
    h32 :- S2(x,y), S3(x,y)   h33 :- S3(x,y), T(y)

Their disjunction h3 is #P-hard. Two Boolean combinations of them are
asked, the queries QW and Q9 of the paper's Example 3.6:

    qw = (h30 or h32) and (h30 or h33) and (h31 or h33)
    q9 = (h30 or h33) and (h31 or h33) and (h32 or h33) and (h30 or h31 or h32)

Both are safe: inclusion-exclusion over their clauses cancels the term of
h3, and every other term is a product of polynomial sums, so lifted
inference answers them in time polynomial in N (lit_exact.py does). Every
decision-DNNF of their lineage has size 2^Omega(sqrt N) (Theorem 3.4),
which bounds every model counter that searches or compiles the grounded
lineage. Each tuple's probability is drawn from [0.05, 0.95] and scaled by
min(1, 2/N), so that each h3l has about one satisfying pair whatever N and
the answers stay away from 0 and 1; the probabilities do not change the
lineage.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if a.n < 1:
    ap.error('need N >= 1')
rng = random.Random(a.seed)
N = a.n
scale = min(1.0, 2 / N)


def prob():
    return round(rng.uniform(0.05, 0.95) * scale, 2 if scale == 1.0 else 4)


print(f'% the queries QW and Q9 of Beame, Li, Roy and Suciu over a complete domain of {N} constants, seed {a.seed}')
for i in range(N):
    print(f'{prob()}::r(c{i}).')
for j in range(N):
    print(f'{prob()}::t(c{j}).')
for rel in ('s1', 's2', 's3'):
    for i in range(N):
        for j in range(N):
            print(f'{prob()}::{rel}(c{i},c{j}).')
print('h30 :- r(X), s1(X,Y).')
print('h31 :- s1(X,Y), s2(X,Y).')
print('h32 :- s2(X,Y), s3(X,Y).')
print('h33 :- s3(X,Y), t(Y).')
for name, clauses in (('qw', [(0, 2), (0, 3), (1, 3)]),
                      ('q9', [(0, 3), (1, 3), (2, 3), (0, 1, 2)])):
    for k, clause in enumerate(clauses):
        for ell in clause:
            print(f'{name}_{k} :- h3{ell}.')
    print(f'{name} :- ' + ', '.join(f'{name}_{k}' for k in range(len(clauses))) + '.')
print('query(qw).')
print('query(q9).')
