% Evidence on a fact that would otherwise be aliased away. Found by the fuzzer.
person(p1).
person(p2).
0.5::stress(X) :- person(X).
smokes(X) :- stress(X).
0.8::asthma(X) :- smokes(X).
query(asthma(p1)).
query(smokes(p2)).
evidence(stress(p1),false).
evidence(stress(p2),true).
