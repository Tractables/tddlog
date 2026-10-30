% 100 fair dice are thrown. What is the probability that their sum is a
% prime number? That it is divisible by 7? That it is at least 400?
% Run with `tddlog examples/larger/dice_sum.pl`.
dice(100).
die(D) :- dice(N), between(1, N, D).
1/6::face(D, 1); 1/6::face(D, 2); 1/6::face(D, 3);
1/6::face(D, 4); 1/6::face(D, 5); 1/6::face(D, 6) :- die(D).
% sum(D, S): the first D dice add up to S.
sum(0, 0).
sum(D, S) :- die(D), C is D - 1, sum(C, R), face(D, F), S is R + F.
total(S) :- dice(N), sum(N, S).
composite(S) :- between(2, S, K), K * K =< S, S mod K =:= 0.
prime :- total(S), S >= 2, \+ composite(S).
sevens :- total(S), S mod 7 =:= 0.
high :- total(S), dice(N), S >= 4 * N.
query(prime).
query(sevens).
query(high).
