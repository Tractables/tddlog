% Negation through a cycle that is two-valued in every world:
% with f, a holds and b fails; without f, b holds and a fails.
% ProbLog refuses the program; run it with
% `tddlog examples/negcycle_sound.pl`.
0.3::f.
a :- \+b.
b :- \+a.
a :- f.
b :- \+f.
query(a).
query(b).
