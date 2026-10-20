% Rows of a table with one distribution, for row classes
% (src/ground/classes.rs). c's six rows over a and b hold two distributions
% and exclude each other through the values of their parents: c reads a
% class atom per distribution. e's rows are deterministic, a class per
% value, and stay as they are. w's four rows hold one distribution but read
% independent facts, which can hold together: no two of them are a class.
% d's five rows fall into two classes but read one parent each, h, which a
% split on h reads a row at a time: d stays as it is.
0.2::a(1); 0.3::a(2); 0.5::a(3).
0.6::b(t); 0.4::b(f).
0.1::c(x); 0.3::c(y); 0.6::c(z) :- a(1), b(t).
0.1::c(x); 0.3::c(y); 0.6::c(z) :- a(2), b(f).
0.1::c(x); 0.3::c(y); 0.6::c(z) :- a(3), b(t).
0.5::c(x); 0.5::c(y) :- a(1), b(f).
0.5::c(x); 0.5::c(y) :- a(2), b(t).
0.5::c(x); 0.5::c(y) :- a(3), b(f).
e(on) :- c(x), b(t).
e(on) :- c(y), b(t).
e(off) :- c(z), b(t).
e(on) :- c(x), b(f).
e(off) :- c(y), b(f).
e(on) :- c(z), b(f).
0.6::f(1). 0.6::f(2). 0.6::f(3). 0.6::f(4).
0.3::w :- f(1), b(t).
0.3::w :- f(2), b(t).
0.3::w :- f(3), b(t).
0.3::w :- f(4), b(t).
0.1::h(1); 0.2::h(2); 0.3::h(3); 0.15::h(4); 0.25::h(5).
0.7::d :- h(1).
0.7::d :- h(2).
0.7::d :- h(3).
0.2::d :- h(4).
0.2::d :- h(5).
evidence(e(on)).
query(a(1)). query(a(2)). query(a(3)).
query(b(t)).
query(c(x)). query(c(y)). query(c(z)).
query(w).
query(d).
