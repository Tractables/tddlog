"""noDice's purely probabilistic programs (data/nodice) through
scripts/dice2problog.py: every one translates, and on the Survey network
tests/test_dice2problog.py's exact evaluation gives Dice 1.0's answers,
which ProbLog gives on the translation too.
"""
from pathlib import Path
import sys
import unittest

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / 'scripts'))
import source_tree  # noqa: E402,F401  (the source tree's scripts and tests)
sys.path.insert(0, str(root / 'tests'))
import dice2problog  # noqa: E402
from test_dice2problog import answers, exact, queries  # noqa: E402

nodice = root / 'data' / 'nodice'

# program -> its translation's queries with Dice 1.0's answers.
NODICE = {
    'survey': [('t(0)', 0.561833976), ('t(1)', 0.280857252), ('t(2)', 0.157308772)],
}


class NoDiceTests(unittest.TestCase):
    def test_exact_gives_dices_answers(self):
        for name, expected in NODICE.items():
            with self.subTest(name=name):
                text = (nodice / f'{name}.dice').read_text()
                qs = queries(dice2problog.translate(text))
                self.assertEqual(qs, [q for q, _ in expected])
                for (q, x), g in zip(expected, answers(exact(text), qs)):
                    self.assertAlmostEqual(g, x, delta=1e-9, msg=q)

    def test_the_programs_translate(self):
        # Each returns one Boolean (CouponProb, 3sat) or one variable of
        # its network, whose values are the queries.
        names = sorted(p.stem for p in nodice.glob('*.dice'))
        self.assertEqual(len(names), 11)
        for name in names:
            with self.subTest(name=name):
                qs = queries(dice2problog.translate((nodice / f'{name}.dice').read_text()))
                if name.startswith(('coupon', '3sat')):
                    self.assertEqual(len(qs), 1)
                else:
                    self.assertGreater(len(qs), 1)
                    self.assertEqual(len({q.split('(')[0] for q in qs}), 1, qs)


if __name__ == '__main__':
    unittest.main()
