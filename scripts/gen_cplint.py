#!/usr/bin/env python3
"""A cplint example program from data/cplint/ converted to ProbLog by
tddlog's LPAD reader (`tddlog --program`, the binary $TDDLOG or the source
tree's).

    gen_cplint.py NAME        (data/cplint/NAME.pl)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402  (tddlog --program)

name = sys.argv[1]
src = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', name + '.pl')
print(f'% cplint example {name}, converted by tddlog --program')
sys.stdout.write(source_tree.program(src))
