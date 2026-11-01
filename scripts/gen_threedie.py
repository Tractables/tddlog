#!/usr/bin/env python3
"""PITA's three-sided die, thrown until it shows 3 (Riguzzi and Swift, ICLP
2010 Technical Communications, Section 4, Fig. 3; cplint's
threesideddice.pl), asked the probability that throw N shows 1.

usage: gen_threedie.py --template
       gen_threedie.py N --arguments

--template prints the program, whose throw is its place $1 (tddlog reads
it with the throw as its argument: `tddlog threedie.pl 100`); --arguments
prints the argument of throw N, N.

The LPAD's clauses as ProbLog writes them: each throw before the first 3
is followed by another, so throw N shows 1 with probability (2/3)^N / 3
(lit_exact.py threedie).
"""
import argparse

TEMPLATE = """\
% PITA's three-sided die (cplint threesideddice.pl), written by gen_threedie.py --template:
% on($1,1), whether throw $1, the program's argument, shows 1.
1/3::on(0,1); 1/3::on(0,2); 1/3::on(0,3).
1/3::on(X,1); 1/3::on(X,2); 1/3::on(X,3) :- X1 is X-1, X1 >= 0, on(X1,_), \\+ on(X1,3).
query(on($1,1)).
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int, nargs='?')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help='the program, its throw the place $1')
    mode.add_argument('--arguments', action='store_true', help="throw N's argument")
    a = ap.parse_args()
    if a.template:
        if a.n is not None:
            ap.error('--template prints the program of every throw: N is its argument')
        print(TEMPLATE, end='')
        return
    if a.n is None or a.n < 0:
        ap.error('--arguments needs the throw N: 0 or more')
    print(a.n)


if __name__ == '__main__':
    main()
