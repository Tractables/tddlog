% Two definitions, each through the other: g1 is the OR of g2 and a, g2 the
% AND of g1 and b, both facts of probability one half; c(1) is a clause of
% both. ProbLog's least model reads `g1 :- g2. g2 :- g1, b.` as g1 = g2 =
% false where a is false and b true, where the clauses admit both values:
% of the two definitions one is found and the other refused, whose clauses
% stay and read the first, so both stay facts (src/ground/condition.rs,
% gates). Worlds of g1 and g2 per a and b: a, one (g1, g2 = b); not a, b,
% two; neither, one. P(e) = 0.25 * (0.3 + 0.7 * 0.6 * 2 + 0.7 * 0.4) =
% 0.355; P(g1, e) = 0.25 * (0.3 + 0.42) = 0.18, P(g2, e) = 0.25 * (0.18 +
% 0.42) = 0.15.
0.3::a.
0.6::b.
0.5::g1.
0.5::g2.
c(1) :- \+g1, g2.
c(2) :- \+g1, a.
c(3) :- g1, \+g2, \+a.
c(4) :- g2, \+b.
c(5) :- \+g2, g1, b.
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
evidence(c(5), false).
query(g1).
query(g2).
