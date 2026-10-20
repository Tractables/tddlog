% Clauses observed true, as a weighted CNF's program has them: c(1) and c(2)
% make x(1) and x(2) equal, and the conditioning makes one a copy of the
% other (src/ground/condition.rs), which the three-literal clauses c(3) and
% c(4) read. Where x(1) = x(2) = true, weight 0.3 * 0.6 = 0.18, c(4) leaves
% x(3) and x(4) not both true: 0.18 * (1 - 0.2 * 0.5) = 0.162; where both
% are false, 0.7 * 0.4 = 0.28, c(3) wants x(3) or x(4): 0.28 * (1 - 0.8 *
% 0.5) = 0.168. P(e) = 0.33; P(x(3), e) = 0.18 * 0.2 * 0.5 + 0.28 * 0.2 =
% 0.074, P(x(4), e) = 0.18 * 0.8 * 0.5 + 0.28 * 0.5 = 0.212.
0.3::x(1).
0.6::x(2).
0.2::x(3).
0.5::x(4).
c(1) :- x(1), \+x(2).
c(2) :- x(2), \+x(1).
c(3) :- \+x(2), \+x(3), \+x(4).
c(4) :- x(1), x(3), x(4).
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
query(x(1)).
query(x(3)).
query(x(4)).
