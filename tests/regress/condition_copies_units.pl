% A weighted CNF of two units, its clauses observed true as a weighted CNF's
% program has them: c(1) and c(2) make x(1) and x(2) equal, and the
% conditioning makes one a copy of the other (src/ground/condition.rs),
% which c(3) reads; c(4) shares no fact with the others. Where x(1) = x(2)
% = true, weight 0.3 * 0.6 = 0.18, c(3) wants x(5) false: 0.18 * 0.3 =
% 0.054; where both are false, 0.7 * 0.4 = 0.28. c(4) leaves x(4) and x(6)
% not both true: 1 - 0.5 * 0.4 = 0.8. P(e) = (0.054 + 0.28) * 0.8 =
% 0.2672; P(x(5), e) = 0.28 * 0.7 * 0.8 = 0.1568.
0.3::x(1).
0.6::x(2).
0.5::x(4).
0.7::x(5).
0.4::x(6).
c(1) :- x(1), \+x(2).
c(2) :- x(2), \+x(1).
c(3) :- x(2), x(5).
c(4) :- x(4), x(6).
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
query(x(5)).
