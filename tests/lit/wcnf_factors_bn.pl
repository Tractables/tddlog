% log10 Z - log10 P(e) = 0.3979400086720376
% wcnf_factors_bn.cnf: weighted model counting, 14 variables (14 in clauses), 33 clauses; a network of 3 variables and 31 clauses in its tables
0.75::x1(v_0); 0.25::x1(v_1).
0.625::x0(v_1); 0.375::x0(v_2) :- x1(v_0).
0.5::x0(v_0); 0.125::x0(v_1); 0.375::x0(v_2) :- x1(v_1).
x2(v_1) :- x0(v_0), x1(v_0).
x2(v_0) :- x0(v_0), x1(v_1).
x2(v_1) :- x0(v_1), x1(v_0).
x2(v_0) :- x0(v_1), x1(v_1).
x2(v_1) :- x0(v_2), x1(v_0).
x2(v_1) :- x0(v_2), x1(v_1).
evidence(x2(v_1)).
0.3::x(13).
0.8::x(14).
violated(32) :- \+x(13), \+x0(v_0).
evidence(violated(32), false).
violated(33) :- x(14), x(13), \+x0(v_2).
evidence(violated(33), false).
