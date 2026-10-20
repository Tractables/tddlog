"""gen_biomine.py and reliability_exact.py on a made-up graph in Biomine's
format (tests/lit/biomine_toy.bmg): the generator writes the programs
in tests/lit/ again, the union of the best paths grows to the whole
component, and the reference answers as ProbLog does and as enumeration
of the edges does.

tests/it/examples.rs checks tddlog's answers on the same programs against the
same ProbLog answers (ProbLog 2.3.0, `-k ddnnf`, one query at a time).
"""
import itertools
from pathlib import Path
import random
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
lit = root / 'tests' / 'lit'
sys.path.insert(0, str(scripts))
import reliability_exact  # noqa: E402

GRAPH = lit / 'biomine_toy.bmg'
QUERIES = ['path(phenotype_mim_104300,gene_entrezgene_351)', 'path(phenotype_mim_104300,gene_entrezgene_323)',
           'path(phenotype_mim_104300,gene_entrezgene_590)', 'path(phenotype_mim_104300,gene_entrezgene_5126)']
# program -> (K, ProbLog's answers in the order of QUERIES)
EXAMPLES = {
    'biomine3.pl': (3, [0.8622505, 0.9107231, 0.489378, 0.4204916]),
    'biomine.pl': (100, [0.89274736, 0.93024004, 0.57842804, 0.46925437]),
}


def brute_force(edges, s, t):
    """P(s and t connected) by enumerating the edges' worlds."""
    keys = list(edges)
    total = 0.0
    for bits in itertools.product([False, True], repeat=len(keys)):
        adj, w = {}, 1.0
        for on, key in zip(bits, keys):
            w *= edges[key] if on else 1.0 - edges[key]
            if on:
                a, b = tuple(key)
                adj.setdefault(a, set()).add(b)
                adj.setdefault(b, set()).add(a)
        seen, todo = {s}, [s]
        while todo:
            for x in adj.get(todo.pop(), ()):
                if x not in seen:
                    seen.add(x)
                    todo.append(x)
        total += w if t in seen else 0.0
    return total


class BiomineTests(unittest.TestCase):
    def test_the_generator_writes_the_programs(self):
        for name, (k, _) in EXAMPLES.items():
            with self.subTest(name=name):
                out = subprocess.run([sys.executable, str(scripts / 'gen_biomine.py'), str(GRAPH), str(k)],
                                     capture_output=True, text=True, check=True).stdout
                self.assertEqual(out, (lit / name).read_text())

    def test_enough_paths_give_the_component(self):
        # The source's component has 13 links between 8 nodes once the two
        # links of P1 and P2 are one edge and the loop and the link that
        # never holds are dropped; GO:0000001 lies on no path to a gene.
        text = (lit / 'biomine.pl').read_text()
        edges, _ = reliability_exact.parse(text)
        self.assertEqual(len(edges), 13)
        self.assertAlmostEqual(edges[frozenset(('protein_uniprot_p1', 'protein_uniprot_p2'))], 0.82, delta=1e-12)

    def test_the_reference_answers_as_problog_and_enumeration(self):
        for name, (_, expected) in EXAMPLES.items():
            with self.subTest(name=name):
                text = (lit / name).read_text()
                answers, qs = reliability_exact.answers(text)
                self.assertEqual(qs, QUERIES)
                edges, pairs = reliability_exact.parse(text)
                for (q, s, t), p in zip(pairs, expected):
                    self.assertAlmostEqual(answers[q], p, delta=1e-7, msg=q)
                    self.assertAlmostEqual(answers[q], brute_force(edges, s, t), delta=1e-12, msg=q)

    def test_the_reference_on_random_graphs(self):
        # Series, parallel and degree-one reductions and the frontier
        # program against enumeration, on graphs with cycles, parallel
        # edges and parts that s does not reach.
        for seed in range(60):
            rng = random.Random(seed)
            n, m = rng.randint(2, 8), rng.randint(1, 13)
            edges = {}
            for _ in range(m):
                a, b = rng.sample(range(n), 2)
                key = frozenset((str(a), str(b)))
                edges[key] = 1.0 - (1.0 - edges.get(key, 0.0)) * (1.0 - round(rng.uniform(0.05, 1.0), 2))
            for t in range(1, n):
                with self.subTest(seed=seed, t=t):
                    got = reliability_exact.connected(reliability_exact.reduce(edges, '0', str(t)), '0', str(t))
                    self.assertAlmostEqual(got, brute_force(edges, '0', str(t)), delta=1e-12)


if __name__ == '__main__':
    unittest.main()
