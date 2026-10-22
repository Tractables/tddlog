"""The Dice suite's loops, as tddlog answers them on
scripts/dice2problog.py's translation, which writes a loop as recursion
over a step index.

Each of the suite's six loop programs dice_LOOP (data/dice/LOOP.dice) and
its 21 scaled ones dice_LOOP_kK (scripts/dice_scale.py's: the loop run for
K steps) is translated, tddlog answers its queries, and the answers are
checked against tests/dice/loops.json:
the answers tddlog gave on the translation before the loops were written
as recursion (which unrolled them), to 1e-9, and the suite's reference
(ProbLog's d-DNNF, else Alea.jl's, as the results page takes it; none on
the programs that neither answers), to 1e-9 or, for ProbLog, which
prints eight significant digits, to 5e-8 of the answer. The queries are
compared in order: the unrolled translation named them by the loop's
length. The programs need data/dice (the baselines branch) and the
tddlog binary that TDDLOG names; without either the test is skipped.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
data = root / 'data' / 'dice'
TDDLOG = os.environ.get('TDDLOG', '')
LOOPS = json.loads((root / 'tests' / 'dice' / 'loops.json').read_text())


def answers(text):
    """tddlog's answers, in its output's order: (query, probability)."""
    return [(m.group(1), float(m.group(2))) for m in re.finditer(r'^(\S+):\s+(\S+)$', text, re.M)]


@unittest.skipUnless(data.is_dir() and TDDLOG, 'needs data/dice (the baselines branch) and TDDLOG')
class LoopTests(unittest.TestCase):
    def test_the_loops(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name, case in LOOPS.items():
                with self.subTest(name=name):
                    dice, pl = data / f"{case['program']}.dice", Path(tmp) / f'{name}.pl'
                    if case['steps'] is not None:
                        subprocess.run([sys.executable, str(scripts / 'dice_scale.py'), str(dice), str(case['steps']),
                                        str(Path(tmp) / f'{name}.dice')], check=True)
                        dice = Path(tmp) / f'{name}.dice'
                    pl.write_text(subprocess.run([sys.executable, str(scripts / 'dice2problog.py'), str(dice)],
                                                 capture_output=True, text=True, check=True).stdout)
                    out = subprocess.run([TDDLOG, '--precision', '17', str(pl)], capture_output=True, text=True,
                                         check=True, timeout=600).stdout
                    got = [p for _, p in answers(out)]
                    self.assertEqual(len(got), len(case['queries']), out)
                    for q, g, u in zip(case['queries'], got, case['unrolled']):
                        self.assertAlmostEqual(g, u, delta=1e-9, msg=f'{q}: the unrolled translation')
                    ref = case['reference']
                    if ref is not None:
                        rel = 5e-8 if ref['system'].startswith('problog') else 0.0
                        for q, g, r in zip(case['queries'], got, ref['answers']):
                            self.assertAlmostEqual(g, r, delta=max(1e-9, rel * abs(r)), msg=f"{q}: {ref['system']}")


if __name__ == '__main__':
    unittest.main()
