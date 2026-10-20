% tuple-independent database: domain 4, 2 S-tuples per constant, seed 1
0.17::r(c0).
0.81::t(c0).
0.74::r(c1).
0.28::t(c1).
0.5::r(c2).
0.45::t(c2).
0.64::r(c3).
0.76::t(c3).
0.0378::s(c0,c0).
0.4011::s(c0,c1).
0.368::s(c1,c2).
0.0259::s(c1,c3).
0.3497::s(c2,c1).
0.1279::s(c2,c3).
0.0388::s(c3,c0).
0.0365::s(c3,c1).
h0 :- r(X), s(X,Y), t(Y).
h1 :- r(X), s(X,Y).
h1 :- s(X,Y), t(Y).
safe :- r(X), s(X,Y).
query(h0).
query(h1).
query(safe).
