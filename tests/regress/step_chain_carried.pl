%Expected outcome:
% q 0.2436
% b(4) 0.5
% c(4) 0.4872
%
% A loop written as recursion over a time index. The bit b, which a
% switch sets at step 0, every step after carries unchanged: each b(T) is
% read once, by the step after, and is inlined there, and each step's
% copy of it is that switch's literal, which the copies are merged into
% the first of. The bit c is the parity of the switches g(0) to g(3), read
% positively and negated by each step, and stays: b(4) is f(0), 0.5, and
% c(4) holds where an odd number of four independent switches of 0.3 do,
% (1 - 0.4^4) / 2 = 0.4872, which ProbLog 2.3.0 gives.

0.5::f(T) :- step(T).
0.3::g(T) :- step(T).
step(0). step(1). step(2). step(3).
b(1) :- f(0).
b(T1) :- step(T), T > 0, T1 is T + 1, b(T).
c(1) :- g(0).
c(T1) :- step(T), T > 0, T1 is T + 1, c(T), \+g(T).
c(T1) :- step(T), T > 0, T1 is T + 1, \+c(T), g(T).
q :- b(4), c(4).
query(q).
query(b(4)).
query(c(4)).
