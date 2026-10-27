%Expected outcome:
% pal([a, b, a, b, a]) 0.018
% pal([a, b, b, b, a]) 0.018
% pal([a, b, b, a]) 0
%
% The expected outcomes are computed by hand: each odd palindrome has one
% derivation, the product of its rules' probabilities (0.3, 0.3 and 0.2
% for a b a b a), and an even word has none.
% A grammar of odd palindromes over a and b, written as the stack of a
% leftmost derivation and the terminals derived so far, reversed. Asked
% for a word, the magic rewrite demands each state from the next one, so
% the switch of the annotated disjunction has its arguments bound by the
% demand before the recursive literal; it is scheduled after it.
state([], [s]).
0.3::state(W, [a,s,a|S]) ; 0.3::state(W, [b,s,b|S]) ; 0.2::state(W, [a|S]) ; 0.2::state(W, [b|S]) :- state(W, [s|S]).
state([a|W], S) :- state(W, [a|S]).
state([b|W], S) :- state(W, [b|S]).

pal(W) :- reverse(W, [], R), state(R, []).
reverse([], L, L).
reverse([H|T], A, R) :- reverse(T, [H|A], R).

query(pal([a,b,a,b,a])).
query(pal([a,b,b,b,a])).
query(pal([a,b,b,a])).
