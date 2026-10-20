#!/usr/bin/env python3
"""Generate the dynamic sickness program, unrolled over time.

usage: gen_sickness.py N T [--evidence F] [--seed S]

The dynamic domain of Vlasselaer, Van den Broeck, Kimmig, Meert, De Raedt,
"TP-compilation for inference in probabilistic logic programs" (IJAR 2016,
Example 10 and Table 4): N people on a random power-law friendship graph
(preferential attachment, two edges per newcomer, each directed at random)
over T time steps. At each step it is cold outside with probability 0.2,
which makes everyone sick; someone sick stays sick with probability 0.4;
and whoever has a sick friend is sick in the same step, so the program is
cyclic wherever the friendship graph is. The paper gives the transition
model; here the first step is a transition from nobody sick. The value of
each sick atom before the last step is observed with probability F (the
paper uses 0, 0.33 and 0.66), taken from a world sampled forward, and the
sick atoms of the last step are queried: a prediction from the history,
with no query observed.
"""
import argparse
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('t', type=int)
ap.add_argument('--evidence', type=float, default=0.33)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if a.n < 2 or a.t < 1:
    ap.error('N must be at least 2 and T at least 1')
rng = random.Random(a.seed)
N, T = a.n, a.t

# Preferential attachment: newcomer v links to two earlier people chosen in
# proportion to their degree, each link directed at random.
friends = set()
degree = [1] * N
for v in range(1, N):
    targets = set()
    while len(targets) < min(2, v):
        targets.add(rng.choices(range(v), degree[:v])[0])
    for u in sorted(targets):
        friends.add((u, v) if rng.random() < 0.5 else (v, u))
        degree[u] += 1
        degree[v] += 1

# A world sampled forward: friend(X,Y) and Y sick make X sick.
sick = []
for t in range(T):
    cold = rng.random() < 0.2
    stay = [rng.random() < 0.4 for _ in range(N)]
    now = {x for x in range(N) if cold or (t > 0 and x in sick[-1] and stay[x])}
    grew = True
    while grew:
        grew = False
        for x, y in friends:
            if y in now and x not in now:
                now.add(x)
                grew = True
    sick.append(now)
observed = [[rng.random() < a.evidence for _ in range(N)] for _ in range(T - 1)]

print(f'% sickness: {N} people, {T} steps, power-law friendships, evidence {a.evidence}, seed {a.seed}')
print('0.2::cold(T) :- time(T).')
print('0.4::stay_sick(X,T) :- person(X), time(T).')
print('sick(X,T) :- person(X), cold(T).')
print('sick(X,T) :- friend(X,Y), sick(Y,T).')
print('sick(X,T) :- next(T0,T), sick(X,T0), stay_sick(X,T).')
for x in range(N):
    print(f'person(p{x}).')
for x, y in sorted(friends):
    print(f'friend(p{x},p{y}).')
for t in range(T):
    print(f'time({t}).')
for t in range(1, T):
    print(f'next({t - 1},{t}).')
for t in range(T - 1):
    for x in range(N):
        if observed[t][x]:
            print(f'evidence(sick(p{x},{t}),{"true" if x in sick[t] else "false"}).')
for x in range(N):
    print(f'query(sick(p{x},{T - 1})).')
