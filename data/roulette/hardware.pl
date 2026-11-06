% The interpreter of Roulette's scaling/hardware.rkt, ported to ProbLog: a
% loop over a memory of two cells of $2 bits, the total and the counter,
% whose every read first resets the cell to 0 with probability 0.0001
% (z(J, R) before the R-th read of iteration J), running `tri n`: the
% counter starts at $3 (n as `(int n)` holds it in $2 bits), then while
% the counter is not 0, total := total + counter and counter := counter +
% 1, for at most $4 iterations, the fuel; entered with none left, the loop
% ends the run with no result. The cells are bit vectors as Roulette's,
% their set bits atoms: t(J, K) and c(J, K), bit K of the total and the
% counter entering iteration J, on(J) while the loop runs, x(J) when it
% leaves at iteration J. The reads: d (the counter, its test), u (the
% total), g (the counter, its sum), f (the counter, its step). The sums
% ripple their carries up to bit $2 - 1 and drop the last, as bvadd does:
% tx and tc are the total's bit without the carry and the carry into it,
% cc the counter's carry. The queries, consulted from $1 with the totals a
% run can end with (total/1), are res(T) for each of them and res(L), L
% one past the largest, for a run out of fuel (out_of_fuel/1).
:- consult($1).
width($2).
start($3).
fuel($4).
step(0) :- fuel(F), F > 0.
step(J1) :- step(J), fuel(F), J1 is J + 1, J1 < F.
0.0001::z(J, 1) :- step(J).
0.0001::z(J, 2) :- step(J).
0.0001::z(J, 3) :- step(J).
0.0001::z(J, 4) :- step(J).
% one(X, K): bit K of the number X is set, for the numbers the program reads
number_read(C) :- start(C).
number_read(T) :- total(T).
digits(X, 0, X) :- number_read(X).
digits(X, K1, R1) :- digits(X, K, R), R > 1, K1 is K + 1, R1 is R // 2.
one(X, K) :- digits(X, K, R), R mod 2 =:= 1.
c(0, K) :- start(C), one(C, K).
on(0).
d(J, K) :- step(J), c(J, K), \+z(J, 1).
nz(J) :- d(J, _).
x(J) :- step(J), on(J), \+nz(J).
on(J1) :- step(J), J1 is J + 1, on(J), nz(J).
u(J, K) :- step(J), t(J, K), \+z(J, 2).
g(J, K) :- d(J, K), \+z(J, 3).
f(J, K) :- g(J, K), \+z(J, 4).
% the total := u + g
tx(J1, K) :- step(J), J1 is J + 1, u(J, K), \+g(J, K).
tx(J1, K) :- step(J), J1 is J + 1, g(J, K), \+u(J, K).
tc(J1, K1) :- step(J), J1 is J + 1, u(J, K), g(J, K), K1 is K + 1, width(W), K1 < W.
tc(J, K1) :- tc(J, K), tx(J, K), K1 is K + 1, width(W), K1 < W.
t(J, K) :- tx(J, K), \+tc(J, K).
t(J, K) :- tc(J, K), \+tx(J, K).
% the counter := f + 1
c(J1, 0) :- step(J), J1 is J + 1, \+f(J, 0).
cc(J1, 1) :- step(J), J1 is J + 1, f(J, 0), width(W), 1 < W.
cc(J, K1) :- cc(J, K), P is J - 1, f(P, K), K1 is K + 1, width(W), K1 < W.
c(J1, K) :- step(J), J1 is J + 1, f(J, K), K > 0, \+cc(J1, K).
c(J, K) :- cc(J, K), P is J - 1, \+f(P, K).
% the total the loop left with, and whether it is T
done :- x(_).
r(K) :- x(J), t(J, K).
mismatch(T) :- total(T), one(T, K), \+r(K).
mismatch(T) :- total(T), r(K), \+one(T, K).
res(T) :- total(T), done, \+mismatch(T).
res(L) :- out_of_fuel(L), fuel(F), on(F).
