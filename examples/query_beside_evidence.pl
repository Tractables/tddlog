% A query's formula next to evidence it does not read: no rule above w's
% facts reads them but w's own, and d reads neither w nor the evidence.
0.2::a(1); 0.3::a(2); 0.5::a(3).
0.6::b(t); 0.4::b(f).
0.1::c(x); 0.3::c(y); 0.6::c(z) :- a(3), b(t).
0.5::c(x); 0.5::c(y) :- a(2), b(t).
0.5::c(x); 0.5::c(y) :- a(3), b(f).
e(on) :- c(x), b(f).
e(on) :- c(z), b(f).
0.6::f(1). 0.6::f(2). 0.6::f(3). 0.6::f(4).
0.3::w :- f(1), b(t).
0.3::w :- f(2), b(t).
0.3::w :- f(3), b(t).
0.3::w :- f(4), b(t).
0.1::h(1); 0.2::h(2); 0.3::h(3); 0.15::h(4); 0.25::h(5).
0.7::d :- h(3).
0.2::d :- h(4).
0.2::d :- h(5).
evidence(e(on)).
query(w).
query(d).
