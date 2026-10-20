"""Storm (scripts/storm_weather.py) and PRISM on Rubicon's weather factory
against gen_geni.py's reference (the dynamic program over the chain's
states): STORM_PYTHON (a Python with stormpy), PRISM_DIR (PRISM's
directory) and WEATHER_MODELS (the directory with weatherfactoryN.prism),
as install_modelcheckers.sh sets them up."""
import os
import re
import subprocess
import sys
import unittest
from pathlib import Path

scripts = Path(__file__).resolve().parents[1] / 'scripts'
MODELS = os.environ.get('WEATHER_MODELS')


def reference(n, h):
    out = subprocess.run([sys.executable, str(scripts / 'gen_geni.py'), 'weather', str(n), str(h), '--reference'],
                         capture_output=True, text=True, check=True).stdout
    return float(out.splitlines()[0].split(':')[-1])   # the goal's; the second query is the constant false


@unittest.skipUnless(MODELS, 'WEATHER_MODELS names the PRISM models')
class WeatherTests(unittest.TestCase):
    CASES = [(2, 10), (2, 3), (7, 10)]

    @unittest.skipUnless(os.environ.get('STORM_PYTHON'), 'STORM_PYTHON')
    def test_storm(self):
        for n, h in self.CASES:
            for engine in ('sparse', 'exact', 'dd'):
                with self.subTest(n=n, h=h, engine=engine):
                    out = subprocess.run([os.environ['STORM_PYTHON'], str(scripts / 'storm_weather.py'),
                                          f'{MODELS}/weatherfactory{n}.prism', str(h), engine],
                                         capture_output=True, text=True, check=True).stdout
                    got = float(re.search(r'^result (\S+)', out, re.M).group(1))
                    self.assertAlmostEqual(got, reference(n, h), delta=1e-12 * max(1, reference(n, h)) + 1e-15)

    @unittest.skipUnless(os.environ.get('PRISM_DIR'), 'PRISM_DIR')
    def test_prism(self):
        for n, h in self.CASES:
            for engine in ('sparse', 'hybrid'):
                with self.subTest(n=n, h=h, engine=engine):
                    out = subprocess.run([f"{os.environ['PRISM_DIR']}/bin/prism", f'{MODELS}/weatherfactory{n}.prism',
                                          '-pf', f'P=? [F<={h} "allStrike"]', '-' + engine],
                                         capture_output=True, text=True, check=True).stdout
                    got = float(re.search(r'^Result: (\S+)', out, re.M).group(1))
                    self.assertAlmostEqual(got, reference(n, h), delta=1e-12 * max(1, reference(n, h)) + 1e-15)


if __name__ == '__main__':
    unittest.main()
