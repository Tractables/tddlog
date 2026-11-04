%Expected outcome:
% r(1) 0.3
% r(9) 0.3
% r(3.5) 0.3
% r(0) 0.0
% s 0.3
% t(4) 0.51
%
% Joins whose atom is reached by a later test `V is E`, `V` bound and `E`
% over the atom's variables alone: the grounder takes the atom's tuples from
% an index of its relation by the value of `E`, and the test still decides.
% `a + 1` is undefined, so num(a) is in no list; 2.5 + 1 is the float 3.5;
% `5 is X * 2 - 1` tests against a constant; `X + Z` reads both arguments
% of pair. The answers, by hand, are of the independent facts of p: r(Y) is
% p(Y - 1), r(0) has no rule, s is p(3), and t(4) holds through pair(1, 3)
% and pair(2, 2), p(1) or p(2): 1 - 0.7 * 0.7.

num(0). num(1). num(2). num(3). num(4). num(5). num(6). num(7). num(8). num(9).
num(a). num(2.5). num(3.5).
0.3::p(X) :- num(X).
r(Y) :- num(Y), num(X), Y is X + 1, p(X).
s :- num(X), 5 is X * 2 - 1, p(X).
pair(1, 3). pair(2, 2). pair(3, 0).
t(Y) :- num(Y), pair(X, Z), Y is X + Z, p(X).
query(r(1)).
query(r(9)).
query(r(3.5)).
query(r(0)).
query(s).
query(t(4)).
