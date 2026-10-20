% gate_nested.pl with nothing reading h: k(7) wants a or c. h goes with its
% definition, then g, which only h read, and b, which only g read
% (src/ground/condition.rs, gates). P(e) = 0.25 * (1 - 0.8 * 0.7) = 0.11;
% P(a | e) = 0.2 / 0.44.
0.5::h.
0.2::a.
0.6::b.
0.3::c.
0.5::g.
k(1) :- h, \+g.
k(2) :- h, \+c.
k(3) :- \+h, g, c.
k(4) :- a, \+g.
k(5) :- b, \+g.
k(6) :- g, \+a, \+b.
k(7) :- \+a, \+c.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
evidence(k(5), false).
evidence(k(6), false).
evidence(k(7), false).
query(a).
