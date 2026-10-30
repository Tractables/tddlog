% Two-terminal network reliability: 8 by 8 routers in a grid, each
% joined to its neighbours by a link that works with probability 0.5,
% independently. A message travels both ways along a working link. What is
% the probability that the router in one corner reaches the opposite one?
% Run with `tddlog examples/larger/grid_reliability.pl`.
size(8).
router(X, Y) :- size(N), between(1, N, X), between(1, N, Y).
0.5::link(X, Y, X1, Y) :- router(X, Y), size(N), X < N, X1 is X + 1.
0.5::link(X, Y, X, Y1) :- router(X, Y), size(N), Y < N, Y1 is Y + 1.
hop(X, Y, U, V) :- link(X, Y, U, V).
hop(X, Y, U, V) :- link(U, V, X, Y).
reaches(1, 1).
reaches(U, V) :- reaches(X, Y), hop(X, Y, U, V).
query(reaches(N, N)) :- size(N).
query(reaches(N, 1)) :- size(N).
