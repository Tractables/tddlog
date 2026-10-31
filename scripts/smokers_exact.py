#!/usr/bin/env python3
"""Exact P(smokes(x)) and P(asthma(x)): the reference for gen_smokers.py's
programs of up to 20 people, for the sizes tddlog may answer alone (nsym10 to nsym16).

usage: smokers_exact.py program.pl [--precision N]

The program is gen_smokers.py's: stress(X) for each person with
probability 0.3, influences(X,Y) for each friend(X,Y) fact with 0.2,
smokes(X) :- stress(X), smokes(X) :- friend(X,Y), influences(Y,X),
smokes(Y), and 0.4::asthma(X) :- smokes(X), with no evidence. So x smokes
exactly when a stressed person reaches x over the edges u -> x that are
present: an edge for each pair written in both directions, friend(x,u)
and friend(u,x), present when influences(u,x) is, independently with 0.2.

The method shares nothing with tddlog's: a dynamic program over the set R
of smokers. For a set S of people, C(S) is the probability that the
stress of S's people and the edges within S reach all of S, and
g(S, v) = 0.7 * 0.8^(the edges u -> v from S) the probability that
nothing in S reaches v outside it. Then P(R = S) = C(S) * prod over v
outside S of g(S, v), and C(S) = 1 - sum over T strictly inside S of
C(T) * prod over v in S - T of g(T, v). Taking the sets in increasing
order as numbers, each subset before its supersets, it costs 3^n steps.
P(smokes(x)) sums P(R = S) over the sets S that hold x, and P(asthma(x))
is 0.4 times it. The script stops on a program whose rules or
probabilities are not gen_smokers.py's, with evidence, or of more than
20 people (3^20 steps take about half an hour).

Prints `q:\tp` per query, people in the order of their person facts.
"""
import argparse
import re
import sys

RULES = ['0.3::stress(X) :- person(X).',
         '0.2::influences(X,Y) :- friend(X,Y).',
         'smokes(X) :- stress(X).',
         'smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).',
         '0.4::asthma(X) :- smokes(X).']
QUERIES = {'query(smokes(X)) :- person(X).': 'smokes',
           'query(asthma(X)) :- person(X).': 'asthma'}
STRESS, INFLUENCE, ASTHMA = 0.3, 0.2, 0.4
MAX_PEOPLE = 20


def parse(text):
    """(people, friend pairs, query predicates) of a gen_smokers.py program."""
    people, friends, queries, rules = [], set(), [], []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith('%'): continue
        m = re.fullmatch(r'person\((\w+)\)\.', line)
        if m: people.append(m.group(1)); continue
        m = re.fullmatch(r'friend\((\w+),(\w+)\)\.', line)
        if m: friends.add((m.group(1), m.group(2))); continue
        if line in QUERIES: queries.append(QUERIES[line]); continue
        rules.append(line)
    if rules != RULES:
        sys.exit(f'smokers_exact.py: not a gen_smokers.py program (or with evidence): {rules}')
    if len(set(people)) != len(people) or not queries or any(x not in people or y not in people for x, y in friends):
        sys.exit('smokers_exact.py: not a gen_smokers.py program: its people, friends or queries')
    if len(people) > MAX_PEOPLE:
        sys.exit(f'smokers_exact.py: {len(people)} people, past the {MAX_PEOPLE} its 3^n steps answer')
    return people, friends, queries


def smokes(people, friends):
    """P(smokes(x)) for each person x, in order."""
    n = len(people); ix = {p: k for k, p in enumerate(people)}
    full = (1 << n) - 1
    # into[v]: the people u with an edge u -> v, friend(v,u) and friend(u,v)
    into = [[ix[u] for (w, u) in friends if w == v and (u, w) in friends] for v in people]
    p = [0.0] * n
    total = 0.0
    acc = [0.0] * (1 << n)   # acc[S]: sum over T strictly inside S of C(T) * prod g(T, v), v in S - T
    for t in range(1 << n):
        c = 1.0 - acc[t]
        g = [(1 - STRESS) * (1 - INFLUENCE) ** sum(1 for u in into[v] if t >> u & 1) for v in range(n)]
        rest = full & ~t
        subs = []
        u = rest
        while u:             # the nonempty subsets of rest, largest first
            subs.append(u); u = (u - 1) & rest
        prod = {0: 1.0}      # prod[U]: prod over v in U of g(t, v), from U without its lowest bit
        for u in reversed(subs):
            low = u & -u
            prod[u] = prod[u ^ low] * g[low.bit_length() - 1]
            acc[t | u] += c * prod[u]
        r = c * prod[rest]   # P(R = t)
        total += r
        for k in range(n):
            if t >> k & 1: p[k] += r
    if abs(total - 1) > 1e-9:
        sys.exit(f'smokers_exact.py: the sets of smokers sum to {total!r}, not 1')
    return p


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    a = ap.parse_args()
    people, friends, queries = parse(open(a.program).read())
    ps = smokes(people, friends)
    for q in queries:
        for x, pr in zip(people, ps):
            print(f'{q}({x}):\t{(pr if q == "smokes" else ASTHMA * pr):.{a.precision}f}')


if __name__ == '__main__':
    main()
