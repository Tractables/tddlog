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
from unittest import mock

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
    # between/3, ProbLog's built-in: an input relation of its rows where it
    # binds X (of constant bounds, and of a bound n(N) states, the ProbLog
    # tutorial's bayesian_dataflow), comparisons where p binds it, and one
    # draw per X of a probabilistic rule whose body is between alone. dom is
    # asked through g: asked itself, a relation whose every tuple is certain,
    # PSouffle printed none of its tuples
    'between': ("""dom(X) :- between(0,2,X).
0.5::g(X) :- dom(X).
n(2).
0.5::a(I,1); 0.5::a(I,0) :- n(N), between(1,N,I).
p(1). p(5).
0.5::q(X) :- p(X), between(0,3,X).
0.3::h(X) :- between(1,2,X).
query(g(_)). query(a(1,1)). query(a(2,1)). query(q(_)). query(h(_)).
""", {'g(0)': 0.5, 'g(1)': 0.5, 'g(2)': 0.5, 'a(1,1)': 0.5, 'a(2,1)': 0.5, 'q(1)': 0.5,
      'h(1)': 0.3, 'h(2)': 0.3}),
    # not/1, ProbLog's negation as \+ is
    'not': ("""0.5::a. 0.4::b.
c :- a, not(b).
query(c).
""", {'c': 0.3}),
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

    def test_runs_of_facts_read_as_the_parser_reads_each(self):
        # the runs of facts read whole (fact_run: windows, a relation's rows
        # at once, the run ending before a fact that is not of plain
        # constants) translate as the facts read token by token do
        import problog2scallop
        many = ''.join(f'0.{k % 9 + 1}::r({k}, a{k % 7}, {k % 5}.5).\n' for k in range(3000))
        programs = {
            'spaces': '0.5 :: f( 1 , a ).\nf(2,b).  0.25::f(-3, c).\n0.5::f(1, a).\nquery(f(1,a)).\n',
            'numbers': 'g(1.5). g(-0.0). g(2e3). g(007). 0.1::g(1.5). g(7).\nq(X) :- g(X).\nquery(q(7)).\n',
            'stops': ("0.5::h(1).\n0.5::h(X) :- k(X).\n0.5::h('a b').\nk(2). % a comment\n"
                      'k(3). /* c */ k(4).\nh(5) :- k(5).\nquery(h(_)).\n'),
            'arities': 'p(1). p(1,2). p(a). 0.5::p(b). 0.3::p(1,2).\nquery(p(1,2)).\n',
            'many': many + '0.5::r(1, a1, 1.5).\nquery(r(1, a1, 1.5)).\n',
            'range': 'f(1). 1.5::f(2).\nquery(f(1)).\n',
            'variable': 'f(1). f(X).\nquery(f(1)).\n',
            'lines': 'f(1,\n 2).\n0.5::f(3,\n4). f(5,6).\nquery(f(3,4)).\n',
            'comment': 'f(1). f(a % x).\n).\nquery(f(a)).\n',
        }
        self.assertGreater(sum(k == 'facts' for k, _ in problog2souffle.tokens(many)), 2)

        def read(module, text):
            try:
                return module.translate(text)
            except Refused as e:
                return 'refused', str(e)

        for name, text in programs.items():
            for module in (problog2souffle, problog2scallop):
                with self.subTest(name, module=module.__name__):
                    fast = read(module, text)
                    with mock.patch.object(problog2souffle, 'FACT_START', re.compile('(?!)')):
                        self.assertEqual(fast, read(module, text))

    def test_probabilities_are_written_without_exponents(self):
        probs = [0.5, 1.2345e-05, 0.000123456, 1e-300, '0.1', 1.0]
        texts = problog2souffle.prob_texts(probs)
        self.assertEqual(texts, [problog2souffle.fmt_prob(p) for p in probs])
        self.assertEqual(texts[:3], ['0.5', '0.000012345', '0.000123456'])
        self.assertTrue(all('e' not in t for t in texts))

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

    def test_a_query_with_a_body_of_between(self):
        # graph's program: the queried relation is output, each of its ground queries asked
        dl, _, _, _, asked = translate('0.6::edge(1,2).\n0.5::edge(2,3).\npath(X,Y) :- edge(X,Y).\n'
                                       'path(X,Y) :- edge(X,Z), path(Z,Y).\n'
                                       'query(path(1,Y)) :- between(2,3,Y).\n')
        self.assertEqual(asked, ['path(1,2)', 'path(1,3)'])
        self.assertIn('query(', dl)
        for text, reason in [('0.5::p(1).\nquery(p(X)) :- between(1,N,X).\n', 'integer bounds'),
                             ('0.5::p(1).\nquery(p(X)) :- node(X).\n', 'no facts')]:
            with self.subTest(text=text), self.assertRaisesRegex(Refused, reason):
                translate(text)

    def test_a_query_with_a_body_of_certain_rules_and_comparisons(self):
        # the Circulette reader's program: gate/1 a rule over the gates' facts, and the
        # pairs of gates ordered by a comparison
        text = ('gate_and(1, 7, 8).\ngate_xor(2, 7, 8).\ngate(G) :- gate_and(G, _, _).\n'
                'gate(G) :- gate_xor(G, _, _).\n0.01::fault(G) :- gate(G).\n'
                'faults(A, B) :- fault(A), fault(B).\n'
                'query(fault(G)) :- gate(G).\nquery(faults(A, B)) :- gate(A), gate(B), A < B.\n')
        _, _, _, _, asked = translate(text)
        self.assertEqual(asked, ['fault(1)', 'fault(2)', 'faults(1,2)'])
        # a body over a relation that is not certain is refused
        with self.assertRaisesRegex(Refused, 'not a relation of certain facts|has no facts'):
            translate('0.5::e(1).\nr(X) :- e(X).\n0.5::p(X) :- r(X).\nquery(p(X)) :- r(X).\n')

    def test_between_is_an_input_relation_or_comparisons(self):
        dl, rows, probs, _, _ = translate('dom(X) :- between(0,2,X).\nquery(dom(_)).\n')
        self.assertIn('dom(X) :- between(0, 2, X).', dl)
        self.assertIn('.input between', dl)
        self.assertEqual((rows['between'], probs['between']), (['0\t2\t0', '0\t2\t1', '0\t2\t2'], ['1.0'] * 3))
        # bounds that certain facts state: the rows of their values
        _, rows, _, _, _ = translate('n(2).\nn(3).\n0.5::a(I,1); 0.5::a(I,0) :- n(N), between(1,N,I).\n'
                                     'query(a(_,1)).\n')
        self.assertEqual(sorted(rows['between']), ['1\t2\t1', '1\t2\t2', '1\t3\t1', '1\t3\t2', '1\t3\t3'])
        # X bound by another atom: comparisons, and no relation
        dl, rows, _, _, _ = translate('p(1).\n0.5::q(X) :- p(X), between(0,3,X).\nquery(q(_)).\n')
        self.assertIn('0.5::q(X) :- p(X), 0 <= X, X <= 3.', dl)
        self.assertNotIn('between', rows)
        # a program that defines between/3 reads its own
        _, rows, _, _, _ = translate('between(1,2,3).\np(X) :- between(1,2,X).\nquery(p(_)).\n')
        self.assertEqual(rows['between'], ['1\t2\t3'])
        for text, reason in [('q(1).\np(X) :- q(X), \\+ between(0,3,X).\n', 'negated between/3'),
                             ('p(X) :- between(0,H,X).\n', 'between/3 whose bounds are not bound'),
                             ('0.5::n(2).\np(X) :- n(N), between(1,N,X).\n', 'between/3 whose bounds are not integers'),
                             ('n(2).\np(X) :- n(N), M is N + 1, between(1,M,X).\n',
                              'between/3 whose bounds are not integers')]:
            with self.subTest(text=text), self.assertRaisesRegex(Refused, reason):
                translate(text)

    def test_not_is_negation(self):
        self.assertIn('c() :- a(), !b().', rules('0.5::a. 0.5::b.\nc :- a, not(b).\nquery(c).\n'))

    def test_a_predicate_no_clause_defines_is_refused(self):
        # ProbLog stops on an unknown clause; a relation of no rows in its place would answer
        for text, name in [('q(1).\np(X) :- q(X), r(X).\nquery(p(_)).\n', 'r/1'),
                           ('q(1).\np(X) :- q(X), \\+ r(X).\nquery(p(_)).\n', 'r/1'),
                           ('q(1).\nquery(r(1)).\n', 'r/1'),
                           ('0.5::q(1).\nevidence(r).\nquery(q(1)).\n', 'r/0'),
                           ('q(1).\np(Y) :- q(X), succ(X,Y).\nquery(p(_)).\n', 'succ/2')]:
            with self.subTest(text=text), self.assertRaisesRegex(Refused, f'^{name}, which no clause defines'):
                translate(text)
        # a relation a clause declares empty is one
        self.assertIn('b(X) :- d(X), !z(X).', rules(CASES['empty'][0]))

    def test_quoted_atoms_are_symbols(self):
        _, rows, _, _, _ = translate("0.5::f('Hello world').\nquery(f(_)).\n")
        self.assertEqual(rows['f'], ['Hello world'])

    def test_a_certain_disjunction_is_a_clause_per_disjunct_and_the_other_neck_is_read(self):
        # ProbLog's tutorial programs: smokers_or's rule and add's `any`, with `<-`
        text = ("0.3::p(1); 0.4::p(2).\nany <- p(1) ; p(2).\nq(X) :- p(X), X > 1 ; p(X), X < 2.\n"
                "query(any). query(q(1)).\n")
        _, rules, _, _, queries, _ = problog2souffle.parse(text)
        self.assertEqual([(prob, head) for prob, head, _ in rules],
                         [(None, ('any', [])), (None, ('any', [])), (None, ('q', [('var', 'X')])),
                          (None, ('q', [('var', 'X')]))])
        self.assertEqual([body for _, _, body in rules][:2], [[('pos', ('p', [('int', 1)]))],
                                                              [('pos', ('p', [('int', 2)]))]])
        self.assertEqual([len(body) for _, _, body in rules][2:], [2, 2])
        self.assertEqual(queries, [('any', []), ('q', [('int', 1)])])

    def test_evidence_on_a_negation_is_its_atom_observed_false(self):
        # smokers_or's `evidence(\+influences(4,2)).`, as ProbLog reads it
        *_, evidence = problog2souffle.parse('0.2::i(4,2).\n0.5::a.\nevidence(\\+i(4,2)).\nevidence(\\+a).\n')
        self.assertEqual(evidence, [(('i', [('int', 4), ('int', 2)]), False), (('a', []), False)])
        with self.assertRaises(problog2souffle.Refused):
            problog2souffle.parse('0.5::a.\nevidence(\\+a, true).\n')

    def test_refusals(self):
        for text, reason in [('p([1,2]).\n', 'a list'),
                             ('p(f(1)).\n', 'function symbol'),
                             ('q(1).\n0.5::p(X) :- q(X) ; q(X).\n', 'disjunction in the body of a probabilistic'),
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
