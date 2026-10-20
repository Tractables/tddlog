"""The literature families: each generator writes its small program in
tests/lit/ again from the same arguments, and lit_exact.py answers it as
ProbLog does.

tests/it/examples.rs checks tddlog's answers on the same programs against the
same ProbLog answers (ProbLog 2.3.0, `-k ddnnf`).
"""
from pathlib import Path
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
lit = root / 'tests' / 'lit'
sys.path.insert(0, str(scripts))
import lit_exact  # noqa: E402

# program -> (family, generator arguments, ProbLog's answers, one query at a
# time: asked several at once, ProbLog 2.3.0's -k ddnnf answers pedigree.pl
# wrongly)
EXAMPLES = {
    'mnist_sum.pl': ('mnist_sum', ['2'], [
        ('sum(0)', 0.00140544),
        ('sum(1)', 0.00374336),
        ('sum(2)', 0.022827),
        ('sum(3)', 0.03859655),
        ('sum(4)', 0.00591345),
        ('sum(5)', 0.02648234),
        ('sum(6)', 0.04219473),
        ('sum(7)', 0.02116928),
        ('sum(8)', 0.05089386),
        ('sum(9)', 0.08413355),
        ('sum(10)', 0.04095259),
        ('sum(11)', 0.50704523),
        ('sum(12)', 0.02676586),
        ('sum(13)', 0.02359843),
        ('sum(14)', 0.02757976),
        ('sum(15)', 0.03501714),
        ('sum(16)', 0.00668589),
        ('sum(17)', 0.00121972),
        ('sum(18)', 0.03377582),
    ]),
    'pedigree.pl': ('pedigree', ['3', '3'], [
        ('bloodtype(i2_0,a)', 0.31673708),
        ('bloodtype(i2_0,b)', 0.2366961),
        ('bloodtype(i2_0,ab)', 0.17176306),
        ('bloodtype(i2_0,null)', 0.19319018),
        ('bloodtype(i2_1,a)', 0.31798337),
        ('bloodtype(i2_1,b)', 0.23088535),
        ('bloodtype(i2_1,ab)', 0.18444069),
        ('bloodtype(i2_1,null)', 0.17794811),
        ('bloodtype(i2_2,a)', 0.31673708),
        ('bloodtype(i2_2,b)', 0.2366961),
        ('bloodtype(i2_2,ab)', 0.17176306),
        ('bloodtype(i2_2,null)', 0.19319018),
    ]),
    'sickness.pl': ('sickness', ['4', '4'], [
        ('sick(p0,3)', 0.8272),
        ('sick(p1,3)', 0.89632),
        ('sick(p2,3)', 0.712),
        ('sick(p3,3)', 0.52),
    ]),
    'noisyor.pl': ('noisyor', ['6', '8'], [
        ('disease(d0)', 0.00028898761),
        ('disease(d1)', 0.23203622),
        ('disease(d2)', 0.21170946),
        ('disease(d3)', 0.00032129522),
        ('disease(d4)', 0.22811529),
        ('disease(d5)', 0.11132415),
    ]),
    'mastermind.pl': ('mastermind', ['4', '3', '1'], [
        ('code(1,c0)', 0.0),
        ('code(1,c1)', 0.35714286),
        ('code(1,c2)', 0.35714286),
        ('code(1,c3)', 0.28571429),
        ('code(2,c0)', 0.42857143),
        ('code(2,c1)', 0.28571429),
        ('code(2,c2)', 0.28571429),
        ('code(2,c3)', 0.0),
        ('code(3,c0)', 0.42857143),
        ('code(3,c1)', 0.28571429),
        ('code(3,c2)', 0.28571429),
        ('code(3,c3)', 0.0),
    ]),
    'pdb.pl': ('pdb', ['4'], [
        ('h0', 0.2511066),
        ('h1', 0.64132759),
        ('safe', 0.50430046),
    ]),
}


class LitTests(unittest.TestCase):
    def test_generators_write_the_programs(self):
        for name, (family, args, _) in EXAMPLES.items():
            with self.subTest(name=name):
                out = subprocess.run([sys.executable, str(scripts / f'gen_{family}.py')] + args,
                                     capture_output=True, text=True, check=True).stdout
                self.assertEqual(out, (lit / name).read_text())

    def test_the_reference_answers_as_problog(self):
        for name, (family, _, expected) in EXAMPLES.items():
            with self.subTest(name=name):
                text = (lit / name).read_text()
                answers = lit_exact.FAMILIES[family](text)
                self.assertEqual(lit_exact.queries(text), [q for q, _ in expected])
                for q, p in expected:
                    self.assertAlmostEqual(answers[q], p, delta=1e-7, msg=q)


if __name__ == '__main__':
    unittest.main()
