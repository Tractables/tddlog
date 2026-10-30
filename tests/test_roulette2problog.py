"""scripts/roulette2problog.rkt writes, from each Roulette program of
tests/roulette/ (the .rkt files), the ProbLog program committed beside it,
whose answers tests/it/examples.rs checks against the ones computed by hand
in the program's comment; with --tddlog it runs the program with tddlog
answering its queries; and it refuses what it does not read.

Racket runs only where TDDLOG_ROULETTE names it, as a command template with
{file} for the script, e.g. "env PLTUSERHOME=R/home PLTADDONDIR=R/addon
R/racket/bin/racket {file}" with R an installation of Roulette
(https://github.com/camoy/roulette); otherwise the tests are skipped. The
test of --tddlog runs the binary that TDDLOG_BIN names, by default
target/release/tddlog.
"""
import os
from pathlib import Path
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


def program(d, text):
    path = Path(d, 'program.rkt')
    path.write_text('#lang roulette/example/disrupt\n' + text)
    return path


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

    def test_usage(self):
        r = racket()
        self.assertEqual(r.returncode, 2)
        self.assertIn('usage: racket roulette2problog.rkt [--tddlog TDDLOG] PROGRAM.rkt', r.stderr)


if __name__ == '__main__':
    unittest.main()
