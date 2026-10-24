"""Dice programs as Genfer programs (scripts/dice2genfer.py, over
scripts/dice_ir.py) and how bench.py reads what Genfer prints.

Every Dice program of data/dice and data/nodice that dice2problog.py
translates translates here, except where it has recursion, with a program
for each component of its value. Where Genfer is installed (GENFER, the
command that runs it, e.g. `$HOME/local/genfer/bin/genfer`), the
translated programs run, and their answers, read as bench.py reads them,
are checked against the exact distribution of test_dice2problog.py.
"""
from pathlib import Path
import math
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
import dice2genfer  # noqa: E402
import dice2problog  # noqa: E402
import dice_ir  # noqa: E402
import test_dice2problog as reference  # noqa: E402

GENFER = os.environ.get('GENFER', '')
SMALL = ['arith', 'compare', 'walk', 'twocoins', 'evidence1', 'evidence2', 'grass', 'alarm', 'noisyOr',
         'murderMystery', 'nested', 'counter', 'diamond', 'ladder']
# integer operations, each on two discrete integers of 3 bits, the third
# component its result
X = ', '.join(str(k / 36) for k in range(1, 9))
Y = ', '.join(str(k / 36) for k in range(8, 0, -1))
OPS = ['+', '-', '*', '==', '!=', '<', '<=', '>', '>=']
# three Booleans asked jointly (--joint, as a circuit's `joint` program),
# the first the most significant, and their distribution given a || b
JOINT = ('let a = flip 0.1 in let b = flip 0.2 in let c = flip 0.3 in let o = observe (a || b) in '
         '(a, (b, c))')
JOINT_MASSES = {4 * a + 2 * b + c: (0.1 if a else 0.9) * (0.2 if b else 0.8) * (0.3 if c else 0.7) / 0.28
                * (a or b) for a in (0, 1) for b in (0, 1) for c in (0, 1)}
# the cipher of caesar-iter.dice over three characters: the observed
# character is carried through the loop, a variable with one value
CAESAR = (data / 'dice' / 'caesar-iter.dice').read_text().replace(', 3000)', ', 3)')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def genfer_answers(text, qs):
    """bench.py's reading of Genfer's runs, one run a component."""
    _, types = dice2genfer.translate(text, None)
    comps = []
    for k, t in enumerate(types):
        code, _ = dice2genfer.translate(text, k)
        n = 2 if t == dice_ir.BOOL else 2 ** t[1]
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.sgcl'
            path.write_text(code)
            out = subprocess.run(shlex.split(GENFER) + [str(path), '--limit', str(n)], capture_output=True,
                                 text=True, check=True, timeout=300).stdout
        _, masses = bench.parse_genfer(out)
        comps.append({'true': masses[1], 'false': masses[0]} if t == dice_ir.BOOL else
                     {str(v): masses[v] for v in range(n)})
    return bench.alea_answers(qs, comps)


class Dice2GenferTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = sorted((data / 'dice').glob('*.dice')) + sorted((data / 'nodice').glob('*.dice'))
        for path in paths:
            with self.subTest(name=path.name):
                text = path.read_text()
                try:
                    _, types = dice2genfer.translate(text, None)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                for k in range(len(types)):
                    code, _ = dice2genfer.translate(text, k, path.name)
                    self.assertEqual(code.count('{'), code.count('}'))
                    self.assertRegex(code, r'\nreturn [a-z0-9]+\n$')

    def test_registers_are_reused(self):
        # a chain of a hundred flips needs a few variables, not a hundred
        text = 'let a0 = flip 0.5 in ' + ''.join(f'let a{i} = a{i - 1} ^ flip 0.5 in ' for i in range(1, 100)) + 'a99'
        code, _ = dice2genfer.translate(text, 0)
        self.assertLessEqual(len(set(re.findall(r'\bx\d+\b', code))), 3)

    def test_a_component_needs_only_its_statements(self):
        text = 'let a = flip 0.1 in let b = discrete(0.5, 0.25, 0.25) in (a, b)'
        self.assertEqual(dice2genfer.translate(text, None)[1], [dice_ir.BOOL, ('int', 2)])
        code, _ = dice2genfer.translate(text, 1)
        self.assertIn('~ Categorical(0.5, 0.25, 0.25);', code)
        self.assertNotIn('Bernoulli', code)

    def test_an_event_read_once_is_inlined(self):
        code, _ = dice2genfer.translate('let a = flip 0.1 in let b = flip 0.2 in let c = observe (a || !b) in a', 0)
        self.assertRegex(code, r'observe \(x\d+ = 1\) \|\| \(!\(x\d+ = 1\)\);')

    def test_types_are_checked(self):
        for program, error in [('int(2, 1) + int(3, 1)', TypeError), ('flip 0.5 + int(2, 1)', TypeError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError), ('fst flip 0.5', TypeError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    dice2genfer.translate(program, 0)

    def test_recursion_is_refused(self):
        with self.assertRaises(NotImplementedError):
            dice2genfer.translate(program_text('geometric'), 0)
        r = subprocess.run([sys.executable, str(scripts / 'dice2genfer.py'), str(data / 'dice' / 'geometric.dice'),
                            '--components'], capture_output=True, text=True)
        self.assertEqual(r.returncode, bench.REFUSED)
        self.assertIn('recursion', r.stderr)

    def test_a_joint_is_one_integer(self):
        self.assertEqual(dice2genfer.translate(JOINT, None, joint=True)[1], [('int', 3)])
        with self.assertRaises(TypeError):
            dice2genfer.translate('(flip 0.5, int(2, 1))', None, joint=True)

    def test_a_probability_has_at_most_19_decimals(self):
        # Genfer reads a decimal's digits into a 64-bit integer
        self.assertEqual(dice_ir.decimal(0.25), '0.25')
        d = dice_ir.decimal(1.2345678901234567e-05)
        self.assertEqual(len(d.split('.')[1]), 19)
        self.assertAlmostEqual(float(d), 1.2345678901234567e-05, delta=1e-19)

    def test_genfers_masses_are_read(self):
        normalized = 'Normalized:   p(0) / Z = 0.25\nUnnormalized: p(1)     = 0.5\nNormalized:   p(1) / Z = 0.75\n'
        self.assertEqual(bench.parse_genfer(normalized)[1], {0: 0.25, 1: 0.75})
        self.assertEqual(bench.parse_genfer('p(0) = 0.25\np(1) = 3/4\n')[1], {0: 0.25, 1: 0.75})
        nan = bench.parse_genfer('p(0) = NaN\np(1) = NaN\n')[1]
        self.assertTrue(all(math.isnan(p) for p in nan.values()) and sorted(nan) == [0, 1])

    @unittest.skipUnless(GENFER, 'GENFER is not set')
    def test_genfer_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = genfer_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    @unittest.skipUnless(GENFER, 'GENFER is not set')
    def test_genfer_gives_the_joint(self):
        code, _ = dice2genfer.translate(JOINT, 0, joint=True)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.sgcl'
            path.write_text(code)
            out = subprocess.run(shlex.split(GENFER) + [str(path), '--limit', '8'], capture_output=True, text=True,
                                 check=True, timeout=300).stdout
        masses = bench.parse_genfer(out)[1]
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses[u], p, delta=1e-12, msg=u)

    @unittest.skipUnless(GENFER, 'GENFER is not set')
    def test_integer_operations_wrap_as_dices(self):
        programs = [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
                   [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
                    f'let x = discrete({X}) in (x, x * int(3, 3))', CAESAR]
        for text in programs:
            with self.subTest(program=text[-24:]):
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = genfer_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)


if __name__ == '__main__':
    unittest.main()
