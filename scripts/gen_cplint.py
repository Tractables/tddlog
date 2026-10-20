#!/usr/bin/env python3
"""A cplint example program from data/cplint/ converted to ProbLog.

    gen_cplint.py NAME        (data/cplint/NAME.pl)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lpad2problog  # noqa: E402

name = sys.argv[1]
src = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', name + '.pl')
print(f'% cplint example {name}, converted by lpad2problog.py')
print(lpad2problog.convert(open(src).read()))
