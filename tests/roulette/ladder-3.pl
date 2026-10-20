% gen_roulette.py ladder 3
% Roulette ladder.rkt: from (true, false), a rung gives (false, false) if s1 and a (flip 0.001),
% else (b, not b) with b ~ flip 0.5 if s1 or s2, else (false, false).
0.001::a(3).
0.5::b(3).
0.001::a(2).
0.5::b(2).
0.001::a(1).
0.5::b(1).
o1(0).
o1(1) :- o1(0), \+a(1), b(1).
o1(1) :- \+o1(0), o2(0), b(1).
o2(1) :- o1(0), \+a(1), \+b(1).
o2(1) :- \+o1(0), o2(0), \+b(1).
o1(2) :- o1(1), \+a(2), b(2).
o1(2) :- \+o1(1), o2(1), b(2).
o2(2) :- o1(1), \+a(2), \+b(2).
o2(2) :- \+o1(1), o2(1), \+b(2).
o1(3) :- o1(2), \+a(3), b(3).
o1(3) :- \+o1(2), o2(2), b(3).
o2(3) :- o1(2), \+a(3), \+b(3).
o2(3) :- \+o1(2), o2(2), \+b(3).
query(o1(3)).
query(o2(3)).
