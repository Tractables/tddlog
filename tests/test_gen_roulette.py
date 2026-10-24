"""scripts/gen_roulette.py: the scaling programs of Roulette's evaluation
(data/roulette/README.md) as ProbLog and Dice programs, and their answers.

The reference answers (a dynamic program over each model's states) are
checked against an exact enumeration of the Dice program the script
writes for the same size (tests/test_dice2problog.py's `exact`), and
against Roulette's own answers, printed in full by the drivers in
data/roulette/; the committed programs in tests/roulette/ against what
the script writes now. tests/it/examples.rs checks tddlog's answers on those
programs against the same references.
"""
from pathlib import Path
import math
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
from source_tree import SOURCE  # noqa: E402
programs = SOURCE / 'tests' / 'roulette'
sys.path.insert(0, str(root / 'tests'))
import gen_roulette  # noqa: E402
from test_dice2problog import exact  # noqa: E402

# Sizes whose Dice programs the enumeration evaluates in a few seconds (it
# keeps no state between a loop's steps, so its time grows exponentially).
SIZES = [('motivating', 1), ('motivating', 3), ('motivating', 6), ('diamond', 3), ('diamond', 5),
         ('ladder', 3), ('ladder', 5), ('caesar', 1), ('caesar', 2), ('caesar', 3), ('network', 1),
         ('network', 2), ('network', 3), ('hardware', 1), ('hardware', 2), ('hardware', 3), ('hardware', 4)] + \
        [(f'hmm-{q}', n) for q in gen_roulette.HMM_QUERIES for n in (1, 3, 4, 5)]

# Roulette's answers (the artifact's programs through data/roulette/'s
# drivers, Racket 8.16, rsdd backend), as it printed them.
ROULETTE = {
    ('ladder', 7): {'o1(7)': 0.4980033718767183, 'o2(7)': 0.49800337187671806},
    ('hardware', 5): {'res(0)': 0.00010001999899848953, 'res(1)': 9.989007097339569e-5,
                      'res(2)': 9.990005098448052e-5, 'res(3)': 0.9997000699750082,
                      'res(4)': 1.1990403549192487e-7},
    ('network', 5): {'m(5,0)': 2.900390625000001e-10, 'm(5,7)': 0.006003916488696291,
                     'm(5,15)': 0.9510506940708986, 'm(5,16)': 0.024751246878125025},
    ('hmm-smoothing-joint', 4): {'q': 0.19408598591176313},
    # caesar.rkt --no-complement: (query e #f).
    ('caesar', 5): {'key(0)': 0.005887296724512673, 'key(3)': 0.513273152628699,
                    'key(4)': 0.18778374138033535},
    ('caesar', 30): {'key(0)': 0.9999078156176907, 'key(3)': 6.786794063326315e-14,
                     'key(4)': 5.706584496754692e-9},
}
# Roulette's default query computes one value as one minus the others
# (`#:complement-of`), which loses that value once the evidence is below
# about 1e-16: on caesar 30 (evidence about 1e-49) it printed key(0) as 0.
ROULETTE_COMPLEMENT = {('caesar', 5): {'key(0)': 0.005887305638263181},
                       ('caesar', 30): {'key(0)': 0.0}}


def dice_answers(family, n, dist):
    """The reference's atoms from the distribution of the Dice program's
    value: a Boolean for one query, the ladder's pair, an integer whose
    value k is the k-th query."""
    def pr(v):
        return float(sum(p for value, p in dist.items() if value == v))
    if family in ('motivating', 'diamond') or family.startswith('hmm-'):
        atom = {'motivating': f'x({n})', 'diamond': f's({n})'}.get(family, 'q')
        return {atom: pr(True)}
    if family == 'ladder':
        return {f'o1({n})': pr((True, False)), f'o2({n})': pr((False, True))}
    if family == 'caesar':
        return {f'key({k})': pr((k, 64)) for k in range(26)}
    if family == 'network':
        return {f'm({n},{v})': pr((v, 32)) for v in range(17)}
    if family == 'hardware':
        return {f'res({t})': pr((t, 8)) for t in range(5)}
    raise AssertionError(family)


def reference(family, n):
    return {a: math.exp(x.l) for a, x in gen_roulette.reference(family, n).items() if a != 'P(e)'}


def run(*args):
    return subprocess.run([sys.executable, str(scripts / 'gen_roulette.py'), *args], capture_output=True,
                          text=True)


class GenRouletteTests(unittest.TestCase):
    def close(self, what, got, want):
        self.assertTrue(math.isclose(got, want, rel_tol=1e-9, abs_tol=1e-15), f'{what}: {got} against {want}')

    def test_references_agree_with_the_dice_programs(self):
        for family, n in SIZES:
            ref = reference(family, n)
            dice = dice_answers(family, n, exact('\n'.join(gen_roulette.program(family, n)[1])))
            self.assertEqual(set(dice), set(ref), (family, n))
            for atom, p in dice.items():
                self.close(f'{family} {n} {atom}', p, ref[atom])

    def test_references_agree_with_roulette(self):
        for (family, n), answers in ROULETTE.items():
            ref = reference(family, n)
            for atom, p in answers.items():
                self.close(f'{family} {n} {atom}', p, ref[atom])
        for (family, n), answers in ROULETTE_COMPLEMENT.items():
            ref = reference(family, n)
            for atom, p in answers.items():
                self.assertFalse(math.isclose(p, ref[atom], rel_tol=1e-9), (family, n, atom))
                self.assertTrue(math.isclose(p, ref[atom], rel_tol=1e-5, abs_tol=1.0), (family, n, atom))

    def test_the_queries_are_the_reference_atoms_in_order(self):
        for family in gen_roulette.FAMILIES:
            n = 4
            pl, _ = gen_roulette.program(family, n)
            queries = [line[len('query('):-2] for line in pl if line.startswith('query(')]
            self.assertEqual(queries, [a for a in gen_roulette.reference(family, n) if a != 'P(e)'], family)

    def test_the_committed_programs_are_what_the_script_writes(self):
        files = sorted(programs.glob('*.pl'))
        self.assertEqual({f.read_text().split()[2] for f in files}, set(gen_roulette.FAMILIES))
        for f in files:
            _, _, family, size = f.read_text().split('\n', 1)[0].split()
            self.assertEqual(f.name, f'{family}-{size}.pl')
            out = run(family, size)
            self.assertEqual(out.returncode, 0, out.stderr)
            self.assertEqual(out.stdout, f.read_text(), f.name)
            ref = run('--reference-of', str(f))
            self.assertEqual(ref.stdout, run(family, size, '--reference').stdout, f.name)

    def test_the_evidence_probability(self):
        # P(e) of the conditional queries is the marginal likelihood's
        # answer; the caesar keys sum to one.
        for q in gen_roulette.HMM_QUERIES[1:]:
            self.assertEqual(gen_roulette.reference(f'hmm-{q}', 7)['P(e)'].l,
                             gen_roulette.reference('hmm-marginal-likelihood', 7)['q'].l)
        self.close('caesar keys', sum(reference('caesar', 40).values()), 1.0)

    def test_answers_below_the_doubles_print_with_their_exponent(self):
        out = run('hmm-marginal-likelihood', '1000', '--reference')
        self.assertEqual(out.stdout, 'q: 2.66318174128589e-346\n')
        # The marginal likelihood, a product of N factors in (0.3, 0.65).
        lq = gen_roulette.reference('hmm-marginal-likelihood', 1000)['q'].l
        self.assertTrue(1000 * math.log(0.3) < lq < 1000 * math.log(0.65))

    def test_inert_options_fail(self):
        with open(programs / 'diamond-3.pl') as f:
            self.assertTrue(f.readline().startswith(gen_roulette.HEADER))
        for args in (['diamond', '3', '--reference', '--dice', 'x.dice'], ['diamond', '0'], ['diamond'],
                     ['--reference-of', str(programs / 'diamond-3.pl'), 'diamond', '3'],
                     ['--reference-of', str(SOURCE / 'tests' / 'dice' / 'diamond.pl')]):
            out = run(*args)
            self.assertNotEqual(out.returncode, 0, args)
            self.assertFalse(Path('x.dice').exists())


if __name__ == '__main__':
    unittest.main()
