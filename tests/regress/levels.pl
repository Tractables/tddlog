%Expected outcome:
% on(4) 0.5749
% any(4) 1.0
% dead(4) 0.0
% r(c,3) 0.14575
% q 0.5749
%
% A loop over a time index written as recursion: on, any, dead and r are
% one recursive stratum whose last argument is a level that no rule
% lowers, and the grounder grounds it level by level, each level's atoms
% complete before a later level reads them. any(T) holds by structure
% (on(T) or not) and dead(T) never (flip(0) and not): the and-inverter
% graph proves them constant as their levels complete, and the rules
% grounded after read them so, dead(2) to dead(5) never derived. r(_, T)
% is reachability over the gates of level T, a cyclic group within its
% level, whose atoms stay inputs of the graph. A negated atom of the
% level, \+on(T), is read in any(T) once on(T) is complete.

step(0). step(1). step(2). step(3). step(4).
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
