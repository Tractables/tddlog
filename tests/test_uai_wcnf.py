"""uai_wcnf.py, written by `gen_bn.py --cnf`: the weighted CNF of a UAI
network with its evidence, whose weighted model count is the partition
function Z given the evidence. Checked against enumeration of the
network's worlds in exact fractions on the networks of tests/regress and
on random small ones, MARKOV and BAYES, with zeros, ones, evidence,
variables of one, two, three and four values, functions of empty scope
and functions over three variables.
"""
from fractions import Fraction
import math
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
regress = root / 'tests' / 'regress'
sys.path.insert(0, str(scripts))
sys.path.insert(0, str(root / 'tests'))
import gen_bn  # noqa: E402
import uai_wcnf  # noqa: E402
from test_gen_bn import brute_force_z, random_markov, random_network_as_markov  # noqa: E402


def weighted_count(text):
    """The weighted model count of a DIMACS CNF with weight lines, in
    exact fractions: a literal weighs what its weight line says, the other
    literal of a weight 0 < w < 1 given alone 1 - w, and 1 otherwise. A
    search over the variables in their order that checks each clause when
    its last variable is set, memoized on the variable and the clauses not
    yet decided that the assignment so far meets: a CNF of uai_wcnf.py,
    whose parameters follow from the indicators numbered before them or
    (enc5) are free where no clause of theirs is left, takes time linear
    in its models."""
    n, declared, clauses, w = None, None, [], {}
    for line in text.splitlines():
        s = line.split()
        if s[:3] == ['c', 'p', 'weight']:
            assert len(s) == 6 and s[5] == '0', line
            w[int(s[3])] = Fraction(s[4])
        elif s and s[0] == 'p':
            n, declared = int(s[2]), int(s[3])
        elif s and s[0] != 'c':
            assert s[-1] == '0', line
            clauses.append([int(x) for x in s[:-1]])
    assert len(clauses) == declared, (len(clauses), declared)
    for v in range(1, n + 1):
        if (v in w) != (-v in w):
            given = w.get(v, w.get(-v))
            assert 0 < given < 1
            w.setdefault(v, 1 - given)
            w.setdefault(-v, 1 - given)
    last = [max((abs(x) for x in c), default=0) for c in clauses]
    if 0 in last:
        return Fraction(0)   # the empty clause
    for c in clauses:
        assert all(0 < abs(x) <= n for x in c), c
    holds = [{} for _ in range(n + 1)]   # v: {literal: clauses it meets}
    ending = [[] for _ in range(n + 1)]
    for ci, c in enumerate(clauses):
        for x in c:
            holds[abs(x)].setdefault(x, []).append(ci)
        ending[last[ci]].append(ci)
    memo = {}

    def count(v, met):
        if v > n:
            return Fraction(1)
        key = (v, met)
        if key not in memo:
            total = Fraction(0)
            for x in (-v, v):
                now = met | frozenset(holds[v].get(x, ()))
                if all(ci in now for ci in ending[v]):
                    total += w.get(x, Fraction(1)) * count(v + 1, frozenset(ci for ci in now if last[ci] > v))
            memo[key] = total
        return memo[key]
    return count(1, frozenset())


def scratch_dir(prefix):
    (root / 'tmp').mkdir(exist_ok=True)
    return tempfile.TemporaryDirectory(dir=root / 'tmp', prefix=prefix)


def cnf_of(d, model, evid):
    """The CNF and its counts of a UAI file's text and an evidence file's,
    through gen_bn.py's readers."""
    path = Path(d) / 'net.uai'
    path.write_text(model)
    evidence = []
    if evid:
        (Path(d) / 'net.uai.evid').write_text(evid)
        order, variables = gen_bn.uai_variables(gen_bn.read_uai(model)[1])
        evidence = gen_bn.read_evid(str(Path(d) / 'net.uai.evid'), (order, variables, None))
    return gen_bn.uai_cnf(str(path), evidence)


# A Bayesian network over a (one value), b (four), c (three) and d (two),
# d's table given a, b and c, so a function over four variables of which
# one has a single value; zeros and ones in b's, c's and d's tables.
BAYES = '''BAYES
4
1 4 3 2
4
1 0
1 1
2 1 2
4 0 1 2 3
1
 1.0
4
 0.125 0.5 0.0 0.375
12
 0.25 0.25 0.5  1.0 0.0 0.0  0.5 0.25 0.25  0.0 0.0 1.0
24
 0.5 0.5  0.25 0.75  1.0 0.0  0.125 0.875  0.75 0.25  0.0 1.0
 0.5 0.5  0.625 0.375  0.875 0.125  0.25 0.75  0.375 0.625  1.0 0.0
'''


class EncodingTests(unittest.TestCase):
    def test_the_count_is_the_partition_function(self):
        cases = [((regress / 'markov.uai').read_text(), ''),
                 ((regress / 'markov.uai').read_text(), (regress / 'markov.uai.evid').read_text()),
                 ((regress / 'markov_pr.uai').read_text(), ''),
                 ((regress / 'markov_pr.uai').read_text(), (regress / 'markov_pr.uai.evid').read_text()),
                 (BAYES, ''), (BAYES, '1 3 1'), (BAYES, '2 1 1 2 2'), (BAYES, '2 1 2 2 0'), (BAYES, '1 0 0')]
        rng = random.Random(5)
        cases += [random_markov(rng) for _ in range(60)]
        cases += [random_network_as_markov(rng) for _ in range(30)]
        # tests/regress's comments, and a Bayesian network's tables sum to
        # one. Its evidence b = 1, c = 2 meets a zero of c's table, and
        # b = 2 has none.
        want = [325, 165, Fraction(19947, 25), Fraction(11853, 40), 1]
        zeros = 0
        with scratch_dir('uai-wcnf-') as d:
            for k, (model, evid) in enumerate(cases):
                z = brute_force_z(model, evid)
                if k < len(want):
                    self.assertEqual(z, want[k], msg=k)
                zeros += z == 0
                text, counts = cnf_of(d, model, evid)
                self.assertEqual(weighted_count(text), z, msg=(k, model, evid))
                self.assertEqual(counts['variables'], counts['indicators'] + counts['parameters'])
                self.assertEqual(counts['clauses'], sum(counts[c] for c in ['exactly_one', 'zeros',
                                                                             'parameter_clauses', 'evidence']))
                self.assertTrue(text.startswith(f"c t wmc\np cnf {counts['variables']} {counts['clauses']}\n"))
        self.assertGreater(zeros, 0)   # evidence that meets a zero entry

    def test_chains_count_the_partition_function(self):
        # Sang, Beame and Kautz's chains for the conditional tables, a
        # parameter per entry for the others: each switch is a quotient of
        # the entries as doubles, so the count is Z to within rounding.
        cases = [(BAYES, ''), (BAYES, '1 3 1'), (BAYES, '2 1 1 2 2'), (BAYES, '2 1 2 2 0'), (BAYES, '1 0 0')]
        rng = random.Random(9)
        cases += [random_network_as_markov(rng) for _ in range(30)] + [random_markov(rng) for _ in range(20)]
        chains = parameters = 0
        for k, (model, evid) in enumerate(cases):
            _, sizes, scopes, tables = gen_bn.read_uai(model)
            e = [int(x) for x in evid.split()[1:]]
            text, counts = uai_wcnf.encode(sizes, scopes, tables, list(zip(e[0::2], e[1::2])), 'chains')
            z, got = brute_force_z(model, evid), weighted_count(text)
            if z == 0:
                self.assertEqual(got, 0, msg=(k, model, evid))
            else:
                self.assertAlmostEqual(float(got / z), 1, delta=1e-12, msg=(k, model, evid))
            self.assertEqual(counts['variables'], counts['indicators'] + counts['parameters'] + counts['switches'])
            self.assertEqual(counts['clauses'], sum(counts[c] for c in ['exactly_one', 'zeros', 'parameter_clauses',
                                                                         'evidence', 'chain_clauses']))
            chains += counts['chains']
            parameters += counts['parameters']
        self.assertGreater(chains, 50)
        self.assertGreater(parameters, 50)   # the Markov networks' tables

    def test_enc5_counts_the_partition_function(self):
        # A parameter per distinct entry of a table, implied by the merged
        # cubes of its entries, weighing p and 1 - p: the count is Z to
        # within the rounding of 1 - p and of the tables divided by their
        # largest entry.
        cases = [((regress / 'markov.uai').read_text(), (regress / 'markov.uai.evid').read_text()),
                 ((regress / 'markov_pr.uai').read_text(), (regress / 'markov_pr.uai.evid').read_text()),
                 (BAYES, ''), (BAYES, '1 3 1'), (BAYES, '2 1 1 2 2'), (BAYES, '2 1 2 2 0'), (BAYES, '1 0 0')]
        rng = random.Random(13)
        cases += [random_network_as_markov(rng) for _ in range(30)] + [random_markov(rng) for _ in range(30)]
        total = {'parameters': 0, 'entries': 0, 'merged': 0, 'constants': 0}
        for k, (model, evid) in enumerate(cases):
            _, sizes, scopes, tables = gen_bn.read_uai(model)
            e = [int(x) for x in evid.split()[1:]]
            text, counts = uai_wcnf.encode(sizes, scopes, tables, list(zip(e[0::2], e[1::2])), 'enc5')
            z, got = brute_force_z(model, evid), weighted_count(text)
            if z == 0:
                self.assertEqual(got, 0, msg=(k, model, evid))
            else:
                self.assertAlmostEqual(float(got / z), 1, delta=1e-12, msg=(k, model, evid))
            self.assertEqual(counts['variables'], counts['indicators'] + counts['parameters'] + counts['constants'])
            self.assertEqual(counts['clauses'], sum(counts[c] for c in ['exactly_one', 'zeros', 'parameter_clauses',
                                                                         'evidence', 'constants']))
            entries = sum(1 for t in tables for x in t if 0 < x < max(t) or (0 < x < 1 and max(t) <= 1))
            total['parameters'] += counts['parameters']
            total['entries'] += entries
            total['merged'] += entries - counts['parameter_clauses']
            total['constants'] += counts['constants']
        # Equal entries share a parameter, cubes merge, and tables with
        # entries above 1 have a constant.
        self.assertLess(total['parameters'], total['entries'])
        self.assertGreater(total['merged'], 0)
        self.assertGreater(total['constants'], 0)

    def test_enc5_by_hand(self):
        """x (two values) and y (three) with y's table given x: rows
        (0.5, 0.25, 0.25) and (0.25, 0.25, 0.5). 0.25 is one parameter of
        four entries, of whose cubes x=0 & y=1 and x=1 & y=1 merge into
        y=1 (three clauses); 0.5 is one of two entries; x's prior (0.5,
        0.5) a parameter of two entries whose cubes merge into the empty
        one, a unit clause. A Markov table (2, 4) over x is divided by 4, a
        constant."""
        sizes, scopes = [2, 3], [[0], [0, 1]]
        tables = [[0.5, 0.5], [0.5, 0.25, 0.25, 0.25, 0.25, 0.5]]
        text, counts = uai_wcnf.encode(sizes, scopes, tables, [], 'enc5')
        # x is variable 1 (true at its value 1), y's values 2, 3, 4.
        self.assertEqual(counts, {'variables': 7, 'indicators': 4, 'parameters': 3, 'clauses': 10,
                                  'exactly_one': 4, 'zeros': 0, 'parameter_clauses': 6, 'evidence': 0,
                                  'constants': 0})
        body = text.split('\n')
        self.assertIn('5 0', body)                     # x's prior, one parameter that always holds
        self.assertIn('-3 7 0', body)                  # y = 1 -> P_0.25, merged over x
        self.assertIn('1 -2 6 0', body)                # x = 0 & y = 0 -> P_0.5
        self.assertIn('-1 -4 6 0', body)               # x = 1 & y = 2 -> P_0.5
        self.assertEqual(weighted_count(text), 1)
        text, counts = uai_wcnf.encode([2], [[0]], [[2.0, 4.0]], [], 'enc5')
        self.assertEqual(counts['constants'], 1)
        self.assertEqual(weighted_count(text), 6)
        self.assertIn('c p weight 2 4.0 0', text)   # x is variable 1

    def test_the_counts_of_markov_pr(self):
        """markov_pr.uai with its evidence: 9 indicators (x1 and x4 of three
        values, 4 clauses each), 34 parameters (every entry but a zero and
        a one of x1's, a 1.0 and a 0.0 of the factor on x1, x2, x3), 105
        clauses for them (an entry over k variables k + 1) and 2 for the
        zeros, and x4 = 2 a unit clause."""
        with scratch_dir('uai-wcnf-') as d:
            text, counts = cnf_of(d, (regress / 'markov_pr.uai').read_text(),
                                  (regress / 'markov_pr.uai.evid').read_text())
        self.assertEqual(counts, {'variables': 43, 'indicators': 9, 'parameters': 34, 'clauses': 116,
                                  'exactly_one': 8, 'zeros': 2, 'parameter_clauses': 105, 'evidence': 1})
        # x4's value 2 is its third indicator: x0 is 1, x1 2 to 4, x2 5,
        # x3 6, x4 7 to 9.
        self.assertTrue(text.endswith('\n9 0\n'))
        # The factor of empty scope, 4.0, is a parameter that holds.
        self.assertIn('c p weight 37 4.0 0\nc p weight -37 1 0\n', text)
        self.assertIn('\n37 0\n', text)

    def test_what_is_no_network_is_refused(self):
        for sizes, scopes, tables, evidence, error in [
                ([2], [[0]], [[0.5, -1.0]], [], 'no weight'),
                ([2], [[0]], [[0.5, math.nan]], [], 'no weight'),
                ([2, 2], [[0, 0]], [[1.0] * 4], [], 'twice'),
                ([2], [[0]], [[1.0]], [], 'entries'),
                ([2], [], [], [(0, 2)], 'beyond'),
                ([2], [], [], [(0, 1), (0, 1)], 'observed twice'),
                ([0], [], [], [], '0 values')]:
            for encoding in uai_wcnf.ENCODINGS:
                with self.subTest(error=error, encoding=encoding), self.assertRaisesRegex(ValueError, error):
                    uai_wcnf.encode(sizes, scopes, tables, evidence, encoding)
        with self.assertRaisesRegex(ValueError, 'no encoding'):
            uai_wcnf.encode([2], [[0]], [[0.5, 0.5]], [], 'enc2')

    def test_gen_bn_writes_it_and_refuses_what_it_cannot_encode(self):
        gen = [sys.executable, str(scripts / 'gen_bn.py')]
        with scratch_dir('uai-wcnf-') as d:
            out = Path(d) / 'out.cnf'
            network = [str(regress / 'markov_pr.uai'), '--evid', str(regress / 'markov_pr.uai.evid')]
            r = subprocess.run(gen + network + ['--cnf', str(out)], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            # The program is the one written without --cnf.
            self.assertEqual(r.stdout, subprocess.run(gen + network, capture_output=True, text=True).stdout)
            self.assertEqual(out.read_text(), cnf_of(d, (regress / 'markov_pr.uai').read_text(),
                                                     (regress / 'markov_pr.uai.evid').read_text())[0])
            # --encoding: the tables as chains or as ENC5.
            bayes = Path(d) / 'bayes.uai'
            bayes.write_text(BAYES)
            _, sizes, scopes, tables = gen_bn.read_uai(BAYES)
            for encoding in uai_wcnf.ENCODINGS:
                r = subprocess.run(gen + [str(bayes), '--cnf', str(out), '--encoding', encoding],
                                   capture_output=True, text=True)
                self.assertEqual(r.returncode, 0, r.stderr)
                self.assertEqual(out.read_text(), uai_wcnf.encode(sizes, scopes, tables, [], encoding)[0])
                r = subprocess.run(gen + [str(bayes), '--encoding', encoding], capture_output=True, text=True)
                self.assertNotEqual(r.returncode, 0)
                self.assertIn('no --cnf', r.stderr)
            self.assertNotEqual(uai_wcnf.encode(sizes, scopes, tables, [], 'enc5')[0],
                                uai_wcnf.encode(sizes, scopes, tables, [])[0])
            for argv, error in [([str(root / 'examples' / 'asia.net')], 'not one'),
                                ([str(regress / 'markov.uai'), '--rescale'], 'rescale'),
                                ([str(regress / 'markov.uai'), '--format', 'dice'], 'Dice')]:
                with self.subTest(error=error):
                    r = subprocess.run(gen + argv + ['--cnf', str(Path(d) / 'refused.cnf')],
                                       capture_output=True, text=True)
                    self.assertNotEqual(r.returncode, 0)
                    self.assertIn(error, r.stderr)
                    self.assertFalse((Path(d) / 'refused.cnf').exists())


if __name__ == '__main__':
    unittest.main()
