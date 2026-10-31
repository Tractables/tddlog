%Expected outcome:
% q([2, 3]) 0.4
%
% The items of a list that a second list holds, by memberchk/2 on each
% item, under a probabilistic fact; computed by hand: inter/3 keeps 2 and
% 3, and q holds where keep does. A semi-naive round of the grounder joins
% from inter/3's new answers, which bind the second list before the
% demand binds the item: memberchk/2 then bound the item to the list's
% first value, 3, and the answer for [2, 3] was lost. memberchk/2 waits
% for the item the literals written before it bind.
:- use_module(library(lists)).
inter([], _, []).
inter([X|T], L, [X|R]) :- memberchk(X, L), inter(T, L, R).
inter([X|T], L, R) :- \+ memberchk(X, L), inter(T, L, R).
0.4::keep.
q(R) :- keep, inter([1,2,3], [3,2], R).
query(q(_)).
