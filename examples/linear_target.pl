% A closure written left-recursively and queried towards a bound target:
% grounded from the target (src/ground/linear.rs), it walks the edges
% backwards from 6 rather than grounding p(X,Z) for every Z.
% Run with `tddlog examples/linear_target.pl`; `tddlog --ground examples/linear_target.pl`
% prints the program the grounder writes.
0.5::e(1,2). 0.6::e(2,3). 0.7::e(3,1). 0.4::e(3,4). 0.5::e(4,5).
0.3::e(5,3). 0.6::e(5,6). 0.5::e(2,6). 0.4::e(6,4).
0.9::p(X,Y) :- e(X,Y).
p(X,Y) :- p(X,Z), e(Z,Y).
query(p(X,6)).
