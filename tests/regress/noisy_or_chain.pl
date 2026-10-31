% Findings as chains of two-parent noisy-or tables, as tddlog reads the
% UAI 2022 or_chain networks (src/ground/noisy_or.rs). x1 and x2 chain f1
% over d1, d2 and d3; y1 chains f2 over d4 and d5; f3 is a noisy-or of d5
% and d6 with a leak, f4 a noisy-or of d6. f1, f2 and f4 are observed
% present and f3 absent, which is f3's present value observed false: every
% table is written as a noisy-or. ProbLog 2.3.0 and enumeration of the 64
% disease states in rationals: P(e) = 0.0048892872971448, P(d2(v_1) | e) =
% 0.35459094930658464, P(d4(v_1) | e) = 0.5876331336451859,
% P(d5(v_1) | e) = 0.19447122066033104. tests/it/evidence_probability.rs.
0.9::d1(v_0); 0.1::d1(v_1).
0.8::d2(v_0); 0.2::d2(v_1).
0.7::d3(v_0); 0.3::d3(v_1).
0.95::d4(v_0); 0.05::d4(v_1).
0.85::d5(v_0); 0.15::d5(v_1).
0.6::d6(v_0); 0.4::d6(v_1).
x1(v_0) :- d1(v_0), d2(v_0).
0.5::x1(v_0); 0.5::x1(v_1) :- d1(v_0), d2(v_1).
0.4::x1(v_0); 0.6::x1(v_1) :- d1(v_1), d2(v_0).
0.2::x1(v_0); 0.8::x1(v_1) :- d1(v_1), d2(v_1).
x2(v_0) :- d3(v_0), x1(v_0).
x2(v_1) :- d3(v_0), x1(v_1).
0.3::x2(v_0); 0.7::x2(v_1) :- d3(v_1), x1(v_0).
x2(v_1) :- d3(v_1), x1(v_1).
0.95::f1(v_0); 0.05::f1(v_1) :- x2(v_0).
f1(v_1) :- x2(v_1).
y1(v_0) :- d4(v_0), d5(v_0).
0.6::y1(v_0); 0.4::y1(v_1) :- d4(v_0), d5(v_1).
0.1::y1(v_0); 0.9::y1(v_1) :- d4(v_1), d5(v_0).
0.06::y1(v_0); 0.94::y1(v_1) :- d4(v_1), d5(v_1).
0.98::f2(v_0); 0.02::f2(v_1) :- y1(v_0).
f2(v_1) :- y1(v_1).
0.99::f3(v_0); 0.01::f3(v_1) :- d5(v_0), d6(v_0).
0.495::f3(v_0); 0.505::f3(v_1) :- d5(v_0), d6(v_1).
0.198::f3(v_0); 0.802::f3(v_1) :- d5(v_1), d6(v_0).
0.099::f3(v_0); 0.901::f3(v_1) :- d5(v_1), d6(v_1).
0.9::f4(v_0); 0.1::f4(v_1) :- d6(v_0).
0.3::f4(v_0); 0.7::f4(v_1) :- d6(v_1).
evidence(f1(v_1)).
evidence(f2(v_1)).
evidence(f3(v_0)).
evidence(f4(v_1)).
query(d2(v_1)).
query(d4(v_1)).
query(d5(v_1)).
