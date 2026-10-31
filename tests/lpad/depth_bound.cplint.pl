/*
PITA's depth bound: a query is asked at depth 2, a clause with a body
holds at a depth where its body holds one level down, a fact at every
depth, and the choice of a clause's instance is one at every depth.
Computed by hand: small(s(0)) needs num(s(0)) at depth 1, from num(0)
at depth 0 and the choice for X = 0 (0.5), and not more(s(0)) (0.7):
0.35. small(s(s(0))) would need num(s(0)) at depth 0, where no clause
with a body holds: 0 (0.175 without the bound). c needs a at depth 1
and b, which needs a at depth 0; a is one choice at both: 0.5.
*/
:- use_module(library(pita)).
:- pita.
:- set_pita(depth_bound,true).
:- set_pita(depth,2).
:- begin_lpad.
num(0).
num(s(X)):0.5 :- num(X).
more(_):0.3.
small(X) :- num(X), \+ more(X).
a:0.5.
b :- a.
c :- a, b.
:- end_lpad.
/** <examples>
?- prob(small(s(0)),P).
?- prob(small(s(s(0))),P).
?- prob(c,P).
*/
