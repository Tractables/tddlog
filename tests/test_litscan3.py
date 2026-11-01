"""The families of the third literature scan:
lit_exact.py answers ProbLog2's probabilistic grid as an enumeration of the
grid's edges does and PITA's three-sided die as a forward recursion over the
throws does, reads the answers a ProbLog example states, and
gen_problogtut.py takes a tutorial page's programs that ask marginals.
"""
import itertools
import math
from pathlib import Path
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
import bench  # noqa: E402
import gen_problogtut  # noqa: E402
import lit_exact  # noqa: E402


def generate(script, args):
    return subprocess.run([sys.executable, str(scripts / script)] + args,
                          capture_output=True, text=True, check=True).stdout


def reach(edges, present, source, target):
    seen, todo = {source}, [source]
    while todo:
        u = todo.pop()
        for (a, b), on in zip(edges, present):
            if on and a == u and b not in seen:
                seen.add(b)
                todo.append(b)
    return target in seen


class Litscan3Tests(unittest.TestCase):
    def test_pgrid_as_an_enumeration_of_the_edges(self):
        for n, d in [(2, 1), (3, 1), (3, 2)]:
            text = bench.folded_program(['gen_pgrid.py', str(n), str(d)])
            got = lit_exact.pgrid(text)
            edges = [((int(a), int(b)), (int(c), int(e))) for _, a, b, c, e in
                     lit_exact.facts(text, r'([0-9.]+)::edge\(n_(\d+)_(\d+),n_(\d+)_(\d+)\)\.')]
            i = n - d
            want = sum(0.5 ** len(edges) for present in itertools.product((0, 1), repeat=len(edges))
                       if reach(edges, present, (i, i), (n, n)))
            self.assertEqual(list(got), [f'path(n_{i}_{i},n_{n}_{n})'])
            self.assertAlmostEqual(got[f'path(n_{i}_{i},n_{n}_{n})'], want, places=12)
        self.assertAlmostEqual(lit_exact.pgrid(bench.folded_program(['gen_pgrid.py', '2', '1']))['path(n_1_1,n_2_2)'],
                               0.71875)

    def test_noisyor_as_an_enumeration_of_the_diseases(self):
        # the findings observed absent are absorbed into their causes' priors
        text = generate('gen_noisyor.py', ['7', '10', '--seed', '3'])
        prior = {d: float(p) for p, d in lit_exact.facts(text, r'([0-9.]+)::disease\((\w+)\)\.')}
        leak = {f: float(p) for p, f in lit_exact.facts(text, r'([0-9.]+)::leak\((\w+)\)\.')}
        causes = {}
        for p, d, f in lit_exact.facts(text, r'([0-9.]+)::cause\((\w+),(\w+)\)\.'):
            causes.setdefault(f, []).append((d, float(p)))
        seen = {f: v == 'true' for f, v in lit_exact.facts(text, r'evidence\(finding\((\w+)\),(true|false)\)\.')}
        self.assertIn(False, seen.values())
        self.assertIn(True, seen.values())
        names = sorted(prior)
        mass, total = dict.fromkeys(names, 0.0), 0.0
        for ill in itertools.product((False, True), repeat=len(names)):
            world = dict(zip(names, ill))
            weight = math.prod(prior[d] if world[d] else 1 - prior[d] for d in names)
            for f, q in leak.items():
                off = (1 - q) * math.prod(1 - p for d, p in causes.get(f, []) if world[d])
                weight *= (1 - off) if seen[f] else off
            total += weight
            for d in names:
                mass[d] += weight if world[d] else 0.0
        got = lit_exact.noisyor(text)
        for d in names:
            self.assertAlmostEqual(got[f'disease({d})'], mass[d] / total, places=12)

    def test_threedie_as_a_forward_recursion(self):
        for n in [0, 1, 5, 40]:
            # the probability that throws 0 to n - 1 show 1 or 2
            going = 1.0
            for _ in range(n):
                going *= 2 / 3
            got = lit_exact.threedie(bench.folded_program(['gen_threedie.py', str(n)]))
            self.assertTrue(math.isclose(got[f'on({n},1)'], going / 3, rel_tol=1e-12))

    def test_a_folded_family_is_one_program_and_its_arguments(self):
        # each program of bench.py's FOLD is one text for all its sizes, with
        # its place $1 outside its comments, and each size one argument
        programs = {}
        for name, (family, gargs) in bench.LIT.items():
            if gargs[0] not in bench.FOLD or (gargs[0] == 'gen_pcfg.py' and not (root / 'data' / 'problog-tutorial').exists()):
                continue
            program, text = bench.fold_template(gargs)
            programs.setdefault(program, set()).add(text)
            self.assertIn('$1', ''.join(line for line in text.splitlines() if not line.startswith('%')), name)
            self.assertEqual(len(bench.fold_arguments(gargs)), 1, name)
        self.assertTrue(all(len(texts) == 1 for texts in programs.values()), programs.keys())
        self.assertLessEqual({'threedie.pl', 'pgrid16.pl', 'pgrid18.pl', 'pgrid20.pl', 'dnahmm.pl', 'plcg.pl',
                              'hmmpos.pl'}, set(programs))

    def test_expected_reads_the_stated_answers(self):
        text = ('%Expected outcome:  \n% smokes(1) 0.5087719298245614\n% all 0\n\n'
                '0.3::p(1); 0.4::p(2).\nquery(smokes(X)).\n')
        self.assertEqual(lit_exact.expected(text), {'smokes(1)': 0.5087719298245614, 'all': 0.0})

    def test_problogtut_takes_the_editors_that_ask_marginals(self):
        page = ('<p>x</p><div class="problog-editor" data-task="prob">0.5::a.\nquery(a &amp; b).</div>'
                '<div class="problog-editor" data-task="lfi">t(_)::a.</div>')
        self.assertEqual(gen_problogtut.editors(page), [('prob', '0.5::a.\nquery(a & b).'), ('lfi', 't(_)::a.')])
        for name, (family, gargs) in bench.LIT.items():
            if family == 'problogtut':
                self.assertEqual(gargs[0], 'gen_problogtut.py')
                path = root / 'data' / 'problog-tutorial' / (gargs[1].rsplit('/', 1)[-1] + '.html')
                if path.exists():
                    found = gen_problogtut.editors(path.read_text(encoding='utf-8'))
                    self.assertEqual(found[int(gargs[2]) - 1][0], 'prob', name)
                    self.assertIn('query(', found[int(gargs[2]) - 1][1], name)

    def test_the_collections(self):
        count = lambda f: sum(1 for family, _ in bench.LIT.values() if family == f)  # noqa: E731
        self.assertEqual([count(f) for f in ['problogtut', 'problogex', 'cplintpita', 'threedie', 'pgrid']],
                         [115, 22, 38, 5, 10])
        for f in ['problogtut', 'problogex', 'cplintpita']:
            self.assertIsNone(bench.series(f, next(n for n, (g, _) in bench.LIT.items() if g == f)))
        for name in bench.PROBLOGEX:
            self.assertIn('Expected outcome', (root / 'data' / 'problog' / f'{name}.pl').read_text())
        for name, (family, gargs) in bench.LIT.items():
            if family == 'cplintpita':   # the LPAD tddlog and PITA read, in its own folder
                where = gargs[2] if len(gargs) > 2 else 'cplint'
                self.assertEqual(Path(bench.original(gargs, '')).resolve(), root / 'data' / where / f'{gargs[1]}.pl', name)
                self.assertTrue(Path(bench.original(gargs, '')).exists(), name)

    def test_aspmc_reads_the_ground_program_with_plain_names(self):
        # tddlog --ground's syntax: switches, plain facts and rules, ground
        # queries and evidence
        ground = ("0.5::'$sw0'(mary).\nthrows(john).\nthrows(mary) :- '$sw0'(mary).\n"
                  "brk :- throws(mary), \\+throws(john).\nquery(throws(john)).\nquery(throws(mary)).\n"
                  "query(brk).\nevidence(throws(mary),true).\n")
        program, names = bench.aspmc_ground(ground)
        self.assertEqual(program.split('\n'), [
            '0.5::p0(c0).', 'p1(c1).', 'p1(c0) :- p0(c0).', 'p2 :- p1(c0), \\+p1(c1).', 'query(p1(c1)).',
            'query(p1(c0)).', 'query(p2).', 'evidence(p1(c0),true).', ''])
        self.assertEqual(names['p1(c0)'], 'throws(mary)')
        self.assertEqual(names['p2'], 'brk')
        self.assertIn('evidence(p0,false).', bench.aspmc_ground('0.5::a.\nevidence(\\+a).\n')[0])
        # what aspmc printed on the scan's programs it refused
        for err in ["lark.exceptions.UnexpectedCharacters: No terminal matches '[' in the current parser context",
                    '<block>:9:1-34: error: unsafe variables in:\n  query_atom(throws(#Anon0))',
                    'RuntimeError: grounding stopped because of errors',
                    '<block>:7:37-40: error: syntax error, unexpected not']:
            self.assertTrue(bench.ASPMC_REFUSED.search(err), err)
        self.assertIsNone(bench.ASPMC_REFUSED.search('Killed'))



if __name__ == '__main__':
    unittest.main()
