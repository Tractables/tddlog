#!/usr/bin/env python3
"""Generate the probabilistic event calculus over a video's tracked people.

usage: gen_eventcalc.py N T [--every K] [--seed S]

The event definitions of Prob-EC: Skarlatidis, Artikis, Filippou,
Paliouras, "A probabilistic logic programming event calculus" (TPLP 15(2),
2015), which recognise long-term activities of people in the CAVIAR videos
from their short-term activities. Each frame gives, for each person in
view, the short-term activity a tracker detected (walking, running,
active or inactive), a probabilistic fact whose probability is the
tracker's confidence, and the crisp facts of the frame: who leaves the
view, and which pairs of people stand within the distances the
definitions name with similar orientations. A long-term activity
(fluent) holds from the frame after an event initiates it until an event
terminates it, by the event calculus's law of inertia.

Prob-EC is written for ProbLog 1 with function symbols (`F=V`, coordinates
as pairs) and an interval search for the events that break a fluent. This
is the same calculus in the Datalog fragment that ProbLog 2 and tddlog
read: each fluent a predicate over the frame, inertia as recursion over
`next/2`,

    holds(F, T+1) :- initiated(F, T).
    holds(F, T+1) :- holds(F, T), not terminated(F, T).

which is Prob-EC's holdsAt for the value true, and distances and
orientations read off the scene as facts. The fluents are person(P) and,
for each ordered pair of people, meeting(P,Q) and moving(P,Q), the two
activities of Prob-EC's evaluation, with their initiation and termination
rules as in Prob-EC's event definitions (fighting and leaving an object
are left out).

The scene is simulated rather than read from CAVIAR, whose annotations have
no licence: N people in a 400 by 400 pixel room over T frames walk (3
pixels a frame) or now and then run (9) to one of three meeting spots or to
a random point, stand there active or inactive for 20 to 100 frames, and
leave, sometimes together with someone who stands at the same spot. Each
activity fact's probability is drawn from [0.05, 0.99], as the noisy
CAVIAR inputs of Prob-EC's evaluation spread over that range. Every K-th
frame (default 10) asks meeting and moving for every ordered pair.
"""
import argparse
import math
import random

ap = argparse.ArgumentParser()
ap.add_argument('n', type=int)
ap.add_argument('t', type=int)
ap.add_argument('--every', type=int, default=10)
ap.add_argument('--seed', type=int, default=1)
a = ap.parse_args()
if a.n < 2 or a.t < 8 or a.every < 1:
    ap.error('need N >= 2, T >= 8 and K >= 1')
rng = random.Random(a.seed)
N, T = a.n, a.t
SIZE, WALK, RUN = 400, 3, 9
SPOTS = [(rng.randrange(60, SIZE - 60), rng.randrange(60, SIZE - 60)) for _ in range(3)]

# Each person is in view over [first, last]: half of them from frame 0, the
# others from a frame of the first quarter; the last one and half of the
# others leave in the last quarter.
first = [0 if i < (N + 1) // 2 else rng.randrange(T // 4 + 1) for i in range(N)]
last = [rng.randrange(3 * T // 4, T - 1) if i == N - 1 or rng.random() < 0.5 else T - 1 for i in range(N)]


def pick_target():
    return rng.choice(SPOTS) if rng.random() < 0.6 else (rng.randrange(SIZE), rng.randrange(SIZE))


pos = [(rng.randrange(SIZE), rng.randrange(SIZE)) for _ in range(N)]
mode = ['walking'] * N
target = [pick_target() for _ in range(N)]
dwell = [0] * N
heading = [rng.randrange(360) for _ in range(N)]
frames = []  # per frame: {person: (x, y, orientation, activity)}
for t in range(T):
    seen = {}
    for i in range(N):
        if not first[i] <= t <= last[i]:
            continue
        x, y = pos[i]
        if mode[i] in ('walking', 'running'):
            tx, ty = target[i]
            d = math.hypot(tx - x, ty - y)
            step = WALK if mode[i] == 'walking' else RUN
            if d <= step:
                x, y = tx, ty
                mode[i] = 'active' if rng.random() < 0.5 else 'inactive'
                dwell[i] = rng.randrange(20, 101)
            else:
                heading[i] = round(math.degrees(math.atan2(ty - y, tx - x))) % 360
                x, y = round(x + step * (tx - x) / d), round(y + step * (ty - y) / d)
        else:
            if mode[i] == 'active':
                x, y = min(SIZE, max(0, x + rng.choice((-1, 0, 1)))), min(SIZE, max(0, y + rng.choice((-1, 0, 1))))
            dwell[i] -= 1
            if dwell[i] <= 0:
                mode[i] = 'walking' if rng.random() < 0.85 else 'running'
                target[i] = pick_target()
                # Someone standing at the same place may leave along.
                for j in range(N):
                    if j != i and first[j] <= t <= last[j] and mode[j] in ('active', 'inactive') \
                            and math.dist(pos[j], (x, y)) <= 25 and rng.random() < 0.5:
                        mode[j], target[j] = mode[i], target[i]
        pos[i] = (x, y)
        seen[i] = (x, y, heading[i], mode[i])
    frames.append(seen)

print(f'% Prob-EC event calculus: {N} people over {T} frames, queries every {a.every} frames, seed {a.seed}')
for act in ('walking', 'running', 'active'):
    print(f'moves({act}).')
print('init_person(P,T) :- happens(A,P,T), moves(A), \\+ disappear(P,T).')
print('term_person(P,T) :- disappear(P,T).')
print('holds_person(P,T1) :- next(T,T1), init_person(P,T).')
print('holds_person(P,T1) :- next(T,T1), holds_person(P,T), \\+ term_person(P,T).')
print('init_meeting(P,Q,T) :- happens(active,P,T), close25(P,Q,T), holds_person(Q,T), \\+ happens(running,Q,T).')
print('init_meeting(P,Q,T) :- holds_person(P,T), happens(inactive,P,T), close25(P,Q,T), holds_person(Q,T), '
      '\\+ happens(running,Q,T), \\+ happens(active,Q,T).')
print('term_meeting(P,Q,T) :- happens(walking,P,T), far34(P,Q,T).')
print('term_meeting(P,Q,T) :- happens(walking,Q,T), far34(P,Q,T).')
print('term_meeting(P,Q,T) :- pair(P,Q), happens(running,P,T).')
print('term_meeting(P,Q,T) :- pair(P,Q), happens(running,Q,T).')
print('term_meeting(P,Q,T) :- pair(P,Q), disappear(P,T).')
print('term_meeting(P,Q,T) :- pair(P,Q), disappear(Q,T).')
print('holds_meeting(P,Q,T1) :- next(T,T1), init_meeting(P,Q,T).')
print('holds_meeting(P,Q,T1) :- next(T,T1), holds_meeting(P,Q,T), \\+ term_meeting(P,Q,T).')
print('init_moving(P,Q,T) :- happens(walking,P,T), close34(P,Q,T), happens(walking,Q,T), orient45(P,Q,T).')
print('term_moving(P,Q,T) :- happens(walking,P,T), far34(P,Q,T).')
print('term_moving(P,Q,T) :- happens(walking,Q,T), far34(P,Q,T).')
print('term_moving(P,Q,T) :- pair(P,Q), happens(active,P,T), happens(active,Q,T).')
print('term_moving(P,Q,T) :- pair(P,Q), happens(active,P,T), happens(inactive,Q,T).')
print('term_moving(P,Q,T) :- pair(P,Q), happens(inactive,P,T), happens(active,Q,T).')
print('term_moving(P,Q,T) :- pair(P,Q), happens(running,P,T).')
print('term_moving(P,Q,T) :- pair(P,Q), happens(running,Q,T).')
print('term_moving(P,Q,T) :- pair(P,Q), disappear(P,T).')
print('term_moving(P,Q,T) :- pair(P,Q), disappear(Q,T).')
print('holds_moving(P,Q,T1) :- next(T,T1), init_moving(P,Q,T).')
print('holds_moving(P,Q,T1) :- next(T,T1), holds_moving(P,Q,T), \\+ term_moving(P,Q,T).')
for i in range(N):
    for j in range(N):
        if i != j:
            print(f'pair(p{i},p{j}).')
for t in range(1, T):
    print(f'next({t - 1},{t}).')
scene = set()  # the relations of the scene with a fact
for t, seen in enumerate(frames):
    for i, (x, y, o, act) in sorted(seen.items()):
        print(f'{round(rng.uniform(0.05, 0.99), 2)}::happens({act},p{i},{t}).')
        if t == last[i] and last[i] < T - 1:
            print(f'disappear(p{i},{t}).')
    for i, (x, y, o, _) in sorted(seen.items()):
        for j, (x2, y2, o2, _) in sorted(seen.items()):
            if i == j:
                continue
            d = math.hypot(x - x2, y - y2)
            for rel, holds in (('close25', d <= 25), ('close34', d <= 34),
                               ('orient45', d <= 34 and abs(o - o2) < 45), ('far34', d > 34)):
                if holds:
                    print(f'{rel}(p{i},p{j},{t}).')
                    scene.add(rel)
# A relation with no fact still has a clause, which ProbLog needs (tddlog
# reads a relation with none as empty).
for rel in sorted({'close25', 'close34', 'orient45', 'far34'} - scene):
    print(f'{rel}(p0,p0,0) :- fail.')
for t in range(a.every, T, a.every):
    for i in range(N):
        for j in range(N):
            if i != j:
                print(f'query(holds_meeting(p{i},p{j},{t})).')
                print(f'query(holds_moving(p{i},p{j},{t})).')
