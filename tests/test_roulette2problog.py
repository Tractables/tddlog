"""scripts/roulette2problog.rkt writes, from each Roulette program of
tests/roulette/ (the .rkt files), the ProbLog program committed beside it,
whose answers tests/it/examples.rs checks against the ones computed by hand
in the program's comment; with --tddlog it runs the program with tddlog
answering its queries; --make compiles a program's modules ahead of its
runs; the circuit is written in time linear in its size; and it refuses
what it does not read.

Racket runs only where TDDLOG_ROULETTE names it, as a command template with
{file} for the script, e.g. "env PLTUSERHOME=R/home PLTADDONDIR=R/addon
R/racket/bin/racket {file}" with R an installation of Roulette
(https://github.com/camoy/roulette); otherwise the tests are skipped. The
test of --tddlog runs the binary that TDDLOG_BIN names, by default
target/release/tddlog.
"""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
script = root / 'scripts' / 'roulette2problog.rkt'
PROGRAMS = sorted((root / 'tests' / 'roulette').glob('*.rkt'))
ROULETTE = os.environ.get('TDDLOG_ROULETTE', '')
TDDLOG = Path(os.environ.get('TDDLOG_BIN', root / 'target' / 'release' / 'tddlog'))


def racket(*args):
    cmd = ROULETTE.format(file=script).split() + [str(a) for a in args]
    return subprocess.run(cmd, capture_output=True, text=True)


def raco_make(path):
    cmd = ROULETTE.format(file='-l- raco make').split() + [str(path)]
    return subprocess.run(cmd, capture_output=True, text=True)


def program(d, text):
    path = Path(d, 'program.rkt')
    path.write_text('#lang roulette/example/disrupt\n' + text)
    return path


# A chain of N flips, each negating the value so far where it holds: N
# flips and at least N gates.
CHAIN = """(define n (string->number (vector-ref (current-command-line-arguments) 0)))
(define (chain i acc) (if (= i n) acc (chain (add1 i) (if (flip 0.5) (not acc) acc))))
(query (chain 0 #f))
"""


@unittest.skipUnless(ROULETTE, 'TDDLOG_ROULETTE names no Racket with Roulette')
class Roulette2ProbLogTests(unittest.TestCase):
    def test_the_committed_circuits(self):
        self.assertTrue(PROGRAMS)
        for p in PROGRAMS:
            with self.subTest(program=p.name):
                r = racket(p)
                self.assertEqual(r.returncode, 0, r.stderr)
                self.assertEqual(r.stdout, p.with_suffix('.pl').read_text())

    @unittest.skipUnless(TDDLOG.exists(), 'no tddlog binary at TDDLOG_BIN')
    def test_tddlog_answers_the_queries(self):
        # Roulette prints each module-level query's distribution, to six digits.
        r = racket('--tddlog', TDDLOG, root / 'tests' / 'roulette' / 'tiny_pr.rkt')
        self.assertEqual(r.returncode, 0, r.stderr)
        lines = r.stdout.splitlines()
        self.assertEqual(len(lines), 4, r.stdout)
        for line, values in zip(lines, [['#t ↦ 0.958333', '#f ↦ 0.041667'],
                                        ["'(#t #f) ↦ 0.25", "'(#t #t) ↦ 0.166667", "'(#f #f) ↦ 0.583333"],
                                        ['1 ↦ 0.416667', '2 ↦ 0.583333'],
                                        ['#t ↦ 0.375', '#f ↦ 0.625']]):
            for v in values:
                self.assertIn(v, line)
        self.assertNotIn('(#f #t)', lines[1])
        self.assertNotIn('3 ↦', lines[2])

    def test_refusals(self):
        with tempfile.TemporaryDirectory() as d:
            for text, message in [
                    ('(define a (flip 0.5))\n', 'program.rkt asks no query'),
                    ('(define a (flip 0.5))\n(query a)\n(observe! a)\n(query a)\n',
                     'query 2 is asked under other observations than query 1'),
                    ('(define-compile (f x) (and x (flip 0.5)))\n(f #t)\n',
                     'staged procedures compile to BDDs and are not supported')]:
                with self.subTest(message=message):
                    r = racket(program(d, text))
                    self.assertNotEqual(r.returncode, 0)
                    self.assertIn(message, r.stderr)

    def test_make_compiles_ahead_of_the_runs(self):
        with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as e:
            p = program(d, '(define a (flip 0.5))\n(define b (flip 0.25))\n(when a (observe! b))\n(query a)\n')
            fresh = racket(program(e, p.read_text().split('\n', 1)[1]))
            self.assertEqual(fresh.returncode, 0, fresh.stderr)
            r = racket('--make', p)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("no module compiled against Roulette's language", r.stderr)
            self.assertEqual(raco_make(p).returncode, 0)
            r = racket('--make', p)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn('compiled 1 of 1 modules in this file\'s language program.rkt', r.stderr)
            self.assertTrue(Path(d, 'compiled', 'roulette2problog', 'program_rkt.zo').exists())
            r = racket('--make', p)
            self.assertIn('compiled 0 of 1 modules', r.stderr)
            r = racket(p)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout, fresh.stdout)
            # A changed source is compiled anew, not read from the stale file.
            p.write_text(p.read_text().replace('(query a)', '(query b)'))
            self.assertEqual(raco_make(p).returncode, 0)
            r = racket(p)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn('q1(0) :- \\+f2.', r.stdout)

    def test_the_circuit_is_written_in_linear_time(self):
        seconds = {}
        with tempfile.TemporaryDirectory() as d:
            p = program(d, CHAIN)
            for n in (20000, 80000):
                r = racket(p, n)
                self.assertEqual(r.returncode, 0, r.stderr)
                m = re.search(r'([0-9]+) gates, .* circuit ([0-9.e-]+) s', r.stderr)
                self.assertGreaterEqual(int(m.group(1)), n)
                seconds[n] = float(m.group(2))
        # Four times the nodes: about four times the time, sixteen were the
        # writing quadratic.
        self.assertLess(seconds[80000], 10 * max(seconds[20000], 0.1), seconds)

    def test_usage(self):
        for args in [(), ('--make',), ('--make', 'a.rkt', '3'), ('--tddlog',)]:
            r = racket(*args)
            self.assertEqual(r.returncode, 2, args)
            self.assertIn('usage: racket roulette2problog.rkt [--make | --tddlog TDDLOG] PROGRAM.rkt', r.stderr)


if __name__ == '__main__':
    unittest.main()
