#!/usr/bin/env python3
"""Differential fuzzer: tddlog against ProbLog on random programs.

Generates small programs inside the Datalog fragment tddlog supports
(constants, ground and non-ground probabilistic facts, deterministic facts,
rules with positive recursion, stratified negation, annotated disjunctions,
probabilistic rules, comparison builtins, `query/1` and `evidence/2`), runs
both engines on each one and compares the printed marginals.

Both tools print `atom:\\tprob`; ProbLog pads and may quote, so atoms are
compared with whitespace removed and simple quoting dropped. Inconsistent
evidence counts as agreement when tddlog prints its `evidence is inconsistent`
line and ProbLog raises InconsistentEvidenceError.

Programs come from five families: a generic stratified program built from a
random signature, a probabilistic graph with transitive closure, a
smokers-style social network, negation over a recursive predicate, and mutual
recursion. A run reports how many programs agreed, disagreed, produced an
error on either side, or timed out.

usage:
  tests/fuzz_problog.py --seeds 300 [--jobs 8] [--start 0]
  tests/fuzz_problog.py --only 15,53,237   # re-run specific seeds
  tests/fuzz_problog.py --seeds 2000 --deep
  tests/fuzz_problog.py --replay tmp/fuzz/fail_17.pl
  tests/fuzz_problog.py --show 17          # print program for one seed

Disagreements are written to tmp/fuzz/fail_<seed>.pl, with a clause-level
minimisation (greedy delta debugging that keeps the disagreement class) in
tmp/fuzz/min_<seed>.pl.

Two outcomes are counted apart from the disagreements. A `report-diff` is a
program where the two engines list a different set of probability-zero atoms
but no probability differs; tddlog prints every ground instance its bottom-up
grounding produced, ProbLog only those with a proof. `--deep` skips programs
in which a query or evidence atom does not survive tddlog's grounding, which
is useful to look past a bug in the grounder that would otherwise hide the
rest.
"""

import argparse
import json
import os
import random
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor

from compare import PROBLOG, TDDLOG

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
OUTDIR = os.path.join(ROOT, 'tmp', 'fuzz')
WORKDIR = os.path.join(OUTDIR, 'work')

PROBS = [0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.9, 0.95]
VARS = ['X', 'Y', 'Z']
CMP_OPS = ['\\=', '\\=', '\\=', '\\=', '=']
# `<`, `=<`, `>`, `>=` are arithmetic in ProbLog but standard-order in tddlog,
# so they only get generated over an integer domain, where the two coincide.
ORDER_OPS = ['<', '>', '=<', '>=']


# ---------------------------------------------------------------- generation

class Pred:
    """A predicate signature with the stratum it is defined in."""

    def __init__(self, name, arity, stratum, prob=False):
        self.name = name
        self.arity = arity
        self.stratum = stratum
        self.prob = prob

    def __repr__(self):
        return '%s/%d@%d' % (self.name, self.arity, self.stratum)


def mk_atom(name, args):
    return name if not args else '%s(%s)' % (name, ','.join(args))


def fmt_prob(p):
    return ('%g' % p)


def pick_args(rng, arity, varpool, consts, var_bias=0.72):
    """Arguments for a positive body literal: fresh or reused variables, or constants."""
    args = []
    for _ in range(arity):
        if varpool and rng.random() < var_bias:
            args.append(rng.choice(varpool))
        else:
            args.append(rng.choice(consts))
    return args


def bound_args(rng, arity, bound, consts):
    """Arguments drawn only from already-bound variables and constants."""
    args = []
    for _ in range(arity):
        if bound and rng.random() < 0.75:
            args.append(rng.choice(bound))
        else:
            args.append(rng.choice(consts))
    return args


def gen_body(rng, consts, pos_pool, neg_pool, nlits):
    """Build a range-safe body.

    Positive literals come first so the body is also safe under ProbLog's
    left-to-right evaluation; negative literals and comparisons only mention
    variables bound by the positive part.
    """
    varpool = VARS[:max(1, min(3, len(consts)))]
    lits = []
    bound = []
    for _ in range(nlits):
        p = rng.choice(pos_pool)
        args = pick_args(rng, p.arity, varpool, consts)
        for a in args:
            if a in varpool and a not in bound:
                bound.append(a)
        lits.append(mk_atom(p.name, args))
    if neg_pool and rng.random() < 0.35:
        p = rng.choice(neg_pool)
        lits.append('\\+' + mk_atom(p.name, bound_args(rng, p.arity, bound, consts)))
    if bound and rng.random() < 0.35:
        ops = CMP_OPS + (ORDER_OPS if all(c.isdigit() for c in consts) else [])
        op = rng.choice(ops)
        lhs = rng.choice(bound)
        if len(bound) > 1 and rng.random() < 0.7:
            rhs = rng.choice([b for b in bound if b != lhs])
        else:
            rhs = rng.choice(consts)
        lits.append('%s %s %s' % (lhs, op, rhs))
    return lits, bound


def gen_random(rng):
    """A random stratified program: EDB facts, then one to three IDB strata."""
    n = rng.randint(2, 5)
    if rng.random() < 0.5:
        consts = [str(i + 1) for i in range(n)]
    else:
        consts = ['c%d' % (i + 1) for i in range(n)]

    stmts = ['dom(%s).' % c for c in consts]
    preds = [Pred('dom', 1, 0)]

    # A deterministic relation over the domain.
    if rng.random() < 0.6:
        ar = rng.choice([1, 2])
        tuples = [t for t in all_tuples(consts, ar) if rng.random() < 0.5]
        if not tuples:
            tuples = [all_tuples(consts, ar)[0]]
        for t in tuples:
            stmts.append('%s.' % mk_atom('rel', list(t)))
        preds.append(Pred('rel', ar, 0))

    # One or two probabilistic EDB predicates, ground or non-ground.
    for i in range(rng.randint(1, 2)):
        name = 'pf%d' % i
        ar = rng.choice([1, 2])
        if rng.random() < 0.5:
            tuples = [t for t in all_tuples(consts, ar) if rng.random() < 0.6]
            if not tuples:
                tuples = [all_tuples(consts, ar)[0]]
            for t in tuples:
                stmts.append('%s::%s.' % (fmt_prob(rng.choice(PROBS)), mk_atom(name, list(t))))
        else:
            vs = VARS[:ar]
            body = ', '.join('dom(%s)' % v for v in vs)
            stmts.append('%s::%s :- %s.' % (fmt_prob(rng.choice(PROBS)), mk_atom(name, vs), body))
        preds.append(Pred(name, ar, 0, prob=True))

    target_rules = rng.randint(3, 8)
    made = 0
    pid = 0
    strata = rng.randint(1, 3)
    for s in range(1, strata + 1):
        if made >= target_rules:
            break
        lower = list(preds)
        new = []
        for _ in range(rng.randint(1, 2)):
            pid += 1
            new.append(Pred('q%d' % pid, rng.choice([1, 1, 2]), s))
        recursive = rng.random() < 0.5
        used_in_ad = set()
        for idx, p in enumerate(new):
            if p.name in used_in_ad:
                continue
            nrules = rng.randint(1, 2)
            for k in range(nrules):
                if made >= target_rules and k > 0:
                    break
                # The first rule of a predicate never recurses, so the
                # fixpoint has a base case.
                pos_pool = lower if (k == 0 or not recursive) else lower + new
                lits, bound = gen_body(rng, consts, pos_pool, lower, rng.randint(1, 3))
                roll = rng.random()
                if roll < 0.18 and len(new) > 1 and k == 0:
                    # annotated disjunction over two heads of this stratum
                    other = new[(idx + 1) % len(new)]
                    p1 = rng.choice(PROBS)
                    p2 = rng.choice([x for x in PROBS if x + p1 <= 0.98] or [0.05])
                    h1 = mk_atom(p.name, bound_args(rng, p.arity, bound, consts))
                    h2 = mk_atom(other.name, bound_args(rng, other.arity, bound, consts))
                    stmts.append('%s::%s; %s::%s :- %s.'
                                 % (fmt_prob(p1), h1, fmt_prob(p2), h2, ', '.join(lits)))
                    used_in_ad.add(other.name)
                elif roll < 0.38:
                    head = mk_atom(p.name, bound_args(rng, p.arity, bound, consts))
                    stmts.append('%s::%s :- %s.' % (fmt_prob(rng.choice(PROBS)), head, ', '.join(lits)))
                else:
                    head = mk_atom(p.name, bound_args(rng, p.arity, bound, consts))
                    stmts.append('%s :- %s.' % (head, ', '.join(lits)))
                made += 1
        preds.extend(new)

    return stmts, preds, consts


def all_tuples(consts, arity):
    if arity == 1:
        return [(c,) for c in consts]
    return [(a, b) for a in consts for b in consts]


def gen_graph(rng):
    """Probabilistic directed graph with transitive closure."""
    n = rng.randint(3, 5)
    nodes = [str(i + 1) for i in range(n)]
    stmts = ['node(%s).' % c for c in nodes]
    edges = [(a, b) for a in nodes for b in nodes if a != b and rng.random() < 0.45]
    if not edges:
        edges = [(nodes[0], nodes[1])]
    for a, b in edges:
        stmts.append('%s::edge(%s,%s).' % (fmt_prob(rng.choice(PROBS)), a, b))
    stmts.append('path(X,Y) :- edge(X,Y).')
    if rng.random() < 0.5:
        stmts.append('path(X,Y) :- edge(X,Z), path(Z,Y).')
    else:
        stmts.append('path(X,Y) :- path(X,Z), edge(Z,Y).')
    preds = [Pred('node', 1, 0), Pred('edge', 2, 0, prob=True), Pred('path', 2, 1)]
    return stmts, preds, nodes


def gen_negrec(rng):
    """Negation over a recursive predicate."""
    stmts, preds, nodes = gen_graph(rng)
    stmts.append('unreachable(X,Y) :- node(X), node(Y), \\+ path(X,Y).')
    preds.append(Pred('unreachable', 2, 2))
    if rng.random() < 0.5:
        stmts.append('%s::isolated(X) :- node(X), \\+ path(X,X).'
                     % fmt_prob(rng.choice(PROBS)))
        preds.append(Pred('isolated', 1, 2))
    return stmts, preds, nodes


def gen_smokers(rng):
    """Social network with recursion through a probabilistic influence fact."""
    n = rng.randint(2, 4)
    people = ['p%d' % (i + 1) for i in range(n)]
    stmts = ['person(%s).' % p for p in people]
    friends = [(a, b) for a in people for b in people if a != b and rng.random() < 0.55]
    if not friends:
        friends = [(people[0], people[-1])]
    for a, b in friends:
        stmts.append('friend(%s,%s).' % (a, b))
    stmts.append('%s::stress(X) :- person(X).' % fmt_prob(rng.choice(PROBS)))
    stmts.append('%s::influences(X,Y) :- person(X), person(Y).' % fmt_prob(rng.choice(PROBS)))
    stmts.append('smokes(X) :- stress(X).')
    stmts.append('smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).')
    stmts.append('%s::asthma(X) :- smokes(X).' % fmt_prob(rng.choice(PROBS)))
    preds = [Pred('person', 1, 0), Pred('friend', 2, 0),
             Pred('stress', 1, 1, prob=True), Pred('influences', 2, 1, prob=True),
             Pred('smokes', 1, 2), Pred('asthma', 1, 3)]
    if rng.random() < 0.4:
        stmts.append('healthy(X) :- person(X), \\+ asthma(X).')
        preds.append(Pred('healthy', 1, 4))
    return stmts, preds, people


def gen_mutual(rng):
    """Mutual recursion between two predicates over a probabilistic relation."""
    n = rng.randint(3, 5)
    nodes = [str(i + 1) for i in range(n)]
    stmts = ['node(%s).' % c for c in nodes]
    links = [(a, b) for a in nodes for b in nodes if a != b and rng.random() < 0.5]
    if not links:
        links = [(nodes[0], nodes[1])]
    for a, b in links:
        stmts.append('%s::link(%s,%s).' % (fmt_prob(rng.choice(PROBS)), a, b))
    start = rng.choice(nodes)
    stmts.append('%s::seed(%s).' % (fmt_prob(rng.choice(PROBS)), start))
    stmts.append('even(X) :- seed(X).')
    stmts.append('odd(X) :- link(X,Y), even(Y).')
    stmts.append('even(X) :- link(X,Y), odd(Y).')
    preds = [Pred('node', 1, 0), Pred('link', 2, 0, prob=True), Pred('seed', 1, 0, prob=True),
             Pred('even', 1, 1), Pred('odd', 1, 1)]
    return stmts, preds, nodes


FAMILIES = [
    (0.50, 'random', gen_random),
    (0.65, 'graph', gen_graph),
    (0.78, 'smokers', gen_smokers),
    (0.90, 'negrec', gen_negrec),
    (1.00, 'mutual', gen_mutual),
]


def gen_program(seed):
    """Return (family, statement list) for a seed."""
    rng = random.Random(seed)
    r = rng.random()
    for cut, name, fn in FAMILIES:
        if r < cut:
            family = name
            stmts, preds, consts = fn(rng)
            break
    queryable = [p for p in preds if p.stratum > 0 or p.prob]
    if not queryable:
        queryable = preds
    out = list(stmts)
    for _ in range(rng.randint(1, 3)):
        p = rng.choice(queryable)
        if p.arity == 0 or rng.random() < 0.55:
            args = [rng.choice(consts) for _ in range(p.arity)]
        else:
            args = VARS[:p.arity]
        out.append('query(%s).' % mk_atom(p.name, args))
    if rng.random() < 0.45:
        for _ in range(rng.randint(1, 2)):
            p = rng.choice(queryable)
            args = [rng.choice(consts) for _ in range(p.arity)]
            val = 'true' if rng.random() < 0.6 else 'false'
            out.append('evidence(%s,%s).' % (mk_atom(p.name, args), val))
    return family, out


# ------------------------------------------------------------------- running

RESULT_LINE = re.compile(r'^(.+?):\s*([-+0-9][-+0-9.eE]*|inf|nan)\s*$')
VAR_IN_TERM = re.compile(r'(?:^|[(,])([A-Z_][A-Za-z0-9_]*)')
ERROR_CLASS = re.compile(r'\b([A-Za-z_][A-Za-z0-9_]*(?:Error|Exception|Clause|Cycle))\b')


def norm_term(t):
    t = re.sub(r'\s+', '', t)
    t = re.sub(r"'([a-z][A-Za-z0-9_]*)'", r'\1', t)
    return t


def parse_results(text):
    """Return (ground results, predicates reported with a free variable)."""
    res = {}
    open_patterns = set()
    for line in text.splitlines():
        m = RESULT_LINE.match(line.rstrip())
        if not m:
            continue
        try:
            v = float(m.group(2))
        except ValueError:
            continue
        term = norm_term(m.group(1))
        if VAR_IN_TERM.search(term):
            # ProbLog echoes the pattern when a non-ground query has no
            # ground instance at all.
            open_patterns.add(pred_key(term))
        else:
            res[term] = v
    return res, open_patterns


def pred_key(term):
    if '(' not in term:
        return (term, 0)
    name, rest = term.split('(', 1)
    depth = 0
    n = 1
    for ch in rest[:-1]:
        if ch == ',' and depth == 0:
            n += 1
    return (name, n)


def ground_heads(path, timeout):
    """Atoms that survive into tddlog's ground program, and the directive atoms."""
    try:
        p = subprocess.run([TDDLOG, '--ground', path], capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None, None
    if p.returncode != 0:
        return None, None
    heads, directives = set(), set()
    for line in p.stdout.splitlines():
        line = line.strip()
        if not line.endswith('.'):
            continue
        line = line[:-1]
        if line.startswith('query(') and line.endswith(')'):
            directives.add(norm_term(line[len('query('):-1].lstrip('\\+')))
            continue
        if line.startswith('evidence(') and line.endswith(')'):
            inner = line[len('evidence('):-1]
            directives.add(norm_term(inner.rsplit(',', 1)[0]))
            continue
        if ' :- ' in line:
            line = line.split(' :- ', 1)[0]
        if '::' in line:
            line = line.split('::', 1)[1]
        heads.add(norm_term(line))
    return heads, directives


class Outcome:
    """What one engine did with one program."""

    def __init__(self, status, results=None, open_patterns=None, message=''):
        self.status = status          # ok | inconsistent | error | timeout
        self.results = results or {}
        self.open_patterns = open_patterns or set()
        self.message = message


def run_tddlog(path, timeout):
    try:
        p = subprocess.run([TDDLOG, '--precision', '10', path],
                           capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return Outcome('timeout')
    if 'evidence is inconsistent' in p.stdout:
        return Outcome('inconsistent')
    if p.returncode != 0:
        msg = (p.stderr.strip() or p.stdout.strip() or 'exit %d' % p.returncode)
        return Outcome('error', message=msg.splitlines()[-1])
    res, pats = parse_results(p.stdout)
    return Outcome('ok', res, pats)


def run_problog(path, timeout):
    try:
        p = subprocess.run([PROBLOG, path], capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return Outcome('timeout')
    blob = p.stdout + '\n' + p.stderr
    if 'InconsistentEvidenceError' in blob:
        return Outcome('inconsistent')
    if p.returncode != 0:
        msg = (p.stderr.strip() or p.stdout.strip() or 'exit %d' % p.returncode)
        return Outcome('error', message=msg.splitlines()[-1])
    res, pats = parse_results(p.stdout)
    return Outcome('ok', res, pats)


def classify(td, pl, tol):
    """Return (kind, klass, detail).

    kind is one of agree, disagree, tddlog-error, problog-error,
    tddlog-timeout, problog-timeout.  klass names the disagreement so that
    minimisation can insist on preserving it.
    """
    if td.status == 'timeout':
        return 'tddlog-timeout', None, ''
    if pl.status == 'timeout':
        return 'problog-timeout', None, ''
    if td.status == 'error':
        return 'tddlog-error', None, td.message
    if pl.status == 'error':
        return 'problog-error', None, pl.message
    if td.status == 'inconsistent' and pl.status == 'inconsistent':
        return 'agree', None, ''
    if td.status == 'inconsistent':
        return 'disagree', 'tddlog-says-inconsistent', 'tddlog: inconsistent evidence; ProbLog answered %s' % fmt_map(pl.results)
    if pl.status == 'inconsistent':
        return 'disagree', 'problog-says-inconsistent', 'ProbLog: InconsistentEvidenceError; tddlog answered %s' % fmt_map(td.results)

    classes = set()
    detail = []
    report_only = []
    for q in sorted(set(pl.results) | set(td.results)):
        a, b = td.results.get(q), pl.results.get(q)
        if a is None:
            if abs(b) <= tol:
                report_only.append('%s listed as 0 by problog only' % q)
            else:
                classes.add('missing-in-tddlog')
                detail.append('%s: tddlog (none) vs problog %.10g' % (q, b))
        elif b is None:
            if abs(a) <= tol:
                # tddlog enumerates ground instances that are impossible and
                # prints them as 0; ProbLog leaves them out.  Same answer.
                report_only.append('%s listed as 0 by tddlog only' % q)
            else:
                classes.add('extra-in-tddlog')
                detail.append('%s: tddlog %.10g vs problog (none)' % (q, a))
        elif abs(a - b) > tol:
            classes.add('value-mismatch')
            detail.append('%s: tddlog %.10g vs problog %.10g' % (q, a, b))
    if classes:
        return 'disagree', '+'.join(sorted(classes)), '; '.join(detail)
    if report_only:
        return 'report-diff', 'zero-listed-on-one-side', '; '.join(report_only)
    return 'agree', None, ''


def fmt_map(d):
    return '{' + ', '.join('%s: %.10g' % (k, v) for k, v in sorted(d.items())) + '}'


_counter = threading.local()


def work_path(tag):
    n = getattr(_counter, 'n', 0)
    _counter.n = n + 1
    return os.path.join(WORKDIR, 'w_%s_%d_%d.pl' % (tag, threading.get_ident(), n))


def evaluate(stmts, tol, timeout, tag='x'):
    path = work_path(tag)
    with open(path, 'w') as f:
        f.write('\n'.join(stmts) + '\n')
    td = run_tddlog(path, timeout)
    pl = run_problog(path, timeout)
    os.unlink(path)
    return classify(td, pl, tol), td, pl


def minimise(stmts, klass, tol, timeout, max_rounds=6):
    """Greedy delta debugging: drop clauses while the same disagreement holds."""
    cur = list(stmts)
    for _ in range(max_rounds):
        changed = False
        i = 0
        while i < len(cur):
            if len(cur) == 1:
                break
            cand = cur[:i] + cur[i + 1:]
            if not any(c.startswith('query(') for c in cand):
                i += 1
                continue
            (kind, k, _), _, _ = evaluate(cand, tol, timeout, tag='min')
            if kind == 'disagree' and k == klass:
                cur = cand
                changed = True
            else:
                i += 1
        if not changed:
            break
    return cur


# ---------------------------------------------------------------------- main

def one_seed(seed, tol, timeout, deep=False):
    family, stmts = gen_program(seed)
    if deep:
        path = work_path('d%d' % seed)
        with open(path, 'w') as f:
            f.write('\n'.join(stmts) + '\n')
        heads, directives = ground_heads(path, timeout)
        os.unlink(path)
        if heads is not None and not directives <= heads:
            return dict(seed=seed, family=family, stmts=stmts, kind='shadowed',
                        klass=None, detail='', td_msg='', pl_msg='')
    (kind, klass, detail), td, pl = evaluate(stmts, tol, timeout, tag='s%d' % seed)
    return dict(seed=seed, family=family, stmts=stmts, kind=kind, klass=klass,
                detail=detail, td_msg=td.message, pl_msg=pl.message)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--seeds', type=int, default=300)
    ap.add_argument('--only', help='comma separated list of seeds to run instead of a range')
    ap.add_argument('--start', type=int, default=0)
    ap.add_argument('--jobs', type=int, default=8)
    ap.add_argument('--tol', type=float, default=1e-6)
    ap.add_argument('--timeout', type=float, default=60)
    ap.add_argument('--no-minimise', action='store_true')
    ap.add_argument('--deep', action='store_true',
                    help='skip programs whose query/evidence atoms tddlog deletes during '
                         'grounding, so that bugs hidden behind that one do not stay hidden')
    ap.add_argument('--replay', help='run one program file and print both outputs')
    ap.add_argument('--show', type=int, help='print the program for one seed')
    args = ap.parse_args()

    os.makedirs(OUTDIR, exist_ok=True)
    os.makedirs(WORKDIR, exist_ok=True)

    if args.show is not None:
        family, stmts = gen_program(args.show)
        print('%% seed %d (%s)' % (args.show, family))
        print('\n'.join(stmts))
        return 0

    if args.replay:
        td = run_tddlog(args.replay, args.timeout)
        pl = run_problog(args.replay, args.timeout)
        kind, klass, detail = classify(td, pl, args.tol)
        print('tddlog  :', td.status, td.message or fmt_map(td.results))
        print('problog :', pl.status, pl.message or fmt_map(pl.results))
        print('verdict :', kind, klass or '', detail)
        return 0

    counts = {'agree': 0, 'disagree': 0, 'report-diff': 0, 'shadowed': 0, 'tddlog-error': 0,
              'problog-error': 0, 'tddlog-timeout': 0, 'problog-timeout': 0}
    by_family = {}
    tddlog_errors = {}
    problog_errors = {}
    disagreements = []
    report_diffs = []
    timeouts = {'tddlog-timeout': [], 'problog-timeout': []}

    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        if args.only:
            seeds = [int(x) for x in args.only.split(',')]
        else:
            seeds = range(args.start, args.start + args.seeds)
        for r in ex.map(lambda s: one_seed(s, args.tol, args.timeout, args.deep), seeds):
            counts[r['kind']] += 1
            fam = by_family.setdefault(r['family'], dict(total=0, disagree=0))
            fam['total'] += 1
            if r['kind'] == 'tddlog-error':
                tddlog_errors.setdefault(r['td_msg'], []).append(r['seed'])
            elif r['kind'] == 'problog-error':
                problog_errors.setdefault(error_key(r['pl_msg']), []).append(r['seed'])
            elif r['kind'].endswith('-timeout'):
                timeouts[r['kind']].append(r['seed'])
                print('%s seed %d (%s)' % (r['kind'].upper(), r['seed'], r['family']))
                sys.stdout.flush()
            elif r['kind'] == 'report-diff':
                report_diffs.append(r)
            elif r['kind'] == 'disagree':
                fam['disagree'] += 1
                disagreements.append(r)
                print('DISAGREE seed %d (%s) [%s]\n    %s' % (r['seed'], r['family'], r['klass'], r['detail']))
                sys.stdout.flush()

    for r in disagreements:
        path = os.path.join(OUTDIR, 'fail_%d.pl' % r['seed'])
        with open(path, 'w') as f:
            f.write('%% seed %d (%s), class %s\n' % (r['seed'], r['family'], r['klass']))
            f.write('\n'.join(r['stmts']) + '\n')
        if not args.no_minimise:
            small = minimise(r['stmts'], r['klass'], args.tol, args.timeout)
            mpath = os.path.join(OUTDIR, 'min_%d.pl' % r['seed'])
            with open(mpath, 'w') as f:
                f.write('\n'.join(small) + '\n')
            r['min'] = small
            print('minimised seed %d: %d -> %d clauses -> %s' % (r['seed'], len(r['stmts']), len(small), mpath))

    # One saved example per distinct tddlog error message.
    for i, (msg, seeds_) in enumerate(sorted(tddlog_errors.items())):
        _, stmts = gen_program(seeds_[0])
        with open(os.path.join(OUTDIR, 'err_%d.pl' % seeds_[0]), 'w') as f:
            f.write('%% %s\n' % msg)
            f.write('\n'.join(stmts) + '\n')

    print()
    print('counts:', json.dumps(counts))
    print('by family:', json.dumps(by_family, sort_keys=True))
    if tddlog_errors:
        print('\ntddlog errors (message: count, example seeds):')
        for msg, s in sorted(tddlog_errors.items(), key=lambda kv: -len(kv[1])):
            print('  %-60s %4d  %s' % (msg[:60], len(s), s[:5]))
    if problog_errors:
        print('\nproblog errors (message: count, example seeds):')
        for msg, s in sorted(problog_errors.items(), key=lambda kv: -len(kv[1])):
            print('  %-60s %4d  %s' % (msg[:60], len(s), s[:5]))
    for k, v in timeouts.items():
        if v:
            print('\n%s seeds: %s' % (k, v))
    if report_diffs:
        print('\nreporting-only differences (both answers are 0), %d seeds: %s'
              % (len(report_diffs), [r['seed'] for r in report_diffs][:8]))
        print('  e.g. seed %d: %s' % (report_diffs[0]['seed'], report_diffs[0]['detail'][:200]))
    if disagreements:
        print('\ndisagreement classes:')
        klasses = {}
        for r in disagreements:
            klasses.setdefault(r['klass'], []).append(r['seed'])
        for k, s in sorted(klasses.items(), key=lambda kv: -len(kv[1])):
            print('  %-40s %4d  %s' % (k, len(s), s[:8]))
    with open(os.path.join(OUTDIR, 'report.json'), 'w') as f:
        json.dump(dict(counts=counts, by_family=by_family,
                       tddlog_errors={k: v for k, v in tddlog_errors.items()},
                       problog_errors={k: v for k, v in problog_errors.items()},
                       disagreements=disagreements,
                       timeouts=timeouts,
                       report_diffs=[dict(seed=r['seed'], family=r['family'], detail=r['detail'])
                                     for r in report_diffs]), f, indent=1)
    return 1 if counts['disagree'] else 0


def error_key(msg):
    m = ERROR_CLASS.search(msg or '')
    return m.group(1) if m else (msg or 'unknown')[:60]


if __name__ == '__main__':
    sys.exit(main())
