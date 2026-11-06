% The hidden Markov model of Roulette's hmm.rkt, ported to ProbLog: a
% hidden chain h(0) ~ flip 0.5, h(T + 1) ~ flip 0.3 (t1) after a true h(T)
% and flip 0.65 (t0) after a false one, and an observation v(T + 1) ~ flip
% 0.65 (e1) where h(T + 1) holds and flip 0.3 (e0) where it does not, for
% $2 steps. The observations, consulted from $1, are every v true, or none
% for the marginal likelihood. The question $3 is q:
%   marginal_likelihood  v(1) .. v($2) all true (seen);
%   filtering_marginal   h($2);
%   smoothing_marginal   h(0);
%   filtering_joint      h(T) for T = $2 // 2 .. $2 all true (held);
%   smoothing_joint      h(T) for T = 0 .. $2 // 2 - 1 all true.
:- consult($1).
steps($2).
step(0) :- steps(N), N > 0.
step(T1) :- step(T), steps(N), T1 is T + 1, T1 < N.
0.5::h(0).
0.3::t1(T) :- step(T).
0.65::t0(T) :- step(T).
0.65::e1(T) :- step(T).
0.3::e0(T) :- step(T).
h(T1) :- step(T), T1 is T + 1, h(T), t1(T).
h(T1) :- step(T), T1 is T + 1, \+h(T), t0(T).
v(T1) :- step(T), T1 is T + 1, h(T1), e1(T).
v(T1) :- step(T), T1 is T + 1, \+h(T1), e0(T).
likelihood(marginal_likelihood).
seen(0).
seen(T1) :- step(T), T1 is T + 1, seen(T), v(T1).
q :- likelihood($3), steps(N), seen(N).
asks(filtering_marginal, N, N) :- steps(N).
asks(smoothing_marginal, 0, 0).
asks(filtering_joint, F, N) :- steps(N), F is N // 2.
asks(smoothing_joint, 0, L) :- steps(N), L is N // 2 - 1.
held(F) :- asks($3, F, L), F =< L, h(F).
held(T1) :- asks($3, _, L), held(T), T1 is T + 1, T1 =< L, h(T1).
q :- asks($3, F, L), L < F.
q :- asks($3, _, L), held(L).
query(q).
