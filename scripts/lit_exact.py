#!/usr/bin/env python3
"""Exact answers for the programs of the literature families, the reference
to check tddlog against where ProbLog does not finish.

usage: lit_exact.py FAMILY program.pl [--precision N]

Reads the program a family's generator writes (scripts/gen_FAMILY.py) and
computes its answers by a method that shares nothing with tddlog's:

    mnist_sum  convolution of the digits' distributions
    pedigree   clique-tree calibration (bn_exact.py) of the Bayesian network
               over each person's two alleles, its tables read from the rules
    sickness   forward filtering over the 2^N joint states of a time step
    noisyor    clique-tree calibration of the two-layer network
    mastermind enumeration of the codes the feedback admits
    pdb        variable elimination of the pairwise model whose partition
               function is the probability that h0 (or h1) fails
    liftsep    inclusion-exclusion over the query's clauses, each term a
               product over the domain (lifted inference)
    rgraph     the weighted sum of the graph's matchings, by dynamic
               programming over the subsets of nodes
    eventcalc  forward filtering, for each pair of people, over the joint
               states of the fluents that pair's queries read
    pcfg       the inside algorithm over the observed word (either encoding)
    dnahmm     forward over the states, a sequence of states being exclusive
               of every other
    plcg       recursion over the left-corner parser's goals and positions,
               each derivation exclusive of every other
    hmmpos     forward over the tags, a sequence of tags being exclusive of
               every other
    hwf        the sum over the sequences of operators of the distributions of
               the formula's terms, summed from the right

Prints `q:\tp` per query, in the order of the query facts, as tddlog does,
or `evidence is inconsistent`.
"""
import argparse
import functools
import itertools
import math
import os
import re
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bn_exact  # noqa: E402


def facts(text, pattern):
    """The groups of every match of `pattern` at the start of a line."""
    return [m.groups() for m in re.finditer(r'^' + pattern, text, re.M)]


def queries(text):
    return [q.replace(' ', '') for (q,) in facts(text, r'query\((.*)\)\.')]


def calibrate(states, tables, evidence):
    """bn_exact.marginals on the variables numbered in the order of `states`.
    Its elimination order breaks ties by set order, which for strings changes
    from run to run with the hash seed, and with it the memory it takes; for
    integers it does not."""
    number = {x: i for i, x in enumerate(states)}
    res = bn_exact.marginals({number[x]: s for x, s in states.items()},
                             {number[x]: ([number[v] for v in scope], t) for x, (scope, t) in tables.items()},
                             [(number[x], v) for x, v in evidence])
    if res is None:
        return None
    out, observed, _ = res
    name = list(states)
    return {name[i]: m for i, m in out.items()}, {name[i]: v for i, v in observed.items()}


def mnist_sum(text):
    dists = []
    for line in text.splitlines():
        heads = re.findall(r'([0-9.]+)::digit\((\d+),(\d+)\)', line)
        if heads:
            dists.append({int(d): float(p) for p, _, d in heads})
    total = {0: 1.0}
    for dist in dists:
        step = {}
        for s, p in total.items():
            for d, q in dist.items():
                step[s + d] = step.get(s + d, 0.0) + p * q
        total = step
    return {q: total.get(int(re.fullmatch(r'sum\((\d+)\)', q).group(1)), 0.0) for q in queries(text)}


def pedigree(text):
    import gen_pedigree
    rules = [line for line in text.splitlines() if ':-' in line]
    inherit = {'p': gen_pedigree.table(rules, 'pchrom'), 'm': gen_pedigree.table(rules, 'mchrom')}
    blood = gen_pedigree.table(rules, 'bloodtype')
    alleles = ['a', 'b', 'null']
    types = ['a', 'b', 'ab', 'null', None]
    father = {c: f for f, c in facts(text, r'father\((\w+),\s*(\w+)\)\.')}
    mother = {c: m for m, c in facts(text, r'mother\((\w+),\s*(\w+)\)\.')}
    founders = {}
    for line in text.splitlines():
        heads = re.findall(r'([0-9.]+)::([mp])chrom\((\w+),(\w+)\)', line)
        if heads and ':-' not in line:
            founders[f'{heads[0][1]}:{heads[0][2]}'] = {v: float(p) for p, _, _, v in heads}
    evidence = [(f'bt:{p}', v) for p, v in facts(text, r'evidence\(bloodtype\((\w+),(\w+)\)\)\.')]
    asked = []
    for q in queries(text):
        p, v = re.fullmatch(r'bloodtype\((\w+),(\w+)\)', q).groups()
        asked.append((q, f'bt:{p}', v))
    states, tables = {}, {}
    for x, prior in founders.items():
        states[x] = alleles
        tables[x] = ([x], np.array([prior.get(v, 0.0) for v in alleles]))
    for child in father:
        for c, parent in (('p', father[child]), ('m', mother[child])):
            x, scope = f'{c}:{child}', [f'p:{parent}', f'm:{parent}']
            t = np.zeros((3, 3, 3))
            for i, u in enumerate(alleles):
                for j, w in enumerate(alleles):
                    for v, p in inherit[c][(u, w)]:
                        t[i, j, alleles.index(v)] = p
            states[x] = alleles
            tables[x] = (scope + [x], t)
    for person in {x.split(':')[1] for x, _ in evidence} | {x.split(':')[1] for _, x, _ in asked}:
        x = f'bt:{person}'
        t = np.zeros((3, 3, 5))
        for i, u in enumerate(alleles):
            for j, w in enumerate(alleles):
                for v, p in blood[(u, w)]:
                    t[i, j, types.index(v)] = p
                t[i, j, 4] = max(0.0, 1.0 - t[i, j, :4].sum())
        states[x] = types
        tables[x] = ([f'p:{person}', f'm:{person}', x], t)
    res = calibrate(states, tables, evidence)
    if res is None:
        return None
    out, observed = res
    return {q: float(observed[x] == v) if x in observed else float(out[x][types.index(v)]) for q, x, v in asked}


def sickness(text):
    people = [p for (p,) in facts(text, r'person\((\w+)\)\.')]
    index = {p: i for i, p in enumerate(people)}
    n = len(people)
    edges = [(index[x], index[y]) for x, y in facts(text, r'friend\((\w+),(\w+)\)\.')]
    later = dict(facts(text, r'next\((\d+),(\d+)\)\.'))
    times = [t for (t,) in facts(text, r'time\((\d+)\)\.')]
    order = [t for t in times if t not in later.values()]
    while order[-1] in later:
        order.append(later[order[-1]])
    assert sorted(order) == sorted(times), 'time steps form one chain'
    evidence = {}
    for p, t, v in facts(text, r'evidence\(sick\((\w+),(\d+)\),(true|false)\)\.'):
        evidence.setdefault(t, []).append((index[p], v == 'true'))
    cold, stay = (float(re.search(rf'^([0-9.]+)::{a}\(', text, re.M).group(1)) for a in ('cold', 'stay_sick'))
    # X is sick when someone X reaches over friend edges, X included, is a
    # seed: sick at the step before and staying sick.
    reach = [1 << x for x in range(n)]
    for _ in range(n):
        for x, y in edges:
            reach[x] |= reach[y]
    states = np.arange(1 << n, dtype=np.int64)
    closure = np.zeros(1 << n, dtype=np.int64)
    for x in range(n):
        closure |= ((states & reach[x]) != 0).astype(np.int64) << x
    everyone = (1 << n) - 1
    alpha = None
    for t in order:
        if alpha is None:
            seeds = np.zeros(1 << n)
            seeds[0] = 1.0
        else:
            # The distribution of the seeds: each sick person stays sick
            # independently, one bit at a time.
            seeds = alpha.copy()
            for x in range(n):
                v = seeds.reshape(1 << (n - 1 - x), 2, 1 << x)
                v[:, 0, :] += (1 - stay) * v[:, 1, :]
                v[:, 1, :] *= stay
        alpha = (1 - cold) * np.bincount(closure, weights=seeds, minlength=1 << n)
        alpha[everyone] += cold * seeds.sum()
        for x, v in evidence.get(t, []):
            alpha[((states >> x) & 1) != v] = 0.0
        total = alpha.sum()
        if total == 0.0:
            return None
        alpha /= total
    out = {}
    for q in queries(text):
        p, t = re.fullmatch(r'sick\((\w+),(\d+)\)', q).groups()
        assert t == order[-1], 'queries ask for the last step'
        out[q] = float(alpha[((states >> index[p]) & 1) == 1].sum())
    return out


def noisyor(text):
    prior = {d: float(p) for p, d in facts(text, r'([0-9.]+)::disease\((\w+)\)\.')}
    leak = {f: float(p) for p, f in facts(text, r'([0-9.]+)::leak\((\w+)\)\.')}
    causes = {}
    for p, d, f in facts(text, r'([0-9.]+)::cause\((\w+),(\w+)\)\.'):
        causes.setdefault(f, []).append((d, float(p)))
    states, tables = {}, {}
    for d, p in prior.items():
        states[d] = [True, False]
        tables[d] = ([d], np.array([p, 1 - p]))
    for f, q in leak.items():
        ds = causes.get(f, [])
        t = np.zeros([2] * len(ds) + [2])
        for key in np.ndindex(*[2] * len(ds)):
            off = (1 - q) * math.prod(1 - p for (_, p), k in zip(ds, key) if k == 0)
            t[key] = [1 - off, off]
        states[f] = [True, False]
        tables[f] = ([d for d, _ in ds] + [f], t)
    evidence = [(f, v == 'true') for f, v in facts(text, r'evidence\(finding\((\w+)\),(true|false)\)\.')]
    res = calibrate(states, tables, evidence)
    if res is None:
        return None
    out, observed = res
    answers = {}
    for q in queries(text):
        d = re.fullmatch(r'disease\((\w+)\)', q).group(1)
        answers[q] = float(observed[d]) if d in observed else float(out[d][0])
    return answers


def partition(factors, sizes):
    """The sum over every assignment of the product of `factors`, each
    (scope, table), eliminating the variable of least cluster first. Each
    product comes divided by a constant (`bn_exact.scaled_product`), whose
    logarithms are added up and multiplied back at the end."""
    constant, shift = 1.0, 0.0
    while factors:
        constant *= math.prod(float(t) for scope, t in factors if not scope)
        scoped = [f for f in factors if f[0]]
        if not scoped:
            break
        neighbors = {}
        for scope, _ in scoped:
            for v in scope:
                neighbors.setdefault(v, set()).update(scope)
        x = min(neighbors, key=lambda v: (len(neighbors[v]), str(v)))
        touching = [f for f in scoped if x in f[0]]
        keep = sorted(neighbors[x] - {x}, key=str)
        t, s = bn_exact.scaled_product(touching, keep, sizes)
        factors, shift = [f for f in scoped if x not in f[0]] + [(keep, t)], shift + s
    return constant * 10 ** shift


def pdb(text):
    r = {c: float(p) for p, c in facts(text, r'([0-9.]+)::r\((\w+)\)\.')}
    t = {c: float(p) for p, c in facts(text, r'([0-9.]+)::t\((\w+)\)\.')}
    s = {(x, y): float(p) for p, x, y in facts(text, r'([0-9.]+)::s\((\w+),(\w+)\)\.')}
    assert r.keys() == t.keys() and all(x in r and y in r for x, y in s), 'every constant has an R and a T tuple'
    sizes = {(rel, c): 2 for c in r for rel in 'rt'}

    def fails(joins):
        """P(no tuple of S joins): R and T each a variable (index 1 true),
        S summed out per pair, which `joins(r, t)` says it completes."""
        fs = [([('r', c)], np.array([1 - p, p])) for c, p in r.items()]
        fs += [([('t', c)], np.array([1 - p, p])) for c, p in t.items()]
        for (x, y), p in s.items():
            fs.append(([('r', x), ('t', y)], np.array([[1 - p * joins(i, j) for j in (0, 1)] for i in (0, 1)])))
        return partition(fs, sizes)

    answers = {'h0': 1 - fails(lambda i, j: i and j), 'h1': 1 - fails(lambda i, j: i or j)}
    answers['safe'] = 1 - math.prod(1 - p * (1 - math.prod(1 - q for (x, _), q in s.items() if x == c))
                                    for c, p in r.items())
    return answers


def mastermind(text):
    pegs = {}
    for line in text.splitlines():
        heads = re.findall(r'([0-9.]+)::code\((\d+),(\w+)\)', line)
        if heads:
            pegs[int(heads[0][1])] = [(c, float(p)) for p, _, c in heads]
    guess = {}
    for g, k, c in facts(text, r'guess\((\w+),(\d+),(\w+)\)\.'):
        guess.setdefault(g, {})[int(k)] = c
    black = {g: int(b) for g, _, b in facts(text, r'evidence\(black\((\w+),(\d+),(\d+)\)\)\.')}
    white = {g: int(w) for g, w in facts(text, r'evidence\(white\((\w+),(\d+)\)\)\.')}
    order = sorted(pegs)
    weight = {}
    for code in itertools.product(*(pegs[k] for k in order)):
        colours = [c for c, _ in code]
        ok = True
        for g, gs in guess.items():
            guessed = [gs[k] for k in order]
            b = sum(x == y for x, y in zip(colours, guessed))
            common = sum(min(colours.count(c), guessed.count(c)) for c in set(guessed))
            if black.get(g, b) != b or white.get(g, common - b) != common - b:
                ok = False
                break
        if ok:
            weight[tuple(colours)] = math.prod(p for _, p in code)
    total = sum(weight.values())
    if total == 0.0:
        return None
    answers = {}
    for q in queries(text):
        k, c = re.fullmatch(r'code\((\d+),(\w+)\)', q).groups()
        i = order.index(int(k))
        answers[q] = sum(w for code, w in weight.items() if code[i] == c) / total
    return answers


FAMILIES = {'mnist_sum': mnist_sum, 'mastermind': mastermind, 'pedigree': pedigree, 'sickness': sickness, 'noisyor': noisyor, 'pdb': pdb}


# The families of the second literature scan.

def chain_ok(probs):
    """The probability that no two consecutive variables of a chain are
    both true, the variables independent with these probabilities."""
    off, on = 1 - probs[0], probs[0]
    for q in probs[1:]:
        off, on = (off + on) * (1 - q), off * q
    return off + on


def liftsep(text):
    """Inclusion-exclusion over the CNF of each query over h30..h33 (Beame,
    Li, Roy, Suciu, TODS 2017, Example 3.6): a term whose atoms miss one
    h3l splits into runs of consecutive h3l on disjoint relations, and a
    run is a product over x (it reads R), over y (it reads T) or over the
    pairs (neither)."""
    r = {c: float(p) for p, c in facts(text, r'([0-9.]+)::r\((\w+)\)\.')}
    t = {c: float(p) for p, c in facts(text, r'([0-9.]+)::t\((\w+)\)\.')}
    s = {}
    for p, rel, x, y in facts(text, r'([0-9.]+)::s([123])\((\w+),(\w+)\)\.'):
        s[int(rel), x, y] = float(p)
    xs, ys = sorted(r), sorted(t)
    assert len(s) == 3 * len(xs) * len(ys), 'S1, S2 and S3 hold every pair'

    def run_false(lo, hi):
        """P(no h3l with lo <= l <= hi holds); nodes z0 = R(x), zl = Sl(x,y)
        for 1 <= l <= 3, z4 = T(y), and h3l reads z_l and z_l+1."""
        def nodes(x, y, zr=None, zt=None):
            z = [zr, *(s[ell, x, y] for ell in (1, 2, 3)), zt]
            return z[lo:hi + 2]
        if lo == 0:
            return math.prod((1 - r[x]) * math.prod(chain_ok(nodes(x, y, zr=0.0)) for y in ys)
                             + r[x] * math.prod(chain_ok(nodes(x, y, zr=1.0)) for y in ys) for x in xs)
        if hi == 3:
            return math.prod((1 - t[y]) * math.prod(chain_ok(nodes(x, y, zt=0.0)) for x in xs)
                             + t[y] * math.prod(chain_ok(nodes(x, y, zt=1.0)) for x in xs) for y in ys)
        return math.prod(chain_ok(nodes(x, y)) for x in xs for y in ys)

    def union(atoms):
        """P(h3l for some l in atoms), a proper subset of {0, 1, 2, 3}."""
        none, run = 1.0, []
        for ell in range(5):
            if ell in atoms:
                run.append(ell)
            elif run:
                none *= run_false(run[0], run[-1])
                run = []
        return 1 - none

    clauses = {}
    for name, k, ell in facts(text, r'(\w+)_(\d+) :- h3(\d)\.'):
        clauses.setdefault(name, {}).setdefault(int(k), set()).add(int(ell))
    answers = {}
    for name, cnf in clauses.items():
        cnf = list(cnf.values())
        coefficient = {}
        for size in range(1, len(cnf) + 1):
            for chosen in itertools.combinations(cnf, size):
                atoms = frozenset().union(*chosen)
                coefficient[atoms] = coefficient.get(atoms, 0) + (-1) ** (size + 1)
        assert coefficient.pop(frozenset(range(4)), 0) == 0, f'{name} is not safe: its h3 term does not cancel'
        answers[name] = sum(c * union(atoms) for atoms, c in coefficient.items())
    return answers


def rgraph(text):
    """No path of two edges: the edges present form a matching. The weighted
    sum of the matchings of each set of nodes, by its lowest node."""
    edges = {(int(x), int(y)): float(p) for p, x, y in facts(text, r'([0-9.]+)::e\((\d+),(\d+)\)\.')}
    assert queries(text) == ['path2'], 'rgraph asks path2'
    n = 1 + max(max(e) for e in edges)
    if n > 26:
        sys.exit(f'lit_exact: the subsets of {n} nodes do not fit')
    w = np.zeros((n, n))
    for (x, y), p in edges.items():
        w[x, y] = w[y, x] = p / (1 - p)
    m = np.zeros(1 << n)
    m[0] = 1.0
    for v in range(n - 1, -1, -1):
        rest = np.arange(1 << (n - 1 - v), dtype=np.int64) << (v + 1)
        acc = m[rest].copy()
        for u in range(v + 1, n):
            has = ((rest >> u) & 1) == 1
            acc[has] += w[v, u] * m[rest[has] ^ (1 << u)]
        m[rest | (1 << v)] = acc
    return {'path2': 1 - math.prod(1 - p for p in edges.values()) * m[-1]}


EC_MOVES = {'walking', 'running', 'active'}


def eventcalc(text):
    """Each queried pair (P,Q): the joint distribution of person(P),
    person(Q), meeting(P,Q) and moving(P,Q) at each frame, filtered forward
    over the frames; the events of a frame are independent of earlier
    frames and of other people."""
    nxt = dict(facts(text, r'next\((\d+),(\d+)\)\.'))
    frames = int(max(nxt.values(), key=int)) + 1
    assert all(int(b) == int(a) + 1 for a, b in nxt.items()) and len(nxt) == frames - 1, 'frames 0 to T-1'
    happens = {(who, int(t)): (act, float(p)) for p, act, who, t in facts(text, r'([0-9.]+)::happens\((\w+),(\w+),(\d+)\)\.')}
    gone = {(who, int(t)) for who, t in facts(text, r'disappear\((\w+),(\d+)\)\.')}
    scene = {(rel, x, y, int(t)) for rel, x, y, t in facts(text, r'(close25|close34|far34|orient45)\((\w+),(\w+),(\d+)\)\.')}
    asked = {}
    for q in queries(text):
        fluent, x, y, t = re.fullmatch(r'holds_(meeting|moving)\((\w+),(\w+),(\d+)\)', q).groups()
        asked.setdefault((x, y), set()).add(int(t))
    answers = {}
    for (x, y), times in asked.items():
        dist = {(False, False, False, False): 1.0}
        for t in range(frames):
            if t in times:
                answers[f'holds_meeting({x},{y},{t})'] = sum(p for s, p in dist.items() if s[2])
                answers[f'holds_moving({x},{y},{t})'] = sum(p for s, p in dist.items() if s[3])
            if t == frames - 1:
                break
            far = ('far34', x, y, t) in scene
            out_x, out_y = (x, t) in gone, (y, t) in gone
            cases = [((), 1.0)]
            for who in (x, y):
                act, p = happens.get((who, t), (None, 0.0))
                cases = [(c + (act,), w * p) for c, w in cases] + [(c + (None,), w * (1 - p)) for c, w in cases]
            step = {}
            for (hx, hy), w in cases:
                if w == 0.0:
                    continue
                stop = hx == 'running' or hy == 'running' or out_x or out_y or \
                    (far and (hx == 'walking' or hy == 'walking'))
                term_move = stop or (hx, hy) in (('active', 'active'), ('active', 'inactive'), ('inactive', 'active'))
                init_move = hx == 'walking' and hy == 'walking' and ('close34', x, y, t) in scene \
                    and ('orient45', x, y, t) in scene
                for (px, py, meet, move), p in dist.items():
                    near = ('close25', x, y, t) in scene and py
                    init_meet = near and ((hx == 'active' and hy != 'running')
                                          or (px and hx == 'inactive' and hy not in ('running', 'active')))
                    s = ((hx in EC_MOVES and not out_x) or (px and not out_x),
                         (hy in EC_MOVES and not out_y) or (py and not out_y),
                         init_meet or (meet and not stop),
                         init_move or (move and not term_move))
                    step[s] = step.get(s, 0.0) + w * p
            dist = step
    return answers


FAMILIES.update({'liftsep': liftsep, 'rgraph': rgraph, 'eventcalc': eventcalc})


def pcfg(text):
    """The inside algorithm over the observed word for the ProbLog
    tutorial's grammar S -> O M | S S, O -> a, M -> b | S C, C -> b, its
    probabilities read from either encoding gen_pcfg.py writes: the
    probability that the first S and the first S it expands to both
    expand to S -> S S, given the word, in long doubles, whose range holds
    the word's probability."""
    t = re.sub(r'\s+', '', text)
    stack = re.search(r'([0-9.]+)::state\(W,\[o,m\|S\]\);([0-9.]+)::state\(W,\[s,s\|S\]\):-state\(W,\[s\|S\]\)\.', t)
    if stack:
        m = re.search(r'([0-9.]+)::state\(\[b\|W\],S\);([0-9.]+)::state\(W,\[s,c\|S\]\):-state\(W,\[m\|S\]\)\.', t)
        assert 'state([a|W],S):-state(W,[o|S]).' in t and 'state([b|W],S):-state(W,[c|S]).' in t
        (om, ss), (mb, sc) = stack.groups(), m.groups()
    else:
        s, m = re.search(r'([0-9.]+)::s_to\(N,om\);([0-9.]+)::s_to\(N,ss\)\.', t), \
            re.search(r'([0-9.]+)::m_to\(N,b\);([0-9.]+)::m_to\(N,sc\)\.', t)
        assert "o([a],s(N,N),m(M,M))." in t and "c([b],s(N,N),m(M,M))." in t
        (om, ss), (mb, sc) = s.groups(), m.groups()
    om, ss, mb, sc = (np.longdouble(p) for p in (om, ss, mb, sc))
    w = re.search(r'evidence\(word\(\[([a-z,]*)\]\)\)\.', t).group(1).split(',')
    n = len(w)
    # S[i, j] and M[i, j]: the inside probabilities of w[i:j].
    S = np.zeros((n + 1, n + 1), dtype=np.longdouble)
    M = np.zeros((n + 1, n + 1), dtype=np.longdouble)
    for i in range(n):
        M[i, i + 1] = mb if w[i] == 'b' else 0
    for span in range(2, n + 1):
        for i in range(n - span + 1):
            j = i + span
            split = np.dot(S[i, i + 1:j], S[i + 1:j, j])
            S[i, j] = (om * M[i + 1, j] if w[i] == 'a' else 0) + ss * split
            M[i, j] = sc * S[i, j - 1] if w[j - 1] == 'b' else 0
    # The prefixes w[:j] derived by an S that expands to S S, each followed
    # by an S that derives the rest.
    second = ss * sum(ss * np.dot(S[0, 1:j], S[1:j, j]) * S[j, n] for j in range(2, n))
    (q,) = queries(text)
    assert q in ('state([],[s,s,s])', 's_to(1,ss)'), q
    return {q: float(second / S[0, n])}


def dnahmm(text):
    """Forward over cplint's HMM of DNA: the chain starts in q1, and from
    a state other than end chooses the next state and a letter, each
    choice a variable of its own for each sequence of states before it, so
    the sequences of states are exclusive and the probability of the
    observed letters is the sum over them, the last step moving to end."""
    nxt, letter = {}, {}
    for line in text.splitlines():
        for p, q, r in re.findall(r'([0-9.eE-]+)::next_state\((\w+),(\w+),S\)', line):
            nxt.setdefault(q, {})[r] = float(p)
        for p, q, x in re.findall(r'([0-9.eE-]+)::letter\((\w+),(\w+),S\)', line):
            letter.setdefault(q, {})[x] = float(p)
    answers = {}
    for q in queries(text):
        seq = re.fullmatch(r'hmm\(\[([a-z,]*)\]\)', q).group(1).split(',')
        alpha = {'q1': 1.0}
        for k, x in enumerate(seq):
            last = k == len(seq) - 1
            step = {}
            for s, p in alpha.items():
                for r, pr in nxt[s].items():
                    if (r == 'end') == last:
                        step[r] = step.get(r, 0.0) + p * letter[s].get(x, 0.0) * pr
            alpha = step
        answers[q] = alpha.get('end', 0.0)
    return answers


FAMILIES.update({'pcfg': pcfg, 'dnahmm': dnahmm})


def plcg(text):
    """cplint's left-corner parser for S -> S S | a | b, by recursion over
    its goals and the positions in the word: a goal S chooses its first
    word, and each rule whose left corner is found attaches to the goal or
    projects to a new left corner, every choice indexed by the derivation
    so far, so that the derivations are exclusive and the probability of
    the word is the sum over them of the products of their choices."""
    t = re.sub(r'\s+', '', text)
    first = dict((w, float(p)) for p, w in re.findall(r"(\d+(?:\.\d+)?)::first\('S',Der,(\w+)\)", t))
    op = dict((o, float(p)) for p, o in re.findall(r"(\d+(?:\.\d+)?)::attach\('S',Der,(\w+)\)", t))
    for rule in ["lc('S','S',_Der,rule('S',['S','S']))", "lc('S',a,_Der,rule('S',[a]))",
                 "lc('S',b,_Der,rule('S',[b]))", 'terminal(a).', 'terminal(b).']:
        assert rule in t, rule
    assert set(first) == {'a', 'b'} and set(op) == {'attach', 'project'}
    rhs2 = {'S': ('S',), 'a': (), 'b': ()}
    answers = {}
    for q in queries(text):
        w = re.fullmatch(r'plc\(\[([a-z,]*)\]\)', q).group(1).split(',')
        n = len(w)

        @functools.lru_cache(maxsize=None)
        def goals(gs, i):
            """{j: probability} that the goals gs parse w[i:j]."""
            if not gs:
                return {i: 1.0}
            g, rest = gs[0], gs[1:]
            out = {}
            if g in ('a', 'b'):
                return goals(rest, i + 1) if i < n and w[i] == g else {}
            if i >= n:
                return {}
            for j, p in corner(w[i], i + 1).items():
                for k, r in goals(rest, j).items():
                    out[k] = out.get(k, 0.0) + first[w[i]] * p * r
            return out

        @functools.lru_cache(maxsize=None)
        def corner(b, i):
            """{j: probability} that the left corner b, found before i,
            completes the goal S through w[i:j]."""
            out = {}
            for j, p in goals(rhs2[b], i).items():
                out[j] = out.get(j, 0.0) + op['attach'] * p
                for k, r in corner('S', j).items():
                    out[k] = out.get(k, 0.0) + op['project'] * p * r
            return out

        answers[q] = goals(('S',), 0).get(n, 0.0)
    return answers


def hmmpos(text):
    """Forward over cplint's part-of-speech HMM: the chain chooses a first
    tag from start and then each next tag, each choice a variable of its
    own for each sequence of tags before it, so the sequences of tags are
    exclusive; a tag emits only the words its out/3 facts allow, and the
    choice after the last word is summed out."""
    trans = {}
    for p, q, r in re.findall(r'([0-9.eE-]+)::trans\((\w+),(\w+),S\)', text):
        trans.setdefault(q, {})[r] = float(p)
    emits = set(re.findall(r'^out\((\w+),(\w+),_\)\.', text, re.M))
    answers = {}
    for q in queries(text):
        words = re.fullmatch(r'hmm\(\[([a-z,]*)\]\)', q).group(1).split(',')
        alpha = {t: p for t, p in trans['start'].items() if (words[0], t) in emits}
        for x in words[1:]:
            alpha = {r: sum(a * trans[t].get(r, 0.0) for t, a in alpha.items())
                     for r in trans['start'] if (x, r) in emits}
        answers[q] = sum(a * sum(trans[t].values()) for t, a in alpha.items())
    return answers


FAMILIES.update({'plcg': plcg, 'hmmpos': hmmpos})


def hwf(text):
    """DeepProbLog's handwritten formulas: the probability that the
    classified symbols make a formula within 1e-4 of the label, its value
    computed as parse/2 does (products and quotients from the left in
    Python's arithmetic, a quotient by zero no parse, then the sum from
    the right, a difference adding the next term times -1). A sequence of
    symbols has one parse, so the sequences are exclusive: the sum runs
    over the sequences of operators, and for each over the distributions
    of its terms' values, each term's digits independent of the others'."""
    num, op = {}, {}
    for p, img, d in re.findall(r'([0-9.]+)::detect_number\((\w+),(\d+)\)', text):
        num.setdefault(img, {})[int(d)] = float(p)
    for p, img, o in re.findall(r'([0-9.]+)::detect_operator\((\w+),([-+*/])\)', text):
        op.setdefault(img, {})[o] = float(p)
    answers = {}
    for q in queries(text):
        m = re.fullmatch(r'expression\(\[([\w,]+)\],(-?[0-9.]+)\)', q)
        images, label = m.group(1).split(','), float(m.group(2))
        digits, ops = images[0::2], images[1::2]
        total = 0.0
        for seq in itertools.product('+-*/', repeat=len(ops)):
            weight = math.prod(op[img][o] for img, o in zip(ops, seq))
            if weight == 0.0:
                continue
            # The terms: each its first digit's image, whether it is
            # negated, and its operators and digits' images after.
            terms = [[digits[0], False, []]]
            for k, o in enumerate(seq):
                if o in '+-':
                    terms.append([digits[k + 1], o == '-', []])
                else:
                    terms[-1][2].append((o, digits[k + 1]))
            dists = []
            for first, negated, rest in terms:
                dist = {}
                for d0, p0 in num[first].items():
                    vals = {(-1 * d0 if negated else d0): p0}
                    for o, img in rest:
                        nxt = {}
                        for v, pv in vals.items():
                            for d, pd in num[img].items():
                                if o == '/' and d == 0:
                                    continue
                                w = v * d if o == '*' else v / d
                                nxt[w] = nxt.get(w, 0.0) + pv * pd
                        vals = nxt
                    for v, pv in vals.items():
                        dist[v] = dist.get(v, 0.0) + pv
                dists.append(dist)
            if len(dists) == 1:
                total += weight * sum(p for v, p in dists[0].items() if abs(v - label) < 0.0001)
                continue
            # The sum from the right: float(t_j + v_(j+1)), the last float(t_m).
            right = {}
            for v, p in dists[-1].items():
                right[float(v)] = right.get(float(v), 0.0) + p
            for dist in reversed(dists[1:-1]):
                nxt = {}
                for t, pt in dist.items():
                    for v, pv in right.items():
                        s = float(t + v)
                        nxt[s] = nxt.get(s, 0.0) + pt * pv
                right = nxt
            total += weight * sum(pt * pv for t, pt in dists[0].items() for v, pv in right.items()
                                  if abs(t + v - label) < 0.0001)
        answers[q] = total
    return answers


FAMILIES.update({'hwf': hwf})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('family', choices=sorted(FAMILIES))
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    a = ap.parse_args()
    text = open(a.program).read()
    answers = FAMILIES[a.family](text)
    if answers is None:
        print('evidence is inconsistent')
        return
    for q in queries(text):
        if q not in answers:
            sys.exit(f'lit_exact: no answer for {q}')
        # Fixed point would print an answer under 1e-precision as 0 (dnahmm's).
        p = answers[q]
        print(f'{q}:\t{p:.{a.precision}f}' if p == 0 or p >= 1e-3 else f'{q}:\t{p:.{a.precision}e}')


if __name__ == '__main__':
    main()
