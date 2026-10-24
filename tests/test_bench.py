"""Benchmark statistics must include the work after atom compilation."""
import importlib.util
from contextlib import redirect_stdout
import io
import os
from pathlib import Path
import re
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

    def test_values_that_are_not_boolean_tuples_have_no_marginals(self):
        rows = bench.parse_dice('Value\tProbability\t\n0\t0.25\t\n1\t0.75\t\n')
        self.assertEqual(rows, {'0': 0.25, '1': 0.75})
        self.assertIsNone(bench.dice_marginals(rows, 2))


class PplTests(unittest.TestCase):
    """The ppl suite: Dice on an Alea.jl program's circuit, Alea.jl on the
    program, and the files the suite names."""

    def args(self, **kw):
        base = dict(suite='ppl', only=[], families=[], dir='bench', timeout=30, alea='', dice='')
        base.update(kw)
        return SimpleNamespace(**base)

    def test_the_suite_names_programs_that_exist(self):
        spec = bench.instances(self.args())
        self.assertEqual({family for family, _ in spec.values()}, {'arith', 'hybit', 'pluck', 'nodice'})
        self.assertEqual(len(spec), 62)
        for name, (family, gargs) in spec.items():
            self.assertTrue(Path(gargs[1]).is_file(), name)
            self.assertEqual(gargs[0], 'dice2problog.py' if family == 'nodice' else 'alea_bench.jl')

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
        # leaf, which observes the evidence, its rows the queries' answers.
        asia = str(SOURCE / 'examples' / 'asia.net')
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
                r = bench.run_dice(self.args(dir=d, dice='dice {file}'), 'asia_ev', str(program))
            qs = [q.replace(' ', '') for q in bench.queries(str(program))]
        self.assertEqual(r['status'], 'ok', r)
        self.assertEqual(len(seen), len({q.split('(')[0] for q in qs}))
        self.assertTrue(all(text.count('observe') == len(evidence) for text in seen))
        self.assertEqual(sorted(r['answers']), sorted(qs))
        self.assertEqual(sorted(set(r['answers'].values())), [0.25, 0.75])

    def test_a_row_of_the_wrong_width_is_an_error(self):
        with tempfile.TemporaryDirectory() as d:
            Path(d, 'p.dicemap').write_text('p__d1.dice joint q1 3\n')
            result = {'status': 'ok', 'seconds': 0.1, 'stdout': 'Value\tProbability\t\n(true, false)\t1.\t\n',
                      'stderr': ''}
            with patch.object(bench, 'timed', return_value=result):
                r = bench.run_dice_circuit(self.args(dir=d, dice='dice {file}'), 'p', os.path.join(d, 'p.dicemap'))
        self.assertEqual(r['status'], 'error')

    def test_a_translated_system_answers_a_circuit_as_dice_does(self):
        name = next(n for n, (_, gargs) in bench.instances(self.args()).items() if gargs[0] == 'alea_bench.jl')
        got = {'a.dice': ([{'true': 0.25, 'false': 0.75}], {'seconds': 1.0}),
               'b.dice': ([{'0': 0.125, '1': 0.375, '2': 0.5, '3': 0.0}], {'seconds': 2.0,
                                                                         'stats': {'inproc_seconds': 0.5}})}
        calls = []

        def masses(args, label, source, joint, budget):
            calls.append((label, joint))
            return got[label]
        with tempfile.TemporaryDirectory() as d:
            Path(d, f'{name}.dicemap').write_text('a.dice bool q1\nb.dice joint q2 2\n')
            r = bench.run_translated(self.args(dir=d), name, None, 'sys', masses)
        self.assertEqual(calls, [('a.dice', False), ('b.dice', True)])
        self.assertEqual(r, {'status': 'ok', 'seconds': 3.0, 'stats': {'inproc_seconds': 0.5},
                             'answers': {'q1': 0.25, 'q2(0)': 0.125, 'q2(1)': 0.375, 'q2(2)': 0.5, 'q2(3)': 0.0}})

    def test_roulette_answers_a_circuit_through_its_dice_programs(self):
        # The joint program's two Booleans are asked as one int, its first
        # Boolean the most significant bit (dice2roulette.py --joint).
        name = next(n for n, (_, gargs) in bench.instances(self.args()).items() if gargs[0] == 'alea_bench.jl')
        outputs = {'a': 'cpu time: 5 real time: 250 gc time: 0\nroulette 1 true 0.25\nroulette 1 false 0.75\n',
                   'b': 'cpu time: 5 real time: 500 gc time: 0\nroulette 1 0 0.125\nroulette 1 1 0.375\n'
                        'roulette 1 2 0.5\nroulette 1 3 0.0\n'}
        runs = []

        def timed(cmd, timeout):
            if 'raco' in cmd:
                return {'status': 'ok', 'seconds': 9.0, 'stdout': '', 'stderr': ''}
            runs.append(Path(cmd[-1]).read_text())
            return {'status': 'ok', 'seconds': 1.5, 'stdout': outputs[Path(cmd[-1]).name[0]], 'stderr': ''}
        with tempfile.TemporaryDirectory() as d:
            Path(d, f'{name}.dicemap').write_text('a.dice bool q1\nb.dice joint q2 2\n')
            Path(d, 'a.dice').write_text('let x = flip 0.25 in x\n')
            Path(d, 'b.dice').write_text('let x = flip 0.5 in let y = flip 0.25 in (x, y)\n')
            args = self.args(dir=d, roulette='racket {file}')
            with patch.object(bench, 'timed', side_effect=timed):
                r = bench.run_roulette(args, name, None, 'rsdd')
        self.assertEqual(r, {'status': 'ok', 'seconds': 3.0, 'stats': {'inproc_seconds': 0.75},
                             'answers': {'q1': 0.25, 'q2(0)': 0.125, 'q2(1)': 0.375, 'q2(2)': 0.5, 'q2(3)': 0.0}})
        self.assertIn('(report-bool 1 ', runs[0])
        self.assertRegex(runs[1], r'\(query \(list \(cadr (\S+)\) \(car \1\)\) complement\?\)')
        self.assertIn('(report-int 1 (list-ref pmfs 0) 2)', runs[1])

    def test_a_program_the_language_cannot_express_is_unsupported(self):
        refused = SimpleNamespace(returncode=bench.REFUSED, stdout='', stderr='cannot express the program: recursion')
        with patch.object(bench.subprocess, 'run', return_value=refused):
            out, r = bench.translation(self.args(), 'dice2genfer.py', ['p.dice', '--components'])
        self.assertIsNone(out)
        self.assertEqual(r['status'], 'unsupported')
        self.assertIn('recursion', r['error'])
        self.assertEqual(bench.fmt(r), 'n/a')

    def test_alea_runs_the_program_and_reports_three_times(self):
        out = ('alea_q q1(0) 0.25\nalea_value q1(0) 3\nalea_q q1(1) 0.75\nalea_seconds 12.5\n'
               'alea_compile_seconds 12.25\nalea_warm_seconds 0.0625\nalea_bdd_evidence 7\n'
               'alea_bdd_query_max 5\nalea_bdd_joint_max 9\nalea_bdd_peak 40\n')
        args = self.args(alea='julia --project=A {file}', timeout=30)
        with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 20.0, 'stdout': out,
                                                        'stderr': ''}) as timed:
            r = bench.run_alea(args, 'arith_book', 'bench/arith_book.pl')
        cmd = timed.call_args.args[0]
        self.assertEqual(cmd[:2], ['julia', '--project=A'])
        self.assertTrue(cmd[2].endswith('alea_bench.jl'))
        self.assertEqual(cmd[3:6], ['run', '--warm-below', '10'])
        self.assertTrue(cmd[6].endswith('data/alea/arith/book.jl'))
        self.assertEqual(r['answers'], {'q1(0)': 0.25, 'q1(1)': 0.75})
        self.assertEqual(r['stats'], {
            'inproc_seconds': 12.5, 'jit_seconds': 12.25, 'warm_seconds': 0.062, 'bdd_evidence': 7,
            'bdd_query_max': 5, 'bdd_joint_max': 9, 'bdd_peak': 40,
            'printed': ['alea_value q1(0) 3', 'alea_seconds 12.5', 'alea_compile_seconds 12.25',
                        'alea_warm_seconds 0.0625', 'alea_bdd_evidence 7', 'alea_bdd_query_max 5',
                        'alea_bdd_joint_max 9', 'alea_bdd_peak 40']})

    def test_gen_writes_alea_circuits_only_under_alea(self):
        with self.assertRaisesRegex(SystemExit, 'under --alea'):
            bench.gen(self.args(only=['arith_book'], cnf=False))
        with self.assertRaisesRegex(SystemExit, 'none is selected'):
            bench.gen(self.args(only=['nodice_survey'], cnf=False, alea='julia {file}'))
        with tempfile.TemporaryDirectory() as d, patch.object(bench.subprocess, 'run') as run, \
                redirect_stdout(io.StringIO()):
            bench.gen(self.args(only=['hybit_pi'], cnf=False, alea='julia {file}', dir=d))
        cmd = run.call_args.args[0]
        self.assertEqual(cmd[2:], ['export', str(scripts.parent / 'data' / 'alea' / 'hybit' / 'pi.jl'),
                                   os.path.abspath(os.path.join(d, 'hybit_pi'))])

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

    def test_roulette_runs_the_artifacts_programs(self):
        args = self.args()
        data = bench.ROULETTE_DATA
        self.assertEqual(bench.roulette_program(args, 'rl_caesar_25', 'rsdd'), (f'{data}/caesar.rkt', ['25']))
        self.assertEqual(bench.roulette_program(args, 'rl_caesar_25', 'nocomplement'),
                         (f'{data}/caesar.rkt', ['25', '--no-complement']))
        # The complement applies to caesar's query alone.
        self.assertIsNone(bench.roulette_program(args, 'rl_motivating_1000', 'nocomplement'))
        self.assertEqual(bench.roulette_program(args, 'rl_hmm_smoothing_joint_100', 'rsdd'),
                         ('rb/hmm.rkt', ['roulette', 'smoothing-joint', '100']))
        self.assertEqual(bench.roulette_program(args, 'rl_bn_alarm', 'rsdd'), (f'{data}/bayes.rkt', ['alarm']))
        for program in ['motivating', 'diamond', 'ladder', 'caesar', 'network', 'hardware', 'bayes']:
            self.assertTrue(Path(data, f'{program}.rkt').exists(), program)

    def run_roulette(self, name, path, out):
        with patch.object(bench, 'timed', return_value={'status': 'ok', 'seconds': 3.0, 'stdout': out,
                                                        'stderr': ''}) as timed:
            r = bench.run_roulette(self.args(), name, path, 'rsdd')
        return r, timed.call_args[0][0]

    def test_the_answers_are_keyed_by_the_queries(self):
        path = str(SOURCE / 'tests' / 'roulette' / 'network-2.pl')
        out = ('cpu time: 60 real time: 61 gc time: 3\nroulette_q other(#t) 0.0\n'
               'roulette_q m(2,15) 0.98\nroulette_q m(2,16) 0.01\n')
        r, cmd = self.run_roulette('rl_network_10', path, out)
        self.assertEqual(cmd, ['racket', f'{bench.ROULETTE_DATA}/network.rkt', '10'])
        self.assertEqual(r['status'], 'ok')
        self.assertEqual(list(r['answers']), [f'm(2,{v})' for v in range(17)])
        # A value Roulette leaves out has probability 0.
        self.assertEqual((r['answers']['m(2,15)'], r['answers']['m(2,16)'], r['answers']['m(2,0)']),
                         (0.98, 0.01, 0.0))
        self.assertEqual(r['stats'], {'inproc_seconds': 0.061, 'other': {'other(#t)': 0.0}})
        # hmm.rkt prints its one answer on its last line.
        path = str(SOURCE / 'tests' / 'roulette' / 'hmm-smoothing-joint-4.pl')
        r, cmd = self.run_roulette('rl_hmm_smoothing_joint_100', path, 'cpu time: 5 real time: 5 gc time: 0\n0.25\n')
        self.assertEqual(r['answers'], {'q': 0.25})
        # A network's values are named as gen_bn.py names them.
        with tempfile.TemporaryDirectory() as d:
            path = Path(d, 'rl_bn_water.pl')
            path.write_text('query(cbodd_12_45(v_0_5)).\nquery(cbodd_12_45(v_1)).\n')
            r, _ = self.run_roulette('rl_bn_water', str(path), 'roulette_v 0.5 0.75\nroulette_v 1 0.25\n')
        self.assertEqual(r['answers'], {'cbodd_12_45(v_0_5)': 0.75, 'cbodd_12_45(v_1)': 0.25})
        r, _ = self.run_roulette('rl_hmm_smoothing_joint_100', str(SOURCE / 'tests' / 'roulette' /
                                                                    'hmm-smoothing-joint-4.pl'), 'error: x\n')
        self.assertEqual(r['status'], 'error')

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
        with self.assertRaisesRegex(SystemExit, 'translated Dice programs, and none is selected'):
            bench.run(run_args(roulette_variants=['rsdd', 'nocomplement']))

        def parsed(cmd, **given):
            fields = dict(cmd=cmd, suite='roulette', ace_benchmarks=None, uai_benchmarks=None,
                          biomine_benchmarks=None, roulette_benchmarks=None, ace='',
                          ace_variants=None, ace_heap=None, ace_java=None, sharpsat_td='', sharpsat_td_decot=None,
                          roulette='', roulette_variants=None)
            return SimpleNamespace(**(fields | given))
        with self.assertRaisesRegex(SystemExit, '--roulette-variants without --roulette'):
            bench.ace_defaults(parsed('run', roulette_variants=['nocomplement']))
        with self.assertRaisesRegex(SystemExit, 'and the suite is ppl'):
            bench.ace_defaults(parsed('gen', suite='ppl', roulette_benchmarks='rb'))
        args = parsed('run', roulette='racket {file}')
        bench.ace_defaults(args)
        self.assertEqual(args.roulette_variants, ['rsdd'])
        self.assertTrue(args.roulette_benchmarks.endswith('roulette-benchmarks'))

    def test_the_reference_is_the_generators(self):
        self.assertEqual(bench.EXACT['gen_roulette.py'][0], 'dp')
        self.assertEqual(bench.EXACT['gen_roulette.py'][1]('x.pl'), ['gen_roulette.py', '--reference-of', 'x.pl'])


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
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
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


class SuiteTests(unittest.TestCase):
    def test_large_instances_are_selected_and_recorded(self):
        args = SimpleNamespace(suite='indicators', only=['grid40'], families=[],
                               strategies=['closed'], kbs=[], pita='', aspmc='', scallop='', dice='',
                               dir='bench', out='results.jsonl', tddlog='tddlog', timeout=1, no_skip=True,
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

    def test_an_observation_with_a_body_is_refused_before_launch(self):
        with patch('builtins.open', mock_open(read_data='0.5::obs. evidence(obs) :- true.')), \
                patch.object(bench.os, 'makedirs'), patch.object(bench, 'timed') as run:
            result = bench.run_aspmc(SimpleNamespace(dir='bench', aspmc='python3', timeout=1),
                                     'coin', 'coin.pl')
            self.assertEqual(result['status'], 'error')
            run.assert_not_called()

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

    def test_is_is_written_as_clingos_equality(self):
        program = ('step(0).\nstep(T1) :- step(T), T < 9, T1 is T + 1.\n'
                   'd(T1) :- step(T), T1 is T+1, r(T).\nquery(this(is)).\n')
        written = ('step(0).\nstep(T1) :- step(T), T < 9, T1 = T + 1.\n'
                   'd(T1) :- step(T), T1 = T+1, r(T).\nquery(this(is)).\n')
        self.assertEqual(bench.aspmc_is(program), written)

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
                patch.object(bench, 'timed', return_value={
                    'status': 'ok', 'seconds': 0.1,
                    'stderr': '[RESULT] aspmc: evidence: 0.5',
                }) as run:
            result = bench.run_aspmc(args, 'coin', 'coin.pl')
            self.assertEqual(result['answers'], {'evidence': 0.5})
            run.assert_called_once()


# Ace's marginals file for examples/asia.net with smoke=yes and xray=yes, as
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

    def test_the_manifest_names_the_bnlearn_networks_by_their_sha256(self):
        files = bench.fetched()
        self.assertEqual(len(files), 22)
        for path, (sha, url) in files.items():
            self.assertRegex(sha, '^[0-9a-f]{64}$')
            self.assertTrue(url.startswith('https://'), url)
            self.assertIn(os.path.relpath(os.path.dirname(path), bench.ROOT), ['data/bn', 'data/bn-large'])

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
        network = bench.gen_bn.parse(str(SOURCE / 'examples' / 'asia.net'))
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
        markov.pl (no evidence but the factors'): tddlog's P(e) line
        (65/192) and the clique tree's, run for real, plus the program's
        offset (log10 960), and Merlin's PR file, each log10 325."""
        regress = SOURCE / 'tests' / 'regress'
        calls = []
        real = bench.timed
        tddlog_out = f'P(e):\t3.385416666666667e-1\tlog10 {bench.math.log10(65 / 192)!r}\n'

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
        self.assertEqual(calls[0][-5:-1], ['--vtree', 'auto', '--evidence-probability', '--no-queries'])
        self.assertEqual(calls[2][-2:], [str(Path(d) / 'uai_markov.pl'), '--evidence-probability'])
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
        tree's."""
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
            # Ace reads the .inst files, which gen --pr does not write; Scallop approximates
            pr_dir = Path(d) / 'pr'
            with redirect_stdout(io.StringIO()):
                bench.gen(SimpleNamespace(**{**selection, 'dir': str(pr_dir)}, pr=True))
            with self.assertRaisesRegex(SystemExit, r'has none \(gen --pr writes none\)'):
                bench.run(SimpleNamespace(**{**vars(args), 'dir': str(pr_dir), 'kbs': []}))
            with self.assertRaisesRegex(SystemExit, "--scallop's top-k proofs"):
                bench.run(SimpleNamespace(**{**vars(args), 'scallop': 'scli'}))


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
        args = SimpleNamespace(suite='standard', only=only, families=[], strategies=['closed'], kbs=[],
                               pita='', aspmc='', scallop='', dice='', dir='bench', out='results.jsonl',
                               tddlog='tddlog', timeout=1, no_skip=no_skip, exact=False, ace='', merlin='', ganak='',
                               pr=False, cpus=cpus)
        with patch('builtins.open', mock_open()) as output, patch.object(bench, 'timed', timed), \
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

    def test_default_table_separates_scallop_approximation(self):
        systems = [('tddlog', 'auto'), ('problog', 'ddnnf'), ('scallop', 'topk10')]
        rows = {('dice_twocoins', system, variant): {
                    'system': system, 'variant': variant, 'status': 'ok', 'seconds': 1,
                    'answers': {'firstcoin': 0.5 if system == 'scallop' else 1 / 3},
                } for system, variant in systems}
        output = io.StringIO()
        with patch.object(sys, 'argv', ['bench.py', 'table', 'results.jsonl']), \
                patch.object(bench, 'load', return_value=(rows, systems)), redirect_stdout(output):
            bench.main()
        self.assertIn('approximate: max diff', output.getvalue())
        self.assertIn('| 0.0e+00 (problog:ddnnf) | 1.7e-01 |', output.getvalue())

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


if __name__ == '__main__':
    unittest.main()
