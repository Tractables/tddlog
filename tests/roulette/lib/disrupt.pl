% disrupt.rkt: the circuit Roulette builds, written by tddlog's scripts/roulette2problog.rkt.
% 2 flips, 2 gates, 0 observations, 6 queries.
% A value's weight is P(e) times its query's probability, both of which tddlog --evidence-probability prints.
% infer was called 2 times; query K is its K-th call.
0.3::f1.
0.6::f2.
g1 :- f2, \+f1.
g2 :- f1.
g2 :- g1.
% q1(0): x
q1(0) :- f1.
query(q1(0)).
% q1(1): y
q1(1) :- f2, \+f1.
query(q1(1)).
% q1(2): z
q1(2) :- g2, \+f1, \+f2.
query(q1(2)).
% q1(3): ⊥16263
q1(3) :- \+g2.
query(q1(3)).
% q2(0): #f
q2(0) :- \+g2.
query(q2(0)).
% q2(1): #t
q2(1) :- g2.
query(q2(1)).
