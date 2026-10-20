% Evidence in two parts that no query shares a fact with, beside a query
% whose own part is observed. Propagation settles neither detached part: z
% and d each have rules that could hold. ProbLog 2.3.0: q = 0.12 / 0.82,
% the detached parts cancelling.
0.3::a. 0.6::b.
q :- a.
r :- a, b.
evidence(r, false).
query(q).
0.2::x; 0.5::y.
0.4::w.
z :- x.
z :- y, \+w.
evidence(z, true).
0.5::c1. 0.5::c2. 0.5::c3.
d :- c1, \+c2.
d :- c2, \+c3.
d :- c3, \+c1.
evidence(d, true).
e :- c1, c2, c3.
evidence(e, false).
