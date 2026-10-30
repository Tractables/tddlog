% A game of stones: from a heap, the players in turn take 1, 2 or 3 stones,
% but each move from each heap size is allowed only with probability 0.6,
% independently; a player who cannot move loses. A heap is won where some
% allowed move leads to a heap that is not: winning is defined through its
% own negation. From a heap of 1, 2, 4, 5, 10, 50 and 4000 stones, what is
% the probability that the player to move wins?
% Run with `tddlog examples/larger/stones.pl`.
:- use_module(library(lists)).
heap(4000).
0.6::move(N, M) :- heap(H), between(1, H, N), member(T, [1, 2, 3]), M is N - T, M >= 0.
win(N) :- move(N, M), \+ win(M).
query(win(N)) :- heap(H), member(N, [1, 2, 4, 5, 10, 50, H]).
