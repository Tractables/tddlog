% Two gates, one an input of the other: h is the AND of g and c, g the OR of
% a and b, both facts of probability one half (src/ground/condition.rs,
% gates). k(7) wants h or d, so both stay facts with their clauses (and
% the queries read them). P(e) = 0.25 * P((a or b) c or d): P(a or b) =
% 1 - 0.8 * 0.4 = 0.68, with c 0.204, or d 1 - 0.796 * 0.9 = 0.2836, P(e) =
% 0.0709. P(g, e) = 0.25 * 0.68 * (1 - 0.7 * 0.9) = 0.0629, P(g | e) =
% 0.2516 / 0.2836; P(h | e) = 0.204 / 0.2836.
0.5::h.
0.2::a.
0.6::b.
0.3::c.
0.1::d.
0.5::g.
k(1) :- h, \+g.
k(2) :- h, \+c.
k(3) :- \+h, g, c.
k(4) :- a, \+g.
k(5) :- b, \+g.
k(6) :- g, \+a, \+b.
k(7) :- \+h, \+d.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
evidence(k(5), false).
evidence(k(6), false).
evidence(k(7), false).
query(g).
query(h).
