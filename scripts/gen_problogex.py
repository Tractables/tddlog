#!/usr/bin/env python3
"""An example program of ProbLog's repository, as it is.

usage: gen_problogex.py NAME

Prints data/problog/NAME.pl, a copy of ProbLog's test/NAME.pl (Apache
License 2.0; data/problog/README.md). Each states its answers in a comment,
`%Expected outcome:` followed by `% ATOM PROBABILITY` lines, which
lit_exact.py's `expected` reads as the reference.
"""
import os
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
name = sys.argv[1]
text = open(os.path.join(ROOT, 'data', 'problog', name + '.pl'), encoding='utf-8').read()
print(f"% ProbLog's example {name} (test/{name}.pl)")
sys.stdout.write(text if text.endswith('\n') else text + '\n')
