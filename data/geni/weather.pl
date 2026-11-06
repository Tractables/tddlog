% The weather factory of Geni's evaluation (Rubicon's translation of
% weatherfactoryN.prism, Holtzen et al., CAV 2021; the artifact's
% bench/ICFP/weather-factory/rubicon-dice/weather-factory-N-H.dice),
% ported to ProbLog: factories, each on strike or not, and the weather, for
% $2 days. The weather starts sunny and is sunny the next day with
% probability 0.7 after a sunny day and 0.6 after a rainy one. Factory I
% starts off strike and is on strike the next day with a probability that
% depends on its strike and the weather today (strikes(I, T, S, W), S on
% or off, W sun or rain). The factories and these probabilities are
% consulted from $1. The questions are Rubicon's program's value, the pair
% of whether every factory is on strike on some day T = 0 .. $2 (reached)
% and false (second).
:- consult($1).
days($2).
step(0) :- days(H), H > 0.
step(T1) :- step(T), days(H), T1 is T + 1, T1 < H.
time(0).
time(T1) :- step(T), T1 is T + 1.
sun(0).
0.7::sunny(T, sun) :- step(T).
0.6::sunny(T, rain) :- step(T).
sky(T, sun) :- sun(T).
sky(T, rain) :- time(T), \+sun(T).
sun(T1) :- step(T), T1 is T + 1, sky(T, W), sunny(T, W).
state(I, T, on) :- strike(I, T).
state(I, T, off) :- factory(I), time(T), \+strike(I, T).
strike(I, T1) :- step(T), T1 is T + 1, state(I, T, S), sky(T, W), strikes(I, T, S, W).
some_off(T) :- state(_, T, off).
reached :- time(T), \+some_off(T).
0.0::second.
query(reached).
query(second).
