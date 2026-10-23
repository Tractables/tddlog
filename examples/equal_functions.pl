% Gates equal in function but written differently, as a circuit compiler
% writes a value it computes twice by two routes. x + y is computed twice:
% the gates of s write each sum bit as (a and not b) or (b and not a) and
% the carry as a majority of three, the gates of t write the sum bits as
% (a or b) and not (a and b) and the carry as (a and b) or (c and (a or
% b)). No two of t's gates have the rules of one of s's, so structural
% hashing merges none of them; random simulation proposes each pair and
% the SAT solver proves it, and the grounder merges each of t's gates into
% s's (src/ground/aig.rs, `reduce`).
% n0 is the complement of s0 and is read as its negation, and k and
% differ are false only in function. A queried or observed gate stays.
% Run with `tddlog examples/equal_functions.pl`; `tddlog --ground examples/equal_functions.pl`
% prints the program the grounder writes.
0.3::x0. 0.6::x1. 0.7::y0. 0.2::y1. 0.4::c. 0.5::z.

% s = x + y + c: bits s0 and s1, carry out s2.
s0 :- x0, \+y0, \+c.
s0 :- y0, \+x0, \+c.
s0 :- c, x0, y0.
s0 :- c, \+x0, \+y0.
c1 :- x0, y0.
c1 :- x0, c.
c1 :- y0, c.
s1 :- x1, \+y1, \+c1.
s1 :- y1, \+x1, \+c1.
s1 :- c1, x1, y1.
s1 :- c1, \+x1, \+y1.
s2 :- x1, y1.
s2 :- x1, c1.
s2 :- y1, c1.
% t = x + y + c again, by other gates.
o0 :- x0.
o0 :- y0.
b0 :- x0, y0.
h0 :- o0, \+b0.
oc :- h0.
oc :- c.
bc :- h0, c.
t0 :- oc, \+bc.
d1 :- b0.
d1 :- bc.
o1 :- x1.
o1 :- y1.
b1 :- x1, y1.
h1 :- o1, \+b1.
od :- h1.
od :- d1.
bd :- h1, d1.
t1 :- od, \+bd.
t2 :- b1.
t2 :- bd.
% The complement of s0, a constant, and an absorption.
n0 :- x0, y0, \+c.
n0 :- \+x0, \+y0, \+c.
n0 :- c, x0, \+y0.
n0 :- c, y0, \+x0.
k :- t0, n0.
a :- x0, o0.
% Readers of both.
differ :- s1, \+t1.
differ :- t1, \+s1.
alarm :- differ, z.
low :- \+n0, z.
low :- k.
both :- t1, t2, z.
either :- t1, a.
either :- s2, \+z.

evidence(either, true).
query(s1).
query(alarm).
query(low).
query(both).
query(t0).
