"""tpch2provsql.py writes TPC-H programs, the lit suite's probabilistic
databases (pdb) and its noisy-or networks (noisyor, whose findings are
evidence) as SQL; with PROVSQL_PREFIX (the directory install_provsql.sh
installs in) ProvSQL's answers on tests/lit/tpch.pl and
tests/lit/tpch_views.pl are checked against tpch_exact.py's, and on
tests/lit/pdb.pl, tests/regress/pdb40.pl and tests/lit/noisyor.pl against
lit_exact.py's."""
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
from source_tree import SOURCE  # noqa: E402
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

EVIDENCE = """% a noisy-or of two causes, one finding observed true and one false
0.3::d(x).
0.6::d(y).
0.5::c(x,f).
0.5::c(y,g).
f(F) :- d(D), c(D,F).
evidence(f(f),true).
evidence(f(g),false).
query(d(x)).
query(d(y)).
"""


class TranslateTests(unittest.TestCase):
    def test_tables_views_and_queries(self):
        sql, qs, ev = tpch2provsql.translate(SMALL)
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
        self.assertIsNone(ev)

    def test_the_views_program_defines_views_before_their_readers(self):
        text = (SOURCE / 'tests' / 'lit' / 'tpch_views.pl').read_text()
        sql, qs, _ = tpch2provsql.translate(text)
        views = re.findall(r'CREATE VIEW (\w+) AS (.*);', sql)
        seen = set(re.findall(r'CREATE TABLE (\w+)', sql))
        for name, body in views:
            for used in re.findall(r'(?:FROM|,)\s+(\w+) t\d', body):
                self.assertIn(used, seen, f'{name} reads {used} before it exists')
            seen.add(name)
        self.assertEqual([q for q, _ in qs], re.findall(r'^query\((\w+)\)\.', text, re.M))

    def test_evidence_is_a_conjunction_less_the_false_atoms(self):
        sql, qs, ev = tpch2provsql.translate(EVIDENCE)
        self.assertIn("CREATE VIEW pq_atom0 AS SELECT DISTINCT 1 AS c1 FROM f t0 WHERE t0.c1 = 'f';", sql)
        self.assertIn("CREATE VIEW pq_atom1 AS SELECT DISTINCT 1 AS c1 FROM f t0 WHERE t0.c1 = 'g';", sql)
        self.assertIn('CREATE VIEW pq_evfalse AS SELECT c1 FROM pq_atom1;', sql)
        self.assertIn('CREATE VIEW pq_ev AS SELECT DISTINCT 1 AS c1 FROM pq_atom0 e0 EXCEPT SELECT c1 FROM pq_evfalse;', sql)
        self.assertIn("CREATE VIEW pq_atom2 AS SELECT DISTINCT 1 AS c1 FROM d t0 WHERE t0.c1 = 'x';", sql)
        self.assertIn('CREATE VIEW pq_q0 AS SELECT DISTINCT 1 AS c1 FROM pq_atom2 e0, pq_atom0 e1 '
                      'EXCEPT SELECT c1 FROM pq_evfalse;', sql)
        self.assertEqual([q for q, _ in qs], ['d(x)', 'd(y)'])
        self.assertEqual(ev, 'SELECT probability_evaluate(provenance()) FROM pq_ev')

    def test_the_lit_programs_translate(self):
        for name in ('pdb.pl', 'noisyor.pl'):
            with self.subTest(name=name):
                text = (SOURCE / 'tests' / 'lit' / name).read_text()
                sql, qs, ev = tpch2provsql.translate(text)
                self.assertEqual([q for q, _ in qs], [q.replace(' ', '') for q in
                                                      re.findall(r'^query\((.*)\)\.', text, re.M)])
                self.assertEqual(ev is None, 'evidence(' not in text)

    def test_bench_gives_provsql_the_tuple_independent_families(self):
        self.assertEqual(bench.LIT['pdb10'][1][0], 'gen_pdb.py')
        self.assertEqual(bench.LIT['noisyor6_9'][1][0], 'gen_noisyor.py')
        self.assertEqual(set(bench.PROVSQL_WRITERS), {'gen_tpch.py', 'gen_pdb.py', 'gen_noisyor.py'})


@unittest.skipUnless(os.environ.get('PROVSQL_PREFIX'), 'PROVSQL_PREFIX names the ProvSQL install')
class AnswerTests(unittest.TestCase):
    def check(self, path, ref):
        with tempfile.TemporaryDirectory() as d:
            r = bench.run_provsql(SimpleNamespace(dir=d, provsql=os.environ['PROVSQL_PREFIX'], timeout=600), 'x', str(path))
        self.assertEqual(r['status'], 'ok', r)
        self.assertEqual(sorted(r['answers']), sorted(ref))
        for q, p in ref.items():
            self.assertAlmostEqual(r['answers'][q], p, places=9, msg=q)

    def check_tpch(self, program):
        path = SOURCE / 'tests' / 'lit' / program
        ref = subprocess.run([sys.executable, str(scripts / 'tpch_exact.py'), str(path), '--precision', '17'],
                             capture_output=True, text=True, check=True).stdout
        self.check(path, {q.rstrip(':'): float(p) for q, p in (line.split('\t') for line in ref.splitlines())})

    def check_lit(self, family, path):
        ref = subprocess.run([sys.executable, str(scripts / 'lit_exact.py'), family, str(path), '--precision', '17'],
                             capture_output=True, text=True, check=True).stdout
        self.check(path, {q.rstrip(':'): float(p) for q, p in (line.split('\t') for line in ref.splitlines())})

    def test_flat_program(self):
        self.check_tpch('tpch.pl')

    def test_views_program(self):
        self.check_tpch('tpch_views.pl')

    def test_probabilistic_databases(self):
        self.check_lit('pdb', SOURCE / 'tests' / 'lit' / 'pdb.pl')
        self.check_lit('pdb', SOURCE / 'tests' / 'regress' / 'pdb40.pl')

    def test_a_noisy_or_network_under_evidence(self):
        self.check_lit('noisyor', SOURCE / 'tests' / 'lit' / 'noisyor.pl')

    def test_evidence_by_hand(self):
        # f(f) needs x and c(x,f): P(e) = 0.15 (1 - 0.6 * 0.5) = 0.105;
        # P(d(x) | e) = 1, P(d(y) | e) = 0.6 * 0.5 / 0.7 = 3/7
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'ev.pl'
            p.write_text(EVIDENCE)
            self.check(p, {'d(x)': 1.0, 'd(y)': 3 / 7})


if __name__ == '__main__':
    unittest.main()
