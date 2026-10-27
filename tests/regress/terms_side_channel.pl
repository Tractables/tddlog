%Expected outcome:
% masked(x,k,band(12,6)) 0.8
% masked(x,y,band(12,10)) 0.8
% masked(y,k,band(10,6)) 0.6
% masked(y,x,band(10,12)) 0.6
% leak(x) 0.24
% leak(y) 0.18
% step(12,12-1) 1
% countdown(x) 0.8
% nonzero(10-1) 1
% nonzero(12-1) 1
% nonzero(6-1) 1
% zero(x,0) 0.8
% zero(y,0) 0.6
% gap(x,y,2) 0.6
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% Terms built by `=`, as a side-channel analysis written for ProbLog has
% them: `T = band(X, Y)` and `N1 = (N - 1)` build terms, which `=`
% compares as terms and `is` and `=:=` evaluate.
0.8::rand(x). 0.6::rand(y). 0.3::key(k).
var(x, 12). var(y, 10). var(k, 6).

masked(A, B, T) :- var(A, X), var(B, Y), A \== B, rand(A), (T = band(X, Y)).
leak(A) :- masked(A, k, band(_, Y)), var(k, Y), key(k).
step(N, N1) :- var(_, N), (N1 = (N - 1)).
countdown(A) :- var(A, N), step(N, N1), N1 =:= 11, rand(A).
nonzero(N1) :- step(_, N1), \+((N1 = 0)).
zero(A, XY) :- rand(A), (XY = 0).
gap(A, B, D) :- var(A, X), var(B, Y), X > Y, rand(B), D is X - Y.

query(masked(_, _, _)).
query(leak(_)).
query(step(12, _)).
query(countdown(_)).
query(nonzero(_)).
query(zero(_, _)).
query(gap(_, _, _)).
