% The Markov chain of Roulette's motivating.rkt (the paper's Figure 2),
% ported to ProbLog: x(0) ~ flip 0.1, and after step T, x(T + 1) is flip
% 0.2 (a) where x(T) holds and flip 0.3 (b) where it does not, for $1
% steps, the program's argument; the query is x($1).
steps($1).
step(0) :- steps(N), N > 0.
step(T1) :- step(T), steps(N), T1 is T + 1, T1 < N.
0.2::a(T) :- step(T).
0.3::b(T) :- step(T).
0.1::x(0).
x(T1) :- step(T), T1 is T + 1, x(T), a(T).
x(T1) :- step(T), T1 is T + 1, \+x(T), b(T).
query(x($1)).
