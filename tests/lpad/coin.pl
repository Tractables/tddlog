0.5::heads(Coin); 0.5::tails(Coin) :- toss(Coin), \+ biased(Coin).
0.6::heads(Coin); 0.4::tails(Coin) :- toss(Coin), biased(Coin).
0.9::fair(Coin); 0.1::biased(Coin).
toss(coin).
evidence(toss(coin)).
evidence(\+ fair(coin)).
query(heads(coin)).
query(tails(coin)).
% left out, asked under other evidence: prob(heads(coin), P)
