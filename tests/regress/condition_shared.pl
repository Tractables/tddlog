% A true atom whose live rules share a literal. With c false, x holds only
% through its first two rules, and both read y: the evidence fixes y, a
% fact. x is then y and (a or b), and P(e) = 0.8 * 0.3 * (1 - 0.6 * 0.5)
% = 0.168; given the evidence y is certain and P(a) = 0.096 / 0.168 = 4/7.
0.3::y.
0.4::a.
0.5::b.
0.2::c.
x :- y, a.
x :- y, b.
x :- c.
evidence(x).
evidence(c, false).
query(y).
query(a).
