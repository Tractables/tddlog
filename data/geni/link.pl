% The link failure grid of Geni's evaluation (its artifact's
% bench/ICFP/grid-networks/link-failure.dice.py, mode 3-flow, p 0.5),
% ported to ProbLog: a $1 x $1 grid of routers, (X, Y) for X, Y = 1 .. $1.
% A packet leaves the router (1, 1) and crosses each link with probability
% 0.5: a router is reached (conn) when its left, upper or upper-left
% neighbour is reached and the link from it works. The query is whether
% the router ($1, $1) is reached. Fierens et al.'s grid (TPLP 2015).
size($1).
coord(1).
coord(I1) :- coord(I), size(N), I1 is I + 1, I1 =< N.
cell(X, Y) :- coord(X), coord(Y).
0.5::left(X, Y) :- cell(X, Y), Y > 1.
0.5::up(X, Y) :- cell(X, Y), X > 1.
0.5::diagonal(X, Y) :- cell(X, Y), X > 1, Y > 1.
conn(1, 1).
conn(X, Y) :- left(X, Y), Y0 is Y - 1, conn(X, Y0).
conn(X, Y) :- up(X, Y), X0 is X - 1, conn(X0, Y).
conn(X, Y) :- diagonal(X, Y), X0 is X - 1, Y0 is Y - 1, conn(X0, Y0).
query(conn($1,$1)).
