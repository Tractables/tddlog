"""Dice programs as Roulette programs (scripts/dice2roulette.py) and how
bench.py reads what they print.

Every Dice program of data/dice, data/nodice and the networks gen_bn.py
writes translates, with its parentheses balanced, its flips in the
program's order, and every function it calls defined before the call.
Where Roulette is installed (ROULETTE_RACKET, the command that runs
Racket with Roulette, e.g. `env PLTUSERHOME=... racket`), the translated
programs run, and their answers, read as bench.py reads them, are checked
against the exact distribution of test_dice2problog.py and Dice's answers.
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
from source_tree import SOURCE  # noqa: E402
sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench  # noqa: E402
import dice2problog  # noqa: E402
import dice2roulette  # noqa: E402
import test_dice2problog as reference  # noqa: E402

RACKET = os.environ.get('ROULETTE_RACKET', '')
# The programs whose exact distribution the enumeration computes in a
# second; their answers are also Dice's (test_dice2problog.DICE) or by hand.
SMALL = ['arith', 'compare', 'walk', 'geometric', 'counter', 'recursion', 'caesar-recur', 'nested', 'twocoins',
         'evidence1', 'evidence2', 'grass', 'alarm', 'noisyOr', 'murderMystery', 'diamond', 'ladder']


def translate(text):
    return dice2roulette.translate(text, 'test')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def balanced(code):
    """Whether the parentheses and brackets of Racket code nest, strings
    and comments left out."""
    code = re.sub(r'"(\\.|[^"\\])*"', '""', code)
    code = re.sub(r';[^\n]*', '', code)
    stack = []
    for ch in code:
        if ch in '([':
            stack.append(ch)
        elif ch in ')]':
            if not stack or {'(': ')', '[': ']'}[stack.pop()] != ch:
                return False
    return not stack


def run(code, *args):
    """What Racket prints running the program `code`."""
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / 'p.rkt'
        path.write_text(code)
        return subprocess.run(shlex.split(RACKET) + [str(path), *args], capture_output=True, text=True,
                              check=True, timeout=300).stdout


class Dice2RouletteTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = sorted((data / 'dice').glob('*.dice')) + sorted((data / 'nodice').glob('*.dice'))
        for path in paths:
            with self.subTest(name=path.name):
                try:
                    code = dice2roulette.translate(path.read_text(), path.name)
                except NotImplementedError:
                    continue   # lists, /, % and the like, which dice2problog.py refuses too
                self.assertTrue(balanced(code))
                self.assertIn('(define pmfs', code)

    def test_a_network_translates(self):
        text = subprocess.run([sys.executable, str(SOURCE / 'scripts' / 'gen_bn.py'), str(SOURCE / 'examples' / 'asia.net'),
                               '--query', 'leaves', '--format', 'dice'], capture_output=True, text=True,
                              check=True).stdout
        code = translate(text)
        self.assertTrue(balanced(code))

    def test_flips_come_in_the_programs_order(self):
        code = translate('let a = flip 0.1 in let b = if a then flip 0.2 else flip 0.3 in '
                         'let c = discrete(0.5, 0.25, 0.25) in (b, c)')
        flips = re.findall(r'\(flip ([0-9.e-]+)\)', code.split('(define clock (start-clock))')[1])
        self.assertEqual(flips, ['0.1', '0.2', '0.3'])
        self.assertIn('(discrete 2 (list 0.5 0.25 0.25))', code)

    def test_a_recursive_function_takes_its_depth(self):
        code = translate(program_text('geometric'))
        self.assertIn(f'[(> depth {dice2problog.RECURSION_LIMIT}) (observe! #f)', code)
        self.assertRegex(code, r'\(f-geom \(add1 depth\) ')
        self.assertRegex(code, r'\(f-geom 0 ')

    def test_types_are_checked(self):
        for program, error in [('int(2, 1) + int(3, 1)', TypeError), ('flip 0.5 + int(2, 1)', TypeError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError), ('fst flip 0.5', TypeError),
                               ('fun f(x) { x } f(true)', TypeError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    translate(program)

    def test_components_answer_the_queries_in_order(self):
        # A translated program prints each component's distribution, which
        # bench.py reads as it reads Alea's.
        out = 'roulette 1 true 0.25\nroulette 1 false 0.75\nroulette 2 0 0.5\nroulette 2 1 0.0\n' \
              'roulette 2 2 0.5\nroulette 2 3 0.0\ncpu time: 3 real time: 4 gc time: 0\n'
        comps, _ = bench.parse_alea(out, 'roulette')
        self.assertEqual(bench.alea_answers(['result_1', 'c(0)', 'c(1)', 'c(2)'], comps),
                         {'result_1': 0.25, 'c(0)': 0.5, 'c(1)': 0.0, 'c(2)': 0.5})

    JOINT = 'let x = flip 0.5 in let y = flip 0.25 in let o = observe (x || y) in (x, y)\n'

    def test_joint_asks_the_booleans_as_one_int(self):
        code = dice2roulette.translate(self.JOINT, 'test', joint=True)
        self.assertTrue(balanced(code))
        self.assertEqual(code.count('(report-int 1 (list-ref pmfs 0) 2)'), 1)
        self.assertNotIn('report-bool', code.split('(define pmfs')[1])
        with self.assertRaisesRegex(NotImplementedError, r'--joint asks a value of Booleans.*int\(2\)'):
            dice2roulette.translate('let x = int(2, 1) in (x, true)\n', 'test', joint=True)

    @unittest.skipUnless(RACKET, 'ROULETTE_RACKET is not set')
    def test_joint_gives_the_joint_distribution(self):
        # given x || y: (0, 1) 0.125, (1, 0) 0.375, (1, 1) 0.125 of 0.625
        out = run(dice2roulette.translate(self.JOINT, 'test', joint=True))
        comps = bench.parse_alea(out, 'roulette')[0]
        self.assertEqual(len(comps), 1)
        for u, p in enumerate([0.0, 0.2, 0.6, 0.2]):
            self.assertAlmostEqual(comps[0][str(u)], p, delta=1e-12)

    @unittest.skipUnless(RACKET, 'ROULETTE_RACKET is not set')
    def test_roulette_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            code = translate(text)
            for flags in [(), ('--no-complement',)]:
                with self.subTest(name=name, flags=flags):
                    out = run(code, *flags)
                    self.assertRegex(out, r'cpu time: \d+ real time: \d+ gc time: \d+')
                    got = bench.alea_answers(qs, bench.parse_alea(out, 'roulette')[0])
                    for q, w in zip(qs, want):
                        self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    @unittest.skipUnless(RACKET, 'ROULETTE_RACKET is not set')
    def test_integer_operations_wrap_as_dices(self):
        # Every operation on two 3-bit values of distinct masses, whose
        # result's marginal tells its values apart, against the enumeration.
        x = ', '.join(str(k / 36) for k in range(1, 9))
        y = ', '.join(str(k / 36) for k in range(8, 0, -1))
        for op in ['+', '-', '*', '==', '!=', '<', '<=', '>', '>=']:
            with self.subTest(op=op):
                text = f'let x = discrete({x}) in let y = discrete({y}) in (x, (y, x {op} y))'
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = bench.alea_answers(qs, bench.parse_alea(run(translate(text)), 'roulette')[0])
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-12, msg=q)


if __name__ == '__main__':
    unittest.main()
