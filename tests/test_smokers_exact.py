"""smokers_exact.py against ProbLog's and aspmc's answers on gen_smokers.py's programs."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / 'scripts'

# The standard suite's answers (its table of 2026-10-29):
# ProbLog 2.3.0's (-k ddnnf, eight digits) on s6, s10 and s15, and aspmc's
# (sharpsat-td) on ssym10, which share nothing with the dynamic program.
PROBLOG = {
    ('s6', (6, 2, False)): {'p1': 0.342, 'p2': 0.342, 'p3': 0.3, 'p4': 0.342, 'p5': 0.3, 'p6': 0.342},
    ('s10', (10, 3, False)): {'p1': 0.3932519, 'p2': 0.34883458, 'p3': 0.342, 'p4': 0.3932519, 'p5': 0.3,
                              'p6': 0.3, 'p7': 0.342, 'p8': 0.38788934, 'p9': 0.34883458, 'p10': 0.38788934},
    ('s15', (15, 3, False)): {'p1': 0.342, 'p2': 0.3, 'p3': 0.3, 'p4': 0.3, 'p5': 0.3, 'p6': 0.3, 'p7': 0.342,
                              'p8': 0.3, 'p9': 0.3, 'p10': 0.3, 'p11': 0.342, 'p12': 0.3, 'p13': 0.3,
                              'p14': 0.342, 'p15': 0.3}}
ASPMC = {
    ('ssym10', (10, 3, True)): {'p1': 0.6283028009456268, 'p2': 0.5166916014430301, 'p3': 0.6274486965851394,
                                'p4': 0.5587833453790939, 'p5': 0.5946613411824447, 'p6': 0.5077507096879454,
                                'p7': 0.5170214145396581, 'p8': 0.6232397830889864, 'p9': 0.6568859738634976,
                                'p10': 0.5959954033785615}}


def program(n, friends, symmetric, queries='smokes'):
    return subprocess.run([sys.executable, str(SCRIPTS / 'gen_smokers.py'), str(n), '--friends-per', str(friends),
                           '--seed', '1', '--queries', queries] + (['--symmetric'] if symmetric else []),
                          capture_output=True, text=True, check=True).stdout


def answers(program_text):
    # Scratch files go in the repository's tmp/, which git ignores.
    scratch = ROOT / 'tmp'
    scratch.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile('w', suffix='.pl', dir=scratch) as f:
        f.write(program_text)
        f.flush()
        r = subprocess.run([sys.executable, str(SCRIPTS / 'smokers_exact.py'), f.name, '--precision', '17'],
                           capture_output=True, text=True)
    return r, dict((q, float(p)) for q, p in (line.split(':\t') for line in r.stdout.splitlines()))


class SmokersExact(unittest.TestCase):
    def check(self, refs, delta):
        for (name, args), ref in refs.items():
            r, got = answers(program(*args))
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(set(got), {f'smokes({x})' for x in ref}, name)
            for x, p in ref.items():
                self.assertAlmostEqual(got[f'smokes({x})'], p, delta=delta, msg=f'{name} {x}')

    def test_problog_answers(self):
        self.check(PROBLOG, 1e-8)

    def test_aspmc_answers(self):
        self.check(ASPMC, 1e-12)

    def test_asthma_is_four_tenths_of_smokes(self):
        r, got = answers(program(10, 3, True, 'all'))
        self.assertEqual(r.returncode, 0, r.stderr)
        for x, p in ASPMC[('ssym10', (10, 3, True))].items():
            self.assertAlmostEqual(got[f'asthma({x})'], 0.4 * p, delta=1e-12, msg=x)

    def test_another_program_stops(self):
        text = program(6, 2, False).replace('0.2::influences', '0.25::influences')
        r, _ = answers(text)
        self.assertNotEqual(r.returncode, 0)
        r, _ = answers(program(6, 2, False) + 'evidence(smokes(p1)).\n')
        self.assertNotEqual(r.returncode, 0)


if __name__ == '__main__':
    unittest.main()
