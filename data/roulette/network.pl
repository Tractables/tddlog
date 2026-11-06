% The network of Roulette's scaling/network.rkt, ported to ProbLog: the
% 4-bit message 1111 sent through $1 diamonds. In diamond T + 1 (step T) a
% flip 0.5 (sp) sends it to a noisy node, which with probability 0.01 (nz)
% flips one of its bits chosen uniformly (sel), or else to a node that
% forwards it with probability 0.99 (fw) and loses it otherwise. b(T, B)
% is bit B of the message after T diamonds and lost(T) whether it is lost;
% the queries are the distribution of the message, m($1, V) for V = 0 .. 15
% (bit B of the message bit B of V) and m($1, 16) for a lost message.
diamonds($1).
step(0) :- diamonds(N), N > 0.
step(T1) :- step(T), diamonds(N), T1 is T + 1, T1 < N.
weight(0, 1).
weight(1, 2).
weight(2, 4).
weight(3, 8).
0.25::sel(T, 0); 0.25::sel(T, 1); 0.25::sel(T, 2); 0.25::sel(T, 3) :- step(T).
0.01::nz(T) :- step(T).
0.99::fw(T) :- step(T).
0.5::sp(T) :- step(T).
flips(T, B) :- sp(T), nz(T), sel(T, B).
b(0, B) :- weight(B, _).
b(T1, B) :- step(T), T1 is T + 1, b(T, B), \+flips(T, B).
b(T1, B) :- step(T), T1 is T + 1, flips(T, B), \+b(T, B).
lost(T1) :- step(T), T1 is T + 1, lost(T).
lost(T1) :- step(T), T1 is T + 1, \+sp(T), \+fw(T).
% value(B, V): the bits below B of the message read V
value(0, 0).
value(B1, V1) :- value(B, V), weight(B, W), diamonds(K), b(K, B), B1 is B + 1, V1 is V + W.
value(B1, V) :- value(B, V), weight(B, _), diamonds(K), \+b(K, B), B1 is B + 1.
m(K, V) :- diamonds(K), \+lost(K), value(4, V).
m(K, 16) :- diamonds(K), lost(K).
query(m($1,0)).
query(m($1,1)).
query(m($1,2)).
query(m($1,3)).
query(m($1,4)).
query(m($1,5)).
query(m($1,6)).
query(m($1,7)).
query(m($1,8)).
query(m($1,9)).
query(m($1,10)).
query(m($1,11)).
query(m($1,12)).
query(m($1,13)).
query(m($1,14)).
query(m($1,15)).
query(m($1,16)).
