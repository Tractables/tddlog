"""Dice programs as Geni programs (scripts/dice2geni.py, over
scripts/dice_ir.py) and how bench.py reads what Geni prints.

Every Dice program of data/dice and data/nodice (test_nodice.dice_programs) that dice2problog.py
translates translates here, except where it has recursion, with a program
for each component of its value. Where Geni is installed (GENI, the command
that runs it, e.g. `$HOME/local/geni/bin/gennifer -l -i`), the translated
programs run, and their answers, read as bench.py reads them, are checked
against the exact distribution of test_dice2problog.py, to Geni's f32.
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
from test_nodice import dice_programs  # noqa: E402
import dice2geni  # noqa: E402
import dice2problog  # noqa: E402
import dice_ir  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import CAESAR, JOINT, JOINT_MASSES, OPS, SMALL, X, Y  # noqa: E402

GENI = os.environ.get('GENI', '')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def geni_answers(text, qs):
    """bench.py's reading of Geni's runs, one run a component."""
    _, types = dice2geni.translate(text, None)
    comps = []
    for k, t in enumerate(types):
        code, _ = dice2geni.translate(text, k)
        n = 2 if t == dice_ir.BOOL else 2 ** t[1]
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.gir'
            path.write_text(code)
            out = subprocess.run(shlex.split(GENI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        masses = bench.geni_masses(out)
        assert sorted(masses) == list(range(n)), out[-300:]
        comps.append({'true': masses[1], 'false': masses[0]} if t == dice_ir.BOOL else
                     {str(v): masses[v] for v in range(n)})
    return bench.alea_answers(qs, comps)


class Dice2GeniTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = dice_programs()
        for path in paths:
            with self.subTest(name=path.name):
                text = path.read_text()
                try:
                    _, types = dice2geni.translate(text, None)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                for k in range(len(types)):
                    code, _ = dice2geni.translate(text, k, path.name)
                    self.assertEqual(code.count('('), code.count(')'))
                    self.assertNotIn('None', code)
                    self.assertNotRegex(code.splitlines()[-1], r'^let ')

    def test_an_integer_is_its_bits(self):
        # the most significant bit first, then the next given it
        code, types = dice2geni.translate('discrete(0.1, 0.2, 0.3, 0.4)', 0)
        self.assertEqual(types, [('int', 2)])
        self.assertRegex(code, r'let g1 = sample\(Bernoulli\(0\.(7|69)\d*\)\) in')
        self.assertRegex(code, r'let g2 = if g1 then \(sample\(Bernoulli\(0\.57\d*\)\)\) else '
                         r'\(sample\(Bernoulli\(0\.66\d*\)\)\) in')
        self.assertEqual(code.splitlines()[-1], '(g1, g2)')

    def test_a_bit_the_values_fix_is_a_constant(self):
        code, _ = dice2geni.translate('discrete(0.5, 0.5, 0.0, 0.0)', 0)
        self.assertEqual(code.splitlines()[-1], '(false, g1)')
        code, _ = dice2geni.translate('let x = discrete(0.5, 0.5) in let y = discrete(0.5, 0.5) in '
                                      'int(2, 0) + int(2, 1)', 0)
        self.assertEqual(code.splitlines()[-1], '(false, true)')

    def test_a_conditional_returns_the_bits_that_differ(self):
        code, _ = dice2geni.translate('let a = flip 0.5 in if a then (flip 0.2, flip 0.3) else (flip 0.4, a)', 1)
        self.assertRegex(code, r'let g\d+ = if g1 then \(let g\d+ = sample\(Bernoulli\(0\.3\)\) in g\d+\) '
                         r'else \(g1\) in')

    def test_types_are_checked(self):
        for program, error in [('int(2, 1) + int(3, 1)', TypeError), ('flip 0.5 + int(2, 1)', TypeError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError), ('fst flip 0.5', TypeError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    dice2geni.translate(program, 0)

    def test_recursion_is_refused(self):
        with self.assertRaises(NotImplementedError):
            dice2geni.translate(program_text('geometric'), 0)
        r = subprocess.run([sys.executable, str(scripts / 'dice2geni.py'), str(data / 'dice' / 'geometric.dice'),
                            '--components'], capture_output=True, text=True)
        self.assertEqual(r.returncode, bench.REFUSED)
        self.assertIn('recursion', r.stderr)

    def test_a_joint_is_one_integer(self):
        # the Booleans are the integer's bits, without a gate
        code, types = dice2geni.translate(JOINT, 0, joint=True)
        self.assertEqual(types, [('int', 3)])
        self.assertEqual(code.splitlines()[-1], '(g1, (g2, g3))')

    def test_genis_masses_are_read(self):
        # a tuple of bits, the most significant first; the masses normalized
        out = 'Pr((0, (0, 1))) = 0.1000000\nPr((1, (1, 0))) = 0.3000000\n'
        self.assertEqual({k: round(v, 9) for k, v in bench.geni_masses(out).items()}, {1: 0.25, 6: 0.75})
        self.assertEqual({k: round(v, 9) for k, v in bench.geni_masses('Pr(0) = 0.2\nPr(1) = 0.6\n').items()},
                         {0: 0.25, 1: 0.75})
        # observations less probable than seven decimals: bench.py reports them
        self.assertEqual(bench.geni_masses('Pr(0) = 0.0000000\nPr(1) = 0.0000000\n'), {0: 0.0, 1: 0.0})

    @unittest.skipUnless(GENI, 'GENI is not set')
    def test_geni_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = geni_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-5, msg=q)

    @unittest.skipUnless(GENI, 'GENI is not set')
    def test_geni_gives_the_joint(self):
        code, _ = dice2geni.translate(JOINT, 0, joint=True)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.gir'
            path.write_text(code)
            out = subprocess.run(shlex.split(GENI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        masses = bench.geni_masses(out)
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses[u], p, delta=1e-5, msg=u)

    @unittest.skipUnless(GENI, 'GENI is not set')
    def test_integer_operations_wrap_as_dices(self):
        programs = [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
                   [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
                    f'let x = discrete({X}) in (x, x * int(3, 3))', CAESAR]
        for text in programs:
            with self.subTest(program=text[-24:]):
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = geni_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-5, msg=q)


if __name__ == '__main__':
    unittest.main()
