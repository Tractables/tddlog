%Expected outcome:
% h(1) 0
% h2(1) 0.12
% h2(2) 0
%
% The expected outcomes are computed by hand: b never holds, so h(1) is
% false; h2(1) needs a(1), d(5) and c(1, 3), 0.5 * 0.4 * 0.6, and h2(2)
% needs a(2), which nothing derives. The demand of c reads a, which binds
% its first argument, and not b or d, which bind nothing it reads.
0.5::a(1).
0.4::d(5).
b(7) :- d(9).
0.6::c(1, 3).
0.7::c(2, 4).
h(X) :- a(X), b(Y), c(X, Z).
h2(X) :- a(X), d(Y), c(X, Z).
query(h(1)).
query(h2(1)).
query(h2(2)).
