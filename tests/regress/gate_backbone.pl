% Clauses observed true, as a weighted CNF's program has them. g, a fact of
% probability one half, is the OR of x(1) and x(2): c(1) to c(3) say so,
% and nothing else reads g. c(4) and c(5) want x(1) or y and x(2) or not
% y, which imply x(1) or x(2), so g holds in every world the evidence
% allows: a backbone that unit propagation does not find. The SAT search
% leaves g alone, and the gate goes with its definition, as it does
% without the search (src/ground/condition.rs, on_evidence).
% P(e) = 0.5 * (0.6 * 0.4 + 0.4 * 0.3) = 0.18; P(x(1), e) = 0.5 * 0.3 *
% (0.6 * 0.4 + 0.4) = 0.096, P(y, e) = 0.5 * 0.6 * 0.4 = 0.12;
% P(x(1) | e) = 0.096 / 0.18, P(y | e) = 0.12 / 0.18.
0.3::x(1).
0.4::x(2).
0.6::y.
0.5::g.
c(1) :- x(1), \+g.
c(2) :- x(2), \+g.
c(3) :- g, \+x(1), \+x(2).
c(4) :- \+x(1), \+y.
c(5) :- \+x(2), y.
evidence(c(1), false).
evidence(c(2), false).
evidence(c(3), false).
evidence(c(4), false).
evidence(c(5), false).
query(x(1)).
query(y).
