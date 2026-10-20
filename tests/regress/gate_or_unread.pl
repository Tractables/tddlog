% Clauses observed true, as a weighted CNF's program has them. g, a fact of
% probability one half, is the OR of x(1) and x(2): c(1) to c(3) say so
% (src/ground/condition.rs, gates). Nothing else reads g, so the gate goes
% with its definition and x(2) with it, and summing g out leaves one half.
% c(4) wants x(1) or x(3). P(e) = 0.5 * (1 - 0.8 * 0.7) = 0.22; P(x(1), e)
% = 0.5 * 0.2 = 0.1, P(x(1) | e) = 0.1 / 0.22.
0.2::x(1).
0.6::x(2).
0.3::x(3).
0.5::g.
c(1) :- x(1), \+g.
c(2) :- x(2), \+g.
c(3) :- g, \+x(1), \+x(2).
c(4) :- \+x(1), \+x(3).
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
query(x(1)).
