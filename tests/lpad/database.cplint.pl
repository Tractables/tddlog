/*
Database clauses outside the LPAD, which its bodies call through db/1
and findall/3, as cplint's testdb.pl and persons.pl do, and a program
over the LPAD that ProbLog's language does not read.
*/
:- use_module(library(pita)).

:- pita.

parent(ann, bob).
parent(bob, carl).

:- begin_lpad.

% Each person is sampled with probability one half.
sampled(X):0.5 :- db(person(X)).

% A trait passes from a parent to a child with probability 0.7.
trait(ann):0.4.
trait(Y):0.7 :- db(parent(X, Y)), trait(X).

big_family:0.9 :- findall(C, child(C), Cs), length(Cs, 2).

:- end_lpad.

person(X) :- parent(X, _).
person(X) :- parent(_, X).

child(Y) :- parent(_, Y).

% The probability of the trait, or 0 where it cannot be computed.
show(P) :- ( prob(trait(carl), Q) -> P = Q ; P = 0 ).

/** <examples>
?- prob(trait(carl), P). % 0.7 * 0.7 * 0.4 = 0.196
?- prob(sampled(carl), P). % 0.5
?- prob(big_family, P). % 0.9
?- show(P).
*/
