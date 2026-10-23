#!/usr/bin/env python3
"""Run tddlog and ProbLog on the same program(s); compare answers and time.

usage: compare.py [--problog-k KB] [--timeout S] [--tddlog-args "..."] file.pl...
"""
import argparse
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TDDLOG = os.path.join(ROOT, 'target', 'release', 'tddlog')
# ProbLog: $PROBLOG, else a virtualenv in the checkout's tmp/venv, else the
# one on PATH (pip install problog).
VENV_PROBLOG = os.path.join(ROOT, 'tmp', 'venv', 'bin', 'problog')
PROBLOG = os.environ.get('PROBLOG') or (VENV_PROBLOG if os.path.exists(VENV_PROBLOG) else 'problog')


def parse(out):
    res = {}
    for line in out.splitlines():
        if line.startswith('evidence is inconsistent'):
            return 'inconsistent'
        if ':' in line:
            q, v = line.rsplit(':', 1)
            try:
                res[re.sub(r'\s+', '', q)] = float(v)
            except ValueError:
                pass
    return res


def run(cmd, timeout):
    t0 = time.time()
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None, timeout, 'timeout'
    dt = time.time() - t0
    if p.returncode != 0:
        err = p.stderr.strip().splitlines()
        return None, dt, err[-1] if err else f'exit {p.returncode}'
    return parse(p.stdout), dt, None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--problog-k', default=None, help='ProbLog knowledge compilation backend (ddnnf, sdd, fsdd, ...)')
    ap.add_argument('--timeout', type=float, default=300)
    ap.add_argument('--tol', type=float, default=1e-6)
    ap.add_argument('--tddlog-args', default='')
    ap.add_argument('--no-problog', action='store_true')
    ap.add_argument('files', nargs='+')
    a = ap.parse_args()
    status = 0
    for f in a.files:
        r1, t1, e1 = run([TDDLOG, '--precision', '10'] + a.tddlog_args.split() + [f], a.timeout)
        line = f'{os.path.basename(f):30s} tddlog {t1:7.2f}s' + (f' ERROR {e1}' if e1 else '')
        if not a.no_problog:
            cmd = [PROBLOG] + (['-k', a.problog_k] if a.problog_k else []) + [f]
            r2, t2, e2 = run(cmd, a.timeout)
            line += f'   problog{"-" + a.problog_k if a.problog_k else ""} {t2:7.2f}s' + (f' ERROR {e2}' if e2 else '')
            if r1 is not None and r2 is not None:
                if r1 == 'inconsistent' or r2 == 'inconsistent':
                    line += '   ' + ('agree (inconsistent)' if r1 == r2 else 'DISAGREE on inconsistency')
                else:
                    bad = [(q, r1.get(q), v) for q, v in r2.items() if r1.get(q) is None or abs(r1[q] - v) > a.tol]
                    if bad:
                        status = 1
                        line += f'   DISAGREE {len(bad)}/{len(r2)}: ' + '; '.join(f'{q} {x} vs {v}' for q, x, v in bad[:3])
                    else:
                        line += f'   agree ({len(r2)} queries)'
        print(line, flush=True)
    return status


if __name__ == '__main__':
    sys.exit(main())
