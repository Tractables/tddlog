% The router failure grid of Geni's evaluation (its artifact's
% bench/ICFP/grid-networks/router-failure.dice.py), ported to ProbLog: a
% $1 x $1 grid of routers, (R, C) for R, C = 1 .. $1, each working (x)
% with a probability that depends on whether its left and upper neighbours
% work: 0.5 for the router (1, 1); on the first row and column, which have
% one neighbour, 0.5000001 after a working one and 0.5000002 after a
% failed one (ok(R, C, t) and ok(R, C, f)); inside, 0.5000003 to 0.5000006
% for left and upper working, left alone, upper alone and neither (ok(R,
% C, tt), tf, ft and ff). The query is whether the router ($1, $1) works.
size($1).
coord(1).
coord(I1) :- coord(I), size(N), I1 is I + 1, I1 =< N.
% the router before one on the first row or column
before(1, C, 1, D) :- coord(C), C > 1, D is C - 1.
before(R, 1, Q, 1) :- coord(R), R > 1, Q is R - 1.
inner(R, C) :- coord(R), R > 1, coord(C), C > 1.
0.5::x(1, 1).
0.5000001::ok(R, C, t) :- before(R, C, _, _).
0.5000002::ok(R, C, f) :- before(R, C, _, _).
0.5000003::ok(R, C, tt) :- inner(R, C).
0.5000004::ok(R, C, tf) :- inner(R, C).
0.5000005::ok(R, C, ft) :- inner(R, C).
0.5000006::ok(R, C, ff) :- inner(R, C).
x(R, C) :- before(R, C, Q, D), x(Q, D), ok(R, C, t).
x(R, C) :- before(R, C, Q, D), \+x(Q, D), ok(R, C, f).
x(R, C) :- inner(R, C), D is C - 1, Q is R - 1, x(R, D), x(Q, C), ok(R, C, tt).
x(R, C) :- inner(R, C), D is C - 1, Q is R - 1, x(R, D), \+x(Q, C), ok(R, C, tf).
x(R, C) :- inner(R, C), D is C - 1, Q is R - 1, \+x(R, D), x(Q, C), ok(R, C, ft).
x(R, C) :- inner(R, C), D is C - 1, Q is R - 1, \+x(R, D), \+x(Q, C), ok(R, C, ff).
query(x($1,$1)).
