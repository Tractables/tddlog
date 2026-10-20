% A weighted CNF whose gate a clause reads: g, of probability one half, is
% the OR of a and b (k(1) to k(3)), and k(4) says g implies c. Taken as a
% derived atom, g would make the program read a derived atom; it is left a
% fact with its clauses (src/ground/condition.rs, gates), and the program
% stays a CNF over its facts. P(e) = 0.5 * (P(not a, not b) + P(a or b)
% P(c)) = 0.5 * (0.32 + 0.68 * 0.3) = 0.262; P(c | e) = 0.15 / 0.262,
% P(a | e) = 0.03 / 0.262.
0.2::a.
0.6::b.
0.3::c.
0.5::g.
k(1) :- a, \+g.
k(2) :- b, \+g.
k(3) :- g, \+a, \+b.
k(4) :- g, \+c.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
query(c).
query(a).
