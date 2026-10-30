% Mastermind: a secret code of 12 pegs, each one of 12 colours, all
% 8916100448256 codes equally likely. Each guess below was scored with black
% pegs (the right colour in the right place) and white pegs (a right colour
% in the wrong place). What is the code?
% Run with `tddlog examples/larger/mastermind.pl`. `--evidence-probability` adds P(e).
:- use_module(library(lists)).
pegs(12).
colours(12).
peg(P) :- pegs(N), between(1, N, P).
colour(C) :- colours(N), between(1, N, C).
1/12::code(P, 1); 1/12::code(P, 2); 1/12::code(P, 3); 1/12::code(P, 4);
1/12::code(P, 5); 1/12::code(P, 6); 1/12::code(P, 7); 1/12::code(P, 8);
1/12::code(P, 9); 1/12::code(P, 10); 1/12::code(P, 11); 1/12::code(P, 12) :- peg(P).
guess(1, [3, 2, 8, 5, 3, 2, 9, 12, 11, 1, 10, 7]).
guess(2, [8, 11, 12, 10, 11, 3, 10, 1, 9, 2, 1, 1]).
guess(3, [4, 4, 10, 1, 8, 6, 8, 10, 4, 9, 4, 11]).
guess(4, [5, 8, 1, 11, 2, 8, 11, 5, 7, 9, 2, 12]).
guess(5, [5, 6, 4, 9, 5, 1, 2, 10, 2, 7, 2, 5]).
guess(6, [7, 2, 1, 11, 1, 4, 4, 1, 8, 7, 12, 7]).
guess(7, [7, 2, 10, 11, 4, 11, 5, 6, 2, 5, 6, 1]).
guess(8, [7, 2, 3, 4, 12, 2, 1, 1, 8, 8, 3, 11]).
guess(9, [9, 4, 8, 9, 4, 12, 3, 7, 11, 7, 2, 7]).
guess(10, [7, 4, 1, 5, 10, 5, 1, 4, 3, 7, 10, 11]).
% holds(X, P, C): peg P of X, the code or a guess, has colour C.
holds(code, P, C) :- code(P, C).
holds(G, P, C) :- guess(G, Cs), peg(P), nth1(P, Cs, C).
row(code).
row(G) :- guess(G, _).
beyond(P) :- pegs(N), P is N + 1.
% count(X, C, P, K): K of the pegs of X from peg P on have colour C.
count(X, C, P, 0) :- row(X), colour(C), beyond(P).
count(X, C, P, K) :- peg(P), Q is P + 1, count(X, C, Q, J), holds(X, P, C), K is J + 1.
count(X, C, P, K) :- peg(P), Q is P + 1, count(X, C, Q, K), \+ holds(X, P, C).
% black(G, P, B): B of the pegs of guess G from peg P on match the code.
black(G, P, 0) :- guess(G, _), beyond(P).
black(G, P, B) :- peg(P), Q is P + 1, black(G, Q, A), holds(G, P, C), code(P, C), B is A + 1.
black(G, P, B) :- peg(P), Q is P + 1, black(G, Q, B), holds(G, P, C), \+ code(P, C).
% common(G, C, M): colours 1 to C of guess G match M of the code's pegs.
common(G, 0, 0) :- guess(G, _).
common(G, C, M) :- colour(C), B is C - 1, common(G, B, L),
    count(G, C, 1, J), count(code, C, 1, K), M is L + min(J, K).
score(G, B, W) :- black(G, 1, B), colours(N), common(G, N, M), W is M - B.
evidence(score(1, 1, 4)).
evidence(score(2, 2, 4)).
evidence(score(3, 0, 7)).
evidence(score(4, 0, 4)).
evidence(score(5, 0, 3)).
evidence(score(6, 1, 2)).
evidence(score(7, 0, 3)).
evidence(score(8, 1, 4)).
evidence(score(9, 1, 5)).
evidence(score(10, 0, 5)).
query(code(1, C)) :- colour(C).
