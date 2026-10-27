"""scripts/alea2problog.jl writes, from each Alea.jl program of tests/alea/
and examples/, the ProbLog program committed beside it, whose answers
tests/it/examples.rs checks against the ones computed by hand in the
program's comment; and it refuses a program not in the form it reads.

Julia runs only where TDDLOG_ALEA names it, as a command template with
{file} for the script, e.g. "julia --project=ALEA {file}" with ALEA a
checkout of Alea.jl (https://github.com/Tractables/Alea.jl); otherwise the
tests are skipped.
"""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
script = root / 'scripts' / 'alea2problog.jl'
PROGRAMS = sorted((root / 'tests' / 'alea').glob('*.jl')) + sorted((root / 'examples').glob('*.jl'))
ALEA = os.environ.get('TDDLOG_ALEA', '')


def julia(*args):
    cmd = ALEA.format(file=script).split() + [str(a) for a in args]
    return subprocess.run(cmd, capture_output=True, text=True)


@unittest.skipUnless(ALEA, 'TDDLOG_ALEA names no Julia with Alea.jl')
class Alea2ProbLogTests(unittest.TestCase):
    def test_the_committed_circuits(self):
        self.assertTrue(PROGRAMS)
        for program in PROGRAMS:
            with self.subTest(program=program.name):
                r = julia(program)
                self.assertEqual(r.returncode, 0, r.stderr)
                self.assertEqual(r.stdout, program.with_suffix('.pl').read_text())

    def test_a_program_without_a_query_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            program = Path(d, 'noquery.jl')
            program.write_text('using Alea\n\nmodel() = flip(0.5)\n')
            r = julia(program)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn('noquery.jl defines no QUERY', r.stderr)
            program.write_text('using Alea\n\nconst QUERY = :mean\n\nmodel() = flip(0.5)\n')
            r = julia(program)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn('QUERY must be :pr or :expectation, not mean', r.stderr)

    def test_usage(self):
        r = julia()
        self.assertEqual(r.returncode, 2)
        self.assertIn('usage: julia --project=ALEA alea2problog.jl PROGRAM.jl', r.stderr)


if __name__ == '__main__':
    unittest.main()
