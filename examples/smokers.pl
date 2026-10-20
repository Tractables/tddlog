% Social network example (Fierens et al. 2015).
0.3::stress(X) :- person(X).
0.2::influences(X,Y) :- person(X), person(Y).
person(p1). person(p2). person(p3).
friend(p1,p2). friend(p2,p1). friend(p2,p3). friend(p3,p2).
smokes(X) :- stress(X).
smokes(X) :- friend(X,Y), influences(Y,X), smokes(Y).
0.4::asthma(X) :- smokes(X).
query(smokes(p1)).
query(smokes(p2)).
query(asthma(p3)).
