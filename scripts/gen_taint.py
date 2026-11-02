#!/usr/bin/env python3
"""Write a taint analysis of an Android app as tddlog and ProbLog read it.

usage: gen_taint.py PROGRAM --template
       gen_taint.py PROGRAM --data KEY

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

The program is otherwise unchanged. The family is one program and its data
(bench.py's FOLD): --template prints the 104 rules and the 30 queries, which
consult the app (their place $1); --data prints the rest of the app's
program, its declarations, facts and header, KEY being only its name. A rule
is a clause whose body is more than `fail`; every app has the same rules.
"""
import argparse
import gzip
import re
import sys

ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[1])
ap.add_argument('program')
mode = ap.add_mutually_exclusive_group(required=True)
mode.add_argument('--template', action='store_true')
mode.add_argument('--arguments', action='store_true')
mode.add_argument('--data', metavar='KEY')
a = ap.parse_args()
with (gzip.open if a.program.endswith('.gz') else open)(a.program, 'rt') as f:
    text = f.read()
text, negations = re.subn(r'\\\+\((\w+) = (\w+)\)', r'\1 \\= \2', text)
text, stubs = re.subn(r'^(\w+)\((_(?:, _)*)\) :- fail\.$',
                      lambda m: f'{m[1]}({", ".join("0" * m[2].count("_"))}) :- fail.', text, flags=re.M)
if negations == 0 or stubs == 0:
    sys.exit(f'gen_taint: {a.program} has {negations} negated comparisons and {stubs} declarations; '
             f'is it a PSouffle taint program?')
lines = text.splitlines(keepends=True)


def program_line(x):
    """A rule or a query: the part of the app's text that is the analysis."""
    return (':-' in x and not x.rstrip().endswith(':- fail.')) or x.startswith('query(')


if a.template:
    sys.stdout.write("% PSouffle's taint analysis (points-to and taint rules); $1 is an app's declarations and facts\n"
                     ':- consult($1).\n' + ''.join(x for x in lines if program_line(x)))
elif a.data:
    sys.stdout.write(''.join(x for x in lines if not program_line(x)))
