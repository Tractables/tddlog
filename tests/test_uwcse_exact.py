"""uwcse_exact.py against Ace's answers on gen_uwcse.py's programs."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / 'scripts'

# Ace 3.0's answers (auto, on bench.py's network of the ground program;
# uq1b, 2026-10-27), which share nothing with the lifted computation.
ACE = {20: 0.10468882605074412, 80: 0.14217535627981878, 640: 0.35394592748217835}


def answer(program_text):
    # Scratch files go in the repository's tmp/, which git ignores.
    scratch = ROOT / 'tmp'
    scratch.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile('w', suffix='.pl', dir=scratch) as f:
        f.write(program_text)
        f.flush()
        r = subprocess.run([sys.executable, str(SCRIPTS / 'uwcse_exact.py'), f.name, '--precision', '17'],
                           capture_output=True, text=True)
    return r


def program(n):
    return subprocess.run([sys.executable, str(SCRIPTS / 'gen_uwcse.py'), str(n)],
                          capture_output=True, text=True, check=True).stdout


class UwcseExact(unittest.TestCase):
    def test_ace_answers(self):
        for n, p in ACE.items():
            r = answer(program(n))
            self.assertEqual(r.returncode, 0, r.stderr)
            q, value = r.stdout.strip().split(':\t')
            self.assertEqual(q, 'taught_by(c1,p1)')
            self.assertAlmostEqual(float(value), p, delta=1e-13, msg=f'uwcse{n}')

    def test_other_program_refused(self):
        text = program(20).replace('0.125::shared_publication', '0.25::shared_publication')
        r = answer(text)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not gen_uwcse.py's program", r.stderr)


if __name__ == '__main__':
    unittest.main()
