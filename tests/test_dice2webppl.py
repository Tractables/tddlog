"""Dice programs as WebPPL programs (scripts/dice2webppl.py, run by
scripts/webppl_dice.js) and how bench.py reads what webppl_dice.js prints.

Every Dice program of data/dice and data/nodice (test_nodice.dice_programs)
that dice2problog.py translates translates here, except where it divides.
Where WebPPL is installed (WEBPPL, the command that runs a node script with
WebPPL on its NODE_PATH, e.g. `env NODE_PATH=$HOME/local/webppl/node_modules
$HOME/local/webppl/bin/node`), the translated programs run, and their
answers are checked against the exact distribution of test_dice2problog.py.
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
import dice2webppl  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import CAESAR, JOINT, JOINT_MASSES, OPS, SMALL, X, Y, program_text  # noqa: E402

WEBPPL = os.environ.get('WEBPPL', '')
# a branch that observes, a function that observes called as the second
# operand of && and of ||, and a discrete with a value of no mass
OBSERVING = '''fun seen(a: bool): bool {
  let o = observe a in true
}
let x = flip 0.4 in
let y = flip 0.7 in
let z = if x then (let o = observe y in flip 0.2) else flip 0.9 in
let u = discrete(0.5, 0.0, 0.5) in
let w = (x && seen(y)) || (!x || seen(u == int(2, 0))) in
(z, (u, w))'''
# a recursion of three calls, within Dice's limit
RECURSION = (data / 'dice' / 'recursion.dice').read_text()


def run(code):
    """webppl_dice.js's output on the WebPPL program code."""
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / 'p.wppl'
        path.write_text(code)
        return subprocess.run(shlex.split(WEBPPL) + [str(scripts / 'webppl_dice.js'), str(path)],
                              capture_output=True, text=True, check=True, timeout=600).stdout


def components(out):
    """bench.py's reading of webppl_dice.js's run (webppl_masses)."""
    comps = {}
    for line in out.splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == 'webppl_c':
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
    return [comps[k] for k in sorted(comps)]


class Dice2WebpplTests(unittest.TestCase):
    def assert_exact(self, text):
        qs = reference.queries(dice2problog.translate(text))
        want = reference.answers(reference.exact(text), qs)
        got = bench.alea_answers(qs, components(run(dice2webppl.translate(text))))
        for q, w in zip(qs, want):
            self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)

    def test_the_baseline_programs_translate(self):
        for path in dice_programs():
            with self.subTest(name=path.name):
                try:
                    code = dice2webppl.translate(path.read_text(), path.name)
                except NotImplementedError:
                    continue   # lists, / and %
                self.assertIn('\n// component 0 ', code)
                self.assertTrue(code.endswith("\nInfer({method: 'enumerate'}, model);\n"))
                self.assertEqual(code.count('Infer('), 1)

    def test_the_components_are_declared_in_order(self):
        code = dice2webppl.translate('let a = flip 0.5 in let n = discrete(0.2, 0.3, 0.5) in (a, (n, (n == int(2, 1), a)))')
        self.assertEqual(re.findall(r'^// component (\d+) (.*)$', code, re.M),
                         [('0', 'bool'), ('1', 'int 2'), ('2', 'bool'), ('3', 'bool')])

    def test_an_observing_branch_is_a_statement(self):
        code = dice2webppl.translate(OBSERVING)
        body = code[code.index('var model'):]
        self.assertIn('(function() {', body)
        self.assertRegex(body, r'if \(x\) \{\n\s+condition\(y\);')
        # the second operands that observe are bound before the && and || that read them
        self.assertEqual(len(re.findall(r'var \w+ = seen\(', body)), 2)
        self.assertIn('categorical({ps: [0.5, 0.5], vs: [0, 2]})', body)

    def test_an_iterate_is_a_recursive_function(self):
        code = dice2webppl.translate(CAESAR)
        self.assertIn('var iterate = function(f, x, k) {', code)
        self.assertNotIn('for (', code)

    def test_a_joint_is_one_int(self):
        code = dice2webppl.translate(JOINT, joint=True)
        self.assertEqual(re.findall(r'^// component (\d+) (.*)$', code, re.M), [('0', 'int 3')])

    def test_what_webppl_cannot_express_is_refused(self):
        for text in ['let x = discrete(0.5, 0.5) in x / int(1, 1)', 'let x = discrete(0.5, 0.5) in x % int(1, 1)',
                     'int(60, 3)', 'let x = uniform(30, 0, 1000) in x * x', 'let x = uniform(40, 0, 100) in x << 20']:
            with self.subTest(program=text):
                with self.assertRaises(NotImplementedError):
                    dice2webppl.translate(text)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.dice'
            path.write_text('int(60, 3)')
            r = subprocess.run([sys.executable, str(scripts / 'dice2webppl.py'), str(path)], capture_output=True, text=True)
        self.assertEqual(r.returncode, bench.REFUSED)

    def test_nth_bit_counts_from_the_most_significant_bit(self):
        code = dice2webppl.translate(reference.NTH_BIT)
        # bit 0 of an int(3) is the quotient by 4, bit 2 the number itself, modulo 2
        self.assertRegex(code, r'Math\.floor\(x / 4\) % 2 == 1')
        self.assertRegex(code, r'Math\.floor\(x / 2\) % 2 == 1')
        self.assertRegex(code, r'[^(]x % 2 == 1')
        with self.assertRaises(TypeError):
            dice2webppl.translate('nth_bit(int(4, 0), int(3, 1))')
        with self.assertRaisesRegex(ValueError, 'no bit 3 of 3'):
            dice2webppl.translate('let x = int(3, 1) in nth_bit(int(3, 3), x)')

    def test_a_long_chain_is_one_call_over_its_operands(self):
        code = reference.shallow(lambda: dice2webppl.translate(reference.CHAINS))
        calls = re.findall(r'\b(all|any)\(function\(x\) \{ return x; \}, \[([ab](?:, [ab])*)\]\)', code)
        self.assertEqual([f for f, _ in calls], ['all', 'any'])
        self.assertEqual([len(xs.split(', ')) for _, xs in calls], [reference.LONG] * 2)
        self.assertNotRegex(code, r'&&|\|\|')
        # A chain of CHAIN operands or fewer is written as it was.
        chain = ' && '.join('ab'[k % 2] for k in range(dice2problog.CHAIN))
        self.assertIn(chain, dice2webppl.translate(f'let a = flip 0.3 in let b = flip 0.6 in {chain}'))

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_a_long_chain_evaluates_every_operand(self):
        # seen(y) observes y whatever x is, as Dice evaluates both operands:
        # P(x) stays 0.4, where JavaScript's && would make it 0.28 / 0.88
        chain = ' && '.join(['x', 'seen(y)'] + ['x'] * dice2problog.CHAIN)
        text = OBSERVING.split('let x =')[0] + f'let x = flip 0.4 in\nlet y = flip 0.7 in\n(x, {chain})'
        comps = components(run(reference.shallow(lambda: dice2webppl.translate(text))))
        self.assertEqual(len(comps), 2)
        for comp in comps:
            self.assertAlmostEqual(comp['true'], 0.4, delta=1e-12)

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_webppl_gives_nth_bit(self):
        qs = reference.queries(dice2problog.translate(reference.NTH_BIT_AS_SETS))
        got = bench.alea_answers(qs, components(run(dice2webppl.translate(reference.NTH_BIT))))
        for q, w in zip(qs, reference.NTH_BIT_ANSWERS):
            self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_a_long_chain_gives_the_exact_answers(self):
        comps = components(run(dice2webppl.translate(reference.CHAINS)))
        for comp, want in zip(comps, reference.CHAINS_ANSWERS):
            self.assertAlmostEqual(comp['true'], want, delta=1e-12)

    def test_dice_names_are_javascript_names(self):
        code = dice2webppl.translate("let var = flip 0.5 in let x' = flip 0.5 in let _y = var && x' in (_y, x')")
        self.assertNotRegex(code, r"\bvar var\b|x'|var _y\b")

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_webppl_gives_the_exact_answers(self):
        # the Dice paper's diamond and ladder have too many paths to enumerate (no run of either
        # ended within 600 s): their sizes are the runs' to time, not the test's
        for text in [program_text(name) for name in SMALL if name not in ('diamond', 'ladder')] + \
                [OBSERVING, RECURSION, CAESAR]:
            with self.subTest(program=text[:40]):
                self.assert_exact(text)

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_webppl_gives_the_joint(self):
        comps = components(run(dice2webppl.translate(JOINT, joint=True)))
        self.assertEqual(len(comps), 1)
        self.assertEqual(sorted(comps[0], key=int), [str(u) for u in range(8)])
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(comps[0][str(u)], p, delta=1e-12, msg=u)

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_integer_operations_wrap_as_dices(self):
        programs = [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
                   [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
                    'let x = uniform(3, 2, 7) in (x, x + int(3, 4))']
        for text in programs:
            with self.subTest(program=text[-24:]):
                self.assert_exact(text)

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_the_driver_prints_its_times(self):
        out = run(dice2webppl.translate('let a = flip 0.5 in a'))
        times = {f[0]: float(f[1]) for f in map(str.split, out.splitlines()) if f[0].endswith('_seconds')}
        self.assertEqual(sorted(times), ['webppl_compile_seconds', 'webppl_header_seconds', 'webppl_seconds'])
        self.assertLessEqual(times['webppl_compile_seconds'], times['webppl_seconds'])

    @unittest.skipUnless(WEBPPL, 'WEBPPL is not set')
    def test_an_impossible_observation_fails(self):
        with self.assertRaises(subprocess.CalledProcessError):
            run(dice2webppl.translate('let a = flip 0.5 in let o = observe (a && !a) in a'))


if __name__ == '__main__':
    unittest.main()
