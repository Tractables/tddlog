%Expected outcome:
% p(3,5) 0.57035603
% p(5,1) 0
% q 0.79106678
% r(1) 0.24457936
% r(2) 0.35354727
% r(3) 0.35354727
% r(4) 0.1925412
%
% The chain rule over three exits (a rule, a probabilistic rule and a fact),
% with evidence on the closure, a negated use and a cycle query.
% The expected outcomes are ProbLog 2.3.0's answers on the same closure
% written right-linear, which a count over every world confirms.

0.6::e(1,2). 0.5::e(2,3). 0.4::e(3,1). 0.7::e(3,4). 0.3::e(4,2).
0.35::f(4,1). 0.25::f(5,3).
p(X,Y) :- e(X,Y).
0.7::p(X,Y) :- f(Y,X).
p(2,5).
p(X,Y) :- p(X,Z), p(Z,Y).
q :- \+ p(4,3).
r(X) :- p(X,X).
evidence(p(1,4), true).
query(p(3,5)).
query(p(5,1)).
query(q).
query(r(X)).
