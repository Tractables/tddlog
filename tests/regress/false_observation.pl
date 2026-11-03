% An impossible observation (Dice's `observe false`) as tddlog's Dice
% reader writes it: evidence on an atom defined false. ProbLog 2.3.0 stops
% with InconsistentEvidenceError, by default and with -k sdd, and without
% the rule with UnknownClause. tddlog finds the evidence inconsistent,
% P(e) = 0.
0.5::x.
false_observation :- fail.
evidence(false_observation, true).
query(x).
