% gate_or_unread.pl with g of probability 0.4: summing g out leaves 0.4 or
% 0.6 by the value of x(1) or x(2), and the gate stays a fact with its
% clauses (src/ground/condition.rs, gates). P(e) = 0.4 * P(x(1) or x(2) x(3))
% + 0.6 * P(not x(1), not x(2), x(3)) = 0.4 * 0.344 + 0.6 * 0.096 = 0.1952;
% P(g | e) = 0.1376 / 0.1952.
0.2::x(1).
0.6::x(2).
0.3::x(3).
0.4::g.
c(1) :- x(1), \+g.
c(2) :- x(2), \+g.
c(3) :- g, \+x(1), \+x(2).
c(4) :- \+x(1), \+x(3).
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
query(g).
