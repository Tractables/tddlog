#!/usr/bin/env python3
"""UW-CSE (cplint's uwcse3s example) with a chosen number of students.

    gen_uwcse.py N --template
    gen_uwcse.py N --arguments

The family is one program and its argument (bench.py's FOLD): --template
prints data/cplint/uwcse3s.pl converted to ProbLog (`tddlog --program`, the
binary $TDDLOG or the source tree's), with its three students replaced by
the rule `student(S) :- between(1, $1, S).`, the students 1 to its place $1;
the course, the professor and the query taught_by(c1,p1) stay. --arguments
prints $1, N. The formula of taught_by aggregates over the students, which
is what makes the family grow.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402  (tddlog --program)

SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', 'uwcse3s.pl')


def template():
    out = []
    for line in source_tree.program(SRC).splitlines():
        if line.startswith('student(s'):
            continue
        out.append(line)
        if line == 'course(c1).':
            out.append('student(S) :- between(1,$1,S).')
    return '% UW-CSE (cplint uwcse3s) with $1 students\n' + '\n'.join(out) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true')
    mode.add_argument('--arguments', action='store_true')
    a = ap.parse_args()
    if a.template:
        print(template(), end='')
    else:
        print(a.n)


if __name__ == '__main__':
    main()
