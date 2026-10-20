%Expected outcome:
% p(1,1) 0.12
% p(1,2) 0.6
% p(1,3) 0.3
% p(1,4) 0.3504
% p(1,5) 0.48
%
% A transitive closure written with the chain rule, queried from a source:
% written right-linear and grounded from the bound end.
% The expected outcomes are ProbLog 2.3.0's answers on the same closure
% written right-linear, which a count over every world confirms.

0.6::e(1,2). 0.5::e(2,3). 0.4::e(3,1). 0.7::e(3,4). 0.3::e(4,2). 0.8::e(2,5). 0.45::e(5,4).
p(X,Y) :- e(X,Y).
p(X,Y) :- p(X,Z), p(Z,Y).
query(p(1,Y)).
