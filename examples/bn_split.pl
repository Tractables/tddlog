% A table large enough for the indicator rewrite to split on a parent:
% c has nine rows over a and b, three values each; d reads c and is observed.
0.2::a(x); 0.5::a(y); 0.3::a(z).
0.6::b(x); 0.1::b(y); 0.3::b(z).
0.1::c(p); 0.3::c(q); 0.6::c(r) :- a(x), b(x).
0.7::c(p); 0.2::c(q); 0.1::c(r) :- a(x), b(y).
0.3::c(p); 0.3::c(q); 0.4::c(r) :- a(x), b(z).
0.5::c(p); 0.4::c(q); 0.1::c(r) :- a(y), b(x).
0.2::c(p); 0.2::c(q); 0.6::c(r) :- a(y), b(y).
0.9::c(p); 0.05::c(q); 0.05::c(r) :- a(y), b(z).
0.25::c(p); 0.5::c(q); 0.25::c(r) :- a(z), b(x).
0.4::c(p); 0.1::c(q); 0.5::c(r) :- a(z), b(y).
0.15::c(p); 0.15::c(q); 0.7::c(r) :- a(z), b(z).
0.8::d(t); 0.2::d(f) :- c(p).
0.3::d(t); 0.7::d(f) :- c(q).
0.1::d(t); 0.9::d(f) :- c(r).
0.6::e(t); 0.4::e(f) :- b(x), c(p).
0.2::e(t); 0.8::e(f) :- b(x), c(q).
0.5::e(t); 0.5::e(f) :- b(x), c(r).
0.9::e(t); 0.1::e(f) :- b(y), c(p).
0.4::e(t); 0.6::e(f) :- b(y), c(q).
0.3::e(t); 0.7::e(f) :- b(y), c(r).
0.1::e(t); 0.9::e(f) :- b(z), c(p).
0.7::e(t); 0.3::e(f) :- b(z), c(q).
0.5::e(t); 0.5::e(f) :- b(z), c(r).
evidence(d(t)).
query(a(x)).
query(a(y)).
query(b(z)).
query(c(p)).
query(c(q)).
query(e(t)).
