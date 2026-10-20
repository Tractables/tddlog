% gen_roulette.py motivating 3
% Roulette motivating.rkt: x0 ~ flip 0.1, x_k = if x_{k-1} then flip 0.2 else flip 0.3.
0.2::a(3).
0.3::b(3).
0.2::a(2).
0.3::b(2).
0.2::a(1).
0.3::b(1).
0.1::x(0).
x(1) :- x(0), a(1).
x(1) :- \+x(0), b(1).
x(2) :- x(1), a(2).
x(2) :- \+x(1), b(2).
x(3) :- x(2), a(3).
x(3) :- \+x(2), b(3).
query(x(3)).
