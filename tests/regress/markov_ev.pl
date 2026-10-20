% log10 Z - log10 P(e) = 2.9822712330395684
% gen_bn.py's program for markov.uai given markov.uai.evid (x2 = 0),
% querying x0 and x1. The marginals by enumeration of the four worlds left:
% x0: 2/11 9/11; x1: 5/33 28/33. Given x2 = 0, Z = 33 * 5 and
% P(e) = 165/960 = 33/192.
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
evidence(x2(v_0)).
query(x0(v_0)).
query(x0(v_1)).
query(x1(v_0)).
query(x1(v_1)).
