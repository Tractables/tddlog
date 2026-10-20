"""sample_paths.py against answers computed by hand."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts' / 'sample_paths.py'

RULES = 'path(X,Y) :- edge(X,Y).\npath(X,Y) :- edge(X,Z), path(Z,Y).\n'
# a reaches c directly (0.4) or through b (0.5 * 0.5): 1 - 0.6 * 0.75;
# a reaches a only around the cycle a -> b -> a: 0.5 * 0.2.
PROGRAM = '''% a triangle
0.5::edge(a,b).
0.5::edge(b,c).
0.4::edge(a,c).
0.2::edge(b,a).
''' + RULES + 'query(path(a,c)).\nquery(path(a,a)).\nquery(path(c,a)).\n'
EXACT = {'path(a,c)': 0.55, 'path(a,a)': 0.1, 'path(c,a)': 0.0}


def run(program, *args):
    # Scratch files go in the repository's tmp/, which git ignores.
    scratch = ROOT / 'tmp'
    scratch.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile('w', suffix='.pl', dir=scratch) as f:
        f.write(program)
        f.flush()
        return subprocess.run([sys.executable, str(SCRIPT), f.name, '--samples', '200000', *args],
                              capture_output=True, text=True)


class SamplePaths(unittest.TestCase):
    def test_estimates_are_near_the_hand_counts(self):
        out = run(PROGRAM)
        self.assertEqual(out.returncode, 0, out.stderr)
        for line in out.stdout.splitlines():
            q, p, se = line.split('\t')
            q = q.rstrip(':')
            self.assertLessEqual(abs(float(p) - EXACT[q]), 5 * float(se) + 1e-12, line)

    def test_against_answers(self):
        scratch = ROOT / 'tmp'
        with tempfile.NamedTemporaryFile('w', suffix='.txt', dir=scratch) as f:
            f.write(''.join(f'{q}:\t{p}\n' for q, p in EXACT.items()))
            f.flush()
            out = run(PROGRAM, '--against', f.name)
        self.assertEqual(out.returncode, 0, out.stderr)
        z = float(out.stdout.split('the farthest ')[1].split()[0])
        self.assertLess(z, 5)

    def test_refuses_other_programs(self):
        out = run(PROGRAM + 'path(X,Y) :- path(Y,X).\n')
        self.assertNotEqual(out.returncode, 0)
        self.assertIn('not a reachability program', out.stderr)
        out = run(PROGRAM.replace('path(X,Y) :- edge(X,Z), path(Z,Y).\n', ''))
        self.assertNotEqual(out.returncode, 0)
        self.assertIn('transitive closure', out.stderr)


if __name__ == '__main__':
    unittest.main()
