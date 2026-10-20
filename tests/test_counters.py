"""The weighted model counters Ganak and sharpSAT-TD on the weighted CNFs
of the networks of tests/regress (uai_wcnf.py, through `gen_bn.py
--cnf`): with GANAK set to a Ganak binary in the environment, Ganak's
count of the CNFs `bench.py gen` writes, through `bench.py run --pr
--exact --ganak`, against the clique tree's (bn_exact.py) and against Z
by enumeration; likewise with SHARPSAT_TD set to a sharpSAT-TD binary
that has flow_cutter_pace17 beside it. Each is skipped without its
binary.
"""
from contextlib import redirect_stdout
import io
import json
import math
import os
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest
import unittest.mock

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
regress = root / 'tests' / 'regress'
sys.path.insert(0, str(scripts))
sys.path.insert(0, str(root / 'tests'))
import bench  # noqa: E402
from test_gen_bn import brute_force_z  # noqa: E402
from test_uai_wcnf import scratch_dir  # noqa: E402


class CounterTests(unittest.TestCase):
    """The counters on the CNFs `bench.py gen` writes for tests/regress's
    networks, through `bench.py run --pr --exact`: each one's log10 Z and
    the clique tree's P(e) plus the program's offset, against Z by
    enumeration. Each runs where its binary is given in the environment."""

    def run_counter(self, **given):
        import resource
        with scratch_dir('uai-counter-') as d:
            args = SimpleNamespace(suite='uai', only=[], families=[], uai_benchmarks=str(regress), dir=d, cnf=True)
            with redirect_stdout(io.StringIO()):
                bench.gen(args)
                args = SimpleNamespace(**vars(args), strategies=[], kbs=[], pita='', aspmc='', scallop='', dice='',
                                       exact=True, ace='', merlin='', ganak='', sharpsat_td='', pr=True,
                                       out=str(Path(d) / 'results.jsonl'), timeout=60, no_skip=True)
                for k, v in given.items():
                    setattr(args, k, v)
                # sharpSAT-TD's cache is half the address-space limit
                # less 500 MB (bench.sharpsat_td_cache_mb).
                with unittest.mock.patch.object(resource, 'getrlimit', return_value=(4_000_000_000,) * 2):
                    bench.run(args)
            records = [json.loads(line) for line in Path(args.out).read_text().splitlines()]
        z = {'uai_markov': brute_force_z((regress / 'markov.uai').read_text(),
                                         (regress / 'markov.uai.evid').read_text()),
             'uai_markov_pr': brute_force_z((regress / 'markov_pr.uai').read_text(),
                                            (regress / 'markov_pr.uai.evid').read_text())}
        return records, z

    @unittest.skipUnless(os.environ.get('GANAK'), 'GANAK, a Ganak binary, is not set')
    def test_ganak_counts_what_the_clique_tree_does(self):
        records, z = self.run_counter(ganak=os.environ.get('GANAK'))
        self.assertEqual(sorted((r['instance'], r['system']) for r in records),
                         [(n, s) for n in sorted(z) for s in ['exact', 'ganak']])
        for r in records:
            self.assertEqual(r['status'], 'ok', r)
            self.assertAlmostEqual(r['log10_z'], math.log10(z[r['instance']]), delta=1e-12, msg=r)

    @unittest.skipUnless(os.environ.get('SHARPSAT_TD'), 'SHARPSAT_TD, a sharpSAT-TD binary with '
                         'flow_cutter_pace17 beside it, is not set')
    def test_sharpsat_td_counts_what_the_clique_tree_does(self):
        records, z = self.run_counter(sharpsat_td=os.environ.get('SHARPSAT_TD'), sharpsat_td_decot=['1'])
        self.assertEqual(sorted((r['instance'], r['system']) for r in records),
                         [(n, s) for n in sorted(z) for s in ['exact', 'sharpsat-td']])
        for r in records:
            self.assertEqual(r['status'], 'ok', r)
            self.assertAlmostEqual(r['log10_z'], math.log10(z[r['instance']]), delta=1e-12, msg=r)


if __name__ == '__main__':
    unittest.main()
