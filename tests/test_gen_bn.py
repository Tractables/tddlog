from fractions import Fraction
import itertools
import math
import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from source_tree import SOURCE  # noqa: E402  (the source tree's examples and fixtures)
import gen_bn  # noqa: E402
try:
    import bn_exact   # on the baselines branch only; its tests skip without it
except ImportError:
    bn_exact = None


def brute_force_z(model, evid=''):
    """The partition function of a MARKOV UAI file given a UAI evidence
    file, the sum over the worlds of the product of the factors, in exact
    fractions of the decimals the file writes; for a BAYES file, whose
    functions are laid out alike, P(e)."""
    tok = model.split()
    assert tok[0] in ('MARKOV', 'BAYES')
    n = int(tok[1])
    sizes, pos = [int(t) for t in tok[2:2 + n]], 2 + n
    scopes = []
    for _ in range(int(tok[pos])):
        k = int(tok[pos + 1])
        scopes.append([int(t) for t in tok[pos + 2:pos + 2 + k]])
        pos += 1 + k
    pos += 1
    tables = []
    for _ in scopes:
        c = int(tok[pos])
        tables.append([Fraction(t) for t in tok[pos + 1:pos + 1 + c]])
        pos += 1 + c
    e = [int(t) for t in evid.split()]
    given = dict(zip(e[1::2], e[2::2]))
    z = Fraction(0)
    for world in itertools.product(*(range(k) for k in sizes)):
        if any(world[i] != u for i, u in given.items()):
            continue
        w = Fraction(1)
        for scope, table in zip(scopes, tables):
            j = 0
            for v in scope:
                j = j * sizes[v] + world[v]
            w *= table[j]
        z += w
    return z


def program_pe(network, evidence):
    """P(e) of gen_bn.py's tables for a Markov network, by enumeration: a
    world of the network's variables weighs its priors times the chance of
    each observed factor's `ok` in it."""
    order, variables, tables = network
    given = dict(evidence)
    xs = [v for v in order if v.startswith('x')]
    total = []
    for world in itertools.product(*(variables[v] for v in xs)):
        value = dict(zip(xs, world))
        if any(value[v] != u for v, u in given.items() if v in value):
            continue
        w = 1.0
        for v in order:
            parents, rows = tables[v]
            row = rows[tuple(value[u] for u in parents)]
            w *= row[variables[v].index(value[v])] if v in value else row[0]
        total.append(w)
    return math.fsum(total)


def random_markov(rng):
    """A MARKOV UAI file of two to four variables with factors of scope zero
    to three, whose entries take few values so that zeros, ties and constant
    factors come up, and a UAI evidence file observing none or one of its
    variables."""
    n = rng.randint(2, 4)
    sizes = [rng.randint(2, 3) for _ in range(n)]
    scopes, tables = [], []
    for _ in range(rng.randint(1, 5)):
        scope = rng.sample(range(n), rng.randint(0, min(3, n)))
        entries = math.prod(sizes[v] for v in scope)
        if rng.random() < 0.2:
            table = [rng.choice(['0.5', '2', '3.25'])] * entries
        elif len(scope) == 1:
            table = [rng.choice(['0.5', '1', '1.5', '2.25']) for _ in range(entries)]
        else:
            table = [rng.choice(['0', '0.5', '1', '1.5', '2.25', '4']) for _ in range(entries)]
            table[rng.randrange(entries)] = '3'
        scopes.append(scope)
        tables.append(table)
    lines = ['MARKOV', str(n), ' '.join(map(str, sizes)), str(len(scopes))]
    lines += [' '.join(map(str, [len(s)] + s)) for s in scopes]
    for t in tables:
        lines += [str(len(t)), ' ' + ' '.join(t)]
    v = rng.randrange(n)
    evid = rng.choice(['', f'1 {v} {rng.randrange(sizes[v])}'])
    return '\n'.join(lines) + '\n', evid


def random_potts(rng):
    """A MARKOV UAI file of two to six variables of k = 2 to 4 values each
    and one to eight Potts tables over two of them, one entry on the
    diagonal and one off it, zeros among them (a graph colouring's tables
    are 0 on the diagonal and 1 off it)."""
    n, k = rng.randint(2, 6), rng.randint(2, 4)
    scopes, tables = [], []
    for _ in range(rng.randint(1, 8)):
        scope = rng.sample(range(n), 2)
        same, other = rng.choice([('0', '1'), ('1', '0'), ('2', '0.5'), ('0.5', '3'), ('1.5', '1.5')])
        scopes.append(scope)
        tables.append([same if i == j else other for i in range(k) for j in range(k)])
    lines = ['MARKOV', str(n), ' '.join([str(k)] * n), str(len(scopes))]
    lines += [' '.join(map(str, [len(s)] + s)) for s in scopes]
    for t in tables:
        lines += [str(len(t)), ' ' + ' '.join(t)]
    return '\n'.join(lines) + '\n'


def random_network_as_markov(rng):
    """A random Bayesian network of two to four variables written as a
    MARKOV UAI file: each variable's table over its parents and itself, the
    scope in random order and the table laid out to match, the rows of
    eighths so that they sum to one exactly; and a UAI evidence file
    observing none or one variable."""
    n = rng.randint(2, 4)
    sizes = [rng.randint(2, 3) for _ in range(n)]
    scopes, tables = [], []
    for v in range(n):
        parents = rng.sample(range(v), rng.randint(0, min(2, v)))
        rows = {}
        for key in itertools.product(*(range(sizes[u]) for u in parents)):
            cuts = sorted(rng.randint(0, 8) for _ in range(sizes[v] - 1))
            rows[key] = [Fraction(b - a, 8) for a, b in zip([0] + cuts, cuts + [8])]
        scope = parents + [v]
        rng.shuffle(scope)
        table = []
        for world in itertools.product(*(range(sizes[u]) for u in scope)):
            value = dict(zip(scope, world))
            table.append(str(float(rows[tuple(value[u] for u in parents)][value[v]])))
        scopes.append(scope)
        tables.append(table)
    lines = ['MARKOV', str(n), ' '.join(map(str, sizes)), str(n)]
    lines += [' '.join(map(str, [len(s)] + s)) for s in scopes]
    for t in tables:
        lines += [str(len(t)), ' ' + ' '.join(t)]
    v = rng.randrange(n)
    evid = rng.choice(['', f'1 {v} {rng.randrange(sizes[v])}'])
    return '\n'.join(lines) + '\n', evid


class EvidencePoolTests(unittest.TestCase):
    def test_internal_observations_come_from_a_consistent_world(self):
        variables = {v: ['no', 'yes'] for v in ['root', 'mid', 'leaf']}
        copy = {('no',): [1, 0], ('yes',): [0, 1]}
        network = (list(variables), variables, {
            'root': ([], {(): [0, 1]}), 'mid': (['root'], copy), 'leaf': (['mid'], copy),
        })
        self.assertEqual(gen_bn.sampled_evidence(network, 1, 1), [('leaf', 'yes')])
        self.assertEqual(gen_bn.sampled_evidence(network, 2, 1, 'internal'), [('root', 'yes'), ('mid', 'yes')])
        self.assertEqual(gen_bn.sampled_evidence(network, 3, 1, 'all'), [(v, 'yes') for v in variables])
        text = gen_bn.to_problog(network, 'leaves', gen_bn.sampled_evidence(network, 2, 1, 'internal'))
        self.assertIn('evidence(mid(yes)).', text)
        self.assertIn('query(leaf(yes)).', text)
        self.assertNotIn('query(mid(yes)).', text)


class RowTests(unittest.TestCase):
    def test_a_gap_that_rounding_explains_is_rescaled(self):
        for row in ([0.3333333] * 3, [0.333] * 3, [0.3333334, 0.3333334, 0.3333333], [0.12345678, 0.43209877, 0.44444444],
                    [0.9999, 9.999e-05]):
            self.assertAlmostEqual(math.fsum(gen_bn.normalized(row, 'x')), 1.0, places=15, msg=row)

    def test_a_gap_that_rounding_does_not_explain_is_the_tables(self):
        self.assertEqual(gen_bn.normalized([0.5, 0.499], 'x'), [0.5, 0.499])
        self.assertEqual(gen_bn.normalized([0.999], 'x'), [0.999])
        self.assertEqual(gen_bn.normalized([0.3333333, 0.3333333, 0.333333], 'x'), [0.3333333, 0.3333333, 0.333333])
        with self.assertRaisesRegex(SystemExit, 'x: probabilities sum to 1.001'):
            gen_bn.normalized([0.9, 0.101], 'x')

    def test_an_excess_of_binary_rounding_is_rescaled(self):
        # Two doubles that sum to 1.0000000000000003 in the decimals they
        # mean, more than the slack of their own digits allows.
        row = [0.3, 0.7000000000000003]
        self.assertAlmostEqual(math.fsum(gen_bn.normalized(row, 'x')), 1.0, places=15)
        with self.assertRaisesRegex(SystemExit, 'x: probabilities sum to 1.000000002'):
            gen_bn.normalized([0.3, 0.700000002], 'x')

    def test_a_row_that_sums_to_one_keeps_its_numbers(self):
        self.assertEqual(gen_bn.normalized([0.2, 0.8], 'x'), [0.2, 0.8])
        self.assertEqual(gen_bn.normalized([1, 0], 'x'), [1, 0])


class SourceTests(unittest.TestCase):
    def test_a_gzipped_network_reads_as_the_plain_one(self):
        root = Path(__file__).resolve().parents[1]
        plain = gen_bn.parse(str(SOURCE / 'examples' / 'asia.net'))
        target = root / 'tmp' / 'asia.net.gz'
        target.parent.mkdir(exist_ok=True)
        import gzip
        with gzip.open(target, 'wt') as f:
            f.write((SOURCE / 'examples' / 'asia.net').read_text())
        self.assertEqual(gen_bn.parse(str(target)), plain)


XDSL = b'''<?xml version="1.0" encoding="ISO-8859-1"?>
<smile version="1.0" id="n">
  <nodes>
    <cpt id="a">
      <state id="x" />
      <state id="y" />
      <probabilities>0.3 0.7</probabilities>
    </cpt>
    <cpt id="b">
      <state id="u" />
      <state id="v" />
      <state id="w" />
      <parents>a</parents>
      <probabilities>0.9 0.1 0 0.2 0.5 0.30000000000000004</probabilities>
    </cpt>
  </nodes>
</smile>
'''


class BifFormatTests(unittest.TestCase):
    """`to_bif`: the BIF file and queries of the Roulette artifact's own
    network driver (benchmark/bayes.rkt under data/roulette/bif.rkt)."""

    def test_a_network_written_as_bif_reads_back_as_its_tables(self):
        """asia's BIF file reads back as asia's normalized tables under the
        names x<i> and s<k>, with a line for each leaf: the leaf, then it
        and its ancestors in program_order."""
        network = gen_bn.parse(str(SOURCE / 'examples' / 'asia.net'))
        order, variables, tables = network
        text, lines, names = gen_bn.to_bif(network, 'leaves')
        back_order, back_variables, back_tables = gen_bn.parse_bif(text)
        word = {v: f'x{i}' for i, v in enumerate(order)}
        self.assertEqual(back_order, [word[v] for v in order])
        for v in order:
            self.assertEqual(back_variables[word[v]], [f's{k}' for k in range(len(variables[v]))])
            parents, rows = tables[v]
            self.assertEqual(back_tables[word[v]][0], [word[p] for p in parents])
            for key, row in rows.items():
                back = back_tables[word[v]][1][tuple(f's{variables[p].index(u)}' for p, u in zip(parents, key))]
                self.assertEqual([float(p) for p in back], gen_bn.normalized(row, v))
        leaves = gen_bn.queried_variables(order, variables, tables, 'leaves')
        self.assertEqual(lines.splitlines(),
                         [' '.join([word[q]] + [word[v] for v in gen_bn.program_order(order, tables, [], [q])])
                          for q in leaves])
        self.assertEqual(names, {word[q]: (gen_bn.ident(q), [gen_bn.ident(u) for u in variables[q]]) for q in leaves})

    def test_numbers_are_positional(self):
        """The artifact's lexer reads `1e-06` as a word: a small number is
        written in positional notation, and reads back as itself."""
        variables = {'a': ['x', 'y'], 'b': ['f', 't']}
        tables = {'a': ([], {(): [0.999999, 0.000001]}),
                  'b': (['a'], {('x',): [1.0, 0.0], ('y',): [1e-07, 1 - 1e-07]})}
        text, _, _ = gen_bn.to_bif((['a', 'b'], variables, tables), 'b')
        self.assertNotRegex(text, r'[0-9][eE]')
        self.assertIn('table 0.999999, 0.000001;', text)
        self.assertIn('(s1) 0.0000001, 0.9999999;', text)
        self.assertEqual(gen_bn.bif_number(1e-300), '0.' + '0' * 299 + '1')

    def test_observations_and_factors_have_no_bif_file(self):
        """BIF holds no observations and no factors: bif_programs is None for
        a network with evidence and for a Markov network."""
        asia = str(SOURCE / 'examples' / 'asia.net')
        self.assertIsNotNone(gen_bn.bif_programs([asia, '--query', 'leaves']))
        self.assertIsNone(gen_bn.bif_programs([asia, '--query', 'leaves', '--evidence', '1']))
        regress = SOURCE / 'tests' / 'regress'
        self.assertIsNone(gen_bn.bif_programs([str(regress / 'markov.uai'), '--query', 'all']))


class DiceFormatTests(unittest.TestCase):
    """The Dice format with observations (`dice_programs`): one program per
    queried variable, each defining the variables it returns or observes
    and their ancestors, a Markov network's factors as observed flips. The
    marginals of the programs, by test_dice2problog.py's enumeration of a
    Dice program, are the network's, by enumeration of its tables."""

    @staticmethod
    def marginal(network, evidence, v):
        order, variables, tables = network
        given = dict(evidence)
        mass = [Fraction(0)] * len(variables[v])
        for world in itertools.product(*(variables[u] for u in order)):
            value = dict(zip(order, world))
            if any(value[u] != x for u, x in given.items()):
                continue
            mass[variables[v].index(value[v])] += math.prod(
                Fraction(tables[u][1][tuple(value[p] for p in tables[u][0])][variables[u].index(value[u])])
                for u in order)
        return [m / sum(mass) for m in mass]

    @staticmethod
    def dice_marginal(text, n):
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import test_dice2problog
        dist = test_dice2problog.exact(text)
        return [sum(p for value, p in dist.items() if value[0] == k) for k in range(n)]

    def test_a_bayesian_network_observes_its_evidence(self):
        net = str(SOURCE / 'examples' / 'asia.net')
        network = gen_bn.parse(net)
        order, variables, _ = network
        evidence = gen_bn.sampled_evidence(network, 'half', 1)
        self.assertTrue(evidence)
        programs = gen_bn.dice_programs([net, '--query', 'all', '--evidence', 'half'])
        by_ident = {gen_bn.ident(v): v for v in order}
        self.assertEqual([by_ident[i] for i, _ in programs], [v for v in order if v not in dict(evidence)])
        for i, text in programs:
            v = by_ident[i]
            with self.subTest(variable=v):
                self.assertEqual(text.count('observe'), len(evidence))
                for got, want in zip(self.dice_marginal(text, len(variables[v])),
                                     self.marginal(network, evidence, v)):
                    self.assertAlmostEqual(float(got), float(want), places=12)

    def test_a_markov_network_observes_its_factors(self):
        """markov.uai given markov.uai.evid: the marginals of markov_ev.pl."""
        regress = SOURCE / 'tests' / 'regress'
        programs = dict(gen_bn.dice_programs([str(regress / 'markov.uai'), '--evid', str(regress / 'markov.uai.evid')]))
        self.assertEqual(sorted(programs), ['x0', 'x1'])
        self.assertIn('flip', programs['x0'])
        for v, probs in {'x0': ['2/11', '9/11'], 'x1': ['5/33', '28/33']}.items():
            for got, want in zip(self.dice_marginal(programs[v], 2), probs):
                self.assertAlmostEqual(float(got), float(Fraction(want)), places=12)

    def test_the_program_of_p_e_returns_the_observations(self):
        """With `pe`, one program that observes nothing and returns the
        conjunction of the observations, its probability of true P(e):
        markov.uai's given markov.uai.evid is 33/192, and with the offset
        its ProbLog program states, log10 Z is log10 165 (markov_ev.pl);
        asia's given half its leaves is what bn_exact.py finds for the
        ProbLog format's program, the program tddlog is given."""
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import test_dice2problog
        regress = SOURCE / 'tests' / 'regress'
        uai = str(regress / 'markov.uai')
        [(name, text)] = gen_bn.dice_programs([uai, '--evid', uai + '.evid'], pe=True)
        self.assertEqual(name, 'pe')
        self.assertNotIn('observe', text)
        p = test_dice2problog.exact(text).get(True, 0.0)
        self.assertAlmostEqual(p, 33 / 192, places=12)
        _, _, offset = gen_bn.parse(uai)
        self.assertAlmostEqual(math.log10(p) + offset, math.log10(165), places=12)
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        net = str(SOURCE / 'examples' / 'asia.net')
        network = gen_bn.parse(net)
        evidence = gen_bn.sampled_evidence(network, 'half', 1)
        [(_, text)] = gen_bn.dice_programs([net, '--query', 'all', '--evidence', 'half'], pe=True)
        rows, values, ev, _ = bn_exact.parse(gen_bn.to_problog(network, 'all', evidence))
        states, tables = bn_exact.factors(rows, values)
        _, _, log10_pe = bn_exact.marginals(states, tables, ev)
        self.assertAlmostEqual(test_dice2problog.exact(text).get(True, 0.0), 10 ** log10_pe, places=12)

    def test_the_joint_program_returns_the_observations_beside_the_variable(self):
        """With `joint`, a program that observes nothing and returns the pair
        of the variable and the conjunction of the observations: its rows
        where the conjunction holds, over their sum, are the marginals (Dice
        1.0 prints zeros where the observations' probability is under 1e-6)."""
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        import test_dice2problog
        net = str(SOURCE / 'examples' / 'asia.net')
        network = gen_bn.parse(net)
        order, variables, _ = network
        evidence = gen_bn.sampled_evidence(network, 'half', 1)
        by_ident = {gen_bn.ident(v): v for v in order}
        regress = SOURCE / 'tests' / 'regress'
        markov = [str(regress / 'markov.uai'), '--evid', str(regress / 'markov.uai.evid')]
        cases = [(i, text, self.marginal(network, evidence, by_ident[i]))
                 for i, text in gen_bn.dice_programs([net, '--query', 'all', '--evidence', 'half'], joint=True)]
        cases += [(v, text, [Fraction(p) for p in {'x0': ['2/11', '9/11'], 'x1': ['5/33', '28/33']}[v]])
                  for v, text in gen_bn.dice_programs(markov, joint=True)]
        self.assertEqual(len(cases), len(order) - len(evidence) + 2)
        for i, text, want in cases:
            with self.subTest(variable=i):
                self.assertNotIn('observe', text)
                dist = test_dice2problog.exact(text)
                held = [sum(p for value, p in dist.items() if value[1] is True and value[0][0] == k)
                        for k in range(len(want))]
                for got, w in zip(held, want):
                    self.assertAlmostEqual(float(got / sum(held)), float(w), places=12)

    def test_each_let_follows_its_parents_on_a_chain_longer_than_python_recursion(self):
        """`program_order`: the observations first, then each returned
        variable after the parents it reads, on a chain of 3 000 declared
        last first."""
        n = 3000
        chain = [f'v{i}' for i in range(n)]
        variables = {v: ['f', 't'] for v in chain}
        tables = {'v0': ([], {(): [0.3, 0.7]})}
        for i in range(1, n):
            tables[f'v{i}'] = ([f'v{i - 1}'], {('f',): [0.9, 0.1], ('t',): [0.2, 0.8]})
        text = gen_bn.to_dice((list(reversed(chain)), variables, tables), f'v{n - 1}', [('v0', 't')])
        lets = [line.split()[1] for line in text.splitlines() if line.startswith('let ')]
        self.assertEqual([x for x in lets if x.isupper()], [gen_bn.ident(v).upper() for v in chain])

    def test_small_values_are_flips_and_certain_rows_constants(self):
        """Dice 1.0 sets to zero a discrete's value whose bit carries under
        1e-4 (gen_discrete): such a row is written as flips over the value's
        bits, which keep it, and a row certain of one value as that value."""
        variables = {'a': ['x', 'y', 'z'], 'b': ['f', 't']}
        tables = {'a': ([], {(): [0.99995, 0.00004, 0.00001]}),
                  'b': (['a'], {('x',): [1.0, 0.0], ('y',): [0.25, 0.75], ('z',): [0.5, 0.5]})}
        network = (['a', 'b'], variables, tables)
        text = gen_bn.to_dice(network, 'a', [('b', 't')])
        let_a = next(line for line in text.splitlines() if line.startswith('let A '))
        self.assertNotIn('discrete', let_a)
        self.assertIn('flip', let_a)
        self.assertIn('int(1, 0)', next(line for line in text.splitlines() if line.startswith('let B ')))
        for got, want in zip(self.dice_marginal(text, 3), self.marginal(network, [('b', 't')], 'a')):
            self.assertAlmostEqual(float(got), float(want), places=12)

    def test_a_program_defines_the_ancestors_it_needs(self):
        network = gen_bn.parse(str(SOURCE / 'examples' / 'asia.net'))
        order, _, tables = network
        top = next(v for v in order if not tables[v][0])
        text = gen_bn.to_dice(network, top)
        self.assertEqual(text.count('let '), 1)
        self.assertEqual(text.strip().splitlines()[-1], gen_bn.ident(top).upper())


class FormatTests(unittest.TestCase):
    """The formats of Ace's benchmarks: GeNIe .xdsl, Hugin .hugin, .inst."""

    def scratch(self, name, data):
        path = Path(__file__).resolve().parents[1] / 'tmp' / name
        path.parent.mkdir(exist_ok=True)
        path.write_bytes(data)
        return str(path)

    def test_an_xdsl_table_reads_as_a_hugin_table(self):
        order, variables, tables = gen_bn.parse(self.scratch('two.xdsl', XDSL))
        self.assertEqual(order, ['a', 'b'])
        self.assertEqual(variables['b'], ['u', 'v', 'w'])
        self.assertEqual(tables['b'], (['a'], {('x',): [0.9, 0.1, 0.0], ('y',): [0.2, 0.5, 0.30000000000000004]}))

    def test_a_noisy_max_node_is_refused(self):
        noisy = XDSL.replace(b'<cpt id="b">', b'<noisymax id="b">').replace(b'</cpt>\n  </nodes>', b'</noisymax>\n  </nodes>')
        with self.assertRaisesRegex(SystemExit, 'b is a noisymax node, not a table'):
            gen_bn.parse(self.scratch('noisy.xdsl', noisy))

    def test_a_hugin_file_keeps_percent_signs_in_quotes_and_reads_latin1(self):
        text = ('% K\xf8benhavn\nnet\n{\n}\nnode q\n{\n  label = "Q";\n  states = ("<12%" ">24%"); % two\n}\n'
                'potential (q)\n{\n  data = (0.25 0.75);\n}\n')
        order, variables, tables = gen_bn.parse(self.scratch('pct.hugin', text.encode('latin-1')))
        self.assertEqual(variables['q'], ['<12%', '>24%'])
        self.assertEqual(tables['q'][1][()], [0.25, 0.75])

    def test_doubles_written_in_full_are_read_as_the_decimals_they_mean(self):
        self.assertEqual(str(gen_bn.meant_decimal(0.0009940400000000005)), '0.00099404')
        self.assertEqual(str(gen_bn.meant_decimal(0.6666666666666667)), '0.6666666666666667')
        # Ace's pathfinder.xdsl and blockmap_10_01: rounding, not the table's own gap.
        for row in ([0.0, 0.99403579, 0.00497018, 0.0009940400000000005], [0.9090909090909092, 0.09090909090909088],
                    [0.6666666666666667, 0.3333333333333333]):
            self.assertAlmostEqual(math.fsum(gen_bn.normalized(row, 'x')), 1.0, places=15, msg=row)
        with self.assertRaisesRegex(SystemExit, 'probabilities sum to 100'):
            gen_bn.normalized([1.0] * 15 + [85.0], 'x')   # Ace's diabetes.hugin, in percent

    def test_inst_evidence_conditions_the_program(self):
        """An xdsl network and .inst evidence, against the posterior by hand:
        P(a=x | b=u) = 0.3 * 0.9 / (0.3 * 0.9 + 0.7 * 0.2)."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        network = gen_bn.parse(self.scratch('two.xdsl', XDSL))
        inst = self.scratch('two.inst', gen_bn.write_inst([('b', 'u')]).encode())
        evidence = gen_bn.read_inst(inst, network)
        self.assertEqual(evidence, [('b', 'u')])
        text = gen_bn.to_problog(network, 'all', evidence)
        self.assertIn('evidence(b(u)).', text)
        self.assertNotIn('query(b(', text)
        rows, values, ev, _ = bn_exact.parse(text)
        states, tables = bn_exact.factors(rows, values)
        out, _, _ = bn_exact.marginals(states, tables, ev)
        self.assertAlmostEqual(float(out['a'][states['a'].index('x')]), 0.27 / 0.41, places=15)

    def test_a_uai_bayes_file_reads_as_its_tables(self):
        """The xdsl network above in the UAI format: a function's table is
        its scope's last variable given the others."""
        text = b'BAYES\n2\n2 3\n2\n1 0\n2 0 1\n2\n 0.3 0.7\n6\n 0.9 0.1 0 0.2 0.5 0.3\n'
        (order, variables, tables), forced, offset = gen_bn.parse(self.scratch('two.uai', text))
        self.assertEqual((order, forced, offset), (['x0', 'x1'], [], None))
        self.assertEqual(variables['x1'], ['0', '1', '2'])
        self.assertEqual(tables['x1'], (['x0'], {('0',): [0.9, 0.1, 0.0], ('1',): [0.2, 0.5, 0.3]}))
        with self.assertRaisesRegex(SystemExit, 'UAI table of 5 entries'):
            gen_bn.parse(self.scratch('short.uai', text.replace(b'6\n', b'5\n')))

    def test_a_network_written_as_uai_reads_back_as_its_tables(self):
        """`--uai`, the files Merlin reads: variable i is the network's i-th
        and value u its u-th."""
        network = gen_bn.parse(self.scratch('two.xdsl', XDSL))
        model, evid = gen_bn.write_uai(network, [('b', 'w')])
        self.assertEqual(evid, '1 1 2\n')
        (order, variables, tables), forced, _ = gen_bn.parse(self.scratch('back.uai', model.encode()))
        self.assertEqual((order, forced), (['x0', 'x1'], []))
        self.assertEqual(tables['x0'], ([], {(): [0.3, 0.7]}))
        self.assertEqual(tables['x1'], (['x0'], {('0',): [0.9, 0.1, 0.0], ('1',): [0.2, 0.5, 0.30000000000000004]}))
        evidence = gen_bn.read_evid(self.scratch('back.uai.evid', evid.encode()), (order, variables, tables))
        self.assertEqual(evidence, [('x1', '2')])

    def test_a_markov_network_observes_its_factors(self):
        """tests/regress/markov.pl and markov_ev.pl are this script's
        programs for markov.uai, without and with markov.uai.evid; the
        marginals by enumeration in their comments hold for bn_exact here
        and for tddlog in tests/it/examples.rs."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        from fractions import Fraction
        regress = SOURCE / 'tests' / 'regress'
        program = lambda name: ''.join(
            line for line in (regress / name).read_text().splitlines(keepends=True) if not line.startswith('%'))
        network, forced, offset = gen_bn.parse(str(regress / 'markov.uai'))
        self.assertAlmostEqual(offset, math.log10(4 * 2 * 3 * 2 * 4 * 5), places=14)
        self.assertEqual(forced, [('f0', 'ok'), ('f1', 'ok')])   # the constant factor is dropped
        evidence = forced + gen_bn.read_evid(str(regress / 'markov.uai.evid'), network)
        self.assertEqual(evidence[2:], [('x2', '0')])
        cases = [
            ('markov.pl', 'all', forced,
             {'x0': ['17/65', '48/65'], 'x1': ['6/13', '7/13'], 'x2': ['33/65', '2/13', '22/65']}),
            ('markov_ev.pl', 'x0,x1', evidence, {'x0': ['2/11', '9/11'], 'x1': ['5/33', '28/33']}),
        ]
        for name, query, observed, expected in cases:
            text = gen_bn.to_problog(network, query, observed)
            self.assertEqual(text, program(name))
            self.assertEqual((regress / name).read_text().split('\n', 1)[0] + '\n', gen_bn.offset_line(offset))
            rows, values, ev, _ = bn_exact.parse(text)
            states, tables = bn_exact.factors(rows, values)
            out, _, _ = bn_exact.marginals(states, tables, ev)
            for v, probs in expected.items():
                for u, p in enumerate(probs):
                    self.assertAlmostEqual(float(out[v][states[v].index(f'v_{u}')]), float(Fraction(p)), places=14)
        zero = (regress / 'markov.uai').read_text().replace(' 5 5 5 5 5 5', ' 0 0 0 0 0 0')
        with self.assertRaisesRegex(SystemExit, 'zero everywhere'):
            gen_bn.parse(self.scratch('zero.uai', zero.encode()))

    def test_a_markov_network_written_as_hugin_answers_as_its_program(self):
        """`write_hugin`, the file Ace reads for a UAI network: markov.uai's
        factors become variables of two values whose rows sum to one, and
        the marginals given the factors' `ok` and markov.uai.evid, by
        enumeration of the Hugin file's tables, are markov_ev.pl's."""
        from fractions import Fraction
        regress = SOURCE / 'tests' / 'regress'
        network, forced, _ = gen_bn.parse(str(regress / 'markov.uai'))
        evidence = forced + gen_bn.read_evid(str(regress / 'markov.uai.evid'), network)
        order, variables, tables = gen_bn.parse(self.scratch('markov.net', gen_bn.write_hugin(network).encode()))
        self.assertEqual(order, network[0])
        self.assertEqual(variables['f0'], ['ok', 'not_ok'])
        for v in order:
            for row in tables[v][1].values():
                self.assertAlmostEqual(math.fsum(row), 1.0, places=15)
        given = dict(evidence)
        mass = {v: [0.0] * len(variables[v]) for v in ['x0', 'x1']}
        for world in itertools.product(*(variables[v] for v in order)):
            value = dict(zip(order, world))
            if any(value[v] != u for v, u in given.items()):
                continue
            w = math.prod(tables[v][1][tuple(value[u] for u in tables[v][0])][variables[v].index(value[v])]
                          for v in order)
            for v in mass:
                mass[v][variables[v].index(value[v])] += w
        for v, probs in {'x0': ['2/11', '9/11'], 'x1': ['5/33', '28/33']}.items():
            for u, p in enumerate(probs):
                self.assertAlmostEqual(mass[v][u] / math.fsum(mass[v]), float(Fraction(p)), places=14)

    def test_a_bif_network_written_as_hugin_reads_back_the_same(self):
        """`write_hugin` of a BIF network, the file Ace reads for Roulette's
        networks: the Hugin file's variables, values and tables are the BIF
        file's."""
        bif = """network unknown {}
variable Smoker { type discrete [ 2 ] { True, False }; }
variable Cancer { type discrete [ 3 ] { none, benign, malignant }; }
probability ( Smoker ) { table 0.3, 0.7; }
probability ( Cancer | Smoker ) {
  (True) 0.5, 0.3, 0.2;
  (False) 0.9, 0.08, 0.02;
}
"""
        network = gen_bn.parse(self.scratch('cancer.bif', bif.encode()))
        self.assertEqual(network[1]['Cancer'], ['none', 'benign', 'malignant'])
        again = gen_bn.parse(self.scratch('cancer.net', gen_bn.write_hugin(network).encode()))
        self.assertEqual(again[:2], network[:2])
        for v in network[0]:
            self.assertEqual(again[2][v][0], network[2][v][0])
            for key, row in network[2][v][1].items():
                self.assertEqual([float(p) for p in again[2][v][1][key]], [float(p) for p in row])

    def test_the_offset_line_turns_the_probability_of_the_evidence_into_log10_z(self):
        """For markov.uai, markov_pr.uai and random small Markov networks,
        with and without evidence: Z by enumeration of the UAI file's worlds
        in exact fractions, against the program's P(e) by bn_exact.py (baselines branch) and by
        enumeration of gen_bn.py's tables, plus the offset. markov_pr.uai
        has zeros, a constant factor and a factor of empty scope."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        import random
        regress = SOURCE / 'tests' / 'regress'
        cases = [((regress / 'markov.uai').read_text(), ''),
                 ((regress / 'markov.uai').read_text(), (regress / 'markov.uai.evid').read_text()),
                 ((regress / 'markov_pr.uai').read_text(), ''),
                 ((regress / 'markov_pr.uai').read_text(), (regress / 'markov_pr.uai.evid').read_text())]
        rng = random.Random(7)
        for _ in range(40):
            cases.append(random_markov(rng))
        # The programs in tests/regress, by hand above and in their comments.
        self.assertEqual([brute_force_z(*case) for case in cases[:4]],
                         [325, 165, Fraction(19947, 25), Fraction(11853, 40)])
        for k, (model, evid) in enumerate(cases):
            z = brute_force_z(model, evid)
            want = math.log10(z) if z else -math.inf
            path = self.scratch('pr.uai', model.encode())
            network, forced, offset = gen_bn.parse(path)
            evidence = forced
            if evid:
                evidence = forced + gen_bn.read_evid(self.scratch('pr.uai.evid', evid.encode()), network)
            text = gen_bn.to_problog(network, 'all', evidence)
            rows, values, ev, _ = bn_exact.parse(gen_bn.offset_line(offset) + text)
            states, tables = bn_exact.factors(rows, values)
            res = bn_exact.marginals(states, tables, ev)
            got = [-math.inf if res is None else res[2] + offset,
                   math.log10(pe) + offset if (pe := program_pe(network, evidence)) else -math.inf]
            for g in got:
                if want == -math.inf:
                    self.assertEqual(g, want, msg=(k, model, evid))
                else:
                    self.assertAlmostEqual(g, want, places=12, msg=(k, model, evid))
        self.assertEqual(gen_bn.read_offset(gen_bn.offset_line(0.1) + text), 0.1)
        self.assertIsNone(gen_bn.read_offset(text))
        self.assertEqual(gen_bn.offset_line(None), '')

    def test_a_markov_file_of_conditional_tables_is_written_as_the_network(self):
        """A MARKOV file whose functions are each a table of one variable,
        the child anywhere in the scope, is the Bayesian network: Z is one,
        so the offset is 0 and P(e) is Z given the evidence. Below f0 is an
        identity, a table of x2 given x1 and of x1 given x2; f2 is x2's
        table, so f0 is x1's and x2 the parent. Random networks written so,
        their scopes shuffled, read back with P(e) equal to Z by enumeration.
        A cycle of tables, or a function that is no table, keeps the factor
        encoding."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        import random
        text = ('MARKOV\n3\n2 2 2\n3\n2 2 1\n1 0\n2 2 0\n'
                '4\n 1 0 0 1\n2\n 0.25 0.75\n4\n 0.5 0.125 0.5 0.875\n')
        (order, variables, tables), forced, offset = gen_bn.parse(self.scratch('bn.uai', text.encode()))
        self.assertEqual((order, forced, offset), (['x0', 'x1', 'x2'], [], 0.0))
        self.assertEqual(tables['x0'], ([], {(): [0.25, 0.75]}))
        self.assertEqual(tables['x2'], (['x0'], {('0',): [0.5, 0.5], ('1',): [0.125, 0.875]}))
        self.assertEqual(tables['x1'], (['x2'], {('0',): [1.0, 0.0], ('1',): [0.0, 1.0]}))
        cyclic = 'MARKOV\n2\n2 2\n2\n2 0 1\n2 1 0\n4\n 0.5 0.5 0.25 0.75\n4\n 0.5 0.5 0.25 0.75\n'
        _, forced, offset = gen_bn.parse(self.scratch('cyclic.uai', cyclic.encode()))
        self.assertEqual(forced, [('f0', 'ok'), ('f1', 'ok')])
        self.assertNotEqual(offset, 0.0)
        rng = random.Random(11)
        cases = [(text, ''), (text, '1 1 1')]
        for _ in range(30):
            cases.append(random_network_as_markov(rng))
        for k, (model, evid) in enumerate(cases):
            z = brute_force_z(model, evid)
            network, forced, offset = gen_bn.parse(self.scratch('net.uai', model.encode()))
            self.assertEqual((forced, offset), ([], 0.0), msg=(k, model))
            evidence = gen_bn.read_evid(self.scratch('net.uai.evid', evid.encode()), network) if evid else []
            program = gen_bn.to_problog(network, 'all', evidence)
            rows, values, ev, _ = bn_exact.parse(gen_bn.offset_line(offset) + program)
            states, tables = bn_exact.factors(rows, values)
            res = bn_exact.marginals(states, tables, ev)
            got = -math.inf if res is None else res[2] + offset
            if z == 0:
                self.assertEqual(got, -math.inf, msg=(k, model, evid))
            else:
                self.assertAlmostEqual(got, math.log10(z), places=12, msg=(k, model, evid))

    def test_a_program_that_asks_nothing_breaks_the_colour_symmetry(self):
        """Random Potts networks, k = 2 to 4 values: under `--query none`
        the program fixes a variable and restricts two more
        (`colour_symmetry`), and its P(e) plus the offset, which holds
        log10 k besides, is log10 Z by enumeration of the UAI file's worlds
        in exact fractions. A network with a unary table, a table that is not
        Potts or variables of different sizes keeps its symmetry; so does a
        program with queries or evidence (gen_bn.py's main)."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        import random
        rng = random.Random(5)
        broke = 0
        for c in range(60):
            model = random_potts(rng)
            z = brute_force_z(model)
            path = self.scratch('potts.uai', model.encode())
            whole, _, _ = gen_bn.parse(path)
            network, forced, offset = gen_bn.parse(path, colours=True)
            broke += network != whole
            text = gen_bn.to_problog(network, 'none', forced)
            self.assertNotIn('query(', text)
            rows, values, ev, _ = bn_exact.parse(gen_bn.offset_line(offset) + text)
            states, tables = bn_exact.factors(rows, values)
            res = bn_exact.marginals(states, tables, ev)
            got = [-math.inf if res is None else res[2] + offset,
                   math.log10(pe) + offset if (pe := program_pe(network, forced)) else -math.inf]
            for g in got:
                if z == 0:
                    self.assertEqual(g, -math.inf, msg=(c, model))
                else:
                    self.assertAlmostEqual(g, math.log10(z), places=12, msg=(c, model))
        self.assertGreater(broke, 50)
        # tests/regress/potts.pl, which tests/it/evidence_probability.rs runs
        # through tddlog, is this script's program for potts.uai.
        regress = SOURCE / 'tests' / 'regress'
        network, forced, offset = gen_bn.parse(str(regress / 'potts.uai'), colours=True)
        self.assertEqual(forced, [(f'f{j}', 'ok') for j in range(7)])
        written = (regress / 'potts.pl').read_text()
        self.assertEqual(gen_bn.to_problog(network, 'none', forced),
                         ''.join(line for line in written.splitlines(keepends=True) if not line.startswith('%')))
        self.assertEqual(written.split('\n', 1)[0] + '\n', gen_bn.offset_line(offset))
        z = brute_force_z((regress / 'potts.uai').read_text())
        self.assertEqual(z, Fraction(225, 2))
        self.assertAlmostEqual(math.log10(program_pe(network, forced)) + offset, math.log10(z), places=13)
        # A colouring of a triangle and a pendant with three colours: v1 is
        # x1 (degree 3), v2 x0, v3 x2 (their common neighbour). 3! = 6
        # colourings of the triangle, times 2 for x3; the program counts 2,
        # a third of them.
        triangle = ('MARKOV\n4\n3 3 3 3\n4\n2 0 1\n2 1 2\n2 0 2\n2 1 3\n'
                    + '9\n 0 1 1 1 0 1 1 1 0\n' * 4)
        self.assertEqual(brute_force_z(triangle), 12)
        path = self.scratch('triangle.uai', triangle.encode())
        network, forced, offset = gen_bn.parse(path, colours=True)
        self.assertEqual(network[2]['x1'], ([], {(): [1.0, 0.0, 0.0]}))
        self.assertEqual(network[2]['x0'], ([], {(): [1 / 3, 2 / 3, 0.0]}))
        self.assertAlmostEqual(program_pe(network, forced) * 10 ** offset, 12, places=12)
        # A unary table, a table that is not Potts, or two sizes.
        unary = triangle.replace('4\n2 0 1', '5\n2 0 1').replace('2 1 3\n', '2 1 3\n1 3\n') + '3\n 1 2 3\n'
        skew = triangle[:triangle.rindex('9\n')] + '9\n 0 1 1 1 0 1 1 2 0\n'
        for model in [unary, skew]:
            path = self.scratch('kept.uai', model.encode())
            self.assertEqual(gen_bn.parse(path, colours=True), gen_bn.parse(path), msg=model)
        kind, sizes_, scopes, tables = gen_bn.read_uai(triangle)
        self.assertIsNone(gen_bn.colour_symmetry([3, 3, 3, 2], scopes, tables))
        self.assertIsNotNone(gen_bn.colour_symmetry(sizes_, scopes, tables))
        # gen_bn.py's main breaks the symmetry only for a program that asks
        # nothing and observes nothing.
        import subprocess
        script = str(Path(__file__).resolve().parents[1] / 'scripts' / 'gen_bn.py')
        path = self.scratch('triangle.uai', triangle.encode())
        run = lambda *a: subprocess.run([sys.executable, script, path, *a], capture_output=True, text=True)
        none = run('--query', 'none').stdout
        self.assertNotIn('query(', none)
        self.assertIn('\nx1(v_0).\n', none)
        rows, values, ev, _ = bn_exact.parse(none)
        states, tables = bn_exact.factors(rows, values)
        self.assertAlmostEqual(bn_exact.marginals(states, tables, ev)[2] + gen_bn.read_offset(none),
                               math.log10(12), places=12)
        self.assertNotIn('\nx1(v_0).\n', run().stdout)
        for evid, observes in [('0\n', False), ('1 3 2\n', True)]:
            ev = self.scratch('triangle.uai.evid', evid.encode())
            out = run('--query', 'none', '--evid', ev).stdout
            self.assertEqual('\nx1(v_0).\n' in out, not observes, msg=evid)
        for flags, message in [(['--inst', self.scratch('t.inst', b'')], '--inst and --uai'),
                               (['--format', 'dice'], 'Dice')]:
            r = run('--query', 'none', *flags)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn(message, r.stderr)

    def test_rescale_divides_rows_written_in_percent_or_as_weights(self):
        """Ace's diabetes and munins write tables in percent, its water and
        mildew a root as weights. Against the posterior by hand:
        P(a=x | b=u) = 0.25 * 0.1 / (0.25 * 0.1 + 0.75 * 0.5)."""
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        text = ('net\n{\n}\nnode a\n{\n  states = ("x" "y");\n}\nnode b\n{\n  states = ("u" "v");\n}\n'
                'potential (a)\n{\n  data = (1 3);\n}\npotential (b | a)\n{\n  data = ((10 90)(50 50));\n}\n')
        path = self.scratch('weights.net', text.encode())
        with self.assertRaisesRegex(SystemExit, 'probabilities sum to 4'):
            gen_bn.to_problog(gen_bn.parse(path))
        network = gen_bn.rescaled(gen_bn.parse(path))
        self.assertEqual(network[2]['a'][1][()], [0.25, 0.75])
        self.assertEqual(network[2]['b'][1][('y',)], [0.5, 0.5])
        rows, values, ev, _ = bn_exact.parse(gen_bn.to_problog(network, 'all', [('b', 'u')]))
        states, tables = bn_exact.factors(rows, values)
        out, _, _ = bn_exact.marginals(states, tables, ev)
        self.assertAlmostEqual(float(out['a'][states['a'].index('x')]), 0.025 / 0.4, places=15)
        zero = self.scratch('zero.net', text.replace('(50 50)', '(0 0)').encode())
        with self.assertRaisesRegex(SystemExit, r"b \('y',\): a row of zeros"):
            gen_bn.rescaled(gen_bn.parse(zero))


class ReaderTests(unittest.TestCase):
    def test_the_programs_tddlog_reads_networks_as_are_gen_bn_pys(self):
        """tddlog reads each network file as the program beside it
        (tests/it/networks.rs, its comments left out), which gen_bn.py
        wrote: gen_bn.py still writes it."""
        net, regress, examples = SOURCE / 'tests' / 'networks', SOURCE / 'tests' / 'regress', SOURCE / 'examples'
        offset = '% log10 Z - log10 P(e) = '
        for args, program in [
            ([regress / 'markov.uai'], regress / 'markov.pl'),
            ([regress / 'markov.uai', '--evid', regress / 'markov.uai.evid'], regress / 'markov_ev.pl'),
            ([regress / 'markov_pr.uai'], regress / 'markov_pr.pl'),
            ([regress / 'markov_pr.uai', '--evid', regress / 'markov_pr.uai.evid'], regress / 'markov_pr_ev.pl'),
            ([regress / 'potts.uai', '--query', 'none'], regress / 'potts.pl'),
            ([examples / 'cancer.net', '--query', 'leaves'], examples / 'bn_cancer.pl'),
            ([examples / 'asia.net', '--evidence-file', net / 'asia_leaves.inst'], examples / 'bn_asia_evidence.pl'),
            ([net / 'sprinkler.bif', '--query', 'WetGrass'], net / 'sprinkler_bif.pl'),
            ([net / 'sprinkler.xdsl', '--query', 'leaves', '--evidence-file', net / 'sprinkler.inst'],
             net / 'sprinkler_xdsl.pl'),
            ([net / 'percent.hugin', '--rescale'], net / 'percent.pl'),
            ([net / 'chain.uai', '--query', 'leaves'], net / 'chain.pl'),
            ([net / 'noisy.xdsl'], net / 'noisy.pl'),
        ]:
            a = gen_bn.arguments().parse_args([str(x) for x in args])
            network, off, evidence, _ = gen_bn.model(a)
            written = gen_bn.offset_line(off) + gen_bn.to_problog(network, a.query, evidence)
            lines = program.read_text().splitlines()
            expected = [l for i, l in enumerate(lines) if (i == 0 and l.startswith(offset)) or not l.startswith('%')]
            self.assertEqual(written.splitlines(), expected, program.name)


if __name__ == '__main__':
    unittest.main()
