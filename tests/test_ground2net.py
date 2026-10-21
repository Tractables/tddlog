"""ground2net.py: the network and the Dice programs written from a ground
program answer its queries as the program does.

The reference enumerates the worlds of the program's facts, each world's
model the well-founded model by the alternating fixpoint (two-valued for
the programs here), and conditions on the evidence. The network's
marginals are bn_exact.py's on the network written as gen_bn.py writes a
network's ProbLog program; the Dice programs' are test_dice2problog.py's
enumeration. bench.py's ground route is checked on the files the script
writes.
"""
import itertools
import json
import math
import os
from pathlib import Path
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / 'scripts'))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_bn  # noqa: E402
import ground2net  # noqa: E402
import test_dice2problog  # noqa: E402
try:
    import bn_exact   # on the baselines branch only; the network tests skip without it
except ImportError:
    bn_exact = None

# Negation outside recursion, a fact with rules, a deterministic fact, an
# atom no rule derives, and evidence on an absent atom (false: no
# constraint).
ACYCLIC = """0.3::a.
0.6::b.
0.2::c.
0.5::d.
t.
d :- a, \\+b.
d :- c.
e(1) :- d, b.
e(1) :- \\+a, c, t.
f :- e(1), g.
evidence(e(1),false).
evidence(h,false).
query(d).
query(\\+d).
query(a).
query(f).
query(t).
"""

# Reachability over a cycle of three nodes that needs all three stages: in
# the world with s(3) and every edge, r(3) is derived at stage 1, r(1) at
# stage 2, r(2) at stage 3.
CYCLIC = """0.3::s(1).
0.8::s(3).
0.5::e(1,2).
0.4::e(2,3).
0.7::e(3,1).
0.6::e(2,1).
0.9::e(3,4).
r(1) :- s(1).
r(3) :- s(3).
r(2) :- r(1), e(1,2).
r(3) :- r(2), e(2,3).
r(1) :- r(3), e(3,1).
r(1) :- r(2), e(2,1).
r(4) :- r(3), e(3,4).
evidence(r(4),true).
query(r(1)).
query(r(2)).
query(r(3)).
"""


def reference(text):
    """{query: P(q | e)}, or None where no world satisfies the evidence."""
    facts, rules, evidence, queries = ground2net.parse(text)
    names = list(facts)
    mass, joint = 0.0, dict.fromkeys((k for k, _, _ in queries), 0.0)
    for bits in itertools.product([False, True], repeat=len(names)):
        world = {n for n, b in zip(names, bits) if b}
        weight = math.prod(facts[n] if b else 1 - facts[n] for n, b in zip(names, bits))

        def gamma(m):
            """The least model of the program with \\+b read true where b is not in m."""
            s, grew = set(world), True
            while grew:
                grew = False
                for h, bodies in rules.items():
                    if h not in s and any(all((a in s) if pos else (a not in m) for a, pos in body)
                                          for body in bodies):
                        s.add(h)
                        grew = True
            return s
        true = set()
        while gamma(gamma(true)) != true:
            true = gamma(gamma(true))
        assert gamma(true) == true, 'a well-founded model that is not two-valued'
        if any((a in true) != v for a, v in evidence):
            continue
        mass += weight
        for k, a, pos in queries:
            joint[k] += weight * ((a in true) == pos)
    return None if mass == 0 else {k: p / mass for k, p in joint.items()}


def network_answers(text):
    g, observed, asked = ground2net.build(text)
    net, evidence, cells = ground2net.network(g, observed, asked)
    order, variables, _ = net
    rows, values, ev, _ = bn_exact.parse(gen_bn.to_problog(net, 'all', evidence))
    states, tables = bn_exact.factors(rows, values)
    res = bn_exact.marginals(states, tables, ev)
    if res is None:
        return None
    out, given = res[0], dict(evidence)
    answers = {}
    for key, (v, u) in cells.items():
        st = states.get(gen_bn.ident(v), [])
        if v in given:
            answers[key] = float(given[v] == u)
        else:   # a value no row gives (the constant's false) is not a state of bn_exact.py's
            answers[key] = float(out[gen_bn.ident(v)][st.index(u)]) if u in st else 0.0
    return answers


def dice_answers(text):
    g, observed, asked = ground2net.build(text)
    dist = test_dice2problog.exact(ground2net.dice(g, observed, [s for _, s in asked]))
    tuple_answers = {key: sum(p for v, p in dist.items() if test_dice2problog.leaves(v)[j] is True)
                     for j, (key, _) in enumerate(asked)}
    single = {key: test_dice2problog.exact(ground2net.dice(g, observed, [s])).get(True, 0.0) for key, s in asked}
    return tuple_answers, single


class TranslationTests(unittest.TestCase):
    def check(self, text):
        want = reference(text)
        self.assertIsNotNone(want)
        tuple_answers, single = dice_answers(text)
        for key, p in want.items():
            with self.subTest(query=key):
                self.assertAlmostEqual(tuple_answers[key], p, places=12)
                self.assertAlmostEqual(single[key], p, places=12)
        if bn_exact is None:
            self.skipTest('bn_exact.py is on the baselines branch')
        got = network_answers(text)
        for key, p in want.items():
            with self.subTest(query=key, of='network'):
                self.assertAlmostEqual(got[key], p, places=12)
        return want

    def test_an_acyclic_program_with_negation(self):
        want = self.check(ACYCLIC)
        self.assertEqual(want['f'], 0.0)
        self.assertEqual(want['t'], 1.0)
        self.assertAlmostEqual(want['d'] + want['\\+d'], 1.0, places=12)
        self.assertNotAlmostEqual(want['a'], 0.3, places=3)

    def test_a_cycle_is_unrolled_to_its_least_model(self):
        # with two stages the world above would leave r(2) out, and the
        # answers would differ from the reference's
        self.check(CYCLIC)
        comps = ground2net.components(ground2net.parse(CYCLIC)[1])
        self.assertEqual([sorted(c) for c in comps], [['r(1)', 'r(2)', 'r(3)'], ['r(4)']])

    def test_negation_through_recursion_is_refused(self):
        with self.assertRaisesRegex(ground2net.Refused, 'negation through recursion'):
            ground2net.build('0.5::c.\na :- c, \\+b.\nb :- \\+a.\nquery(a).\n')

    def test_contradictory_evidence_observes_the_constant_false(self):
        g, observed, _ = ground2net.build('0.5::a.\nevidence(a,true).\nevidence(a,false).\nquery(a).\n')
        self.assertEqual(observed, [(ground2net.TRUE, False)])
        g, observed, _ = ground2net.build('0.5::a.\nevidence(z,true).\nquery(a).\n')
        self.assertEqual(observed, [(ground2net.TRUE, False)])

    def test_a_large_network_is_refused(self):
        with patch.object(ground2net, 'LIMIT', 3):
            with self.assertRaisesRegex(ground2net.Refused, 'more than 3 conjunctions'):
                ground2net.build(CYCLIC)

    def test_the_command_writes_each_format(self):
        with tempfile.TemporaryDirectory() as d:
            ground = os.path.join(d, 'p.ground.pl')
            Path(ground).write_text(CYCLIC)
            for form, files in [('uai', ['.uai', '.uai.evid']), ('hugin', ['.net', '.inst']),
                                ('dice', ['.dice']), ('dicemap', ['.dicemap', '__q0.dice', '__q2.dice'])]:
                stem = os.path.join(d, f'p.{form}')
                with patch.object(sys, 'argv', ['ground2net.py', ground, stem, '--format', form]):
                    ground2net.main()
                for suffix in files + ['.json']:
                    self.assertTrue(os.path.exists(stem + suffix), stem + suffix)
                with open(stem + '.json') as f:
                    self.assertEqual(sorted(json.load(f)['queries']), ['r(1)', 'r(2)', 'r(3)'])
            self.assertEqual(Path(d, 'p.dicemap.dicemap').read_text().splitlines()[1], 'p.dicemap__q1.dice bool r(2)')
            with patch.object(sys, 'argv', ['ground2net.py', ground, os.path.join(d, 'n'), '--format', 'uai']), \
                    patch.object(ground2net, 'build', side_effect=ground2net.Refused('no')), \
                    self.assertRaises(SystemExit) as stop:
                ground2net.main()
            self.assertEqual(stop.exception.code, ground2net.REFUSED)


class BenchTests(unittest.TestCase):
    """bench.py's ground route on a program that is neither a network nor a
    Dice program."""

    def test_the_ground_route_times_both_steps_and_reads_the_cells(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location('bench', root / 'scripts' / 'bench.py')
        bench = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(bench)
        real = bench.timed

        def timed(cmd, timeout, **kw):
            if '--ground' in cmd:
                return {'status': 'ok', 'seconds': 0.25, 'stdout': CYCLIC, 'stderr': ''}
            return real(cmd, timeout, **kw)
        with tempfile.TemporaryDirectory() as d:
            args = SimpleNamespace(dir=d, tddlog='tddlog', timeout=60)
            with patch.object(bench, 'timed', side_effect=timed), \
                    patch.object(bench, 'instances', return_value={'g': ('graph', ['gen_graph.py', '3'])}):
                self.assertEqual(bench.ground_format(args, 'g', 'merlin', 'bte'), 'uai')
                self.assertEqual(bench.ground_format(args, 'g', 'merlin', 'bte-pr'), None)
                self.assertEqual(bench.ground_format(args, 'g', 'dice', 'dice'), 'dicemap')
                self.assertEqual(bench.ground_format(args, 'g', 'alea', 'alea'), 'dice')
                self.assertEqual(bench.ground_format(args, 'g', 'pita', 'pita'), None)
                g = bench.ground_route(args, 'g', os.path.join(d, 'g.pl'), 'uai')
                self.assertEqual(g['status'], 'ok')
                self.assertGreater(g['seconds'], 0.25)
                order, model, evid, cells = bench.uai_inputs(args, 'g', os.path.join(d, 'g.pl'))
                self.assertEqual(model, g['stem'] + '.uai')
                self.assertEqual(sorted(cells), ['r(1)', 'r(2)', 'r(3)'])
                self.assertTrue(all(v in order and i in (0, 1) for v, i in cells.values()))   # a query true where its node is the value i
                self.assertEqual(bench.queries(g['ground']), ['r(1)', 'r(2)', 'r(3)'])
                self.assertEqual(bench.dice_source(args, 'g'), os.path.join(bench.ground_dir(args, 'g'), 'g.dice.dice'))
                bench.drop_ground(args, 'g')
                self.assertFalse(os.path.exists(bench.ground_dir(args, 'g')))
                self.assertFalse(bench.GROUNDED)

    def test_a_refused_translation_is_unsupported(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location('bench', root / 'scripts' / 'bench.py')
        bench = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(bench)
        real = bench.timed

        def timed(cmd, timeout, **kw):
            if '--ground' in cmd:
                return {'status': 'ok', 'seconds': 0.1, 'stdout': 'a :- \\+b.\nb :- \\+a.\nquery(a).\n', 'stderr': ''}
            return real(cmd, timeout, **kw)
        with tempfile.TemporaryDirectory() as d:
            args = SimpleNamespace(dir=d, tddlog='tddlog', timeout=60)
            with patch.object(bench, 'timed', side_effect=timed):
                g = bench.ground_route(args, 'w', os.path.join(d, 'w.pl'), 'hugin')
            self.assertEqual(g['status'], 'unsupported')
            self.assertIn('negation through recursion', g['error'])


if __name__ == '__main__':
    unittest.main()
