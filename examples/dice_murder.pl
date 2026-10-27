% Dice murderMystery: a function returning a tuple, inlined at its call
% (Dice answers 0.0158172).
% The program tddlog reads Dice's murderMystery.dice as (`tddlog --program`).
% Run with `tddlog examples/dice_murder.pl`.
0.3::alicedunnit.
0.03::withgun_then.
0.8::withgun_else.
withgun :- alicedunnit, withgun_then.
withgun :- \+alicedunnit, withgun_else.
obs :- withgun.
obs_1 :- obs.
evidence(obs_1, true).
query(alicedunnit).
