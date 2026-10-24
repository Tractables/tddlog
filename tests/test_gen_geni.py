"""scripts/gen_geni.py: the programs of Geni's evaluation that Dice runs
(data/geni/README.md) as ProbLog and Dice programs, and their answers.

The reference answers (a dynamic program over each model's states) are
checked against an exact enumeration of the Dice program the script
writes for the same size (tests/test_dice2problog.py's `exact`), and on
small grids against hand counts; the bench suite (`--suite geni`) against
its ladders.
"""
from pathlib import Path
from types import SimpleNamespace
import math
import os
import shlex
import subprocess
import tempfile
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
import source_tree  # noqa: E402,F401  (the source tree's scripts and tests)
sys.path.insert(0, str(root / 'tests'))
import bench  # noqa: E402
import dice2problog  # noqa: E402
import gen_geni  # noqa: E402
from test_dice2problog import exact  # noqa: E402

# Sizes whose Dice programs the enumeration evaluates in a few seconds.
SIZES = [('link', [2]), ('link', [3]), ('router', [2]), ('router', [3]),
         ('weather', [2, 1]), ('weather', [2, 2]), ('weather', [2, 3])]


def gen(*args):
    return subprocess.run([sys.executable, str(scripts / 'gen_geni.py'), *args], capture_output=True, text=True)


class Reference(unittest.TestCase):
    def test_the_dynamic_program_is_the_dice_programs_answer(self):
        for family, sizes in SIZES:
            with self.subTest(family=family, sizes=sizes):
                dist = exact(gen_geni.dice_program(family, sizes))
                if family == 'weather':
                    # The value (hit(res), false).
                    want = sum(p for v, p in dist.items() if v[0] is True)
                    self.assertTrue(all(v[1] is False for v in dist))
                else:
                    want = dist.get(True, 0.0)
                got = gen_geni.reference(family, sizes)
                self.assertTrue(math.isclose(got[0], want, rel_tol=1e-12), (got, want))
                if family == 'weather':
                    self.assertEqual(got[1], 0.0)

    def test_small_grids_by_hand(self):
        # 2 x 2 links: the corner is reached from the left, the upper or
        # the upper-left router, each reached with 1/2, 1/2, 1 and its link
        # working with 1/2: 1 - (1 - 1/4)^2 (1 - 1/2) = 0.71875.
        self.assertEqual(gen_geni.reference('link', [2]), [0.71875])
        # 1 x 1 would be the first router alone; a 2 x 2 router grid's corner
        # works with probability between the inner parameters'.
        p = gen_geni.reference('router', [2])[0]
        self.assertTrue(0.5000003 <= p <= 0.5000006)

    def test_the_weather_factory_by_hand(self):
        # One day from (sun, no strikes): factory i strikes with
        # P(strike | no strike, sun), 0.14 and 0.21 for the 2 factories.
        self.assertTrue(math.isclose(gen_geni.reference('weather', [2, 1])[0], 0.14 * 0.21, rel_tol=1e-15))

    def test_the_stored_weather_programs_parse(self):
        for n in [2, 7, 13, 19]:
            with self.subTest(n=n):
                n2, init, sun_p, strike_p, h = gen_geni.weather_chain(gen_geni.dice_program('weather', [n, 10]))
                self.assertEqual((n2, h), (n, 10))
                self.assertEqual(init, [True] + [False] * n)
                self.assertEqual(sun_p, [0.6, 0.7])          # P(sun tomorrow | rain, sun)

    def test_a_horizon_replaces_the_steps(self):
        text = gen_geni.dice_program('weather', [10, 3])
        self.assertEqual(text.count('step(res)'), 3)
        self.assertTrue(text.endswith(gen_geni.WEATHER_LAST))
        self.assertEqual(gen_geni.weather_chain(text)[4], 3)

    def test_a_weather_program_the_artifact_lacks_fails(self):
        r = gen('weather', '14', '10')
        self.assertNotEqual(r.returncode, 0)
        self.assertIn('no program of 14 factories', r.stderr)


class Programs(unittest.TestCase):
    def test_the_problog_program_answers_by_its_first_line(self):
        for family, sizes in [('link', ['3']), ('router', ['3']), ('weather', ['2', '2'])]:
            with self.subTest(family=family):
                out = gen(family, *sizes).stdout
                self.assertTrue(out.startswith(f'% gen_geni.py {family} {" ".join(sizes)}\n'))
                self.assertEqual(out.split('\n', 1)[1], dice2problog.translate(
                    gen_geni.dice_program(family, [int(s) for s in sizes])))
                (root / 'tmp').mkdir(exist_ok=True)
                path = root / 'tmp' / f'test_gen_geni_{family}.pl'
                path.write_text(out)
                try:
                    r = gen('--reference-of', str(path))
                finally:
                    path.unlink()
                self.assertEqual(r.returncode, 0, r.stderr)
                queries = [line[len('query('):-2] for line in out.splitlines() if line.startswith('query(')]
                answers = [line.rsplit(': ', 1) for line in r.stdout.splitlines()]
                self.assertEqual([q for q, _ in answers], queries)
                want = gen_geni.reference(family, [int(s) for s in sizes])
                self.assertEqual([float(v) for _, v in answers], want)

    def test_inert_options_fail(self):
        self.assertNotEqual(gen('link', '3', '--reference', '--dice', 'x.dice').returncode, 0)
        self.assertNotEqual(gen('link', '3', '4').returncode, 0)
        self.assertNotEqual(gen('weather', '10').returncode, 0)
        self.assertNotEqual(gen('--reference-of', 'x.pl', 'link').returncode, 0)


GENI = Path(os.environ['GENI_BENCHMARKS']) if os.environ.get('GENI_BENCHMARKS') else None  # the artifact's bench/ICFP
GENNIFER = os.environ.get('GENNIFER', '')              # the command that runs gennifer, with -l -i


class GeniPrograms(unittest.TestCase):
    """The programs written for Geni itself (gen_geni.py --gir)."""

    def test_a_grid_is_the_artifacts_generators_text(self):
        self.assertEqual(gen_geni.link_gir(2).splitlines(), [
            'let conn_1_1 = true in',
            'let conn_1_2 = (conn_1_1 && sample(Bernoulli(0.5))) in',
            'let conn_2_1 = (conn_1_1 && sample(Bernoulli(0.5))) in',
            'let conn_2_2 = ((conn_2_1 && sample(Bernoulli(0.5))) || (conn_1_2 && sample(Bernoulli(0.5))) || '
            '(conn_1_1 && sample(Bernoulli(0.5)))) in',
            'conn_2_2'])
        self.assertEqual(gen_geni.router_gir(2).splitlines()[:2], [
            'let x_0_0 = sample(Bernoulli(0.5)) in',
            'let x_0_1 = if x_0_0 then sample(Bernoulli(0.5000001)) else sample(Bernoulli(0.5000002)) in'])
        self.assertNotIn('flip', gen_geni.router_gir(3) + gen_geni.link_gir(3))

    @unittest.skipUnless(GENI and GENI.is_dir(), 'GENI_BENCHMARKS is not set')
    def test_the_weather_factory_is_the_artifacts_file_at_every_horizon(self):
        d = GENI / 'weather-factory' / 'rubicon-geni'
        for n, h in [(2, 10), (10, 10), (19, 10), (10, 1), (10, 3), (10, 100), (10, 1000)]:
            with self.subTest(n=n, h=h):
                self.assertEqual(gen_geni.weather_gir(n, h, str(GENI)), (d / f'weather-factory-{n}-{h}.gir').read_text()
                                 .rstrip('\n') + '\n')

    def test_geni_output_is_read(self):
        self.assertEqual(bench.parse_geni('Pr(0) = 0.1840589\nPr(1) = 0.8159411\n'), {'0': 0.1840589, '1': 0.8159411})
        self.assertEqual(bench.parse_geni('Pr(1) = 0.25\nPr(0) = 0.75\n'), {'1': 0.25, '0': 0.75})

    @unittest.skipUnless(GENI and GENI.is_dir() and GENNIFER, 'GENI_BENCHMARKS and GENNIFER are not set')
    def test_geni_gives_the_references_answer_in_single_precision(self):
        for family, sizes in [('link', [3]), ('router', [3]), ('weather', [2, 3]), ('weather', [7, 10])]:
            with self.subTest(family=family, sizes=sizes):
                with tempfile.TemporaryDirectory() as d:
                    path = Path(d) / 'p.gir'
                    path.write_text(gen_geni.gir_program(family, sizes, str(GENI)))
                    out = subprocess.run(shlex.split(GENNIFER) + [str(path)], capture_output=True, text=True,
                                         check=True).stdout
                dist = bench.parse_geni(out)
                got = dist['0'] if family == 'weather' else dist['1']
                self.assertAlmostEqual(got, gen_geni.reference(family, sizes)[0], delta=2e-7)


class Suite(unittest.TestCase):
    def test_the_suite_names_the_programs_on_their_ladders(self):
        args = SimpleNamespace(suite='geni', only=[], families=[])
        spec = bench.instances(args)
        families = {}
        for name, (family, gargs) in spec.items():
            self.assertEqual(gargs[0], 'gen_geni.py')
            families.setdefault(family, []).append((name, gargs[1:]))
        self.assertEqual(sorted(families), ['geni_link', 'geni_router', 'geni_weather_h', 'geni_weather_n'])
        for family, entries in families.items():
            # Smallest first, so that a failure skips the larger.
            key = (lambda g: int(g[2])) if family == 'geni_weather_h' else (lambda g: int(g[1]))
            sizes = [key(g) for _, g in entries]
            self.assertEqual(sizes, sorted(sizes))
        # Ten factories at ten days once, in geni_weather_n.
        tens = [n for n, (_, g) in spec.items() if g[1:] == ['weather', '10', '10']]
        self.assertEqual(tens, ['geni_weather_n10'])
        self.assertEqual(bench.EXACT['gen_geni.py'][1]('x.pl'), ['gen_geni.py', '--reference-of', 'x.pl'])
        self.assertIn('gen_geni.py', bench.DICE_WRITERS)


if __name__ == '__main__':
    unittest.main()
