% log10 Z - log10 P(e) = 2.4593924877592306
% wcnf_factors_mn.cnf: weighted model counting, 19 variables (19 in clauses), 43 clauses; a network of 3 variables and 42 clauses in its tables
0.25::x0(v_0); 0.625::x0(v_1); 0.125::x0(v_2).
0.3333333333333333::x1(v_0); 0.3333333333333333::x1(v_1); 0.3333333333333333::x1(v_2).
0.5::x2(v_0); 0.5::x2(v_1).
0.6666666666666666::f0(ok) :- x0(v_0), x1(v_0).
0.5::f0(ok) :- x0(v_0), x1(v_2).
0.3333333333333333::f0(ok) :- x0(v_1), x1(v_0).
0.16666666666666666::f0(ok) :- x0(v_1), x1(v_1).
f0(ok) :- x0(v_2), x1(v_0).
0.3333333333333333::f0(ok) :- x0(v_2), x1(v_1).
0.6666666666666666::f0(ok) :- x0(v_2), x1(v_2).
0.375::f1(ok) :- x1(v_0), x2(v_0).
0.25::f1(ok) :- x1(v_0), x2(v_1).
0.5::f1(ok) :- x1(v_1), x2(v_1).
f1(ok) :- x1(v_2), x2(v_0).
0.125::f1(ok) :- x1(v_2), x2(v_1).
evidence(f0(ok)).
evidence(f1(ok)).
0.4::x(19).
violated(43) :- \+x(19), x2(v_1), \+x0(v_0).
evidence(violated(43), false).
