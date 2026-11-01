#!/usr/bin/env python3
"""ProbLog's answers to a program, one query at a time: the reference of
the collections of small programs (the ProbLog tutorial's, cplint's) where
no method of their own computes the answers.

usage: problog_each.py PROGRAM [--problog BIN] [--kb ddnnf]

For each `query(...)` clause of PROGRAM (with or without a body) in turn,
ProbLog (BIN, default $BENCH_PROBLOG, which bench.py run sets from
--problog) reads the program with that query clause alone, its evidence
kept, and the answers of all the runs are printed as ProbLog prints them,
`q:\tp`. ProbLog asked every query at once can answer wrongly (its d-DNNF
route off by 0.078 on a pedigree); one at a time it does not share a
circuit between them. A query ProbLog fails on stops the reference with
its error; inconsistent evidence prints `evidence is inconsistent`.
"""
import argparse
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lpad2problog import split_clauses, strip_comments  # noqa: E402
from problog2pita import split_neck  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--problog', default=os.environ.get('BENCH_PROBLOG', ''))
    ap.add_argument('--kb', default='ddnnf')
    a = ap.parse_args()
    if not a.problog:
        raise SystemExit('problog_each.py: no ProbLog (--problog or $BENCH_PROBLOG)')
    clauses = split_clauses(strip_comments(open(a.program).read()))
    is_query = [bool(re.fullmatch(r'\s*query\s*\(.*\)\s*', split_neck(c)[0], re.S)) for c in clauses]
    rest = [c for c, q in zip(clauses, is_query) if not q]
    queries = [c for c, q in zip(clauses, is_query) if q]
    if not queries:
        raise SystemExit('problog_each.py: the program asks no query')
    seen = set()
    with tempfile.TemporaryDirectory() as tmp:
        for k, q in enumerate(queries):
            path = os.path.join(tmp, f'q{k}.pl')
            with open(path, 'w') as f:
                f.write(''.join(c.strip() + '.\n' for c in rest + [q]))
            r = subprocess.run([a.problog, '-k', a.kb, path], capture_output=True, text=True)
            text = r.stdout + r.stderr
            if 'InconsistentEvidenceError' in text or 'evidence is inconsistent' in text.lower():
                print('evidence is inconsistent')
                return
            if r.returncode != 0:
                sys.exit(f'problog_each.py: ProbLog failed on {q.strip()}: {text.strip()[-500:]}')
            for line in r.stdout.splitlines():
                m = re.fullmatch(r'\s*(.+?):\s+([0-9.eE+-]+)\s*', line)
                if m and m.group(1) not in seen:
                    seen.add(m.group(1))
                    print(f'{m.group(1)}:\t{m.group(2)}')


if __name__ == '__main__':
    main()
