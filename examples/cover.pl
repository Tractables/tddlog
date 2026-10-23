% A derived atom whose rule bodies together are valid is certain
% (src/ground/cover.rs).
%
% u's rows all hold one distribution and cover every value of a and b:
% its class atom is certain, u has no parents left and b is read by
% nothing.
% Run with `tddlog examples/cover.pl`; `tddlog --ground examples/cover.pl`
% prints the program the grounder writes.
0.3::a(1); 0.7::a(2).
0.4::b(1); 0.6::b(2).
0.2::u(1); 0.8::u(2) :- a(1), b(1).
0.2::u(1); 0.8::u(2) :- a(1), b(2).
0.2::u(1); 0.8::u(2) :- a(2), b(1).
0.2::u(1); 0.8::u(2) :- a(2), b(2).
% u's values are exhaustive once its class atom is certain, and t's rows,
% one distribution over every value of u and a, have a certain class atom
% in turn.
0.9::t(1); 0.1::t(2) :- u(1), a(1).
0.9::t(1); 0.1::t(2) :- u(1), a(2).
0.9::t(1); 0.1::t(2) :- u(2), a(1).
0.9::t(1); 0.1::t(2) :- u(2), a(2).
% w's rows do not cover a(2), b(2): its class atom stays.
0.25::w(1); 0.75::w(2) :- a(1), b(1).
0.25::w(1); 0.75::w(2) :- a(1), b(2).
0.25::w(1); 0.75::w(2) :- a(2), b(1).
% e's rows sum below one: e's values are not exhaustive, and y, which
% reads each of them, is not certain.
0.5::e(1); 0.3::e(2) :- a(1).
0.5::e(1); 0.4::e(2) :- a(2).
y :- e(1).
y :- e(2).
% z reads a fact and its negation; z2 as well, under a(2).
0.35::f.
z :- f.
z :- \+f.
z2 :- a(2), f.
z2 :- a(2), \+f.
query(u(1)).
query(t(1)).
query(w(1)).
query(y).
query(z).
query(z2).
