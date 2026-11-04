% The ladder network of the Dice paper's network verification benchmark
% (bench/dicebench.ml, gen_ladder), ported to ProbLog: $1 rungs over two
% wires, the program's argument. From (true, false), rung T + 1 (step T)
% gives (false, false) where the first wire is up and the rung breaks
% (flip 0.001), else (b, not b), b a flip 0.5, where either wire is up, and
% (false, false) where neither is. o1(T) and o2(T) are the wires after T
% rungs; the queries are the wires at the end, o1($1) and o2($1) as
% Roulette's ladder.rkt prints them.
rungs($1).
step(0) :- rungs(N), N > 0.
step(T1) :- step(T), rungs(N), T1 is T + 1, T1 < N.
0.001::breaks(T) :- step(T).
0.5::b(T) :- step(T).
o1(0).
o1(T1) :- step(T), T1 is T + 1, o1(T), \+breaks(T), b(T).
o1(T1) :- step(T), T1 is T + 1, \+o1(T), o2(T), b(T).
o2(T1) :- step(T), T1 is T + 1, o1(T), \+breaks(T), \+b(T).
o2(T1) :- step(T), T1 is T + 1, \+o1(T), o2(T), \+b(T).
query(o1($1)).
query(o2($1)).
