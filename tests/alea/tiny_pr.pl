% tiny_pr.jl: the circuit Alea.jl builds, written by tddlog's scripts/alea_bench.jl (baselines branch).
% 4 flips, 9 gates, 1 observations, 9 queries.
0.3::f1.
0.6::f2.
0.5::f3.
0.5::f4.
g1 :- f1, \+f2.
g2 :- f4, \+f1.
g3 :- f1.
g3 :- g2.
g4 :- f3, \+f1.
g5 :- g4, g3.
g6 :- g4, g3.
g7 :- g4.
g7 :- g3.
g8 :- g7, \+g6.
g9 :- f1.
g9 :- f2.
evidence(g9, true).
q1 :- g1.
query(q1).
q2(0) :- \+g5, \+g8, g3.
query(q2(0)).
q2(1) :- \+g5, \+g8, \+g3.
query(q2(1)).
q2(2) :- \+g5, g8, g3.
query(q2(2)).
q2(3) :- \+g5, g8, \+g3.
query(q2(3)).
q2(4) :- g5, \+g8, g3.
query(q2(4)).
q2(5) :- g5, \+g8, \+g3.
query(q2(5)).
q2(6) :- g5, g8, g3.
query(q2(6)).
q2(7) :- g5, g8, \+g3.
query(q2(7)).
