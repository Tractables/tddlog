% gen_roulette.py diamond 3
% Roulette diamond.rkt: s_k = (route_k and s_{k-1}) or (not route_k and s_{k-1} and not drop_k).
0.0001::drop(3).
0.5::route(3).
0.0001::drop(2).
0.5::route(2).
0.0001::drop(1).
0.5::route(1).
s(0).
s(1) :- s(0), route(1).
s(1) :- s(0), \+route(1), \+drop(1).
s(2) :- s(1), route(2).
s(2) :- s(1), \+route(2), \+drop(2).
s(3) :- s(2), route(3).
s(3) :- s(2), \+route(3), \+drop(3).
query(s(3)).
