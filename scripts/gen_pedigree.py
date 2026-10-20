#!/usr/bin/env python3
"""Generate a blood type program on a random pedigree.

usage: gen_pedigree.py G W [--observe F] [--seed S]

The rules are cplint's bloodtype example (data/cplint/bloodtype.pl; Meert,
Struyf, Blockeel, ILP 2009), the benchmark PITA scales by family size
(Riguzzi and Swift, TPLP 2011, Fig. 5a). There the family is a tree of
ancestors; here it is a pedigree of G generations of W people, as in
genetic linkage networks. Founders have the example's allele priors, and
everyone after them a father and a mother drawn at random from the
generation before, so half siblings and cousins have children together and
the pedigree has loops. Each person of the first G-1 generations has
their blood type observed with probability F, the value sampled forward
from the model; the blood types of the last generation are queried.
"""
import argparse
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lpad2problog  # noqa: E402

RULES = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', 'bloodtype.pl')
PRIOR = [('a', 0.3), ('b', 0.3), ('null', 0.4)]


def table(rules, head):
    """The rows of the bloodtype rules for `head` (pchrom, mchrom or
    bloodtype): the pchrom and mchrom alleles the body reads, of the parent
    or of the person, to the head's values and probabilities."""
    rows = {}
    for r in rules:
        heads, _, body = r.partition(':-')
        if heads.split('::')[1].startswith(head + '('):
            key = tuple(re.search(rf'{c}chrom\(\w+,\s*(\w+)\s*\)', body).group(1) for c in 'pm')
            rows[key] = [(re.search(r',(\w+)\)', h).group(1), float(h.split('::')[0])) for h in heads.split(';')]
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('g', type=int)
    ap.add_argument('w', type=int)
    ap.add_argument('--observe', type=float, default=0.3)
    ap.add_argument('--seed', type=int, default=1)
    a = ap.parse_args()
    if a.g < 2 or a.w < 2:
        ap.error('a pedigree needs two generations of at least two people')
    rng = random.Random(a.seed)
    rules = [line for line in lpad2problog.convert(open(RULES).read()).splitlines() if ':-' in line]
    assert len(rules) == 27, len(rules)

    def draw(row):
        """A value of `row`, or None for the mass it leaves to no value."""
        u, acc = rng.random(), 0.0
        for v, p in row:
            acc += p
            if u < acc:
                return v
        return None

    people = [[f'i{g}_{k}' for k in range(a.w)] for g in range(a.g)]
    parents = {}
    for g in range(1, a.g):
        males, females = people[g - 1][0::2], people[g - 1][1::2]
        for p in people[g]:
            parents[p] = (rng.choice(males), rng.choice(females))
    # A world sampled forward gives the observed blood types; a person whose
    # row leaves them no blood type goes unobserved.
    inherit = {'p': table(rules, 'pchrom'), 'm': table(rules, 'mchrom')}
    blood = table(rules, 'bloodtype')
    alleles = {}
    for g in people:
        for p in g:
            if p in parents:
                f, m = parents[p]
                alleles[p] = (draw(inherit['p'][alleles[f]]), draw(inherit['m'][alleles[m]]))
            else:
                alleles[p] = (draw(PRIOR), draw(PRIOR))
    evidence = []
    for g in people[:-1]:
        for p in g:
            if rng.random() < a.observe:
                v = draw(blood[alleles[p]])
                if v is not None:
                    evidence.append((p, v))

    print(f'% blood type on a random pedigree: {a.g} generations of {a.w}, observed {a.observe}, seed {a.seed}')
    print("% rules of cplint's bloodtype example (data/cplint/bloodtype.pl)")
    print('\n'.join(rules))
    for p in people[0]:
        for c in ('mchrom', 'pchrom'):
            print('; '.join(f'{q}::{c}({p},{v})' for v, q in PRIOR) + '.')
    for p, (f, m) in parents.items():
        print(f'father({f},{p}).')
        print(f'mother({m},{p}).')
    for p, v in evidence:
        print(f'evidence(bloodtype({p},{v})).')
    for p in people[-1]:
        for v in ('a', 'b', 'ab', 'null'):
            print(f'query(bloodtype({p},{v})).')


if __name__ == '__main__':
    main()
