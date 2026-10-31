"""Dice programs as Pluck programs (scripts/dice2pluck.py) and how bench.py
reads what Pluck prints.

Every Dice program of data/dice and data/nodice (test_nodice.dice_programs) that dice2problog.py
translates translates here, except where it has recursion, with its
parentheses balanced and a query for each component. Where Pluck is
installed (PLUCK, bench.py's `--pluck` template, e.g. `env
JULIA_DEPOT_PATH=P/depot JULIA --project=P/src/PluckArtifact.jl/Pluck.jl
{file} P/src`), the translated programs run through scripts/pluck_dice.jl,
and their answers, read as bench.py reads them, are checked against the
exact distribution of test_dice2problog.py: the small programs, loops with
observations in their body (diamond, ladder, caesar), and every integer
operation, whose bits Pluck computes.
"""
from pathlib import Path
import os
import re
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
import dice2pluck  # noqa: E402
import dice2problog  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import CAESAR, JOINT, JOINT_MASSES, OPS, SMALL, X, Y  # noqa: E402

PLUCK = os.environ.get('PLUCK', '')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def run_pluck(code):
    """What pluck_dice.jl prints for a program."""
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / 'p.pluck'
        path.write_text(code)
        cmd = PLUCK.format(file=str(scripts / 'pluck_dice.jl')).split() + [str(path)]
        return subprocess.run(cmd, capture_output=True, text=True, check=True, timeout=600).stdout


def pluck_answers(text, qs):
    """bench.py's reading of Pluck's run, one run for all the components."""
    code, types = dice2pluck.translate(text)
    dists = bench.parse_pluck(run_pluck(code))
    comps = []
    for k, t in enumerate(types):
        got = dists[k]
        comps.append({'true': got.get(1, 0.0), 'false': got.get(0, 0.0)} if t == dice2pluck.BOOL else
                     {str(v): got.get(v, 0.0) for v in range(2 ** t[1])})
    return bench.alea_answers(qs, comps)


class Dice2PluckTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = dice_programs()
        for path in paths:
            with self.subTest(name=path.name):
                try:
                    code, types = dice2pluck.translate(path.read_text(), path.name)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                body = '\n'.join(line for line in code.splitlines() if not line.startswith(';;'))
                self.assertEqual(body.count('('), body.count(')'))
                self.assertIn('(define (dice_model)', code)
                self.assertEqual(len(re.findall(r'^\(query c\d+ ', code, re.M)), len(types))

    def test_components_and_their_bits(self):
        text = 'let a = flip 0.1 in let b = discrete(0.5, 0.25, 0.25) in (a, (b, a && flip 0.5))'
        code, types = dice2pluck.translate(text)
        self.assertEqual(types, [dice2pluck.BOOL, ('int', 2), dice2pluck.BOOL])
        # the int is the list of its two bits: a flip draws the high bit, and the low
        # one is 1 only where the high one is 0 (the masses 0.5, 0.25, 0.25, 0)
        self.assertIn('(Cons ', code)
        self.assertEqual(code.count('(flip '), 1 + 2 + 1)
        self.assertEqual(code.count('(Marginal '), 3)

    def test_observations_condition_every_query(self):
        code, _ = dice2pluck.translate(JOINT)
        self.assertEqual(code.count('(Posterior '), 3)
        self.assertNotIn('(Marginal ', code)

    def test_a_probability_has_no_exponent(self):
        code, _ = dice2pluck.translate('flip 0.00001')
        self.assertIn('(flip 0.00001)', code)

    def test_a_loop_is_a_recursion(self):
        code, _ = dice2pluck.translate(program_text('diamond'))
        self.assertIn('(Y (lambda ', code)

    def test_types_are_checked(self):
        for program, error in [('int(2, 1) + int(3, 1)', TypeError), ('flip 0.5 + int(2, 1)', TypeError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError), ('fst flip 0.5', TypeError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    dice2pluck.translate(program)

    def test_a_joint_is_one_integer(self):
        # the Booleans are the integer's bits, the first the most significant
        code, types = dice2pluck.translate(JOINT, joint=True)
        self.assertEqual(types, [('int', 3)])
        self.assertEqual(code.count('(query '), 1)
        self.assertEqual(code.count('(Cons '), 3)

    def test_plucks_distributions_are_read(self):
        out = ('c0:\n  false  0.7\n  true   0.3\n\nc1:\n  Any[true, false]  0.25\n  Any[false, true]  0.5\n'
               '  Any[false, true]  0.25\n\nc2:\n  impossible constraint\npluck_seconds 1.5\n')
        self.assertEqual(bench.parse_pluck(out), {0: {0: 0.7, 1: 0.3}, 1: {2: 0.25, 1: 0.75}, 2: {}})
        self.assertEqual(bench.parse_pluck('c0:\n  Bool[1, 1]  1.0\n'), {0: {3: 1.0}})

    def test_an_inexpressible_program_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'g.dice'
            path.write_text(program_text('geometric'))
            r = subprocess.run([sys.executable, str(scripts / 'dice2pluck.py'), str(path)], capture_output=True,
                               text=True)
        self.assertEqual(r.returncode, bench.REFUSED, r.stderr)

    @unittest.skipUnless(PLUCK, 'PLUCK is not set')
    def test_pluck_gives_the_joint(self):
        code, _ = dice2pluck.translate(JOINT, joint=True)
        masses = bench.parse_pluck(run_pluck(code))[0]
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses.get(u, 0.0), p, delta=1e-12, msg=u)

    @unittest.skipUnless(PLUCK, 'PLUCK is not set')
    def test_pluck_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = pluck_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    @unittest.skipUnless(PLUCK, 'PLUCK is not set')
    def test_integer_operations_wrap_as_dices(self):
        programs = [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
                   [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
                    f'let x = discrete({X}) in (x, x * int(3, 3))', CAESAR]
        for text in programs:
            with self.subTest(program=text[-24:]):
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = pluck_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)


if __name__ == '__main__':
    unittest.main()
