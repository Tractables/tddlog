#!/usr/bin/env python3
"""A smokers social-network program (Fierens et al. 2015 style).

usage: gen_smokers.py N [--friends-per F] [--seed S] [--symmetric [--nested]] --template
       gen_smokers.py N [--friends-per F] [--seed S] [--symmetric [--nested]] --arguments
       gen_smokers.py N [--friends-per F] [--seed S] [--symmetric [--nested]] --data KEY

The family is one program and its data (bench.py's FOLD): --template prints
the program, which consults the people and their friendships (its place $1)
and queries who smokes; --arguments prints nothing (the program takes no
other argument); --data prints the people and friendships, KEY being only
its name: N people, each with F random friends (directed). --symmetric makes
friendship mutual: each random pair is written in both directions,
friend(X,Y) and friend(Y,X), once each.

--nested (with --symmetric) makes the sizes of one seed nested: person i
draws min(F, i - 1) friends among persons 1 to i - 1, from a random stream
of its own (seeded by the seed and i), so that the data of N + 1 people is
the data of N people plus person N + 1 and that person's friendships. The
graph has 3N - 6 edges for F = 3 and N >= 3, about the average degree of the
plain symmetric graph of the same size, whose sizes are unrelated graphs.
"""
import argparse
import random

TEMPLATE = """% smokers (Fierens et al. 2015): a person smokes from stress or because a friend who smokes
% influences them, and a smoker may have asthma. $1 is the people and their friendships, a
% file of facts `person(P).` and `friend(P,Q).`.
0.3::stress(X) :- person(X).
0.2::influences(X,Y) :- friend(X,Y).
smokes(X) :- stress(X).
smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
0.4::asthma(X) :- smokes(X).
:- consult($1).
query(smokes(X)) :- person(X).
"""


def people_and_friends(n, friends_per, seed, symmetric, nested):
    rng = random.Random(seed)
    lines = [f'% smokers: {n} people, {friends_per} friends each, seed {seed}'
             + (', symmetric' if symmetric else '') + (', nested' if nested else '')]
    lines += [f'person(p{i}).' for i in range(1, n + 1)]
    written = set()
    for i in range(1, n + 1):
        if nested:
            # the people before i, each pair new: written once each way, in i's order
            for k in random.Random(f'{seed}/{i}').sample(range(i - 1), min(friends_per, i - 1)):
                lines += [f'friend(p{i},p{k + 1}).', f'friend(p{k + 1},p{i}).']
            continue
        # the others are 1..n without i; sampling their indices draws what sampling
        # their list would, without building it (linear in n, not quadratic)
        for k in rng.sample(range(n - 1), min(friends_per, n - 1)):
            j = k + 1 if k + 1 < i else k + 2
            for x, y in ((i, j), (j, i)) if symmetric else ((i, j),):
                if (x, y) not in written:
                    written.add((x, y))
                    lines.append(f'friend(p{x},p{y}).')
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    ap.add_argument('--friends-per', type=int, default=2)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--symmetric', action='store_true')
    ap.add_argument('--nested', action='store_true')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true')
    mode.add_argument('--arguments', action='store_true')
    mode.add_argument('--data', metavar='KEY')
    a = ap.parse_args()
    if a.nested and not a.symmetric:
        ap.error('--nested needs --symmetric')
    if a.template:
        print(TEMPLATE, end='')
    elif a.data:
        print(people_and_friends(a.n, a.friends_per, a.seed, a.symmetric, a.nested), end='')


if __name__ == '__main__':
    main()
