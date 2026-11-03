% tiny.rkt: the circuit Roulette builds, written by Roulette's tddlog engine.
% 3 flips, 3 gates, 0 observations, 8 queries.
% A value's weight is P(e) times its query's probability, both of which tddlog --evidence-probability prints; a failed observation's value answers 0 under the evidence.
% infer was called 4 times; query K is its K-th call.
0.3::f1.
0.6::f2.
0.9::f3.
g1 :- f2, \+f1.
g2 :- f1, f3.
g3 :- g1.
g3 :- g2.
% q1(0): #f
q1(0) :- \+g3.
query(q1(0)).
% q1(1): #t
q1(1) :- g3.
query(q1(1)).
% q2(0): (#f #f)
q2(0) :- f2, \+f1.
query(q2(0)).
% q2(1): (#f #t)
q2(1) :- \+f1, \+f2.
query(q2(1)).
% q2(2): (#t #f)
q2(2) :- f1, f2.
query(q2(2)).
% q2(3): (#t #t)
q2(3) :- f1, \+f2.
query(q2(3)).
% q3(1): #t
q3(1) :- f1, f2.
query(q3(1)).
% q3(2): none
q3(2) :- fail.
query(q3(2)).
