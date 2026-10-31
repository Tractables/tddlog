%Expected outcome:
% w(0) 0.5
% v(1) 0.5
% v(2) 0.5
%
% ProbLog makes a test as the terms stand when the body reaches it.
% get(s, N) calls member(count(s, N), ...) with N unbound, and
% count(s, N) \= count(s, 0) fails there: member/2 written so stops at
% the first count of s, and w(N) holds for N = 0 only, not for 2. v(X)
% negates p(Y) after q(X, Y) binds Y, which reads it on each value. The
% answers are ProbLog's.
0.5::f.
member(A, [A|_]).
member(A, [H|T]) :- A \= H, member(A, T).
get(X, N) :- member(count(X, N), [count(s, 0), count(m, 1), count(s, 2)]).
w(N) :- f, get(s, N).
q(1, a). q(2, b). q(3, c).
p(c).
v(X) :- q(X, Y), \+ p(Y), f.
query(w(_)).
query(v(_)).
