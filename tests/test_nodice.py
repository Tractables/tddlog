"""noDice's purely probabilistic programs (data/nodice) through
scripts/dice2problog.py: every one translates (a program's two smallest sizes,
translated()), and on the Survey network
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
import bench  # noqa: E402
import dice2problog  # noqa: E402
from test_dice2problog import answers, exact, queries  # noqa: E402

nodice = root / 'data' / 'nodice'


def translated(k=2):
    """The programs the translation tests take: bench.NODICE but the sizes
    of a program (bench.series) past its k smallest. The larger sizes
    (diamond-10000-p, brp16384-2) are the same text at other constants, and
    their symbolic evaluation took the unit tests past 20 minutes; the
    suite's runs read all of them."""
    seen, names = {}, []
    for name in bench.NODICE:
        s = bench.series('nodice', f'nodice_{name}')
        seen[s] = seen.get(s, 0) + 1
        if s is None or seen[s] <= k:
            names.append(name)
    return names


def dice_programs():
    """The Dice programs every translator's tests take (test_dice2*.py):
    data/dice's and noDice's of translated()."""
    return sorted((root / 'data' / 'dice').glob('*.dice')) + [nodice / f'{n}.dice' for n in translated()]

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
        # Each returns one Boolean (CouponProb, 3sat and the uf formulas,
        # brp, diamond), a ladder's pair of Booleans, or one variable of
        # its network, whose values are the queries.
        names = sorted(p.stem for p in nodice.glob('*.dice'))
        self.assertEqual(len(names), 69)
        self.assertEqual(names, sorted(bench.NODICE))
        for name in translated():
            with self.subTest(name=name):
                qs = queries(dice2problog.translate((nodice / f'{name}.dice').read_text()))
                if name.startswith(('coupon', '3sat', 'uf', 'brp', 'diamond')):
                    self.assertEqual(len(qs), 1)
                elif name.startswith('ladder'):
                    self.assertEqual(len(qs), 2)
                else:
                    self.assertGreater(len(qs), 1)
                    self.assertEqual(len({q.split('(')[0] for q in qs}), 1, qs)


if __name__ == '__main__':
    unittest.main()
