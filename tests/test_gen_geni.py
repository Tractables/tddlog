"""scripts/gen_geni.py: the programs of Geni's evaluation that Dice runs
(data/geni/README.md) as ProbLog and Dice programs, and their answers.

The reference answers (a dynamic program over each model's states) are
checked against an exact enumeration of the Dice program the script
writes for the same size (tests/test_dice2problog.py's `exact`), and on
small grids against hand counts; tddlog's answers on each family's
ProbLog program (bench.py's FOLD) against the references; the bench
suite (`--suite geni`) against its ladders.
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
                n2, init, sun_p, strike_p, h = gen_geni.weather_chain(gen_geni.weather_artifact(n, 10))
                self.assertEqual((n2, h), (n, 10))
                self.assertEqual(init, [True] + [False] * n)
                self.assertEqual(sun_p, [0.6, 0.7])          # P(sun tomorrow | rain, sun)

    def test_a_horizon_replaces_the_steps(self):
        text = gen_geni.weather_artifact(10, 3)
        self.assertEqual(text.count('step(res)'), 3)
        self.assertTrue(text.endswith(gen_geni.WEATHER_LAST))
        self.assertEqual(gen_geni.weather_chain(text)[4], 3)

    def test_the_dice_programs_loop(self):
        # A day and a row of the grid are functions that `iterate` runs, so
        # that a program does not grow with the horizon nor, past its rows'
        # width, with the grid; at the sizes of SIZES the enumeration above
        # checks their answers.
        for days in (3, 1000):
            text = gen_geni.dice_program('weather', [10, days])
            self.assertEqual(text.count('step(res)'), 1)
            self.assertIn(f'iterate(day, (false, res), {days})', text)
            self.assertTrue(text.rstrip('\n').endswith(gen_geni.WEATHER_LAST))
        for family in ('link', 'router'):
            for n in (3, 12):
                text = gen_geni.dice_program(family, [n])
                self.assertIn(f'let last = iterate(row, ', text)
                self.assertTrue(text.rstrip('\n').endswith(f'{n - 1}) in\n' + gen_geni.row_cell('last', n - 1, n)))
                # the first row's flips and the row function's: link's first
                # cell is the source, each later one 1 and 3 (left, up,
                # upper left); a router's 1 and 2, 2 and 4 (both branches)
                self.assertEqual(text.count('flip'), {'link': 4 * n - 3, 'router': 6 * n - 3}[family])
        self.assertEqual(gen_geni.row_cell('up', 0, 3), 'fst up')
        self.assertEqual(gen_geni.row_cell('up', 1, 3), 'fst (snd up)')
        self.assertEqual(gen_geni.row_cell('up', 2, 3), 'snd (snd up)')
        self.assertEqual(gen_geni.row_type(3), '(bool, (bool, bool))')

    def test_a_weather_program_the_artifact_lacks_fails(self):
        r = gen('weather', '14', '10', '--reference')
        self.assertNotEqual(r.returncode, 0)
        self.assertIn('no program of 14 factories', r.stderr)


def tddlog_answers(gargs):
    """tddlog's answers on a FOLD instance as bench.py runs it: the family's
    program and its data files in a directory of their own, the instance's
    arguments after the program."""
    program, text = bench.fold_template(gargs)
    (root / 'tmp').mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=root / 'tmp') as d:
        os.makedirs(os.path.join(d, bench.FOLD_DIR))
        os.makedirs(os.path.join(d, bench.DATA_DIR))
        for key in bench.fold_of(gargs)[2]:
            Path(d, bench.DATA_DIR, key + '.pl').write_text(bench.fold_data(gargs, key))
        path = Path(d, bench.FOLD_DIR, program)
        path.write_text(text)
        out = subprocess.run([str(bench.source_tree.BINARY), '--precision', '17', str(path),
                              *bench.fold_arguments(gargs)], capture_output=True, text=True)
    if out.returncode != 0:
        raise AssertionError(f'{gargs}: {out.stderr[-2000:]}')
    return bench.parse_answers(out.stdout)


class Programs(unittest.TestCase):
    """Each family's ProbLog program (bench.py's FOLD): data/geni/FAMILY.pl,
    the grid's side or the weather's days its argument, the factories a
    data file."""

    def test_tddlog_answers_each_program_as_the_reference(self):
        for family, sizes in SIZES + [('link', [6]), ('router', [8]), ('weather', [7, 10]), ('weather', [10, 30])]:
            with self.subTest(family=family, sizes=sizes):
                got = tddlog_answers(['gen_geni.py', family, *map(str, sizes)])
                want = gen_geni.reference(family, sizes)
                self.assertEqual(list(got), gen_geni.query_names(family, sizes))
                for q, p in zip(got, want):
                    self.assertAlmostEqual(got[q], p, delta=1e-12, msg=q)

    def test_a_program_is_answered_by_its_instance_name(self):
        (root / 'tmp').mkdir(exist_ok=True)
        for name, family, sizes in [('geni_link_4', 'link', [4]), ('geni_router_4', 'router', [4]),
                                    ('geni_weather_n2', 'weather', [2, 10]), ('geni_weather_h3', 'weather', [10, 3])]:
            with self.subTest(name=name), tempfile.TemporaryDirectory(dir=root / 'tmp') as d:
                self.assertEqual(bench.GENI[name][1], ['gen_geni.py', family, *map(str, sizes)])
                path = Path(d, name + '.pl')
                path.write_text(bench.folded_program(bench.GENI[name][1]))
                queries = [line[len('query('):-2] for line in path.read_text().splitlines()
                           if line.startswith('query(')]
                self.assertEqual(queries, gen_geni.query_names(family, sizes))
                r = gen('--reference-of', str(path))
                self.assertEqual(r.returncode, 0, r.stderr)
                self.assertEqual(r.stdout, gen(family, *map(str, sizes), '--reference').stdout)

    def test_a_family_is_one_program_its_arguments_and_data(self):
        for family in gen_geni.FAMILIES:
            self.assertEqual(gen(family, '--template').stdout, (root / 'data' / 'geni' / f'{family}.pl').read_text())
        self.assertEqual(gen('link', '5', '--arguments').stdout, '5\n')
        self.assertEqual(gen('weather', '7', '30', '--arguments').stdout, '30\n')
        self.assertEqual(gen_geni.fold('weather', [7, 30]), ('weather', ['weather'], ['weather_7']))
        self.assertEqual(gen_geni.fold('router', [5]), ('router', ['router'], []))
        _, _, _, strike_p, _ = gen_geni.weather_chain(gen_geni.weather_artifact(2, 10))
        lines = gen('weather', '2', '10', '--data', 'weather_2').stdout.splitlines()
        self.assertIn('factory(2).', lines)
        self.assertIn(f'{strike_p[1][0][1]!r}::strikes(2, T, off, sun) :- step(T).', lines)
        self.assertEqual(sum(line.startswith(('0', '1')) for line in lines), 8)

    def test_inert_options_fail(self):
        self.assertNotEqual(gen('link', '3', '--reference', '--dice', 'x.dice').returncode, 0)
        self.assertNotEqual(gen('link', '3', '4', '--reference').returncode, 0)
        self.assertNotEqual(gen('weather', '10', '--reference').returncode, 0)
        self.assertNotEqual(gen('link', '3').returncode, 0)
        self.assertNotEqual(gen('link', '3', '--template').returncode, 0)
        self.assertNotEqual(gen('link', '3', '--data', 'weather_3').returncode, 0)
        self.assertNotEqual(gen('weather', '2', '10', '--data', 'weather_7').returncode, 0)
        self.assertNotEqual(gen('--reference-of', 'x.pl', 'link').returncode, 0)
        self.assertFalse(Path('x.dice').exists())


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
