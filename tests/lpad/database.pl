parent(ann, bob).
parent(bob, carl).
0.5::sampled(X) :- person(X).
0.4::trait(ann).
0.7::trait(Y) :- parent(X, Y), trait(X).
0.9::big_family :- findall(C, child(C), Cs), length(Cs, 2).
person(X) :- parent(X, _).
person(X) :- parent(_, X).
child(Y) :- parent(_, Y).
query(trait(carl)).
query(sampled(carl)).
query(big_family).
