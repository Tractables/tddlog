"""tpch2provsql.py writes TPC-H programs as SQL; with PROVSQL_PREFIX (the
directory install_provsql.sh installs in) ProvSQL's answers on
tests/lit/tpch.pl and tests/lit/tpch_views.pl are checked against
tpch_exact.py's."""
import importlib.util
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
import tpch2provsql  # noqa: E402

spec = importlib.util.spec_from_file_location('bench', scripts / 'bench.py')
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)

SMALL = """% a small program
0.5::r(1,a).
0.25::r(2,b).
s(a).
0.4::t(1).
q :- r(X,Y), s(Y), t(X), X >= 1.
w(X) :- r(X,_), X \\= 3.
w(X) :- t(X).
query(q).
"""


class TranslateTests(unittest.TestCase):
    def test_tables_views_and_queries(self):
        sql, qs = tpch2provsql.translate(SMALL)
        self.assertIn('CREATE TABLE r (c1 bigint, c2 text, prob float8);', sql)
        self.assertIn('CREATE TABLE s (c1 text);', sql)
        self.assertIn("SELECT add_provenance('r');", sql)
        self.assertNotIn("add_provenance('s')", sql)
        self.assertIn('1\ta\t0.5\n2\tb\t0.25\n', sql)
        self.assertIn('CREATE VIEW q AS SELECT DISTINCT 1 AS c1 FROM r t0, s t1, t t2 WHERE '
                      't1.c1 = t0.c2 AND t2.c1 = t0.c1 AND t0.c1 >= 1;', sql)
        self.assertIn('CREATE VIEW w AS SELECT DISTINCT t0.c1 AS c1 FROM r t0 WHERE t0.c1 <> 3 UNION '
                      'SELECT DISTINCT t0.c1 AS c1 FROM t t0;', sql)
        self.assertEqual(qs, [('q', 'SELECT probability_evaluate(provenance()) FROM q')])

    def test_the_views_program_defines_views_before_their_readers(self):
        text = (root / 'tests' / 'lit' / 'tpch_views.pl').read_text()
        sql, qs = tpch2provsql.translate(text)
        views = re.findall(r'CREATE VIEW (\w+) AS (.*);', sql)
        seen = set(re.findall(r'CREATE TABLE (\w+)', sql))
        for name, body in views:
            for used in re.findall(r'(?:FROM|,)\s+(\w+) t\d', body):
                self.assertIn(used, seen, f'{name} reads {used} before it exists')
            seen.add(name)
        self.assertEqual([q for q, _ in qs], re.findall(r'^query\((\w+)\)\.', text, re.M))


@unittest.skipUnless(os.environ.get('PROVSQL_PREFIX'), 'PROVSQL_PREFIX names the ProvSQL install')
class AnswerTests(unittest.TestCase):
    def check(self, program):
        path = root / 'tests' / 'lit' / program
        ref = subprocess.run([sys.executable, str(scripts / 'tpch_exact.py'), str(path), '--precision', '17'],
                             capture_output=True, text=True, check=True).stdout
        ref = {q.rstrip(':'): float(p) for q, p in (line.split('\t') for line in ref.splitlines())}
        with tempfile.TemporaryDirectory() as d:
            r = bench.run_provsql(SimpleNamespace(dir=d, provsql=os.environ['PROVSQL_PREFIX'], timeout=600), 'x', str(path))
        self.assertEqual(r['status'], 'ok', r)
        for q, p in ref.items():
            self.assertAlmostEqual(r['answers'][q], p, places=9, msg=q)

    def test_flat_program(self):
        self.check('tpch.pl')

    def test_views_program(self):
        self.check('tpch_views.pl')


if __name__ == '__main__':
    unittest.main()
