#!/usr/bin/env python3
"""uai_types.py SUITE=DIR...: whether each network of the UAI sets is a
Bayesian or a Markov network, by its structure, as a table on stdout (the
file data/uai_types.tsv, which the results' processing reads to put each
UAI instance in its category).

Every file of the UAI 2022 sets has the header MARKOV, so the header does
not tell, and the files do not always list a table's child last
(Promedus's tables list it first). A network is Bayesian when its factors
are conditional probability tables over an acyclic graph:

  - there are as many factors as variables;
  - each factor can be given a child: a scope variable over which its
    table sums to 1 for every configuration of the others (within TOL);
  - the factors can be given distinct children, so that each variable is
    the child of exactly one table (a matching of factors to variables);
  - the parent graph, from the other scope variables of a variable's table
    to the variable, is acyclic.

Any other network is a Markov network. DIR is a directory of the UAI
files a suite of bench.py reads (`--uai-benchmarks`: NAME.uai beside
NAME.uai.evid), and each of its networks is an instance of SUITE, named as
bench.py names it. Written: a header line, then one line per instance,
tab-separated: suite, instance, type (bayesian or markov), the SHA-256 of
the .uai file, and for a Markov network the first test it fails."""
import collections
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bench  # noqa: E402  (uai_suite: the instances and files of a suite)

TOL = 1e-5  # a table's rows are written with a few digits: 0.3333333 three times


def parse(text):
    """(cardinalities, [(scope, table)]) of a UAI network's text."""
    tok = text.split()
    pos = 0

    def take(k=1):
        nonlocal pos
        out = tok[pos:pos + k]
        if len(out) < k:
            raise ValueError('the file ends early')
        pos += k
        return out

    take()  # the header, MARKOV or BAYES
    n = int(take()[0])
    cards = [int(x) for x in take(n)]
    m = int(take()[0])
    scopes = []
    for _ in range(m):
        k = int(take()[0])
        scopes.append([int(x) for x in take(k)])
    tables = []
    for scope in scopes:
        size = int(take()[0])
        want = 1
        for v in scope:
            want *= cards[v]
        if size != want:
            raise ValueError(f'a table of {size} entries for a scope of {want} configurations')
        tables.append([float(x) for x in take(size)])
    return cards, list(zip(scopes, tables))


def sums_to_one(cards, scope, table, j):
    """Whether the table sums to 1 over scope[j] for every configuration of
    the scope's other variables (the last scope variable changes fastest)."""
    stride = 1
    for v in scope[j + 1:]:
        stride *= cards[v]
    k = cards[scope[j]]
    for start in range(len(table)):
        if (start // stride) % k:
            continue
        if abs(sum(table[start + t * stride] for t in range(k)) - 1) > TOL:
            return False
    return True


def children(candidates):
    """{variable: its factor}, each factor given one of its candidates and
    each variable at most one factor, as many as can be (Kuhn's augmenting
    paths, without recursion)."""
    owner = {}  # variable: factor
    for f in range(len(candidates)):
        # A depth-first search for an augmenting path from factor f.
        path = []  # (factor, the variable it takes)
        stack = [(f, iter(candidates[f]))]
        seen = set()
        while stack:
            g, it = stack[-1]
            v = next((v for v in it if v not in seen), None)
            if v is None:
                stack.pop()
                if path:
                    path.pop()
                continue
            seen.add(v)
            path.append((g, v))
            if v not in owner:
                for g2, v2 in path:
                    owner[v2] = g2
                break
            stack.append((owner[v], iter(candidates[owner[v]])))
    return owner


def kind(cards, factors):
    """('bayesian', '') or ('markov', the first test the network fails)."""
    n = len(cards)
    if len(factors) != n:
        return 'markov', f'{len(factors)} factors for {n} variables'
    candidates = []
    for f, (scope, table) in enumerate(factors):
        if not scope:
            return 'markov', f'factor {f} has an empty scope'
        if len(set(scope)) != len(scope):
            return 'markov', f'factor {f} repeats a variable'
        candidates.append([v for j, v in enumerate(scope) if sums_to_one(cards, scope, table, j)])
        if not candidates[-1]:
            return 'markov', f'factor {f} sums to 1 over none of its variables'
    owner = children(candidates)
    missing = [v for v in range(n) if v not in owner]
    if missing:
        return 'markov', f'no table can be variable {missing[0]}\'s'
    parents = {v: [p for p in factors[f][0] if p != v] for v, f in owner.items()}
    # Kahn's algorithm on the parent graph.
    indegree = {v: len(ps) for v, ps in parents.items()}
    readers = collections.defaultdict(list)
    for v, ps in parents.items():
        for p in ps:
            readers[p].append(v)
    ready = [v for v, d in indegree.items() if d == 0]
    seen = 0
    while ready:
        v = ready.pop()
        seen += 1
        for w in readers[v]:
            indegree[w] -= 1
            if indegree[w] == 0:
                ready.append(w)
    if seen < len(parents):
        return 'markov', 'the parent graph has a cycle'
    return 'bayesian', ''


def main():
    args = sys.argv[1:]
    if not args or any('=' not in a for a in args):
        sys.exit(__doc__.split('\n\n')[0])
    print('suite\tinstance\ttype\tsha256\twhy')
    for arg in args:
        suite, base = arg.split('=', 1)
        for name, (_, gargs) in bench.uai_suite(base).items():
            path = gargs[1]
            data = open(path, 'rb').read()
            t, why = kind(*parse(data.decode()))
            print('\t'.join([suite, name, t, hashlib.sha256(data).hexdigest(), why]))


if __name__ == '__main__':
    main()
