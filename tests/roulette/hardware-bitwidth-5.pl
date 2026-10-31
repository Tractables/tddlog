% gen_roulette.py hardware-bitwidth 5
% Roulette hardware.rkt, bit width 5, n = 2, fuel 3: t(J, K) and c(J, K),
% bit K of the total and the counter entering the loop with J iterations done, on(J) while
% it runs; z(J, R) resets a cell before the R-th read of iteration J: counter (test, d),
% total (u), counter (sum, g), counter (step, f); x(J) leaves the loop at iteration J.
% tx(J, K) and tc(J, K): the sum bit without the carry, and the carry into it, of the
% total; cc(J, K) the carry of the counter (+ 1); r(K) the total at exit.
width(5).
fuel(3).
step(0).
step(J1) :- step(J), fuel(F), J1 is J + 1, J1 < F.
0.0001::z(J,1) :- step(J).
0.0001::z(J,2) :- step(J).
0.0001::z(J,3) :- step(J).
0.0001::z(J,4) :- step(J).
c(0,1).
on(0).
d(J,K) :- step(J), c(J,K), \+z(J,1).
nz(J) :- d(J,_).
x(J) :- step(J), on(J), \+nz(J).
on(J1) :- step(J), J1 is J + 1, on(J), nz(J).
u(J,K) :- step(J), t(J,K), \+z(J,2).
g(J,K) :- d(J,K), \+z(J,3).
f(J,K) :- g(J,K), \+z(J,4).
tx(J1,K) :- step(J), J1 is J + 1, u(J,K), \+g(J,K).
tx(J1,K) :- step(J), J1 is J + 1, g(J,K), \+u(J,K).
tc(J1,K1) :- step(J), J1 is J + 1, u(J,K), g(J,K), K1 is K + 1, width(W), K1 < W.
tc(J,K1) :- tc(J,K), tx(J,K), K1 is K + 1, width(W), K1 < W.
t(J,K) :- tx(J,K), \+tc(J,K).
t(J,K) :- tc(J,K), \+tx(J,K).
c(J1,0) :- step(J), J1 is J + 1, \+f(J,0).
cc(J1,1) :- step(J), J1 is J + 1, f(J,0), width(W), 1 < W.
cc(J,K1) :- cc(J,K), P is J - 1, f(P,K), K1 is K + 1, width(W), K1 < W.
c(J1,K) :- step(J), J1 is J + 1, f(J,K), K > 0, \+cc(J1,K).
c(J,K) :- cc(J,K), P is J - 1, \+f(P,K).
done :- x(_).
r(K) :- x(J), t(J,K).
hi :- r(K), K >= 3.
res(0) :- done, \+r(0), \+r(1), \+r(2), \+hi.
res(1) :- done, r(0), \+r(1), \+r(2), \+hi.
res(2) :- done, \+r(0), r(1), \+r(2), \+hi.
res(3) :- done, r(0), r(1), \+r(2), \+hi.
res(5) :- done, r(0), \+r(1), r(2), \+hi.
res(6) :- fuel(F), on(F).
query(res(0)).
query(res(1)).
query(res(2)).
query(res(3)).
query(res(5)).
query(res(6)).
