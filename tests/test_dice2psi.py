"""Dice programs as Psi programs (scripts/dice2psi.py) and how bench.py reads
what Psi prints.

Every Dice program of data/dice and data/nodice that dice2problog.py
translates translates here, except where it has recursion (Psi has none),
with its braces balanced and a component's program for each component. Where
Psi is installed (PSI, the command that runs it, e.g. `prlimit
--stack=unlimited $HOME/local/psi/psi-src/psi`), the translated programs run,
and their answers, read as bench.py reads them, are checked against the
exact distribution of test_dice2problog.py.
"""
from pathlib import Path
import os
import re
import shlex
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
data = root / 'data'
sys.path.insert(0, str(scripts))
import source_tree  # noqa: E402,F401  (the source tree's scripts and tests)
sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench  # noqa: E402
import dice2problog  # noqa: E402
import dice2psi  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import JOINT, JOINT_MASSES  # noqa: E402

PSI = os.environ.get('PSI', '')
# Psi 's build (ef8a9e5) dumps core on murderMystery (a function that returns a
# tuple) and on diamond (a hundred calls of a function in a loop), which
# translate correctly; they are not run here.
SMALL = ['arith', 'compare', 'walk', 'twocoins', 'evidence1', 'evidence2', 'grass', 'alarm', 'noisyOr']


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def psi_answers(text, qs):
    """bench.py's reading of Psi's runs, one run a component."""
    _, types = dice2psi.translate(text, None)
    comps = []
    for k, t in enumerate(types):
        code, _ = dice2psi.translate(text, k)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.psi'
            path.write_text(code)
            out = subprocess.run(shlex.split(PSI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        dist = bench.parse_psi(out)
        comps.append({'true': dist.get(1, 0.0), 'false': dist.get(0, 0.0)} if t == dice2psi.BOOL else
                     {str(v): dist.get(v, 0.0) for v in range(2 ** t[1])})
    return bench.alea_answers(qs, comps)


class Dice2PsiTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = sorted((data / 'dice').glob('*.dice')) + sorted((data / 'nodice').glob('*.dice'))
        for path in paths:
            with self.subTest(name=path.name):
                try:
                    code, types = dice2psi.translate(path.read_text(), 0, path.name)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                self.assertEqual(code.count('{'), code.count('}'))
                self.assertIn('def main()', code)
                self.assertGreaterEqual(len(types), 1)

    def test_a_component_is_returned(self):
        text = 'let a = flip 0.1 in let b = discrete(0.5, 0.25, 0.25) in (a, (b, a && flip 0.5))'
        self.assertEqual(dice2psi.translate(text, None)[1], [dice2psi.BOOL, ('int', 2), dice2psi.BOOL])
        self.assertIn('return v2_b;', dice2psi.translate(text, 1)[0])
        self.assertIn('categorical([1/2, 1/4, 1/4])', dice2psi.translate(text, 1)[0])

    def test_types_are_checked(self):
        for program, error in [('int(2, 1) + int(3, 1)', TypeError), ('flip 0.5 + int(2, 1)', TypeError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError), ('fst flip 0.5', TypeError),
                               ('fun f(x) { x } f(true)', TypeError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    dice2psi.translate(program, 0)

    def test_recursion_is_refused(self):
        with self.assertRaises(NotImplementedError):
            dice2psi.translate(program_text('geometric'), 0)

    def test_psis_distribution_is_read(self):
        self.assertEqual(bench.parse_psi('p(x) = 3/10·δ(1)[x]+7/10·δ(0)[x]\n'), {1: 0.3, 0: 0.7})
        self.assertEqual(bench.parse_psi('p(r) = δ(2)[r]\n'), {2: 1.0})
        self.assertEqual(bench.parse_psi('p(r) = (1/2)·(1/4)·δ(0)[r]+7/8·δ(1)[r]\n'), {0: 0.125, 1: 0.875})
        self.assertIsNone(bench.parse_psi('Pr[error] = 1\n'))

    def test_a_joint_is_one_integer(self):
        # the Booleans are the integer's bits, the first the most significant
        code, types = dice2psi.translate(JOINT, 0, joint=True)
        self.assertEqual(types, [('int', 3)])
        self.assertRegex(code.splitlines()[-2], r'return \(.+\)\*4 \+ \(.+\)\*2 \+ \(.+\)\*1;')

    def test_psis_masses_cover_every_value(self):
        # a value Psi does not print has mass 0; no distribution, no masses
        self.assertEqual(bench.psi_masses('p(r) = 1/4·δ(3)[r]+3/4·δ(0)[r]\n', 4), {0: 0.75, 1: 0.0, 2: 0.0, 3: 0.25})
        self.assertEqual(bench.psi_masses('Pr[error] = 1\n', 2), {})

    def test_an_inexpressible_program_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'g.dice'
            path.write_text(program_text('geometric'))
            r = subprocess.run([sys.executable, str(scripts / 'dice2psi.py'), str(path), '0'], capture_output=True,
                               text=True)
        self.assertEqual(r.returncode, bench.REFUSED, r.stderr)

    @unittest.skipUnless(PSI, 'PSI is not set')
    def test_psi_gives_the_joint(self):
        code, _ = dice2psi.translate(JOINT, 0, joint=True)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.psi'
            path.write_text(code)
            out = subprocess.run(shlex.split(PSI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        masses = bench.psi_masses(out, 8)
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses[u], p, delta=1e-12, msg=u)

    @unittest.skipUnless(PSI, 'PSI is not set')
    def test_psi_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = psi_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    @unittest.skipUnless(PSI, 'PSI is not set')
    def test_integer_operations_wrap_as_dices(self):
        x = ', '.join(str(k / 36) for k in range(1, 9))
        y = ', '.join(str(k / 36) for k in range(8, 0, -1))
        for op in ['+', '-', '*', '==', '!=', '<', '<=', '>', '>=']:
            with self.subTest(op=op):
                text = f'let x = discrete({x}) in let y = discrete({y}) in (x, (y, x {op} y))'
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = psi_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)


if __name__ == '__main__':
    unittest.main()
