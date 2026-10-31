#!/usr/bin/env python3
"""A cplint example program converted to ProbLog by tddlog's LPAD reader
(`tddlog --program`, the binary $TDDLOG or the source tree's).

    gen_cplint.py NAME        (data/cplint/NAME.pl, the cplint pack's)
    gen_cplint.py NAME swish  (data/swish/NAME.pl, cplint on SWISH's)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402  (tddlog --program)

name, where = (sys.argv[1:] + ['cplint'])[:2]
if where not in ('cplint', 'swish') or len(sys.argv) > 3:
    sys.exit('usage: gen_cplint.py NAME [swish]')
src = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', where, name + '.pl')
print(f'% cplint example {name}, converted by tddlog --program')
sys.stdout.write(source_tree.program(src))
