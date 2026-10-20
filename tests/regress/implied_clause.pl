% A weighted CNF with a clause of four literals the others imply: k(1)
% says a or b, k(2) not b or c, and their resolvent a or c implies k(3),
% a or c or d or e. src/ground/preprocess/implied.rs drops k(3) before the search
% for a support; nothing else reads e, which is then dropped too. No fact
% is a function of the others. P(e) = P(b) P(c) + P(\+b) P(a) = 0.6 * 0.5
% + 0.4 * 0.3 = 0.42; P(c | e) = (0.3 + 0.4 * 0.3 * 0.5) / 0.42 = 0.36 /
% 0.42, P(d | e) = 0.8 as d is read by k(3) alone.
0.3::a.
0.6::b.
0.5::c.
0.8::d.
0.5::e.
k(1) :- \+a, \+b.
k(2) :- b, \+c.
k(3) :- \+a, \+c, \+d, \+e.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
query(c).
query(d).
