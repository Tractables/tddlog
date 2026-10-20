"""gen_tpch.py and tpch_exact.py: the generator writes tests/lit/tpch.pl
and tests/lit/tpch_views.pl again from the same arguments, the reference
answers the first as ProbLog does, every query written through views has
the lineage of the query written as one rule and of the reference, and the
reference's decomposition and core elimination answer random monotone
DNFs as enumeration does.

tests/it/examples.rs checks tddlog's answers on both programs against the
same ProbLog answers (ProbLog 2.3.0, `-k ddnnf`, one query at a time).
"""
import itertools
import math
from pathlib import Path
import random
import re
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
lit = root / 'tests' / 'lit'
sys.path.insert(0, str(scripts))
import tpch_exact  # noqa: E402

ARGS = ['0.0004', '--queries', 'q2,q3,q4,q6,q7,q10,q11,q12,q14,q15,q17', '--seed', '4']
# ProbLog's answers, one query at a time.
PROBLOG = [
    ('q2', 0.017751203),
    ('q3', 0.02461413),
    ('q4', 0.46913878),
    ('q6', 0.61112731),
    ('q7', 0.0155477),
    ('q10', 0.048477216),
    ('q11', 0.080382831),
    ('q12', 0.17013583),
    ('q14', 0.33707206),
    ('q15', 0.58742125),
    ('q17', 0.38035305),
]


def minimal(clauses):
    """The clauses of a monotone DNF that contain no other clause."""
    clauses = sorted(set(clauses), key=len)
    kept = []
    for c in clauses:
        if not any(k <= c for k in kept):
            kept.append(c)
    return frozenset(kept)


TERM = re.compile(r"\s*(?:'([^']*)'|(-?\d+)|([A-Z_]\w*)|([a-z]\w*))\s*")
COMPARE = re.compile(r'^(.+?)\s*(=<|>=|\\=|<|>)\s*(.+)$')


def term(text):
    """A term: ('var', name) for a variable (None for `_`), else ('const', value)."""
    m = TERM.fullmatch(text)
    quoted, number, var, name = m.groups()
    if var is not None:
        return ('var', None if var == '_' else var)
    return ('const', quoted if quoted is not None else int(number) if number is not None else name)


def literal(text):
    """A literal: ('atom', predicate, terms) or ('cmp', op, left, right)."""
    m = re.fullmatch(r'\s*(\w+)(?:\((.*)\))?\s*', text)
    if m:
        return ('atom', m.group(1), [term(t) for t in re.findall(r"'[^']*'|[^,]+", m.group(2) or '')])
    m = COMPARE.match(text.strip())
    return ('cmp', m.group(2), term(m.group(1)), term(m.group(3)))


def split_body(body):
    """The literals of a body, split at the commas outside parentheses."""
    parts, depth, start = [], 0, 0
    for i, ch in enumerate(body):
        depth += ch == '('
        depth -= ch == ')'
        if ch == ',' and depth == 0:
            parts.append(body[start:i])
            start = i + 1
    return parts + [body[start:]]


def lineages(text):
    """The lineage of every ground atom of a TPC-H program that holds in some
    world, by predicate: a monotone DNF over the probabilistic facts' ids,
    its clauses minimal, evaluated bottom up from the program's rules (which
    gen_tpch.py writes without recursion, each body's comparisons after the
    literals that bind them)."""
    rels, _ = tpch_exact.parse(text)
    holds = {r: {f: {frozenset([t])} for t, f in tuples} for r, tuples in rels.items()}
    rules = []
    for line in text.splitlines():
        if '::' in line or line.startswith(('%', 'query(')) or not line.strip():
            continue
        head, _, body = line.rstrip('.').partition(':-')
        rules.append((literal(head), [literal(x) for x in split_body(body)] if body else []))
    for (_, pred, args), body in rules:
        if not body:  # a deterministic fact
            holds.setdefault(pred, {})[tuple(a[1] for a in args)] = {frozenset()}
    defined = {head[1] for head, body in rules if body}
    done = set(holds) - defined
    while defined - done:
        for pred in sorted(defined - done):
            ready = all(lit[1] in done for head, body in rules if head[1] == pred for lit in body if lit[0] == 'atom')
            if not ready:
                continue
            out = {}
            for head, body in rules:
                if head[1] != pred or not body:
                    continue
                for env, clause in joins(body, holds, {}, frozenset()):
                    key = tuple(env[a[1]] if a[0] == 'var' else a[1] for a in head[2])
                    out.setdefault(key, set()).add(clause)
            holds[pred] = {k: set(minimal(v)) for k, v in out.items()}
            done.add(pred)
    return holds


def joins(body, holds, env, clause):
    """The bindings and lineage clauses of the derivations of `body`."""
    if not body:
        yield env, clause
        return
    lit, rest = body[0], body[1:]
    value = lambda t: env[t[1]] if t[0] == 'var' else t[1]  # noqa: E731
    if lit[0] == 'cmp':
        x, y = value(lit[2]), value(lit[3])
        if {'<': x < y, '>': x > y, '=<': x <= y, '>=': x >= y, '\\=': x != y}[lit[1]]:
            yield from joins(rest, holds, env, clause)
        return
    # The atoms of the predicate that agree with the first argument bound,
    # through an index on that argument.
    at = next((i for i, t in enumerate(lit[2]) if t[0] == 'const' or t[1] in env), None)
    if at is None:
        candidates = holds.get(lit[1], {}).items()
    else:
        index = holds.setdefault(('index', lit[1], at), {})
        if not index:
            for fields, dnf in holds.get(lit[1], {}).items():
                index.setdefault(fields[at], []).append((fields, dnf))
        candidates = index.get(value(lit[2][at]), [])
    for fields, dnf in candidates:
        bound = dict(env)
        if len(fields) != len(lit[2]):
            continue
        for t, f in zip(lit[2], fields):
            if t[0] == 'const':
                if t[1] != f:
                    break
            elif t[1] is not None:
                if bound.setdefault(t[1], f) != f:
                    break
        else:
            for c in dnf:
                yield from joins(rest, holds, bound, clause | c)


def brute_force(clauses, p):
    tuples = sorted({v for c in clauses for v in c})
    total = 0.0
    for bits in itertools.product([False, True], repeat=len(tuples)):
        world = {v for v, on in zip(tuples, bits) if on}
        if any(c <= world for c in clauses):
            total += math.prod(p[v] if v in world else 1.0 - p[v] for v in tuples)
    return total


class TpchTests(unittest.TestCase):
    def test_the_generator_writes_the_program(self):
        out = subprocess.run([sys.executable, str(scripts / 'gen_tpch.py')] + ARGS,
                             capture_output=True, text=True, check=True).stdout
        self.assertEqual(out, (lit / 'tpch.pl').read_text())

    def test_the_reference_answers_as_problog(self):
        text = (lit / 'tpch.pl').read_text()
        answers = tpch_exact.answers(text)
        self.assertEqual(tpch_exact.queries(text), [q for q, _ in PROBLOG])
        for q, p in PROBLOG:
            self.assertAlmostEqual(answers[q], p, delta=1e-7, msg=q)

    def test_the_generator_writes_the_views_program(self):
        out = subprocess.run([sys.executable, str(scripts / 'gen_tpch.py')] + ARGS + ['--views'],
                             capture_output=True, text=True, check=True).stdout
        self.assertEqual(out, (lit / 'tpch_views.pl').read_text())

    def test_views_have_the_lineage_of_the_flat_query(self):
        # Every query, written as one rule and through views, over tiny
        # databases: the lineage each program derives for the query is the
        # reference's, as minimal clauses, so the answers are the same. The
        # second database is the suite's at SF 0.002, where q16, q20 and q21
        # select tuples; q19's selections leave none at either.
        nonempty = set()
        for args in (['0.0004', '--queries', 'all', '--seed', '4'], ['0.002', '--queries', 'q16,q20,q21', '--seed', '1']):
            flat, views = (subprocess.run([sys.executable, str(scripts / 'gen_tpch.py')] + args + extra,
                                          capture_output=True, text=True, check=True).stdout for extra in ([], ['--views']))
            rels, p = tpch_exact.parse(flat)
            self.assertEqual(tpch_exact.parse(views), (rels, p))
            of_flat, of_views = lineages(flat), lineages(views)
            for q in tpch_exact.queries(flat):
                with self.subTest(sf=args[0], query=q):
                    reference = minimal(tpch_exact.lineage(q, rels))
                    self.assertEqual(frozenset(of_flat.get(q, {}).get((), set())), reference)
                    self.assertEqual(frozenset(of_views.get(q, {}).get((), set())), reference)
                    if reference:
                        nonempty.add(q)
        self.assertEqual(len(nonempty), 16)

    def test_the_decomposition_answers_as_enumeration(self):
        # Random monotone DNFs over few tuples: independent parts, common
        # tuples, private tuples folded into their clause and clauses over
        # the same tuples merged, the rest eliminated.
        for seed in range(200):
            rng = random.Random(seed)
            n = rng.randint(1, 10)
            p = [round(rng.uniform(0.05, 0.95), 2) for _ in range(n)]
            clauses = [frozenset(rng.sample(range(n), rng.randint(1, min(4, n))))
                       for _ in range(rng.randint(1, 8))]
            with self.subTest(seed=seed):
                self.assertAlmostEqual(tpch_exact.probability(clauses, p), brute_force(clauses, p), delta=1e-12)


if __name__ == '__main__':
    unittest.main()
