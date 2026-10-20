% log10 Z - log10 P(e) = 3.260667536990012
% gen_bn.py's program for potts.uai with --query none, a Markov network
% over x0..x4 of three values each, all its factors Potts tables: the
% colourings x0-x1, x3-x4 and x4-x0 (0 on the diagonal, 1 off it), x1-x2
% (2, 1), x2-x3 (0.5, 1.5) and x0-x2 (1.25, 0.75). The program breaks the
% colour symmetry (gen_bn.colour_symmetry): x0 is 0, x2 is 0 or 1 with
% weights 1 and 2, and given those x1 takes the values used and the next
% one; the offset holds log10 3 besides. By enumeration of the 243 worlds
% of potts.uai in exact fractions, Z = 225/2, log10 Z = 2.0511525224473814.
x0(v_0).
0.3333333333333333::x1(v_0); 0.3333333333333333::x1(v_1); 0.3333333333333333::x1(v_2).
0.3333333333333333::x2(v_0); 0.6666666666666666::x2(v_1).
0.3333333333333333::x3(v_0); 0.3333333333333333::x3(v_1); 0.3333333333333333::x3(v_2).
0.3333333333333333::x4(v_0); 0.3333333333333333::x4(v_1); 0.3333333333333333::x4(v_2).
f0(ok) :- x0(v_0), x1(v_1).
f0(ok) :- x0(v_0), x1(v_2).
f0(ok) :- x0(v_1), x1(v_0).
f0(ok) :- x0(v_1), x1(v_2).
f0(ok) :- x0(v_2), x1(v_0).
f0(ok) :- x0(v_2), x1(v_1).
f1(ok) :- x1(v_0), x2(v_0).
0.5::f1(ok) :- x1(v_0), x2(v_1).
0.5::f1(ok) :- x1(v_0), x2(v_2).
0.5::f1(ok) :- x1(v_1), x2(v_0).
f1(ok) :- x1(v_1), x2(v_1).
0.5::f1(ok) :- x1(v_1), x2(v_2).
0.5::f1(ok) :- x1(v_2), x2(v_0).
0.5::f1(ok) :- x1(v_2), x2(v_1).
f1(ok) :- x1(v_2), x2(v_2).
0.3333333333333333::f2(ok) :- x2(v_0), x3(v_0).
f2(ok) :- x2(v_0), x3(v_1).
f2(ok) :- x2(v_0), x3(v_2).
f2(ok) :- x2(v_1), x3(v_0).
0.3333333333333333::f2(ok) :- x2(v_1), x3(v_1).
f2(ok) :- x2(v_1), x3(v_2).
f2(ok) :- x2(v_2), x3(v_0).
f2(ok) :- x2(v_2), x3(v_1).
0.3333333333333333::f2(ok) :- x2(v_2), x3(v_2).
f3(ok) :- x3(v_0), x4(v_1).
f3(ok) :- x3(v_0), x4(v_2).
f3(ok) :- x3(v_1), x4(v_0).
f3(ok) :- x3(v_1), x4(v_2).
f3(ok) :- x3(v_2), x4(v_0).
f3(ok) :- x3(v_2), x4(v_1).
f4(ok) :- x4(v_0), x0(v_0).
0.6::f4(ok) :- x4(v_0), x0(v_1).
0.6::f4(ok) :- x4(v_0), x0(v_2).
0.6::f4(ok) :- x4(v_1), x0(v_0).
f4(ok) :- x4(v_1), x0(v_1).
0.6::f4(ok) :- x4(v_1), x0(v_2).
0.6::f4(ok) :- x4(v_2), x0(v_0).
0.6::f4(ok) :- x4(v_2), x0(v_1).
f4(ok) :- x4(v_2), x0(v_2).
f5(ok) :- x0(v_0), x2(v_1).
f5(ok) :- x0(v_0), x2(v_2).
f5(ok) :- x0(v_1), x2(v_0).
f5(ok) :- x0(v_1), x2(v_2).
f5(ok) :- x0(v_2), x2(v_0).
f5(ok) :- x0(v_2), x2(v_1).
0.5::f6(ok) :- x2(v_0), x1(v_0).
f6(ok) :- x2(v_0), x1(v_1).
0.5::f6(ok) :- x2(v_1), x1(v_0).
0.5::f6(ok) :- x2(v_1), x1(v_1).
0.5::f6(ok) :- x2(v_1), x1(v_2).
evidence(f0(ok)).
evidence(f1(ok)).
evidence(f2(ok)).
evidence(f3(ok)).
evidence(f4(ok)).
evidence(f5(ok)).
evidence(f6(ok)).
