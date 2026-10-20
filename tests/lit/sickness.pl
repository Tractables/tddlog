% sickness: 4 people, 4 steps, power-law friendships, evidence 0.33, seed 1
0.2::cold(T) :- time(T).
0.4::stay_sick(X,T) :- person(X), time(T).
sick(X,T) :- person(X), cold(T).
sick(X,T) :- friend(X,Y), sick(Y,T).
sick(X,T) :- next(T0,T), sick(X,T0), stay_sick(X,T).
person(p0).
person(p1).
person(p2).
person(p3).
friend(p0,p2).
friend(p1,p0).
friend(p1,p2).
friend(p1,p3).
friend(p2,p3).
time(0).
time(1).
time(2).
time(3).
next(0,1).
next(1,2).
next(2,3).
evidence(sick(p0,0),false).
evidence(sick(p1,0),false).
evidence(sick(p2,0),false).
evidence(sick(p0,1),false).
evidence(sick(p1,1),false).
evidence(sick(p1,2),true).
query(sick(p0,3)).
query(sick(p1,3)).
query(sick(p2,3)).
query(sick(p3,3)).
