#!/usr/bin/env python3
"""Cora entity resolution as a ProbLog program, from the cplint_datasets file.

    gen_cora.py coralearn.pl [--fold m1] [--queries N] [--transitive] [--seed S]

The theory is the initial LPAD of the learning problem (the `in([...])`
block: every rule has probability 0.3, the same program as cplint's
`cora.pl` example but over a whole fold), the facts are one fold's
author/title/venue/hasword* facts, and the queries are `samebib(A,B)` for N
citation pairs drawn from the fold's positive and negative labels. With
--transitive the transitive rules for samebib/sameauthor/sametitle/samevenue
stay in, making the program recursive; without them it is acyclic, as in
cplint's example.
"""
import argparse
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lpad2problog import convert_clause, split_clauses, split_top, strip_comments  # noqa: E402

ap = argparse.ArgumentParser()
ap.add_argument('file')
ap.add_argument('--fold', default='m1')
ap.add_argument('--queries', type=int, default=10)
ap.add_argument('--transitive', action='store_true')
ap.add_argument('--seed', type=int, default=1)
ap.add_argument('--evidence', type=int, default=0, help='observe this many sampled citation pairs false instead of querying them')
a = ap.parse_args()
if not 0 <= a.evidence < a.queries:
    ap.error('--evidence must be nonnegative and smaller than --queries')
text = open(a.file).read()

# Theory: the clauses inside `in([ ... ])`.
start = re.search(r'^in\(\[', text, re.M).start()
depth, i = 0, start + 3
while True:
    if text[i] == '[':
        depth += 1
    elif text[i] == ']':
        depth -= 1
        if depth == 0:
            break
    i += 1
theory = strip_comments(text[start + 4:i])
rules = []
for clause in split_top(theory, ','):
    clause = clause.strip()
    if clause.startswith('(') and clause.endswith(')'):
        clause = clause[1:-1].strip()
    if not clause:
        continue
    body = clause.split(':-', 1)[1] if ':-' in clause else ''
    head = clause.split(':-', 1)[0]
    pred = re.match(r'\s*(\w+)', head).group(1)
    transitive = body.count(pred + '(') == 2
    if transitive and not a.transitive:
        continue
    rules.append(convert_clause(clause))

# Facts and labels of the fold.
m = re.search(r'^begin\(model\(%s\)\)\.\n(.*?)^end\(model\(%s\)\)\.' % (a.fold, a.fold), text, re.S | re.M)
if not m:
    sys.exit(f'fold {a.fold} not found')
facts, pos, neg = [], [], []
for line in m.group(1).splitlines():
    line = line.strip()
    if not line or line.startswith('%'):
        continue
    if line.startswith('neg(samebib('):
        neg.append(line[4:-2])
    elif line.startswith('samebib('):
        pos.append(line[:-1])
    elif line.startswith(('same', 'neg(')):
        continue  # labels of the other target predicates
    else:
        facts.append(line)
rng = random.Random(a.seed)
pos = [q for q in pos if q.split(',')[0] != q.split(',')[1][:-1]]  # drop samebib(x,x)
picks = rng.sample(pos, min(a.queries - a.queries // 2, len(pos))) + rng.sample(neg, min(a.queries // 2, len(neg)))

print(f'% Cora fold {a.fold} from cplint_datasets; {len(rules)} rules, {len(facts)} facts, {len(picks)} queries'
      f'{", transitive" if a.transitive else ""}')
print('\n'.join(rules))
print('\n'.join(facts))
for q in picks[:a.evidence]:
    # False observations are consistent: all probabilistic rule switches
    # can be false. These are inference workloads, not dataset labels.
    print(f'evidence({q},false).')
for q in picks[a.evidence:]:
    print(f'query({q}).')
