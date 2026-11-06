"""problog2scallop.py writes ProbLog programs in Scallop, first order; with
SCLI (the scli binary install_baselines.sh builds) Scallop's answers on
small programs are checked against hand-computed ones, and on small
programs of the lit suite's eventcalc, liftsep and rgraph against
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
import problog2scallop  # noqa: E402
from problog2scallop import translate, answers  # noqa: E402
from problog2souffle import Refused  # noqa: E402

spec = importlib.util.spec_from_file_location('bench', scripts / 'bench.py')
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)

SCLI = os.environ.get('SCLI', '')
HAVE_SCLI = bool(SCLI) and os.access(SCLI, os.X_OK)

# Each program with its answers by hand, ProbLog's semantics: a
# probabilistic clause draws once per ground instance, every variable bound.
CASES = {
    # a body atom with two proofs: one draw for h(1), P = 0.3 * P(b(1)); a
    # draw per proof of b(1) would give 1 - (1 - 0.15)^2
    'twoproofs': ("""0.5::f(1). 0.5::g(1).
b(X) :- f(X).
b(X) :- g(X).
0.3::h(X) :- b(X).
query(h(1)).
""", {'h(1)': 0.3 * 0.75}),
    # two ground instances of one clause, (1, 1) and (1, 2), that share the
    # tuple a(1) of its first atom: two draws, P = 1 - (1 - 0.3 * 0.5)^2
    'instances2': ("""a(1).
0.5::c(1,1). 0.5::c(1,2).
0.3::h(X) :- a(X), c(X,Y).
query(h(1)).
""", {'h(1)': 1 - 0.85 ** 2}),
    # a probabilistic rule in a recursion, of one positive atom: p(1,3) by
    # e(1,3) or by the draw of the instance (1, 3) of p's rule over q(1,3)
    'recursion': ("""0.5::e(1,2). 0.5::e(2,3). 0.5::e(1,3).
q(X,Y) :- e(X,Y).
q(X,Y) :- e(X,Z), p(Z,Y).
0.8::p(X,Y) :- q(X,Y).
query(p(1,3)). query(p(2,_)).
""", {'p(1,3)': 0.8 * (1 - 0.5 * (1 - 0.25 * 0.8)), 'p(2,3)': 0.4}),
    # a probabilistic rule of no variables in a recursion: a switch fact
    'cycle': ("""0.5::a.
0.6::loop :- a.
0.7::a :- loop.
query(a). query(loop).
""", {'a': 0.5, 'loop': 0.3}),
    # an annotated disjunction whose body reads its own head (PSouffle's
    # translation refuses it): on(2) by start(1) and the first choice of the
    # instance (1, 2), off(2) by its second, on(3) and off(3) after on(2)
    'recursivead': ("""0.5::start(1).
link(1,2). link(2,3).
on(X) :- start(X).
0.6::on(Y); 0.3::off(Y) :- link(X,Y), on(X).
query(on(3)). query(off(3)). query(off(2)).
""", {'on(3)': 0.18, 'off(3)': 0.09, 'off(2)': 0.15}),
    # ground annotated disjunctions with bodies, as a network's program
    'network': ("""0.3::a(t); 0.7::a(f).
0.9::b(t); 0.1::b(f) :- a(t).
0.2::b(t); 0.8::b(f) :- a(f).
evidence(b(t)).
query(a(t)). query(a(f)).
""", {'a(t)': 0.27 / 0.41, 'a(f)': 0.14 / 0.41}),
    # evidence of a negation alone
    'negevidence': ("""0.5::a. 0.5::b.
c :- a, b.
evidence(c, false).
query(a).
""", {'a': 0.25 / 0.75}),
    # annotated disjunctions of facts, a predicate of facts and rules, a
    # name Scallop reserves, negation of probabilistic atoms
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
    # a body of one atom with a wildcard, through a derived relation
    'wildcard': ("""0.5::a(1). 0.5::a(2).
b(X) :- a(X).
q :- b(_).
query(q).
""", {'q': 0.75}),
    # symbols that need quotes, and an argument of integers and symbols
    'symbols': ("""0.5::f('Hello world'). 0.25::f('it''s').
0.5::g(1). 0.5::g(a).
query(f(_)). query(g(_)).
""", {"f('Hello world')": 0.5, "f('it''s')": 0.25, 'g(1)': 0.5, 'g(a)': 0.5}),
    # a probability in exponent notation, which Scallop's grammar does not read: written out
    'small': ("""1e-08::a. 0.5::b.
c :- a.
c :- b.
query(c).
""", {'c': 0.5 + 0.5e-08}),
    # a rule's probability below f32's range, which Scallop reads exactly
    # only as a string (an f32 there is 0); its answer at 1e-50 of it
    'tinyrule': ("""person(a).
1e-50::sick(X) :- person(X).
query(sick(a)).
""", {'sick(a)': 1e-50}),
    # between/3, ProbLog's built-in: Scallop's range where it binds X (of a
    # constant end, and of a variable one, the ProbLog tutorial's
    # bayesian_dataflow), comparisons where p binds it, and one draw per X
    # of a probabilistic rule whose body is between alone
    'between': ("""dom(X) :- between(0,2,X).
n(2).
0.5::a(I,1); 0.5::a(I,0) :- n(N), between(1,N,I).
p(1). p(5).
0.5::q(X) :- p(X), between(0,3,X).
0.3::h(X) :- between(1,2,X).
query(dom(_)). query(a(_,1)). query(q(_)). query(h(_)).
""", {'dom(0)': 1.0, 'dom(1)': 1.0, 'dom(2)': 1.0, 'a(1,1)': 0.5, 'a(2,1)': 0.5, 'q(1)': 0.5,
      'h(1)': 0.3, 'h(2)': 0.3}),
    # not/1, ProbLog's negation as \+ is
    'not': ("""0.5::a. 0.4::b.
c :- a, not(b).
query(c).
""", {'c': 0.3}),
}


class TranslateTests(unittest.TestCase):
    def test_relations_types_and_facts(self):
        scl = translate('0.5::e(1,a).\ne(2,b).\np(X) :- e(X,Y).\nquery(p(_)).\n')
        lines = scl.splitlines()
        self.assertIn('type e(i64, String)', lines)
        self.assertIn('type p(i64)', lines)
        self.assertIn('rel e = {', lines)
        self.assertIn('  0.5::(1, "a"),', lines)
        self.assertIn('  (2, "b")', lines)
        self.assertIn('rel p(x) = e(x, y)', lines)
        self.assertIn('// open q__0 p', lines)
        self.assertIn('rel q__0(q0) = p(q0)', lines)
        self.assertIn('query q__0', lines)

    def test_a_fact_given_twice_is_a_noisy_or(self):
        self.assertIn('rel 0.75::e()', translate('0.5::e.\n0.5::e.\nquery(e).\n').splitlines())

    def test_a_small_probability_is_written_without_an_exponent(self):
        scl = translate('1e-08::a.\n2.5E-5::b(1).\n0.5::b(2).\nquery(a).\nquery(b(_)).\n').splitlines()
        self.assertIn('rel 0.00000001::a()', scl)
        self.assertIn('  0.000025::(1),', scl)

    def test_reserved_names_are_renamed(self):
        scl = translate('0.5::count(1).\nsum(X) :- count(X).\nquery(sum(1)).\n')
        self.assertIn('rel sum_(x) = count_(x)', scl.splitlines())
        self.assertIn('// query q__0 sum(1)', scl.splitlines())

    def test_scallops_own_probabilistic_rule_where_it_draws_as_problog(self):
        # one positive atom, outside a recursion: a draw for each of its tuples
        scl = translate('person(a).\n0.3::stress(X) :- person(X), X \\= b.\nquery(stress(a)).\n')
        self.assertIn('rel "0.3"::stress(x) = person(x) and x != "b"', scl.splitlines())
        self.assertNotIn('sw__', scl)

    def test_a_draw_over_the_clauses_instances_elsewhere(self):
        scl = translate(CASES['instances2'][0]).splitlines()
        # two atoms: the instances (x, y) are the domain, read with c certain
        self.assertIn('rel dom__0(x, y) = a(x) and c__d(x, y)', scl)
        self.assertIn('rel "0.3"::sw__0(x, y) = dom__0(x, y)', scl)
        self.assertIn('rel h(x) = sw__0(x, y) and a(x) and c(x, y)', scl)
        self.assertIn('rel c__d = {', scl)
        self.assertIn('  (1, 1),', scl)
        self.assertNotIn('a__d', '\n'.join(scl))    # a is certain: its domain is itself

    def test_a_recursive_rule_draws_over_the_copies(self):
        scl = translate(CASES['recursion'][0]).splitlines()
        self.assertIn('rel dom__0(x, y) = q__d(x, y)', scl)
        self.assertIn('rel q__d(x, y) = e__d(x, y)', scl)
        self.assertIn('rel q__d(x, y) = e__d(x, z) and p__d(z, y)', scl)
        self.assertIn('rel p__d(x, y) = q__d(x, y)', scl)
        self.assertIn('rel p(x, y) = sw__0(x, y) and q(x, y)', scl)

    def test_a_clause_of_no_variables_draws_a_switch_fact(self):
        scl = translate(CASES['cycle'][0]).splitlines()
        self.assertIn('rel loop() = a() and choice__(0, 1)', scl)
        self.assertIn('rel a() = loop() and choice__(1, 1)', scl)
        self.assertIn('  0.6::(0, 1),', scl)

    def test_annotated_disjunctions(self):
        scl = translate('0.5::c(a); 0.25::c(b); 0.25::c(d).\nquery(c(_)).\n').splitlines()
        # the choices: 0.5, then 0.25 of the remaining 0.5; the last is certain
        self.assertIn('  0.5::(0, 1),', scl)
        self.assertIn('  0.5::(0, 2)', scl)
        self.assertIn('rel c(x0) = ad__0_a1(id, x0) and ad__0_c(id, 1)', scl)
        self.assertIn('rel c(x0) = ad__0_a3(id, x0) and not ad__0_c(id, 1) and not ad__0_c(id, 2)', scl)
        scl = translate('b(1).\n0.4::d(X,a); 0.6::d(X,b) :- b(X).\nquery(d(_,_)).\n').splitlines()
        self.assertIn('rel "0.4"::ch__0(1, x) = dom__0(x)', scl)
        self.assertIn('rel d(x, "a") = b(x) and ch__0(1, x)', scl)
        self.assertIn('rel d(x, "b") = b(x) and not ch__0(1, x)', scl)
        scl = translate(CASES['network'][0]).splitlines()
        self.assertIn('rel b("t") = a("t") and choice__(0, 1)', scl)
        self.assertIn('rel b("f") = a("t") and not choice__(0, 1)', scl)

    def test_comparisons_arithmetic_and_negation(self):
        scl = translate('n(1).\nm(J) :- n(I), I =< 3, I \\= 2, \\+(I = 0), J is I * 2 + 1.\nquery(m(_)).\n')
        self.assertIn('rel m(j) = n(i) and i <= 3 and i != 2 and i != 0 and j == ((i * 2) + 1)',
                      scl.splitlines())

    def test_between_is_scallops_range_or_comparisons(self):
        scl = translate('dom(X) :- between(0,5,X).\nquery(dom(_)).\n').splitlines()
        self.assertIn('rel dom(x) = range<i64>(0, 6, x)', scl)
        self.assertFalse([line for line in scl if line.startswith('type range') or line.startswith('type between')])
        # a variable end: its successor a variable of its own; the draws over
        # the clause's instances, between's X among them
        scl = translate('n(3).\n0.5::a(I,1); 0.5::a(I,0) :- n(N), between(1,N,I).\nquery(a(_,1)).\n').splitlines()
        self.assertIn('rel dom__0(i, n, between1) = n(n) and between1 == (n + 1) and range<i64>(1, between1, i)', scl)
        self.assertIn('rel a(i, 1) = n(n) and between1 == (n + 1) and range<i64>(1, between1, i) and '
                      'ch__0(1, i, n, between1)', scl)
        # X bound by another atom: comparisons, and Scallop's own probabilistic rule
        scl = translate('p(1).\n0.5::q(X) :- p(X), between(0,3,X).\nquery(q(_)).\n').splitlines()
        self.assertIn('rel "0.5"::q(x) = p(x) and 0 <= x and x <= 3', scl)
        # between alone: a draw for each X, over a domain
        scl = translate('0.3::h(X) :- between(1,2,X).\nquery(h(_)).\n').splitlines()
        self.assertIn('rel dom__0(x) = range<i64>(1, 3, x)', scl)
        self.assertIn('rel "0.3"::sw__0(x) = dom__0(x)', scl)
        self.assertIn('rel h(x) = sw__0(x) and range<i64>(1, 3, x)', scl)
        # a program that defines between/3 reads its own
        scl = translate('between(1,2,3).\np(X) :- between(1,2,X).\nquery(p(_)).\n').splitlines()
        self.assertIn('rel p(x) = between(1, 2, x)', scl)
        for text in ['q(1).\np(X) :- q(X), \\+ between(0,3,X).\n', 'p(X) :- between(0,H,X).\n']:
            with self.subTest(text=text), self.assertRaisesRegex(Refused, 'between/3'):
                translate(text)

    def test_not_is_negation(self):
        scl = translate('0.5::a. 0.5::b.\nc :- a, not(b).\nquery(c).\n').splitlines()
        self.assertIn('rel c() = a() and not b()', scl)

    def test_evidence(self):
        scl = translate('0.5::a(x).\nb(X) :- a(X).\nevidence(b(x), false).\nquery(a(x)).\n').splitlines()
        self.assertIn('rel e__() = not b("x")', scl)
        self.assertIn('rel qe__0() = a("x") and not b("x")', scl)
        self.assertEqual([line for line in scl if line.startswith('query')], ['query e__', 'query qe__0'])
        self.assertIn('// evidence e__', scl)

    def test_refusals(self):
        for text, reason in [('p([1,2]).\n', '^translator: a list'),
                             ('p(f(1)).\n', '^translator: a function symbol'),
                             ('q(1).\n0.5::p(X) :- q(X) ; q(X).\n', '^translator: a disjunction in the body of a probabilistic'),
                             ('q(1).\np(X+1) :- q(X).\n', '^translator: arithmetic as a term'),
                             ('q(1).\np(Y) :- q(X), Y is X / 2.\n', '^translator: the division'),
                             ('p(X).\n', '^a fact that is not ground'),
                             ('q(1).\np(X) :- q(X), \\+ p(X).\n', '^a negation through recursion'),
                             ('q(a).\np(Y) :- q(X), Y is X + 1.\n', '^arithmetic or an order on symbols'),
                             ('q(1).\np(Y) :- q(X).\n', 'binds'),
                             # a predicate no clause defines, ProbLog's unknown clause, and a
                             # built-in other than between/3, never a relation of no rows
                             ('q(1).\np(X) :- q(X), r(X).\nquery(p(_)).\n', '^translator: r/1, which no clause'),
                             ('q(1).\np(X) :- q(X), \\+ r(X).\nquery(p(_)).\n', '^translator: r/1, which no'),
                             ('q(1).\nquery(r(1)).\n', '^translator: r/1, which no'),
                             ('q(1).\np(Y) :- q(X), succ(X,Y).\nquery(p(_)).\n', '^translator: succ/2, which no')]:
            with self.subTest(text=text), tempfile.TemporaryDirectory() as d:
                src = os.path.join(d, 'p.pl')
                with open(src, 'w') as f:
                    f.write(text)
                r = subprocess.run([sys.executable, str(scripts / 'problog2scallop.py'), src],
                                   capture_output=True, text=True)
                self.assertEqual(r.returncode, problog2scallop.REFUSED, r.stderr)
                self.assertRegex(r.stderr.removeprefix('problog2scallop: '), reason)
        with self.assertRaisesRegex(Refused, 'list'):
            translate('p([1]).\n')

    def test_a_negation_through_recursion_has_one_reason(self):
        # whichever relations close the cycle, the reason is the same text
        for text in ['q(1).\np(X) :- q(X), \\+ p(X).\n',
                     'e(1,2).\nr(X) :- e(X,_), \\+ s(X).\ns(X) :- e(_,X), r(X).\n',
                     '0.5::f(1).\nw(X) :- f(X), \\+ l(X).\nl(X) :- f(X), \\+ w(X).\nquery(w(1)).\n']:
            with self.subTest(text=text):
                with self.assertRaises(Refused) as c:
                    translate(text)
                self.assertEqual(str(c.exception), problog2scallop.NEGATION_CYCLE)

    def test_answers_read_back_as_problog_atoms(self):
        scl = ('// translated\n// query q__0 p(1,a)\n// open q__1 g\n// query q__2 r\n'
               'type p(i64, String)\n')
        out = 'q__0: {0.25::()}\nq__1: {0.5::("a b", 2), 0.125::("x", -1)}\nq__2: {}\n'
        self.assertEqual(answers(out, scl), {'p(1,a)': 0.25, "g('a b',2)": 0.5, 'g(x,-1)': 0.125, 'r': 0.0})
        self.assertEqual(answers('q__0: {1::("it\'s \\"q\\"")}\n', '// open q__0 f\n'),
                         {"f('it''s \"q\"')": 1.0})
        scl = '// query q__0 a\n// evidence e__\n'
        self.assertEqual(answers('e__: {0.5::()}\nqe__0: {0.25::()}\n', scl), {'a': 0.5})
        self.assertEqual(answers('e__: {}\nqe__0: {}\n', scl), 'inconsistent')
        with self.assertRaisesRegex(ValueError, 'no e__'):
            answers('qe__0: {}\n', scl)


@unittest.skipUnless(HAVE_SCLI, 'no Scallop: SCLI names no scli')
class ScallopTests(unittest.TestCase):
    def run_scallop(self, text, d):
        path = os.path.join(d, 'p.pl')
        with open(path, 'w') as f:
            f.write(text)
        args = SimpleNamespace(dir=d, timeout=600, scallop=SCLI)
        r = bench.run_scallop(args, 'p', path, bench.SCALLOP_VARIANT)
        self.assertEqual(r['status'], 'ok', r)
        self.assertEqual(r['stats'], {'scallop_route': 'first-order'})
        return r['answers']

    def test_hand_computed(self):
        # to 1e-12 of each answer: the rules' probabilities are read as
        # doubles, as the facts' are (an f32 is off by about 1e-8)
        for name, (text, want) in CASES.items():
            with self.subTest(name), tempfile.TemporaryDirectory(dir=root / 'tests') as d:
                got = self.run_scallop(text, d)
                self.assertEqual(sorted(got), sorted(want))
                for q, p in want.items():
                    self.assertAlmostEqual(got[q], p, delta=1e-12 * p, msg=q)

    def test_lit_families_against_lit_exact(self):
        for family, gargs in [('liftsep', ['1']), ('rgraph', ['5']), ('eventcalc', ['2', '200', '--every', '50', '--seed', '1'])]:
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
                got = self.run_scallop(text, d)
                self.assertTrue(want)
                for q, p in want.items():
                    self.assertAlmostEqual(got.get(q, 0.0), p, places=9, msg=q)


if __name__ == '__main__':
    unittest.main()
