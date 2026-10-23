% Alarm example (Fierens et al. 2015, Example 1): a burglary or an
% earthquake sets off the alarm, and a neighbour who hears it calls. Given
% that John calls, how likely are a burglary, an earthquake and a call
% from Mary? Run with `tddlog examples/alarm.pl`; the README and
% docs/design.md work through it.
0.1::burglary.
0.2::earthquake.
0.7::hears_alarm(john).
0.4::hears_alarm(mary).
alarm :- burglary.
alarm :- earthquake.
calls(X) :- alarm, hears_alarm(X).
evidence(calls(john), true).
query(burglary).
query(earthquake).
query(calls(mary)).
