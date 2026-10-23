% Gear wheels (Bogaerts & Van den Broeck 2015, Example 4.13): two wheels that
% turn each other, buttons that toggle them. Negation through a positive loop;
% needs the well-founded (unfoundedness) refinement.
% Run with `tddlog examples/gearwheels.pl`; the guide works through it:
% https://tractables.github.io/tddlog/guide/#negation
0.3::button1(0).
0.6::button2(0).
turns1(0) :- turns2(0).
turns2(0) :- turns1(0).
turns1(1) :- turns2(1).
turns2(1) :- turns1(1).
turns1(1) :- turns1(0), \+button1(0).
turns2(1) :- turns2(0), \+button2(0).
turns1(1) :- \+turns1(0), button1(0).
turns2(1) :- \+turns2(0), button2(0).
query(turns1(0)).
query(turns1(1)).
query(turns2(1)).
