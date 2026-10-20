% Inconsistent evidence on an atom the aliasing pass must keep.
person(p1).
0.5::stress(X) :- person(X).
smokes(X) :- stress(X).
0.8::asthma(X) :- smokes(X).
query(asthma(p1)).
evidence(stress(p1),false).
evidence(asthma(p1),true).
