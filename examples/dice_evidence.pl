% Dice evidence2: an observation inside one branch of a conditional holds
% only when that branch is taken (Dice answers 0.666667).
0.5::evidence.
0.5::coin1.
obs :- evidence, coin1.
obs :- \+evidence.
0.5::result_else.
result :- evidence, coin1.
result :- \+evidence, result_else.
evidence(obs, true).
query(result).
