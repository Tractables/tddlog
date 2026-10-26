%Expected outcome:
% some_item 0.91
% prefix_items(0) 1
% prefix_items(1) 0.4
% prefix_items(2) 0.28
% prefix_items(3) 0.14
% last_item 0.5
% second_item 0.7
% first_index(0) 0.4
% first_index(1) 0.7
% first_index(2) 0.5
% reversed_first 0.5
% all_items([b, c]) 0.35
% joined([a, b, c, d]) 1
% pair(a-b) 0.28
% pair(a-c) 0.2
% pair(b-c) 0.35
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% The list predicates over lists whose elements are probabilistic facts.
:- use_module(library(lists)).
0.4::item(a). 0.7::item(b). 0.5::item(c).
basket([a, b, c]).

some_item :- basket(B), member(X, B), item(X).
all_items([]).
all_items([X|Xs]) :- item(X), all_items(Xs).
prefix_items(N) :- basket(B), append(P, _, B), length(P, N), all_items(P).
last_item :- basket(B), last(B, X), item(X).
second_item :- basket(B), nth1(2, B, X), item(X).
first_index(I) :- basket(B), nth0(I, B, X), item(X).
reversed_first :- basket(B), reverse(B, [X|_]), item(X).
joined(L) :- basket(B), append(B, [d], L).
pair(X-Y) :- basket(B), member(X, B), member(Y, B), X @< Y, item(X), item(Y).

query(some_item).
query(prefix_items(_)).
query(last_item).
query(second_item).
query(first_index(_)).
query(reversed_first).
query(all_items([b, c])).
query(joined(_)).
query(pair(_)).
