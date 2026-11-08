"""problog2ppl.py writes ProbLog programs over their facts in Alea.jl,
Roulette, WebPPL, Psi, Pluck and SPPL, and over their tuples in Dice: the
program's text, its facts file (Psi's, Pluck's, SPPL's and Dice's programs
hold their facts) and the queries it answers, in order, and the programs it
refuses. SPPL's and Dice's programs are answered here by enumeration; the
answers are checked on each run against the instance's reference (bench.py's
checks)."""
import ast
import itertools
import math
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
import problog2ppl  # noqa: E402
from problog2ppl import Refused, translate  # noqa: E402
sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_dice2problog import exact  # noqa: E402

# the languages whose programs hold relations: Dice's holds the tuples of the demand's relations as true
RELATIONAL = [lang for lang in problog2ppl.LANGS if lang != 'dice']

PATHS = """0.6::edge(a, b). 0.7::edge(b, c). 0.5::edge(a, c).
path(X, Y) :- edge(X, Y).
path(X, Y) :- edge(X, Z), path(Z, Y).
query(path(a, c)). query(path(b, c)).
"""


class Event:
    """An SPPL event in one world: whether it holds."""
    def __init__(self, holds):
        self.holds = holds

    def __and__(self, other):
        return Event(self.holds and other.holds)

    def __or__(self, other):
        return Event(self.holds or other.holds)

    def __bool__(self):
        return self.holds


class Value:
    """An SPPL variable's value in one world: `x << {v, ...}` its event."""
    def __init__(self, v):
        self.v = v

    def __lshift__(self, values):
        return Event(self.v in values)


def sppl_world(text, world):
    """The names of an SPPL program problog2ppl.py wrote, run as Python in
    one world, as SPPL's compiler reads it (`~=` an assignment), each coin
    drawing the next value of WORLD, an `if` reading the event of the world;
    and whether its evidence holds there."""
    draws, evidence = iter(world), []
    names = {'atomic': lambda loc: Value(loc), 'discrete': lambda d: Value(next(draws)),
             'array': lambda n: [None] * n,
             'condition': evidence.append}
    exec(text.replace('~=', '='), names)
    return names, all(evidence)


def sppl_answers(text):
    """The probabilities of the queries of an SPPL program problog2ppl.py
    wrote, given its evidence, from its worlds, once for each value of each
    coin."""
    dists = sppl_world(text, iter(lambda: 1, None))[0]['coin_d_']
    total, mass = 0.0, None
    for world in itertools.product(*[sorted(d) for d in dists]):
        weight = math.prod(d[v] for d, v in zip(dists, world))
        names, observed = sppl_world(text, world)
        if observed:
            total += weight
            holds = [bool(e) for e in names['queries_']()]
            mass = [m + weight * h for m, h in zip(mass or [0.0] * len(holds), holds)]
    return [m / total for m in mass]


def dice_answers(text):
    """The probabilities that the Booleans a Dice program problog2ppl.py
    wrote returns (nested pairs) are true, given its observations, by
    enumeration (test_dice2problog.py's `exact`)."""
    def leaves(v):
        return leaves(v[0]) + leaves(v[1]) if isinstance(v, tuple) else [v]
    dist = exact(text)
    return [float(sum(p for v, p in dist.items() if leaves(v)[k])) for k in range(len(leaves(next(iter(dist)))))]


class Translation(unittest.TestCase):
    def test_every_language_reads_the_facts_from_a_file(self):
        for lang in problog2ppl.LANGS:
            text, rows, queries = translate(lang, PATHS, 'paths.pl')
            self.assertEqual(queries, ['path(a,c)', 'path(b,c)'], lang)
            if lang in problog2ppl.INLINE:   # Psi, Pluck and SPPL: the facts are in the program
                self.assertEqual(rows, [], lang)
                continue
            self.assertEqual(sorted(rows), sorted(['edge/2\t0.6\ta\tb', 'edge/2\t0.7\tb\tc', 'edge/2\t0.5\ta\tc']), lang)

    def test_psi_programs_hold_their_facts_and_return_their_queries(self):
        text, _, _ = translate('psi', PATHS, 'paths.pl')
        self.assertTrue(text.startswith('// paths.pl: a ProbLog program written in Psi'))
        self.assertEqual(text.count('return queries;'), 1)   # bench.py asks queries[K] a run
        # a, b and c as numbers past every integer of the program, in order
        self.assertIn('edge_facts_1 := [1000000, 1000001, 1000000]: ℤ[];', text)
        self.assertIn('edge_facts_p := [0.6, 0.7, 0.5]: ℝ[];', text)
        self.assertIn('while stage', text)   # path/2 is recursive
        self.assertNotIn('readCSV', text)
        for program, why in [('0.5::p(1.5). query(p(1.5)).', 'not an integer'),
                             ('0.5::e(4). p(Y) :- e(X), Y is X // 2. query(p(2)).', 'integer operation //')]:
            with self.assertRaises(Refused) as e:
                translate('psi', program)
            self.assertIn(why, str(e.exception))

    def test_pluck_programs_hold_their_facts_as_lists_and_ask_their_queries(self):
        text, _, _ = translate('pluck', PATHS, 'paths.pl')
        self.assertTrue(text.startswith(';; paths.pl: a ProbLog program written in Pluck'))
        # one s-expression after another, balanced once the comments are left out
        depth = 0
        for line in text.splitlines():
            for ch in line.split(';', 1)[0]:
                depth += {'(': 1, ')': -1}.get(ch, 0)
                self.assertGreaterEqual(depth, 0, line)
        self.assertEqual(depth, 0)
        self.assertIn('(define-type sym (c_a) (c_b) (c_c))', text)   # the symbols as constructors
        self.assertIn('(Pair (T2 (c_a) (c_b)) (Chance 0.6))', text)
        self.assertIn('(Y (lambda rec rels stage before ->', text)   # path/2 is recursive
        self.assertIn('(query c0 (let ((m (program))) (Marginal (index 0 (snd m)))))', text)
        self.assertIn('(query c1 ', text)
        self.assertIn('(Posterior (index 0 (snd m)) (fst m))',
                      translate('pluck', PATHS + 'evidence(edge(a, b), true).\n')[0])
        for program, why in [('0.5::p(1.5). query(p(1.5)).', 'not an integer'),
                             ('0.5::e(4). p(Y) :- e(X), Y is X // 2. query(p(2)).', 'integer operation //'),
                             ('0.5::e(a). 0.5::e(1). p :- e(X), X > 0. query(p).', 'holds symbols and is compared'),
                             ('0.5::e(20000). p(Y) :- e(X), Y is X + 1. query(p(20001)).', 'above 10000 in arithmetic'),
                             ('0.5::e(3). p(Y) :- e(X), Y is X * -2. query(p(0)).', 'negative integer in arithmetic')]:
            with self.assertRaises(Refused) as e:
                translate('pluck', program)
            self.assertIn(why, str(e.exception))

    def test_pluck_writes_keys_as_constructors_and_compared_integers_as_ranks(self):
        # TPC-H's shape: keys only joined, a date column compared with a constant that no fact holds
        program = ('0.6::ship(1, 19950315). 0.7::ship(2, 19960101). 0.5::ship(3, 19940101).\n'
                   '0.4::order(1, 100000). 0.3::order(2, 100001). 0.2::order(3, 100000).\n'
                   'late(O) :- order(O, _), ship(O, D), D >= 19950201.\n'
                   'q :- late(_).\nquery(q). query(late(2)).\n')
        text, _, _ = translate('pluck', program)
        self.assertIn('(define-type sym (c_1) (c_2) (c_3) (c_100000) (c_100001))', text)
        ranks = problog2ppl.Pluck(problog2ppl.read(program), 'p.pl').ranks
        self.assertEqual(list(ranks.values()), [{19940101: 0, 19950201: 1, 19950315: 2, 19960101: 3}])
        self.assertIn('(Pair (T2 (c_1) 2) (Chance 0.6))', text)
        self.assertIn('(Pair (T2 (c_3) 0) (Chance 0.5))', text)
        self.assertIn('(>= vD 1)', text)   # 19950201, which no fact holds, between the dates it parts
        self.assertIn('(T1 (c_2))', text)   # the query's key a constructor
        # the program with each date its rank answers as the program does
        ranked = program
        for v, r in ranks[next(iter(ranks))].items():
            ranked = ranked.replace(str(v), str(r))
        want = [1 - (1 - 0.4 * 0.6) * (1 - 0.3 * 0.7), 0.3 * 0.7]
        for p in (program, ranked):
            for got, w in zip(sppl_answers(translate('sppl', p)[0]), want):
                self.assertAlmostEqual(got, w)
        # a class that meets arithmetic keeps its values: no ranks
        self.assertEqual(problog2ppl.Pluck(problog2ppl.read(
            '0.5::e(3). p(Y) :- e(X), X > 1, Y is X + 1. query(p(4)).'), 'p.pl').ranks, {})

    def test_sppl_programs_compute_their_joins_before_their_samples(self):
        text, _, _ = translate('sppl', PATHS, 'paths.pl')
        self.assertTrue(text.startswith('# paths.pl: a ProbLog program written in SPPL'))
        # sppl_dice.py's plan: the queries are the events of queries_()
        self.assertIn('\n# queries\n# component 0 bool query0\n# component 1 bool query1\n', text)
        constants, commands = text.split('\ntrue_ ~= atomic(loc=1)\n')
        self.assertNotIn('~=', constants)   # SPPL takes a constant only before the first sample
        self.assertIn("edge_ft = [('a', 'b'), ('b', 'c'), ('a', 'c')]", constants)
        self.assertIn('comp2_t = fix_(lambda ts_: ', constants)   # path/2 is recursive
        self.assertIn('for k_ in range(len(comp2_steps)):', commands)
        # path(a, b) is not read, and no tuple of path/2 reads one of its own cycle: three instances, a stage each
        steps = sppl_world(text, iter(lambda: 1, None))[0]['comp2_steps']
        self.assertEqual([(q, s) for q, _, s, _ in steps], [('path', 1)] * 3)
        self.assertNotIn('\ndef ', text)   # no statement SPPL's compiler drops
        for got, want in [(sppl_answers(text), [0.5 + 0.5 * 0.42, 0.7]),
                          (sppl_answers(translate('sppl', PATHS + 'evidence(edge(a, b), true).\n')[0]),
                           [0.5 + 0.5 * 0.7, 0.7])]:
            for g, w in zip(got, want):
                self.assertAlmostEqual(g, w)
        # a probabilistic rule, an annotated disjunction, a negation, a cycle, a symbol with `~=`
        program = PATHS + ("0.3::edge(c, a).\n0.7::reach(X) :- path(a, X).\npick(b).\n"
                           "0.2::color(X, red) ; 0.5::color(X, blue) :- pick(X).\n"
                           "node(a). node(b). node(c).\nlonely(X) :- node(X), \\+ path(X, _).\n"
                           "0.5::s('x ~= y').\nquery(reach(c)). query(color(b, blue)). query(lonely(b)).\n"
                           "query(path(c, b)). query(s('x ~= y')).\n")
        pc = 0.5 + 0.5 * 0.42
        got = sppl_answers(translate('sppl', program)[0])
        for g, want in zip(got, [pc, 0.7, 0.7 * pc, 0.5, 0.3, 0.3 * 0.6, 0.5]):
            self.assertAlmostEqual(g, want)
        # with edge(c, a), the nine tuples of path/2 go in three cycles of three, each in three stages: its 16
        # instances in each stage
        names = sppl_world(translate('sppl', program)[0], iter(lambda: 1, None))[0]
        self.assertEqual(sorted(s for k, v in names.items() if k.endswith('_steps') for q, _, s, _ in v if q == 'path'),
                         sorted([1, 2, 3] * 16))
        # the demand: weather(sun,T) for the T its callers ask
        weather = ('0.5::weather(sun,0) ; 0.5::weather(rain,0) <- true.\n'
                   '0.6::weather(sun,T) ; 0.4::weather(rain,T) <- T>0, Tprev is T-1, weather(sun,Tprev).\n'
                   '0.2::weather(sun,T) ; 0.8::weather(rain,T) <- T>0, Tprev is T-1, weather(rain,Tprev).\n'
                   'query(weather(sun,2)).\n')
        sun1 = 0.5 * 0.6 + 0.5 * 0.2
        self.assertAlmostEqual(sppl_answers(translate('sppl', weather)[0])[0], sun1 * 0.6 + (1 - sun1) * 0.2)

    def test_sppl_programs_nest_few_sums(self):
        # SPPL's compiler writes each binary operation back in a parenthesis of its own, and Python's parser
        # refuses more than 200 nested: cora's sametitle/2 has 559 rules, each a list of matches
        program = '0.5::c.\n' + ''.join(f'e{i}.\nq :- c, e{i}.\n' for i in range(300)) + 'query(q).\n'
        text = translate('sppl', program)[0]

        def depth(node):
            return isinstance(node, ast.BinOp) + max([depth(c) for c in ast.iter_child_nodes(node)], default=0)
        self.assertLess(depth(ast.parse(text.replace('~=', '='))), 200)
        self.assertAlmostEqual(sppl_answers(text)[0], 0.5)

    def test_dice_programs_spell_out_the_relations_and_iterate_the_steps(self):
        text, rows, queries = translate('dice', PATHS, 'paths.pl')
        self.assertTrue(text.startswith('// paths.pl: a ProbLog program written in Dice'))
        self.assertIn('// path/2: a recursion through data', text)
        self.assertIn('let edge_a_b = flip 0.6 in', text)
        self.assertNotIn('fun ', text)
        for program, want in [(PATHS, [0.5 + 0.5 * 0.42, 0.7]),
                              (PATHS + 'evidence(edge(a, b), true).\n', [0.5 + 0.5 * 0.7, 0.7])]:
            for got, w in zip(dice_answers(translate('dice', program)[0]), want):
                self.assertAlmostEqual(got, w)
        # a probabilistic rule, an annotated disjunction, a negation, a cycle, a symbol that is no name
        program = PATHS + ("0.3::edge(c, a).\n0.7::reach(X) :- path(a, X).\npick(b).\n"
                           "0.2::color(X, red) ; 0.5::color(X, blue) :- pick(X).\n"
                           "node(a). node(b). node(c).\nlonely(X) :- node(X), \\+ path(X, _).\n"
                           "0.5::s('x ~= y').\nquery(reach(c)). query(color(b, blue)). query(lonely(b)).\n"
                           "query(path(c, b)). query(s('x ~= y')).\n")
        pc = 0.5 + 0.5 * 0.42
        text = translate('dice', program)[0]
        self.assertIn('discrete(0.2, 0.5, 0.3)', text)
        for got, want in zip(dice_answers(text), [pc, 0.7, 0.7 * pc, 0.5, 0.3, 0.3 * 0.6, 0.5]):
            self.assertAlmostEqual(got, want)
        # a step recursion whose steps are alike: infected(T) of infected(T - 1), a coin of each step's own
        # (cure(T), spread(T)) a flip in the function; the first step its fact, the others iterated from it
        sick = ('0.3::infected(0).\n0.4::cure(T) :- between(1, 3, T).\n'
                'infected(T) :- between(1, 3, T), T0 is T - 1, infected(T0), \\+ cure(T).\n'
                '0.2::infected(T) :- between(1, 3, T).\nquery(infected(1)).\nquery(infected(3)).\n')
        text = translate('dice', sick)[0]
        self.assertIn('// infected/1: a recursion over a step (infected/1 argument 1), f_infected from step 1, '
                      'iterated between the steps the program reads\n', text)
        self.assertIn('fun f_infected(s: bool) {', text)
        self.assertIn('iterate(f_infected, ', text)
        p, want = 0.3, []
        for _ in range(3):
            p = 1 - (1 - p * 0.6) * 0.8
            want.append(p)
        for got, w in zip(dice_answers(text), [want[0], want[2]]):
            self.assertAlmostEqual(got, w)
        # steps that differ, keep(T)'s probability: an argument of the function, a call a step
        keep = ('0.5::x(0).\n0.9::keep(1). 0.8::keep(2). 0.7::keep(3).\n'
                'x(T) :- between(1, 3, T), T0 is T - 1, x(T0), keep(T).\nquery(x(3)).\n')
        text = translate('dice', keep)[0]
        self.assertIn('fun f_x_(s: bool, a1: bool) {', text)
        self.assertNotIn('iterate(', text)
        self.assertAlmostEqual(dice_answers(text)[0], 0.5 * 0.9 * 0.8 * 0.7)
        # a recursion none of whose tuples holds: said so, its query false
        text = translate('dice', PATHS + 'r(X) :- edge(X, Y), r(Y).\nquery(r(a)).\n')[0]
        self.assertIn('// r/1: a recursion none of whose tuples holds in any world\n', text)
        self.assertAlmostEqual(dice_answers(text)[-1], 0.0)
        # an observation of a step between: the state read there, the iteration resumed from it
        weather = ('0.5::weather(sun,0) ; 0.5::weather(rain,0) <- true.\n'
                   '0.6::weather(sun,T) ; 0.4::weather(rain,T) <- T>0, Tprev is T-1, weather(sun,Tprev).\n'
                   '0.2::weather(sun,T) ; 0.8::weather(rain,T) <- T>0, Tprev is T-1, weather(rain,Tprev).\n'
                   'evidence(weather(rain,2), true).\nquery(weather(sun,4)).\nquery(weather(sun,1)).\n')
        sun = [0.5]
        for _ in range(4):
            sun.append(sun[-1] * 0.6 + (1 - sun[-1]) * 0.2)
        rain2 = 1 - sun[2]
        sun1 = sun[1] * 0.4 / (sun[1] * 0.4 + (1 - sun[1]) * 0.8)   # P(sun1 | rain2), Bayes
        sun3 = 0.2   # from rain2
        text = translate('dice', weather)[0]
        self.assertGreater(rain2, 0)
        self.assertEqual(text.count('iterate(f_weather, '), 1)
        for got, w in zip(dice_answers(text), [sun3 * 0.6 + (1 - sun3) * 0.2, sun1]):
            self.assertAlmostEqual(got, w)
        # a program a query, as bench.py runs ground2net.py's
        each = problog2ppl.Dice(problog2ppl.read(sick), 'sick.pl').each()
        self.assertEqual([q for q, _ in each], ['infected(1)', 'infected(3)'])
        for (_, text), w in zip(each, [want[0], want[2]]):
            self.assertAlmostEqual(dice_answers(text)[0], w)
        for program, why in [('0.5::e(4). p(Y) :- e(X), Y is X / 2. query(p(2)).', 'translator: the division /'),
                             ('0.5::e. query(e). query(f).', 'f/0')]:
            with self.assertRaises(Refused) as e:
                translate('dice', program)
            self.assertIn(why, str(e.exception))

    def test_alea_programs_are_in_the_form_alea_bench_reads(self):
        text, _, _ = translate('alea', PATHS, 'paths.pl')
        self.assertIn('const QUERY = :pr\n', text)
        self.assertEqual(text.count('\nconst FACTS = '), 1)   # alea_bench.jl --file FACTS PATH
        self.assertIn('\nfunction model()\n', text)
        self.assertIn('observe(ev)', text)
        self.assertNotIn('\nmodule ', text)   # alea_bench.jl evaluates it in a module of its own
        self.assertIn('queries = (holds(path, (:a, :c)), holds(path, (:b, :c)))', text)

    def test_the_other_neck_reads_as_the_usual_one(self):
        program = "0.3::a. 0.2::r <- a.\nh <- r. q('x <- y') :- h. % a <- b\nquery(h). query(q('x <- y')).\n"
        for lang in problog2ppl.LANGS:
            _, _, queries = translate(lang, program)
            self.assertEqual(queries, ['h', "q('x <- y')"], lang)

    def test_a_clause_its_callers_bind_is_evaluated_for_their_arguments(self):
        # ProbLog's tutorial HMM: T is bound by the query and by the call, not by the body
        weather = ('0.5::weather(sun,0) ; 0.5::weather(rain,0) <- true.\n'
                   '0.6::weather(sun,T) ; 0.4::weather(rain,T) <- T>0, Tprev is T-1, weather(sun,Tprev).\n'
                   '0.2::weather(sun,T) ; 0.8::weather(rain,T) <- T>0, Tprev is T-1, weather(rain,Tprev).\n'
                   'query(weather(sun,3)).\n')
        prog = problog2ppl.read(weather)
        demand = [r for r in prog.rules if r.demand]
        self.assertEqual(sorted({r.heads[0][0] for r in demand}), [('demand_2', 1), ('demand_3', 1)])
        # the query's argument, and each call's from each of the two clauses that call it
        self.assertEqual(len(demand), 2 + 2 * 2)
        guarded = [r for r in prog.rules if r.probs and len(r.probs) == 2 and r.steps]
        self.assertTrue(all(isinstance(r.steps[0], problog2ppl.Atom) and r.steps[0].pred[0].startswith('demand_')
                            for r in guarded))
        self.assertTrue(all(r.heads[0][0] != r.steps[0].pred for r in guarded))
        # and cplint's coin: an annotated disjunction of facts with a variable, called with it bound
        coin = '0.9::fair(C) ; 0.1::biased(C).\n0.5::toss(coin).\nh(C) :- toss(C), biased(C).\nquery(h(coin)).\n'
        for lang in RELATIONAL:
            for program in (weather, coin):
                text, _, queries = translate(lang, program)
                self.assertIn('demand_', text, lang)
        # in Dice the weather of a day a function of the day before's, iterated; the coin's choice a discrete
        sun = 0.5
        for _ in range(3):
            sun = sun * 0.6 + (1 - sun) * 0.2
        for program, want in [(weather, sun), (coin, 0.5 * 0.1)]:
            text = translate('dice', program)[0]
            self.assertNotIn('demand_', text)
            self.assertAlmostEqual(dice_answers(text)[0], want)
        self.assertIn('iterate(f_weather, ', translate('dice', weather)[0])
        with self.assertRaises(Refused) as e:
            translate('alea', '0.5::p(X) :- X > 1.\nq :- p(_).\nquery(q).\n')
        self.assertIn('leaves an argument', str(e.exception))

    def test_an_anonymous_head_variable_is_evaluated_for_its_callers_arguments(self):
        # Pluck's sorted list: `list(N, _) :- elements(N), ends(N).`, called with its second argument bound
        program = ('0.4::ends(1).\n0.3::ok(0).\nat(0, 3).\n'
                   'list(1, _) :- ends(1).\n'
                   'list(I, Min) :- at(I, V), V >= Min, ok(I), I1 is I + 1, list(I1, V).\n'
                   'query(list(0, 2)).\nquery(list(1, 7)).\n')
        for lang in problog2ppl.LANGS:
            text, _, queries = translate(lang, program)
            self.assertEqual('demand_' in text, lang in RELATIONAL, lang)
            self.assertEqual(queries, ['list(0,2)', 'list(1,7)'], lang)
        # list(0, 2) through ok(0) and list(1, 3), list(1, 7) on ends(1) alone
        for answers in (sppl_answers(translate('sppl', program)[0]), dice_answers(translate('dice', program)[0])):
            for got, want in zip(answers, [0.3 * 0.4, 0.4]):
                self.assertAlmostEqual(got, want)

    def test_a_head_variable_a_test_reads_before_the_recursive_call_is_demanded(self):
        # Pluck's sorted list's step: D is bound only by the recursive call, which K < D bounds
        program = ('0.5::stops(K) :- between(0, 3, K).\n'
                   'step(D, D) :- stops(D).\n'
                   'step(K, D) :- K < D, \\+stops(K), K1 is K + 1, step(K1, D).\n'
                   'query(step(0, 2)).\n')
        prog = problog2ppl.read(program)
        self.assertEqual([r.heads[0][0][1] for r in prog.rules if r.demand][:1], [2])   # demand_N(K, D)
        for lang in RELATIONAL:
            self.assertIn('demand_', translate(lang, program)[0], lang)
        self.assertAlmostEqual(sppl_answers(translate('sppl', program)[0])[0], 0.5 ** 3)
        self.assertAlmostEqual(dice_answers(translate('dice', program)[0])[0], 0.5 ** 3)
        # and the port itself, in the languages its logic-program route gives it in
        port = (scripts.parent / 'data' / 'alea' / 'pluck' / 'sorted_list.pl').read_text()
        for lang in ('roulette', 'webppl', 'psi', 'sppl', 'dice'):
            self.assertEqual(translate(lang, port)[2], ['generated'], lang)

    def test_refusals(self):
        for program, why in [('p([1, 2]). query(p(_)).', ''),
                             ('0.5::e(1, 2). p(X) :- e(X, _). query(p(Y)).', 'translator: a query that is not ground')]:
            for lang in problog2ppl.LANGS:
                with self.assertRaises(Refused, msg=lang) as e:
                    translate(lang, program)
                self.assertIn(why, str(e.exception))

    def test_the_command_line_writes_the_program_and_its_facts(self):
        with tempfile.TemporaryDirectory() as d:
            pl, out, facts = (os.path.join(d, f) for f in ('p.pl', 'p.jl', 'p.facts'))
            Path(pl).write_text(PATHS)
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'alea', pl, out, facts],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertEqual(len(Path(facts).read_text().splitlines()), 3)
            self.assertTrue(Path(out).read_text().startswith('# p.pl: a ProbLog program'))
            Path(pl).write_text('p([1]). query(p(_)).')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'webppl', pl, out, facts],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, problog2ppl.REFUSED, r.stderr)
            # Psi's program holds its facts: no facts file
            Path(pl).write_text(PATHS)
            psi = os.path.join(d, 'p.psi')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'psi', pl, psi],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertIn('def main() {', Path(psi).read_text())
            pluck = os.path.join(d, 'p.pluck')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'pluck', pl, pluck],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertIn('(define (program)', Path(pluck).read_text())
            sppl = os.path.join(d, 'p.sppl')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'sppl', pl, sppl],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertIn('\nqueries_ = lambda: [', Path(sppl).read_text())
            # Dice: one program, or a program a query listed in a dicemap, as bench.py's run_dice_circuit reads it
            dice = os.path.join(d, 'p.dice')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'dice', pl, dice],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertEqual(len(dice_answers(Path(dice).read_text())), 2)
            dicemap = os.path.join(d, 'p.dicemap')
            r = subprocess.run([sys.executable, str(scripts / 'problog2ppl.py'), 'dicemap', pl, dicemap],
                               capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(r.stdout.split(), ['path(a,c)', 'path(b,c)'])
            self.assertEqual(Path(dicemap).read_text(), 'p__q0.dice bool path(a,c)\np__q1.dice bool path(b,c)\n')
            for j, want in enumerate([0.5 + 0.5 * 0.42, 0.7]):
                self.assertAlmostEqual(dice_answers(Path(d, f'p__q{j}.dice').read_text())[0], want)


if __name__ == '__main__':
    unittest.main()
