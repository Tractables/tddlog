"""Bayesian or Markov network, by structure (scripts/uai_types.py), on
small networks in the UAI format."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import uai_types  # noqa: E402


def network(scopes, tables, cards):
    """A UAI file's text with the header MARKOV, as the UAI 2022 sets have."""
    lines = ['MARKOV', str(len(cards)), ' '.join(map(str, cards)), str(len(scopes))]
    lines += [' '.join(map(str, [len(s)] + s)) for s in scopes]
    for t in tables:
        lines += [str(len(t)), ' '.join(map(str, t))]
    return '\n'.join(lines) + '\n'


class Types(unittest.TestCase):
    def kind(self, scopes, tables, cards=(2, 2)):
        return uai_types.kind(*uai_types.parse(network(scopes, tables, list(cards))))

    def test_a_chain_of_tables_is_bayesian(self):
        # P(a) and P(b | a), rows over b summing to 1 (b, the last variable, changes fastest).
        self.assertEqual(self.kind([[0], [0, 1]], [[0.3, 0.7], [0.9, 0.1, 0.2, 0.8]]), ('bayesian', ''))

    def test_a_table_listed_with_its_child_first(self):
        # P(b | a) over the scope (b, a), as Promedus lists its tables: a changes fastest.
        self.assertEqual(self.kind([[0], [1, 0]], [[0.3, 0.7], [0.9, 0.2, 0.1, 0.8]]), ('bayesian', ''))

    def test_children_found_by_an_augmenting_path(self):
        # Factor 0 sums to 1 over either variable, factor 1 only over variable 0,
        # so factor 0 gives up variable 0 and takes variable 1.
        self.assertEqual(self.kind([[0, 1], [0]], [[0.5, 0.5, 0.5, 0.5], [0.3, 0.7]]), ('bayesian', ''))

    def test_rounding_within_the_tolerance(self):
        third = 0.3333333
        self.assertEqual(self.kind([[0]], [[third, third, third]], cards=(3,))[0], 'bayesian')

    def test_rounding_of_each_entry_of_a_longer_row(self):
        # Ihler's diagnose_a: a row of four entries written with five decimals sums to 1.00001.
        table = [0.08824, 0.44118, 0.44118, 0.02941, 0.25, 0.25, 0.25, 0.25]
        self.assertEqual(self.kind([[0], [0, 1]], [[0.3, 0.7], table], cards=(2, 4))[0], 'bayesian')
        off = [0.08824, 0.44118, 0.44118, 0.02944] + table[4:]
        self.assertEqual(self.kind([[0], [0, 1]], [[0.3, 0.7], off], cards=(2, 4))[0], 'markov')

    def test_a_table_that_does_not_sum_to_one(self):
        self.assertEqual(self.kind([[0], [0, 1]], [[0.3, 0.7], [0.9, 0.3, 0.2, 0.8]])[0], 'markov')

    def test_as_many_factors_as_variables_each_with_a_child(self):
        t = [0.5, 0.5]
        self.assertEqual(self.kind([[0], [0], [1]], [t, t, t])[1], '3 factors for 2 variables')
        self.assertEqual(self.kind([[0]], [t])[1], '1 factors for 2 variables')
        self.assertEqual(self.kind([[0], [0]], [t, t])[1], "no table can be variable 1's")

    def test_a_cycle(self):
        t = [0.5, 0.5, 0.5, 0.5]
        self.assertEqual(self.kind([[1, 0], [0, 1]], [t, t]), ('markov', 'the parent graph has a cycle'))

    def test_a_pairwise_markov_network(self):
        # A grid's factors: potentials over pairs, unnormalized.
        self.assertEqual(self.kind([[0, 1], [1]], [[2.0, 1.0, 1.0, 2.0], [1.0, 1.0]])[0], 'markov')

    def test_a_table_of_the_wrong_size(self):
        with self.assertRaises(ValueError):
            uai_types.parse(network([[0, 1]], [[0.5, 0.5]], [2, 2]))


if __name__ == '__main__':
    unittest.main()
