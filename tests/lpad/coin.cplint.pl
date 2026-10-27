/*
A coin that may be biased, after cplint's coin.pl: the example queries
ask under two sets of evidence, and one ProbLog program holds one.
*/
:- use_module(library(pita)).

:- pita.

:- begin_lpad.

heads(Coin):1/2 ; tails(Coin):1/2 :- toss(Coin), \+ biased(Coin).
% A biased coin lands heads more often.
heads(Coin):0.6 ; tails(Coin):0.4 :- toss(Coin), biased(Coin).

fair(Coin):0.9 ; biased(Coin):0.1.

toss(coin).

:- end_lpad.

/** <examples>
?- prob(heads(coin), (toss(coin), \+ fair(coin)), Prob). % 0.6
?- prob(tails(coin), (toss(coin), \+ fair(coin)), Prob). % 0.4
?- prob(heads(coin), Prob). % 0.51, under no evidence
*/
