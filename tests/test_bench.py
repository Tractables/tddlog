"""Benchmark statistics must include the work after atom compilation."""
import importlib.util
from contextlib import ExitStack, redirect_stdout
import fcntl
import io
import math
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import mock_open, patch


scripts = Path(__file__).resolve().parents[1] / 'scripts'
sys.path.insert(0, str(scripts))
from source_tree import SOURCE  # noqa: E402
spec = importlib.util.spec_from_file_location('bench', scripts / 'bench.py')
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


class StatsTests(unittest.TestCase):
    def test_evidence_network_peak(self):
        stats = bench.parse_stats(
            'largest formula 1803 pairs (final 1803), 0.020s\n'
            'evidence and queries: 0.100s, evidence 14479 pairs, '
            'largest formula 16103 pairs\n')
        self.assertEqual(stats, {
            'pairs': 16103, 'compile_pairs': 1803, 'final_pairs': 1803,
            'compile_s': 0.020, 'query_s': 0.100,
            'evidence_pairs': 14479, 'query_pairs': 16103,
        })

    def test_compilation_can_be_larger(self):
        stats = bench.parse_stats(
            'largest formula 3879443 pairs (final 3879443), 2.0s\n'
            'evidence and queries: 1.0s, evidence 995827 pairs, '
            'largest formula 2169404 pairs\n')
        self.assertEqual(stats['pairs'], 3879443)
        self.assertEqual(stats['query_pairs'], 2169404)

    def test_older_compile_only_output(self):
        stats = bench.parse_stats('largest formula 840 pairs, 0.010s\n')
        self.assertEqual(stats['pairs'], 840)
        self.assertEqual(stats['compile_pairs'], 840)
        self.assertNotIn('query_pairs', stats)

    def test_auto_trials_include_query_failures(self):
        stats = bench.parse_stats('trial: closed indicators=off, watch=16, work=100, phase=query, size, 0.010s\n'
                                  'trial: families indicators=all, watch=16, work=90, phase=query, finished, 0.020s\n')
        self.assertEqual([r['outcome'] for r in stats['trials']], ['size', 'finished'])
        self.assertEqual(stats['trials'][0]['phase'], 'query')
        self.assertEqual(stats['trials'][1]['indicators'], 'all')

    def test_auto_trials_include_stopped_compiles(self):
        stats = bench.parse_stats('trial: closed indicators=off, watch=16, work=100, phase=compile, '
                                  'components 3/7, size, 0.010s\n'
                                  'trial: dynamic:minfill indicators=off, watch=16, work=90, phase=compile, '
                                  'components 0/1 at round 2, stopped, 0.020s\n'
                                  'trial: td indicators=off, watch=32, work=80, phase=query, steps 2/9, stopped, '
                                  '0.030s\n'
                                  'trial: td indicators=off, watch=32, work=80, phase=query, finished, 0.030s\n')
        self.assertEqual([(r['vtree'], r['phase'], r['outcome']) for r in stats['trials']],
                         [('closed', 'compile', 'size'), ('dynamic:minfill', 'compile', 'stopped'),
                          ('td', 'query', 'stopped'), ('td', 'query', 'finished')])

    def test_auto_trials_with_a_cap(self):
        """An attempt that may hold operations past its watch prints its cap."""
        stats = bench.parse_stats('trial: families:0 indicators=all, watch=1048576, work=10023284, phase=query, '
                                  'stopped, 0.112s\n'
                                  'trial: families:0 indicators=all, watch=16777216, cap=268435456, work=431016123, '
                                  'phase=query, finished, 2.255s\n')
        self.assertEqual([(r['watch'], r.get('cap'), r['outcome']) for r in stats['trials']],
                         [(1048576, None, 'stopped'), (16777216, 268435456, 'finished')])

    def test_layout_seconds(self):
        """auto's seconds per layout, a families layout's searches and
        weighing, and td's search, as `--stats` prints them."""
        stats = bench.parse_stats(
            'families: 9 families in 4 groups over 6 indicators, 0 stray facts; td over the groups width 2 '
            '(treewidth at least 2), '
            '4 bags, 3 edges, layout cost 1.000e2 (the least of 2); the most copies one group reads 2, '
            'the most facts in one group 1, 0 branches of 0 groups; searched 0 units charged, 0.000s, '
            'laid out and weighed 0.001s\n'
            'layout: families:0 indicators=all, 0.002s\n'
            'td: width 3 (treewidth at least 2), 5 bags, widest root path 4 atoms, 9 vertices + 12 edges, '
            '38750000 units budgeted, 38800000 charged, 0.050s\n'
            'layout: td indicators=off, 0.051s\n'
            'td: width 2 (treewidth at least 2), 4 bags, widest root path 3 atoms, 9 vertices + 12 edges, '
            'min-degree at the bound, no search, 1200 charged, 0.000s\n'
            'families: 9 families in 4 groups over 6 indicators, 0 stray facts; td over the groups width 2, '
            '4 bags, 3 edges, layout cost 1.000e2 (the least of 2); the most copies one group reads 2, '
            'the most facts in one group 1, 0 branches of 0 groups; min-fill 5170 units charged, 0.002s; '
            'searched 0 units charged, 0.000s, laid out and weighed 0.001s\n'
            'td: width 4 (treewidth at least 2), 5 bags, widest root path 4 atoms, 9 vertices + 12 edges, '
            'eliminations min-degree 700, min-fill 900 units, 38750000 units budgeted, 38801600 charged, 0.051s\n'
            'td: width 2 (treewidth at least 2), 4 bags, widest root path 3 atoms, 9 vertices + 12 edges, '
            'eliminations min-degree 640 units, an elimination at the bound, no search, 640 charged, 0.000s\n'
            'dynamic: min-fill 1350 units charged\n')
        self.assertEqual(stats['layouts'], [dict(vtree='families:0', indicators='all', seconds=0.002),
                                            dict(vtree='td', indicators='off', seconds=0.051)])
        self.assertEqual(stats['families_layouts'], [dict(charged=0, search_s=0.0, weigh_s=0.001),
                                                      dict(min_fill_charged=5170, min_fill_s=0.002, charged=0,
                                                           search_s=0.0, weigh_s=0.001)])
        self.assertEqual(stats['td_searches'], [dict(charged=38800000, seconds=0.05), dict(charged=1200, seconds=0.0),
                                                dict(min_degree_charged=700, min_fill_charged=900, charged=38801600,
                                                     seconds=0.051),
                                                dict(min_degree_charged=640, charged=640, seconds=0.0)])
        self.assertEqual(stats['dynamic_layouts'], [dict(charged=1350)])

    def test_indicator_counts(self):
        self.assertEqual(bench.parse_stats('indicators: all, 27 copies of 9 atoms\n'), {
            'indicator_policy': 'all', 'indicator_copies': 27, 'indicator_atoms': 9,
        })


class DiceTests(unittest.TestCase):
    def test_a_returned_tuple_answers_each_component(self):
        rows = bench.parse_dice('================[ Joint Distribution ]================\n'
                                'Value\tProbability\t\n(true, true)\t0.\t\n(true, false)\t0.475\t\n'
                                '(false, true)\t0.475\t\n(false, false)\t0.05\t\n')
        self.assertEqual(rows, {'(true,true)': 0.0, '(true,false)': 0.475, '(false,true)': 0.475,
                                '(false,false)': 0.05})
        self.assertEqual(bench.dice_marginals(rows, 2), [0.475, 0.475])
        self.assertIsNone(bench.dice_marginals(rows, 3))

    def test_nodices_runtimes_are_not_values(self):
        # noDice's table of a returned pair (its ladder-N-p), then its runtimes
        rows = bench.parse_dice('================[ Maximum Probabilities (noDice) ]================\n'
                                'Value   Probability\n(true, true)    0.000999\n(true, false)   0.498251\n'
                                '(false, true)   0.498251\n(false, false)  0.002499\n\n'
                                '================[ Runtimes ]================\nTask                    Time (seconds)\n'
                                'Compilation     0.569023\nModel Checking  0.000029\nnoDice (total)  0.569052\n')
        self.assertEqual(rows, {'(true,true)': 0.000999, '(true,false)': 0.498251, '(false,true)': 0.498251,
                                '(false,false)': 0.002499})
        for p in bench.dice_marginals(rows, 2):
            self.assertAlmostEqual(p, 0.49925, places=12)

    def test_values_that_are_not_boolean_tuples_have_no_marginals(self):
        rows = bench.parse_dice('Value\tProbability\t\n0\t0.25\t\n1\t0.75\t\n')
        self.assertEqual(rows, {'0': 0.25, '1': 0.75})
        self.assertIsNone(bench.dice_marginals(rows, 2))


class PplTests(unittest.TestCase):
    """The ppl suite: the ports of Alea.jl's programs, Alea.jl on its own
    programs, Dice on an Alea.jl program's circuit (the bitblast suite's),
    and the files the suites name."""

    def args(self, **kw):
        base = dict(suite='ppl', only=[], families=[], dir='bench', timeout=30, alea='', dice='')
        base.update(kw)
        return SimpleNamespace(**base)

    def test_psi_asks_a_logic_programs_queries_one_run_each(self):
        # problog2ppl.py's Psi program returns the array of its queries; a run asks queries[K]
        outs = ['p(x) = 3/10·δ(1)[x]+7/10·δ(0)[x]\n', 'p(x) = δ(0)[x]\n']
        seen, cmds = [], []

        def timed(cmd, timeout, cwd=None):
            cmds.append(cmd[:-1])
            seen.append(Path(cmd[-1]).read_text())
            return {'status': 'ok', 'seconds': 1.5, 'stdout': outs[len(seen) - 1], 'stderr': ''}
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', side_effect=timed):
            prog = Path(d, 'p.lp.psi.psi')
            prog.write_text('def main() {\n  queries := [q1, q2];\n  return queries;\n}\n')
            r = bench.run_lp_psi(self.args(dir=d, psi='prlimit --stack=unlimited psi {file}'), 'p', str(prog),
                                 ['p(1)', 'p(a, b)'])
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(r['answers'], {'p(1)': 0.3, 'p(a,b)': 0.0})
        self.assertEqual(r['seconds'], 3.0)
        self.assertEqual([s.count('return queries[0];') for s in seen], [1, 0])
        self.assertEqual([s.count('return queries[1];') for s in seen], [0, 1])
        self.assertEqual(cmds, [['prlimit', '--stack=unlimited', 'psi', '--dp']] * 2)   # its exact discrete back end

    def test_the_suite_names_programs_that_exist(self):
        spec = bench.instances(self.args())
        self.assertEqual({family for family, _ in spec.values()}, {'arith', 'pluck', 'nodice'})
        # noDice's diamonds and ladders are the roulette suite's rl_diamond and rl_ladder
        self.assertEqual(len(spec), 75)
        for name, (family, gargs) in spec.items():
            if family == 'pluck':
                # a ProbLog port (and hmm's Dice port) of the Alea.jl program beside it
                self.assertEqual(gargs[0], 'gen_pluck.py')
                self.assertTrue(Path(gargs[1]).is_file() and gargs[1].startswith(bench.PLUCK_DIR), name)
            else:
                self.assertEqual(gargs[0], 'dice2problog.py')
                self.assertTrue(Path(gargs[1]).is_file(), name)
            if family != 'nodice':
                self.assertTrue(Path(bench.ALEA_OF[tuple(gargs)][0]).is_file(), name)
        self.assertEqual(sorted(n for n, (_, g) in spec.items() if bench.writes_dice(g)), ['pluck_hmm'])

    def test_a_port_with_places_is_its_program_and_the_instances_arguments(self):
        spec = bench.instances(self.args())
        family, gargs = spec['arith_gcd_medium']
        self.assertEqual(gargs, ['dice2problog.py', os.path.join(bench.ARITH_DIR, 'gcd.dice'), '11', '1024', '16'])
        self.assertEqual(bench.folded(self.args(dir='b'), 'arith_gcd_medium', gargs, 'auto'),
                         (gargs[1], os.path.join('b', 'arith_gcd_medium.args')))
        self.assertEqual(spec['arith_book'][1], ['dice2problog.py', os.path.join(bench.ARITH_DIR, 'book.dice')])
        # caesar's sizes are one Alea.jl program and its LENGTH, each a port of its own
        self.assertEqual(bench.alea_program(self.args(), spec['arith_caesar_huge'][1]),
                         [os.path.join(bench.ARITH_DIR, 'caesar.jl'), 'LENGTH=250'])
        self.assertEqual(bench.alea_program(self.args(), spec['arith_gcd_medium'][1]),
                         [os.path.join(bench.ARITH_DIR, 'gcd_medium.jl')])
        self.assertEqual(bench.alea_program(self.args(), spec['pluck_hmm'][1]),
                         [os.path.join(bench.PLUCK_DIR, 'hmm.jl')])
        self.assertIsNone(bench.alea_program(self.args(suite='bitblast'), spec['arith_book'][1]))

    def test_places_are_filled_outside_comments(self):
        text = '// $1 is the width\nlet x = uniform $1 0 $2 in x // $2\n'
        self.assertEqual(bench.fill_places(text, ['8', '-3']), '// $1 is the width\nlet x = uniform 8 0 -3 in x // $2\n')
        with self.assertRaisesRegex(ValueError, r'\$3'):
            bench.fill_places('let x = $3 in x\n', ['1', '2'])
        with self.assertRaisesRegex(ValueError, r'2 arguments for the places \[1\]'):
            bench.fill_places('let x = $1 in x\n', ['1', '2'])

    def test_the_ports_systems_read_their_own_programs(self):
        # the ProbLog family reads the port's program; Alea.jl and Pluck read their own,
        # and the Dice family the Dice port where there is one (hmm), none of them charged
        # for a ground program. The capture's arguments are rooted (capture.py), and name
        # the same ports.
        hmm, pcfg = (os.path.join(bench.PLUCK_DIR, f) for f in ('hmm.pl', 'pcfg.pl'))
        for g in (['gen_pluck.py', hmm], ['gen_pluck.py', os.path.relpath(hmm, bench.ROOT)]):
            self.assertIsNone(bench.ground_form(g, 'alea', 'cudd'))
            self.assertIsNone(bench.ground_form(g, 'pluck', 'default'))
            self.assertIsNone(bench.ground_form(g, 'dice', 'bdd'))
            self.assertTrue(bench.writes_dice(g))
            self.assertFalse(bench.grounds_dice(g))
        for g in (['gen_pluck.py', pcfg], ['gen_pluck.py', os.path.relpath(pcfg, bench.ROOT)]):
            self.assertEqual(bench.ground_form(g, 'dice', 'bdd'), 'dicemap')
            self.assertEqual(bench.ground_form(g, 'storm', 'exact'), 'dice')
            self.assertFalse(bench.writes_dice(g))
            self.assertTrue(bench.grounds_dice(g))

    def test_the_dice_systems_read_problog2ppls_dice_of_a_logic_program(self):
        # problog2ppl.py writes the Dice programs of a logic program, uncharged; the systems answer in the
        # order of the queries it lists; a program it refuses takes the ground program's route
        args = SimpleNamespace(dir='', timeout=30, translation_timeout=None, pr=False)
        with tempfile.TemporaryDirectory() as d:
            args.dir = d
            Path(d, 'p.pl').write_text('0.6::e(a, b). 0.7::e(b, c).\np(X, Y) :- e(X, Y).\np(X, Y) :- e(X, Z), p(Z, Y).\n'
                                       'query(p(a, c)). query(p(b, c)).\n')
            for form in bench.DICE_FORMS:
                r = bench.dice_route(args, 'p', str(Path(d, 'p.pl')), form)
                self.assertEqual((r['status'], r['seconds']), ('ok', 0.0), form)
                self.assertEqual(bench.queries(r['ground']), ['p(a,c)', 'p(b,c)'])
                self.assertTrue(r['stem'].endswith('.lp'))
            self.assertEqual(Path(d, 'ground-p', 'p.lp.dicemap').read_text(),
                             'p.lp__q0.dice bool p(a,c)\np.lp__q1.dice bool p(b,c)\n')
            self.assertIn('flip 0.6', Path(d, 'ground-p', 'p.lp.dice').read_text())
            Path(d, 'q.pl').write_text('0.5::p([1]).\nq :- p([1]).\nquery(q).\n')
            self.assertIsNone(bench.dice_route(args, 'q', str(Path(d, 'q.pl')), 'dicemap'))

    def test_alea_program_answers_name_the_ports_queries(self):
        # a Boolean, then a number of four values whose two non-constant bits
        # give worlds of values 3, -3 (1 modulo 4), 0 and 3
        answers = {'q1': 0.5, 'q2(0)': 0.125, 'q2(1)': 0.25, 'q2(2)': 0.375, 'q2(3)': 0.25}
        values = {'q2(0)': 3, 'q2(1)': -3, 'q2(2)': 0, 'q2(3)': 3}
        with tempfile.TemporaryDirectory() as d:
            p = Path(d, 'p.pl')
            p.write_text('query(ok).\nquery(v(0)).\nquery(v(1)).\nquery(v(2)).\nquery(v(3)).\n')
            self.assertEqual(bench.alea_program_answers(str(p), answers, values),
                             {'ok': 0.5, 'v(0)': 0.375, 'v(1)': 0.25, 'v(2)': 0.0, 'v(3)': 0.375})
            with self.assertRaisesRegex(ValueError, r'no value of q2\(0\)'):
                bench.alea_program_answers(str(p), answers)
            # a world of no mass, whose value `run` does not print
            self.assertEqual(bench.alea_program_answers(str(p), dict(answers, **{'q2(3)': 0.0}),
                                                        {k: v for k, v in values.items() if k != 'q2(3)'}),
                             {'ok': 0.5, 'v(0)': 0.375, 'v(1)': 0.25, 'v(2)': 0.0, 'v(3)': 0.125})
            p.write_text('query(ok).\nquery(done).\n')
            with self.assertRaisesRegex(ValueError, 'ask none'):
                bench.alea_program_answers(str(p), answers, values)

    def test_the_queries_components_are_booleans_and_numbers(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d, 'p.pl')
            p.write_text('query(ok).\nquery(v( 0)).\nquery(v(1)).\nquery(done).\nquery(w(0)).\n')
            self.assertEqual(bench.query_components(str(p)), [(None, 'ok'), ('v', 2), (None, 'done'), ('w', 1)])

    def test_the_circuit_dice_programs_answer_bools_and_worlds(self):
        outputs = {
            'p__d1.dice': 'Value\tProbability\t\ntrue\t0.25\t\nfalse\t0.75\t\n',
            # Two bits, the first the most significant; a missing row is 0.
            'p__d2.dice': 'Value\tProbability\t\n(true, false)\t0.5\t\n(false, true)\t0.375\t\n'
                          '(false, false)\t0.125\t\n',
        }
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'p.dicemap').write_text('p__d1.dice bool q1\np__d2.dice joint q2 2\n')
            args = self.args(dir=d, dice='dice {file}')

            def timed(cmd, timeout):
                return {'status': 'ok', 'seconds': 1.5, 'stdout': outputs[Path(cmd[-1]).name], 'stderr': ''}
            with patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_dice_circuit(args, 'p', os.path.join(d, 'p.dicemap'))
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(r['seconds'], 3.0)
        self.assertEqual(r['answers'], {'q1': 0.25, 'q2(0)': 0.125, 'q2(1)': 0.375, 'q2(2)': 0.5, 'q2(3)': 0.0})

    def test_a_network_with_observations_runs_a_program_per_queried_variable(self):
        # asia with half its leaves observed: one program for each other
        # leaf, which observes nothing and returns the leaf beside whether
        # the observations hold; the rows where they hold, over their sum,
        # are the queries' answers, also where their sum is under 1e-6.
        asia = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        spec = {'asia_ev': ('bnev', ['gen_bn.py', asia, '--query', 'leaves', '--evidence', 'half'])}
        with tempfile.TemporaryDirectory() as d:
            program = Path(d, 'asia_ev.pl')
            network = bench.gen_bn.parse(asia)
            evidence = bench.gen_bn.sampled_evidence(network, 'half', 1)
            program.write_text(bench.gen_bn.to_problog(network, 'leaves', evidence))
            seen = []

            def timed(cmd, timeout):
                seen.append(Path(cmd[-1]).read_text())
                rows = [('(0, true)', 2 ** -24), ('(1, true)', 3 * 2 ** -24), ('(0, false)', 0.5), ('(1, false)', 0.5)]
                return {'status': 'ok', 'seconds': 0.5, 'stderr': '',
                        'stdout': 'Value\tProbability\t\n' + ''.join(f'{v}\t{p!r}\t\n' for v, p in rows)}
            with patch.object(bench, 'instances', return_value=spec), patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_dice(self.args(dir=d, dice='dice {file}'), 'asia_ev', str(program))
            qs = [q.replace(' ', '') for q in bench.queries(str(program))]
        self.assertEqual(r['status'], 'ok', r)
        self.assertEqual(len(seen), len({q.split('(')[0] for q in qs}))
        self.assertTrue(evidence)
        self.assertFalse(any('observe' in text for text in seen))
        self.assertEqual(sorted(r['answers']), sorted(qs))
        self.assertEqual(sorted(set(r['answers'].values())), [0.25, 0.75])

    def test_nodice_reads_a_network_program_that_observes_the_evidence(self):
        # noDice prints six decimals, too few for the joint of a variable and
        # improbable observations: its programs observe them, its rows the answers.
        asia = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        spec = {'asia_ev': ('bnev', ['gen_bn.py', asia, '--query', 'leaves', '--evidence', 'half'])}
        with tempfile.TemporaryDirectory() as d:
            program = Path(d, 'asia_ev.pl')
            network = bench.gen_bn.parse(asia)
            evidence = bench.gen_bn.sampled_evidence(network, 'half', 1)
            program.write_text(bench.gen_bn.to_problog(network, 'leaves', evidence))
            seen = []

            def timed(cmd, timeout):
                seen.append(Path(cmd[-1]).read_text())
                return {'status': 'ok', 'seconds': 0.5, 'stdout': 'Value\tProbability\t\n0\t0.25\t\n1\t0.75\t\n',
                        'stderr': ''}
            with patch.object(bench, 'instances', return_value=spec), patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_dice(self.args(dir=d, nodice='nDice.exe {file}'), 'asia_ev', str(program), 'nodice')
            qs = [q.replace(' ', '') for q in bench.queries(str(program))]
        self.assertEqual(r['status'], 'ok', r)
        self.assertTrue(seen and all(text.count('observe') == len(evidence) for text in seen))
        self.assertEqual(sorted(r['answers']), sorted(qs))
        self.assertEqual(sorted(set(r['answers'].values())), [0.25, 0.75])

    def test_observations_of_probability_zero_are_an_error_after_every_program(self):
        # Each program still runs (a capture answers every run with an
        # empty table), and the instance is an error.
        asia = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        spec = {'asia_ev': ('bnev', ['gen_bn.py', asia, '--query', 'leaves', '--evidence', 'half'])}
        with tempfile.TemporaryDirectory() as d:
            program = Path(d, 'asia_ev.pl')
            network = bench.gen_bn.parse(asia)
            evidence = bench.gen_bn.sampled_evidence(network, 'half', 1)
            program.write_text(bench.gen_bn.to_problog(network, 'leaves', evidence))
            seen = []

            def timed(cmd, timeout):
                seen.append(cmd[-1])
                return {'status': 'ok', 'seconds': 0.5, 'stdout': '', 'stderr': ''}
            with patch.object(bench, 'instances', return_value=spec), patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_dice(self.args(dir=d, dice='dice {file}'), 'asia_ev', str(program))
            qs = bench.queries(str(program))
        self.assertEqual(r['status'], 'error', r)
        self.assertIn('probability 0.0', r['error'])
        self.assertEqual(len(seen), len({q.split('(')[0] for q in qs}))
        self.assertEqual(r['seconds'], 0.5 * len(seen))

    def test_a_row_of_the_wrong_width_is_an_error(self):
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'p.dicemap').write_text('p__d1.dice joint q1 3\n')
            result = {'status': 'ok', 'seconds': 0.1, 'stdout': 'Value\tProbability\t\n(true, false)\t1.\t\n',
                      'stderr': ''}
            with patch.object(bench, 'timed', return_value=result):
                r = bench.run_dice_circuit(self.args(dir=d, dice='dice {file}'), 'p', os.path.join(d, 'p.dicemap'))
        self.assertEqual(r['status'], 'error')

    def test_a_translated_system_answers_a_circuit_as_dice_does(self):
        name = next(iter(bench.BITBLAST))   # a circuit Alea.jl exports
        got = {'a.dice': ([{'true': 0.25, 'false': 0.75}], {'seconds': 1.0}),
               'b.dice': ([{'0': 0.125, '1': 0.375, '2': 0.5, '3': 0.0}], {'seconds': 2.0,
                                                                         'stats': {'inproc_seconds': 0.5}})}
        calls = []

        def masses(args, label, source, joint, budget):
            calls.append((label, joint))
            return got[label]
        with tempfile.TemporaryDirectory() as d:
            Path(d, f'{name}.dicemap').write_text('a.dice bool q1\nb.dice joint q2 2\n')
            r = bench.run_translated(self.args(dir=d, suite='bitblast'), name, None, 'sys', masses)
        self.assertEqual(calls, [('a.dice', False), ('b.dice', True)])
        self.assertEqual(r, {'status': 'ok', 'seconds': 3.0, 'stats': {'inproc_seconds': 0.5},
                             'answers': {'q1': 0.25, 'q2(0)': 0.125, 'q2(1)': 0.375, 'q2(2)': 0.5, 'q2(3)': 0.0}})

    def test_roulette_answers_a_circuit_through_its_dice_programs(self):
        # The joint program's two Booleans are asked as one int, its first
        # Boolean the most significant bit (dice2roulette.py --joint).
        name = next(iter(bench.BITBLAST))
        outputs = {'a': 'roulette 1 true 0.25\nroulette 1 false 0.75\n',
                   'b': 'roulette 1 0 0.125\nroulette 1 1 0.375\nroulette 1 2 0.5\nroulette 1 3 0.0\n'}
        timer = {'a': 'roulette_timer: program 0.25 s\n', 'b': 'roulette_timer: program 0.5 s\n'}
        runs = []

        def timed(cmd, timeout, cwd=None):
            if 'raco' in cmd:
                return {'status': 'ok', 'seconds': 9.0, 'stdout': '', 'stderr': ''}
            self.assertEqual(cmd[:2], ['racket', bench.ROULETTE_TIMER])
            runs.append(Path(cmd[-1]).read_text())
            first = Path(cmd[-1]).name[0]
            return {'status': 'ok', 'seconds': 1.5, 'stdout': outputs[first], 'stderr': timer[first]}
        with tempfile.TemporaryDirectory() as d:
            Path(d, f'{name}.dicemap').write_text('a.dice bool q1\nb.dice joint q2 2\n')
            Path(d, 'a.dice').write_text('let x = flip 0.25 in x\n')
            Path(d, 'b.dice').write_text('let x = flip 0.5 in let y = flip 0.25 in (x, y)\n')
            args = self.args(dir=d, roulette='racket {file}', suite='bitblast')
            with patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_roulette(args, name, None, 'rsdd')
        self.assertEqual(r, {'status': 'ok', 'seconds': 3.0, 'stats': {'inproc_seconds': 0.75},
                             'answers': {'q1': 0.25, 'q2(0)': 0.125, 'q2(1)': 0.375, 'q2(2)': 0.5, 'q2(3)': 0.0}})
        self.assertIn("(report dists '(#f))", runs[0])
        self.assertRegex(runs[1], r'\(distribution \(list (\S+-y) (\S+-x)\)\)')
        self.assertIn("(report dists '(2))", runs[1])

    def test_a_chain_whose_translation_passes_its_bound_is_the_harness_time_out(self):
        # dice2prism.py's translation of a chain, made once for every engine,
        # passed its bound after it wrote the first component: each engine
        # checks that one, and its record is the harness's time-out, the
        # rest of the program not handed to the checker.
        late = harness_timeout = bench.harness_timeout_record('dice2prism.py', 'no translation within 30 s')
        ok = {'status': 'ok', 'seconds': 1.0, 'stdout': ''}
        with patch.dict(bench.CHAINS, clear=True), \
                patch.object(bench, 'translation', return_value=('bool\n', late)) as made, \
                patch.object(bench, 'timed', return_value=ok) as timed, \
                patch.object(bench, 'checker_command', side_effect=lambda args, system, variant, *files: list(files)), \
                patch.object(bench, 'checker_distribution', return_value={'true': 0.5, 'false': 0.5}):
            for variant in ('hybrid', 'sparse'):
                comps, r = bench.checker_masses('prism', variant)(self.args(dir='/d', translation_timeout=30.0), 'p',
                                                                  '/p.dice', False, 30.0)
                self.assertIsNone(comps)
                self.assertEqual(r, {**harness_timeout, 'seconds': 1.0})
        self.assertEqual(made.call_count, 1)
        self.assertEqual(made.call_args.args[1:], ('dice2prism.py', ['/p.dice', '--each', '/d/p.dtmc']))
        self.assertEqual([c.args[0] for c in timed.call_args_list], [['/d/p.dtmc0.prism', '/d/p.dtmc0.props']] * 2)

    def test_a_system_run_by_component_has_its_translation_bounded(self):
        # Geni on a program of three Boolean components, each run 1 s: one
        # translation writes them all and lists each one written (--each).
        # Where it passed its bound after two, those two run and the record
        # is the harness's time-out with their 2 s; where it passed it
        # before the first, Geni is handed nothing; where a component is
        # refused, the record is the refusal after the runs of those before it.
        late = bench.harness_timeout_record('dice2geni.py', 'no translation within 30 s')
        refused = {'status': 'unsupported', 'seconds': 0, 'error': 'dice2geni.py: cannot express the program: x'}
        for listing, failed, ran in [('bool\nbool\nbool\n', None, 3), ('bool\nbool\n', late, 2), ('', late, 0),
                                     ('bool\n', refused, 1)]:
            with tempfile.TemporaryDirectory() as d, \
                    patch.object(bench, 'translation', return_value=(listing, failed)) as made, \
                    patch.object(bench, 'program_command', side_effect=lambda args, system, file: ['geni', file]), \
                    patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 1.0, 'stdout': ''}) as timed:
                masses = bench.by_component('geni', 'dice2geni.py', 'gir', lambda n: [], lambda out, n: {0: 0.5, 1: 0.5})
                comps, r = masses(self.args(dir=d), 'p', '/p.dice', True, 30.0)
            self.assertEqual(made.call_args.args[1:], ('dice2geni.py', ['/p.dice', '--each', f'{d}/p.geni', '--joint']))
            self.assertEqual([c.args for c in timed.call_args_list],
                             [(['geni', f'{d}/p.geni{k}.gir'], 30.0 - k) for k in range(ran)])
            if failed is None:
                self.assertEqual((len(comps), r), (3, {'seconds': 3.0}))
            else:
                self.assertIsNone(comps)
                self.assertEqual(r, {**failed, 'seconds': float(ran)})

    def test_a_translation_past_its_bound_keeps_the_lines_it_printed(self):
        # what a translator printed before its bound, without a line it had not ended
        late = subprocess.TimeoutExpired(['dice2geni.py'], 30.0, output=b'bool\nint 3\nbo')
        with patch.object(bench.subprocess, 'run', side_effect=late):
            out, r = bench.translation(self.args(translation_timeout=30.0), 'dice2geni.py', ['p.dice', '--each', 'p.'])
        self.assertEqual(out, 'bool\nint 3\n')
        self.assertTrue(bench.harness_timeout(r) and bench.translation_limit(r), r)
        with patch.object(bench.subprocess, 'run', side_effect=subprocess.TimeoutExpired(['x'], 30.0)):
            self.assertEqual(bench.translation(self.args(), 'dice2geni.py', [])[0], '')

    def test_a_program_the_language_cannot_express_is_unsupported(self):
        refused = SimpleNamespace(returncode=bench.REFUSED, stdout='', stderr='cannot express the program: recursion')
        with patch.object(bench.subprocess, 'run', return_value=refused):
            out, r = bench.translation(self.args(), 'dice2genfer.py', ['p.dice', '--components'])
        self.assertEqual(out, '')
        self.assertEqual(r['status'], 'unsupported')
        self.assertIn('recursion', r['error'])
        self.assertEqual(bench.fmt(r), 'n/a')

    def test_a_translator_that_fails_says_how_it_ended(self):
        """A translator killed by a signal leaves nothing on stderr (bp3rf's
        five `dice2roulette.py: ` on taint): its record names the signal; one
        that exits without a word names its exit status; an exception's
        traceback ends in its type and message. Roulette's and Alea.jl's
        translations run through `translation` too."""
        cases = [(SimpleNamespace(returncode=-11, stdout='', stderr=''), 'dice2roulette.py: killed by SIGSEGV'),
                 (SimpleNamespace(returncode=-9, stdout='', stderr='partial\n'), 'dice2roulette.py: killed by SIGKILL: partial'),
                 (SimpleNamespace(returncode=1, stdout='', stderr=''), 'dice2roulette.py: exit 1 with nothing on stderr'),
                 (SimpleNamespace(returncode=1, stdout='', stderr='Traceback (most recent call last):\n  ...\nKeyError\n'),
                  'dice2roulette.py: Traceback (most recent call last):\n  ...\nKeyError')]
        with tempfile.TemporaryDirectory() as d:
            for t, error in cases:
                with patch.object(bench.subprocess, 'run', return_value=t) as run:
                    r = bench.roulette_translate(self.args(), 'p.dice', os.path.join(d, 'p.rkt'))
                self.assertEqual(run.call_args.args[0][1:], [str(bench.source_tree.script('dice2roulette.py')), 'p.dice'])
                self.assertEqual(r, {'status': 'error', 'seconds': 0, 'error': error})
            self.assertEqual(os.listdir(d), [])
            with patch.object(bench.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout='(x)\n', stderr='')):
                self.assertIsNone(bench.roulette_translate(self.args(), 'p.dice', os.path.join(d, 'p.rkt')))
            self.assertEqual(Path(d, 'p.rkt').read_text(), '(x)\n')

    def test_alea_runs_the_program_and_reports_three_times(self):
        # caesar's Alea.jl program at its LENGTH, its number's worlds named as the port's queries
        out = ('alea_q q1(0) 0.25\nalea_value q1(0) 3\nalea_q q1(1) 0.75\nalea_value q1(1) -3\nalea_seconds 12.5\n'
               'alea_compile_seconds 12.25\nalea_warm_seconds 0.0625\nalea_bdd_evidence 7\n'
               'alea_bdd_query_max 5\nalea_bdd_joint_max 9\nalea_bdd_peak 40\n')
        args = self.args(alea='julia --project=A {file}', timeout=30)
        with tempfile.TemporaryDirectory() as d, \
                patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 20.0, 'stdout': out,
                                                           'stderr': ''}) as timed:
            port = Path(d, 'arith_caesar_huge.pl')
            port.write_text(''.join(f'query(v({k})).\n' for k in range(4)))
            r = bench.run_alea(args, 'arith_caesar_huge', str(port))
        cmd = timed.call_args.args[0]
        self.assertEqual(cmd[:2], ['julia', '--project=A'])
        self.assertTrue(cmd[2].endswith('alea_bench.jl'))
        self.assertEqual(cmd[3:8], ['run', '--warm-below', '10', '--set', 'LENGTH=250'])
        self.assertTrue(cmd[8].endswith('data/alea/arith/caesar.jl'))
        self.assertEqual(r['answers'], {'v(0)': 0.0, 'v(1)': 0.75, 'v(2)': 0.0, 'v(3)': 0.25})
        self.assertEqual(r['stats'], {
            'inproc_seconds': 12.5, 'jit_seconds': 12.25, 'warm_seconds': 0.062, 'bdd_evidence': 7,
            'bdd_query_max': 5, 'bdd_joint_max': 9, 'bdd_peak': 40,
            'printed': ['alea_value q1(0) 3', 'alea_value q1(1) -3', 'alea_seconds 12.5', 'alea_compile_seconds 12.25',
                        'alea_warm_seconds 0.0625', 'alea_bdd_evidence 7', 'alea_bdd_query_max 5',
                        'alea_bdd_joint_max 9', 'alea_bdd_peak 40']})

    def test_gen_writes_alea_circuits_only_under_alea(self):
        name, (_, gargs) = next(iter(bench.BITBLAST.items()))
        with self.assertRaisesRegex(SystemExit, 'under --alea'):
            bench.gen(self.args(suite='bitblast', only=[name], cnf=False))
        # the ppl suite's arith and pluck programs are ports, not circuits
        with self.assertRaisesRegex(SystemExit, 'none is selected'):
            bench.gen(self.args(only=['arith_book'], cnf=False, alea='julia {file}'))
        with tempfile.TemporaryDirectory() as d, patch.object(bench.subprocess, 'run') as run, \
                redirect_stdout(io.StringIO()):
            bench.gen(self.args(suite='bitblast', only=[name], cnf=False, alea='julia {file}', dir=d))
        cmd = run.call_args.args[0]
        self.assertEqual(cmd[2:], ['export', '--set', gargs[2], gargs[1], os.path.abspath(os.path.join(d, name))])

    def test_the_table_shows_alea_times_in_the_process(self):
        recs = [{'instance': 'arith_book', 'family': 'arith', 'system': 'alea', 'variant': 'cudd', 'status': 'ok',
                 'seconds': 20.0, 'inproc_seconds': 12.5, 'warm_seconds': 0.06, 'answers': {'q1': 0.5}},
                {'instance': 'arith_book', 'family': 'arith', 'system': 'tddlog', 'variant': 'auto', 'status': 'ok',
                 'seconds': 0.1, 'answers': {'q1': 0.5}}]
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'r.jsonl')
            path.write_text(''.join(bench.json.dumps(r) + '\n' for r in recs))
            out = io.StringIO()
            with redirect_stdout(out):
                bench.table(SimpleNamespace(results=[str(path)], ref='alea:cudd', pr_ref='exact:cliquetree',
                                            approx=[]))
        head, _, row = out.getvalue().splitlines()
        self.assertIn('alea:cudd in-process | alea:cudd warm', head)
        self.assertIn('| 20.00 | 12.50 | 0.06 | 0.10 |', row)


class ScratchTests(unittest.TestCase):
    """A translation written for one call of a system (a query's, a
    component's or a variable's program, the program of one of a circuit's
    Dice programs, a logic program's) is gone once the run returns, whether
    its calls succeeded or failed: one kept for each query or component
    leaves a copy of the program for each beside the instance, and a run
    over many instances fills the disk."""

    OK = {'status': 'ok', 'seconds': 1.0, 'stdout': '', 'stderr': ''}
    FAILED = {'status': 'error', 'seconds': 1.0, 'error': 'killed by SIGSEGV', 'stdout': '', 'stderr': ''}

    def args(self, d):
        return SimpleNamespace(dir=d, timeout=30, translation_timeout=None, pr=False, suite='ppl', psi='psi {file}',
                               geni='geni {file}', pluck='julia {file}', sppl='python {file}', webppl='node {file}',
                               dice='dice {file}')

    def assert_leaves_nothing(self, run, stdout='', **patches):
        """Runs run(args) in an empty directory, its system's calls answered
        with `stdout` and then failed, and checks that it leaves no file
        there. Each of `patches` replaces that name of bench.py: a function
        as its side effect, another value as what it returns."""
        for outcome in ({**self.OK, 'stdout': stdout}, dict(self.FAILED)):
            with self.subTest(status=outcome['status']), tempfile.TemporaryDirectory() as d, ExitStack() as stack:
                stack.enter_context(patch.object(bench, 'timed', return_value=outcome))
                for name, value in patches.items():
                    how = {'side_effect': value} if callable(value) else {'return_value': value}
                    stack.enter_context(patch.object(bench, name, **how))
                run(self.args(d))
                self.assertEqual(os.listdir(d), [])

    def test_psi_on_a_logic_programs_queries(self):
        # problog2ppl.py's program, then a copy of it a query (NAME.lp.psiK.psi)
        def translation(args, script, argv):
            Path(argv[2]).write_text('def main() {\n  return queries;\n}\n')
            return 'q1\nq2\n', None
        self.assert_leaves_nothing(lambda args: bench.run_lp(args, 'p', 'p.pl', 'psi'), 'p(x) = δ(1)[x]\n',
                                   translation=translation)

    def test_webppl_on_a_logic_programs_program_and_facts(self):
        def translation(args, script, argv):
            for f in argv[2:]:
                Path(f).write_text('x\n')
            return 'q1\n', None
        self.assert_leaves_nothing(lambda args: bench.run_lp(args, 'p', 'p.pl', 'webppl'), translation=translation)

    def test_psi_on_its_own_programs_components(self):
        with tempfile.TemporaryDirectory() as own:
            Path(own, 'p.psi').write_text('def main() {\n  ps := [a, b];\n  return ps;\n}\n')
            self.assert_leaves_nothing(lambda args: bench.run_psi_own(args, 'arith_p', 'p.pl'), 'p(x) = δ(1)[x]\n',
                                       psi_program=str(Path(own, 'p.psi')),
                                       query_components=[(None, 'q1'), (None, 'q2')], queries=['q1', 'q2'])

    def test_psi_on_its_linear_regressions_variables(self):
        self.assert_leaves_nothing(lambda args: bench.run_psi_linreg(args, 'bb_linreg_w4', 'p.pl'), 'E[a] = 1/2\n',
                                   translation=('def main() {}\n', None))

    def test_a_system_run_by_component(self):
        # every component the translation listed, and the one it left
        # unfinished where it failed
        late = bench.harness_timeout_record('dice2geni.py', 'no translation within 30 s')
        masses = bench.by_component('geni', 'dice2geni.py', 'gir', lambda n: [], lambda out, n: {0: 0.5, 1: 0.5})
        for failed in (None, late):
            def translation(args, script, argv):
                for k in range(3):
                    Path(f'{argv[2]}{k}.gir').write_text('x\n')
                return 'bool\nbool\n' if failed else 'bool\nbool\nbool\n', failed
            self.assert_leaves_nothing(lambda args: masses(args, 'p', '/p.dice', False, 30.0),
                                       translation=translation)

    def test_the_program_of_a_circuits_dice_program(self):
        for masses in (bench.pluck_masses, bench.sppl_masses, bench.webppl_masses):
            with self.subTest(system=masses.__name__):
                self.assert_leaves_nothing(lambda args: masses(args, 'p__d1.dice', '/p__d1.dice', False, 30.0),
                                           translation=('bool\n', None))
        for outcome in (self.OK, self.FAILED):
            with self.subTest(system='roulette', status=outcome['status']), tempfile.TemporaryDirectory() as d, \
                    patch.object(bench, 'translation', return_value=('#lang roulette\n', None)), \
                    patch.object(bench, 'roulette_timed', return_value={**outcome, 'inproc_seconds': 0.5}):
                bench.roulette_masses(self.args(d), 'p__d1.dice', '/p__d1.dice', False, 30.0)
                self.assertEqual(os.listdir(d), [])

    def test_dice_on_a_networks_variables(self):
        # a program a queried variable (NAME__V.dice)
        with patch.object(bench.gen_bn, 'dice_programs', return_value=[('a', 'A\n'), ('b', 'B\n')]):
            self.assert_leaves_nothing(lambda args: bench.run_dice(args, 'n', 'n.pl'),
                                       instances={'n': ('bn', ['gen_bn.py', 'n.bif'])}, queries=['a(0)', 'b(0)'])


class BitblastTests(unittest.TestCase):
    """The bitblast suite: the bit-blasting paper's programs on ladders of
    size, each instance the program run with one constant set."""

    def args(self, **kw):
        base = dict(suite='bitblast', only=[], families=[], dir='bench', timeout=30, alea='', dice='')
        base.update(kw)
        return SimpleNamespace(**base)

    def test_the_ladders_name_programs_and_constants_that_exist(self):
        spec = bench.instances(self.args())
        families = {}
        for name, (family, gargs) in spec.items():
            families.setdefault(family, []).append(name)
            self.assertEqual(gargs[0], 'alea_bench.jl')
            source = Path(gargs[1]).read_text()
            # The last argument is the constant the family is laddered by;
            # the ones before it are fixed for the family (a tree's dt).
            for setting in gargs[2:]:
                const, value = setting.split('=')
                self.assertEqual(len(re.findall(rf'^const {const} = ', source, re.M)), 1, (name, const))
            # An instance is named by the width of its numbers (w), the
            # precision plus the integer bits of the program's DistFix, or
            # by the value of its size constant (t, p, n, s).
            m = re.fullmatch(r'bb_(\w+)_([wtpns])(\d+)', name)
            self.assertIsNotNone(m, name)
            if m.group(2) != 'w':
                self.assertEqual(int(m.group(3)), int(value), name)
            elif family[3:] in bench.BB_PAPERS:
                self.assertEqual(int(m.group(3)), bench.BB_PAPERS[family[3:]][5] + int(value), name)
            else:
                w = re.search(r'DistFix\{\s*(?:(\d+)\s*\+\s*' + const + r'|' + const + r'\s*\+\s*(\d+))\s*,\s*'
                              + const + r'\s*\}', source)
                self.assertIsNotNone(w, name)
                self.assertEqual(int(m.group(3)), int(w.group(1) or w.group(2)) + int(value), name)
        self.assertEqual(len(families), 38)
        # Every Table 2 program at the paper's precision, which its file has.
        for program, (const, paper, _) in bench.BB_PRECISION.items():
            source = Path(scripts.parent / 'data' / 'alea' / 'hybit' / f'{program}.jl').read_text()
            self.assertRegex(source, rf'(?m)^const {const} = {paper}$')
            self.assertIn(f'{const}={paper}', [spec[n][1][2] for n in families[f'bb_{program}']])
        # Smallest first, so that a failure skips the larger.
        for names in families.values():
            values = [int(spec[n][1][-1].split('=')[1]) for n in names]
            self.assertEqual(values, sorted(values))
        self.assertEqual(families['bb_weekend'][0], 'bb_weekend_w18')
        self.assertEqual(families['bb_genes'][:2], ['bb_genes_t5', 'bb_genes_t10'])
        # The precision ladder of genes.jl with 5 genes stops below the
        # file's own precision, which bb_genes_t5 runs.
        self.assertEqual(families['bb_genes5'][-1], 'bb_genes5_w27')
        self.assertRegex(Path(spec['bb_genes_t5'][1][1]).read_text(), r'(?m)^const bits = 20$')

    def test_the_circuit_size_is_read_from_the_header(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'c.pl')
            path.write_text('% pi.jl b=6: the circuit Alea.jl builds, written by tddlog\'s scripts/alea_bench.jl.\n'
                            '% 24 flips, 12633 gates, 0 observations, 1 queries.\n0.5::f1.\n')
            self.assertEqual(bench.circuit_size(str(path)), {'flips': 24, 'gates': 12633})
            path.write_text('0.5::f1.\n')
            self.assertEqual(bench.circuit_size(str(path)), {})

    def test_alea_runs_the_program_with_its_constant_set(self):
        args = self.args(alea='julia {file}', timeout=30)
        out = 'alea_q e1(1) 0.5\nalea_seconds 2.5\nalea_compile_seconds 2.25\n'
        with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                        'stderr': ''}) as timed:
            r = bench.run_alea(args, 'bb_pi_w8', 'bench/bb_pi_w8.pl')
        cmd = timed.call_args.args[0]
        self.assertEqual(cmd[2:7], ['run', '--warm-below', '10', '--set', 'b=6'])
        self.assertTrue(cmd[7].endswith('data/alea/hybit/pi.jl'))
        self.assertEqual(r['answers'], {'e1(1)': 0.5})
        with tempfile.TemporaryDirectory() as d, patch.object(bench.subprocess, 'run') as run, \
                redirect_stdout(io.StringIO()):
            bench.gen(self.args(only=['bb_genes_t20'], cnf=False, alea='julia {file}', dir=d))
        self.assertEqual(run.call_args.args[0][2:], [
            'export', '--set', 'T=20', str(scripts.parent / 'data' / 'alea' / 'bitblast' / 'genes.jl'),
            os.path.abspath(os.path.join(d, 'bb_genes_t20'))])


class RouletteTests(unittest.TestCase):
    """The roulette suite: the scaling programs of Roulette's evaluation
    (gen_roulette.py) on ladders of size and the artifact's networks, and
    Roulette itself on the artifact's programs."""

    def args(self, **kw):
        base = dict(suite='roulette', only=[], families=[], dir='bench', timeout=30,
                    roulette_benchmarks='rb', roulette='racket {file}', roulette_variants=['rsdd'])
        base.update(kw)
        return SimpleNamespace(**base)

    def test_the_suite_names_the_programs_on_their_ladders(self):
        spec = bench.instances(self.args())
        families = {}
        for name, (family, gargs) in spec.items():
            families.setdefault(family, []).append((name, gargs))
        self.assertEqual(len(families['bn']), 9)
        for name, gargs in families['bn']:
            net = name.removeprefix('rl_bn_')
            self.assertEqual(gargs, ['gen_bn.py', f'rb/bayesian-networks/{net}.bif', '--query',
                                     dict(bench.RL_NETWORKS)[net]])
        del families['bn']
        self.assertEqual(sorted(families), sorted(bench.rl_family(p) for p in bench.RL_SIZES))
        for family, entries in families.items():
            sizes = [int(g[2]) for _, g in entries]
            # Smallest first, so that a failure skips the larger.
            self.assertEqual(sizes, sorted(sizes))
            for name, gargs in entries:
                self.assertEqual(gargs[0], 'gen_roulette.py')
                self.assertEqual(name, f'{family}_{gargs[2]}')
                self.assertEqual(bench.rl_family(gargs[1]), family)
        self.assertEqual(families['rl_hmm_marginal_likelihood'][-1][0], 'rl_hmm_marginal_likelihood_900')

    def test_alea_runs_its_own_diamond_and_the_ladders_port(self):
        args = self.args(alea='julia --project=A {file}', timeout=30)
        self.assertIsNone(bench.alea_program(args, ['gen_roulette.py', 'caesar', '25']))
        self.assertIsNone(bench.alea_program(self.args(suite='dice'), ['gen_roulette.py', 'diamond', '1000']))
        out = 'alea_q q1 0.25\nalea_q q2 0.5\nalea_seconds 1.5\nalea_compile_seconds 0.25\n'
        with tempfile.TemporaryDirectory() as d:
            pl = Path(d, 'rl_ladder_1000.pl')
            pl.write_text('a.\nquery(o1(1000)).\nquery(o2(1000)).\n')
            with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                            'stderr': ''}) as timed:
                r = bench.run_alea(args, 'rl_ladder_1000', str(pl))
        cmd = timed.call_args.args[0]
        self.assertTrue(cmd[2].endswith('alea_bench.jl'))
        self.assertEqual(cmd[3:8], ['run', '--warm-below', '10', '--set', 'N=1000'])
        self.assertTrue(cmd[8].endswith('data/alea/verification/ladder.jl'))
        self.assertEqual(r['answers'], {'o1(1000)': 0.25, 'o2(1000)': 0.5})
        self.assertEqual(r['stats']['inproc_seconds'], 1.5)
        for f in bench.ALEA_PROGRAMS.values():
            self.assertEqual(len(re.findall(r'^const N = ', Path(f).read_text(), re.M)), 1, f)

    def test_alea_tddlog_answers_from_the_circuit_alea_exports(self):
        # Alea.jl+tddlog: the bitblast suite's export of the instance's Alea.jl program, timed warm,
        # then tddlog on the circuit; the record holds both, and the circuit's files go after
        self.assertIn('alea-tddlog', bench.OWN_TIME)
        with tempfile.TemporaryDirectory() as d:
            args = self.args(**{'alea-tddlog': 'julia --project=A {file}'}, dir=d, tddlog='tddlog')
            self.assertFalse(bench.accepts_alea_tddlog(args, 'rl_caesar_25'))
            self.assertTrue(bench.accepts_alea_tddlog(args, 'rl_ladder_1000'))
            pl = Path(d, 'rl_ladder_1000.pl')
            pl.write_text('a.\nquery(o1(1000)).\nquery(o2(1000)).\n')
            circuit = Path(d, 'rl_ladder_1000.alea.pl')

            def timed(export_out, tddlog_s):
                def run(cmd, timeout, cwd=None, env=None):
                    if 'export' in cmd:
                        circuit.write_text('% ladder.jl N=1000\n% 3002 flips, 9000 gates, 2 queries\n')
                        Path(d, 'rl_ladder_1000.alea.dicemap').write_text('')
                        Path(d, 'rl_ladder_1000.alea__d1.dice').write_text('')
                        return {'status': 'ok', 'seconds': 9.0, 'stdout': export_out, 'stderr': ''}
                    self.assertEqual(cmd[-1], str(circuit))
                    self.assertTrue(circuit.exists())
                    return {'status': 'ok', 'seconds': tddlog_s, 'stdout': 'q1: 0.25\nq2: 0.5\n', 'stderr': ''}
                return run

            warm = 'alea_seconds 2.5\nalea_compile_seconds 2.0\nalea_warm_seconds 0.75\n'
            with patch.object(bench, 'timed', side_effect=timed(warm, 0.5)) as t:
                r = bench.run_alea_tddlog(args, 'rl_ladder_1000', str(pl))
            export, tddlog = (c.args for c in t.call_args_list)
            self.assertEqual(export[0][:2], ['julia', '--project=A'])
            self.assertTrue(export[0][2].endswith('alea_bench.jl'))
            self.assertEqual(export[0][3:8], ['export', '--warm-below', '10', '--set', 'N=1000'])
            self.assertTrue(export[0][8].endswith('data/alea/verification/ladder.jl'))
            self.assertEqual(export[0][9:], [str(Path(d, 'rl_ladder_1000.alea'))])
            self.assertEqual(tddlog[1], 29.25)
            self.assertEqual(r['status'], 'ok')
            self.assertEqual(r['answers'], {'o1(1000)': 0.25, 'o2(1000)': 0.5})
            self.assertEqual(r['seconds'], 9.5)
            self.assertEqual(r['stats'], {'export_s': 0.75, 'flips': 3002, 'gates': 9000, 'tddlog_s': 0.5,
                                          'inproc_seconds': 1.25})
            self.assertEqual(bench.own_seconds(r['stats']), 1.25)
            self.assertEqual(sorted(os.listdir(d)), ['rl_ladder_1000.pl'])
            # no warm run: the first less Julia's compilation; past the limit with the export, a timeout
            with patch.object(bench, 'timed', side_effect=timed('alea_seconds 31.0\nalea_compile_seconds 2.0\n', 1.5)):
                r = bench.run_alea_tddlog(args, 'rl_ladder_1000', str(pl))
            self.assertEqual((r['status'], r['stats']['export_s'], r['stats']['inproc_seconds']), ('timeout', 29.0, 30.5))
            self.assertEqual(sorted(os.listdir(d)), ['rl_ladder_1000.pl'])

    def test_roulette_runs_the_artifacts_programs(self):
        args = self.args()
        data = bench.ROULETTE_DATA
        self.assertEqual(bench.roulette_program(args, 'rl_caesar_25'), (f'{data}/caesar.rkt', ['25']))
        self.assertEqual(bench.roulette_command('racket {file}', 'p.rkt'), ['racket', 'p.rkt'])
        self.assertEqual(bench.roulette_command('racket {file}', 'p.rkt', '-l-', 'raco', 'make'),
                         ['racket', '-l-', 'raco', 'make', 'p.rkt'])
        self.assertEqual(bench.roulette_program(args, 'rl_hmm_smoothing_joint_100'),
                         (f'{data}/hmm.rkt', ['smoothing-joint', '100']))
        for program in ['motivating', 'diamond', 'ladder', 'caesar', 'network', 'hardware', 'hardware-bitwidth',
                        'hmm', 'bn', 'print', 'scaling/network', 'scaling/hardware']:
            self.assertTrue(Path(data, f'{program}.rkt').exists(), program)
        # every network of the artifact has its order, which names every variable once
        orders = {e[0]: e for e in bench.rktd_lists(Path(data, 'networks.rktd').read_text())[0]}
        for net, var in bench.RL_NETWORKS:
            self.assertEqual(orders[net][1], var)
            self.assertIn(var, orders[net][2])
            self.assertEqual(len(set(orders[net][2])), len(orders[net][2]))

    def run_roulette(self, name, path, out):
        with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                        'stderr': 'roulette_timer: program 0.061 s\n'}) as timed:
            r = bench.run_roulette(self.args(), name, path, 'rsdd')
        return r, timed.call_args[0][0]

    def test_the_answers_are_keyed_by_the_queries(self):
        path = str(SOURCE / 'tests' / 'roulette' / 'network-2.pl')
        out = ('cpu time: 60 real time: 61 gc time: 3\nroulette_q other(#t) 0.0 #t\n'
               'roulette_q m(2,15) 0.98 (#t #t #t #t)\nroulette_q m(2,16) 0.01 #f\n')
        r, cmd = self.run_roulette('rl_network_10', path, out)
        self.assertEqual(cmd, ['racket', bench.ROULETTE_TIMER, f'{bench.ROULETTE_DATA}/network.rkt', '10'])
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(list(r['answers']), [f'm(2,{v})' for v in range(17)])
        # A value Roulette leaves out has probability 0.
        self.assertEqual((r['answers']['m(2,15)'], r['answers']['m(2,16)'], r['answers']['m(2,0)']),
                         (0.98, 0.01, 0.0))
        self.assertEqual(r['stats'], {'inproc_seconds': 0.061, 'other': {'other(#t)': 0.0}})
        path = str(SOURCE / 'tests' / 'roulette' / 'hmm-smoothing-joint-4.pl')
        r, cmd = self.run_roulette('rl_hmm_smoothing_joint_100', path, 'roulette_q q 0.25 #t\n')
        self.assertEqual(r['answers'], {'q': 0.25})
        r, _ = self.run_roulette('rl_hmm_smoothing_joint_100', str(SOURCE / 'tests' / 'roulette' /
                                                                    'hmm-smoothing-joint-4.pl'), 'error: x\n')
        self.assertEqual(r['status'], 'error')

    # bnlearn's cancer network as a BIF file, in the form of the artifact's
    # benchmark/bayesian-networks/cancer.bif.
    CANCER = '''network unknown {
}
variable Pollution {
  type discrete [ 2 ] { low, high };
}
variable Smoker {
  type discrete [ 2 ] { True, False };
}
variable Cancer {
  type discrete [ 2 ] { True, False };
}
variable Xray {
  type discrete [ 2 ] { positive, negative };
}
variable Dyspnoea {
  type discrete [ 2 ] { True, False };
}
probability ( Pollution ) {
  table 0.9, 0.1;
}
probability ( Smoker ) {
  table 0.3, 0.7;
}
probability ( Cancer | Pollution, Smoker ) {
  (low, True) 0.03, 0.97;
  (high, True) 0.05, 0.95;
  (low, False) 0.001, 0.999;
  (high, False) 0.02, 0.98;
}
probability ( Xray | Cancer ) {
  (True) 0.9, 0.1;
  (False) 0.2, 0.8;
}
probability ( Dyspnoea | Cancer ) {
  (True) 0.65, 0.35;
  (False) 0.3, 0.7;
}
'''

    def test_an_artifacts_network_runs_in_the_bn_language(self):
        """A network of the artifact runs data/roulette/bn.rkt on its own BIF
        file, asked its queried variable in the artifact's order
        (networks.rktd), its values named as gen_bn.py names them."""
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'bayesian-networks').mkdir()
            Path(d, 'bayesian-networks', 'cancer.bif').write_text(self.CANCER)
            args = self.args(dir=d, roulette_benchmarks=d)
            prog, rest = bench.roulette_program(args, 'rl_bn_cancer')
            self.assertEqual((prog, rest), (f'{bench.ROULETTE_DATA}/bn.rkt',
                                            [f'{d}/bayesian-networks/cancer.bif', f'{d}/rl_bn_cancer.roulette.asks']))
            self.assertEqual(Path(d, 'rl_bn_cancer.roulette.asks').read_text(),
                             'order Pollution Smoker Cancer Dyspnoea Xray\nquery Xray\n')
            path = Path(d, 'rl_bn_cancer.pl')
            path.write_text('query(xray(positive)).\nquery(xray(negative)).\n')
            with patch.object(bench, 'timed', return_value={
                    'status': 'ok', 'seconds': 3.0, 'stdout': 'roulette_b Xray positive 0.208\n'
                    'roulette_b Xray negative 0.792\n', 'stderr': 'roulette_timer: program 0.061 s\n'}):
                r = bench.run_roulette(args, 'rl_bn_cancer', str(path), 'rsdd')
        self.assertEqual(r['answers'], {'xray(positive)': 0.208, 'xray(negative)': 0.792})

    def test_raco_make_runs_one_at_a_time(self):
        """roulette_make holds a lock of its template's while raco make
        runs, in the temporary directory, so that another lane's compile
        under the same template waits; another template's does not."""
        held = []

        def probe(template):
            key = bench.hashlib.sha256(template.encode()).hexdigest()[:16]
            with open(os.path.join(tempfile.gettempdir(), f'raco-make-{key}.lock'), 'w') as f:
                try:
                    fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    return True
                fcntl.flock(f, fcntl.LOCK_UN)
                return False

        def timed(cmd, timeout, **kw):
            held.append((probe('racket {file}'), probe('other {file}')))
            return {'status': 'error', 'seconds': 0.1, 'error': 'cannot open module file'}
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {'TMPDIR': d}), \
                patch.object(tempfile, 'tempdir', None), patch.object(bench, 'timed', side_effect=timed) as t:
            err = bench.roulette_make(self.args(), 'racket {file}', '/p/bn.rkt', '/p/timer.rkt')
            self.assertEqual(t.call_args[0][0], ['racket', '-l-', 'raco', 'make', '/p/bn.rkt', '/p/timer.rkt'])
            self.assertFalse(probe('racket {file}'))
        self.assertEqual(held, [(True, False)])
        self.assertEqual(err, {'status': 'error', 'seconds': 0, 'error': 'raco make: error cannot open module file'})

    def test_a_module_that_does_not_compile_within_the_limit_is_unsupported(self):
        """A compile that runs past the time limit (TPC-H q1 at 200 to 1000,
        savage_knife's 24 MB module) is a program Roulette cannot express,
        not an error and not a timeout; the compile is not charged, past
        the limit as before it."""
        args = self.args(timeout=300)
        want = {'status': 'unsupported', 'seconds': 0,
                'error': 'raco make: the module did not compile within the time limit of 300 s'}
        with patch.object(bench, 'timed', return_value={'status': 'timeout', 'seconds': 300}) as t:
            self.assertEqual(bench.roulette_timed(args, '/p/q1.roulette.rkt', [], 300), want)
            self.assertEqual(t.call_count, 1)   # the program is not run
            self.assertEqual(bench.roulette_make(args, 'racket {file}', '/p/q1.roulette.rkt'), want)
            self.assertFalse(bench.ran_out(want))

    @unittest.skipUnless(os.environ.get('ROULETTE_RACKET'), 'ROULETTE_RACKET is not set')
    def test_bn_rkt_reads_values_that_print_between_bars(self):
        """data/roulette/bn.rkt under Roulette: values that read as numbers,
        which Racket writes between bars (|0|), in display-pmf's table (a)
        and alone where one value holds all the mass (c), and values of
        ordinary names (b), each distribution in full."""
        bif = ('network unknown {\n}\n'
               'variable a {\n  type discrete [ 3 ] { 0, 1, 2 };\n}\n'
               'variable b {\n  type discrete [ 2 ] { yes, no };\n}\n'
               'variable c {\n  type discrete [ 2 ] { 0, 1 };\n}\n'
               'probability ( a ) {\n  table 0.25, 0.5, 0.25;\n}\n'
               'probability ( b | a ) {\n  (0) 0.5, 0.5;\n  (1) 0.1, 0.9;\n  (2) 1.0, 0.0;\n}\n'
               'probability ( c ) {\n  table 1.0, 0.0;\n}\n')
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'n.bif').write_text(bif)
            Path(d, 'n.asks').write_text('order a b c\nquery a\nquery b\nquery c\n')
            r = subprocess.run(shlex.split(os.environ['ROULETTE_RACKET']) +
                               [f'{bench.ROULETTE_DATA}/bn.rkt', f'{d}/n.bif', f'{d}/n.asks'],
                               capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        got = {}
        for line in r.stdout.splitlines():
            if line.startswith('roulette_b '):
                _, v, x, p = line.split()
                got[(v, x)] = float(p)
        want = {('a', '|0|'): 0.25, ('a', '|1|'): 0.5, ('a', '|2|'): 0.25,
                ('b', 'yes'): 0.425, ('b', 'no'): 0.575, ('c', '|0|'): 1.0}
        self.assertEqual(sorted(got), sorted(want))
        for k, p in want.items():
            self.assertAlmostEqual(got[k], p, delta=1e-12, msg=k)

    @unittest.skipUnless(os.environ.get('ROULETTE_RACKET'), 'ROULETTE_RACKET is not set')
    def test_bn_rkt_reads_a_certain_variable_of_one_value(self):
        """A Bayesian network's certain variable of one value (Ace's munin1's
        DIFFN_DUMMY_1 to 3) as gen_bn.py writes it in BIF, a child's parent
        here: bn.rkt reads it, and gives it and its child their
        distributions."""
        variables = {'a': ['x', 'y'], 'd': ['dummy'], 'b': ['f', 't']}
        tables = {'a': ([], {(): [0.25, 0.75]}), 'd': (['a'], {('x',): [1.0], ('y',): [1.0]}),
                  'b': (['d'], {('dummy',): [0.4, 0.6]})}
        text, _ = bench.gen_bn.to_bif((['a', 'd', 'b'], variables, tables))
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'n.bif').write_text(text)
            Path(d, 'n.asks').write_text('order x0 x1 x2\nquery x1\nquery x2\n')
            r = subprocess.run(shlex.split(os.environ['ROULETTE_RACKET']) +
                               [f'{bench.ROULETTE_DATA}/bn.rkt', f'{d}/n.bif', f'{d}/n.asks'],
                               capture_output=True, text=True, timeout=600)
        self.assertEqual(r.returncode, 0, r.stderr[-2000:])
        got = {tuple(line.split()[1:3]): float(line.split()[3]) for line in r.stdout.splitlines()
               if line.startswith('roulette_b ')}
        want = {('x1', 's0'): 1.0, ('x2', 's0'): 0.4, ('x2', 's1'): 0.6}
        self.assertEqual(sorted(got), sorted(want))
        for k, p in want.items():
            self.assertAlmostEqual(got[k], p, delta=1e-12, msg=k)

    def test_a_network_runs_in_the_bn_language(self):
        """A Bayesian network of another suite runs data/roulette/bn.rkt on
        the BIF file gen_bn.py writes and what it asks of it: its order,
        observations and queries, or under --pr the holds lines of P(e);
        its answers are keyed as the ProbLog program's queries (a value left
        out answers 0), its in-process time kept. A Markov network runs the
        Dice program dice2roulette.py translates."""
        with tempfile.TemporaryDirectory() as d:
            args = self.args(suite='standard', dir=d)
            prog, rest = bench.roulette_program(args, 'bn_asia')
            self.assertEqual((prog, rest), (f'{bench.ROULETTE_DATA}/bn.rkt',
                                            [f'{d}/bn_asia.roulette.bif', f'{d}/bn_asia.roulette.asks']))
            asks = Path(d, 'bn_asia.roulette.asks').read_text().splitlines()
            self.assertEqual(asks[0].split()[0], 'order')
            self.assertEqual(asks[1:], ['query x6', 'query x7'])
            bench.roulette_program(args, 'bn_asia_ev')
            asks = Path(d, 'bn_asia_ev.roulette.asks').read_text().splitlines()
            self.assertEqual([a.split()[0] for a in asks], ['order', 'observe', 'query'])
            self.assertIsNone(bench.roulette_bif(args, 'dice_twocoins'))
            path = Path(d, 'bn_asia.pl')
            path.write_text('query(xray(yes)).\nquery(xray(no)).\nquery(dysp(yes)).\nquery(dysp(no)).\n')
            out = 'roulette_b x6 s0 0.11\nroulette_b x6 s1 0.89\nroulette_b x7 s1 1.0\n'
            with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                            'stderr': 'roulette_timer: program 0.033 s\n'}) as timed:
                r = bench.run_roulette(args, 'bn_asia', str(path), 'rsdd')
            self.assertEqual(timed.call_args_list[0][0][0],
                             ['racket', '-l-', 'raco', 'make', f'{bench.ROULETTE_DATA}/bn.rkt', bench.ROULETTE_TIMER])
            self.assertEqual(timed.call_args_list[1][0][0],
                             ['racket', bench.ROULETTE_TIMER, f'{bench.ROULETTE_DATA}/bn.rkt',
                              f'{d}/bn_asia.roulette.bif', f'{d}/bn_asia.roulette.asks'])
            self.assertEqual(r['answers'], {'xray(yes)': 0.11, 'xray(no)': 0.89, 'dysp(yes)': 0.0, 'dysp(no)': 1.0})
            self.assertAlmostEqual(r['stats']['inproc_seconds'], 0.033)
            with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0,
                                                            'stdout': 'roulette_b x6 s2 1.0\n',
                                                            'stderr': 'roulette_timer: program 0.033 s\n'}):
                r = bench.run_roulette(args, 'bn_asia', str(path), 'rsdd')
            self.assertEqual(r['status'], 'error')
            self.assertIn('x6 takes s2', r['error'])
            # P(e): the observations as holds lines, the answer PE_QUERY's
            pr = self.args(suite='standard', dir=os.path.join(d, 'pr'), pr=True)
            os.makedirs(pr.dir)
            bench.roulette_program(pr, 'bn_asia_ev')
            asks = Path(pr.dir, 'bn_asia_ev.roulette.asks').read_text().splitlines()
            self.assertEqual([a.split()[0] for a in asks], ['order', 'holds'])
            path = Path(pr.dir, 'bn_asia_ev.pe.pl')
            path.write_text(f'{bench.PE_QUERY} :- xray(yes).\nquery({bench.PE_QUERY}).\n')
            with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': 'roulette_e 0.11\n',
                                                            'stderr': 'roulette_timer: program 0.033 s\n'}):
                r = bench.run_roulette(pr, 'bn_asia_ev', str(path), 'rsdd')
            self.assertEqual(r['answers'], {bench.PE_QUERY: 0.11})
            # a Markov network has no BIF file
            markov = self.args(suite='uai', dir=d, uai_benchmarks=str(SOURCE / 'tests' / 'regress'))
            with patch.object(bench, 'instances', return_value={
                    'uai_markov': ('uaimarkov', ['gen_bn.py', str(SOURCE / 'tests' / 'regress' / 'markov.uai')])}):
                self.assertIsNone(bench.roulette_bif(markov, 'uai_markov'))
                self.assertEqual(bench.roulette_program(markov, 'uai_markov'), (f'{d}/uai_markov.roulette.rkt', []))

    def test_roulette_options_fail_where_they_do_nothing(self):
        def run_args(**kw):
            base = dict(suite='roulette', only=['rl_motivating_1000'], families=[], strategies=[], kbs=[],
                        pita='', aspmc='', scallop='', dice='', alea='', dir='bench', out='results.jsonl',
                        tddlog='tddlog', timeout=1, no_skip=True, exact=False, ace='', merlin='', ganak='',
                        sharpsat_td='', pr=False, rotate=None, roulette='racket {file}',
                        roulette_variants=['rsdd'], roulette_benchmarks='rb')
            base.update(kw)
            return SimpleNamespace(**base)
        with self.assertRaisesRegex(SystemExit, '--roulette-variants cudd'):
            bench.run(run_args(roulette_variants=['cudd']))
        for gone in ['nocomplement', 'extexp']:
            with self.assertRaisesRegex(SystemExit, f'--roulette-variants {gone} was a variant of the Roulette artifact'):
                bench.run(run_args(roulette_variants=['rsdd', gone]))
        with self.assertRaisesRegex(SystemExit, "every Bayesian network now runs in Roulette's BN language"):
            bench.run(run_args(roulette_variants=['rsdd', 'bayes']))

        def parsed(cmd, **given):
            fields = dict(cmd=cmd, suite='roulette', ace_benchmarks=None, uai_benchmarks=None,
                          biomine_benchmarks=None, roulette_benchmarks=None, ace='',
                          ace_variants=None, ace_heap=None, ace_java=None, sharpsat_td='', sharpsat_td_decot=None,
                          roulette='', roulette_variants=None)
            return SimpleNamespace(**(fields | given))
        with self.assertRaisesRegex(SystemExit, '--roulette-variants without --roulette'):
            bench.ace_defaults(parsed('run', roulette_variants=['bayes']))
        with self.assertRaisesRegex(SystemExit, 'and the suite is ppl'):
            bench.ace_defaults(parsed('gen', suite='ppl', roulette_benchmarks='rb'))
        args = parsed('run', roulette='racket {file}')
        bench.ace_defaults(args)
        self.assertEqual(args.roulette_variants, ['rsdd'])
        self.assertTrue(args.roulette_benchmarks.endswith('roulette-benchmarks'))

    def test_the_reference_is_the_generators(self):
        self.assertEqual(bench.EXACT['gen_roulette.py'][0], 'dp')
        self.assertEqual(bench.EXACT['gen_roulette.py'][1]('x.pl'), ['gen_roulette.py', '--reference-of', 'x.pl'])


class RouletteTddlogTests(unittest.TestCase):
    """Roulette+tddlog: Roulette's tddlog engine writes the circuit of
    Roulette's run as a ProbLog program, tddlog answers it, and the answers
    are read as Roulette's query reads its engine's."""

    # A circuit of disrupt.rkt's shape: call 1 the value (x, y, the failed
    # observation), call 2 the observation.
    CIRCUIT = ('% infer was called 2 times\n% q1(1): x\n% q1(2): y\n% q1(3): \u22a5123\n'
               '% q2(1): #t\n% q2(2): #f\nquery(q1(1)).\nquery(q1(2)).\nquery(q1(3)).\n')
    ANSWERS = {'q1(1)': 0.3, 'q1(2)': 0.42, 'q1(3)': 0.28, 'q2(1)': 0.72, 'q2(2)': 0.28}

    def test_each_call_is_divided_by_its_values_but_the_failure(self):
        calls = bench.roulette_tddlog_calls(self.CIRCUIT, self.ANSWERS)
        self.assertEqual(sorted(calls), [1, 2])
        self.assertEqual(sorted(calls[1]), ['x', 'y'])
        self.assertAlmostEqual(calls[1]['x'], 5 / 12)
        self.assertAlmostEqual(calls[1]['y'], 7 / 12)
        self.assertAlmostEqual(calls[2]['#t'], 0.72)
        with self.assertRaisesRegex(ValueError, 'tddlog answers no q1\\(2\\)'):
            bench.roulette_tddlog_calls(self.CIRCUIT, {'q1(1)': 0.3})
        # Where every value but the failure has weight 0, they answer 0.
        self.assertEqual(bench.roulette_tddlog_calls(self.CIRCUIT, {**self.ANSWERS, 'q1(1)': 0, 'q1(2)': 0})[1],
                         {'x': 0.0, 'y': 0.0})

    def test_a_dice_programs_components_are_its_odd_calls(self):
        comps = bench.roulette_tddlog_components({
            1: {'#t': 0.25, '#f': 0.75}, 2: {'#t': 1.0},
            3: {'(#t #f)': 0.5, '(#f #f)': 0.5}, 4: {'#t': 1.0}})
        self.assertEqual(comps, [{'true': 0.25, 'false': 0.75}, {'0': 0.5, '1': 0.5, '2': 0.0, '3': 0.0}])
        with self.assertRaisesRegex(ValueError, 'neither Booleans nor lists of one width'):
            bench.roulette_tddlog_components({1: {'(#t)': 0.5, '(#t #f)': 0.5}})

    def args(self, **kw):
        base = dict(suite='roulette', only=[], families=[], dir='bench', timeout=30, roulette_benchmarks='rb',
                    roulette_tddlog='racket {file}', tddlog='tddlog')
        base.update(kw)
        return SimpleNamespace(**base)

    def run_it(self, args, name, path, export_err, tddlog_out, export_out=None, kept=()):
        """The run's record and commands; the circuit is removed once
        tddlog has read it, and the run leaves `kept` alone (a network's
        BIF file and asks)."""
        cmds = []

        def timed(cmd, timeout, cwd=None):
            cmds.append((cmd, timeout))
            if cmd[0] == 'tddlog':
                self.assertTrue(Path(cmd[-1]).exists())
                return {'status': 'ok', 'seconds': 0.5, 'stdout': tddlog_out, 'stderr': ''}
            if 'make' in cmd:
                return {'status': 'ok', 'seconds': 9.0, 'stdout': '', 'stderr': ''}
            return {'status': 'ok', 'seconds': 2.0, 'stdout': export_out or self.CIRCUIT, 'stderr': export_err}
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', side_effect=timed):
            r = bench.run_roulette_tddlog(self.args(dir=d, **args), name, path)
            self.assertEqual(sorted(os.listdir(d)), sorted(kept))
        return r, cmds

    SUMMARY = 'tddlog-engine: 1 queries, 5 values, 2 flips, 3 gates, 1 observations; program 1.25 s, circuit 0.25 s\n'

    def test_a_driver_is_answered_as_the_atoms_it_prints(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'p.pl')
            path.write_text('query(a).\nquery(b).\nquery(c).\n')
            # print.rkt's emit: the atom, the recording engine's weight, the value
            err = 'roulette_q a 1 x\nroulette_q b 1 y\nroulette_q c 0 z\n' + self.SUMMARY
            out = ''.join(f'{q}: {p}\n' for q, p in self.ANSWERS.items())
            r, cmds = self.run_it({}, 'rl_motivating_1000', str(path), err, out)
        data = bench.ROULETTE_DATA
        self.assertEqual([c for c, _ in cmds[:2]], [
            ['racket', '-l-', 'raco', 'make', f'{data}/motivating.rkt'],
            ['racket', '-l-', 'roulette/engine/tddlog', f'{data}/motivating.rkt', '1000']])
        self.assertEqual(cmds[2][0][:6], ['tddlog', '--stats', '--precision', '17', '--vtree', 'auto'])
        # tddlog's budget is what the export leaves of the limit
        self.assertAlmostEqual(cmds[2][1], 30 - 1.5)
        self.assertEqual(r['status'], 'ok')
        self.assertAlmostEqual(r['answers']['a'], 5 / 12)
        self.assertAlmostEqual(r['answers']['b'], 7 / 12)
        self.assertEqual(r['answers']['c'], 0.0)
        self.assertEqual(r['seconds'], 2.5)
        self.assertEqual(r['stats']['inproc_seconds'], 2.0)
        self.assertEqual((r['stats']['program_s'], r['stats']['circuit_s'], r['stats']['tddlog_s']),
                         (1.25, 0.25, 0.5))

    def test_a_logic_programs_answers_are_keyed_as_problog2ppl_lists_its_queries(self):
        # problog2ppl.py's module: a Boolean a query, each call followed by its observations';
        # the ProbLog program asks them with a rule, which names no query of its own
        circuit = '% q1(1): #t\n% q1(2): #f\n% q2(1): #t\n% q3(1): #t\n% q3(2): #f\n% q4(1): #t\n'
        answers = {'q1(1)': 0.3, 'q1(2)': 0.7, 'q2(1)': 1.0, 'q3(1)': 0.6, 'q3(2)': 0.4, 'q4(1)': 1.0}
        out = ''.join(f'{q}: {p}\n' for q, p in answers.items())

        def timed(cmd, timeout, cwd=None):
            if cmd[0] == 'tddlog':
                return {'status': 'ok', 'seconds': 0.5, 'stdout': out, 'stderr': ''}
            if 'make' in cmd:
                return {'status': 'ok', 'seconds': 9.0, 'stdout': '', 'stderr': ''}
            return {'status': 'ok', 'seconds': 2.0, 'stdout': circuit, 'stderr': self.SUMMARY}
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', side_effect=timed):
            path = Path(d, 'p.pl')
            path.write_text('query(p(X)) :- between(1,2,X).\n')
            lp = (str(Path(d, 'p.lp.roulette.rkt')), [str(Path(d, 'p.lp.roulette.facts'))], ['p(1)', 'p(2)'])
            r = bench.run_roulette_tddlog(self.args(dir=d), 'rl_motivating_1000', str(path), lp)
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(sorted(r['answers']), ['p(1)', 'p(2)'])
        self.assertAlmostEqual(r['answers']['p(1)'], 0.3)
        self.assertAlmostEqual(r['answers']['p(2)'], 0.6)

    def test_a_networks_value_that_reads_as_a_number_loses_its_bars(self):
        # The circuit's note writes the symbol 0 as |0|, the ProbLog program's atom reads v_0.
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'bayesian-networks').mkdir()
            Path(d, 'bayesian-networks', 'pigs.bif').write_text(
                'network unknown {\n}\nvariable p392115290 {\n  type discrete [ 2 ] { 0, 1 };\n}\n'
                'probability ( p392115290 ) {\n  table 0.25, 0.75;\n}\n')
            path = Path(d, 'p.pl')
            path.write_text('query(p392115290(v_0)).\nquery(p392115290(v_1)).\n')
            r, cmds = self.run_it({'roulette_benchmarks': d}, 'rl_bn_pigs', str(path), self.SUMMARY,
                                  'q1(0): 0.25\nq1(1): 0.75\nq2(0): 1\n',
                                  export_out='% infer was called 2 times\n% q1(0): |0|\n% q1(1): |1|\n% q2(0): #t\n',
                                  kept=['rl_bn_pigs.roulette.asks'])
        self.assertEqual(cmds[1][0][:4], ['racket', '-l-', 'roulette/engine/tddlog', f'{bench.ROULETTE_DATA}/bn.rkt'])
        self.assertEqual(r['answers'], {'p392115290(v_0)': 0.25, 'p392115290(v_1)': 0.75})
        self.assertNotIn('other', r['stats'])

    def test_a_networks_queries_are_its_odd_calls(self):
        """bn.rkt's K-th query is call 2K-1; under --pr P(e) is call 1's #t.
        A circuit of another number of calls is an error: bn.rkt with an
        expression at the module's level, which the language asks as a
        query of its own (two calls), read #<void> as the first query."""
        network = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        spec = {'bn_asia': ('bn', ['gen_bn.py', network, '--query', 'leaves'])}
        circuit = '% infer was called 4 times\n' + ''.join(
            f'% q{k}({i}): {v}\n' for k, i, v in [(1, 0, 's0'), (1, 1, 's1'), (2, 0, '#t'),
                                                  (3, 0, 's0'), (3, 1, 's1'), (4, 0, '#t')])
        out = 'q1(0): 0.25\nq1(1): 0.75\nq2(0): 1\nq3(0): 0.5\nq3(1): 0.5\nq4(0): 1\n'
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'instances', return_value=spec):
            path = Path(d, 'p.pl')
            path.write_text('query(xray(yes)).\nquery(xray(no)).\nquery(dysp(yes)).\nquery(dysp(no)).\n')
            r, _ = self.run_it({'suite': 'standard'}, 'bn_asia', str(path), self.SUMMARY, out, export_out=circuit,
                               kept=['bn_asia.roulette.bif', 'bn_asia.roulette.asks'])
            self.assertEqual(r['answers'], {'xray(yes)': 0.25, 'xray(no)': 0.75, 'dysp(yes)': 0.5, 'dysp(no)': 0.5})
            r, _ = self.run_it({'suite': 'standard'}, 'bn_asia', str(path), self.SUMMARY, out,
                               export_out=circuit.replace('called 4 times', 'called 10 times'),
                               kept=['bn_asia.roulette.bif', 'bn_asia.roulette.asks'])
            self.assertEqual(r['status'], 'error')
            self.assertIn('bn.rkt called infer 10 times, not twice for each of its 2 queries', r['error'])
            path.write_text(f'query({bench.PE_QUERY}).\n')
            r, _ = self.run_it({'suite': 'standard', 'pr': True}, 'bn_asia', str(path), self.SUMMARY,
                               'q1(0): 0.25\nq1(1): 0.75\nq2(0): 1\n',
                               export_out='% infer was called 2 times\n% q1(0): #t\n% q1(1): #f\n% q2(0): #t\n',
                               kept=['bn_asia.roulette.bif', 'bn_asia.roulette.asks'])
            self.assertEqual(r['answers'], {bench.PE_QUERY: 0.25})

    def test_out_of_fuel_is_one_past_the_largest_total_of_positive_probability(self):
        # hardware-bitwidth.rkt under the exporter: every total the circuit can take (0 to 7),
        # #f labelled one past the largest (res(8)); Roulette's run leaves out totals 4, 6 and 7
        # of probability 0, and labels #f res(6)
        circuit = '% infer was called 1 times\n' + ''.join(f'% q1({k}): {k}\n' for k in range(8)) + '% q1(8): #f\n'
        answers = {f'q1({k})': p for k, p in enumerate([0.1, 0.1, 0.1, 0.1, 0, 0.1, 0, 0, 0.5])}
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'p.pl')
            path.write_text(''.join(f'query(res({k})).\n' for k in (0, 1, 2, 3, 5, 6)))
            # #f first: total 6's line after it, of probability 0, does not replace its answer
            err = 'roulette_q res(8) 0.1 #f\n' + ''.join(f'roulette_q res({k}) 0.1 {k}\n' for k in range(8)) + self.SUMMARY
            out = ''.join(f'{q}: {p}\n' for q, p in answers.items())
            r, _ = self.run_it({}, 'rl_hardware_bitwidth_5', str(path), err, out, export_out=circuit)
        self.assertEqual(r['status'], 'ok')
        self.assertAlmostEqual(r['answers']['res(6)'], 0.5)
        self.assertAlmostEqual(r['answers']['res(5)'], 0.1)
        self.assertNotIn('res(8)', r['stats'].get('other', {}))
        self.assertEqual(bench.roulette_tddlog_atom('rl_hardware', 'res(4)', '#f', {'0': 1.0}), 'res(4)')

    def test_the_charge_past_the_limit_is_a_timeout(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'p.pl')
            path.write_text('query(a).\n')
            err = 'roulette_q a 1 x\n' + self.SUMMARY.replace('program 1.25', 'program 29.75')
            out = ''.join(f'{q}: {p}\n' for q, p in self.ANSWERS.items())
            r, cmds = self.run_it({}, 'rl_motivating_1000', str(path), err, out)
        self.assertEqual(cmds[2][1], 1.0)
        self.assertEqual(r['status'], 'timeout')

    def test_an_export_without_its_summary_is_an_error(self):
        r, cmds = self.run_it({}, 'rl_motivating_1000', 'p.pl', 'oops\n', '')
        self.assertEqual(len(cmds), 2)
        self.assertEqual((r['status'], r['error']), ('error', "Roulette's tddlog engine wrote no program: oops"))

    def test_it_runs_the_roulette_suite_the_networks_and_the_dice_programs_roulette_translates(self):
        self.assertTrue(bench.roulette_tddlog_accepts(self.args(), 'rl_motivating_1000'))
        with tempfile.TemporaryDirectory() as d:
            # the Dice paper's programs, the Bayesian networks, which Roulette reads in its BN language,
            # and the logic programs, which it reads over their facts (problog2ppl.py)
            args = self.args(suite='standard', dir=d, only=['bn_asia', 'bn_asia_ev', 'dice_twocoins', 'dice_grass',
                                                            'grid4', 'g10_25'])
            taken = [n for n in bench.instances(args) if bench.roulette_tddlog_accepts(args, n)]
            self.assertEqual(set(taken), {'bn_asia', 'bn_asia_ev', 'dice_twocoins', 'dice_grass', 'grid4', 'g10_25'})
            self.assertTrue(bench.lp_route(bench.instances(args)['grid4'][1], 'roulette-tddlog'))
            self.assertEqual(os.listdir(d), [])   # deciding writes nothing (rt2_jobs.py plans with it)
            # a Markov network's Dice program is not one
            markov = {'uai_markov': ('uaimarkov', ['gen_bn.py', str(SOURCE / 'tests' / 'regress' / 'markov.uai')])}
            with patch.object(bench, 'instances', return_value=markov):
                self.assertFalse(bench.roulette_tddlog_accepts(self.args(suite='uai', dir=d), 'uai_markov'))
        # noDice's programs and the Dice ports of the arith programs; of the pluck programs' ProbLog
        # ports, those Roulette reads over their facts (not hmm's, whose Dice port it reads)
        args = self.args(suite='ppl')
        spec = bench.instances(args)
        taken = {n for n in spec if bench.roulette_tddlog_accepts(args, n)}
        self.assertEqual(taken, {n for n in spec if n.startswith(('nodice_', 'arith_'))
                                 or bench.lp_route(spec[n][1], 'roulette')})
        self.assertFalse(any(bench.lp_route(spec[n][1], 'alea') for n in spec))   # Alea.jl's own programs
        self.assertTrue(taken)
        args = self.args(suite='geni')
        self.assertTrue(all(bench.roulette_tddlog_accepts(args, n) for n in bench.instances(args)))


class CirculetteTests(unittest.TestCase):
    """The circulette suite: Circulette's netlists, which tddlog reads
    itself, Roulette's circulette.rkt on each, and every answer keyed as
    the queries of the program tddlog's reader writes of the netlist
    (NAME.pl): fault(G), faults(A,B) and faults(A,B,C)."""

    GATES = [5, 7, 9]
    # the reader's program of a netlist of two gates, 3 and 4, its faults 0 and 1 to Circulette
    PROGRAM = ('% a netlist\n0.01::broken(3).\n0.01::broken(4).\ngate_and(3, a, b).\ngate_not(4, c).\n'
               '% gate_or(8, x, y).\nevidence(failure).\nquery(fault(3)).\nquery(fault(4)).\nquery(faults(3,4)).\n')
    OUT = ('Diagnosis: (Circuit-Fault: #t) | Prob: 0.28\n'
           'Gate 0 Fault | Posterior P(Gate|Fail): 0.357\n'
           'Gate 1 Fault | Posterior P(Gate|Fail): 0.714\n'
           'Gates 0 & 1 Fault | Posterior P(i,j | Fail): 0.0714\n')

    def test_the_queries_are_the_readers_atoms(self):
        self.assertEqual([bench.circulette_atom(self.GATES, f) for f in ((1,), (2, 0), (0, 1, 2))],
                         ['fault(7)', 'faults(5,9)', 'faults(5,7,9)'])
        self.assertEqual([bench.circulette_size(q) for q in ('fault(3)', 'faults(3, 4)', 'faults(1,2,3)', 'q1(1)')],
                         [1, 2, 3, None])
        self.assertEqual((bench.circulette_asked(3), bench.circulette_faults(8)), (7, 3))
        with self.assertRaises(ValueError):
            bench.circulette_faults(9)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'ct_x.pl')
            path.write_text(self.PROGRAM)
            self.assertEqual(bench.circulette_gates(str(path)), [3, 4])

    def test_the_calls_are_numbered_as_circulette_loops(self):
        for n in range(1, 8):
            order = [(i,) for i in range(n)] + [(i, j) for i in range(n) for j in range(i + 1, n)] + \
                    [(i, j, k) for i in range(n) for j in range(i + 1, n) for k in range(j + 1, n)]
            self.assertEqual(list(bench.circulette_order(n)), order, n)

    def test_an_exports_calls_are_rekeyed_by_the_gates(self):
        # 3 faults: q1 P(fail), q2 to q4 the singles, q5 to q7 the pairs, q8 the triple
        self.assertEqual(bench.circulette_rekey({'q1(1)': 1.0, 'q2(1)': 0.5, 'q5(1)': 0.25, 'q8(1)': 1e-9}, self.GATES),
                         {'fault(5)': 0.5, 'faults(5,7)': 0.25, 'faults(5,7,9)': 1e-9})
        for bad in ({'x': 0.5}, {'q9(1)': 0.5}, {'q2(1)': 0.5, 'q2(2)': 0.5}):
            with self.assertRaises(ValueError):
                bench.circulette_rekey(bad, self.GATES)

    def test_roulette_tddlogs_circuit_is_answered_as_the_netlist(self):
        # the circuit Roulette's tddlog engine writes: 4 calls of infer for 2 faults, q1 the failure
        text = ('% infer was called 4 times; query K is its K-th call.\n'
                '% q1(1): #t\nq1(1).\n% q2(1): #t\n% q3(1): #t\n% q4(1): #t\n')
        tddlog = {'q1(1)': 1.0, 'q2(1)': 0.357, 'q3(1)': 0.714, 'q4(1)': 0.0714}
        answers, p_fail = bench.circulette_circuit_answers(text, tddlog, math.log10(0.28), [3, 4])
        self.assertEqual(answers, {'fault(3)': 0.357, 'fault(4)': 0.714, 'faults(3,4)': 0.0714})
        self.assertAlmostEqual(p_fail, 0.28)
        # no fault fails the outputs: no posterior, as tddlog's on the netlist
        self.assertEqual(bench.circulette_circuit_answers(text, dict(tddlog, **{'q1(1)': 0.0}), 0.0, [3, 4]),
                         ('inconsistent', 0.0))
        self.assertEqual(bench.circulette_circuit_answers(text, 'inconsistent', -math.inf, [3, 4]), ('inconsistent', 0.0))
        # evidence that is not the failure, or a circuit of other faults, is refused
        with self.assertRaises(ValueError):
            bench.circulette_circuit_answers(text, dict(tddlog, **{'q1(1)': 0.5}), 0.0, [3, 4])
        with self.assertRaises(ValueError):
            bench.circulette_circuit_answers(text, tddlog, 0.0, [3, 4, 5])

    def test_a_posterior_circulette_does_not_print_is_answered_only_below_its_threshold(self):
        ref = {'system': 'tddlog',
               'answers': {'fault(5)': 0.5, 'fault(7)': 0.0, 'fault(9)': 0.5,
                           'faults(5,7)': 0.25, 'faults(5,9)': 1e-4, 'faults(7,9)': 0.25, 'faults(5,7,9)': 1e-9}}
        printed = {'fault(5)': 0.5, 'fault(9)': 0.5, 'faults(5,7)': 0.25, 'faults(7,9)': 0.25}
        rec = {'instance': 'ct_exp', 'system': 'roulette', 'answers': printed}
        self.assertEqual(bench.printed_answers(rec, ref), ref['answers'])
        # left out above its threshold: a pair at 3e-4, a single above 0, a triple at 1e-7
        for q, p in (('faults(5,9)', 3e-4), ('fault(7)', 1e-3), ('faults(5,7,9)', 1e-7)):
            high = dict(ref, answers=dict(ref['answers'], **{q: p}))
            self.assertNotIn(q, bench.printed_answers(rec, high))
        # any other system, or instance, keeps its answers as they are, as does no posterior
        for other in (dict(rec, system='tddlog'), dict(rec, instance='rl_hmm_8'), dict(rec, answers='inconsistent')):
            self.assertIs(bench.printed_answers(other, ref), other['answers'])

    def test_the_check_takes_a_posterior_circulette_does_not_print_below_its_threshold_as_answered(self):
        # ct_fulladder2: Roulette prints 166 of the 175 posteriors, the 9 it
        # leaves out pairs at 7e-5 to 8e-5, under Circulette's 2e-4
        ref = {'system': 'merlin', 'variant': 'bte',
               'answers': {'fault(5)': 0.5, 'fault(9)': 0.5, 'faults(5,9)': 7.1e-5, 'faults(5,7,9)': 1e-9}}
        rec = {'instance': 'ct_exp', 'system': 'roulette', 'variant': 'rsdd',
               'answers': {'fault(5)': 0.5, 'fault(9)': 0.5}}
        self.assertEqual(bench.left_out(rec, ref), [])
        self.assertEqual(bench.answers_check(rec, ref), (True, 0.0, 2))
        # one above its threshold is dropped, and the record is wrong
        high = dict(ref, answers=dict(ref['answers'], **{'faults(5,9)': 3e-4}))
        self.assertEqual(bench.left_out(rec, high), ['faults(5,9)'])
        self.assertEqual(bench.answers_check(rec, high)[:2], (False, math.inf))

    def args(self, **kw):
        base = dict(suite='circulette', only=[], families=[], dir='bench', timeout=30,
                    circulette_benchmarks='cb', roulette='racket {file}', roulette_variants=['rsdd'])
        base.update(kw)
        return SimpleNamespace(**base)

    def test_the_suite_grows_two_families(self):
        spec = bench.instances(self.args())
        self.assertEqual(len(spec), 42)
        self.assertTrue(all(g[0] == 'gen_circulette.py' for _, g in spec.values()))
        self.assertEqual([n for n, (f, _) in spec.items() if f == 'ct_mac'], [f'ct_mac{w}' for w in range(1, 7)])
        self.assertEqual(bench.series('ct_fulladder', 'ct_fulladder8'), 'ct_fulladder')
        # Roulette reads the netlist, the network tools the program's ground route
        self.assertIsNone(bench.ground_form(['gen_circulette.py'], 'roulette', 'rsdd'))
        self.assertEqual(bench.ground_form(['gen_circulette.py'], 'ace', 'auto'), 'hugin')
        # tddlog reads the netlist gen wrote beside its program, and the ProbLog family that program
        self.assertIn('gen_circulette.py', bench.PROGRAM_OF_TDDLOG)
        self.assertEqual(bench.original(['gen_circulette.py', 'root', '1-bit-mux'], '', 'd/ct_1-bit-mux'),
                         os.path.abspath('d/ct_1-bit-mux.cql'))
        with self.assertRaises(ValueError):
            bench.original(['gen_circulette.py', 'root', '1-bit-mux'], '')
        # every netlist has the SHA-256 gen checks
        self.assertEqual(set(bench.CT_SHA256), set(spec))
        self.assertTrue(all(re.fullmatch('[0-9a-f]{64}', h) for h in bench.CT_SHA256.values()))

    def test_gen_copies_each_netlist_and_checks_its_sha256(self):
        text = b'(def-netlist x\n(hash 0 1)\n(hash 1 1)\n(hash)\n(1 (w~ 0))\n)\n'
        gargs = ['gen_circulette.py', 'root', '1-bit-mux']
        with tempfile.TemporaryDirectory() as src, tempfile.TemporaryDirectory() as d, \
                patch.dict(bench.CT_SHA256, {'ct_1-bit-mux': bench.hashlib.sha256(text).hexdigest()}), \
                patch.object(bench, 'write_program') as wp:
            Path(src, 'ct_1-bit-mux.cql').write_bytes(text)
            a = self.args(dir=d, circulette_netlists=src)
            bench.gen_circulette(a, 'ct_1-bit-mux', gargs)
            self.assertEqual(Path(d, 'ct_1-bit-mux.cql').read_bytes(), text)
            wp.assert_called_once_with(a, 'ct_1-bit-mux', [os.path.abspath(os.path.join(d, 'ct_1-bit-mux.cql'))])
            Path(d, 'ct_1-bit-mux.cql').unlink()
            # gen stops, writing nothing, on another file, a missing one, an instance without a SHA-256
            Path(src, 'ct_1-bit-mux.cql').write_bytes(text.replace(b'w~', b'w&'))
            for name, why in (('ct_1-bit-mux', 'SHA-256'), ('ct_exp', 'ct_exp.cql'), ('ct_new', 'no SHA-256')):
                with self.subTest(name=name), self.assertRaises(SystemExit) as e:
                    bench.gen_circulette(a, name, gargs)
                self.assertIn(why, str(e.exception))
            self.assertEqual(os.listdir(d), [])
            self.assertEqual(wp.call_count, 1)

    def test_gen_needs_the_netlists_for_a_circulette_instance_and_them_only(self):
        with tempfile.TemporaryDirectory() as d:
            for kw, why in ((dict(), '--circulette-netlists DIR'),
                            (dict(suite='ppl', only=['nodice_survey'], circulette_netlists=d), 'none is selected')):
                with self.subTest(kw=kw), self.assertRaises(SystemExit) as e:
                    bench.gen(self.args(dir=d, cnf=False, alea='', pr=False, **kw))
                self.assertIn(why, str(e.exception))

    def run_circulette(self, out, err='roulette_timer: program 2.25 s\n', program=None):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'ct_1-bit-mux.pl')
            path.write_text(program or self.PROGRAM)
            with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                            'stderr': err}) as timed:
                r = bench.run_roulette(self.args(dir=d), 'ct_1-bit-mux', str(path), 'rsdd')
        return r, timed.call_args_list

    def test_the_posteriors_are_keyed_by_the_gates(self):
        r, calls = self.run_circulette(self.OUT)
        self.assertEqual(calls[0][0][0], ['racket', '-l-', 'raco', 'make', 'cb/circulette.rkt', bench.ROULETTE_TIMER])
        # under the timer, whose span is Roulette's own time (own_seconds)
        self.assertEqual(calls[1][0][0][:3], ['racket', bench.ROULETTE_TIMER, 'cb/circulette.rkt'])
        self.assertTrue(calls[1][0][0][3].endswith('/ct_1-bit-mux.cql'))
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(r['answers'], {'fault(3)': 0.357, 'fault(4)': 0.714, 'faults(3,4)': 0.0714})
        self.assertEqual(r['stats'], {'p_fail': 0.28, 'inproc_seconds': 2.25})
        self.assertEqual(bench.own_seconds(dict(r, system='roulette', **r['stats'])), 2.25)   # as the record holds it
        r, _ = self.run_circulette(self.OUT, err='')
        self.assertEqual((r['status'], r['error']), ('error', 'no roulette_timer line: ' + self.OUT.splitlines()[-1]))

    def test_no_fault_that_fails_the_outputs_has_no_posterior(self):
        r, _ = self.run_circulette('Diagnosis: (Circuit-Fault: #t) | Prob: 0\n')
        self.assertEqual((r['status'], r['answers'], r['stats']['p_fail']), ('ok', 'inconsistent', 0.0))
        # posteriors given a failure of probability 0, or of a fault the netlist lacks, are refused
        r, _ = self.run_circulette(self.OUT.replace('Prob: 0.28', 'Prob: 0'))
        self.assertIn('given a failure of probability 0', r['error'])
        r, _ = self.run_circulette(self.OUT.replace('Gate 1 Fault', 'Gate 2 Fault'))
        self.assertIn('the netlist has 2 gates', r['error'])
        r, _ = self.run_circulette(self.OUT + self.OUT.splitlines()[1] + '\n')
        self.assertIn('printed twice', r['error'])


def fold_files(directory, names, suite='standard'):
    """The files tddlog reads of the FOLD instances `names` (bench.py gen
    writes them): their family's program, their data files and NAME.args,
    empty, in `directory`, so that a run with a mocked `timed` finds its inputs."""
    spec = {'standard': bench.SPEC, 'indicators': bench.INDICATORS}[suite]
    directory = Path(directory)
    for name in names:
        gargs = spec.get(name, (None, ['none']))[1]
        if not bench.fold_of(gargs):
            continue
        program, _, keys = bench.fold_of(gargs)
        for rel in [Path(bench.FOLD_DIR) / (program + '.pl'), *(Path(bench.DATA_DIR) / (k + '.pl') for k in keys),
                    Path(name + '.args')]:
            (directory / rel).parent.mkdir(parents=True, exist_ok=True)
            (directory / rel).touch()
    return str(directory)


class SkipTests(unittest.TestCase):
    """A system skips the larger sizes of a program once it ran out of time
    or memory on a smaller one, and nothing else."""

    def test_series(self):
        series = bench.series
        self.assertEqual(series('grid', 'grid10'), 'grid')
        self.assertEqual(series('bb_GPA', 'bb_GPA_w19'), 'bb_GPA')
        self.assertEqual(series('arith', 'arith_gcd_small'), 'arith_gcd')
        self.assertEqual(series('nodice', 'nodice_coupon2-12-obs'), 'nodice_coupon2')
        self.assertEqual(series('nodice', 'nodice_coupon2-2-obs'), 'nodice_coupon2')
        self.assertEqual(series('nodice', 'nodice_brp1024-2'), 'nodice_brp')
        self.assertEqual(series('dice_scaled', 'dice_pmc1_k40'), 'dice_pmc1')
        for family, name in [('arith', 'arith_book'), ('nodice', 'nodice_alarm'), ('nodice', 'nodice_uf20-0990'),
                             ('nodice', 'nodice_3sat-0'),
                             ('dice', 'dice_alarm'), ('bn', 'bn_asia'), ('bnev', 'bn_asia_ev'), ('bn', 'rl_bn_alarm'),
                             ('aceblockmap', 'ace_blockmap_05_01'), ('uaipromedus', 'uai_promedus_30')]:
            self.assertIsNone(series(family, name), name)

    def test_ran_out(self):
        self.assertTrue(bench.ran_out({'status': 'timeout', 'seconds': 1}))
        self.assertTrue(bench.ran_out({'status': 'error', 'error': "terminate called after 'std::bad_alloc'"}))
        self.assertTrue(bench.ran_out({'status': 'error', 'error': 'memory allocation of 8 bytes failed'}))
        self.assertFalse(bench.ran_out({'status': 'error', 'error': 'parse error: line 1'}))
        self.assertFalse(bench.ran_out({'status': 'unsupported', 'error': 'no translation'}))
        # a translator's time limit: an error, its text kept, that skips the larger sizes as a timeout does
        limit = {'status': 'error', 'seconds': 0, 'error': 'problog2souffle.py: no translation within 600 s'}
        self.assertTrue(bench.translation_limit(limit) and bench.ran_out(limit))
        # PSouffle's compilation stopped at its bound: charged as a timeout (rel/charge.py), so it skips as one
        stopped = {'status': 'error', 'seconds': 0.0, 'error': "PSouffle's compilation: timeout ", 'stats': {'compile_s': 3600.0}}
        self.assertTrue(bench.compile_timeout(stopped) and bench.ran_out(stopped))
        failed = {'status': 'error', 'seconds': 0.0, 'error': "PSouffle's compilation: error souffle: syntax error"}
        self.assertFalse(bench.compile_timeout(failed) or bench.ran_out(failed))
        self.assertFalse(bench.ran_out({'status': 'error', 'error': 'dice2prism.py: Traceback ...\nKeyError'}))
        self.assertFalse(bench.ran_out({'status': 'ok', 'seconds': 1}))

    def test_memout_is_an_allocation_failure_or_the_cap(self):
        # the systems' allocation failures as the records keep them
        for text in ["terminate called after throwing an instance of 'std::bad_alloc'",
                     'memory allocation of 17448304656 bytes failed', 'CUDD: unable to allocate 32736 bytes',
                     'numpy._core._exceptions._ArrayMemoryError: Unable to allocate 2.1 GiB for an array',
                     'Exception in thread "main" java.lang.OutOfMemoryError: Java heap space',
                     'ERROR: LoadError: OutOfMemoryError()', 'malloc failed in new_sdd_node',
                     'GNU MP: Cannot reallocate memory (old_size=8 new_size=16)', 'Fatal error: out of memory',
                     'core.exception.OutOfMemoryError@core/lifetime.d(126): Memory allocation failed',
                     'compilation error: TDD operation failed: memory allocation refused by the byte budget or the '
                     'allocator', 'grounding error: the grounding ran out of memory',
                     'Unable to resize subtable 201 for lack of memory',
                     '12: 0x12e543f v8::internal::Runtime_AllocateInYoungGeneration(int, unsigned long*)',
                     bench.cap_error(31 << 20)]:
            self.assertTrue(bench.memout({'status': 'error', 'error': text}), text)
            self.assertTrue(bench.ran_out({'status': 'error', 'error': text}), text)
        # texts that name memory without an allocation failure: an error, not a memout
        # (PRISM's MTBDD until past 2^63 states, where it builds no ODD, stops with a
        # NullPointerException at any memory: an error, its header naming memory)
        for text in ['Error: Property not found\nMemory limits: cudd=1g, java(heap)=3.9g',
                     'Memory limits: cudd=5g, java(heap)=4.8g\nStates:      4.539391276497261E19 (1 initial)\n'
                     'Warning: Switching to MTBDD engine, as number of states is too large for Sparse engine.\n'
                     'Engine: MTBDD\n\njava.lang.NullPointerException\n    at mtbdd.PrismMTBDD.ProbUntil(PrismMTBDD.java:413)\n'
                     '\nError: Caught unhandled exception, aborting....',
                     'Assertion failed: makeOrBalanced(const std::vector<BddNodeRef, std::allocator<BddNodeRef> >&)',
                     'geni_wrap.sh: line 9: ulimit: virtual memory: cannot modify limit: Invalid argument',
                     '[2340709] signal 15: Terminated\nAllocations: 12625286 (Pool: 12624805; Big: 481); GC: 10',
                     'ERROR: LoadError: syntax: invalid syntax (memory-error out of gc handles)',
                     'parse error: line 1']:
            self.assertFalse(bench.memout({'status': 'error', 'error': text}), text)
        self.assertFalse(bench.memout({'status': 'timeout', 'seconds': 300}))
        # a text that names memory in a way neither list knows is refused, and does not skip
        unknown = {'status': 'error', 'error': 'shared memory segment detached'}
        with self.assertRaisesRegex(bench.Unclassed, 'shared memory segment'):
            bench.memout(unknown)
        self.assertFalse(bench.ran_out(unknown))

    def test_the_cap_sets_the_systems_memory_options(self):
        def parsed(**given):
            fields = dict(cmd='run', suite='standard', ace_benchmarks=None, uai_benchmarks=None, biomine_benchmarks=None,
                          ace='ace', ace_variants=None, ace_heap=None, ace_java=None, sharpsat_td='',
                          sharpsat_td_decot=None, prism_heap=None, mem_cap=None)
            return SimpleNamespace(**(fields | given))
        args = parsed()
        bench.ace_defaults(args)
        self.assertEqual((args.ace_heap, args.prism_heap), ('8g', '4g'))
        args = parsed(mem_cap=30 << 30)
        bench.ace_defaults(args)
        self.assertEqual((args.ace_heap, args.prism_heap), ('24576m', '24576m'))
        args = parsed(mem_cap=30 << 30, ace_heap='20g', prism_heap='8g')
        bench.ace_defaults(args)
        self.assertEqual((args.ace_heap, args.prism_heap), ('20g', '8g'))
        with self.assertRaisesRegex(SystemExit, '--ace-heap 28g is more than 0.8 of --mem-cap'):
            bench.ace_defaults(parsed(mem_cap=30 << 30, ace_heap='28g'))
        with self.assertRaisesRegex(SystemExit, '--prism-heap 28g is more than 0.8 of --mem-cap'):
            bench.ace_defaults(parsed(mem_cap=30 << 30, prism_heap='28g'))
        cmd = ['env', 'JULIA_DEPOT_PATH=d', '/x/julia-1.11.9/bin/julia', '--startup-file=no', 'f.jl']
        self.assertEqual(bench.julia_heap_hint(cmd, 10 << 30),
                         cmd[:3] + ['--heap-size-hint=8192M'] + cmd[3:])
        self.assertEqual(bench.julia_heap_hint(cmd[:3] + ['--heap-size-hint=1G'], 10 << 30), cmd[:3] + ['--heap-size-hint=1G'])
        self.assertEqual(bench.julia_heap_hint(['java', '-Xmx1g'], 10 << 30), ['java', '-Xmx1g'])

    def test_prism_gets_its_heap_for_its_decision_diagrams_on_every_model(self):
        args = SimpleNamespace(prism='/p', prism_heap='24576m')
        for dice, props in [(True, 'm.props'), (False, 'P=? [F<=10 "allStrike"]')]:
            cmd = bench.checker_command(args, 'prism', 'hybrid', 'm.prism', props, dice=dice)
            self.assertEqual(cmd[cmd.index('-javamaxmem') + 1], '24576m')
            self.assertEqual(cmd[cmd.index('-cuddmaxmem') + 1], '24576m')   # not CUDD's default of 1 GB
            self.assertEqual('-epsilon' in cmd, dice)

    def test_a_record_keeps_its_commands_memory(self):
        bench.LANE.usage = {}
        r = bench.timed([sys.executable, '-c', 'b = bytearray(64 << 20); print(len(b))'], 60)
        self.assertEqual(r['status'], 'ok')
        rec = {'status': 'ok'}
        self.assertIs(bench.take_usage(rec, r), r)
        self.assertGreater(rec['maxrss_kb'], 64 << 10)
        self.assertIsNone(bench.LANE.usage)
        # a record whose command the cap killed is that memout, whatever the system made of it
        bench.LANE.usage = {'maxrss_kb': 10, 'mem_peak_kb': 1 << 20, 'mem_cap_kb': 1 << 20, 'cap_killed': True}
        rec = {'status': 'error', 'error': 'alea: no output'}
        r = bench.take_usage(rec, {'status': 'error', 'error': 'alea: no output'})
        self.assertEqual(rec, {'status': 'error', 'error': bench.cap_error(1 << 20), 'maxrss_kb': 10,
                               'mem_peak_kb': 1 << 20, 'mem_cap_kb': 1 << 20})
        self.assertTrue(bench.ran_out(r))

    def test_only_a_smaller_size_out_of_time_or_memory_skips(self):
        only = ['grid4', 'grid5', 'grid6', 's6', 's10', 'bn_cancer', 'bn_asia']
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        args = SimpleNamespace(suite='standard', only=only,
                               families=[], strategies=['auto'], kbs=[], pita='', aspmc='', scallop='', dice='',
                               dir=fold_files(directory.name, only), out='results.jsonl', tddlog='tddlog', timeout=1, no_skip=False,
                               exact=False, ace='', merlin='', ganak='', pr=False, cnf=False)
        fails = {'grid4': {'status': 'timeout', 'seconds': 1}, 'bn_cancer': {'status': 'timeout', 'seconds': 1},
                 's6': {'status': 'error', 'seconds': 0.1, 'error': 'parse error: line 1', 'returncode': 1}}

        def timed(cmd, timeout, cwd=None, env=None):
            ok = {'status': 'ok', 'seconds': 0.1, 'stdout': 'q:\t0.5\n', 'stderr': ''}
            # a program's stem, or a network's (tddlog reads bn_cancer's cancer.net itself)
            stems = {Path(a).stem for a in cmd}
            return dict(next((f for n, f in fails.items() if {n, n.removeprefix('bn_')} & stems), ok))
        # a folded instance's command ends in its arguments: its name, for the mock to tell them apart
        with patch('builtins.open', mock_open()) as output, patch.object(bench, 'timed', timed), \
                patch.object(bench, 'arguments_of', lambda path: [Path(path).stem]), \
                patch.object(bench, 'queries', return_value=['q']), redirect_stdout(io.StringIO()):
            bench.run(args)
        records = [bench.json.loads(c.args[0]) for c in output().write.call_args_list]
        # the grid's larger sizes after its timeout; the smokers after an error, and
        # the networks, are no sizes of a program that ran out
        self.assertEqual({r['instance']: r['status'] for r in records},
                         {'grid4': 'timeout', 'grid5': 'skipped', 'grid6': 'skipped', 's6': 'error', 's10': 'ok',
                          'bn_cancer': 'timeout', 'bn_asia': 'ok'})


class RotateTests(unittest.TestCase):
    def test_rotations_put_each_system_in_each_place(self):
        systems = [('tddlog', 'auto'), ('dice', 'bdd'), ('alea', 'cudd')]
        self.assertEqual(bench.system_order(systems, 'dice_alarm', None), systems)
        for name in ['bn_asia', 'dice_alarm', 'dice_pmc1']:
            orders = [bench.system_order(systems, name, r) for r in range(3)]
            for order in orders:
                self.assertEqual(sorted(order), sorted(systems))
            for place in range(3):
                self.assertEqual(sorted(o[place] for o in orders), sorted(systems))
        # Instances that start together start with different systems.
        firsts = {bench.system_order(systems, n, 0)[0] for n in ['bn_asia', 'bn_alarm', 'bn_child', 'bn_water']}
        self.assertGreater(len(firsts), 1)

    def test_rotate_needs_two_systems(self):
        args = SimpleNamespace(suite='standard', only=['bn_asia'], families=[], strategies=['auto'], kbs=[],
                               pita='', aspmc='', scallop='', dice='', dir='bench', out='results.jsonl',
                               tddlog='tddlog', timeout=1, no_skip=True, exact=False, ace='', merlin='',
                               ganak='', pr=False, rotate=0)
        with self.assertRaisesRegex(SystemExit, '--rotate'):
            bench.run(args)

    def test_systems_keeps_the_named_systems(self):
        # tddlog and ProbLog given, tddlog alone run: one system, which --rotate cannot order
        args = SimpleNamespace(suite='standard', only=['bn_asia'], families=[], strategies=['auto'], kbs=['ddnnf'],
                               pita='', aspmc='', scallop='', dice='', dir='bench', out='results.jsonl',
                               tddlog='tddlog', timeout=1, no_skip=True, exact=False, ace='', merlin='',
                               ganak='', pr=False, rotate=0, systems=['tddlog'])
        with self.assertRaisesRegex(SystemExit, '--rotate'):
            bench.run(args)
        args.systems = ['pita']
        with self.assertRaisesRegex(SystemExit, '--systems pita: not one of the systems the other options run '
                                                r'\(tddlog, problog\)'):
            bench.run(args)


class TddlogArmTests(unittest.TestCase):
    """`--tddlog-arm STRATEGY=PATH` runs one strategy with a binary of its
    own, an arm built from another commit."""

    def args(self, d, **kw):
        base = dict(suite='standard', only=['bn_asia'], families=[], strategies=['auto', 'program'], kbs=[],
                    pita='', aspmc='', scallop='', dice='', dir=d, out=str(Path(d) / 'results.jsonl'),
                    tddlog='tddlog', timeout=10, no_skip=True, exact=False, ace='', merlin='', ganak='',
                    pr=False, cnf=False)
        base.update(kw)
        return SimpleNamespace(**base)

    def test_the_strategy_runs_with_its_binary(self):
        with scratch_dir('arm-') as d:
            other = Path(d) / 'tddlog-other'
            other.write_text('#!/bin/sh\n')
            other.chmod(0o755)
            args = self.args(d, tddlog_arm=[f'program={other}'])
            with redirect_stdout(io.StringIO()):   # the network's program by the built tddlog
                bench.gen(SimpleNamespace(**{**vars(args), 'tddlog': str(bench.source_tree.BINARY)}))
            calls = []

            def timed(cmd, timeout, cwd=None, env=None):
                calls.append(cmd)
                return {'status': 'ok', 'seconds': 0.5, 'stdout': '', 'stderr': ''}
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()):
                bench.run(args)
        self.assertEqual([(c[0], c[c.index('--vtree') + 1]) for c in calls],
                         [('tddlog', 'auto'), (str(other), 'program')])

    def test_a_wrong_arm_fails_before_any_run(self):
        with scratch_dir('arm-') as d:
            exe = Path(d) / 'tddlog-other'
            exe.write_text('#!/bin/sh\n')
            exe.chmod(0o755)
            plain = Path(d) / 'not-executable'
            plain.write_text('')
            for arm, msg in [([f'decision={exe}'], 'decision is not one of the strategies run'),
                             (['program'], 'the form is STRATEGY=PATH'),
                             ([f'program={exe}', f'program={exe}'], 'program has a binary already'),
                             ([f'program={plain}'], 'is not an executable')]:
                with self.assertRaisesRegex(SystemExit, msg), \
                        patch.object(bench, 'timed', side_effect=AssertionError('ran')):
                    bench.run(self.args(d, tddlog_arm=arm))


class FoldTests(unittest.TestCase):
    """A family of FOLD is one program and its arguments: gen writes the
    program once (programs/PROGRAM.pl), each instance's arguments
    (NAME.args) and NAME.pl, the program with them in its places, which
    the other systems read; tddlog reads the program and the arguments."""

    def test_tddlog_reads_the_program_and_its_arguments(self):
        with scratch_dir('fold-') as d:
            args = SimpleNamespace(suite='lit', only=['threedie10', 'threedie100', 'pgrid16_4'], families=[],
                                   strategies=['auto'], kbs=['ddnnf'], problog='problog', pita='', aspmc='',
                                   scallop='', dice='', dir=d, out=str(Path(d) / 'results.jsonl'), tddlog='tddlog',
                                   timeout=10, no_skip=True, exact=False, ace='', merlin='', ganak='', pr=False,
                                   cnf=False)
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**{**vars(args), 'tddlog': str(bench.source_tree.BINARY)}))
            self.assertEqual(sorted(os.listdir(Path(d) / 'programs')), ['pgrid16.pl', 'threedie.pl'])
            template = (Path(d) / 'programs' / 'threedie.pl').read_text()
            self.assertEqual(template, bench.fold_template(['gen_threedie.py', '10'])[1])
            self.assertEqual((Path(d) / 'threedie100.args').read_text(), '100\n')
            self.assertEqual((Path(d) / 'pgrid16_4.args').read_text(), 'n_12_12\n')
            self.assertEqual((Path(d) / 'threedie100.pl').read_text(),
                             bench.folded_program(['gen_threedie.py', '100']))
            self.assertIn('query(on(100,1)).', (Path(d) / 'threedie100.pl').read_text())
            calls = []

            def timed(cmd, timeout, cwd=None, env=None):
                calls.append(cmd)
                return {'status': 'ok', 'seconds': 0.5, 'stdout': '', 'stderr': ''}
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()):
                bench.run(args)
            recs = [bench.json.loads(line) for line in open(args.out)]
            # the skip rule follows the family's sizes: tddlog out of time on threedie10 is skipped on
            # threedie100 (the record's program, not the series, once held the name the rule keys on)
            skip = SimpleNamespace(**{**vars(args), 'no_skip': False, 'kbs': [], 'out': str(Path(d) / 'skip.jsonl')})

            def out_of_time(cmd, timeout, cwd=None, env=None):
                return {'status': 'timeout', 'seconds': timeout}
            with patch.object(bench, 'timed', out_of_time), redirect_stdout(io.StringIO()):
                bench.run(skip)
            skipped = {r['instance']: r['status'] for r in map(bench.json.loads, open(skip.out))}
        self.assertEqual((skipped['threedie10'], skipped['threedie100']), ('timeout', 'skipped'))
        self.assertIn(['tddlog', '--stats', '--precision', '17', '--vtree', 'auto', f'{d}/programs/threedie.pl', '100'],
                      calls)
        self.assertIn(['tddlog', '--stats', '--precision', '17', '--vtree', 'auto', f'{d}/programs/pgrid16.pl',
                       'n_12_12'], calls)
        self.assertIn(['problog', '-k', 'ddnnf', f'{d}/threedie100.pl'], calls)
        mine = {r['instance']: r for r in recs if r['system'] == 'tddlog'}
        self.assertEqual((mine['threedie100']['original'], mine['threedie100']['arguments']), ('threedie.pl', ['100']))
        self.assertEqual(mine['pgrid16_4']['arguments'], ['n_12_12'])
        self.assertTrue(all('original' not in r for r in recs if r['system'] != 'tddlog'))

    def test_a_scaled_loop_is_read_as_its_program_and_its_count(self):
        # a folded run's layout: D/programs/LOOP.dice and D/NAME.args beside D/NAME.dice
        with scratch_dir('fold-') as d:
            dice = os.path.join(d, 'dice_pmc1_k40.dice')
            gargs = ['dice2problog.py', dice]
            self.assertIsNone(bench.folded(None, 'dice_pmc1_k40', gargs, 'auto'))   # no count's file: the old form
            (Path(d) / 'dice_pmc1_k40.args').write_text('40\n')
            self.assertEqual(bench.folded(None, 'dice_pmc1_k40', gargs, 'auto'),
                             (os.path.join(d, 'programs', 'pmc1.dice'), os.path.join(d, 'dice_pmc1_k40.args')))
            self.assertIsNone(bench.folded(None, 'dice_alarm', ['dice2problog.py', os.path.join(d, 'dice_alarm.dice')],
                                           'auto'))
            self.assertEqual(bench.folded_data(None, gargs), [])


class SuiteTests(unittest.TestCase):
    def test_large_instances_are_selected_and_recorded(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        args = SimpleNamespace(suite='indicators', only=['grid40'], families=[],
                               strategies=['closed'], kbs=[], pita='', aspmc='', scallop='', dice='',
                               dir=fold_files(directory.name, ['grid40'], 'indicators'), out='results.jsonl', tddlog='tddlog', timeout=1, no_skip=True,
                               exact=False, ace='', merlin='', ganak='', pr=False)
        result = {'status': 'ok', 'seconds': 0.1, 'stdout': 'path(n0_0,n39_39): 0.5\n', 'stderr': ''}
        with patch('builtins.open', mock_open()) as output, patch.object(bench, 'timed', return_value=result), \
                patch.object(bench, 'queries', return_value=['path(n0_0,n39_39)']), redirect_stdout(io.StringIO()):
            bench.run(args)
        record = bench.json.loads(output().write.call_args.args[0])
        self.assertEqual(record['instance'], 'grid40')
        self.assertEqual(record['family'], 'grid')
        self.assertEqual(record['status'], 'ok')
        args.suite = 'standard'
        with self.assertRaisesRegex(SystemExit, 'unknown instances: grid40'):
            bench.instances(args)

    def run_exact(self, only, suite='standard'):
        args = SimpleNamespace(suite=suite, only=only, families=[], strategies=[], kbs=[], pita='',
                               aspmc='', scallop='', dice='', dir='bench', out='results.jsonl',
                               tddlog='tddlog', timeout=1, no_skip=True, exact=True, ace='', merlin='', ganak='', pr=False)
        result = {'status': 'ok', 'seconds': 0.1, 'stdout': 'akt(v_1):\t0.5\n', 'stderr': ''}
        with patch('builtins.open', mock_open()) as output, \
                patch.object(bench, 'timed', return_value=result) as timed, \
                patch.object(bench, 'queries', return_value=['akt(v_1)']), redirect_stdout(io.StringIO()):
            bench.run(args)
        return [bench.json.loads(c.args[0]) for c in output().write.call_args_list], timed

    def test_the_exact_reference_answers_the_networks_alone(self):
        records, timed = self.run_exact(['bn_sachs', 'grid6'])
        self.assertEqual([(r['instance'], r['system'], r['variant']) for r in records],
                         [('bn_sachs', 'exact', 'cliquetree')])
        self.assertTrue(timed.call_args.args[0][1].endswith('bn_exact.py'))
        self.assertEqual(records[0]['answers'], {'akt(v_1)': 0.5})

    def test_the_literature_families_have_an_exact_reference_of_their_own(self):
        records, timed = self.run_exact(['pdb10', 'mnist_sum4'], suite='lit')
        self.assertEqual([(r['instance'], r['family'], r['system'], r['variant']) for r in records],
                         [('mnist_sum4', 'mnist_sum', 'exact', 'lit'), ('pdb10', 'pdb', 'exact', 'lit')])
        self.assertEqual([c.args[0][1:4] for c in timed.call_args_list],
                         [[str(scripts / 'lit_exact.py'), 'mnist_sum', str(Path('bench') / 'mnist_sum4.pl')],
                          [str(scripts / 'lit_exact.py'), 'pdb', str(Path('bench') / 'pdb10.pl')]])

    def test_the_exact_reference_without_a_network_is_refused(self):
        with self.assertRaisesRegex(SystemExit, 'none is selected'):
            self.run_exact(['grid6'])


# A small ProbLog program with positive and negative evidence; the
# probabilities of its ground atoms are found by enumerating the worlds of
# its facts (`world_probabilities`).
ALARM = """0.3::b. 0.2::e. 0.9::ab. 0.4::ae. 0.7::c. 0.6::d.
alarm :- b, ab.
alarm :- e, ae.
calls :- alarm, c.
calls :- d.
query(b). query(alarm).
evidence(calls). evidence(e, false).
"""


def world_probabilities(text):
    """{query: probability} of a ground program of facts `p::a.` and rules
    whose bodies are literals: the rules without negation are applied up to
    their least model, then the others once each in program order (the
    programs below negate only atoms that nothing after them defines)."""
    from lpad2problog import split_clauses, split_top
    facts, rules, queries = {}, [], []
    for clause in split_clauses(text):
        m = re.fullmatch(r'([0-9.]+)::(\w+)', clause)
        if m:
            facts[m.group(2)] = float(m.group(1))
        elif clause.startswith('query('):
            queries.append(clause[6:-1])
        else:
            head, body = clause.split(':-')
            rules.append((head.strip(), [lit.strip() for lit in split_top(body, ',')]))
    holds = lambda lit, model: lit[2:] not in model if lit.startswith('\\+') else lit in model
    names, out = list(facts), dict.fromkeys(queries, 0.0)
    for bits in range(1 << len(names)):
        world = [n for i, n in enumerate(names) if bits >> i & 1]
        weight = 1.0
        for i, n in enumerate(names):
            weight *= facts[n] if bits >> i & 1 else 1 - facts[n]
        model, grew = set(world), True
        while grew:
            grew = False
            for head, body in rules:
                if head not in model and all(not l.startswith('\\+') for l in body) \
                        and all(holds(l, model) for l in body):
                    model.add(head)
                    grew = True
        for head, body in rules:
            if any(l.startswith('\\+') for l in body) and all(holds(l, model) for l in body):
                model.add(head)
        for q in queries:
            out[q] += weight * (q in model)
    return out


class AspmcTests(unittest.TestCase):
    def test_evidence_is_asked_as_queries(self):
        conditioned = bench.aspmc_conditioned(ALARM)
        self.assertNotIn('evidence', conditioned)
        p = world_probabilities(conditioned)
        out = ''.join(f'[RESULT] aspmc: {q}: {v!r}\n' for q, v in p.items())
        # P(q | calls, not e), from the worlds of the program without its evidence.
        joint = world_probabilities(ALARM.replace('query(b). query(alarm).', '')
                                    .replace('evidence(calls). evidence(e, false).',
                                             'qb :- b, calls, \\+e. qa :- alarm, calls, \\+e. '
                                             'ev :- calls, \\+e. query(qb). query(qa). query(ev).'))
        answers = bench.parse_aspmc(out)
        self.assertEqual(set(answers), {'b', 'alarm'})
        self.assertAlmostEqual(answers['b'], joint['qb'] / joint['ev'], places=12)
        self.assertAlmostEqual(answers['alarm'], joint['qa'] / joint['ev'], places=12)
        self.assertNotAlmostEqual(answers['b'], 0.3, places=3)

    def test_a_query_with_a_body_keeps_its_solutions(self):
        program = ('person(a). person(b). 0.3::smokes(X) :- person(X).\n'
                   'query(smokes(X)) :- person(X).\n'
                   'evidence (\n smokes(a), false\n).\n')
        conditioned = bench.aspmc_conditioned(program)
        self.assertIn('bench_e :- \\+smokes(a).', conditioned)
        self.assertIn('bench_qe_smokes(X) :- person(X), smokes(X), \\+smokes(a).', conditioned)
        self.assertIn('query(bench_qe_smokes(X)) :- person(X).', conditioned)
        self.assertEqual(bench.parse_aspmc('[RESULT] aspmc: bench_e: 0.7\n'
                                           '[RESULT] aspmc: bench_qe_smokes(a): 0.0\n'
                                           '[RESULT] aspmc: bench_qe_smokes(b): 0.21\n'),
                         {'smokes(a)': 0.0, 'smokes(b)': 0.21 / 0.7})

    def test_impossible_evidence_is_inconsistent(self):
        self.assertEqual(bench.parse_aspmc('[RESULT] aspmc: bench_e: 0.0\n'
                                           '[RESULT] aspmc: bench_qe_q: 0.0\n'), 'inconsistent')
        # bench_e no rule derives, which aspmc's grounder removes and does not print
        # (Circulette's ct_ALU_Controller): inconsistent where the evidence was asked
        self.assertEqual(bench.parse_aspmc('', conditioned=True), 'inconsistent')
        self.assertEqual(bench.parse_aspmc(''), {})

    def test_a_failure_reading_the_grounding_is_an_error(self):
        # aspmc on a Circulette netlist's program, which its language expresses: its own
        # failure, kept as the record, with no ground program asked for
        err = ('    self._process_grounding(clingo_control)\n'
               '  File "aspmc/programs/problogprogram.py", line 151, in _process_grounding\n'
               '    head_name = "" if len(o.head) == 0 else symbol_map[abs(o.head[0])]\nKeyError: 45\n')
        self.assertNotRegex(err, bench.ASPMC_REFUSED)
        with patch('builtins.open', mock_open(read_data='0.5::a. query(a).')), \
                patch.object(bench.os, 'makedirs'), \
                patch.object(bench, 'timed', return_value={'status': 'error', 'seconds': 0.4,
                                                           'error': err}) as run:
            result = bench.run_aspmc(SimpleNamespace(dir='bench', aspmc='python3', timeout=1,
                                                     tddlog='tddlog'), 'ct_x', 'ct_x.pl')
        self.assertEqual((result['status'], result['error']), ('error', err))
        self.assertNotIn('stats', result)
        run.assert_called_once()
        self.assertEqual(run.call_args[0][0][1:5], ['-u', '-m', 'aspmc.main', '-m'])

    def test_an_observation_with_a_body_is_unsupported(self):
        # the harness's rewrite of the evidence cannot write it, and there is no ground fallback
        # (aspmc reads the first-order program)
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'coin.pl')
            with open(path, 'w') as f:
                f.write('0.5::obs. evidence(obs) :- true.\n')
            with patch.object(bench, 'timed') as run:
                result = bench.run_aspmc(SimpleNamespace(dir=d, aspmc='python3', timeout=1), 'coin', path)
            run.assert_not_called()
        self.assertEqual(result['status'], 'unsupported')
        self.assertTrue(result['error'].startswith('translator: an observation with a body'), result['error'])
        self.assertNotIn('stats', result)

    def test_aspmcs_refusal_is_unsupported(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'p.pl')
            with open(path, 'w') as f:
                f.write('0.5::a.\nquery(a).\n')
            for error, status in [('lark.exceptions.UnexpectedCharacters: ...', 'unsupported'),
                                  ('error: unsafe variables in: p(X)', 'unsupported'),
                                  ('MemoryError', 'error')]:
                with self.subTest(error=error), patch.object(bench, 'timed', return_value={
                        'status': 'error', 'seconds': 0.1, 'error': error}):
                    self.assertEqual(bench.run_aspmc(SimpleNamespace(dir=d, aspmc='python3', timeout=1),
                                                     'p', path)['status'], status)

    def test_a_circuit_it_refuses_is_given_as_its_ground_program(self):
        # a circuit has no first-order source: its ground program, renamed
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append((cmd, timeout))
            if cmd[1] == '--ground':
                return {'status': 'ok', 'seconds': 0.25, 'stderr': '',
                        'stdout': "0.5::'$sw0'.\ng_1 :- '$sw0'.\nquery(g_1).\n"}
            if len(calls) == 1:
                return {'status': 'error', 'seconds': 0.1, 'error': 'lark.exceptions.UnexpectedCharacters: ...'}
            return {'status': 'ok', 'seconds': 1.0, 'stdout': '', 'stderr': '[RESULT] aspmc: p1: 0.5\n'}
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'bb_c.pl')
            with open(path, 'w') as f:
                f.write('0.5::a.\nquery(a).\n')
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=10, tddlog='tddlog')
            with patch.object(bench, 'timed', timed):
                r = bench.run_aspmc(args, 'bb_c', path, circuit=True)
            self.assertEqual(calls[1], (['tddlog', '--ground', path], 10))
            ground = os.path.join(d, 'aspmc-bb_c', 'bb_c.ground.pl')
            self.assertEqual(calls[2][0][-1], ground)
            self.assertEqual(calls[2][1], 9.75)
            with open(ground) as f:
                self.assertEqual(f.read(), '0.5::p0.\np1 :- p0.\nquery(p1).\n')
        self.assertEqual((r['status'], r['seconds'], r['answers']), ('ok', 1.25, {'g_1': 0.5}))
        self.assertEqual(r['stats'], {'aspmc_route': 'ground'})

    def test_the_ground_programs_form_keeps_apart_from_the_refused_programs(self):
        # each program aspmc reads in its own notation keeps a path of its own, so that the
        # capture shows the one it refused and the one it answered apart
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            if cmd[1] == '--ground':
                return {'status': 'ok', 'seconds': 0.25, 'stderr': '',
                        'stdout': "1e-05::'$sw0'.\ng_1 :- '$sw0'.\nquery(g_1).\n"}
            if len(calls) == 1:
                return {'status': 'error', 'seconds': 0.1, 'error': 'lark.exceptions.UnexpectedCharacters: ...'}
            return {'status': 'ok', 'seconds': 1.0, 'stdout': '', 'stderr': '[RESULT] aspmc: p1: 0.00001\n'}
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'bb_c.pl')
            with open(path, 'w') as f:
                f.write('1e-05::a.\nquery(a).\n')
            with patch.object(bench, 'timed', timed):
                bench.run_aspmc(SimpleNamespace(dir=d, aspmc='python3', timeout=10, tddlog='tddlog'),
                                'bb_c', path, circuit=True)
            form, ground = (os.path.join(d, 'aspmc-bb_c', f) for f in ('bb_c.pl', 'bb_c.ground.pl'))
            self.assertEqual((calls[0][-1], calls[2][-1]), (form, ground))
            with open(form) as f:
                self.assertEqual(f.read(), '0.00001::a.\nquery(a).\n')
            with open(ground) as f:
                self.assertEqual(f.read(), '0.00001::p0.\np1 :- p0.\nquery(p1).\n')

    def test_a_program_with_evidence_is_run_conditioned(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'alarm.pl')
            with open(path, 'w') as f:
                f.write(ALARM)
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1,
                        'stderr': '[RESULT] aspmc: bench_e: 0.5\n[RESULT] aspmc: bench_qe_b: 0.25\n'
                                  '[RESULT] aspmc: bench_qe_alarm: 0.4\n'}) as run:
                result = bench.run_aspmc(args, 'alarm', path)
            ran = run.call_args[0][0][-1]
            self.assertEqual(ran, os.path.abspath(os.path.join(d, 'aspmc-alarm', 'alarm.pl')))
            with open(ran) as f:
                self.assertEqual(f.read(), bench.aspmc_conditioned(ALARM))
            self.assertEqual(result['answers'], {'b': 0.5, 'alarm': 0.8})

    def test_a_probability_in_exponent_notation_is_written_out(self):
        program = '1e-05::a; 0.99999::b.\n5.25e-07::c :- a(1e2).\nquery(c).\n'
        written = '0.00001::a; 0.99999::b.\n0.000000525::c :- a(1e2).\nquery(c).\n'
        self.assertEqual(bench.aspmc_numbers(program), written)
        self.assertEqual((float('0.00001'), float('0.000000525')), (1e-05, 5.25e-07))
        # a fraction, which aspmc's grammar does not read either; division elsewhere is left
        self.assertEqual(bench.aspmc_numbers('1/3::a; 2/3::b.\n0.5::c :- d(X), Y is X/2.\n'),
                         '0.3333333333333333::a; 0.6666666666666666::b.\n0.5::c :- d(X), Y is X/2.\n')
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'small.pl')
            with open(path, 'w') as f:
                f.write(program)
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1, 'stderr': '[RESULT] aspmc: c: 5.25e-12\n'}) as run:
                result = bench.run_aspmc(args, 'small', path)
            ran = run.call_args[0][0][-1]
            self.assertEqual(ran, os.path.abspath(os.path.join(d, 'aspmc-small', 'small.pl')))
            with open(ran) as f:
                self.assertEqual(f.read(), written)
            self.assertEqual(result['answers'], {'c': 5.25e-12})

    def test_an_empty_comment_is_dropped_for_aspmc(self):
        # aspmc's lexer stops on a `%` with nothing after it; other comments it reads
        program = "% pcfg: a port\n%\n% of the paper's model\n%  \nq('5%'). % 5%\nquery(q(X)).\n%"
        written = "% pcfg: a port\n\n% of the paper's model\n\nq('5%'). % 5%\nquery(q(X)).\n"
        self.assertEqual(bench.aspmc_comments(program), written)
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'port.pl')
            with open(path, 'w') as f:
                f.write(program)
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1, 'stderr': "[RESULT] aspmc: q(bench_q1): 1.0\n"}) as run:
                result = bench.run_aspmc(args, 'port', path)
            with open(run.call_args[0][0][-1]) as f:
                # the quoted atom, which aspmc's grammar does not read, under a plain name (aspmc_quoted)
                self.assertEqual(f.read(), written.replace("q('5%')", 'q(bench_q1)'))
            self.assertEqual(result['answers'], {"q('5%')": 1.0})
        # a program without one is read as it is
        self.assertEqual(bench.aspmc_comments('% a\na.\nquery(a).\n'), '% a\na.\nquery(a).\n')

    def test_is_is_written_as_clingos_equality(self):
        program = ('step(0).\nstep(T1) :- step(T), T < 9, T1 is T + 1.\n'
                   'd(T1) :- step(T), T1 is T+1, r(T).\nquery(this(is)).\n')
        written = ('step(0).\nstep(T1) :- step(T), T < 9, T1 = T + 1.\n'
                   'd(T1) :- step(T), T1 = T+1, r(T).\nquery(this(is)).\n')
        self.assertEqual(bench.aspmc_is(program), written)

    def test_each_variable_aspmc_cannot_read_gets_a_name_it_reads(self):
        program = ("% p(_, _X) in a comment\n0.5::b(1, 1).\nc :- o(W), b(W, _), q(_, _X, 'a_', \"_\", a_B).\n"
                   "d :- g(_X, A_b), b(A_b), 0'_ \\= A, r(V1, _).\n")
        written = ("% p(_, _X) in a comment\n0.5::b(1, 1).\nc :- o(W), b(W, VV1), q(VV2, VV3, 'a_', \"_\", a_B).\n"
                   "d :- g(VV3, VV4), b(VV4), 0'_ \\= A, r(V1, VV5).\n")
        self.assertEqual(bench.aspmc_variables(program), written)
        self.assertEqual(bench.aspmc_variables('c :- o(W), b(W, _), g(_, W).\n'),
                         'c :- o(W), b(W, V1), g(V2, W).\n')
        # names aspmc's grammar reads as variables, so that each grounding of
        # a probabilistic rule's body is a choice of its own
        self.assertEqual(bench.aspmc_variables('0.6::e :- s, f(_).\n0.3::d :- s, f(_F), f(_F).\n'),
                         '0.6::e :- s, f(V1).\n0.3::d :- s, f(V2), f(V2).\n')
        # run_aspmc writes the named form for aspmc
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'anon.pl')
            with open(path, 'w') as f:
                f.write('0.5::b(1, 1).\no(1).\nc :- o(W), b(W, _).\nquery(c).\n')
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1, 'stderr': '[RESULT] aspmc: c: 0.5\n'}) as run:
                result = bench.run_aspmc(args, 'anon', path)
            with open(run.call_args[0][0][-1]) as f:
                self.assertEqual(f.read(), '0.5::b(1, 1).\no(1).\nc :- o(W), b(W, V1).\nquery(c).\n')
            self.assertEqual(result['answers'], {'c': 0.5})

    def test_a_ground_query_aspmc_leaves_out_answers_0(self):
        program = '0.5::a.\nb :- a, c.\nquery(a).\nquery(b).\nquery(r(X)).\nquery(p(X)) :- a.\n'
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'gone.pl')
            with open(path, 'w') as f:
                f.write(program)
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1, 'stderr': '[RESULT] aspmc: a: 0.5\n'}):
                result = bench.run_aspmc(args, 'gone', path)
            self.assertEqual(result['answers'], {'a': 0.5, 'b': 0.0})

    def test_a_ground_query_stated_as_a_fact_that_aspmc_leaves_out_answers_1(self):
        # P(e) of a network where nothing is observed (pe_program)
        program = f'0.5::a.\n{bench.PE_QUERY} :- true.\nquery({bench.PE_QUERY}).\n'
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'pe.pl')
            with open(path, 'w') as f:
                f.write(program)
            args = SimpleNamespace(dir=d, aspmc='python3', timeout=1)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1,
                        'stderr': '[RESULT] aspmc: The overall weight of the program is 0.9999999999999998\n'}):
                result = bench.run_aspmc(args, 'pe', path)
            self.assertEqual(result['answers'], {bench.PE_QUERY: 1.0})

    def test_comments_and_atom_named_evidence_are_allowed(self):
        program = ('% evidence(obs, true).\n'
                   '/* evidence(obs). */\n'
                   '0.5::evidence. query(evidence).\n')
        args = SimpleNamespace(dir='bench', aspmc='python3', timeout=1)
        with patch('builtins.open', mock_open(read_data=program)), \
                patch.object(bench.os, 'makedirs'), \
                patch.object(bench, 'write_new') as wrote, \
                patch.object(bench, 'timed', return_value={
                    'status': 'ok', 'seconds': 0.1,
                    'stderr': '[RESULT] aspmc: evidence: 0.5',
                }) as run:
            result = bench.run_aspmc(args, 'coin', 'coin.pl')
            self.assertEqual(result['answers'], {'evidence': 0.5})
            run.assert_called_once()
        # the block comment, which aspmc's grammar does not read, left out
        wrote.assert_called_once_with(os.path.join('bench', 'aspmc-coin', 'coin.pl'),
                                      '% evidence(obs, true).\n \n0.5::evidence. query(evidence).\n')

    def test_a_block_comment_keeps_its_line_breaks(self):
        self.assertEqual(bench.aspmc_comments('/* a\nb */\na. /* c */ query(a).\n'), '\n\na.   query(a).\n')

    def test_comparisons_and_the_second_neck_are_written_in_clingos_notation(self):
        program = ('a <- b.\n'
                   'p(X) :- q(X), X \\= a, X \\== b, X == X, X =:= 1, X =\\= 2, X =< 3.\n'
                   'r(X, Y) :- q(X), q(Y), X @< Y, X @=< Y, X @> Y, X @>= Y.\n'
                   "s :- t('\\\\=', \"==\", 0'=, 0'<). % X \\= Y\n")
        self.assertEqual(bench.aspmc_operators(program),
                         'a :- b.\n'
                         'p(X) :- q(X), X != a, X != b, X = X, X = 1, X != 2, X <= 3.\n'
                         'r(X, Y) :- q(X), q(Y), X < Y, X <= Y, X > Y, X >= Y.\n'
                         "s :- t('\\\\=', \"==\", 0'=, 0'<). % X \\= Y\n")
        # what the call binds (the head's variables) and what a literal before it binds
        self.assertEqual(bench.aspmc_operators('path(X, Y) :- edge(X, Z), Y \\== Z, path(Z, Y).\n'),
                         'path(X, Y) :- edge(X, Z), Y != Z, path(Z, Y).\n')
        self.assertEqual(bench.aspmc_operators('p(X) :- q(X), \\+ (r(Y), Y \\= X).\n'),
                         'p(X) :- q(X), \\+ (r(Y), Y != X).\n')

    def test_a_comparison_before_its_variables_are_bound_is_unsupported(self):
        # ProbLog compares an unbound variable, where clingo binds it first
        for program, free in [('p :- X \\= a, q(X).\n', 'X'), ('p :- q(X), X \\== _.\n', '_'),
                              ('p :- \\+ (q(X), Y @< X), r(Y).\n', 'Y')]:
            with self.subTest(program=program), self.assertRaisesRegex(ValueError, f'a comparison of {free},'):
                bench.aspmc_operators(program)

    def test_bodies_are_written_as_conjunctions(self):
        program = ('any :- a ; b.\n'
                   'c :- d, (e ; f), g.\n'
                   'd2 :- not e, not(f), \\+ (e, f).\n'
                   'e2 :- true, a.\n'
                   'constraint1 :- \\+ (a ; b).\n'
                   'p(X) :- q(X), r(X, Z), \\+ (s(X, Y), t(Y)).\n'
                   '0.3::h :- a, b.\n'
                   'query(any).\n')
        self.assertEqual(bench.aspmc_bodies(program),
                         'any :- a.\nany :- b.\n'
                         'c :- d, e, g.\nc :- d, f, g.\n'
                         'd2 :- \\+e, \\+f, \\+bench_n1.\nbench_n1 :- e, f.\n'
                         'e2 :- a.\n'
                         'constraint1 :- \\+bench_n2.\nbench_n2 :- a.\nbench_n2 :- b.\n'
                         'p(X) :- q(X), r(X, Z), \\+bench_n3(X).\nbench_n3(X) :- q(X), r(X, Z), s(X, Y), t(Y).\n'
                         '0.3::h :- a, b.\n'
                         'query(any).\n')
        self.assertEqual(bench.aspmc_bodies('a.\nquery(a).\n'), 'a.\nquery(a).\n')
        # a negated comparison, which aspmc's grammar does not negate
        self.assertEqual(bench.aspmc_bodies('n :- s(X), \\+ (X = a).\n'),
                         'n :- s(X), \\+bench_n1(X).\nbench_n1(X) :- s(X), X = a.\n')
        # a built-in the program defines is its own predicate
        self.assertEqual(bench.aspmc_bodies('between(1, 2, 1).\na(X) :- between(1, 3, X).\n'),
                         'between(1, 2, 1).\na(X) :- between(1, 3, X).\n')

    def test_a_body_aspmcs_language_cannot_write_is_unsupported(self):
        for program, reason in [('0.3::a :- b ; c.\n', 'a probabilistic clause with a disjunctive body'),
                                ('a :- b -> c ; d.\n', 'an if-then-else'),
                                ('a(X) :- between(1, 3, X).\n', 'a call of between/3'),
                                ('a :- nl, b.\n', 'a call of nl/0'),
                                ('a :- \\+ (findall(X, b(X), L)).\n', 'a call of findall/3')]:
            with self.subTest(program=program), self.assertRaisesRegex(ValueError, reason):
                bench.aspmc_bodies(program)

    def test_a_negated_head_is_written_as_problog_reads_it(self):
        # the ProbLog tutorial's inhibition effects: e1 holds where a clause of it holds and no
        # negated one does
        program = '0.5::c1.\n0.3::e1 :- c1.\n0.2::\\+e1 :- c2.\ne1.\ne :- e1.\nquery(e1).\n'
        self.assertEqual(bench.aspmc_bodies(program),
                         '0.5::c1.\n0.3::bench_pos_e1 :- c1.\n0.2::bench_neg_e1 :- c2.\nbench_pos_e1.\n'
                         'e :- e1.\nquery(e1).\n\ne1 :- bench_pos_e1, \\+bench_neg_e1.\n')
        self.assertTrue(bench.aspmc_bodies('0.2::\\+p(X) :- q(X).\n0.5::p(X) :- q(X).\n').endswith(
            '\np(X0) :- bench_pos_p(X0), \\+bench_neg_p(X0).\n'))

    def test_quoted_atoms_get_plain_names(self):
        program = "p('A'). q('$choice_3', 'A'). r('c'). % 'B'\ns(\"str\").\nquery(p('A')).\n"
        out, names = bench.aspmc_quoted(program)
        self.assertEqual(out, 'p(bench_q1). q(bench_q2, bench_q1). r(c). % \'B\'\ns("str").\nquery(p(bench_q1)).\n')
        self.assertEqual(names, {'bench_q1': "'A'", 'bench_q2': "'$choice_3'"})
        self.assertEqual(bench.aspmc_unquoted('msw(l,bench_q1)', names), "msw(l,'A')")
        # the standard order of terms is that of the names only where they are the program's own
        with self.assertRaisesRegex(ValueError, 'the standard order of terms'):
            bench.aspmc_quoted("p('A'). r(X, Y) :- p(X), p(Y), X @< Y.\n")
        self.assertEqual(bench.aspmc_quoted("p('a'). r(X, Y) :- p(X), p(Y), X @< Y.\n"),
                         ('p(a). r(X, Y) :- p(X), p(Y), X @< Y.\n', {}))

    def test_a_probabilistic_body_that_compares_is_folded_into_an_atom(self):
        # aspmc 1.1.1's front end reads no comparison in a probabilistic clause's body
        program = ('0.999::p(V) :- m(M, I, V), I != 0.\n'
                   '0.5::t(X1) :- s(X), X1 = X - 1, X1 >= 0, w(V1).\n'
                   'd(V) :- m(M, I, V), I != 0.\n'
                   '0.5::u(X) :- s(f(X >= 1)).\n')
        self.assertEqual(bench.aspmc_folded(program),
                         '0.999::p(V) :- bench_b1(M,I,V).\nbench_b1(M,I,V) :- m(M, I, V), I != 0.\n'
                         '0.5::t(X1) :- bench_b2(X,X1,V1).\nbench_b2(X,X1,V1) :- s(X), X1 = X - 1, X1 >= 0, w(V1).\n'
                         'd(V) :- m(M, I, V), I != 0.\n'
                         '0.5::u(X) :- s(f(X >= 1)).\n')

    def test_a_term_compared_is_bound_through_a_fact(self):
        # aspmc's grammar compares variables and integer expressions only (TPC-H's q16)
        program = ('q16 :- part(P, Brand), Brand != brand45, s(P).\n'
                   'r(X) :- s(X), X = "a", X != 3, X != Y, t(Y).\n'
                   'w :- s(f(a)), u(X), X >= f(a), brand45 < X.\n'
                   'v :- s(X), X != g(X).\n')
        self.assertEqual(bench.aspmc_constants(program),
                         'q16 :- part(P, Brand), bench_k1(K1), Brand != K1, s(P).\n'
                         'r(X) :- s(X), bench_k2(K2), X = K2, X != 3, X != Y, t(Y).\n'
                         'w :- s(f(a)), u(X), bench_k3(K3), X >= K3, bench_k1(K1), K1 < X.\n'
                         'v :- s(X), X != g(X).\n'
                         '\nbench_k1(brand45).\nbench_k2("a").\nbench_k3(f(a)).\n')
        self.assertEqual(bench.aspmc_constants('p(X) :- q(X), X != 1.\n'), 'p(X) :- q(X), X != 1.\n')

    def test_an_anonymous_variable_of_a_query_is_one_name(self):
        program = ('0.5::col(1, green); 0.5::col(1, red).\n0.5::col(2, green).\n'
                   'query(col(1, _)).\nevidence(col(2, green)).\n')
        conditioned = bench.aspmc_conditioned(program)
        self.assertIn('bench_qe_col(1, _Q1) :- col(1, _Q1), col(2, green).', conditioned)
        named = bench.aspmc_variables(conditioned)
        self.assertIn('bench_qe_col(1, V1) :- col(1, V1), col(2, green).', named)
        self.assertIn('query(bench_qe_col(1, V1)).', named)

    def test_run_aspmc_writes_and_answers_the_rewritten_program(self):
        program = ("0.5::msw(l, 'A'); 0.5::msw(l, b).\n"
                   'p(X) :- msw(l, X), X \\= b.\n'
                   "0.9::r(X) :- msw(l, X), X \\= 'A'.\n"
                   "query(p('A')).\nquery(msw(l, 'A')).\nquery(r(b)).\n")
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'prism.pl')
            with open(path, 'w') as f:
                f.write(program)
            with patch.object(bench, 'timed', return_value={
                        'status': 'ok', 'seconds': 0.1,
                        'stderr': '[RESULT] aspmc: p(bench_q1): 0.5\n[RESULT] aspmc: msw(l,bench_q1): 0.5\n'
                                  '[RESULT] aspmc: r(b): 0.45\n'}) as run:
                result = bench.run_aspmc(SimpleNamespace(dir=d, aspmc='python3', timeout=1), 'prism', path)
            with open(run.call_args[0][0][-1]) as f:
                self.assertEqual(f.read(), '0.5::msw(l, bench_q1); 0.5::msw(l, b).\n'
                                           'p(X) :- msw(l, X), bench_k1(K1), X != K1.\n'
                                           '0.9::r(X) :- bench_b1(X,K2).\n'
                                           'bench_b1(X,K2) :- msw(l, X), bench_k2(K2), X != K2.\n'
                                           'query(p(bench_q1)).\nquery(msw(l, bench_q1)).\nquery(r(b)).\n'
                                           '\nbench_k1(b).\nbench_k2(bench_q1).\n')
        self.assertEqual(result['answers'], {"p('A')": 0.5, "msw(l,'A')": 0.5, 'r(b)': 0.45})
        # one the rewrites cannot write is not run
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'cut.pl')
            with open(path, 'w') as f:
                f.write('0.5::b.\na :- b -> c ; d.\nquery(a).\n')
            with patch.object(bench, 'timed') as run:
                result = bench.run_aspmc(SimpleNamespace(dir=d, aspmc='python3', timeout=1), 'cut', path)
            run.assert_not_called()
        self.assertEqual(result['status'], 'unsupported')
        self.assertIn('translator: an if-then-else', result['error'])


# Ace's marginals file for tests/networks/asia.net with smoke=yes and xray=yes, as
# Ace 3.0 wrote it (the times in the header are not measurements anyone
# should use).
ASIA_MARGINALS = """Network : asia.net
Network Read Time (ms) : 203.51681499999998
Number of Evidence Sets : 1
AC :
  Nodes : 69
  Edges : 89
  Total Initialization Time (ms) : 21.173189
  Total Inference Time (ms) : 1.5981459999999998
  Minimum Inference Time (ms) : 1.5981459999999998
  Maximum Inference Time (ms) : 1.5981459999999998
  Median Inference Time (ms) : 1.5981459999999998
  Mean Inference Time (ms) : 1.5981459999999998
  Standard Deviation Inference Time (ms) : 0.0


=============================================

e = {smoke=yes, xray=yes}

Pr(e) = 7.58524E-2

Pr(xray,e) = [1.0, 0.0]
Pr(smoke,e) = [1.0, 0.0]
Pr(bronc,e) = [0.6, 0.4]
Pr(dysp,e) = [0.7319368668624855, 0.26806313313751445]
Pr(lung,e) = [0.6459914254525895, 0.35400857454741047]
Pr(tub,e) = [0.06718310824706931, 0.9328168917529307]
Pr(either,e) = [0.7064562228749519, 0.2935437771250481]
Pr(asia,e) = [0.012184848468868487, 0.9878151515311315]
"""


def scratch_dir(prefix):
    root = scripts.parent / 'tmp'
    root.mkdir(exist_ok=True)
    return tempfile.TemporaryDirectory(dir=root, prefix=prefix)


class FetchTests(unittest.TestCase):
    """The networks the repository does not ship: data/fetch.txt names each,
    `fetch` downloads and checks them, and `gen` stops without them."""

    def test_the_manifest_names_the_bnlearn_networks_and_psouffles_programs_by_their_sha256(self):
        files = bench.fetched()
        # the tutorial: 36 pages; Ihler's collection: 115 + 213 networks with
        # their evidence, 21 + 3 + 18 without
        self.assertEqual(len(files), 22 + 15 + 38 + 36 + 2 * (115 + 213) + 21 + 3 + 18)
        for path, (sha, url) in files.items():
            self.assertRegex(sha, '^[0-9a-f]{64}$')
            self.assertTrue(url.startswith('https://'), url)
            self.assertIn(os.path.relpath(os.path.dirname(path), bench.ROOT),
                          ['data/bn', 'data/bn-large', 'data/psouffle/taint', 'data/psouffle/side_channel',
                           'data/problog-tutorial']
                          + [f'data/uai-ihler/{g}' for g in ['BN', 'pedigree', 'Grids', 'classics', 'promedas']])

    def fetch(self, d, rows):
        manifest = os.path.join(d, 'fetch.txt')
        with open(manifest, 'w') as f:
            f.write('# a comment\n' + ''.join(f'{sha} {path} {url}\n' for sha, path, url in rows))
        with patch.object(bench, 'FETCH', manifest), redirect_stdout(io.StringIO()):
            bench.fetch(SimpleNamespace(timeout=10))

    def test_a_file_is_written_once_its_sha256_checks(self):
        import gzip
        import hashlib
        with tempfile.TemporaryDirectory() as d:
            src = Path(d, 'src.net')
            src.write_bytes(b'net { }\n')
            sha = hashlib.sha256(src.read_bytes()).hexdigest()
            plain, zipped = os.path.join(d, 'out', 'a.net'), os.path.join(d, 'out', 'b.net.gz')
            self.fetch(d, [(sha, plain, src.as_uri()), (sha, zipped, src.as_uri())])
            self.assertEqual(Path(plain).read_bytes(), src.read_bytes())
            self.assertEqual(gzip.decompress(Path(zipped).read_bytes()), src.read_bytes())
            self.fetch(d, [(sha, plain, src.as_uri()), (sha, zipped, src.as_uri())])   # in place: checked
            Path(plain).write_bytes(b'changed\n')
            with self.assertRaisesRegex(SystemExit, 'a.net: its SHA-256 is not that of'):
                self.fetch(d, [(sha, plain, src.as_uri())])
            other = os.path.join(d, 'out', 'c.net')
            with self.assertRaisesRegex(SystemExit, 'not written'):
                self.fetch(d, [('0' * 64, other, src.as_uri())])
            self.assertFalse(os.path.exists(other))

    def test_gen_names_fetch_where_a_network_is_missing(self):
        with tempfile.TemporaryDirectory() as d:
            missing = os.path.join(d, 'x.net')
            spec = {'bn_x': ('bn', ['gen_bn.py', missing, '--query', 'leaves'])}
            with patch.object(bench, 'instances', return_value=spec), \
                    patch.object(bench, 'fetched', return_value={missing: ('0' * 64, 'https://example.org/x.net')}):
                with self.assertRaisesRegex(SystemExit, 'bench.py fetch downloads them'):
                    bench.gen(SimpleNamespace(dir=os.path.join(d, 'bench'), cnf=False))


class AceTests(unittest.TestCase):
    """The parts of the Ace baseline that run without Ace."""

    def test_inst_files_round_trip_names_xml_would_break(self):
        variables = {'Age': ['<5', '>=7.5'], 'R&D': ['"yes"', "no's"]}
        network = (list(variables), variables, {})
        evidence = [('Age', '>=7.5'), ('R&D', '"yes"')]
        with scratch_dir('ace-inst-') as d:
            path = Path(d) / 'names.inst'
            path.write_text(bench.gen_bn.write_inst(evidence))
            self.assertEqual(bench.gen_bn.read_inst(str(path), network), evidence)
            path.write_text(bench.gen_bn.write_inst([]))
            self.assertEqual(bench.gen_bn.read_inst(str(path), network), [])
            for bad, message in [([('Size', 'big')], 'no variable Size'), ([('Age', '5')], 'Age has no value 5'),
                                 ([('Age', '<5'), ('Age', '<5')], 'Age observed twice')]:
                path.write_text(bench.gen_bn.write_inst(bad))
                with self.assertRaisesRegex(SystemExit, message):
                    bench.gen_bn.read_inst(str(path), network)

    def test_the_marginals_file_is_read(self):
        log10_pe, rows, head = bench.parse_ace_marginals(ASIA_MARGINALS)
        self.assertEqual(log10_pe, bench.math.log10(0.0758524))
        self.assertEqual(rows['dysp'], [0.7319368668624855, 0.26806313313751445])
        self.assertEqual(rows['xray'], [1.0, 0.0])
        self.assertEqual(len(rows), 8)
        self.assertEqual((head['ac_nodes'], head['ac_edges']), (69, 89))
        self.assertIn('ace_inference_ms', head)
        with self.assertRaisesRegex(ValueError, 'one evidence set'):
            bench.parse_ace_marginals(ASIA_MARGINALS + ASIA_MARGINALS[ASIA_MARGINALS.index('====='):])
        zero = ASIA_MARGINALS.replace('Pr(e) = 7.58524E-2', 'Pr(e) = 0.0')
        self.assertEqual(bench.parse_ace_marginals(zero)[0], -bench.math.inf)

    def test_the_log_space_marginals_file_is_read(self):
        """-Xlog: the natural logarithms of P(e) and of the posteriors, as
        AceEvaluate writes them (promedus_31 of the UAI MAR set, and Ace's
        sign for minus infinity)."""
        text = ('=============================================\ne = {x195=1, x29=1}\n\n'
                'log_e(Pr(e)) = -4.14417E0\n\n'
                'log_e(Pr(x324,e)) = [-0.2555787453704079, -1.4892938811363532]\n'
                'log_e(Pr(x29,e)) = [-Infinity, 0.0]\n')
        log10_pe, rows, _ = bench.parse_ace_marginals(text)
        self.assertAlmostEqual(log10_pe, -1.7997903522477732, delta=3e-7)
        self.assertAlmostEqual(rows['x324'][0], 0.7744681483780709, places=14)
        self.assertEqual(rows['x29'], [0.0, 1.0])
        # P(e) far below a double's range, and zero
        self.assertAlmostEqual(bench.parse_ace_marginals(text.replace('-4.14417E0', '-1.054235E3'))[0],
                               -1054.235 / bench.math.log(10), places=9)
        self.assertEqual(bench.parse_ace_marginals(text.replace('-4.14417E0', '-∞'))[0], -bench.math.inf)

    def test_the_marginals_agree_with_the_clique_tree(self):
        """Ace's posteriors keyed as the program's queries, against
        bn_exact.py on the program gen_bn.py writes for the same evidence."""
        import bn_exact
        network = bench.gen_bn.parse(str(SOURCE / 'tests' / 'networks' / 'asia.net'))
        program = bench.gen_bn.to_problog(network, 'all', [('smoke', 'yes'), ('xray', 'yes')])
        rows, values, evidence, qs = bn_exact.parse(program)
        states, tables = bn_exact.factors(rows, values)
        exact, _, _ = bn_exact.marginals(states, tables, evidence)
        answers = bench.network_answers(bench.parse_ace_marginals(ASIA_MARGINALS)[1],
                                        bench.query_cells(network, [f'{x}({v})' for x, v in qs]))
        self.assertEqual(len(answers), 12)
        for (x, v) in qs:
            self.assertAlmostEqual(answers[f'{x}({v})'], float(exact[x][states[x].index(v)]), places=12)

    def test_variants_name_compile_options(self):
        self.assertEqual(bench.ace_flags('auto'), [])
        self.assertEqual(bench.ace_flags('forceC2d+cd05+dtClauseMinfill'), ['-forceC2d', '-cd05', '-dtClauseMinfill'])
        self.assertEqual(bench.ace_flags('forceC2d+dtHypergraph=3'), ['-forceC2d', '-dtHypergraph', '3'])
        self.assertEqual(bench.ace_flags('Xseed=1'), ['-Xseed', '1'])
        for harness in ['e=x.inst', 'Xnormalize', 'forceC2d+retainFiles', 'auto+cd05']:
            with self.assertRaisesRegex(SystemExit, 'not a compile method'):
                bench.ace_flags(harness)
        with self.assertRaisesRegex(SystemExit, "-noTabular is the readme's name; the code's is -forceC2d"):
            bench.ace_flags('noTabular+cd05')
        with self.assertRaisesRegex(SystemExit, '-dtHypergraph takes an argument'):
            bench.ace_flags('dtHypergraph')
        with self.assertRaisesRegex(SystemExit, '-cd05 takes none'):
            bench.ace_flags('cd05=1')

    def test_the_ace_suite_reads_the_unpacked_benchmarks(self):
        args = SimpleNamespace(suite='ace', only=[], families=['acefs'], ace_benchmarks='/nonexistent')
        with self.assertRaisesRegex(SystemExit, '--ace-benchmarks'):
            bench.instances(args)
        with scratch_dir('ace-suite-') as d:
            args.ace_benchmarks = d
            spec = bench.instances(args)
        self.assertEqual(list(spec)[:3], ['ace_fs_01', 'ace_fs_04', 'ace_fs_07'])
        self.assertEqual(spec['ace_fs_01'], ('acefs', ['gen_bn.py', f'{d}/2006-ijar/fs-01.net', '--query', 'all',
                                                       '--evidence-file', f'{d}/2006-ijar/fs-01.inst']))
        args.families = []
        with scratch_dir('ace-suite-') as d:
            args.ace_benchmarks = d
            spec = bench.instances(args)
        self.assertEqual(len(spec), 46)
        self.assertEqual(spec['ace_pathfinder'][1][-3:], ['leaves', '--evidence-file', f'{d}/2005-ijcai/pathfinder-0006.inst'])
        self.assertEqual(spec['ace_munin1'], ('acemunin', ['gen_bn.py', f'{d}/2005-ijcai/munin1.hugin', '--query', 'leaves',
                                                           '--evidence-file', f'{d}/2005-ijcai/munin1-0006.inst',
                                                           '--rescale']))
        self.assertEqual(sum('--rescale' in g for _, g in spec.values()), 7)

    def test_ace_options_without_ace_are_refused(self):
        def parsed(cmd, **given):
            fields = dict(cmd=cmd, suite='standard', ace_benchmarks=None, uai_benchmarks=None, biomine_benchmarks=None, ace='',
                          ace_variants=None, ace_heap=None, ace_java=None, sharpsat_td='', sharpsat_td_decot=None)
            return SimpleNamespace(**(fields | given))
        args = parsed('run', ace='ace')
        bench.ace_defaults(args)
        self.assertEqual((args.ace_variants, args.ace_heap, args.ace_java), (['auto'], '8g', 'java'))
        with self.assertRaisesRegex(SystemExit, '--ace-heap without --ace'):
            bench.ace_defaults(parsed('run', ace_heap='4g'))
        with self.assertRaisesRegex(SystemExit, '--ace-stack without --ace'):
            bench.ace_defaults(parsed('run', ace_stack='512m'))
        with self.assertRaisesRegex(SystemExit, '--ace-variants, --ace-java without --ace'):
            bench.ace_defaults(parsed('run', ace_variants=['auto'], ace_java='java'))
        with self.assertRaisesRegex(SystemExit, '--sharpsat-td-decot without --sharpsat-td'):
            bench.ace_defaults(parsed('run', sharpsat_td_decot=['1']))
        args = parsed('run', sharpsat_td='sharpSAT')
        bench.ace_defaults(args)
        self.assertEqual(args.sharpsat_td_decot, ['1'])
        with self.assertRaisesRegex(SystemExit, 'and the suite is large'):
            bench.ace_defaults(parsed('gen', suite='large', ace_benchmarks='b'))
        args = parsed('gen', suite='ace')
        bench.ace_defaults(args)
        self.assertTrue(args.ace_benchmarks.endswith('benchmarks_v1.0'))

    def fake_ace(self, calls, marginals=ASIA_MARGINALS):
        """`timed` for Ace's two processes: compile reports its route,
        evaluate writes the marginals next to the network copy."""
        def timed(cmd, timeout, cwd=None, env=None):
            net = Path(cwd) / cmd[-2 if 'edu.ucla.belief.ace.AceEvaluate' in cmd else -1]
            calls.append((cmd, net.read_bytes()))
            if 'edu.ucla.belief.ace.AceCompile' in cmd:
                return {'status': 'ok', 'seconds': 0.5, 'stdout': 'Method : tabularCompilation\n'
                        'Total Time (s) : 0.25\n', 'stderr': ''}
            Path(str(net) + '.marginals').write_text(marginals)
            return {'status': 'ok', 'seconds': 0.25, 'stdout': '', 'stderr': ''}
        return timed

    def test_a_run_compiles_with_the_evidence_and_records_both_phases(self):
        with scratch_dir('ace-run-') as d:
            ace = Path(d) / 'ace'
            ace.mkdir()
            for f in ['ace.jar', 'inflib.jar', 'jdom.jar', 'c2d_linux']:
                (ace / f).touch()
            selection = dict(suite='standard', only=['bn_asia_ev', 'grid6'], families=[], dir=d, cnf=False)
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**selection))
            self.assertIn('<inst id="dysp" value="no"/>', (Path(d) / 'bn_asia_ev.inst').read_text())
            self.assertFalse((Path(d) / 'grid6.inst').exists())
            # grid6 would reach Ace through its ground program (test_ground2net.py)
            args = SimpleNamespace(**{**selection, 'only': ['bn_asia_ev']}, strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                   exact=False, ace=str(ace), ace_variants=['auto', 'forceC2d', 'forceC2d+sbk05'],
                                   ace_heap='1g', merlin='', ganak='', pr=False,
                                   ace_java='java', out=str(Path(d) / 'results.jsonl'), timeout=10, no_skip=True)
            calls = []
            with patch.object(bench, 'timed', side_effect=self.fake_ace(calls)), redirect_stdout(io.StringIO()):
                bench.run(args)
            records = [bench.json.loads(l) for l in Path(args.out).read_text().splitlines()]
            self.assertEqual([(r['instance'], r['variant']) for r in records],
                             [('bn_asia_ev', 'auto'), ('bn_asia_ev', 'forceC2d'), ('bn_asia_ev', 'forceC2d+sbk05')])
            # Ace's sbk05 circuit compiled with evidence is wrong, so it is not run.
            self.assertEqual((records[2]['status'], len(calls)), ('error', 4))
            self.assertIn('sbk05', records[2]['error'])
            inst = str(Path(d, 'bn_asia_ev.inst').resolve())
            compile_auto, evaluate, compile_c2d, _ = [cmd for cmd, _ in calls]
            self.assertEqual(compile_auto[compile_auto.index('edu.ucla.belief.ace.AceCompile'):],
                             ['edu.ucla.belief.ace.AceCompile', '-Xnormalize', '-e', inst, 'asia.net'])
            self.assertIn('-forceC2d', compile_c2d)
            self.assertEqual(evaluate[-3:], ['edu.ucla.belief.ace.AceEvaluate', 'asia.net', inst])
            self.assertIn('-Xmx1g', evaluate)
            r = records[0]
            self.assertEqual((r['status'], r['seconds'], r['compile_s'], r['evaluate_s']), ('ok', 0.75, 0.5, 0.25))
            self.assertEqual((r['ace_method'], r['ace_compile_total_s'], r['ac_nodes'], r['ace_log10_pe']),
                             ('tabularCompilation', 0.25, 69, bench.math.log10(0.0758524)))
            self.assertEqual(r['answers'], {'xray(yes)': 1.0, 'xray(no)': 0.0})
            self.assertEqual([p.name for p in Path(d).iterdir() if p.name.startswith('ace-')], [])

    def test_a_c2d_circuit_is_not_evaluated_in_log_space(self):
        with scratch_dir('ace-xlog-') as d:
            ace = Path(d) / 'ace'
            ace.mkdir()
            for f in ['ace.jar', 'inflib.jar', 'jdom.jar', 'c2d_linux']:
                (ace / f).touch()
            selection = dict(suite='standard', only=['bn_asia_ev'], families=[], dir=d, cnf=False)
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**selection))
            args = SimpleNamespace(**selection, strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                   exact=False, ace=str(ace), ace_variants=['Xlog'], ace_heap='1g', merlin='',
                                   ganak='', pr=False, ace_java='java', out=str(Path(d) / 'results.jsonl'),
                                   timeout=10, no_skip=True)
            calls = []

            def timed(cmd, timeout, cwd=None, env=None):
                calls.append(cmd)
                return {'status': 'ok', 'seconds': 0.5, 'stdout': 'Method : c2dCompilation\n', 'stderr': ''}
            with patch.object(bench, 'timed', side_effect=timed), redirect_stdout(io.StringIO()):
                bench.run(args)
            [r] = [bench.json.loads(l) for l in Path(args.out).read_text().splitlines()]
            self.assertEqual((r['variant'], r['status'], r['ace_method']), ('Xlog', 'error', 'c2dCompilation'))
            self.assertIn('log space', r['error'])
            self.assertIn('-Xlog', calls[0])
            self.assertEqual([c for c in calls if 'edu.ucla.belief.ace.AceEvaluate' in c], [])

    def test_the_copy_drops_a_plus_in_a_number_outside_quotes_and_comments(self):
        self.assertEqual(bench.hugin_for_ace(b'data = (+0.25e+00 .75); % 1e+5\nnode "x+1"'),
                         b'data = (0.25e00 .75); % 1e+5\nnode "x+1"')
        net = 'net\n{\n}\nnode a\n{\n  states = ("1e+5" "b");\n}\npotential (a)\n{\n  data = (0.25e+00 7.5E-01);\n}\n'
        with scratch_dir('ace-copy-') as d:
            source = Path(d) / 'one.net'
            source.write_text(net)
            spec = {'one': ('bn', ['gen_bn.py', str(source), '--query', 'all'])}
            program = Path(d) / 'one.pl'
            program.write_text(bench.gen_bn.to_problog(bench.gen_bn.parse(str(source)), 'all'))
            (Path(d) / 'one.inst').write_text(bench.gen_bn.write_inst([]))
            marginals = 'e = {}\n\nPr(e) = 1E0\n\nPr(a,e) = [0.25, 0.75]\n'
            calls = []
            args = SimpleNamespace(dir=d, ace=d, ace_heap='1g', ace_java='java', timeout=10)
            with patch.object(bench, 'instances', return_value=spec), \
                    patch.object(bench, 'timed', side_effect=self.fake_ace(calls, marginals)):
                result = bench.run_ace(args, 'one', str(program), 'auto')
            self.assertEqual(result['answers'], {'a(v_1e_5)': 0.25, 'a(b)': 0.75})
            copy = calls[0][1].decode()
            self.assertIn('data = (0.25e00 7.5E-01);', copy)
            self.assertIn('"1e+5"', copy)
            self.assertNotIn('-e', calls[0][0])   # no evidence, no -e
            self.assertFalse(any(a.startswith('-Xss') for a in calls[0][0]))   # the JVM's stack by default
            calls.clear()
            args.ace_stack = '512m'
            with patch.object(bench, 'instances', return_value=spec), \
                    patch.object(bench, 'timed', side_effect=self.fake_ace(calls, marginals)):
                result = bench.run_ace(args, 'one', str(program), 'auto')
            self.assertIn('-Xss512m', calls[0][0])
            self.assertEqual(result['stats']['ace_stack'], '512m')

    def test_an_evidence_set_of_the_archive_is_the_one_inst_file(self):
        """An instance whose observations are an evidence set of Ace's
        archive (`--evidence-file`, the ace suite) has that one .inst file:
        gen writes no copy of it (NAME.inst), and Ace compiles and evaluates
        with the archive's file, which tddlog reads too."""
        asia = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        with scratch_dir('ace-archive-') as d:
            archive = Path(d) / 'asia-0006.inst'
            archive.write_text(bench.gen_bn.write_inst([('smoke', 'yes'), ('xray', 'yes')]))
            spec = {'ace_asia': ('acenet', ['gen_bn.py', asia, '--query', 'leaves', '--evidence-file', str(archive)])}
            out = Path(d) / 'bench'
            with patch.object(bench, 'instances', return_value=spec), redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(dir=str(out), cnf=False))
            self.assertTrue((out / 'ace_asia.pl').exists())
            self.assertFalse((out / 'ace_asia.inst').exists())
            self.assertEqual(bench.network_args(SimpleNamespace(dir=str(out), pr=False), 'ace_asia',
                                                spec['ace_asia'][1], False)[1][-1], str(archive))
            calls = []
            args = SimpleNamespace(dir=str(out), ace=d, ace_heap='1g', ace_java='java', timeout=10)
            with patch.object(bench, 'instances', return_value=spec), \
                    patch.object(bench, 'timed', side_effect=self.fake_ace(calls)):
                result = bench.run_ace(args, 'ace_asia', str(out / 'ace_asia.pl'), 'auto')
            self.assertEqual(result['status'], 'ok')
            (compile_, _), (evaluate, _) = calls
            self.assertEqual(compile_[compile_.index('-e') + 1], str(archive))
            self.assertEqual(evaluate[-1], str(archive))


MERLIN_MAR = """PR
-1.234567 (5.831e-02)
STATUS
true: Consistent evidence
MAR
3 2 0.261538 0.738462 2 0.461538 0.538462 3 0.507692 0.153846 0.338462
"""


class MerlinTests(unittest.TestCase):
    """The UAI suite and the parts of the Merlin baseline that run without
    Merlin."""

    def test_the_uai_suite_lists_the_instances_with_evidence_by_family_and_size(self):
        args = SimpleNamespace(suite='uai', only=[], families=[], uai_benchmarks='/nonexistent')
        with self.assertRaisesRegex(SystemExit, '--uai-benchmarks'):
            bench.instances(args)
        with scratch_dir('uai-suite-') as d:
            for name, size in [('Grids_15', 30), ('Grids_2', 20), ('CSP_1', 10), ('Lone_1', 10)]:
                (Path(d) / f'{name}.uai').write_text('x' * size)
                if name != 'Lone_1':
                    (Path(d) / f'{name}.uai.evid').write_text('0\n')
            args.uai_benchmarks = d
            spec = bench.instances(args)
        self.assertEqual(list(spec), ['uai_csp_1', 'uai_grids_2', 'uai_grids_15'])
        self.assertEqual(spec['uai_grids_2'], ('uaigrids', ['gen_bn.py', f'{d}/Grids_2.uai',
                                                            '--evid', f'{d}/Grids_2.uai.evid']))

    def test_the_marginals_file_is_read(self):
        self.assertEqual(bench.parse_merlin_marginals(MERLIN_MAR),
                         [[0.261538, 0.738462], [0.461538, 0.538462], [0.507692, 0.153846, 0.338462]])
        self.assertIsNone(bench.parse_merlin_marginals(
            MERLIN_MAR.replace('true: Consistent evidence', 'false: Inconsistent evidence or underflow')))

    def test_marginals_that_are_not_numbers_are_an_error(self):
        """Merlin's file where the partition function overflows: `PR inf`,
        a true status and `-nan` marginals."""
        overflow = MERLIN_MAR.replace('-1.234567 (5.831e-02)', 'inf (inf)').replace(
            '0.261538 0.738462', '-nan -nan')
        with self.assertRaisesRegex(ValueError, 'not numbers'):
            bench.parse_merlin_marginals(overflow)

    def test_a_run_answers_the_programs_queries(self):
        """tests/regress/markov.uai: Merlin's marginals to six decimals
        against the program's by enumeration."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            Path(cmd[cmd.index('-o') + 1] + '.MAR').write_text(MERLIN_MAR)
            return {'status': 'ok', 'seconds': 0.5, 'stdout': '[BTE] + induced width    : 1\n', 'stderr': ''}
        with scratch_dir('uai-run-') as d:
            args = SimpleNamespace(suite='uai', only=[], families=[], uai_benchmarks=str(regress), dir=d,
                                   merlin='merlin', timeout=60)
            with patch.object(bench, 'timed', timed):
                r = bench.run_merlin(args, 'uai_markov', str(regress / 'markov.pl'))
        self.assertEqual(calls[0][1:9], ['-f', f'{regress}/markov.uai', '-e', f'{regress}/markov.uai.evid',
                                         '-a', 'bte', '-t', 'MAR'])
        self.assertEqual((r['status'], r['stats']), ('ok', {'merlin_width': 1}))
        expected = {'x0(v_0)': 17 / 65, 'x0(v_1)': 48 / 65, 'x1(v_0)': 6 / 13, 'x1(v_1)': 7 / 13,
                    'x2(v_0)': 33 / 65, 'x2(v_1)': 2 / 13, 'x2(v_2)': 22 / 65}
        self.assertEqual(r['answers'].keys(), expected.keys())
        for q, p in expected.items():
            self.assertAlmostEqual(r['answers'][q], p, delta=5e-7)

    def test_a_network_tool_reads_the_query_variables_from_a_file(self):
        """The UAI drivers' query variables go in a file (`@FILE`): on the
        largest networks the list is longer than one argument may be."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append((cmd, Path(cmd[-1][1:]).read_text()))
            Path(cmd[-3]).write_text(MERLIN_MAR)
            return {'status': 'ok', 'seconds': 0.5, 'stdout': '', 'stderr': ''}
        with scratch_dir('uai-tool-') as d:
            args = SimpleNamespace(suite='uai', only=[], families=[], uai_benchmarks=str(regress), dir=d,
                                   libdai='libdai_uai', timeout=60)
            with patch.object(bench, 'timed', timed):
                r = bench.run_uai_system(args, 'uai_markov', str(regress / 'markov.pl'), 'libdai', 'jtree')
        [(cmd, listed)] = calls
        self.assertEqual(cmd[:5], ['libdai_uai', f'{regress}/markov.uai', f'{regress}/markov.uai.evid',
                                   cmd[3], 'MAR'])
        self.assertTrue(cmd[5].startswith('@'))
        self.assertEqual(listed, '0,1,2')
        self.assertEqual(r['status'], 'ok')
        self.assertAlmostEqual(r['answers']['x2(v_1)'], 2 / 13, delta=5e-7)

    def test_tensorinference_is_timed_warm(self):
        """TensorInference.jl's driver is timed as Alea.jl's:
        after a warm-up on a small model, the instance's first solve, Julia's
        compilation within it, and a second solve where the first took under
        a third of the limit, which bench.py passes it (BN_WARM_BELOW). The
        record's own time is the second solve's, else the first's less its
        compilation (none where Julia reports it as NaN)."""
        regress = SOURCE / 'tests' / 'regress'
        first = 'inference_seconds 12.5\ncompile_seconds 12.25\n'
        for tail, own, stats in (
                (first + 'warm_seconds 0.0625\nwarmup_seconds 31.5\n', 0.0625,
                 dict(inproc_seconds=12.5, jit_seconds=12.25, warm_seconds=0.0625, warmup_seconds=31.5)),
                (first + 'warmup_seconds 31.5\n', 0.25, dict(inproc_seconds=12.5, jit_seconds=12.25, warmup_seconds=31.5)),
                ('inference_seconds 0.25\n', 0.25, dict(inproc_seconds=0.25)),
                (first.replace('12.25', 'NaN') + 'warmup_seconds 31.5\n', 12.5, dict(inproc_seconds=12.5, warmup_seconds=31.5))):
            env = {}
            def timed(cmd, timeout, cwd=None, env=None, seen=env):
                seen.update(env)
                Path(cmd[-3]).write_text(MERLIN_MAR + tail)
                return {'status': 'ok', 'seconds': 40.0, 'stdout': '', 'stderr': ''}
            with scratch_dir('uai-ti-') as d:
                args = SimpleNamespace(suite='uai', only=[], families=[], uai_benchmarks=str(regress), dir=d,
                                       tensorinference='julia', timeout=60)
                with patch.object(bench, 'timed', timed):
                    r = bench.run_uai_system(args, 'uai_markov', str(regress / 'markov.pl'), 'tensorinference', 'treesa')
            self.assertEqual(env['BN_WARM_BELOW'], '20')
            self.assertEqual(r['stats'], stats)
            flat = {'system': 'tensorinference', 'seconds': r['seconds'], **r['stats']}
            self.assertEqual(bench.own_seconds(flat), own)

    def test_own_seconds(self):
        """The own time of a system timed so (bench.OWN_TIME): warm, else in the
        process less its compilation, else the process's."""
        self.assertEqual(set(bench.OWN_TIME), {'alea', 'alea-tddlog', 'roulette', 'roulette-tddlog', 'pluck',
                                                'tensorinference', 'webppl'})
        alea = {'system': 'alea', 'seconds': 20.0, 'inproc_seconds': 12.5, 'jit_seconds': 12.25}
        self.assertEqual(bench.own_seconds(alea), 0.25)
        self.assertEqual(bench.own_seconds(dict(alea, warm_seconds=0.0625)), 0.0625)
        self.assertEqual(bench.own_seconds({'system': 'roulette', 'seconds': 3.0, 'inproc_seconds': 1.5}), 1.5)
        self.assertEqual(bench.own_seconds({'system': 'pluck', 'seconds': 5.0, 'inproc_seconds': 1.5,
                                            'warm_seconds': 0.03}), 0.03)
        self.assertEqual(bench.own_seconds({'system': 'pluck', 'seconds': 5.0, 'inproc_seconds': 1.5}), 1.5)
        self.assertEqual(bench.own_seconds({'system': 'tensorinference', 'seconds': 30.0}), 30.0)
        # WebPPL's: its compile and run less its header's compile, the load of its library
        self.assertEqual(bench.own_seconds({'system': 'webppl', 'seconds': 4.0, 'inproc_seconds': 1.5,
                                            'header_seconds': 0.25}), 1.25)

    def test_the_partition_function_file_is_read(self):
        """Merlin's PR file for tests/regress/markov.uai without evidence,
        Z = 325: the natural logarithm to six decimals. A false status where exp
        underflows keeps the logarithm; `-inf` is zero; `inf` is an
        error."""
        pr = 'PR\n5.783825 (3.250000e+02)\nSTATUS\ntrue: Consistent evidence\nMAR\n3 2 1.000000 0.000000\n'
        self.assertAlmostEqual(bench.parse_merlin_pr(pr), bench.math.log10(325), delta=3e-7)
        under = pr.replace('5.783825 (3.250000e+02)', '-1234.567890 (0.000000e+00)').replace(
            'true: Consistent evidence', 'false: Inconsistent evidence or underflow')
        self.assertAlmostEqual(bench.parse_merlin_pr(under), -1234.56789 / bench.math.log(10), places=12)
        self.assertEqual(bench.parse_merlin_pr(under.replace('-1234.567890', '-inf')), -bench.math.inf)
        with self.assertRaisesRegex(ValueError, 'overflows'):
            bench.parse_merlin_pr(pr.replace('5.783825 (3.250000e+02)', 'inf (inf)'))

    def test_a_pr_run_records_log10_z_of_each_system(self):
        """`--pr` on the uai suite's tests/regress/markov.uai, its program
        markov.pl: tddlog's Z line of the network it reads, the clique tree's
        P(e) line (65/192), run for real on the program, plus the program's
        offset (log10 960), and Merlin's PR file, each log10 325."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []
        real = bench.timed
        tddlog_out = f'Z:\t3.25e2\tlog10 {bench.math.log10(325)!r}\n'

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            if cmd[0] == 'tddlog':
                return {'status': 'ok', 'seconds': 0.5, 'stdout': tddlog_out, 'stderr': ''}
            if cmd[0] == sys.executable:   # bn_exact.py
                return real(cmd, timeout, cwd, env)
            self.assertEqual(cmd[0], '/bin/true')
            task = cmd[cmd.index('-t') + 1]
            Path(cmd[cmd.index('-o') + 1] + '.' + task).write_text(
                MERLIN_MAR if task == 'MAR' else 'PR\n5.783825 (3.250000e+02)\nSTATUS\ntrue: Consistent evidence\n')
            return {'status': 'ok', 'seconds': 0.5, 'stdout': '', 'stderr': ''}
        with scratch_dir('uai-pr-') as d:
            (Path(d) / 'uai_markov.pl').write_text((regress / 'markov.pl').read_text())
            args = SimpleNamespace(suite='uai', only=['uai_markov'], families=[], uai_benchmarks=str(regress), dir=d,
                                   strategies=['auto', 'closed+evidence-probability'], kbs=[], pita='', aspmc='',
                                   scallop='', dice='', exact=True, ace='', merlin='/bin/true', ganak='', pr=True,
                                   tddlog='tddlog', out=str(Path(d) / 'results.jsonl'), timeout=60, no_skip=True)
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()):
                bench.run(args)
            records = {(r['system'], r['variant']): r
                       for r in map(bench.json.loads, Path(args.out).read_text().splitlines())}
        self.assertEqual(list(records), [('tddlog', 'auto+evidence-probability+no-queries'),
                                         ('tddlog', 'closed+evidence-probability+no-queries'), ('exact', 'cliquetree'),
                                         ('merlin', 'bte'), ('merlin', 'bte-pr')])
        # tddlog reads the network and its observations, the clique tree the program
        self.assertEqual(calls[0][-7:], ['--vtree', 'auto', '--evidence-probability', '--no-queries',
                                         str(regress / 'markov.uai'), '--evidence', str(regress / 'markov.uai.evid')])
        self.assertEqual(calls[2][-2:], [str(Path(d) / 'uai_markov.pl'), '--evidence-probability'])
        self.assertEqual(records[('tddlog', 'auto+evidence-probability+no-queries')]['network'], 'markov.uai')
        self.assertNotIn('log10_pe', records[('tddlog', 'auto+evidence-probability+no-queries')])
        for key, delta in [(('tddlog', 'auto+evidence-probability+no-queries'), 1e-12),
                           (('exact', 'cliquetree'), 1e-12),
                           (('merlin', 'bte-pr'), 3e-7)]:
            self.assertEqual(records[key]['status'], 'ok', key)
            self.assertAlmostEqual(records[key]['log10_z'], bench.math.log10(325), delta=delta, msg=key)
        self.assertAlmostEqual(records[('exact', 'cliquetree')]['answers']['x0(v_0)'], 17 / 65, places=14)
        self.assertNotIn('answers', records[('merlin', 'bte-pr')])
        self.assertNotIn('answers', records[('tddlog', 'auto+evidence-probability+no-queries')])
        self.assertNotIn('log10_z', records[('merlin', 'bte')])

    def test_a_pr_variant_asks_for_the_evidence_probability_alone(self):
        for given, run in [('auto', 'auto+evidence-probability+no-queries'),
                           ('closed+evidence-probability', 'closed+evidence-probability+no-queries'),
                           ('td+no-queries+evidence-probability', 'td+no-queries+evidence-probability')]:
            self.assertEqual(bench.pr_variant(given), run)
            self.assertEqual(bench.tddlog_flags(run)[2:], ['--' + o for o in run.split('+')[1:]])

    def test_a_program_without_its_offset_line_has_no_partition_function(self):
        with scratch_dir('uai-pr-') as d:
            path = Path(d) / 'p.pl'
            path.write_text('0.5::a.\nevidence(a).\n')
            with self.assertRaisesRegex(ValueError, 'states no log10 Z'):
                bench.partition_function('P(e):\t5.000000e-1\tlog10 -0.301030\n', str(path))
            path.write_text(bench.gen_bn.offset_line(1.0) + '0.5::a.\nevidence(a).\n')
            self.assertEqual(bench.partition_function('P(e):\t1.0e-1\tlog10 -1.0\n', str(path)), (-1.0, 0.0))
            with self.assertRaisesRegex(ValueError, 'no P\\(e\\) line'):
                bench.partition_function('a:\t0.5\n', str(path))
        self.assertEqual(bench.parse_answers('a:\t0.5\nP(e):\t1.0e-1\tlog10 -1.0\n'), {'a': 0.5})
        self.assertEqual(bench.parse_evidence_probability('evidence is inconsistent\nP(e):\t0.0e0\tlog10 -inf\n'),
                         -bench.math.inf)

    def test_a_bayesian_networks_program_states_that_z_is_p_e(self):
        for kind, offset in [('BAYES', 0.0), ('MARKOV', None)]:
            with scratch_dir('uai-kind-') as d:
                net = os.path.join(d, 'n.uai')
                with open(net, 'w') as f:
                    f.write(f'{kind}\n1\n2\n1\n1 0\n\n2\n 0.4 0.6\n')

                def write(args, name, argv):
                    with open(os.path.join(d, name + '.pl'), 'w') as f:
                        f.write('0.4::v_0.\n')
                with patch.object(bench, 'write_program', write), \
                        patch.object(bench, 'program_args', return_value=[]), \
                        patch.object(bench.subprocess, 'run'):
                    bench.gen_network(SimpleNamespace(dir=d, pr=False, cnf=False), 'n', ['gen_bn.py', net])
                with open(os.path.join(d, 'n.pl')) as f:
                    self.assertEqual(bench.gen_bn.read_offset(f.read()), offset)

    def test_ganak_counts_the_weighted_cnf_gen_writes(self):
        """`gen` writes each UAI network's weighted CNF with its evidence
        beside the program (`gen_bn.py --cnf`), and `run --pr --ganak`
        counts it: tests/regress/markov.uai with markov.uai.evid, Z = 165.
        Ganak answers Z alone, so it needs --pr."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            return {'status': 'ok', 'seconds': 0.5, 'stderr': '',
                    'stdout': 's SATISFIABLE\nc s type wmc\nc s exact arb frac 165\n'}
        with scratch_dir('uai-ganak-') as d:
            args = SimpleNamespace(suite='uai', only=['uai_markov'], families=[], uai_benchmarks=str(regress), dir=d, cnf=True)
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
            network = bench.gen_bn.parse(str(regress / 'markov.uai'))[0]
            evidence = bench.gen_bn.read_evid(str(regress / 'markov.uai.evid'), network)
            self.assertEqual((Path(d) / 'uai_markov.cnf').read_text(),
                             bench.gen_bn.uai_cnf(str(regress / 'markov.uai'), evidence)[0])
            args = SimpleNamespace(**vars(args), strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                   exact=False, ace='', merlin='', ganak='/bin/true', pr=True,
                                   out=str(Path(d) / 'results.jsonl'), timeout=60, no_skip=True)
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()):
                bench.run(args)
            records = [bench.json.loads(line) for line in Path(args.out).read_text().splitlines()]
            args.pr = False
            with self.assertRaisesRegex(SystemExit, '--ganak answers the partition function alone'):
                bench.run(args)
        self.assertEqual(calls, [['/bin/true', '--mode', '1', '--prob', '0', f'{d}/uai_markov.cnf']])
        self.assertEqual([(r['system'], r['variant'], r['status']) for r in records], [('ganak', 'wmc', 'ok')])
        self.assertAlmostEqual(records[0]['log10_z'], bench.math.log10(165), places=14)
        self.assertTrue(bench.has_cnf(('uaimarkov', ['gen_bn.py', 'markov.uai.gz', '--evid', 'markov.uai.evid'])))
        self.assertFalse(bench.has_cnf(bench.SPEC['bn_asia']))
        self.assertFalse(bench.has_cnf(bench.SPEC['grid6']))

    def test_a_uai_cnf_is_written_on_request(self):
        """gen writes a UAI network's CNF only with --cnf, which a suite
        without UAI networks refuses, and a counter refuses a network
        whose CNF was not written."""
        regress = SOURCE / 'tests' / 'regress'
        with scratch_dir('uai-nocnf-') as d:
            args = SimpleNamespace(suite='uai', only=['uai_markov'], families=[], uai_benchmarks=str(regress), dir=d, cnf=False)
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
            self.assertTrue((Path(d) / 'uai_markov.pl').exists())
            self.assertFalse((Path(d) / 'uai_markov.cnf').exists())
            args = SimpleNamespace(**vars(args), strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                   exact=False, ace='', merlin='', ganak='/bin/true', pr=True,
                                   out=str(Path(d) / 'results.jsonl'), timeout=60, no_skip=True)
            with self.assertRaisesRegex(SystemExit, 'no CNF for uai_markov .*bench.py gen --cnf'):
                bench.run(args)
            with self.assertRaisesRegex(SystemExit, '--cnf writes the weighted CNFs of UAI networks'):
                bench.gen(SimpleNamespace(suite='standard', only=['grid6'], families=[], dir=d, cnf=True))

    def test_gen_pr_writes_the_programs_for_the_partition_function(self):
        """`gen --pr` on the uai suite writes the programs of `gen_bn.py
        --query none`, without queries or .inst files: tests/regress/potts.pl
        for potts.uai, its colour symmetry broken. `run --pr` takes Z from
        them, log10(225/2) by the clique tree run for real. Another suite
        refuses --pr."""
        regress = SOURCE / 'tests' / 'regress'
        real = bench.timed

        def timed(cmd, timeout, cwd=None, env=None):
            self.assertEqual(cmd[0], sys.executable)   # bn_exact.py
            return real(cmd, timeout, cwd, env)
        with scratch_dir('uai-genpr-') as d:
            args = SimpleNamespace(suite='uai', only=['uai_potts', 'uai_markov'], families=[],
                                   uai_benchmarks=str(regress), dir=d, cnf=False, pr=True)
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
            written = (regress / 'potts.pl').read_text()
            self.assertEqual((Path(d) / 'uai_potts.pl').read_text(),
                             ''.join(line for line in written.splitlines(keepends=True)
                                     if not line.startswith('% ') or line.startswith('% log10 Z')))
            self.assertEqual(bench.queries(str(Path(d) / 'uai_markov.pl')), [])
            self.assertEqual(sorted(p.name for p in Path(d).iterdir()), ['uai_markov.pl', 'uai_potts.pl'])
            run = SimpleNamespace(suite='uai', only=['uai_potts'], families=[], uai_benchmarks=str(regress), dir=d,
                                  strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='', exact=True, ace='',
                                  merlin='', ganak='', pr=True, out=str(Path(d) / 'results.jsonl'), timeout=60,
                                  no_skip=True)
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()):
                bench.run(run)
            records = [bench.json.loads(line) for line in Path(run.out).read_text().splitlines()]
            with self.assertRaisesRegex(SystemExit, '--pr writes the uai suite'):
                bench.gen(SimpleNamespace(suite='standard', only=['grid6'], families=[], dir=d, cnf=False, pr=True))
        self.assertEqual([(r['system'], r['status']) for r in records], [('exact', 'ok')])
        self.assertAlmostEqual(records[0]['log10_z'], bench.math.log10(225 / 2), places=12)
        self.assertEqual(records[0]['answers'], {})   # the program asks nothing

    def test_tddlog_is_given_the_network_and_writes_its_program(self):
        """tddlog reads a gen_bn.py instance's network itself, with its
        queries and observations (`network_args`): an Ace .inst file, a
        UAI .evid file, sampled observations through the .inst file gen
        writes, --rescale, and under --pr no queries. The program the
        systems that read ProbLog programs are given is `tddlog
        --program`'s (the built tddlog), the text gen_bn.py writes, of a
        gzipped network's copy that gen decompresses once."""
        asia = str(SOURCE / 'tests' / 'networks' / 'asia.net')
        regress = SOURCE / 'tests' / 'regress'
        uai, evid = str(regress / 'markov.uai'), str(regress / 'markov.uai.evid')
        plain, pr = SimpleNamespace(dir='/b', pr=False), SimpleNamespace(dir='/b', pr=True)
        for args, gargs, no_queries, want in [
                (plain, [asia, '--query', 'leaves', '--evidence', 'half'], False,
                 [asia, '--query', 'leaves', '--evidence', '/b/n.inst']),
                (plain, [asia, '--query', 'none', '--evidence-file', 'e.inst', '--rescale'], False,
                 [asia, '--no-queries', '--evidence', 'e.inst', '--rescale']),
                (pr, [uai, '--evid', evid], True, [uai, '--evidence', evid]),
                (plain, [uai + '.gz'], False, ['/b/networks/markov.uai', '--query', 'all'])]:
            with self.subTest(gargs=gargs):
                self.assertEqual(bench.network_args(args, 'n', ['gen_bn.py'] + gargs, no_queries),
                                 (want, [want[0]] + [want[i + 1] for i, x in enumerate(want) if x == '--evidence']))
        with self.assertRaisesRegex(SystemExit, 'a plain gen writes, and --pr writes none'):
            bench.network_args(pr, 'n', ['gen_bn.py', asia, '--evidence', '2'], True)
        with self.assertRaisesRegex(SystemExit, '--format dice'):
            bench.network_args(plain, 'n', ['gen_bn.py', asia, '--format', 'dice'], False)
        with scratch_dir('netread-') as d:
            with open(asia, 'rb') as f, bench.gzip.open(Path(d) / 'asia.net.gz', 'wb') as g:
                g.write(f.read())
            spec = {'bn_a': ('bn', ['gen_bn.py', asia, '--query', 'leaves', '--evidence', 'half']),
                    'bn_z': ('bn', ['gen_bn.py', str(Path(d) / 'asia.net.gz'), '--query', 'all']),
                    'uai_m': ('uai', ['gen_bn.py', uai, '--evid', evid])}
            out = Path(d) / 'bench'
            with patch.object(bench, 'instances', return_value=spec), redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(dir=str(out), cnf=False))
            for name, (_, gargs) in spec.items():
                with self.subTest(name=name):
                    gen_bn = str(bench.source_tree.script('gen_bn.py'))
                    written = bench.subprocess.run([sys.executable, gen_bn, *gargs[1:]], capture_output=True,
                                                   text=True, check=True).stdout
                    self.assertEqual((out / f'{name}.pl').read_text(), written)
            self.assertEqual((out / 'networks' / 'asia.net').read_bytes(), Path(asia).read_bytes())
            self.assertTrue((out / 'bn_a.inst').exists() and (out / 'bn_a.uai').exists())
            with self.assertRaisesRegex(SystemExit, 'not an executable'), \
                    patch.object(bench, 'instances', return_value=spec):
                bench.gen(SimpleNamespace(dir=str(out), cnf=False, tddlog=str(Path(d) / 'none')))
        z = f'Z:\t3.25e2\tlog10 {bench.math.log10(325)!r}\n'
        self.assertEqual(bench.network_partition_function('x0(v_0):\t0.5\n' + z), bench.math.log10(325))
        self.assertEqual(bench.network_partition_function('P(e):\t5.0e-1\tlog10 -0.25\n'), -0.25)
        self.assertEqual(bench.parse_answers('x0(v_0):\t0.5\n' + z), {'x0(v_0)': 0.5})
        with self.assertRaisesRegex(ValueError, 'no Z or P\\(e\\) line'):
            bench.network_partition_function('a:\t0.5\n')

    def test_sharpsat_td_counts_the_same_cnf(self):
        """`run --pr --sharpsat-td` on tests/regress/markov.uai: one run per
        FlowCutter budget, in the binary's directory where FlowCutter is,
        with the cache bounded by half the address-space limit less 500 MB,
        and log10 Z from its log10 line. Refused without --pr, without
        FlowCutter beside the binary, without an address-space limit, and
        for a budget sharpSAT-TD does not take or that fills the timeout."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append((cmd, cwd))
            self.assertTrue(Path(cmd[cmd.index('-tmpdir') + 1]).is_dir())
            return {'status': 'ok', 'seconds': 0.5, 'stderr': '',
                    'stdout': 's SATISFIABLE\nc s type wmc\nc s log10-estimate 2.217483944213906\n'
                              'c s exact arb float 164.9999999999999\n'}
        import resource
        with scratch_dir('uai-sharpsat-') as d:
            bindir = Path(d) / 'bin'
            bindir.mkdir()
            for f in ['sharpSAT', 'flow_cutter_pace17']:
                (bindir / f).write_text('#!/bin/sh\n')
                (bindir / f).chmod(0o755)
            args = SimpleNamespace(suite='uai', only=['uai_markov'], families=[], uai_benchmarks=str(regress), dir=d, cnf=True)
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
            args = SimpleNamespace(**vars(args), strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                   exact=False, ace='', merlin='', ganak='', sharpsat_td=str(bindir / 'sharpSAT'),
                                   sharpsat_td_decot=['1', '2.5'], pr=True, out=str(Path(d) / 'results.jsonl'),
                                   timeout=60, no_skip=True)
            with patch.object(bench, 'timed', timed), redirect_stdout(io.StringIO()), \
                    patch.object(resource, 'getrlimit', return_value=(16_000_000_000, resource.RLIM_INFINITY)):
                bench.run(args)
            records = [bench.json.loads(line) for line in Path(args.out).read_text().splitlines()]
            self.assertEqual(list(Path(d).glob('sharpsat-td-*')), [])   # the scratch directories
            for given, error in [(dict(pr=False), 'answers the partition function alone'),
                                 (dict(sharpsat_td_decot=['0']), 'between 0.0001 and 10000'),
                                 (dict(sharpsat_td_decot=['1s']), 'between 0.0001 and 10000'),
                                 (dict(sharpsat_td_decot=['60']), 'the timeout is 60 s')]:
                with self.subTest(error=error), self.assertRaisesRegex(SystemExit, error), \
                        patch.object(resource, 'getrlimit', return_value=(16_000_000_000, resource.RLIM_INFINITY)):
                    bench.run(SimpleNamespace(**{**vars(args), **given}))
            with self.assertRaisesRegex(SystemExit, 'ulimit -v'), \
                    patch.object(resource, 'getrlimit', return_value=(resource.RLIM_INFINITY,) * 2):
                bench.run(args)
            (bindir / 'flow_cutter_pace17').unlink()
            with self.assertRaisesRegex(SystemExit, 'flow_cutter_pace17'):
                bench.run(args)
        self.assertEqual([c[0][:6] for c in calls], [[str(bindir / 'sharpSAT'), '-WE', '-decot', b, '-decow', '100']
                                                     for b in ['1', '2.5']])
        self.assertEqual([c[0][-3:] for c in calls], [['-cs', '7500', f'{d}/uai_markov.cnf']] * 2)
        self.assertEqual({c[1] for c in calls}, {str(bindir)})
        self.assertEqual([(r['system'], r['variant'], r['status']) for r in records],
                         [('sharpsat-td', 'decot1', 'ok'), ('sharpsat-td', 'decot2.5', 'ok')])
        self.assertAlmostEqual(records[0]['log10_z'], bench.math.log10(165), places=14)
        self.assertEqual(bench.parse_sharpsat_td('s UNSATISFIABLE\nc s log10-estimate -inf\n'), -bench.math.inf)
        with self.assertRaisesRegex(ValueError, 'log10-estimate'):
            bench.parse_sharpsat_td('s SATISFIABLE\nc s type wmc\n')

    def test_pr_outside_the_uai_suite_is_refused(self):
        args = SimpleNamespace(suite='standard', only=['grid6'], families=[], strategies=['auto'], kbs=[], pita='',
                               aspmc='', scallop='', dice='', exact=False, ace='', merlin='', ganak='', pr=True,
                               out='/dev/null', dir='bench')
        with self.assertRaisesRegex(SystemExit, '--pr compares the partition functions of the uai'):
            bench.run(args)

    def test_the_exact_count_is_read(self):
        out = 's SATISFIABLE\nc s type wmc\nc s log10-estimate -1.16e-01\nc s exact arb frac 153/200\n'
        self.assertAlmostEqual(bench.parse_ganak(out), bench.math.log10(153 / 200), places=15)
        self.assertEqual(bench.parse_ganak(out.replace('153/200', '7')), bench.math.log10(7))
        self.assertEqual(bench.parse_ganak('s UNSATISFIABLE\n'), -bench.math.inf)
        self.assertEqual(bench.parse_ganak(out.replace('153/200', '0')), -bench.math.inf)
        # Fractions longer than the 4 300 digits Python converts to an int.
        long = out.replace('153/200', '3' + '0' * 5000 + '/1' + '0' * 4999)
        self.assertAlmostEqual(bench.parse_ganak(long), bench.math.log10(30), places=14)
        with self.assertRaisesRegex(ValueError, 'exact arb frac'):
            bench.parse_ganak('s SATISFIABLE\nc s log10-estimate -1.16e-01\n')

    def test_ganak_outside_the_uai_suite_is_refused(self):
        args = SimpleNamespace(suite='standard', only=['grid6'], families=[], strategies=['auto'], kbs=[], pita='',
                               aspmc='', scallop='', dice='', exact=False, ace='', merlin='', ganak='/bin/true',
                               pr=False, out='/dev/null', dir='bench')
        with self.assertRaisesRegex(SystemExit, '--ganak counts the weighted CNFs of the uai'):
            bench.run(args)

    def test_a_network_system_takes_a_program_but_not_under_pr(self):
        # a program that is not a network reaches Merlin through its ground
        # program (`ground_format`), which states no log10 Z
        spec = {'grid6': ('grid', ['gen_grid.py', '6']), 'bn_asia': ('bn', ['gen_bn.py', 'asia.net'])}
        bench.takes_networks(SimpleNamespace(pr=False), spec, ['grid6'], 'merlin')
        bench.takes_networks(SimpleNamespace(pr=True), spec, ['bn_asia', 'grid6'], 'merlin')
        with self.assertRaisesRegex(SystemExit, '--merlin answers log10 Z of the networks gen_bn.py writes under --pr'):
            bench.takes_networks(SimpleNamespace(pr=True), spec, ['grid6'], 'merlin')
        with self.assertRaisesRegex(SystemExit, 'no instance is selected'):
            bench.takes_networks(SimpleNamespace(pr=False), spec, [], 'merlin')


class AnswersCheckTests(unittest.TestCase):
    def test_a_query_problog_prints_with_its_variables_at_0_checks_its_instances(self):
        """problogtut_basic_08_rule_probs_4: ProbLog derives no instance of
        different_color(X,Y) and prints the query with its variables at 0;
        tddlog grounds its instances over the domain, each of them 0."""
        ref = {'system': 'exact', 'variant': 'problog1', 'answers': {'red(a)': 0.7, 'different_color(X3,X4)': 0.0}}
        rec = {'system': 'tddlog', 'variant': 'auto', 'answers': {
            'red(a)': 0.7, 'different_color(a,a)': 0.0, 'different_color(a,b)': 0.0, 'other(a,b)': 0.4}}
        self.assertEqual(bench.answers_check(rec, ref), (True, 0.0, 2))
        rec['answers']['different_color(a,b)'] = 0.1
        right, worst, n = bench.answers_check(rec, ref)
        self.assertEqual((right, n), (False, 2))
        self.assertAlmostEqual(worst, 0.1)
        # a query at more than 0 is compared as it is written
        ref['answers']['different_color(X3,X4)'] = 0.5
        self.assertEqual(bench.answers_check(rec, ref)[2], 1)

    def test_evidence_found_inconsistent_is_the_reference_where_none_answers(self):
        # Circulette's ct_ALU_Controller asks no query, and no fault makes it fail
        recs = [{'system': 'tddlog', 'variant': 'auto', 'status': 'ok', 'answers': 'inconsistent'},
                {'system': 'roulette', 'variant': 'rsdd', 'status': 'ok', 'answers': 'inconsistent'},
                {'system': 'problog', 'variant': 'sdd', 'status': 'ok', 'answers': 'inconsistent'},
                {'system': 'pita', 'variant': 'bdd', 'status': 'ok', 'answers': {}}]
        refs = bench.reference_records(recs)
        self.assertEqual([r['system'] for r in refs], ['problog', 'tddlog'])
        self.assertEqual(bench.answers_check(recs[3], refs[0])[0], False)
        self.assertEqual(bench.answers_check(dict(recs[3], answers='inconsistent'), refs[0]), (True, 0.0, 1))
        # a record that answers a query is the reference where there is one
        recs.append({'system': 'merlin', 'variant': 'bte', 'status': 'ok', 'answers': {'q': 0.5}})
        self.assertEqual([r['system'] for r in bench.reference_records(recs)], ['merlin'])

    def test_the_instances_of_a_query_with_variables(self):
        # a repeated variable binds one value; a variable below an argument is not read
        self.assertEqual(bench.instances_of('p(X,X,b)', ['p(a,a,b)', 'p(a,c,b)', 'p(a,a,c)', 'q(a,a,b)', 'p(a,a)']),
                         ['p(a,a,b)'])
        self.assertEqual(bench.instances_of('p(_,_)', ['p(a,b)', 'p(f(a,b),[c,d])']), ['p(a,b)', 'p(f(a,b),[c,d])'])
        self.assertIsNone(bench.instances_of('p(a,f(X))', ['p(a,f(b))']))
        self.assertIsNone(bench.instances_of('p', ['p']))


class ReferenceCheckTests(unittest.TestCase):
    """On the uai suite a baseline's answer that the reference contradicts
    is a failed run (`check_against_reference`), and under --pr the systems
    that print no P(e) are asked it as a query (`pe_program`)."""

    @staticmethod
    def rec(system, variant, **kw):
        return {'instance': 'n', 'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1.0, **kw}

    def check(self, rec, *others):
        done = {(r['system'], r['variant']): r for r in (*others, rec)}
        bench.check_against_reference(rec, done)
        return rec

    def test_an_answer_off_the_reference_is_a_failed_run(self):
        exact = self.rec('exact', 'cliquetree', answers={'a': 0.25, 'b': 0.75}, log10_z=-2.5)
        merlin = self.rec('merlin', 'bte', answers={'a': 0.2, 'b': 0.8})
        r = self.check(self.rec('ace', 'auto', answers={'a': 0.25 + 5e-7, 'b': 0.75}, log10_z=-2.5 + 2e-6), exact, merlin)
        self.assertEqual((r['status'], r['ref'], r['pr_ref']), ('ok', 'exact:cliquetree', 'exact:cliquetree'))
        self.assertAlmostEqual(r['pr_ref_diff'], 2e-6, delta=1e-12)
        # log10 Z to 1e-6 times max(1, |log10 Z|): 2.5e-6 here
        r = self.check(self.rec('ace', 'auto', answers={'a': 0.25, 'b': 0.75}, log10_z=-2.5 + 3e-6), exact)
        self.assertEqual(r['status'], 'error')
        self.assertRegex(r['error'], "log10 Z differs from exact:cliquetree's by 3.00e-06")
        self.assertEqual(r['log10_z'], -2.5 + 3e-6)   # kept beside the error
        # P(e) printed as zero reads as inconsistent evidence
        r = self.check(self.rec('ace', 'auto', answers='inconsistent', log10_z=-bench.math.inf), exact)
        self.assertEqual(r['status'], 'error')
        self.assertRegex(r['error'], "the marginals differ from exact:cliquetree's by inf")
        r = self.check(self.rec('libdai', 'jtree', answers={'a': float('nan'), 'b': 0.75}), exact)
        self.assertEqual(r['status'], 'error')

    def test_the_reference_is_the_first_that_answered(self):
        failed = self.rec('exact', 'cliquetree', status='timeout')
        merlin = self.rec('merlin', 'bte', answers={'a': 0.25})
        merlin_pr = self.rec('merlin', 'bte-pr', log10_z=10.0)
        tddlog = self.rec('tddlog', 'auto+evidence-probability+no-queries', log10_z=10.0 + 5e-6)
        r = self.check(self.rec('ace', 'auto', answers={'a': 0.25}, log10_z=10.0 + 5e-6), failed, merlin, merlin_pr, tddlog)
        self.assertEqual((r['status'], r['ref'], r['pr_ref']), ('ok', 'merlin:bte', 'merlin:bte-pr'))
        r = self.check(self.rec('pgmpy', 've-pr', log10_z=10.0 + 2e-5), failed, tddlog)
        self.assertEqual((r['status'], r['pr_ref']), ('error', 'tddlog:auto+evidence-probability+no-queries'))
        # a reference of -inf (inconsistent evidence) against a number
        r = self.check(self.rec('ace', 'auto', log10_z=-3.0), self.rec('exact', 'cliquetree', log10_z=-bench.math.inf))
        self.assertEqual(r['status'], 'error')
        # nothing to check against, and the reference itself, stay as they are
        for r, others in [(self.rec('ace', 'auto', answers={'a': 0.3}, log10_z=1.0), ()),
                          (self.rec('merlin', 'bte', answers={'a': 0.3}), (failed,)),
                          (self.rec('ace', 'auto', answers={'c': 0.3}), (merlin,))]:
            before = dict(r)
            self.assertEqual(self.check(r, *others), before)

    def test_a_pr_run_asks_p_e_and_checks_each_baseline(self):
        """tests/regress/markov.uai with its evidence, Z = 165, under --pr:
        the clique tree run for real; ProbLog (a stub) is asked P(e) of
        the program's P(e) program; Ace (a stub) prints P(e) to six
        digits, twice too large, or as zero, and its marginals as the clique
        tree's; Scallop (a stub) is asked P(e) as ProbLog is."""
        import bn_exact
        regress = SOURCE / 'tests' / 'regress'
        real = bench.timed
        with scratch_dir('uai-check-') as d:
            selection = dict(suite='uai', only=['uai_markov'], families=[], uai_benchmarks=str(regress), dir=d,
                             cnf=False)
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**selection))
            path = Path(d) / 'uai_markov.pl'
            program = path.read_text()
            pe = 10 ** (bench.math.log10(165) - bench.program_offset(str(path)))
            rows, values, evidence, _ = bn_exact.parse(program)
            states, tables = bn_exact.factors(rows, values)
            exact, _, _ = bn_exact.marginals(states, tables, evidence)
            network = bench.gen_bn.parse(str(regress / 'markov.uai'))[0]
            ace_rows = ''.join(
                f'Pr({v},e) = [{", ".join(repr(float(exact[v][states[v].index(bench.gen_bn.ident(u))])) if v in exact else repr(float(i == 0)) for i, u in enumerate(network[1][v]))}]\n'
                for v in network[0])   # an observed variable's row is not read
            ace = Path(d) / 'ace'
            ace.mkdir()
            for f in ['ace.jar', 'inflib.jar', 'jdom.jar', 'c2d_linux']:
                (ace / f).touch()
            args = SimpleNamespace(**selection, strategies=[], kbs=['ddnnf'], pita='', aspmc='', scallop='', dice='',
                                   exact=True, ace=str(ace), ace_variants=['auto'], ace_heap='1g', ace_java='java',
                                   merlin='', ganak='', pr=True, problog='problog', tddlog='tddlog',
                                   out=str(Path(d) / 'results.jsonl'), timeout=60, no_skip=True)
            # -Xlog: the same numbers' natural logarithms, as AceEvaluate writes them
            log_rows = re.sub(r'\[(.*)\]', lambda m: '[' + ', '.join(
                repr(bench.math.log(float(x))) if float(x) > 0 else '-Infinity' for x in m.group(1).split(',')) + ']',
                ace_rows.replace('Pr(', 'log_e(Pr(').replace(',e) =', ',e)) ='))
            problog = []
            for variant, printed, error in [('auto', f'Pr(e) = {pe:.5E}', None),
                                            ('auto', f'Pr(e) = {2 * pe:.5E}', 'log10 Z differs'),
                                            ('auto', 'Pr(e) = 0.0', 'the marginals differ'),
                                            ('Xlog', f'log_e(Pr(e)) = {bench.math.log(pe):.5E}', None)]:
                args.ace_variants = [variant]
                fake_ace = AceTests.fake_ace(None, [], f'e = {{}}\n\n{printed}\n\n'
                                             + (log_rows if variant == 'Xlog' else ace_rows))

                def timed(cmd, timeout, cwd=None, env=None):
                    if cmd[0] == sys.executable:   # bn_exact.py
                        return real(cmd, timeout, cwd, env)
                    if cmd[0] == 'problog':
                        problog.append(cmd)
                        return {'status': 'ok', 'seconds': 0.5, 'stdout': f'{bench.PE_QUERY}:\t{pe!r}\n', 'stderr': ''}
                    r = fake_ace(cmd, timeout, cwd, env)
                    # AceEvaluate prints P(e) in full; the auto cases leave it out, for the file's six digits
                    full = f'log_e(Pr(e)) = {bench.math.log(pe)!r}\n' if variant == 'Xlog' else ''
                    return {**r, 'stdout': full} if 'edu.ucla.belief.ace.AceEvaluate' in cmd else r
                Path(args.out).unlink(missing_ok=True)
                with self.subTest(printed=printed), patch.object(bench, 'timed', timed), \
                        redirect_stdout(io.StringIO()):
                    bench.run(args)
                    records = {(r['system'], r['variant']): r
                               for r in map(bench.json.loads, Path(args.out).read_text().splitlines())}
                    r = records[('ace', variant)]
                    self.assertEqual(r['status'], 'error' if error else 'ok')
                    # the marginals are checked first, and a failed run's log10 Z is not checked
                    self.assertEqual(r['ref' if error == 'the marginals differ' else 'pr_ref'], 'exact:cliquetree')
                    if error:
                        self.assertIn(error, r['error'])
                    else:
                        self.assertAlmostEqual(r['log10_z'], bench.math.log10(165), delta=3e-6)
                        self.assertLess(r['ref_diff'], 1e-12)
                    p = records[('problog', 'ddnnf')]
                    self.assertEqual((p['status'], p['pr_ref']), ('ok', 'exact:cliquetree'))
                    self.assertAlmostEqual(p['log10_z'], bench.math.log10(165), places=12)
                    self.assertNotIn('answers', p)
            self.assertEqual(problog[0][-1], str(Path(d) / 'uai_markov.pe.pl'))
            asked = Path(problog[0][-1]).read_text()
            self.assertEqual(asked.splitlines()[0], program.splitlines()[0])   # the offset line
            self.assertNotIn('evidence(', asked)
            self.assertEqual(bench.queries(problog[0][-1]), [bench.PE_QUERY])
            observed = re.findall(r'^evidence\((.*)\)\.$', program, re.M)
            self.assertIn(f'{bench.PE_QUERY} :- {", ".join(observed)}.', asked)
            # Scallop (a stub) is asked P(e) as ProbLog is: the P(e) program, translated into Scallop
            scallop = []

            def timed_scallop(cmd, timeout, cwd=None, env=None):
                if cmd[0] == sys.executable:   # bn_exact.py
                    return real(cmd, timeout, cwd, env)
                scallop.append(cmd)
                return {'status': 'ok', 'seconds': 0.5, 'stdout': f'q__0: {{{pe!r}::()}}\n', 'stderr': ''}
            Path(args.out).unlink(missing_ok=True)
            with patch.object(bench, 'timed', timed_scallop), redirect_stdout(io.StringIO()):
                bench.run(SimpleNamespace(**{**vars(args), 'kbs': [], 'ace': '', 'scallop': 'scli'}))
            records = {(r['system'], r['variant']): r
                       for r in map(bench.json.loads, Path(args.out).read_text().splitlines())}
            s = records[('scallop', bench.SCALLOP_VARIANT)]
            self.assertEqual((s['status'], s['pr_ref']), ('ok', 'exact:cliquetree'))
            self.assertAlmostEqual(s['log10_z'], bench.math.log10(165), places=12)
            self.assertNotIn('answers', s)
            self.assertEqual(s['scallop_route'], 'first-order')
            scl = str(Path(d) / 'uai_markov.scl')
            self.assertEqual(scallop, [['scli', '-p', 'topkproofs', '-k', str(bench.SCALLOP_K), scl]])
            self.assertIn(f'// query q__0 {bench.PE_QUERY}', Path(scl).read_text().splitlines())
            # Ace reads the .inst files, which gen --pr does not write
            pr_dir = Path(d) / 'pr'
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**{**selection, 'dir': str(pr_dir)}, pr=True))
            with self.assertRaisesRegex(SystemExit, r'has none \(gen --pr writes none\)'):
                bench.run(SimpleNamespace(**{**vars(args), 'dir': str(pr_dir), 'kbs': []}))


class ProcessTests(unittest.TestCase):
    def test_success_and_error_output(self):
        result = bench.timed([sys.executable, '-c',
                              'import sys; print("answer"); print("stats", file=sys.stderr)'], 5)
        self.assertEqual(result['status'], 'ok')
        self.assertEqual(result['stdout'], 'answer\n')
        self.assertEqual(result['stderr'], 'stats\n')
        result = bench.timed([sys.executable, '-c',
                              'import sys; print("failed", file=sys.stderr); sys.exit(2)'], 5)
        self.assertEqual(result['status'], 'error')
        self.assertEqual(result['error'], 'failed')
        self.assertEqual(result['returncode'], 2)

    @unittest.skipUnless(bench.os.name == 'posix', 'signals are POSIX')
    def test_a_run_a_signal_ends_before_its_limit_is_stopped(self):
        # the box's out-of-memory guard (earlyoom's SIGTERM, the kernel's SIGKILL) ended the run before
        # its limit: no failure of the system's, status stopped with the returncode, and no skip
        for sig in ('SIGTERM', 'SIGKILL'):
            r = bench.timed([sys.executable, '-c', f'import os, signal; os.kill(os.getpid(), signal.{sig})'], 30)
            self.assertEqual((r['status'], r['returncode'], r['error']),
                             ('stopped', -getattr(bench.signal, sig), f'exit {r["returncode"]}'), sig)
            self.assertTrue(bench.stopped(r))
            self.assertFalse(bench.memout(r) or bench.ran_out(r))
        # what the run printed is kept: Julia's report of the SIGTERM it was sent
        report = '[2340709] signal 15: Terminated\nAllocations: 12625286 (Pool: 12624805; Big: 481); GC: 10'
        r = bench.timed([sys.executable, '-c', f'import os, signal, sys; print({report!r}, file=sys.stderr); '
                         'sys.stderr.flush(); os.kill(os.getpid(), signal.SIGTERM)'], 30)
        self.assertEqual((r['status'], r['error']), ('stopped', report))
        # an allocation failure before the signal stays the memout it is
        r = bench.timed([sys.executable, '-c', 'import os, signal, sys; print("Fatal error: out of memory", '
                         'file=sys.stderr); sys.stderr.flush(); os.kill(os.getpid(), signal.SIGKILL)'], 30)
        self.assertEqual(r['status'], 'error')
        self.assertTrue(bench.memout(r) and not bench.stopped(r))
        # another signal is the program's own failure (a stack overflow's SIGSEGV)
        r = bench.timed([sys.executable, '-c', 'import os, signal; os.kill(os.getpid(), signal.SIGSEGV)'], 30)
        self.assertEqual(r['status'], 'error')
        self.assertFalse(bench.stopped(r))

    def test_an_earlier_record_of_a_stop_is_stopped(self):
        # records written before the status: a bare exit by SIGTERM or SIGKILL, or Julia's report of a SIGTERM
        for text in ['exit -15', 'exit -9', '[2043848] signal 15: Terminated\nin expression starting at none:0\n'
                     'Allocations: 33002010 (Pool: 33000276; Big: 1734); GC: 29']:
            r = {'status': 'error', 'seconds': 38.2, 'error': text}
            self.assertTrue(bench.stopped(r), text)
            self.assertFalse(bench.memout(r) or bench.ran_out(r), text)
        for r in [{'status': 'error', 'error': 'exit -11'}, {'status': 'error', 'error': 'exit -150'},
                  {'status': 'error', 'error': 'scli: exit -15 in a child'},
                  {'status': 'error', 'error': bench.cap_error(31 << 20)},
                  {'status': 'timeout', 'seconds': 300}, {'status': 'ok', 'seconds': 1}]:
            self.assertFalse(bench.stopped(r), r)

    def test_error_keeps_the_cause_before_a_backtrace_hint(self):
        message = 'memory allocation failed\nnote: run with RUST_BACKTRACE=1'
        result = bench.timed([sys.executable, '-c',
                              f'import sys; print({message!r}, file=sys.stderr); sys.exit(1)'], 5)
        self.assertEqual(result['error'], message)

    @unittest.skipUnless(bench.os.name == 'posix', 'process-group cleanup is POSIX')
    def test_timeout_stops_descendant_after_frontend_exits(self):
        scratch = scripts.parent / 'tmp'
        scratch.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch, prefix='bench-timeout-') as directory:
            marker = Path(directory) / 'survived'
            child = ('import time; from pathlib import Path; time.sleep(1); '
                     f'Path({str(marker)!r}).write_text("orphan survived")')
            # The frontend exits, but its child retains the output pipes.
            # Killing just the frontend cannot stop that child's work.
            parent = f'import subprocess, sys; subprocess.Popen([sys.executable, "-c", {child!r}])'
            result = bench.timed([sys.executable, '-c', parent], 0.2)
            self.assertEqual(result['status'], 'timeout')
            time.sleep(1.1)
            self.assertFalse(marker.exists(), 'timed-out descendant kept running')

    @unittest.skipUnless(bench.os.name == 'posix', 'process-group cleanup is POSIX')
    def test_cancelling_driver_stops_its_separate_compiler_session(self):
        scratch = scripts.parent / 'tmp'
        scratch.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch, prefix='bench-cancel-') as directory:
            directory = Path(directory)
            fold_files(directory, ['g10_25'])
            ready, marker = directory / 'ready', directory / 'survived'
            child = f'import time; from pathlib import Path; time.sleep(1); Path({str(marker)!r}).touch(); time.sleep(30)'
            compiler = directory / 'compiler'
            compiler.write_text('#!/usr/bin/env python3\nimport os, subprocess, sys, time\nfrom pathlib import Path\n'
                                f'subprocess.Popen([sys.executable, "-c", {child!r}])\n'
                                f'Path({str(ready)!r}).write_text(str(os.getpid()))\ntime.sleep(30)\n')
            compiler.chmod(0o755)
            driver = bench.subprocess.Popen([
                sys.executable, str(scripts / 'bench.py'), 'run', '--only', 'g10_25',
                '--dir', str(directory), '--out', str(directory / 'results.jsonl'),
                '--tddlog', str(compiler), '--strategies', 'closed', '--kbs', '', '--timeout', '30',
            ], stdout=bench.subprocess.PIPE, stderr=bench.subprocess.PIPE)
            try:
                deadline = time.monotonic() + 5
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(ready.exists(), 'compiler did not start')
                driver.terminate()
                driver.communicate(timeout=5)
                self.assertEqual(driver.returncode, 128 + bench.signal.SIGTERM)
                time.sleep(1.1)
                self.assertFalse(marker.exists(), 'cancelled compiler descendant survived')
            finally:
                if ready.exists():
                    try:
                        bench.os.killpg(int(ready.read_text()), bench.signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                if driver.poll() is None:
                    driver.kill()
                driver.communicate()


class LaneTests(unittest.TestCase):
    def run_lanes(self, only, cpus, no_skip, timed):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        args = SimpleNamespace(suite='standard', only=only, families=[], strategies=['closed'], kbs=[],
                               pita='', aspmc='', scallop='', dice='', dir=fold_files(directory.name, only), out='results.jsonl',
                               tddlog='tddlog', timeout=1, no_skip=no_skip, exact=False, ace='', merlin='', ganak='',
                               pr=False, cpus=cpus)
        # a folded instance's command ends in its arguments: its name, for the mock to tell them apart
        with patch('builtins.open', mock_open()) as output, patch.object(bench, 'timed', timed), \
                patch.object(bench, 'arguments_of', lambda path: [Path(path).stem]), \
                patch.object(bench, 'queries', return_value=['q']), redirect_stdout(io.StringIO()):
            bench.run(args)
        return [bench.json.loads(c.args[0]) for c in output().write.call_args_list]

    def test_a_family_runs_in_one_lane_and_a_timeout_skips_its_larger_instances(self):
        """Two lanes, two families of three: each family's instances run in
        order of size in one lane, and a timeout on the second instance of a
        family skips the third, as in a sequential run."""
        graphs, grids = ['g10_25', 'g11_29', 'g12_32'], ['grid4', 'grid5', 'grid6']
        calls, lock = [], bench.threading.Lock()

        def timed(cmd, timeout, cwd=None, env=None):
            name = Path(cmd[-1]).stem
            with lock:
                calls.append((name, bench.LANE.cpu))
            time.sleep(0.05)
            if name in ('g11_29', 'grid5'):
                return {'status': 'timeout', 'seconds': timeout}
            return {'status': 'ok', 'seconds': 0.1, 'stdout': 'q: 0.5\n', 'stderr': ''}
        records = self.run_lanes(graphs + grids, [3, 5], False, timed)
        status = {r['instance']: r['status'] for r in records}
        self.assertEqual(status, {'g10_25': 'ok', 'g11_29': 'timeout', 'g12_32': 'skipped',
                                  'grid4': 'ok', 'grid5': 'timeout', 'grid6': 'skipped'})
        for family in (graphs, grids):
            ran = [(n, cpu) for n, cpu in calls if n in family]
            self.assertEqual([n for n, _ in ran], family[:2])
            self.assertEqual(len({cpu for _, cpu in ran}), 1)
        self.assertEqual({cpu for _, cpu in calls}, {3, 5})

    def test_commands_are_pinned_to_their_lane(self):
        """Through the real timed: the command runs under taskset on its
        lane's CPU, and without --cpus it is not pinned."""
        if not bench.shutil.which('taskset'):
            self.skipTest('no taskset')
        cpu = sorted(bench.os.sched_getaffinity(0))[-1]
        bench.LANE.cpu = cpu
        try:
            r = bench.timed([sys.executable, '-c', 'import os; print(sorted(os.sched_getaffinity(0)))'], 5)
        finally:
            bench.LANE.cpu = None
        self.assertEqual(r['stdout'].strip(), f'[{cpu}]')

    def test_a_cpu_listed_twice_is_refused(self):
        with self.assertRaisesRegex(SystemExit, 'listed twice'):
            self.run_lanes(['g10_25'], [2, 2], True, lambda *a, **k: None)


class TableTests(unittest.TestCase):
    def test_reports_do_not_drop_larger_or_external_instances(self):
        systems = [('tddlog', 'closed')]
        rows = {(name, 'tddlog', 'closed'): {
            'system': 'tddlog', 'variant': 'closed', 'status': 'ok', 'seconds': 1,
            'answers': {'q': 0.5},
        } for name in ['grid40', 'external-instance']}
        for argv in [['bench.py', 'table', 'results.jsonl'],
                     ['bench.py', 'compare', '--before', 'before.jsonl', '--after', 'after.jsonl']]:
            output = io.StringIO()
            with patch.object(sys, 'argv', argv), patch.object(bench, 'load', return_value=(rows, systems)), \
                    redirect_stdout(output):
                bench.main()
            for name in ['grid40', 'external-instance']:
                self.assertIn('| ' + name + ' |', output.getvalue())

    def test_table_separates_an_approximation(self):
        """A system named by --approx (an old record of Scallop's top-10
        proofs) gets a column of its own; by default every system is exact,
        Scallop's top-k proofs at SCALLOP_K among them."""
        systems = [('tddlog', 'auto'), ('problog', 'ddnnf'), ('scallop', 'topk10')]
        rows = {('dice_twocoins', system, variant): {
                    'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1,
                    'answers': {'firstcoin': 0.5 if system == 'scallop' else 1 / 3},
                } for system, variant in systems}
        for argv, shown in [(['--approx', 'scallop:topk10'], '| 0.0e+00 (problog:ddnnf) | 1.7e-01 |'),
                            ([], '| 1.7e-01 (problog:ddnnf) |')]:
            output = io.StringIO()
            with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl'] + argv), \
                    patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
                bench.main()
            self.assertEqual('approximate: max diff' in output.getvalue(), bool(argv))
            self.assertIn(shown, output.getvalue())

    def test_an_answer_that_is_not_a_number_is_no_match(self):
        systems = [('tddlog', 'auto'), ('exact', 'cliquetree')]
        rows = {('bn_alarm', system, variant): {
                    'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1,
                    'answers': {'q(a)': 0.25 if system == 'exact' else float('nan')},
                } for system, variant in systems}
        for argv, shown in [(['bench.py', 'table', 'results.jsonl'], '| inf (exact:cliquetree) |'),
                            (['bench.py', 'compare', '--before', 'b.jsonl', '--after', 'a.jsonl',
                              '--systems', 'tddlog:auto'], 'answers moved on bn_alarm')]:
            output = io.StringIO()
            nan_after = {k: v for k, v in rows.items() if k[1] == 'tddlog'}
            before = {k: dict(v, answers={'q(a)': 0.25}) for k, v in nan_after.items()}
            loads = [(rows, systems)] if argv[1] == 'table' else [(before, systems), (nan_after, systems)]
            with patch.object(sys, 'argv', argv), patch.object(bench, 'load', side_effect=loads), \
                    redirect_stdout(output):
                bench.main()
            self.assertIn(shown, output.getvalue())

    def test_evidence_found_inconsistent_against_answers_is_no_match(self):
        """The clique tree's product underflowing to zero on Grids_17 of the
        UAI suite, read as evidence of probability zero."""
        systems = [('tddlog', 'auto'), ('exact', 'cliquetree')]
        rows = {('uai_grids_17', system, variant): {
                    'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1,
                    'answers': 'inconsistent' if system == 'exact' else {'x0(v_0)': 0.5},
                } for system, variant in systems}
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('| inf (exact:cliquetree) |', output.getvalue())
        rows[('uai_grids_17', 'tddlog', 'auto')]['answers'] = 'inconsistent'
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('| 0.0e+00 (exact:cliquetree) |', output.getvalue())

    def test_log10_z_has_a_column_of_its_own(self):
        """`--pr` records: Merlin's PR run has no answers, which the answers'
        column passes over, and the log10 Z column compares with the clique
        tree's; evidence of probability zero on both sides is no
        difference."""
        systems = [('tddlog', 'auto+evidence-probability'), ('exact', 'cliquetree'), ('merlin', 'bte-pr')]
        z = {'tddlog': 2.5 + 1e-12, 'exact': 2.5, 'merlin': 2.5000002}
        rows = {('uai_csp_1', system, variant): dict(
                    {'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1, 'log10_z': z[system]},
                    **({} if system == 'merlin' else {'answers': {'x0(v_0)': 0.5}}))
                for system, variant in systems}
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('| log10 Z: max diff (vs) |', output.getvalue())
        self.assertIn('| 0.0e+00 (exact:cliquetree) | 2.0e-07 (exact:cliquetree) |', output.getvalue())
        for r in rows.values():
            r['log10_z'] = -bench.math.inf
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('| 0.0e+00 (exact:cliquetree) | 0.0e+00 (exact:cliquetree) |', output.getvalue())

    def test_ace_is_the_last_reference(self):
        systems = [('tddlog', 'auto'), ('exact', 'cliquetree'), ('ace', 'auto')]
        rows = {('ace_fs_07', system, variant): {
                    'system': system, 'variant': variant, 'status': 'error' if system == 'exact' else 'ok',
                    'seconds': 1, 'answers': {'q(a)': 0.25 if system == 'ace' else 0.25 + 1e-12},
                } for system, variant in systems}
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('| 1.0e-12 (ace:auto) |', output.getvalue())


class ScallopTests(unittest.TestCase):
    """Scallop runs its top-k proofs at the largest k its command line
    takes, usize::MAX, where no proof is dropped and the answer is exact
    (docs/baselines.md section 3): one variant, named by that k, on the
    program problog2scallop.py writes from the ProbLog program."""

    PROGRAM = '0.5::a.\n0.5::b.\nc :- a.\nc :- b.\nquery(c).\nevidence(a, false).\n'

    def run_scallop(self, variant, out, program=PROGRAM):
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            return {'status': 'ok', 'seconds': 1.0, 'stdout': out, 'stderr': ''}
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.pl'
            path.write_text(program)
            args = SimpleNamespace(dir=d, scallop='scli', timeout=10.0)
            with patch.object(bench, 'timed', timed):
                r = bench.run_scallop(args, 'p', str(path), variant)
            scl = (Path(d) / 'p.scl').read_text() if (Path(d) / 'p.scl').exists() else None
        return r, calls, scl

    def test_the_largest_k_is_the_one_variant(self):
        self.assertEqual(bench.SCALLOP_K, 18446744073709551615)   # usize::MAX, scli's -k
        self.assertEqual(bench.SCALLOP_VARIANT, 'allproofs')
        r, calls, scl = self.run_scallop(bench.SCALLOP_VARIANT, 'e__: {0.5::()}\nqe__0: {0.25::()}\n')
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][:5], ['scli', '-p', 'topkproofs', '-k', '18446744073709551615'])
        self.assertTrue(calls[0][5].endswith('p.scl'))
        self.assertIn('rel e__() = not a()', scl.splitlines())
        # Scallop's seconds alone, the translation being the harness's; P(c | not a) = P(c, not a) / P(not a)
        self.assertEqual(r['seconds'], 1.0)
        self.assertEqual(r['answers'], {'c': 0.5})
        self.assertEqual(r['stats'], {'scallop_route': 'first-order'})

    def test_another_k_is_refused(self):
        with self.assertRaisesRegex(SystemExit, 'unknown Scallop variant topk1000'):
            self.run_scallop('topk1000', '')

    def test_no_proof_is_an_answer_of_zero(self):
        r, _, _ = self.run_scallop(bench.SCALLOP_VARIANT, 'e__: {0.5::()}\nqe__0: {}\n')
        self.assertEqual(r['answers'], {'c': 0.0})

    def test_evidence_without_a_proof_is_inconsistent(self):
        r, _, _ = self.run_scallop(bench.SCALLOP_VARIANT, 'e__: {}\nqe__0: {}\n')
        self.assertEqual(r['answers'], 'inconsistent')
        # the ground route's reading too: Circulette's ct_ALU_Controller, the evidence no
        # fault causes, and no query
        self.assertEqual(bench.ground2scallop.answers('e: {}\n', 'query e\n'), 'inconsistent')

    def test_output_that_cannot_be_read_is_an_error(self):
        r, _, _ = self.run_scallop(bench.SCALLOP_VARIANT, 'qe__0: {}\n')
        self.assertEqual(r['status'], 'error')
        self.assertIn('no e__', r['error'])
        self.assertEqual(r['stats'], {'scallop_route': 'first-order'})

    def test_a_program_the_translation_refuses_is_unsupported(self):
        # no ground fallback: the reason says whose limit it is
        for program, reason in [('p([1,2]).\nquery(p(_)).\n', 'problog2scallop.py: problog2scallop: translator: a list'),
                                ('q(1).\np(X) :- q(X), \\+ p(X).\nquery(p(1)).\n',
                                 'problog2scallop.py: problog2scallop: a negation through recursion, which Scallop '
                                 'cannot express (it stratifies negation by relation)')]:
            with self.subTest(program=program):
                r, calls, scl = self.run_scallop(bench.SCALLOP_VARIANT, '', program)
                self.assertEqual(calls, [])
                self.assertIsNone(scl)
                self.assertEqual(r['status'], 'unsupported')
                self.assertTrue(r['error'].startswith(reason), r['error'])
                self.assertEqual(r['stats'], {'scallop_route': 'first-order'})

    def test_a_circuit_is_read_as_its_ground_program(self):
        # a circuit has no first-order source: its ground program, a relation per gate
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append((cmd, timeout))
            if cmd[1] == '--ground':
                return {'status': 'ok', 'seconds': 0.25, 'stdout': self.PROGRAM, 'stderr': ''}
            return {'status': 'ok', 'seconds': 1.0, 'stdout': 'e: {0.5::()}\nq0e: {0.25::()}\n', 'stderr': ''}
        with tempfile.TemporaryDirectory() as d:
            args = SimpleNamespace(dir=d, tddlog='tddlog', scallop='scli', timeout=10.0)
            with patch.object(bench, 'timed', timed):
                r = bench.run_scallop(args, 'bb_c', str(Path(d) / 'bb_c.pl'), bench.SCALLOP_VARIANT, circuit=True)
            scl = (Path(d) / 'bb_c.scl').read_text()
        self.assertEqual(calls[0], (['tddlog', '--ground', str(Path(d) / 'bb_c.pl')], 10.0))
        self.assertEqual(calls[1], (['scli', '-p', 'topkproofs', '-k', str(bench.SCALLOP_K), str(Path(d) / 'bb_c.scl')],
                                    9.75))
        self.assertEqual(scl, bench.ground2scallop.translate(self.PROGRAM))
        # the grounding's seconds and Scallop's; P(c | not a) = P(c, not a) / P(not a)
        self.assertEqual(r['seconds'], 1.25)
        self.assertEqual(r['answers'], {'c': 0.5})
        self.assertEqual(r['stats'], {'scallop_route': 'ground'})


class Ground2ScallopTests(unittest.TestCase):
    """ground2scallop.py, Scallop's reading of a circuit's ground program."""

    def test_a_name_of_two_arities_is_two_relations(self):
        scl = bench.ground2scallop.translate('0.5::a.\nhmm(x) :- hmm(x,y,z,w).\nhmm(x,y,z,w) :- a.\nquery(hmm(x)).\n')
        # hmm/4 is met first: a rule's body is read before its head
        self.assertIn('type r_hmm(String, String, String, String)', scl)
        self.assertIn('type r_hmm__1(String)', scl)
        self.assertIn('rel r_hmm__1("x") = r_hmm("x", "y", "z", "w")', scl)
        self.assertIn('rel r_hmm("x", "y", "z", "w") = r_a()', scl)

    # A loop's steps, s(2) reading the negation of s(1): a relation that reads
    # its own negation, which Scallop refuses to stratify, though the ground
    # atoms form no cycle. P(s(3)) = 1 - P(s(2)) = 1 - 0.7 * 0.6 = 0.58.
    LADDER = ('0.3::f1.\n0.6::f2.\ns(1) :- f1.\ns(2) :- \\+s(1), f2.\ns(3) :- \\+s(2).\n'
              't(a) :- t(b).\nt(b) :- f1.\nt(b) :- t(a).\nquery(s(3)).\nquery(t(a)).\n')
    # what the translation wrote before it split such a relation by atom
    UNSPLIT = ('// query q0 s(3)\ntype r_f1()\ntype r_f2()\ntype r_s(String)\ntype q0()\nrel 0.3::r_f1()\n'
               'rel 0.6::r_f2()\nrel r_s("1") = r_f1()\nrel r_s("2") = not r_s("1"), r_f2()\n'
               'rel r_s("3") = not r_s("2")\nrel q0() = r_s("3")\nquery q0\n')

    def test_a_relation_on_a_negative_cycle_is_written_atom_by_atom(self):
        scl = bench.ground2scallop.translate(self.LADDER)
        self.assertNotIn('r_s(', scl)
        for line in ('type r_s__g0()', 'rel r_s__g0() = r_f1()', 'rel r_s__g1() = not r_s__g0(), r_f2()',
                     'rel r_s__g2() = not r_s__g1()', 'rel q0() = r_s__g2()'):
            self.assertIn(line, scl.splitlines())
        # a recursion without negation keeps its relation
        self.assertIn('type r_t(String)', scl)
        self.assertIn('rel r_t("a") = r_t("b")', scl)
        self.assertEqual(bench.ground2scallop.negative_cycles([(('r_t', ('"a"',)), [(False, ('r_t', ('"b"',)))])]), set())

    def test_a_small_probability_is_written_without_an_exponent(self):
        scl = bench.ground2scallop.translate('1e-08::a.\n2.5E-5::b.\n0.5::c.\nquery(a).\n')
        for line in ('rel 0.00000001::r_a()', 'rel 0.000025::r_b()', 'rel 0.5::r_c()'):
            self.assertIn(line, scl.splitlines())
        self.assertEqual(float(bench.ground2scallop.probability('1e-300')), 1e-300)

    def test_answers_name_the_queries_and_divide_by_the_evidence(self):
        scl = '// query q0 c\n// query q1 d( x )\n'
        self.assertEqual(bench.ground2scallop.answers('q0: {0.25::()}\nq1: {}\n', scl), {'c': 0.25, 'd(x)': 0.0})
        self.assertEqual(bench.ground2scallop.answers('e: {0.5::()}\nq0e: {0.25::()}\nq1e: {}\n', scl),
                         {'c': 0.5, 'd(x)': 0.0})

    # Scallop's scli, as the runners give it (SCLI=PATH)
    SCLI = os.environ.get('SCLI', '')

    @unittest.skipUnless(SCLI and os.access(SCLI, os.X_OK), 'no Scallop: SCLI names no scli')
    def test_scallop_answers_the_loop_it_refused_unsplit(self):
        import subprocess
        with tempfile.TemporaryDirectory() as d:
            def scli(text):
                path = os.path.join(d, 'p.scl')
                with open(path, 'w') as f:
                    f.write(text)
                return subprocess.run([self.SCLI, '-p', 'topkproofs', '-k', str(bench.SCALLOP_K), path],
                                      capture_output=True, text=True, timeout=60)
            r = scli(self.UNSPLIT)
            self.assertIn('Cannot stratify program', r.stdout + r.stderr)
            scl = bench.ground2scallop.translate(self.LADDER)
            r = scli(scl)
            self.assertEqual(r.returncode, 0, r.stderr)
            answers = bench.ground2scallop.answers(r.stdout, scl)
            self.assertAlmostEqual(answers['s(3)'], 0.58, places=12)
            self.assertAlmostEqual(answers['t(a)'], 0.3, places=12)
            # Scallop refused `1e-08::` as a syntax error (sc1u's promedus and or_chain networks)
            scl = bench.ground2scallop.translate('1e-08::a.\n0.5::b.\nc :- a.\nc :- b.\nquery(c).\n')
            r = scli(scl)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertAlmostEqual(bench.ground2scallop.answers(r.stdout, scl)['c'], 0.5 + 0.5e-08, places=15)


class PitaTests(unittest.TestCase):
    def pita_command(self, soft):
        import resource
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            return {'status': 'ok', 'seconds': 0.1, 'stdout': '', 'stderr': ''}
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.pl'
            path.write_text('0.5::a.\nquery(a).\n')
            args = SimpleNamespace(dir=d, pita='swipl', pita_packs='', timeout=1)
            with patch.object(bench, 'timed', timed), \
                    patch.object(resource, 'getrlimit', return_value=(soft, resource.RLIM_INFINITY)):
                bench.run_pita(args, 'p', str(path))
        return calls[0]

    def test_the_table_space_is_the_address_space_limit(self):
        """SWI's table space is the run's memory limit, set before the
        program is consulted, so that the limit binds and not SWI's default
        of 1 GB; without a limit the default stays."""
        import resource
        cmd = self.pita_command(12_000_000_000)
        flag = cmd.index('set_prolog_flag(table_space, 12000000000)')
        self.assertEqual(cmd[flag - 1], '-g')
        self.assertLess(flag, next(i for i, c in enumerate(cmd) if c.startswith('consult(')))
        self.assertFalse(any('table_space' in c for c in self.pita_command(resource.RLIM_INFINITY)))

    def test_write_new_leaves_a_hard_link_whole(self):
        with tempfile.TemporaryDirectory() as d:
            a, b = Path(d) / 'a', Path(d) / 'b'
            a.write_text('the table\'s file\n')
            os.link(a, b)
            bench.write_new(str(a), 'this job\'s\n')
            self.assertEqual((a.read_text(), b.read_text()), ('this job\'s\n', 'the table\'s file\n'))
            self.assertEqual(sorted(p.name for p in Path(d).iterdir()), ['a', 'b'])

    def test_the_lpad_is_a_new_file_not_one_written_over(self):
        """A run's LPAD replaces the file at its path by a new one: a hard
        link to the old one (another job's copy of the directory) and a
        reader of it keep the whole old text."""
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'p.pl'
            path.write_text('0.5::a.\nquery(a).\n')
            lpad, other = Path(d) / 'p.lpad.pl', Path(d) / 'other.lpad.pl'
            lpad.write_text('old\n')
            os.link(lpad, other)
            args = SimpleNamespace(dir=d, pita='swipl', pita_packs='', timeout=1)
            with patch.object(bench, 'timed', lambda *a, **k: {'status': 'ok', 'seconds': 0.1}):
                bench.run_pita(args, 'p', str(path))
            self.assertEqual(other.read_text(), 'old\n')
            self.assertIn('a:0.5', lpad.read_text())
            self.assertEqual(sorted(p.name for p in Path(d).iterdir()), ['other.lpad.pl', 'p.lpad.pl', 'p.pl'])

    def test_a_full_table_space_ran_out_of_memory(self):
        self.assertTrue(bench.ran_out({'status': 'error', 'error': 'Not enough resources: private_table_space'}))


class PsouffleTests(unittest.TestCase):
    """PSouffle's compilation, timed apart from the run: every record after
    it carries the seconds its text's compilation took (compile_s), which
    the processing charges (rel/charge.py)."""

    @staticmethod
    def timed(cmd, timeout, cwd=None, env=None):
        if '--full-only' in cmd:
            if 'bad' in Path(cwd, 'compute.dl').read_text():     # a compilation that fails, reused
                return {'status': 'error', 'seconds': 7.5, 'returncode': 1, 'error': 'compute.dl: syntax error'}
            return {'status': 'ok', 'seconds': 7.5, 'stdout': '', 'stderr': ''}
        return {'status': 'timeout', 'seconds': 300.0}

    def test_a_text_compiled_once_gives_every_instance_its_seconds(self):
        calls = []

        def timed(cmd, timeout, cwd=None, env=None):
            calls.append(cmd)
            return self.timed(cmd, timeout, cwd, env)
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', timed):
            args = SimpleNamespace(dir=d, psouffle='souffle', timeout=300)
            self.assertEqual(bench.psouffle_binary(args, 'a(1).')[1:], (7.5, None))
            self.assertEqual(bench.psouffle_binary(args, 'a(1).')[1:], (7.5, None))    # reused, its seconds kept
            for _ in range(2):
                _, seconds, err = bench.psouffle_binary(args, 'bad(1).')
                self.assertEqual((seconds, err['status'], err['stats']), (7.5, 'error', {'compile_s': 7.5}))
        self.assertEqual(sum('--full-only' in c for c in calls), 2)

    def test_the_compilation_stops_at_the_runs_limit(self):
        """A compilation past the run's limit is a timeout of the run already
        (its seconds are charged to it), so it is bounded there, and at
        PSOUFFLE_COMPILE_S under a longer limit, where a compilation stopped
        at a lower bound is tried again; its error keeps COMPILE_TIMEOUT's
        text."""
        bounds = []

        def timed(cmd, timeout, cwd=None, env=None):
            bounds.append(timeout)
            return {'status': 'timeout', 'seconds': float(timeout), 'stdout': '', 'stderr': ''}
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', timed):
            _, seconds, err = bench.psouffle_binary(SimpleNamespace(dir=d, psouffle='souffle', timeout=300), 'a(1).')
            bench.psouffle_binary(SimpleNamespace(dir=d, psouffle='souffle', timeout=7200), 'a(1).')
            bench.psouffle_binary(SimpleNamespace(dir=d, psouffle='souffle', timeout=300), 'a(1).')    # reused
        self.assertEqual(bounds, [300.0, bench.PSOUFFLE_COMPILE_S])
        self.assertEqual(seconds, 300.0)
        self.assertTrue(bench.compile_timeout(err))

    def test_a_run_past_the_limit_keeps_the_compilation_seconds(self):
        def translated(args, script, argv, *rest):
            os.makedirs(argv[1], exist_ok=True)
            Path(argv[1], 'compute.dl').write_text('a(1).')
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', self.timed), \
                patch.object(bench, 'translated', translated):
            r = bench.run_psouffle(SimpleNamespace(dir=d, psouffle='souffle', timeout=300), 'p', str(Path(d) / 'p.pl'))
        self.assertEqual((r['status'], r['seconds'], r['stats']), ('timeout', 300.0, {'compile_s': 7.5}))

    def test_inconsistent_evidence_is_its_answer(self):
        # Circulette's ct_ALU_Controller: PSouffle stops on the evidence, as ProbLog does
        def timed(cmd, timeout, cwd=None, env=None):
            if '--full-only' in cmd:
                return self.timed(cmd, timeout, cwd, env)
            return {'status': 'error', 'seconds': 0.5, 'returncode': 1,
                    'error': 'Problog calc failedInconsistent evidence: failure() is deterministically false'}

        def translated(args, script, argv, *rest):
            os.makedirs(argv[1], exist_ok=True)
            Path(argv[1], 'compute.dl').write_text('failure() :- fail().')
        with tempfile.TemporaryDirectory() as d, patch.object(bench, 'timed', timed), \
                patch.object(bench, 'translated', translated):
            r = bench.run_psouffle(SimpleNamespace(dir=d, psouffle='souffle', timeout=300), 'p', str(Path(d) / 'p.pl'))
        self.assertEqual((r['status'], r['answers'], r['seconds']), ('ok', 'inconsistent', 0.5))
        self.assertRegex("InconsistentEvidenceError: Inconsistent evidence detected: 'evidence(failure,true)'.",
                         bench.INCONSISTENT)


class HoldTests(unittest.TestCase):
    def test_a_hold_names_a_run_a_system_and_its_instances(self):
        """A hold TAG=SYSTEM[@INSTANCES] keeps the earlier records in place of a run's on the
        instances its expression matches whole; marked !, it withdraws them, and may leave an
        instance without a record."""
        plain, marked, every = bench.holds(['r2=ace@^(a_3|a_10)$', 'r2=agrum@^a_(30|100)$!', 'r3=pita'], '--hold')
        self.assertEqual(plain[:2] + plain[3:], ('r2', 'ace', False, 'r2=ace@^(a_3|a_10)$'))
        self.assertEqual(marked[:2] + marked[3:], ('r2', 'agrum', True, 'r2=agrum@^a_(30|100)$!'))
        self.assertEqual(every[:2] + every[3:], ('r3', 'pita', False, 'r3=pita'))
        hold = [plain, marked, every]
        self.assertEqual(bench.held(hold, 'r2', 'ace', 'a_10'), 'r2=ace@^(a_3|a_10)$')
        self.assertIsNone(bench.held(hold, 'r2', 'ace', 'a_100'))
        self.assertEqual(bench.held(hold, 'r2', 'agrum', 'a_100'), 'r2=agrum@^a_(30|100)$!')
        self.assertIsNone(bench.held(hold, 'r2', 'agrum', 'a_1000'))
        self.assertEqual(bench.held(hold, 'r3', 'pita', 'b_7'), 'r3=pita')
        for spec in ('r2', 'r2=', '=ace', 'r2=ace@', 'r2=ace@!', 'r2=!'):
            with self.assertRaises(SystemExit):
                bench.holds([spec], '--hold')


if __name__ == '__main__':
    unittest.main()
