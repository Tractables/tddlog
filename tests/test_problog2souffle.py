"""problog2souffle.py writes ProbLog programs in PSouffle's probabilistic
Datalog; with PSOUFFLE (the souffle binary install_psouffle.sh builds)
PSouffle's answers on small programs are checked against hand-computed
ones, and on small programs of the lit suite's eventcalc, liftsep and
rgraph against lit_exact.py's."""
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
import problog2souffle  # noqa: E402
from problog2souffle import Refused, translate  # noqa: E402

spec = importlib.util.spec_from_file_location('bench', scripts / 'bench.py')
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)

PSOUFFLE = os.environ.get('PSOUFFLE')
PRINTED = 1e-8  # PSouffle prints its probabilities with eight decimals

# Each program with its answers by hand.
CASES = {
    # annotated disjunctions of facts, a predicate of facts and rules, a
    # name Souffle reserves, negation of probabilistic atoms
    'ads': ("""0.3::c(1); 0.5::c(2).
0.4::count(1).
count(X) :- c(X).
none :- \\+ c(1), \\+ c(2).
query(c(1)). query(c(2)). query(count(1)). query(none).
""", {'c(1)': 0.3, 'c(2)': 0.5, 'count(1)': 1 - 0.6 * 0.7, 'none': 0.2}),
    # a probabilistic rule and an annotated disjunction with a body: a choice
    # for each ground instance of the clause, its body's variables bound
    'instances': ("""b(1,1). b(1,2).
0.5::h(X) :- b(X,_).
0.2::d(X,a); 0.3::d(X,b) :- b(X,Y).
query(h(1)). query(d(1,a)). query(d(1,b)).
""", {'h(1)': 0.75, 'd(1,a)': 1 - 0.8 ** 2, 'd(1,b)': 1 - 0.7 ** 2}),
    # arithmetic and comparisons, a query with a variable
    'arith': ("""0.5::step(0). 0.5::step(1). 0.5::step(2).
r(0).
r(J) :- r(I), step(I), I < 3, J is I + 1.
query(r(_)).
""", {'r(0)': 1.0, 'r(1)': 0.5, 'r(2)': 0.25, 'r(3)': 0.125}),
    # evidence, and facts of no arguments
    'evidence': ("""0.5::a. 0.5::b.
c :- a.
c :- b.
evidence(c, true).
query(a).
""", {'a': 0.5 / 0.75}),
    # a deterministic relation with no facts, negated
    'empty': ("""0.25::d(1).
z(_) :- fail.
b(X) :- d(X), \\+ z(X).
query(b(1)).
""", {'b(1)': 0.25}),
    # a query with a body over certain facts, as the smokers programs ask
    'querybody': ("""person(a). person(b).
0.3::smokes(X) :- person(X).
query(smokes(X)) :- person(X).
""", {'smokes(a)': 0.3, 'smokes(b)': 0.3}),
    # a body of one atom with a wildcard, through a derived relation, as
    # TPC-H's views ask (PSouffle's --rewrite answered 0.5)
    'wildcard': ("""0.5::a(1). 0.5::a(2).
b(X) :- a(X).
q :- b(_).
query(q).
""", {'q': 0.75}),
}


def rules(text):
    return translate(text)[0]


class TranslateTests(unittest.TestCase):
    def test_facts_are_rows_of_input_relations(self):
        dl, rows, probs, table, _ = translate('0.5::e(1,a).\ne(2,b).\np(X) :- e(X,Y).\nquery(p(_)).\n')
        self.assertIn('.decl e(a0:number, a1:symbol)', dl)
        self.assertIn('.input e', dl)
        self.assertEqual(rows['e'], ['1\ta', '2\tb'])
        self.assertEqual(probs['e'], ['0.5', '1.0'])
        self.assertIn('p(X) :- e(X, Y).', dl)
        self.assertIn('query(p(_)).', dl)
        self.assertIn(('p', 'p', 1), table)

    def test_a_fact_given_twice_is_a_noisy_or(self):
        _, rows, probs, _, _ = translate('0.5::e(1).\n0.5::e(1).\nquery(e(1)).\n')
        self.assertEqual((rows['e'], probs['e']), (['1'], ['0.75']))

    def test_facts_and_rules_of_one_predicate(self):
        dl, rows, _, _, _ = translate('0.2::r(1).\n0.5::r(X) :- q(X).\nq(2).\nquery(r(_)).\n')
        self.assertIn('.input r__f', dl)
        self.assertIn('r(x0) :- r__f(x0).', dl)
        self.assertIn('0.5::r(X) :- q(X).', dl)
        self.assertNotIn('.input r\n', dl)

    def test_a_fact_of_no_arguments(self):
        dl, rows, probs, _, _ = translate('0.3::h.\nq :- h.\nquery(q).\n')
        self.assertIn('h() :- h__f(0).', dl)
        self.assertEqual((rows['h__f'], probs['h__f']), (['0'], ['0.3']))
        self.assertIn('q() :- h().', dl)
        self.assertIn('query(q()).', dl)

    def test_reserved_names_are_renamed(self):
        dl, _, _, table, _ = translate('0.5::count(1).\nsum(X) :- count(X).\nquery(sum(1)).\n')
        self.assertIn('sum_(X) :- count_(X).', dl)
        self.assertIn(('count_', 'count', 1), table)

    def test_an_annotated_disjunction_of_facts_chooses_in_order(self):
        dl, rows, probs, _, _ = translate('0.2::c(a); 0.3::c(b); 0.5::c(d).\nquery(c(_)).\n')
        # the choices: 0.2, then 0.3 of the remaining 0.8; the last is certain
        self.assertEqual([round(float(p), 12) for p in probs['ad0_c']], [0.2, 0.375])
        self.assertIn('c(x0) :- ad0_a1(id, x0), ad0_c(id, 1).', dl)
        self.assertIn('c(x0) :- ad0_a3(id, x0), !ad0_c(id, 1), !ad0_c(id, 2).', dl)

    def test_an_annotated_disjunction_with_a_body(self):
        dl = rules('b(1).\n0.4::d(X,a); 0.6::d(X,b) :- b(X).\nquery(d(_,_)).\n')
        self.assertIn('0.4::ad0_c(1, X) :- b(X).', dl)
        self.assertIn('d(X, "a") :- b(X), ad0_c(1, X).', dl)
        self.assertIn('d(X, "b") :- b(X), !ad0_c(1, X).', dl)

    def test_a_probabilistic_clause_names_its_anonymous_variables(self):
        self.assertIn('0.5::h(X) :- b(X, u_A1).', rules('b(1,1).\n0.5::h(X) :- b(X,_).\nquery(h(_)).\n'))
        self.assertIn('h(X) :- b(X, _).', rules('b(1,1).\nh(X) :- b(X,_).\nquery(h(_)).\n'))

    def test_comparisons_arithmetic_and_negation(self):
        dl = rules('n(1).\nm(J) :- n(I), I =< 3, I \\= 2, \\+(I = 0), J is I * 2 + 1.\nquery(m(_)).\n')
        self.assertIn('m(J) :- n(I), I <= 3, I != 2, I != 0, J = ((I * 2) + 1).', dl)

    def test_evidence_and_ground_queries(self):
        dl = rules('0.5::a(x).\nb(X) :- a(X).\nevidence(b(x), false).\nquery(a(x)).\n')
        self.assertIn('evidence(b("x"), false).', dl)
        self.assertIn('query(a("x")).', dl)

    def test_several_ground_queries_of_one_relation(self):
        # PSouffle names its code for a query after the relation: a second atom of it is
        # asked through a relation of its own
        dl, _, _, table, asked = translate('0.5::e(1,2).\n0.5::e(2,3).\np(X,Y) :- e(X,Y).\n'
                                           'query(p(1,2)).\nquery(p(2,3)).\n')
        self.assertIn('q__0() :- p(1, 2).', dl)
        self.assertIn('q__1() :- p(2, 3).', dl)
        self.assertIn('query(q__0()).', dl)
        self.assertIn('.decl q__1()', dl)
        self.assertIn(('q__1', 'p(2,3)', 0), table)
        self.assertEqual(asked, ['p(1,2)', 'p(2,3)'])

    def test_a_query_with_a_body_of_certain_facts(self):
        _, _, _, _, asked = translate('person(a).\nperson(b).\n0.3::smokes(X) :- person(X).\n'
                                      'query(smokes(X)) :- person(X).\n')
        self.assertEqual(asked, ['smokes(a)', 'smokes(b)'])

    def test_quoted_atoms_are_symbols(self):
        _, rows, _, _, _ = translate("0.5::f('Hello world').\nquery(f(_)).\n")
        self.assertEqual(rows['f'], ['Hello world'])

    def test_refusals(self):
        for text, reason in [('p([1,2]).\n', 'a list'),
                             ('p(f(1)).\n', 'function symbol'),
                             ('p(X) :- q(X) ; r(X).\n', 'disjunction'),
                             (':- use_module(library(lists)).\n', 'directive'),
                             ('p(X).\n', 'not ground'),
                             ('q(1).\np(X) :- q(X), \\+ p(X).\n', 'recursion'),
                             ('q(1).\np(Y) :- q(X), Y is X / 2.\n', 'division'),
                             ('q(a).\np(Y) :- q(X), Y is X + 1.\n', 'symbols'),
                             ('q(1).\np(Y) :- q(X).\n', 'binds')]:
            with self.subTest(text=text), self.assertRaisesRegex(Refused, reason):
                translate(text)

    def test_the_command_exits_refused(self):
        with tempfile.TemporaryDirectory() as d:
            src = os.path.join(d, 'p.pl')
            with open(src, 'w') as f:
                f.write('p([1]).\n')
            r = subprocess.run([sys.executable, str(scripts / 'problog2souffle.py'), src, os.path.join(d, 'out')],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, problog2souffle.REFUSED, r.stderr)

    def test_answers_read_back_as_problog_atoms(self):
        names = 'count_\tcount\t1\nq_\tq\t0\ng\tg\t2\n'
        out = 'count_(3) : 0.25\nq_() : 1\ng("a b",2) : 0.5\nignored line\n'
        self.assertEqual(problog2souffle.answers(out, names), {'count(3)': 0.25, 'q': 1.0, 'g(a b,2)': 0.5})
        self.assertEqual(problog2souffle.answers('q__0() : 0.125\n', 'q__0\tp(1,2)\t0\n'), {'p(1,2)': 0.125})

    def test_bench_matches_queries_with_variables(self):
        self.assertIsNone(bench.query_pattern('p(1,a)'))
        self.assertIsNone(bench.query_pattern('qw'))
        pattern = bench.query_pattern('pt(_,X,3)')
        self.assertTrue(pattern.fullmatch('pt(1,2,3)'))
        self.assertFalse(pattern.fullmatch('pt(1,2,4)'))


@unittest.skipUnless(PSOUFFLE, 'set PSOUFFLE to the souffle binary install_psouffle.sh builds')
class PSouffleTests(unittest.TestCase):
    def run_psouffle(self, text, d):
        path = os.path.join(d, 'p.pl')
        with open(path, 'w') as f:
            f.write(text)
        args = SimpleNamespace(dir=d, timeout=600, psouffle=PSOUFFLE)
        r = bench.run_psouffle(args, 'p', path)
        self.assertEqual(r['status'], 'ok', r)
        return r['answers']

    def test_hand_computed(self):
        for name, (text, want) in CASES.items():
            with self.subTest(name), tempfile.TemporaryDirectory(dir=root / 'tests') as d:
                got = self.run_psouffle(text, d)
                self.assertEqual(sorted(got), sorted(want))
                for q, p in want.items():
                    self.assertAlmostEqual(got[q], p, delta=PRINTED, msg=q)

    def test_lit_families_against_lit_exact(self):
        for family, gargs in [('liftsep', ['2']), ('rgraph', ['5']), ('eventcalc', ['2', '200', '--every', '50', '--seed', '1'])]:
            with self.subTest(family), tempfile.TemporaryDirectory(dir=root / 'tests') as d:
                text = subprocess.run([sys.executable, str(bench.source_tree.script(f'gen_{family}.py')), *gargs],
                                      capture_output=True, text=True, check=True).stdout
                path = os.path.join(d, 'p.pl')
                with open(path, 'w') as f:
                    f.write(text)
                exact = subprocess.run([sys.executable, str(bench.source_tree.script('lit_exact.py')), family, path,
                                        '--precision', '17'], capture_output=True, text=True, check=True).stdout
                want = {re.sub(r'\s+', '', q): float(p) for q, p in
                        (line.rsplit(':', 1) for line in exact.splitlines() if ':' in line)}
                got = self.run_psouffle(text, d)
                self.assertTrue(want)
                for q, p in want.items():
                    self.assertAlmostEqual(got.get(q, 0.0), p, delta=PRINTED, msg=q)


if __name__ == '__main__':
    unittest.main()
