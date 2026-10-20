"""The exact network baselines that read UAI files (scripts/bn_uai_run.py,
libdai_uai.cpp, tensorinference_uai.jl) against a brute-force sum over the
joint assignments, on small BAYES and MARKOV files with observations.
Each system is tested where its tool is given: BN_PYTHON (a Python with
pyAgrum and pgmpy), BN_LIBDAI (the libdai_uai binary), BN_TENSORINFERENCE
(the Julia command with its project, up to the script)."""
import itertools
import math
import os
import shlex
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

scripts = Path(__file__).resolve().parents[1] / 'scripts'
sys.path.insert(0, str(scripts))
import bench  # noqa: E402

BAYES = """BAYES
4
2 3 2 2
4
1 0
2 0 1
2 0 2
3 0 2 3
2
0.3 0.7
6
0.2 0.5 0.3 0.6 0.1 0.3
4
0.9 0.1 0.4 0.6
8
0.5 0.5 0.1 0.9 0.7 0.3 0.25 0.75
"""
MARKOV = """MARKOV
4
2 3 2 3
4
2 0 1
2 1 2
2 2 3
1 3
6
1.5 0.5 2.0 1.0 0.25 3.0
6
2.0 0.5 0.5 4.0 1.0 0.7
6
1.0 2.0 3.0 0.5 0.5 1.5
3
0.5 1.0 2.5
"""
EVID = {'none': '0\n', 'two': '2 1 2 3 0\n'}


def brute(text, evid_text):
    tok = text.split()
    pos = 1
    n = int(tok[pos]); pos += 1
    sizes = [int(x) for x in tok[pos:pos + n]]; pos += n
    m = int(tok[pos]); pos += 1
    scopes = []
    for _ in range(m):
        k = int(tok[pos]); scopes.append([int(x) for x in tok[pos + 1:pos + 1 + k]]); pos += 1 + k
    tables = []
    for sc in scopes:
        e = int(tok[pos]); pos += 1
        tables.append(np.array(tok[pos:pos + e], dtype=float).reshape([sizes[v] for v in sc])); pos += e
    et = evid_text.split()
    evid = {int(et[1 + 2 * i]): int(et[2 + 2 * i]) for i in range(int(et[0]))}
    marg = [np.zeros(k) for k in sizes]
    z = 0.0
    for a in itertools.product(*[range(k) for k in sizes]):
        if any(a[v] != u for v, u in evid.items()):
            continue
        w = 1.0
        for sc, t in zip(scopes, tables):
            w *= t[tuple(a[v] for v in sc)]
        z += w
        for v in range(n):
            marg[v][a[v]] += w
    return math.log10(z), [x / z for x in marg], evid


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise AssertionError(f'{cmd[-5:]}: {r.stderr[-1500:]}')


def systems():
    out = {}
    py = os.environ.get('BN_PYTHON')
    if py:
        for s in ('agrum', 'pgmpy'):
            out[s] = [py, str(scripts / 'bn_uai_run.py'), s]
    if os.environ.get('BN_LIBDAI'):
        out['libdai'] = [os.environ['BN_LIBDAI']]
    if os.environ.get('BN_TENSORINFERENCE'):
        cmd = shlex.split(os.environ['BN_TENSORINFERENCE']) + [str(scripts / 'tensorinference_uai.jl')]
        out['tensorinference'] = cmd
        out['tensorinference_greedy'] = ['env', 'BN_VARIANT=greedy'] + cmd
    return out


class BaselineTests(unittest.TestCase):
    def check(self, system, cmd):
        for name, text in (('bayes', BAYES), ('markov', MARKOV)):
            for en, et in EVID.items():
                with self.subTest(system=system, network=name, evidence=en), tempfile.TemporaryDirectory() as d:
                    model, evid = Path(d, 'm.uai'), Path(d, 'm.evid')
                    model.write_text(text)
                    evid.write_text(et)
                    log10_z, marg, observed = brute(text, et)
                    for task in ('PR', 'MAR'):
                        out = Path(d, 'out.' + task)
                        run(cmd + [str(model), str(evid), str(out), task])
                        body = out.read_text()
                        if task == 'PR':
                            self.assertAlmostEqual(bench.parse_merlin_pr(body), log10_z, places=8)
                        else:
                            rows = bench.parse_merlin_marginals(body)
                            for v, row in enumerate(rows):
                                self.assertEqual(len(row), len(marg[v]))
                                for p, q in zip(row, marg[v]):
                                    self.assertAlmostEqual(p, q, places=8)
                    # a subset of the variables, in the order given
                    run(cmd + [str(model), str(evid), str(out), 'MAR', '3,0'])
                    rows = bench.parse_merlin_marginals(out.read_text())
                    self.assertEqual(len(rows), 2)
                    for p, q in zip(rows[0], marg[3]):
                        self.assertAlmostEqual(p, q, places=8)
                    for p, q in zip(rows[1], marg[0]):
                        self.assertAlmostEqual(p, q, places=8)


for _name, _cmd in systems().items():
    setattr(BaselineTests, f'test_{_name}', lambda self, n=_name, c=_cmd: self.check(n, c))

if __name__ == '__main__':
    unittest.main()
