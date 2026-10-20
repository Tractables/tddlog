% g, a fact of probability one half, is the AND of a and b: c(1) to c(3)
% say so (src/ground/condition.rs, gates), and c(4), which wants g or c,
% reads it, as does the query: g stays a fact with its clauses. P(e) = 0.5
% * P(a b or c) = 0.5 * (1 - 0.88 * 0.7) = 0.192; P(g | e) = 0.12 / 0.384
% = 0.3125, P(c | e) = 0.3 / 0.384 = 0.78125.
0.2::a.
0.6::b.
0.3::c.
0.5::g.
k(1) :- g, \+a.
k(2) :- g, \+b.
k(3) :- \+g, a, b.
k(4) :- \+g, \+c.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
query(g).
query(c).
