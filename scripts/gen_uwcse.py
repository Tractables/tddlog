#!/usr/bin/env python3
"""UW-CSE (cplint's uwcse3s example) with a chosen number of students.

    gen_uwcse.py N --template
    gen_uwcse.py N --arguments
    gen_uwcse.py N --data KEY

The family is one program and its data (bench.py's FOLD): --template
prints data/cplint/uwcse3s.pl converted to ProbLog (`tddlog --program`, the
binary $TDDLOG or the source tree's), with its three students replaced by
`:- consult($1).`, the file of the students; the course, the professor and
the query taught_by(c1,p1) stay. --data prints the facts student(1) to
student(N) (KEY is only its name); --arguments prints nothing. The formula
of taught_by aggregates over the students, which is what makes the family
grow. The students are facts, not a rule over between/3, which aspmc and
PSouffle read as another program.
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
            out.append(':- consult($1).')
    return '% UW-CSE (cplint uwcse3s) with the students of $1\n' + '\n'.join(out) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true')
    mode.add_argument('--arguments', action='store_true')
    mode.add_argument('--data', metavar='KEY')
    a = ap.parse_args()
    if a.template:
        print(template(), end='')
    elif a.data:
        print(f'% UW-CSE: {a.n} students')
        print(''.join(f'student({i}).\n' for i in range(1, a.n + 1)), end='')


if __name__ == '__main__':
    main()
