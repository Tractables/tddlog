%Expected outcome:
% word([a]) 0.5
% word([b]) 0.2
% word([a, a]) 0.075
% word([a, b]) 0.03
% word([b, a]) 0.03
% word([b, b]) 0.012
% word([a, a, a]) 0.0225
% word([a, a, b]) 0.009
% word([a, b, a]) 0.009
% word([a, b, b]) 0.0036
%
% A probabilistic grammar's words of each length, read through an
% accumulator: reverse/3 answers with its accumulator, which only its
% caller binds, and the queries ask for words of variables. No demand
% makes it safe, and it is resolved top-down. The expected outcomes are
% ProbLog's answers.
% s -> 0.5 :: a | 0.2 :: b | 0.3 :: s s
state([], [s]).
0.5::state([a|W], S) ; 0.2::state([b|W], S) ; 0.3::state(W, [s,s|S]) :- state(W, [s|S]).

word(W) :- reverse(W, WR), state(WR, []).

reverse(L, R) :- reverse(L, [], R).
reverse([], L, L).
reverse([H|T], A, R) :- reverse(T, [H|A], R).

query(word([_])).
query(word([_,_])).
query(word([a,_,_])).
