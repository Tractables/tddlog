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


if __name__ == '__main__':
    unittest.main()
