% Evidence on clauses of two literals, as in Ace's friends and smokers
% networks: c(1) and c(2) make f(1) and f(2) equal, c(3) and c(4) make f(3)
% the negation of f(1). Of the eight worlds of the f, two remain:
% f(1) = f(2) = true, f(3) = false weighs 0.3 * 0.6 * 0.2 = 0.036, and
% f(1) = f(2) = false, f(3) = true weighs 0.7 * 0.4 * 0.8 = 0.224.
0.3::f(1,true); 0.7::f(1,false).
0.6::f(2,true); 0.4::f(2,false).
0.8::f(3,true); 0.2::f(3,false).
0.5::g.
c(1) :- f(1,true), f(2,false).
c(2) :- f(2,true), f(1,false).
c(3) :- f(3,true), f(1,true).
c(4) :- f(3,false), f(1,false).
q :- f(1,true), g.
evidence(c(1),false).
evidence(c(2),false).
evidence(c(3),false).
evidence(c(4),false).
query(f(1,true)).
query(f(2,true)).
query(f(3,true)).
query(f(3,false)).
query(q).
