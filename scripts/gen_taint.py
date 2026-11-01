#!/usr/bin/env python3
"""Write a taint analysis of an Android app as tddlog and ProbLog read it.

usage: gen_taint.py PROGRAM

The taint benchmarks of PSouffle: Li, Xia, Adnan, Wang, "PSouffle: Scaling
Exact Probabilistic Logic Inference for Program Analysis" (FMCAD 2026).
Each of the 15 apps is a ProbLog program of the same 104 probabilistic
rules (a context-insensitive and a context-sensitive points-to analysis,
a type filter and a taint analysis, most rules of probability 0.999), the
app's facts, from 8,000 to 57,000, and up to 1,637 probabilistic facts
(the types of variables and objects, the labels of sources and sinks); 30
queries ask every tuple of the derived relations.
PROGRAM is the app's compute.problog.dl from the PSouffle repository
(evaluation/full/taint/APP/, MIT licence), which data/fetch.txt names,
gzipped or not.

The program's two constructs that tddlog does not read are rewritten, as
ProbLog reads the result alike:

- the negation of a comparison, `\\+(I = 0)` with I bound by the literal
  before it, is the comparison `I \\= 0`;
- a relation's declaration `r(_, _) :- fail.`, which ProbLog needs for a
  relation with no other clause, has a head of variables that its body
  does not bind; its head is the ground atom `r(0, 0)`, which the clause
  derives just as little.

The program is otherwise unchanged.
"""
import gzip
import re
import sys

if len(sys.argv) != 2:
    sys.exit(__doc__.split('\n\n')[1])
with (gzip.open if sys.argv[1].endswith('.gz') else open)(sys.argv[1], 'rt') as f:
    text = f.read()
text, negations = re.subn(r'\\\+\((\w+) = (\w+)\)', r'\1 \\= \2', text)
text, stubs = re.subn(r'^(\w+)\((_(?:, _)*)\) :- fail\.$',
                      lambda m: f'{m[1]}({", ".join("0" * m[2].count("_"))}) :- fail.', text, flags=re.M)
if negations == 0 or stubs == 0:
    sys.exit(f'gen_taint: {sys.argv[1]} has {negations} negated comparisons and {stubs} declarations; '
             f'is it a PSouffle taint program?')
sys.stdout.write(text)
