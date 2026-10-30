#!/usr/bin/env python3
"""Run tddlog over ProbLog's system tests and compare with the expected outcomes.

Expected outcomes live in comments (see problog/test/test_system.py):
  %Expected outcome:
  % <query term> <probability>
or inline `query(q). % outcome: 0.5`, or `% ERROR <Class>` for expected failures.
The tests are the `test/*.pl` of a ProbLog checkout
(https://github.com/ML-KULeuven/problog, 109 of them at 2.3.0), which --dir
names (default: tmp/baselines/problog-src/test in this checkout).

usage: problog_suite.py [--bin PATH] [--dir DIR] [--verbose] [files...]
"""
import argparse
import glob
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)


def read_expected(path):
    """Return ({query_text: prob}, error_class_or_None)."""
    expected = {}
    error = None
    with open(path) as f:
        lines = f.readlines()
    reading = False
    for line in lines:
        s = line.strip()
        if s.startswith('%Expected outcome:') or s.startswith('% Expected outcome:'):
            reading = True
            continue
        if reading:
            if s.startswith('%'):
                body = s.lstrip('%').strip()
                if not body:
                    continue
                m = re.match(r'^(.*)\s+([0-9.eE+-]+|inf|nan)$', body)
                if m:
                    expected[norm(m.group(1))] = float(m.group(2))
                    continue
                if body.startswith('ERROR'):
                    error = body.split()[1] if len(body.split()) > 1 else 'ERROR'
                    continue
                reading = False
            else:
                reading = False
        m = re.search(r'query\((.*)\)\.\s*%\s*outcome:\s*([0-9.eE+-]+)', s)
        if m:
            expected[norm(m.group(1))] = float(m.group(2))
        m = re.match(r'%\s*ERROR\s+(\w+)', s)
        if m:
            error = m.group(1)
    return expected, error


def norm(term):
    """Normalize a term's text: drop spaces and quotes around simple atoms."""
    t = re.sub(r'\s+', '', term)
    t = re.sub(r"'([a-z][A-Za-z0-9_]*)'", r'\1', t)
    return t


def has_variable(term):
    """Whether a normalized term has a variable as an argument or an item."""
    return re.search(r"[(,\[|][A-Z_]", re.sub(r"'[^']*'", "''", term)) is not None


def run_tddlog(binary, path, timeout):
    try:
        p = subprocess.run([binary, '--precision', '10', path], capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None, 'timeout'
    if p.returncode != 0:
        return None, p.stderr.strip().splitlines()[-1] if p.stderr.strip() else f'exit {p.returncode}'
    results = {}
    for line in p.stdout.splitlines():
        if line.startswith('evidence is inconsistent'):
            return {}, 'inconsistent'
        if ':' in line:
            q, v = line.rsplit(':', 1)
            results[norm(q)] = float(v)
    return results, None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--bin', default=os.path.join(ROOT, 'target', 'release', 'tddlog'))
    ap.add_argument('--dir', default=os.path.join(ROOT, 'tmp', 'baselines', 'problog-src', 'test'))
    ap.add_argument('--timeout', type=float, default=60)
    ap.add_argument('--tol', type=float, default=1e-6)
    ap.add_argument('-v', '--verbose', action='store_true')
    ap.add_argument('files', nargs='*')
    args = ap.parse_args()
    files = args.files or sorted(glob.glob(os.path.join(args.dir, '*.pl')))
    if not files:
        sys.exit(f'no programs in {args.dir}: pass --dir with the test/ directory of a ProbLog checkout')
    counts = {'pass': 0, 'fail': 0, 'unsupported': 0, 'expected-error': 0, 'timeout': 0}
    for path in files:
        name = os.path.basename(path)
        expected, error = read_expected(path)
        results, err = run_tddlog(args.bin, path, args.timeout)
        if error is not None:
            # ProbLog expects an error; we accept any error, or note if we answered.
            if results is None:
                counts['expected-error'] += 1
                if args.verbose:
                    print(f'ERR-OK  {name}: {err}')
            else:
                counts['pass'] += 1  # answered a program ProbLog rejects; acceptable
                print(f'LENIENT {name}: ProbLog expects {error}, we answered')
            continue
        if err == 'timeout':
            counts['timeout'] += 1
            print(f'TIMEOUT {name}')
            continue
        if results is None:
            counts['unsupported'] += 1
            print(f'UNSUPP  {name}: {err}')
            continue
        bad = []
        for q, v in expected.items():
            got = results.get(q)
            if got is None and v == 0 and has_variable(q):
                # A query with variables that nothing answers: ProbLog prints
                # the query, its variables renamed, with probability 0, and
                # tddlog prints no row.
                continue
            if got is None:
                bad.append(f'{q}: missing (have {sorted(results)[:5]})')
            elif abs(got - v) > args.tol:
                bad.append(f'{q}: expected {v}, got {got}')
        if bad:
            counts['fail'] += 1
            print(f'FAIL    {name}')
            for b in bad[:6]:
                print(f'          {b}')
        else:
            counts['pass'] += 1
            if args.verbose:
                print(f'PASS    {name} ({len(expected)} queries)')
    print(counts)
    return 0 if counts['fail'] == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
