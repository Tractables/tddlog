% log10 Z - log10 P(e) = 3.8715729355458786
% `tddlog --program markov_pr.uai`, a Markov network over x0 (2
% values), x1 (3), x2 (2), x3 (2) and x4 (3): two unary factors on x0, one
% on x1 with a zero, a factor on x0,x1, one on x1,x2,x3 with a zero, one
% on x2,x4, one on x3,x4 that is constant 2.5 and one of empty scope, 4.
% By enumeration of the 72 worlds in exact fractions, Z = 19947/25 = 797.88,
% log10 Z = 2.901937578999016.
0.6::x0(v_0); 0.4::x0(v_1).
0.25::x1(v_1); 0.75::x1(v_2).
0.5::x2(v_0); 0.5::x2(v_1).
0.5::x3(v_0); 0.5::x3(v_1).
0.3333333333333333::x4(v_0); 0.3333333333333333::x4(v_1); 0.3333333333333333::x4(v_2).
0.6::f0(ok) :- x0(v_0), x1(v_0).
0.2::f0(ok) :- x0(v_0), x1(v_1).
f0(ok) :- x0(v_0), x1(v_2).
0.35::f0(ok) :- x0(v_1), x1(v_0).
0.55::f0(ok) :- x0(v_1), x1(v_1).
0.15::f0(ok) :- x0(v_1), x1(v_2).
0.2903225806451613::f1(ok) :- x1(v_0), x2(v_0), x3(v_0).
0.4516129032258064::f1(ok) :- x1(v_0), x2(v_0), x3(v_1).
0.06451612903225806::f1(ok) :- x1(v_0), x2(v_1), x3(v_0).
0.8387096774193549::f1(ok) :- x1(v_0), x2(v_1), x3(v_1).
0.3225806451612903::f1(ok) :- x1(v_1), x2(v_0), x3(v_0).
f1(ok) :- x1(v_1), x2(v_1), x3(v_0).
0.25806451612903225::f1(ok) :- x1(v_1), x2(v_1), x3(v_1).
0.16129032258064516::f1(ok) :- x1(v_2), x2(v_0), x3(v_0).
0.5483870967741935::f1(ok) :- x1(v_2), x2(v_0), x3(v_1).
0.7096774193548387::f1(ok) :- x1(v_2), x2(v_1), x3(v_0).
0.1935483870967742::f1(ok) :- x1(v_2), x2(v_1), x3(v_1).
0.15::f2(ok) :- x2(v_0), x4(v_0).
0.45::f2(ok) :- x2(v_0), x4(v_1).
0.75::f2(ok) :- x2(v_0), x4(v_2).
f2(ok) :- x2(v_1), x4(v_0).
0.05::f2(ok) :- x2(v_1), x4(v_1).
0.35::f2(ok) :- x2(v_1), x4(v_2).
evidence(f0(ok)).
evidence(f1(ok)).
evidence(f2(ok)).
query(x0(v_0)).
query(x0(v_1)).
query(x1(v_0)).
query(x1(v_1)).
query(x1(v_2)).
query(x2(v_0)).
query(x2(v_1)).
query(x3(v_0)).
query(x3(v_1)).
query(x4(v_0)).
query(x4(v_1)).
query(x4(v_2)).
