% tiny_path.rkt: the circuit Roulette builds, written by tddlog's scripts/roulette2problog.rkt.
% 2 flips, 2 gates, 1 observations, 3 queries.
0.5::f1.
0.25::f2.
g1 :- f1, f2.
g2 :- \+f1.
g2 :- g1.
evidence(g2, true).
% q1(0): #f
q1(0) :- \+f1.
query(q1(0)).
% q1(1): #t
q1(1) :- f1.
query(q1(1)).
% q2(0): 7
q2(0).
query(q2(0)).
