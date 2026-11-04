% The diamond network of the Dice paper's network verification benchmark
% (bench/dicebench.ml, gen_diamond), ported to ProbLog: $1 diamonds in a
% row, the program's argument. Diamond T + 1 (step T) routes the signal one
% of two ways (route, flip 0.5); the other way loses it with probability
% 0.0001 (drop). s(T) is the signal after T diamonds, true at the start;
% the query is whether it arrives, s($1) as Roulette's diamond.rkt prints it.
diamonds($1).
step(0) :- diamonds(N), N > 0.
step(T1) :- step(T), diamonds(N), T1 is T + 1, T1 < N.
0.5::route(T) :- step(T).
0.0001::drop(T) :- step(T).
s(0).
s(T1) :- step(T), T1 is T + 1, s(T), route(T).
s(T1) :- step(T), T1 is T + 1, s(T), \+route(T), \+drop(T).
query(s($1)).
