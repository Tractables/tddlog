#!/usr/bin/env python3
"""cplint's hidden Markov model of DNA sequences, emitting a sequence of length N.

usage: gen_dnahmm.py --template
       gen_dnahmm.py N [--seed S] --arguments

cplint's example hmm.pl (data/cplint/hmm.pl; Christiansen and Gallagher,
"Non-discriminating arguments and their uses", ICLP 2009): from state q1
the chain moves to q1, q2 or end and emits one of a, c, g and t, each
choice indexed by the list of states before it, so that every choice of
a sequence of states is a variable of its own, as in PRISM. The program
is the example's, converted to ProbLog by lpad2problog.py, and its
question is the example's, the probability that the model emits a given
sequence (`prob(hmm([a,c]),Prob)`), for a sequence of length N drawn
uniformly with seed S. A sequence of length N has 2^(N-1) state
sequences, each indexing choices of its own.

--template prints the program, whose sequence is its place $1 (tddlog
reads it with the sequence as its argument: `tddlog dnahmm.pl
[c,a,g,a]`); --arguments prints the argument of length N and seed S,
the sequence.
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
    program = lpad2problog.convert(open(os.path.join(ROOT, 'data', 'cplint', 'hmm.pl')).read())
    # The example's own queries, which lpad2problog.py turns into query
    # facts, give way to this one.
    kept = [line for line in program.rstrip().splitlines() if not line.startswith('query(')]
    return ('% cplint example hmm, converted by lpad2problog.py, written by gen_dnahmm.py --template:\n'
            "% the probability of the sequence $1, the program's argument (a list of a, c, g and t).\n"
            + '\n'.join(kept).rstrip() + '\n\nquery(hmm($1)).\n')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int, nargs='?')
    ap.add_argument('--seed', type=int)
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help='the program, its sequence the place $1')
    mode.add_argument('--arguments', action='store_true', help='the sequence of length N and seed S')
    a = ap.parse_args()
    if a.template:
        if a.n is not None or a.seed is not None:
            ap.error('--template prints the program of every sequence: N and S give its argument')
        print(template(), end='')
        return
    if a.n is None or a.n < 1:
        ap.error('--arguments needs N, the length of a sequence: 1 or more')
    rng = random.Random(1 if a.seed is None else a.seed)
    print(f'[{",".join(rng.choice("acgt") for _ in range(a.n))}]')


if __name__ == '__main__':
    main()
