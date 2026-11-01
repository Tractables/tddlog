#!/usr/bin/env python3
"""cplint's probabilistic left-corner grammar, parsing a word of N letters.

usage: gen_plcg.py --template
       gen_plcg.py N [--seed S] --arguments

cplint's example plcg.pl (data/cplint/plcg.pl; Sato, Kameya and Kurihara,
"Variational Bayes via propositionalized probability computation in
PRISM", AMAI 2008): a shift-reduce parser for the grammar S -> S S | a | b
whose choices, the first word of a goal and whether a rule attaches to its
goal or projects to a new left corner, are each indexed by the derivation
so far, a list that grows with every step. The program is the example's,
converted to ProbLog by lpad2problog.py, and its question is the
example's, the probability of a word (`mc_prob(plc([a,b]),P)`), for a word
of N letters drawn uniformly from a and b with seed S. A word of N letters
has Catalan(N - 1) parses, each a sequence of choices of its own, and the
parser tries a goal's every left corner, so the derivations it grounds
grow faster still.

--template prints the program, whose word is its place $1 (tddlog reads
it with the word as its argument: `tddlog plcg.pl [a,a,b,a]`);
--arguments prints the argument of length N and seed S, the word.
"""
import argparse
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
import lpad2problog  # noqa: E402

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')


def template():
    program = lpad2problog.convert(open(os.path.join(ROOT, 'data', 'cplint', 'plcg.pl')).read())
    kept = [line for line in program.rstrip().splitlines() if not line.startswith('query(')]
    return ('% cplint example plcg, converted by lpad2problog.py, written by gen_plcg.py --template:\n'
            "% the probability of the word $1, the program's argument (a list of a and b).\n"
            + '\n'.join(kept).rstrip() + '\n\nquery(plc($1)).\n')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int, nargs='?')
    ap.add_argument('--seed', type=int)
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help='the program, its word the place $1')
    mode.add_argument('--arguments', action='store_true', help='the word of N letters and seed S')
    a = ap.parse_args()
    if a.template:
        if a.n is not None or a.seed is not None:
            ap.error('--template prints the program of every word: N and S give its argument')
        print(template(), end='')
        return
    if a.n is None or a.n < 1:
        ap.error('--arguments needs N, the length of a word: 1 or more')
    rng = random.Random(1 if a.seed is None else a.seed)
    print(f'[{",".join(rng.choice("ab") for _ in range(a.n))}]')


if __name__ == '__main__':
    main()
