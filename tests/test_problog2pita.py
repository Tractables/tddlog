"""The ProbLog-to-PITA translation (`scripts/problog2pita.py`)."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import problog2pita  # noqa: E402


class TranslationTests(unittest.TestCase):
    def clauses(self, text):
        lines = problog2pita.program(text).split('\n')
        return lines[lines.index(':- begin_lpad.') + 1:lines.index(':- end_lpad.')]

    def test_a_fact_of_probability_one_is_plain(self):
        # PITA reads `edge(b,c):1.` as false; a plain fact it reads right.
        text = '0.5::edge(a,b).\n1::edge(b,c).\n1.0::edge(c,d).\n2/2::edge(d,e).\n'
        self.assertEqual(self.clauses(text), ['edge(a,b):0.5.', 'edge(b,c).', 'edge(c,d).', 'edge(d,e).'])

    def test_a_rule_of_probability_one_keeps_its_annotation(self):
        text = '0.5::e.\n1::r :- e.\n0.4::s ; 0.6::t :- e.\n'
        self.assertEqual(self.clauses(text), ['e:0.5.', 'r:1 :- e.', 's:0.4 ; t:0.6 :- e.'])

    def test_the_predicates_on_a_cycle_are_tabled(self):
        # p and q reach each other, r itself; s reads them and t is read only
        text = ('0.5::t.\np :- q.\nq :- p.\nq :- t.\nr :- r, t.\nr :- t.\ns :- p, r.\n')
        self.assertEqual(problog2pita.recursive(
            [(problog2pita.functor(h), problog2pita.body_functors(b))
             for h, b in (problog2pita.split_neck(c.strip()) for c in text.split('.\n') if c.strip()) if b]),
            {'p/0', 'q/0', 'r/0'})
        self.assertEqual([c for c in problog2pita.program(text).split('\n') if c.startswith(':- table')],
                         [':- table p/0.', ':- table q/0.', ':- table r/0.'])

    def test_a_name_pita_keeps_is_renamed(self):
        # PITA's own v/3 is a program's v/1 (Roulette's HMM programs), and
        # db/1 is PITA's call to plain Prolog (its caesar programs' bits)
        text = '0.5::v(1).\n0.4::db(1,one).\nq :- v(1), db(1,one), \\+ vv.\nquery(q).\nquery(v(1)).\n'
        self.assertEqual(self.clauses(text), ['problog2pita_v(1):0.5.', 'problog2pita_db(1,problog2pita_one):0.4.',
                                              'q :- problog2pita_v(1), problog2pita_db(1,problog2pita_one), \\+ vv.'])
        main = problog2pita.program(text, driver=True).split('main :-')[1]
        self.assertIn('Q = problog2pita_v(1)', main)
        self.assertIn("atomic_list_concat(Qs, 'problog2pita_', Q0)", main)
        # a program without such a name prints its queries as before
        self.assertIn('format("~q:\\t~w~n", [Q, P])', problog2pita.program('0.5::a.\nquery(a).\n', driver=True))

    def test_a_long_chain_is_translated_in_linear_time(self):
        # a gate a predicate, as a circuit's program has them: the earlier
        # search from every predicate took quadratic time
        n = 200000
        text = '0.5::g0.\n' + ''.join(f'g{i} :- g{i - 1}.\n' for i in range(1, n)) + f'g{n} :- g{n - 1}, g{n}.\n'
        self.assertEqual(problog2pita.recursive(
            [(f'g{i}/0', [f'g{i - 1}/0']) for i in range(1, n)] + [(f'g{n}/0', [f'g{n - 1}/0', f'g{n}/0'])]),
            {f'g{n}/0'})
        self.assertIn(f':- table g{n}/0.', problog2pita.program(text))


if __name__ == '__main__':
    unittest.main()
