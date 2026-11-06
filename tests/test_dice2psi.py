"""Dice programs as Psi programs (scripts/dice2psi.py) and how bench.py reads
what Psi prints.

Every Dice program of data/dice and data/nodice (test_nodice.dice_programs) that dice2problog.py
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
from test_nodice import dice_programs  # noqa: E402
import dice2problog  # noqa: E402
import dice2psi  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import JOINT, JOINT_MASSES, check_each  # noqa: E402

PSI = os.environ.get('PSI', '')
SMALL = ['arith', 'compare', 'walk', 'twocoins', 'evidence1', 'evidence2', 'grass', 'alarm', 'noisyOr']
# functions of Boolean and tuple parameters, a constant Boolean argument and a
# loop over a tuple state: Psi stopped with signal 11 on them while their
# parameters had no type (Psi takes one for a real) and true was written 1
FUNCTIONS = ['murderMystery', 'counter']
# a tuple passed to a function in one branch and out of the other: each branch builds it (the
# weather factory's unrolled steps; the else branch read a tuple only the then branch had built,
# a variable Psi had not seen, and Psi stopped with signal 11)
BRANCH_TUPLE = '''fun swap(p: (bool, bool)) {
  (snd p, fst p)
}
let r = (flip 0.3, true) in
let s = if flip 0.5 then swap(r) else r in
fst s
'''


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
            path.write_text(code, encoding='utf-8')
            out = subprocess.run(shlex.split(PSI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        dist = bench.parse_psi(out)
        comps.append({'true': dist.get(1, 0.0), 'false': dist.get(0, 0.0)} if t == dice2psi.BOOL else
                     {str(v): dist.get(v, 0.0) for v in range(2 ** t[1])})
    return bench.alea_answers(qs, comps)


class Dice2PsiTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = dice_programs()
        for path in paths:
            with self.subTest(name=path.name):
                try:
                    code, types = dice2psi.translate(path.read_text(), 0, path.name)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                self.assertEqual(code.count('{'), code.count('}'))
                self.assertIn('def main()', code)
                self.assertGreaterEqual(len(types), 1)

    def test_each_component_from_one_translation(self):
        check_each(self, dice2psi, ['psi'])

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
        # --dp's numbers, an exponent's sign not a term's (caesar_small.psi's output)
        dist = bench.parse_psi('p(key) = 1.0579820436719924e-03·δ(14)[key]+2.5e+00·(1/5)·δ(21)[key]'
                               '+1.1959376439790302e-06·δ(11)[key]+1E-1·δ(3)[key]\n')
        self.assertEqual(sorted(dist), [3, 11, 14, 21])
        self.assertAlmostEqual(dist[14], 1.0579820436719924e-03, delta=1e-18)
        self.assertAlmostEqual(dist[21], 0.5, delta=1e-15)
        self.assertAlmostEqual(dist[11], 1.1959376439790302e-06, delta=1e-21)
        self.assertAlmostEqual(dist[3], 0.1, delta=1e-15)
        with self.assertRaises(ValueError):
            bench.parse_psi('p(x) = exp(-1)·δ(0)[x]\n')

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
            path.write_text(code, encoding='utf-8')
            out = subprocess.run(shlex.split(PSI) + [str(path)], capture_output=True, text=True, check=True,
                                 timeout=300).stdout
        masses = bench.psi_masses(out, 8)
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses[u], p, delta=1e-12, msg=u)

    def test_parameters_have_the_signatures_types(self):
        # counter's step takes (int(3), bool), murderMystery's gunFoundAtScene a
        # bool and is called with true
        self.assertRegex(dice2psi.translate(program_text('counter'), 0)[0], r'def f_step\(v\d+_s: ℤ × 𝔹\) \{')
        code = dice2psi.translate(program_text('murderMystery'), 0)[0]
        self.assertRegex(code, r'def f_mystery\(\) \{')
        self.assertRegex(code, r'def f_gunfoundatscene\(v\d+_gunfound: 𝔹\) \{')
        self.assertRegex(code, r':= f_gunfoundatscene\(true\);')
        self.assertEqual(dice2psi.psi_type(('tuple', dice2psi.BOOL, ('tuple', ('int', 2), dice2psi.BOOL))),
                         '𝔹 × (ℤ × 𝔹)')

    def test_booleans_are_true_and_false(self):
        code = dice2psi.translate('let a = flip 0.5 in let b = if a then true else flip 0.2 in (b, flip 0.0)', 0)[0]
        self.assertIn('= true;', code)
        self.assertIn(':= false;', code)          # the variable both branches set
        self.assertNotRegex(code, r'= [01];')
        self.assertIn('return false;', dice2psi.translate('let a = flip 0.5 in (a, flip 0.0)', 1)[0])

    def test_each_branch_builds_the_tuples_it_passes_on(self):
        code = dice2psi.translate(BRANCH_TUPLE, 0)[0]
        built = re.findall(r'(v\d+_r) := \(v\d+_r, true\);', code)
        self.assertEqual(len(built), 2, code)
        self.assertEqual(built[0], built[1])
        then, other = re.search(r'if v\d+_s \{(.*)\} else \{(.*)\}', code).groups()
        self.assertIn(f'{built[0]} := (', then)
        self.assertIn(f'{built[0]} := (', other)

    @unittest.skipUnless(PSI, 'PSI is not set')
    def test_psi_gives_the_exact_answers(self):
        for name in SMALL + FUNCTIONS + ['BRANCH_TUPLE']:
            text = BRANCH_TUPLE if name == 'BRANCH_TUPLE' else program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = psi_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    def test_nth_bit_is_a_shift_and_a_comparison(self):
        _, types = dice2psi.translate(reference.NTH_BIT, None)
        self.assertEqual(types, [dice2psi.BOOL] * 3)
        # bit 1 of an int(3): x * 2 modulo 8, at least 4
        self.assertIn('(((v1_x * 2) - 8 * floor(v1_x * 2 / 8)) >= 4)', dice2psi.translate(reference.NTH_BIT, 1)[0])
        with self.assertRaises(TypeError):
            dice2psi.translate('nth_bit(int(4, 0), int(3, 1))', 0)

    def test_a_long_chain_translates_without_recursing_down_it(self):
        # each operation of the chain is a variable of its own, as before
        for k, op in enumerate(['&&', '||']):
            code = reference.shallow(lambda: dice2psi.translate(reference.CHAINS, k)[0])
            self.assertEqual(max(line.count(op) for line in code.splitlines()), 1)
            self.assertGreaterEqual(code.count(op), reference.LONG - 1)

    @unittest.skipUnless(PSI, 'PSI is not set')
    def test_psi_gives_nth_bit(self):
        qs = reference.queries(dice2problog.translate(reference.NTH_BIT_AS_SETS))
        got = psi_answers(reference.NTH_BIT, qs)
        for q, w in zip(qs, reference.NTH_BIT_ANSWERS):
            self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)

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
