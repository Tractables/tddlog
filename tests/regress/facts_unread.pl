%Expected outcome:
% q 0.7
% t 0.526
% r(1) 0.2
% r(5) 0
% s(1,b) 1
% s(3,a) 1
%
% The expected outcomes are computed by hand. r and s are given by facts
% alone, and the ground program has the atoms of the facts that a rule, a
% query or the evidence names. q reads r(3) and r(4), and the comparison
% passes over r(1) and r(2): q is 1 - 0.6 * 0.5. t reads s(2,a) with r(2)
% and the certain s(3,a) with r(3): 1 - (1 - 0.7 * 0.3) * (1 - 0.4). No
% rule reads r(1), which the query names, nor s(1,b), which the evidence
% names; r(5) is no fact. The evidence on s(1,b) bears on nothing else.
0.2::r(1). 0.3::r(2). 0.4::r(3). 0.5::r(4).
0.6::s(1,b). 0.7::s(2,a). s(3,a).
q :- r(X), X >= 3.
t :- s(X,a), r(X).
evidence(s(1,b)).
query(q). query(t). query(r(1)). query(r(5)). query(s(1,b)). query(s(3,a)).
