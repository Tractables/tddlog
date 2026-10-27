% gen_bn.py's program for chain.uai --query leaves, a UAI BAYES network: x0's
% row 0.3333333 three times is divided by its sum, which rounding explains,
% as is x1's 0.9, 0.09; x2's 0.0, 0.25, 0.5 is kept, its rest left to no value.
0.33333333333333337::x0(v_0); 0.33333333333333337::x0(v_1); 0.33333333333333337::x0(v_2).
0.5::x1(v_0); 0.5::x1(v_1) :- x0(v_0).
0.9090909090909092::x1(v_0); 0.09090909090909091::x1(v_1) :- x0(v_1).
1e-05::x1(v_0); 0.99999::x1(v_1) :- x0(v_2).
0.55::x2(v_0); 0.34::x2(v_1); 0.11::x2(v_2) :- x1(v_0).
0.25::x2(v_1); 0.5::x2(v_2) :- x1(v_1).
query(x2(v_0)).
query(x2(v_1)).
query(x2(v_2)).
