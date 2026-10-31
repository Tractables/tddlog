/*
PRISM's switches under `:- set_pita(prism_memoization, true).`: one
choice for each instance of a switch, however often msw/2 calls it.
Under PITA's default, a new choice at each call, two_heads would be
0.36 and mixed 0.24.
*/
:- use_module(library(pita)).

:- pita.

:- set_pita(prism_memoization, true).

:- begin_lpad.

values(coin, [heads, tails]).
:- set_sw(coin, [0.6, 0.4]).
values(die(_), [1, 2, 3]).
:- set_sw(die(red), [0.5, 0.3, 0.2]).
:- set_sw(die(blue), [0.2, 0.3, 0.5]).

two_heads :- msw(coin, heads), msw(coin, heads).
mixed :- msw(coin, heads), msw(coin, tails).
same(V) :- msw(die(red), V), msw(die(blue), V).
again(V) :- msw(die(red), V), msw(die(red), W), W >= V.

:- end_lpad.

/** <examples>
?- prob(two_heads, P). % 0.6
?- prob(mixed, P). % 0
?- prob(same(2), P). % 0.3 * 0.3 = 0.09
?- prob(again(2), P). % 0.3
*/
