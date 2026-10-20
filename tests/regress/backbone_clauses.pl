% A weighted CNF whose clauses fix a fact that unit propagation does not:
% k(1) says a or b, k(2) a implies c and k(3) b implies c, so c holds in
% every world the evidence allows (a backbone). Once c is fixed, k(4) and
% k(5) say d implies e and e implies d, and d and e are one variable.
% P(e) = P(a or b) P(c) P(d = e) = 0.68 * 0.3 * (0.7 * 0.4 + 0.3 * 0.6)
% = 0.09384; P(a | e) = 0.2 / 0.68, P(c | e) = 1, P(d | e) = 0.28 / 0.46.
0.2::a.
0.6::b.
0.3::c.
0.7::d.
0.4::e.
k(1) :- \+a, \+b.
k(2) :- a, \+c.
k(3) :- b, \+c.
k(4) :- c, d, \+e.
k(5) :- c, e, \+d.
evidence(k(1), false).
evidence(k(2), false).
evidence(k(3), false).
evidence(k(4), false).
evidence(k(5), false).
query(a).
query(c).
query(d).
