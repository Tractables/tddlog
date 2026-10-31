%Expected outcome:
% total(x,2200) 0.2
% total(x,1000) 0.2
% total(x,1200) 0.3
% total(y,800) 0.3
% best(x,1200) 0.5
% best(x,1000) 0.2
% best(y,800) 0.3
% mean(1000.0) 0.29
% mean(1200.0) 0.21
% mean(800.0) 0.09
% mean(1100.0) 0.14
% mean(900.0) 0.06
% Aggregates over probabilistic bodies: each world aggregates the solutions
% it has, as library(aggregate)'s all/3 collects them. By hand: mean(1000.0)
% holds in the worlds {a}, {b, c} and {a, b, c}, 0.4 * 0.5 * 0.7 + 0.6 * 0.5
% * 0.3 + 0.4 * 0.5 * 0.3; no world without a worker has a mean.
:- use_module(library(aggregate)).

0.4::works(a).
0.5::works(b).
0.3::works(c).
salary(a, 1000).
salary(b, 1200).
salary(c, 800).
dept(a, x).
dept(b, x).
dept(c, y).

total(D, sum<S>) :- works(P), salary(P, S), dept(P, D).
best(D, max<S>) :- works(P), salary(P, S), dept(P, D).
mean(avg<S>) :- works(P), salary(P, S).

query(total(D, S)).
query(best(D, S)).
query(mean(A)).
