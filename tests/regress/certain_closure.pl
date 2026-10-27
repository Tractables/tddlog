%Expected outcome:
% q 0.5
% r 0.2
% s 0
% t 0.4
%
% The expected outcomes are computed by hand: eq is the closure of the
% deterministic facts e, true in every world, so it links a to f both
% ways and b to e; q is p, 0.5, r is p and w, 0.2, s needs eq(b, e)
% false and is never true, and t is w alone, its second rule needing
% eq(a, g), which nothing derives.
e(a, b).
e(b, c).
e(c, d).
e(d, e).
e(e, f).
e(g, h).
eq(X, Y) :- e(X, Y).
eq(X, Y) :- e(Y, X).
eq(X, Z) :- eq(X, Y), eq(Y, Z).
0.5::p.
0.4::w.
q :- eq(a, f), p.
r :- eq(f, a), p, w.
s :- p, \+ eq(b, e).
t :- w, eq(c, c).
t :- p, eq(a, g).
query(q).
query(r).
query(s).
query(t).
