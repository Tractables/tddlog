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

Prints `q:\tp` per query, in the order of the query facts, as tddlog does,
or `evidence is inconsistent`.
"""
import argparse
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
        print(f'{q}:\t{answers[q]:.{a.precision}f}')


if __name__ == '__main__':
    main()
