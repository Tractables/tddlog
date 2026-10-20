"""Genfer's, SPPL's and Psi's own programs of the bitblast suite
(scripts/gen_genfer.py, gen_psi_linreg.py, sppl_markov.py) and how bench.py
reads Genfer. Where Genfer is installed (GENFER, its binary), the programs
run and their answers are checked: the population model's posterior mean
against data/alea/genfer/README.md, a Dice program's against its exact
value."""
from pathlib import Path
import os
import shlex
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / 'scripts'))
import bench  # noqa: E402
import gen_genfer  # noqa: E402
import gen_psi_linreg  # noqa: E402

GENFER = os.environ.get('GENFER', '')
POPULATION_4 = """population ~ Poisson(51.4);

population ~ Binomial(population, 0.2636);
population +~ Poisson(232.6);
observe 45 ~ Binomial(population, 0.2);

population ~ Binomial(population, 0.2636);
population +~ Poisson(420.8);
observe 98 ~ Binomial(population, 0.2);

population ~ Binomial(population, 0.2636);
population +~ Poisson(300.8);
observe 73 ~ Binomial(population, 0.2);

population ~ Binomial(population, 0.2636);
population +~ Poisson(85.6);
observe 38 ~ Binomial(population, 0.2);

return population
"""


class GenferTests(unittest.TestCase):
    def test_four_seasons_are_genfers_program(self):
        self.assertEqual(gen_genfer.population(4).strip(), POPULATION_4.strip())

    def test_seasons_repeat_the_four(self):
        text = gen_genfer.population(6)
        self.assertEqual(text.count('observe'), 6)
        self.assertEqual(text.splitlines()[-3], 'observe 98 ~ Binomial(population, 0.2);')

    def test_genfers_output_is_read(self):
        out = ('Total measure:             Z = 3/4\nExpected value:            E = 509/719\n'
               'Normalized:   p(0) / Z = 1/4\nNormalized:   p(1) / Z = 3.0e-1\n'
               'Normalized:   p(n)     <= 1e-3 for all n >= 2\n')
        stats, masses = bench.parse_genfer(out)
        self.assertEqual(stats, {'Z': 0.75, 'E': 509 / 719})
        self.assertEqual(masses, {0: 0.25, 1: 0.3})

    def test_psi_linear_regression_has_the_programs_data(self):
        lines = gen_psi_linreg.program(10, 'b').splitlines()
        observes = [line for line in lines if 'cobserve' in line]
        self.assertEqual(len(observes), 10)
        self.assertEqual(observes[0], '  cobserve(gauss(a * (-1.6) + b, 1), 2.7139);')
        self.assertEqual(lines[-2], '  return b;')

    @unittest.skipUnless(GENFER, 'GENFER is not set')
    def test_genfer_gives_the_population_mean(self):
        for steps, mean in [(1, 241.919232), (4, 194.275228)]:
            with tempfile.TemporaryDirectory() as d:
                path = Path(d) / 'p.sgcl'
                path.write_text(gen_genfer.population(steps))
                out = subprocess.run(shlex.split(GENFER) + [str(path)], capture_output=True, text=True, check=True).stdout
            self.assertAlmostEqual(bench.parse_genfer(out)[0]['E'], mean, delta=1e-5)


if __name__ == '__main__':
    unittest.main()
