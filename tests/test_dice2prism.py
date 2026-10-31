"""Dice programs as Markov chains in the PRISM language
(scripts/dice2prism.py, over scripts/dice_ir.py) and how bench.py reads
what Storm (scripts/storm_check.py) and PRISM print for them.

Every Dice program of data/dice and data/nodice (test_nodice.dice_programs) that dice2problog.py
translates translates here, except where it has recursion, with a model
and its properties for each component of its value. Where the checkers
are installed (STORM_PYTHON, a Python with stormpy, and PRISM_DIR,
PRISM's directory, as install_modelcheckers.sh sets them up), the models
are checked with each engine bench.py runs, and the answers, read as
bench.py reads them, are compared with the exact distribution of
test_dice2problog.py: observations, integers, functions and `iterate`.
"""
from pathlib import Path
import os
import re
import subprocess
import sys
import tempfile
import types
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
data = root / 'data'
sys.path.insert(0, str(scripts))
import source_tree  # noqa: E402,F401  (the source tree's scripts and tests)
sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench  # noqa: E402
from test_nodice import dice_programs  # noqa: E402
import dice2prism  # noqa: E402
import dice2problog  # noqa: E402
import dice_ir  # noqa: E402
import test_dice2problog as reference  # noqa: E402
from test_dice2genfer import CAESAR, JOINT, JOINT_MASSES, OPS, SMALL, X, Y  # noqa: E402

STORM = os.environ.get('STORM_PYTHON', '')
PRISM = os.environ.get('PRISM_DIR', '')
# a function called twice, whose argument is the first call's value, and an observation of the second
FUNCTION = ('fun f(x: bool) { let y = flip 0.3 in x || y } '
            'let a = flip 0.4 in let b = f(a) in let c = f(!b) in let o = observe (b && c) in (a, (b, c))')
# a loop of seven rounds over an int(3) and a Boolean, observed at the end
LOOP = ('fun step(s: (int(3), bool)) { let n = fst s in let t = snd s in '
        'let d = flip 0.3 in (if d then n + int(3, 1) else n, t ^ d) } '
        'let r = iterate(step, (int(3, 0), false), 7) in let o = observe (snd r) in fst r')


def program_text(name):
    return (data / 'dice' / f'{name}.dice').read_text()


def commands(model):
    return [line for line in model.splitlines() if line.startswith('  [] ')]


def checker_answers(text, qs, system, engine, joint=False):
    """bench.py's reading of the checker's runs, one model a component."""
    _, _, tys = dice2prism.translate(text, None, joint=joint)
    args = types.SimpleNamespace(storm=STORM, prism=PRISM, prism_heap='2g')
    comps = []
    for k, t in enumerate(tys):
        model, props, _ = dice2prism.translate(text, k, joint=joint)
        with tempfile.TemporaryDirectory() as d:
            path, ppath = Path(d) / 'p.prism', Path(d) / 'p.props'
            path.write_text(model)
            ppath.write_text(props)
            r = subprocess.run(bench.checker_command(args, system, engine, str(path), str(ppath)),
                               capture_output=True, text=True, timeout=300)
            if r.returncode:
                raise AssertionError(f'{system} {engine}: {r.stdout[-800:]} {r.stderr[-800:]}')
            comps.append(bench.checker_distribution(system, r.stdout, str(ppath),
                                                    ['bool'] if t == dice_ir.BOOL else ['int', str(t[1])]))
    return comps


class Dice2PrismTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        paths = dice_programs()
        for path in paths:
            with self.subTest(name=path.name):
                text = path.read_text()
                try:
                    _, _, tys = dice2prism.translate(text, None)
                except NotImplementedError:
                    continue   # lists, /, % and recursion
                for k in range(len(tys)):
                    try:
                        model, props, _ = dice2prism.translate(text, k, path.name)
                    except NotImplementedError:
                        continue   # an int beyond 30 bits
                    self.assertIn('\ndtmc\n', model)
                    self.assertTrue(model.rstrip().endswith('label "done" = pc=' +
                                                            re.search(r'\[\] pc=(\d+) -> true;', model).group(1) + ';'))
                    for line in commands(model):
                        self.assertRegex(line, r'^  \[\] pc=\d+.* -> .*;$')
                    for line in props.splitlines():
                        self.assertRegex(line, r'^"(obs|v\d+)": P=\? \[F .*\];$')

    def test_registers_are_reused(self):
        # a chain of a hundred flips needs a few variables, not a hundred
        text = 'let a0 = flip 0.5 in ' + ''.join(f'let a{i} = a{i - 1} ^ flip 0.5 in ' for i in range(1, 100)) + 'a99'
        model, _, _ = dice2prism.translate(text, 0)
        self.assertLessEqual(len(re.findall(r'^  x\d+ :', model, re.M)), 2)
        self.assertEqual(len(commands(model)), 100 + 1)   # a step a flip, and the end

    def test_a_component_needs_only_its_statements(self):
        text = 'let a = flip 0.1 in let b = discrete(0.5, 0.25, 0.25) in (a, b)'
        self.assertEqual(dice2prism.translate(text, None)[2], [dice_ir.BOOL, ('int', 2)])
        model, props, _ = dice2prism.translate(text, 1)
        self.assertIn('0.5:', model)
        self.assertNotIn('0.1:', model)
        self.assertEqual(re.findall(r'^"(\w+)"', props, re.M), ['v0', 'v1', 'v2'])

    def test_an_observation_goes_to_an_absorbing_state(self):
        model, props, _ = dice2prism.translate('let a = flip 0.1 in let b = flip 0.2 in let c = observe (a || b) in a', 0)
        self.assertEqual(re.findall(r'^"(\w+)"', props, re.M), ['obs', 'v1'])
        self.assertEqual(len(re.findall(r'-> true;$', model, re.M)), 2)

    def test_a_loop_is_a_counter(self):
        # the rounds are counted, not unrolled: as many commands for 7 rounds as for 70
        small, _, _ = dice2prism.translate(LOOP, 0)
        large, _, _ = dice2prism.translate(LOOP.replace(', 7)', ', 70)'), 0)
        self.assertEqual(len(commands(small)), len(commands(large)))

    def test_probabilities_are_exact(self):
        self.assertEqual(dice2prism.probability(dice2prism.Fraction(1, 10)), '0.1')
        self.assertEqual(dice2prism.probability(dice2prism.Fraction(1, 3)), '1/3')
        model, _, _ = dice2prism.translate('discrete(0.1, 0.2, 0.7)', 0)
        self.assertRegex(model, r'0\.1:.*\+ 0\.2:.*\+ 0\.7:')

    def test_what_prism_cannot_hold_is_refused(self):
        for program in [program_text('geometric'), 'let x = flip 0.5 in if x then int(31, 1) else int(31, 2)']:
            with self.subTest(program=program[:30]):
                with self.assertRaises(NotImplementedError):
                    dice2prism.translate(program, 0)
        r = subprocess.run([sys.executable, str(scripts / 'dice2prism.py'), str(data / 'dice' / 'geometric.dice'),
                            '--components'], capture_output=True, text=True)
        self.assertEqual(r.returncode, bench.REFUSED)
        self.assertIn('recursion', r.stderr)

    def test_a_joint_is_one_integer(self):
        self.assertEqual(dice2prism.translate(JOINT, None, joint=True)[2], [('int', 3)])

    def test_the_checkers_output_is_read(self):
        with tempfile.TemporaryDirectory() as d:
            props = Path(d) / 'p.props'
            props.write_text('"obs": P=? [F "done"];\n"v1": P=? [F ("done" & x0)];\n')
            out = 'result 0.5 1/2\nresult 0.25 1/4\nstates 7\ninference_seconds 0.01\n'
            self.assertEqual(bench.checker_distribution('storm', out, str(props), ['bool']),
                             {'true': 0.5, 'false': 0.5})
            out = 'Result: 0.5 (exact floating point)\n\nResult: 0.125 (+/- 1E-16 estimated)\n'
            self.assertEqual(bench.checker_distribution('prism', out, str(props), ['bool']),
                             {'true': 0.25, 'false': 0.75})
            with self.assertRaises(ValueError):
                bench.checker_distribution('prism', 'Result: 0.5\n', str(props), ['bool'])

    def check(self, system, engines, programs, delta):
        for text in programs:
            qs = reference.queries(dice2problog.translate(text))
            want = reference.answers(reference.exact(text), qs)
            for engine in engines:
                with self.subTest(system=system, engine=engine, program=text[-30:]):
                    got = bench.alea_answers(qs, checker_answers(text, qs, system, engine))
                    for q, w in zip(qs, want):
                        self.assertAlmostEqual(got[re.sub(r'\s+', '', q)], w, delta=delta, msg=q)

    def programs(self):
        return [program_text(name) for name in SMALL] + \
            [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x {op} y))' for op in OPS] + \
            [f'let x = discrete({X}) in (x, (x << 1, x >> 1))', f'let x = discrete({X}) in (x, int(3, 5) - x)',
             f'let x = discrete({X}) in (x, x * int(3, 3))', CAESAR, FUNCTION, LOOP]

    @unittest.skipUnless(STORM, 'STORM_PYTHON is not set')
    def test_storm_gives_the_exact_answers(self):
        self.check('storm', ('sparse', 'exact', 'dd'), self.programs(), 1e-12)

    @unittest.skipUnless(PRISM, 'PRISM_DIR is not set')
    def test_prism_gives_the_exact_answers(self):
        self.check('prism', ('sparse', 'hybrid'), [program_text(name) for name in SMALL[:6]] +
                   [f'let x = discrete({X}) in let y = discrete({Y}) in (x, (y, x - y))', CAESAR, FUNCTION, LOOP], 1e-12)

    @unittest.skipUnless(STORM and PRISM, 'STORM_PYTHON or PRISM_DIR is not set')
    def test_the_checkers_give_the_joint(self):
        for system, engine in [('storm', 'exact'), ('storm', 'sparse'), ('prism', 'sparse')]:
            with self.subTest(system=system, engine=engine):
                comp, = checker_answers(JOINT, None, system, engine, joint=True)
                for u, p in JOINT_MASSES.items():
                    self.assertAlmostEqual(comp[str(u)], p, delta=1e-12, msg=u)


if __name__ == '__main__':
    unittest.main()
