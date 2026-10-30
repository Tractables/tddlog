% A robot moves around a ring of 12 cells: at each step it goes one cell
% on with probability 0.8, stays with 0.1 or slips one back with 0.1. A
% sensor tells whether it is at a door (cells 1, 4 and 8) and is wrong one
% time in ten. It starts at cell 1. Given the 400 readings below (1 for a
% door), where is it at the end?
% Run with `tddlog examples/larger/robot.pl`. `--evidence-probability` adds P(e).
:- use_module(library(lists)).
cells(12).
door(1). door(4). door(8).
readings([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
          1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0,
          0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0,
          1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0,
          1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0,
          1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0,
          0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0,
          1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
          0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
          1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
          1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
          1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0,
          0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0,
          1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0,
          1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1,
          1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0,
          0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
          0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1,
          1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0,
          0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0]).
time(T) :- readings(L), length(L, N), between(1, N, T).
at(1, 1).
0.1::at(T, B); 0.1::at(T, C); 0.8::at(T, F) :-
    time(T), T > 1, S is T - 1, at(S, C), cells(N), F is C mod N + 1, B is (C + N - 2) mod N + 1.
0.9::door_seen(T) :- at(T, C), door(C).
0.1::door_seen(T) :- at(T, C), \+ door(C).
% agrees(T): readings 1 to T are what the sensor said.
agrees(0).
agrees(T) :- time(T), S is T - 1, agrees(S), readings(L), nth1(T, L, 1), door_seen(T).
agrees(T) :- time(T), S is T - 1, agrees(S), readings(L), nth1(T, L, 0), \+ door_seen(T).
all_agree :- readings(L), length(L, N), agrees(N).
evidence(all_agree).
query(at(T, C)) :- readings(L), length(L, T), cells(N), between(1, N, C).
