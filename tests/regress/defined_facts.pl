% A weighted CNF whose facts of probability one half c and d are functions
% of the others in its models: k(1) to k(4) say c = a xor b, k(5) to k(7)
% d = c and f, k(8) g or not d, and k(9) f implies a or b. g is not a
% function of the others: where d is false it is free. An independent
% support is {a, b, f, g}. d is eliminated by resolution
% (src/ground/preprocess/defined.rs), its one resolvent, with k(8), contracting d
% into g; then c, whose resolvents join a and b to g alone (k(9) joins
% them to f), contracting c into g. Each takes a half.
% P(e) = 1/4 * (0.18 + 0.12 * 0.6 + 0.42 * 0.6 + 0.28 * 0.2) = 0.14, where
% c and f hold d and g do; P(a | e) = 0.063 / 0.14, P(c | e) = 0.081 / 0.14,
% P(g | e) = 0.097 / 0.14. A query of c keeps it: d alone is eliminated.
0.3::a.
0.6::b.
0.5::c.
0.5::d.
0.8::f.
0.5::g.
k(1) :- c, \+a, \+b.
k(2) :- c, a, b.
k(3) :- \+c, a, \+b.
k(4) :- \+c, \+a, b.
k(5) :- d, \+c.
k(6) :- d, \+f.
k(7) :- \+d, c, f.
k(8) :- \+g, d.
k(9) :- f, \+a, \+b.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
evidence(k(5), false).
evidence(k(6), false).
evidence(k(7), false).
evidence(k(8), false).
evidence(k(9), false).
query(a).
query(c).
query(g).
