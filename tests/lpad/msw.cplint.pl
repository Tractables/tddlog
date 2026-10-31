/*
PRISM's switches under PITA's default, which makes a new choice at each
call of msw/2, in programs that call each switch once: cplint's
coinmsw.pl, after the coin of Vennekens, Verbaeten and Bruynooghe's
"Logic programs with annotated disjunctions" (ICLP 2004), and SWISH's
sneezing_simple_msw.pl, whose switches have a variable.
*/
:- use_module(library(pita)).

:- pita.

:- begin_lpad.

values(throw(_),[heads,tails]).
:- set_sw(throw(fair),[0.5,0.5]).
:- set_sw(throw(biased),[0.6,0.4]).
values(fairness,[fair,biased]).
:- set_sw(fairness,[0.9,0.1]).

res(Coin,R):- toss(Coin),fairness(Coin,Fairness),msw(throw(Fairness),R).
fairness(_Coin,Fairness):-msw(fairness,Fairness).
toss(coin).

sneezing(X):- flu(X), msw(flu_sneezing(X),1).
sneezing(X):- hay_fever(X),msw(hay_fever_sneezing(X),1).
flu(bob).
hay_fever(bob).
values(flu_sneezing(_X),[1,0]).
values(hay_fever_sneezing(_X),[1,0]).
:- set_sw(flu_sneezing(_X),[0.7,0.3]).
:- set_sw(hay_fever_sneezing(_X),[0.8,0.2]).

:- end_lpad.

/** <examples>
?- prob(res(coin,heads),Prob). % 0.9 * 0.5 + 0.1 * 0.6 = 0.51
?- prob(res(coin,tails),Prob). % 0.49
?- prob(sneezing(bob),Prob). % 1 - 0.3 * 0.2 = 0.94
*/
