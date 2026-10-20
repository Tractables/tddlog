%Expected outcome:
% p(1,4) 0.3504
% p(2,4) 0.584
% p(3,4) 0.72592
% p(4,4) 0.1752
% p(5,4) 0.45
%
% The chain rule with its body the other way round, queried towards a
% target: written left-linear and grounded from the bound end.
% The expected outcomes are ProbLog 2.3.0's answers on the same closure
% written right-linear, which a count over every world confirms.

0.6::e(1,2). 0.5::e(2,3). 0.4::e(3,1). 0.7::e(3,4). 0.3::e(4,2). 0.8::e(2,5). 0.45::e(5,4).
p(X,Y) :- e(X,Y).
p(X,Y) :- p(Z,Y), p(X,Z).
query(p(X,4)).
