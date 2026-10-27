% tiny_expectation.jl: the circuit Alea.jl builds, written by tddlog's scripts/alea2problog.jl.
% 4 flips, 15 gates, 1 observations, 4 queries.
0.6666666666666666::f1.
0.5::f2.
0.5::f3.
0.5::f4.
g1 :- f4, \+f1.
g2 :- f1, f3.
g3 :- g2.
g3 :- g1.
g4 :- f1, f2.
g5 :- g4, g3.
g6 :- g5.
g6 :- \+f1.
g7 :- f1, g5.
g8 :- f1.
g8 :- g5.
g9 :- g8, \+g7.
g10 :- g4, g3.
g11 :- g4.
g11 :- g3.
g12 :- g11, \+g10.
g13 :- g6, \+g9.
g14 :- g13, \+g12.
g15 :- g14, g3.
evidence(g15, false).
e1(1) :- \+g6.
query(e1(1)).
e1(2) :- g9.
query(e1(2)).
e1(3) :- g12.
query(e1(3)).
e1(4) :- \+g3.
query(e1(4)).
