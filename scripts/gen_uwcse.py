#!/usr/bin/env python3
"""UW-CSE (cplint's uwcse3s example) with a chosen number of students.

    gen_uwcse.py N

Converts data/cplint/uwcse3s.pl to ProbLog (`tddlog --program`, the binary
$TDDLOG or the source tree's) and replaces its three students by
s1..sN; the course, the professor and the query taught_by(c1,p1) stay. The
formula of taught_by aggregates over the students, which is what makes the
family grow.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402  (tddlog --program)

n = int(sys.argv[1])
src = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', 'uwcse3s.pl')
out = []
for line in source_tree.program(src).splitlines():
    if line.startswith('student(s'):
        continue
    out.append(line)
    if line == 'course(c1).':
        out += [f'student(s{i}).' for i in range(1, n + 1)]
print(f'% UW-CSE (cplint uwcse3s) with {n} students')
print('\n'.join(out))
