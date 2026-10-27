% log10 Z - log10 P(e) = 2.9822712330395684
% `tddlog --program markov.uai`, a Markov network over x0 (2 values),
% x1 (2) and x2 (3) with factors x0: (1 3); x0,x1: (2 1 1 2);
% x1,x2: (1 2 3 4 0 1); and x0,x2 constant 5, which is dropped. The
% marginals by enumeration of the twelve worlds, Z = 65 * 5:
% x0: 17/65 48/65; x1: 6/13 7/13; x2: 33/65 2/13 22/65.
% The program divides Z by 4 (x0's unary factor), 2 and 3 (x1's and x2's
% uniform priors), 2 and 4 (the two factors' largest entries) and 5 (the
% constant factor), 960 in all: P(e) = 325/960 = 65/192.
0.25::x0(v_0); 0.75::x0(v_1).
0.5::x1(v_0); 0.5::x1(v_1).
0.3333333333333333::x2(v_0); 0.3333333333333333::x2(v_1); 0.3333333333333333::x2(v_2).
f0(ok) :- x0(v_0), x1(v_0).
0.5::f0(ok) :- x0(v_0), x1(v_1).
0.5::f0(ok) :- x0(v_1), x1(v_0).
f0(ok) :- x0(v_1), x1(v_1).
0.25::f1(ok) :- x1(v_0), x2(v_0).
0.5::f1(ok) :- x1(v_0), x2(v_1).
0.75::f1(ok) :- x1(v_0), x2(v_2).
f1(ok) :- x1(v_1), x2(v_0).
0.25::f1(ok) :- x1(v_1), x2(v_2).
evidence(f0(ok)).
evidence(f1(ok)).
query(x0(v_0)).
query(x0(v_1)).
query(x1(v_0)).
query(x1(v_1)).
query(x2(v_0)).
query(x2(v_1)).
query(x2(v_2)).
