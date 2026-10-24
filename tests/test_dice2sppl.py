"""Dice programs as SPPL programs (scripts/dice2sppl.py, over
scripts/dice_ir.py, run by scripts/sppl_dice.py) and how bench.py reads
what sppl_dice.py prints.

Every Dice program of data/dice and data/nodice that dice2problog.py
translates translates here, except where it has recursion. Where SPPL is
installed (SPPL, a Python that has the sppl package, e.g.
`$HOME/local/sppl/venv/bin/python`), the translated programs run, and their
answers are checked against the exact distribution of test_dice2problog.py.
"""
from pathlib import Path
import itertools
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
import dice2sppl  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import CAESAR, JOINT, JOINT_MASSES, OPS, SMALL, X, Y  # noqa: E402

SPPL = os.environ.get('SPPL', '')
# an observed conjunction of 250 flips, past the 200 parentheses Python
# nests, and a flip that depends on the first
LONG_AND = (''.join(f'let a{i} = flip 0.99 in ' for i in range(250)) + 'let b = if a0 then flip 0.3 else flip 0.6 in '
            'let o = observe (' + ' && '.join(f'a{i}' for i in range(250)) + ') in b')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def sppl_answers(text, qs):
    """bench.py's reading of sppl_dice.py's run."""
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / 'p.sppl'
        path.write_text(dice2sppl.translate(text))
        out = subprocess.run(shlex.split(SPPL) + ['-I', str(scripts / 'sppl_dice.py'), str(path)],
                             capture_output=True, text=True, check=True, timeout=600).stdout
    comps = {}
    for line in out.splitlines():
        f = line.split()
        if len(f) == 4 and f[0] == 'sppl_c':
            comps.setdefault(int(f[1]), {})[f[2]] = float(f[3])
    return bench.alea_answers(qs, [comps[k] for k in sorted(comps)])


class Dice2SpplTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = sorted((data / 'dice').glob('*.dice')) + sorted((data / 'nodice').glob('*.dice'))
        for path in paths:
            with self.subTest(name=path.name):
                try:
                    code = dice2sppl.translate(path.read_text(), path.name)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                self.assertIn('# component 0 ', code)
                compile(code.replace('~=', '='), path.name, 'exec')   # SPPL's syntax is Python's

    def test_branches_define_the_same_variables(self):
        code = dice2sppl.translate('let a = flip 0.5 in let b = if a then flip 0.2 else int(1, 1) == int(1, 1) in b')
        then, other = code[code.index('\nif '):].split('else:')
        defined = sorted(set(re.findall(r'(\w+) ~=', then)))
        self.assertEqual(defined, sorted(set(re.findall(r'(\w+) ~=', other))))

    def test_an_else_if_chain_is_an_elif_chain(self):
        code = dice2sppl.translate('let x = discrete(0.2, 0.3, 0.5) in if x == int(2, 0) then flip 0.1 else '
                                   'if x == int(2, 1) then flip 0.2 else flip 0.3')
        self.assertEqual(code.count('elif '), 1)
        self.assertEqual(code.count('else:'), 1)

    def test_recursion_is_refused(self):
        with self.assertRaises(NotImplementedError):
            dice2sppl.translate(program_text('geometric'))
        r = subprocess.run([sys.executable, str(scripts / 'dice2sppl.py'), str(data / 'dice' / 'geometric.dice')],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, bench.REFUSED)

    @unittest.skipUnless(SPPL, 'SPPL is not set')
    def test_sppl_gives_the_exact_answers(self):
        for name in SMALL:
            text = program_text(name)
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            with self.subTest(name=name):
                got = sppl_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)

    def test_a_long_conjunction_nests_log_deep(self):
        line = next(t for t in dice2sppl.translate(LONG_AND).splitlines() if t.startswith('condition('))
        depth = max(itertools.accumulate({'(': 1, ')': -1}.get(c, 0) for c in line))
        self.assertLessEqual(depth, 12)

    @unittest.skipUnless(SPPL, 'SPPL is not set')
    def test_sppl_compiles_a_long_conjunction(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.sppl'
            path.write_text(dice2sppl.translate(LONG_AND))
            out = subprocess.run(shlex.split(SPPL) + ['-I', str(scripts / 'sppl_dice.py'), str(path)],
                                 capture_output=True, text=True, check=True, timeout=600).stdout
        p = next(float(f[3]) for f in map(str.split, out.splitlines()) if f[:3] == ['sppl_c', '0', 'true'])
        self.assertAlmostEqual(p, 0.3, delta=1e-9)

    def test_a_joint_is_asked_of_its_bits(self):
        # SPPL has no sum of two variables: sppl_dice.py asks each assignment
        code = dice2sppl.translate(JOINT, joint=True)
        self.assertIn('\n# joint\n', code)
        self.assertEqual(len(re.findall(r'# component \d bool ', code)), 3)

    @unittest.skipUnless(SPPL, 'SPPL is not set')
    def test_sppl_gives_the_joint(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.sppl'
            path.write_text(dice2sppl.translate(JOINT, joint=True))
            out = subprocess.run(shlex.split(SPPL) + ['-I', str(scripts / 'sppl_dice.py'), str(path)],
                                 capture_output=True, text=True, check=True, timeout=600).stdout
        masses = {int(f[2]): float(f[3]) for f in map(str.split, out.splitlines()) if f[0] == 'sppl_c'}
        for u, p in JOINT_MASSES.items():
            self.assertAlmostEqual(masses[u], p, delta=1e-9, msg=u)

    @unittest.skipUnless(SPPL, 'SPPL is not set')
    def test_integer_operations_wrap_as_dices(self):
        programs = [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
                   [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
                    CAESAR]
        for text in programs:
            with self.subTest(program=text[-24:]):
                qs = reference.queries(dice2problog.translate(text))
                want = reference.answers(reference.exact(text), qs)
                got = sppl_answers(text, qs)
                for q, w in zip(qs, want):
                    self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=1e-9, msg=q)


if __name__ == '__main__':
    unittest.main()
