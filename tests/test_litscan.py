"""The families of the second literature scan:
lit_exact.py answers small programs of eventcalc, liftsep and rgraph as
ProbLog does, liftsep and rgraph as an enumeration of the worlds does,
plcg as its count of parses gives, hmmpos as an enumeration of its
sequences of tags does and hwf as an enumeration of its formulas does;
gen_taint.py rewrites the two constructs of PSouffle's programs that tddlog
does not read, and gen_sidechannel.py reads Souffle's equations as Souffle
does.
"""
import gzip
import itertools
import math
from pathlib import Path
import re
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
sys.path.insert(0, str(scripts))
import bench  # noqa: E402  (the folded families' programs)
import lit_exact  # noqa: E402


def generate(family, args):
    return subprocess.run([sys.executable, str(scripts / f'gen_{family}.py')] + args,
                          capture_output=True, text=True, check=True).stdout


# (family, generator arguments) -> ProbLog's answers (ProbLog 2.3.0, -k
# ddnnf), one query at a time; ProbLog answers a query not listed below
# 1e-7.
EXAMPLES = {
    ('liftsep', ('2',)): [('qw', 0.82104582), ('q9', 0.77120895)],
    ('liftsep', ('3',)): [('qw', 0.50754219), ('q9', 0.40560261)],
    ('rgraph', ('5',)): [('path2', 0.20110309)],
    ('rgraph', ('7',)): [('path2', 0.25688277)],
    ('eventcalc', ('2', '400', '--every', '25', '--seed', '2')): [
        ('holds_meeting(p1,p0,100)', 2.7245385e-05),
        ('holds_meeting(p0,p1,275)', 1.0),
        ('holds_moving(p0,p1,275)', 0.0196),
        ('holds_meeting(p1,p0,275)', 1.0),
        ('holds_moving(p1,p0,275)', 0.0196),
        ('holds_meeting(p0,p1,300)', 1.0),
        ('holds_moving(p0,p1,300)', 0.0196),
        ('holds_meeting(p1,p0,300)', 1.0),
        ('holds_moving(p1,p0,300)', 0.0196),
        ('holds_meeting(p0,p1,325)', 1.0),
        ('holds_moving(p0,p1,325)', 0.0196),
        ('holds_meeting(p1,p0,325)', 1.0),
        ('holds_moving(p1,p0,325)', 0.0196),
        ('holds_meeting(p0,p1,375)', 1.991096e-05),
    ],
    ('eventcalc', ('2', '400', '--every', '25', '--seed', '4')): [
        ('holds_meeting(p0,p1,100)', 0.99927381),
        ('holds_meeting(p1,p0,100)', 0.99999578),
        ('holds_meeting(p0,p1,125)', 1.0),
        ('holds_meeting(p1,p0,125)', 1.0),
        ('holds_meeting(p0,p1,150)', 0.0097008388),
        ('holds_meeting(p1,p0,150)', 0.0097008388),
        ('holds_meeting(p0,p1,175)', 0.0097008388),
        ('holds_meeting(p1,p0,175)', 0.0097008388),
        ('holds_meeting(p0,p1,200)', 0.00042971379),
        ('holds_meeting(p1,p0,200)', 0.00042971379),
        ('holds_meeting(p0,p1,225)', 0.00042971379),
        ('holds_meeting(p1,p0,225)', 0.00042971379),
    ],
}


def worlds(probs):
    """Each assignment of the independent facts `probs` ({fact: p}) with its
    probability."""
    facts = list(probs)
    for bits in itertools.product((False, True), repeat=len(facts)):
        yield {f for f, b in zip(facts, bits) if b}, math.prod(probs[f] if b else 1 - probs[f]
                                                             for f, b in zip(facts, bits))


class LitscanTests(unittest.TestCase):
    def test_the_reference_answers_as_problog(self):
        for (family, args), expected in EXAMPLES.items():
            with self.subTest(family=family, args=args):
                text = generate(family, list(args))
                answers = lit_exact.FAMILIES[family](text)
                expected = dict(expected)
                self.assertLessEqual(set(expected), set(lit_exact.queries(text)))
                for q in lit_exact.queries(text):
                    self.assertAlmostEqual(answers[q], expected.get(q, 0.0), delta=1e-7, msg=q)

    def test_liftsep_as_enumeration(self):
        text = generate('liftsep', ['2', '--seed', '3'])
        probs = {}
        for p, rel, args in re.findall(r'^([0-9.]+)::(\w+)\((\w+(?:,\w+)?)\)\.$', text, re.M):
            probs[rel, tuple(args.split(','))] = float(p)
        dom = ['c0', 'c1']
        pairs = list(itertools.product(dom, dom))
        answers = {'qw': 0.0, 'q9': 0.0}
        for world, w in worlds(probs):
            h = [any(('r', (x,)) in world and ('s1', (x, y)) in world for x, y in pairs),
                 any(('s1', (x, y)) in world and ('s2', (x, y)) in world for x, y in pairs),
                 any(('s2', (x, y)) in world and ('s3', (x, y)) in world for x, y in pairs),
                 any(('s3', (x, y)) in world and ('t', (y,)) in world for x, y in pairs)]
            answers['qw'] += w * ((h[0] or h[2]) and (h[0] or h[3]) and (h[1] or h[3]))
            answers['q9'] += w * ((h[0] or h[3]) and (h[1] or h[3]) and (h[2] or h[3]) and (h[0] or h[1] or h[2]))
        got = lit_exact.liftsep(text)
        for q, p in answers.items():
            self.assertAlmostEqual(got[q], p, delta=1e-12, msg=q)

    def test_rgraph_as_enumeration(self):
        for n in [3, 4, 6]:
            with self.subTest(n=n):
                text = generate('rgraph', [str(n), '--seed', '2'])
                probs = {(int(x), int(y)): float(p) for p, x, y in
                         re.findall(r'^([0-9.]+)::e\((\d+),(\d+)\)\.$', text, re.M)}
                path = sum(w for world, w in worlds(probs)
                           if any(sum(v in e for e in world) >= 2 for v in range(n)))
                self.assertAlmostEqual(lit_exact.rgraph(text)['path2'], path, delta=1e-12)

    def test_plcg_as_its_count_of_parses(self):
        """A parse of n words chooses each word (0.5) and attach or project
        at each of its 2n - 1 rules (0.5), and a word has Catalan(n - 1)
        parses."""
        for n in range(1, 9):
            with self.subTest(n=n):
                text = bench.folded_program(['gen_plcg.py', str(n), '--seed', str(n)])
                (q,) = lit_exact.queries(text)
                self.assertRegex(q, r'^plc\(\[[ab](,[ab]){%d}\]\)$' % (n - 1))
                catalan = math.comb(2 * (n - 1), n - 1) // n
                self.assertAlmostEqual(lit_exact.plcg(text)[q] / (catalan / 2 ** (3 * n - 1)), 1.0, delta=1e-12)

    def test_hmmpos_as_enumeration(self):
        """The sum over every sequence of tags of the product of its
        choices, where each tag allows its word."""
        text = bench.folded_program(['gen_hmmpos.py', '1'])
        self.assertEqual(len(re.findall(r'::trans\(\w+,\w+,S\)', text)), 30)
        self.assertNotIn(',_)', ''.join(x for x in text.splitlines() if '::trans(' in x))
        trans = {}
        for p, q, r in re.findall(r'([0-9.]+)::trans\((\w+),(\w+),S\)', text):
            trans[q, r] = float(p)
        allows = {'he': {'pron'}, 'can': {'aux', 'v', 'n'}, 'a': {'det'}}
        tags = ['det', 'aux', 'v', 'n', 'pron']
        words = ['he', 'can', 'can', 'a', 'can']
        total = 0.0
        for seq in itertools.product(tags, repeat=len(words)):
            if all(t in allows[x] for t, x in zip(seq, words)):
                total += math.prod(trans[a, b] for a, b in zip(('start',) + seq, seq))
        (q,) = lit_exact.queries(text)
        self.assertEqual(q, 'hmm([he,can,can,a,can])')
        self.assertAlmostEqual(lit_exact.hmmpos(text)[q], total, delta=1e-15)
        self.assertGreater(total, 0.0)

    def test_hwf_as_enumeration(self):
        """The sum over every sequence of symbols whose value, parse/2's
        as gen_hwf.py's value() follows its clauses, is within 1e-4 of the
        label; and the two modes' clauses written from the model's."""
        import gen_hwf
        for n in [1, 3, 5]:
            with self.subTest(n=n):
                text = generate('hwf', [str(n), '--seed', str(n)])
                num, op = {}, {}
                for p, img, d in re.findall(r'([0-9.]+)::detect_number\((\w+),(\d)\)', text):
                    num.setdefault(img, []).append((int(d), float(p)))
                for p, img, o in re.findall(r'([0-9.]+)::detect_operator\((\w+),([-+*/])\)', text):
                    op.setdefault(img, []).append((o, float(p)))
                (q,) = lit_exact.queries(text)
                images, label = re.fullmatch(r'expression\(\[([\w,]+)\],(-?[0-9.]+)\)', q).groups()
                images = images.split(',')
                self.assertEqual(len(images), n)
                choices = [num[img] if i % 2 == 0 else op[img] for i, img in enumerate(images)]
                total = 0.0
                for seq in itertools.product(*choices):
                    v = gen_hwf.value([s for s, _ in seq])
                    if v is not None and abs(v - float(label)) < 0.0001:
                        total += math.prod(p for _, p in seq)
                self.assertGreater(total, 0.0)
                self.assertAlmostEqual(lit_exact.hwf(text)[q], total, delta=1e-12)
        text = generate('hwf', ['3'])
        self.assertIn('almost_equal(X,Y) :- abs(X-Y) < 0.0001.\nalmost_equal_free(X,Y) :- Y is float(X).\n', text)
        self.assertIn('parse([N1,+|T], R) :-\n    parse_free(T,R2),\n    almost_equal(N1+R2,R).', text)
        self.assertIn('parse_free([N1,-|T], R) :-\n    parse_free([-1,*|T],R2),\n    almost_equal_free(N1+R2,R).',
                      text)
        self.assertIn('parse([N1,/,N2|T], R) :-\n    N2 \\== 0,\n    N3 is N1/N2,\n    parse([N3|T],R).', text)
        self.assertNotIn('nn(', text)
        self.assertNotRegex(text, r'\b(var|ground)\(')
        # The program of tddlog's test of the formulas (tests/it/terms.rs).
        hand = ('0.6::detect_number(i1,2); 0.4::detect_number(i1,3).\n'
                '0.4::detect_operator(i2,+); 0.1::detect_operator(i2,-); 0.3::detect_operator(i2,*); '
                '0.2::detect_operator(i2,/).\n'
                '0.7::detect_number(i3,3); 0.3::detect_number(i3,0).\n'
                + ''.join(f'query(expression([i1,i2,i3],{x})).\n' for x in ['5', '6', '0', '-1', '0.6667']))
        got = lit_exact.hwf(hand)
        for x, p in [('5', 0.6 * 0.4 * 0.7), ('6', 0.6 * 0.3 * 0.7 + 0.4 * 0.4 * 0.7),
                     ('0', 0.3 * 0.3 + 0.4 * 0.1 * 0.7), ('-1', 0.6 * 0.1 * 0.7), ('0.6667', 0.6 * 0.2 * 0.7)]:
            self.assertAlmostEqual(got[f'expression([i1,i2,i3],{x})'], p, delta=1e-12, msg=x)

    def test_taint_rewrites_what_tddlog_does_not_read(self):
        src = root / 'data' / 'psouffle' / 'taint' / 'angulo.pl.gz'
        with gzip.open(src, 'rt') as f:
            original = f.read()
        out = subprocess.run([sys.executable, str(scripts / 'gen_taint.py'), str(src)],
                             capture_output=True, text=True, check=True).stdout
        self.assertIn('\\+(I = 0)', original)
        self.assertNotIn('\\+(', out)
        self.assertIn('mmetharg(M, I, V), I \\= 0.', out)
        self.assertIn('\nci_im(0, 0) :- fail.\n', out)
        self.assertIn('\ncicm(0, 0, 0, 0) :- fail.\n', out)
        self.assertNotRegex(out, re.compile(r'^\w+\(_', re.M))
        back = re.sub(r'^(\w+)\((0(?:, 0)*)\) :- fail\.$', lambda m: f'{m[1]}({m[2].replace("0", "_")}) :- fail.',
                      out.replace('I \\= 0', '\\+(I = 0)'), flags=re.M)
        self.assertEqual(back, original)

    def test_sidechannel_reads_souffles_equations_as_souffle_does(self):
        """P1's equations evaluate their arithmetic (`is`), after the atoms
        that bind their variables and before the negations; its empty
        relations are declared, its output relations asked; and the
        case's input relations are written back for PSouffle as the
        repository has them."""
        import tempfile
        d = root / 'data' / 'psouffle' / 'side_channel'
        with tempfile.TemporaryDirectory() as tmp:
            out = subprocess.run([sys.executable, str(scripts / 'gen_sidechannel.py'), str(d / 'P1.souffle.dl.gz'),
                                  str(d / 'P1.problog.dl.gz'), '--souffle', tmp],
                                 capture_output=True, text=True, check=True).stdout
            lines = out.splitlines()
            self.assertIn('rec_rand_var(VAR_Z, TIMES, TMP) :- rec_rand_var(VAR_X, TIMES, X), '
                          'rec_rand_var(VAR_Y, TIMES, Y), bvand_rec(VAR_X, VAR_Y, VAR_Z, TIMES), TMP is X /\\ Y.',
                          lines)
            self.assertIn('set_sum_rec(VAR_X, VAR_Y, VAR_Z, TIMES_NEW) :- set_sum_rec(VAR_X, VAR_Y, VAR_Z, TIMES), '
                          'TIMES_NEW is TIMES - 1, TIMES_NEW \\= 0.', lines)
            self.assertIn('0.964874::rec_rand_var(VAR_Y, TIMES, XY) :- rec_rand_var(VAR_X, TIMES, X), '
                          'bv_zero_rec(VAR_X, VAR_Y, TIMES), XY = 0.', lines)
            self.assertIn('bvand_rec(0, 0, 0, 0) :- fail.', lines)
            self.assertIn('hd_sensitive(0, 0) :- fail.', lines)
            self.assertNotIn('share(0, 0) :- fail.', lines)
            self.assertEqual([x for x in lines if x.startswith('query(')],
                             ['query(key_ind(_)).', 'query(key_sensitive(_)).', 'query(hd_sensitive(_,_)).'])
            self.assertEqual(len([x for x in lines if re.fullmatch(r'([0-9.]+::)?\w+\([-0-9, ]*\)\.', x)]), 88)
            with gzip.open(d / 'P1.souffle.dl.gz', 'rt') as f:
                self.assertEqual((Path(tmp) / 'compute.dl').read_text(), f.read())
            self.assertEqual((Path(tmp) / 'input' / 'share.facts').read_text().splitlines()[:2], ['4\t9', '9\t3'])
            self.assertEqual((Path(tmp) / 'input' / 'share.prob').read_text().splitlines()[:2],
                             ['0.992483', '0.888557'])
            self.assertEqual(set((Path(tmp) / 'input' / 'load_assign.prob').read_text().split()), {'1.0'})
            self.assertEqual((Path(tmp) / 'input' / 'SET_SUM.facts').read_text(), '')
            self.assertEqual((Path(tmp) / 'names.tsv').read_text().splitlines()[0], 'KEY_IND\tkey_ind\t1')


if __name__ == '__main__':
    unittest.main()
