% tiny_pr.rkt: the circuit Roulette builds, written by tddlog's scripts/roulette2problog.rkt.
% 3 flips, 8 gates, 1 observations, 11 queries.
0.3::f1.
0.6::f2.
0.9::f3.
g1 :- f2, \+f1.
g2 :- f1, f3.
g3 :- g1.
g3 :- g2.
g4 :- f1.
g4 :- g1.
g5 :- f1, \+f2.
g6 :- f1, f2.
g7 :- \+f1, \+f2.
g8 :- f1, g3.
g8 :- \+f1, \+g3.
evidence(g4, true).
% q1(0): #f
q1(0) :- \+g3.
query(q1(0)).
% q1(1): #t
q1(1) :- g3.
query(q1(1)).
% q2(0): (#f #f)
q2(0) :- g1.
query(q2(0)).
% q2(1): (#f #t)
q2(1) :- g7.
query(q2(1)).
% q2(2): (#t #f)
q2(2) :- g6.
query(q2(2)).
% q2(3): (#t #t)
q2(3) :- g5.
query(q2(3)).
% q3(0): 1
q3(0) :- f1.
query(q3(0)).
% q3(1): 2
q3(1) :- g1.
query(q3(1)).
% q3(2): 3
q3(2) :- g7.
query(q3(2)).
% q4(0): #f
q4(0) :- \+g8.
query(q4(0)).
% q4(1): #t
q4(1) :- g8.
query(q4(1)).
