% diagnosis.rkt: the circuit Roulette builds, written by Roulette's tddlog engine.
% 4 flips, 9 gates, 1 observations, 4 queries.
% A value's weight is P(e) times its query's probability, both of which tddlog --evidence-probability prints; a failed observation's value answers 0 under the evidence.
% infer was called 4 times; query K is its K-th call.
1.0::f1.
0.0::f2.
0.1::f3.
0.2::f4.
g1 :- f1, f2.
g2 :- g1, \+f3.
g3 :- f3, \+g1.
g4 :- g2.
g4 :- g3.
g5 :- f2, \+g4.
g6 :- g4.
g6 :- g5.
g7 :- g6, \+f4.
g8 :- f4, \+g6.
g9 :- g7.
g9 :- g8.
evidence(g9, true).
% q1(1): #t
q1(1).
query(q1(1)).
% q2(1): #t
q2(1) :- f3.
query(q2(1)).
% q3(1): #t
q3(1) :- f4.
query(q3(1)).
% q4(1): #t
q4(1) :- f3, f4.
query(q4(1)).
