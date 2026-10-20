"""bn_exact.py against answers computed by hand."""
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts' / 'bn_exact.py'


def run(program, *flags):
    # Scratch files go in the repository's tmp/, which git ignores.
    scratch = ROOT / 'tmp'
    scratch.mkdir(exist_ok=True)
    with tempfile.NamedTemporaryFile('w', suffix='.pl', dir=scratch) as f:
        f.write(program)
        f.flush()
        return subprocess.run([sys.executable, str(SCRIPT), *flags, f.name],
                              capture_output=True, text=True, check=True).stdout


def answers(program):
    out = run(program, '--precision', '17')
    if out.strip() == 'evidence is inconsistent':
        return 'inconsistent'
    return {q.strip(): float(p) for q, p in (line.rsplit(':', 1) for line in out.splitlines())}


def evidence_probability(program, digits=17):
    """The last line `--evidence-probability` prints, split at its tabs."""
    line = run(program, '--precision', str(digits), '--evidence-probability').splitlines()[-1]
    label, p, log10 = line.split('\t')
    if label != 'P(e):' or not log10.startswith('log10 '):
        raise AssertionError(f'not an evidence line: {line!r}')
    return p, float(log10.removeprefix('log10 '))


# `a` has no value with probability 0.2, `b` with 0.1 more under `a(y)` and
# whenever `a` has none, and `c` has a row for `a(x)` alone.
SHORT = '''0.3::a(x); 0.5::a(y).
0.9::b(t); 0.1::b(f) :- a(x).
0.2::b(t); 0.7::b(f) :- a(y).
c(u) :- a(x).
'''

DIAMOND = '''0.6::a(t); 0.4::a(f).
0.7::b(t); 0.3::b(f) :- a(t).
0.2::b(t); 0.8::b(f) :- a(f).
0.1::c(t); 0.9::c(f) :- a(t).
0.5::c(t); 0.5::c(f) :- a(f).
0.99::d(t); 0.01::d(f) :- b(t), c(t).
0.8::d(t); 0.2::d(f) :- b(t), c(f).
0.6::d(t); 0.4::d(f) :- b(f), c(t).
0.05::d(t); 0.95::d(f) :- b(f), c(f).
'''


class ExactTests(unittest.TestCase):
    def close(self, got, want):
        self.assertEqual(set(got), set(want))
        for q in want:
            self.assertAlmostEqual(got[q], want[q], places=14, msg=q)

    def test_a_variable_can_have_no_value_as_in_problog(self):
        queries = ''.join(f'query({q}).\n' for q in ['a(x)', 'a(y)', 'b(t)', 'b(f)', 'c(u)'])
        self.close(answers(SHORT + queries), {
            'a(x)': 0.3, 'a(y)': 0.5, 'b(t)': 0.3 * 0.9 + 0.5 * 0.2,
            'b(f)': 0.3 * 0.1 + 0.5 * 0.7, 'c(u)': 0.3,
        })

    def test_evidence_conditions_through_the_missing_values(self):
        queries = ''.join(f'query({q}).\n' for q in ['a(x)', 'a(y)', 'b(f)', 'c(u)'])
        bt = 0.3 * 0.9 + 0.5 * 0.2
        self.close(answers(SHORT + 'evidence(b(t)).\n' + queries), {
            'a(x)': 0.3 * 0.9 / bt, 'a(y)': 0.5 * 0.2 / bt, 'b(f)': 0.0, 'c(u)': 0.3 * 0.9 / bt,
        })

    def test_evidence_of_probability_zero_is_inconsistent(self):
        self.assertEqual(answers(SHORT + 'evidence(c(u)).\nevidence(a(y)).\nquery(b(t)).\n'), 'inconsistent')

    def test_evidence_below_a_join_reaches_the_root(self):
        def given_a(pa, pb, pc):
            d = {(1, 1): 0.99, (1, 0): 0.8, (0, 1): 0.6, (0, 0): 0.05}
            return pa * sum((pb if b else 1 - pb) * (pc if c else 1 - pc) * d[b, c]
                            for b in (0, 1) for c in (0, 1))
        t, f = given_a(0.6, 0.7, 0.1), given_a(0.4, 0.2, 0.5)
        # b given d: the message down from d's cluster, past a's.
        bt = 0.6 * 0.7 * (0.1 * 0.99 + 0.9 * 0.8) + 0.4 * 0.2 * (0.5 * 0.99 + 0.5 * 0.8)
        self.close(answers(DIAMOND + 'evidence(d(t)).\nquery(a(t)).\nquery(b(t)).\n'), {
            'a(t)': t / (t + f), 'b(t)': bt / (t + f),
        })

    def test_a_thousand_observed_children_neither_underflow_nor_overflow_einsum(self):
        """A root with a thousand observed children: one cluster of a
        thousand factors, more than einsum takes at once, whose products
        per value of the root, near 1e-400, underflow a double unscaled."""
        rows = ['0.6::a(t); 0.4::a(f).',
                '0.5::d(t); 0.5::d(f) :- a(t).', '0.1::d(t); 0.9::d(f) :- a(f).']
        for i in range(1000):
            rows += [f'0.2::c{i}(t); 0.8::c{i}(f) :- a(t).', f'0.8::c{i}(t); 0.2::c{i}(f) :- a(f).',
                     f'evidence(c{i}({"t" if i < 501 else "f"})).']
        # Two more children observed t than f: a(t) against a(f) is
        # 0.6 * 0.2^2 against 0.4 * 0.8^2.
        at = 0.6 * 0.04 / (0.6 * 0.04 + 0.4 * 0.64)
        self.close(answers('\n'.join(rows) + '\nquery(a(t)).\nquery(d(t)).\n'), {
            'a(t)': at, 'd(t)': at * 0.5 + (1 - at) * 0.1,
        })



class EvidenceProbabilityTests(unittest.TestCase):
    """`--evidence-probability` against P(e) by hand."""

    def test_without_evidence_it_is_one(self):
        self.assertEqual(evidence_probability(DIAMOND + 'query(d(t)).\n', 6), ('1.000000e0', 0.0))

    def test_a_missing_value_is_mass_the_evidence_does_not_have(self):
        p, log10 = evidence_probability(SHORT + 'evidence(b(t)).\nquery(a(x)).\n')
        self.assertAlmostEqual(log10, math.log10(0.3 * 0.9 + 0.5 * 0.2), places=14)
        self.assertAlmostEqual(float(p), 0.37, places=15)

    def test_an_observed_root_and_a_second_component_multiply_in(self):
        """An observed root leaves a constant table; a second component
        gives a second root cluster."""
        def given_a(pa, pb, pc):
            d = {(1, 1): 0.99, (1, 0): 0.8, (0, 1): 0.6, (0, 0): 0.05}
            return pa * sum((pb if b else 1 - pb) * (pc if c else 1 - pc) * d[b, c]
                            for b in (0, 1) for c in (0, 1))
        t, f = given_a(0.6, 0.7, 0.1), given_a(0.4, 0.2, 0.5)
        other = '0.25::z(u); 0.75::z(w).\n0.1::y(t); 0.9::y(f) :- z(u).\n0.4::y(t); 0.6::y(f) :- z(w).\n'
        cases = [('evidence(d(t)).\n', t + f), ('evidence(a(t)).\nevidence(d(t)).\n', t),
                 (other + 'evidence(d(t)).\nevidence(y(t)).\n', (t + f) * (0.25 * 0.1 + 0.75 * 0.4))]
        for given, want in cases:
            _, log10 = evidence_probability(DIAMOND + given + 'query(b(t)).\n')
            self.assertAlmostEqual(log10, math.log10(want), places=14, msg=given)

    def test_inconsistent_evidence_is_zero(self):
        out = run(SHORT + 'evidence(c(u)).\nevidence(a(y)).\nquery(b(t)).\n', '--evidence-probability')
        self.assertEqual(out, 'evidence is inconsistent\nP(e):\t0.000000e0\tlog10 -inf\n')

    def test_the_probability_of_a_thousand_observations_is_below_a_double(self):
        """The thousand children below: P(e) = 0.16^499 * (0.6 * 0.2^2 +
        0.4 * 0.8^2), near 1e-397, printed off its logarithm."""
        rows = ['0.6::a(t); 0.4::a(f).']
        for i in range(1000):
            rows += [f'0.2::c{i}(t); 0.8::c{i}(f) :- a(t).', f'0.8::c{i}(t); 0.2::c{i}(f) :- a(f).',
                     f'evidence(c{i}({"t" if i < 501 else "f"})).']
        want = 499 * math.log10(0.16) + math.log10(0.28)
        program = '\n'.join(rows) + '\nquery(a(t)).\n'
        self.assertAlmostEqual(evidence_probability(program)[1], want, places=10)
        self.assertEqual(evidence_probability(program, 6), (f'{10 ** (want + 398):.6f}e-398', round(want, 6)))


if __name__ == '__main__':
    unittest.main()
