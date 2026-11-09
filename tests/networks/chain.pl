% log10 Z - log10 P(e) = -4.342945038715477e-08
% `tddlog --program --query leaves chain.uai`, a UAI BAYES network read as
% written: each row is divided by its sum, and each table's sums are a
% factor over its parents. x0's row, 0.3333333 three times, sums to a
% constant, which the offset holds; x1's rows sum to 1, 0.99 and 1, and
% x2's to 1 and 0.75, which the factors f0 over x0 and f1 over x1 hold.
0.33333333333333337::x0(v_0); 0.33333333333333337::x0(v_1); 0.33333333333333337::x0(v_2).
0.5::x1(v_0); 0.5::x1(v_1) :- x0(v_0).
0.9090909090909092::x1(v_0); 0.09090909090909091::x1(v_1) :- x0(v_1).
1e-05::x1(v_0); 0.99999::x1(v_1) :- x0(v_2).
0.55::x2(v_0); 0.34::x2(v_1); 0.11::x2(v_2) :- x1(v_0).
0.3333333333333333::x2(v_1); 0.6666666666666666::x2(v_2) :- x1(v_1).
f0(ok) :- x0(v_0).
0.99::f0(ok) :- x0(v_1).
f0(ok) :- x0(v_2).
f1(ok) :- x1(v_0).
0.75::f1(ok) :- x1(v_1).
evidence(f0(ok)).
evidence(f1(ok)).
query(x2(v_0)).
query(x2(v_1)).
query(x2(v_2)).
