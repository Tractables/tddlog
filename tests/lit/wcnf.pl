% log10 Z - log10 P(e) = 0.2730012720637377
% wcnf.cnf: weighted model counting, 5 variables (4 in clauses), 4 clauses
0.3::x(1).
0.6::x(2).
0.2::x(3).
0.5::x(4).
violated(1) :- \+x(1), \+x(2).
evidence(violated(1), false).
violated(2) :- x(1), \+x(3).
evidence(violated(2), false).
violated(3) :- x(2), x(3), \+x(4).
evidence(violated(3), false).
violated(4) :- \+x(2), x(4).
evidence(violated(4), false).
