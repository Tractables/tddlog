% Rows of a table with equal parameters. The rows of b, of c and of d
% exclude each other through the values of their parents, and the rows of
% y through x and \+x; the rows of z read two independent facts. d's first
% row has four equal parameters.
% Run with `tddlog examples/shared_rows.pl`; `tddlog --ground examples/shared_rows.pl`
% prints the program the grounder writes.
0.2::a(1); 0.3::a(2); 0.5::a(3).
0.1::b(1); 0.2::b(2); 0.7::b(3) :- a(1).
0.2::b(1); 0.1::b(2); 0.7::b(3) :- a(2).
0.7::b(1); 0.2::b(2); 0.1::b(3) :- a(3).
0.4::c(t); 0.6::c(f) :- b(1).
0.4::c(t); 0.6::c(f) :- b(2).
0.9::c(t); 0.1::c(f) :- b(3).
0.25::d(1); 0.25::d(2); 0.25::d(3); 0.25::d(4) :- a(1), c(t).
0.25::d(1); 0.25::d(2); 0.5::d(3) :- a(1), c(f).
0.1::d(1); 0.25::d(2); 0.25::d(3); 0.4::d(4) :- \+a(1).
0.5::x.
0.3::y :- x.
0.3::y :- \+x.
0.6::f.
0.6::g.
0.3::z :- f.
0.3::z :- g.
evidence(c(t)).
evidence(d(2), false).
query(a(1)). query(a(2)). query(a(3)).
query(b(1)). query(b(2)). query(b(3)).
query(d(1)). query(d(3)). query(d(4)).
query(y). query(z). query(x).
