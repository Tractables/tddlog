%Expected outcome:
% holds(1) 0.5
% lit(1) 0.3
% holds(2) 0.395
% holds(3) 0.42775
% holds(4) 0.42775
%
% The event calculus's law of inertia over a successor relation given by
% its facts, next/2, as Prob-EC's event definitions are written: a
% fluent holds from the frame after an event starts it until one stops
% it. Every clause of next rises by one from its first argument to its
% second, so holds and lit are one recursive stratum whose last argument
% is a level that no rule lowers, grounded level by level as a loop
% written with `T1 is T + 1` is (levels.pl); lit(T) reads holds(T) at its
% own level. By hand: holds(1) = start(0); lit(1) = holds(1) bright(1);
% holds(2) = holds(1) (not stop(1) or bright(1) kick(1)) = 0.5 (1 - 0.3
% 0.7); holds(3) = start(2) or holds(2) not stop(2) = 1 - 0.75 (1 - 0.395
% 0.6); holds(4) = holds(3), nothing stops it at 3.

next(0,1). next(1,2). next(2,3). next(3,4).
0.5::start(0). 0.25::start(2).
0.3::stop(1). 0.4::stop(2).
0.6::bright(1). 0.5::kick(1).
holds(T1) :- next(T,T1), start(T).
holds(T1) :- next(T,T1), holds(T), \+stop(T).
holds(T1) :- next(T,T1), lit(T), kick(T).
lit(T) :- holds(T), bright(T).
query(holds(1)).
query(lit(1)).
query(holds(2)).
query(holds(3)).
query(holds(4)).
