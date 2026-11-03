%Expected outcome:
% on(4) 0.5749
% any(4) 1.0
% dead(4) 0.0
% r(c,3) 0.14575
% q 0.5749
%
% levels.pl with its steps counted from a fact, as the Dice reader writes
% a loop: steps(5) is true in every world, and so is each step(T) derived
% from it. The and-inverter graph that proves a level's atoms constant
% reads such an atom as true, not as an input of its own, so that any(T),
% on(T) or not, is proved true here as it is where the steps are facts,
% and the program grounds as levels.pl does. The answers are levels.pl's
% (against ProbLog).

steps(5).
step(0) :- steps(N), N > 0.
step(T1) :- step(T), steps(N), T1 is T + 1, T1 < N.
0.6::flip(T) :- step(T).
link(a,b). link(b,a). link(b,c).
0.5::gate(X,Y,T) :- link(X,Y), step(T).
on(0).
on(T1) :- step(T), T1 is T + 1, on(T), flip(T), \+dead(T).
on(T1) :- step(T), T1 is T + 1, \+on(T), any(T), \+flip(T).
on(T1) :- step(T), T1 is T + 1, r(c,T), \+flip(T).
any(T) :- on(T).
any(T) :- step(T), \+on(T).
dead(T1) :- step(T), T1 is T + 1, flip(T), \+flip(T).
dead(T1) :- step(T), T1 is T + 1, dead(T), any(T).
r(a,T) :- on(T).
r(Y,T) :- r(X,T), link(X,Y), gate(X,Y,T).
q :- on(4), any(4), \+dead(4).
query(on(4)).
query(any(4)).
query(dead(4)).
query(r(c,3)).
query(q).
