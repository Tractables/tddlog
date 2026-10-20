% A circuit as Alea.jl writes one: every gate an atom of its own, and the
% same gates written again where the program computes a value twice. Here
% x + y is computed twice, by the gates of s and of t, and t's upper bits
% read t's own carry, so they have s's rules only once that carry is
% merged into s's. The grounder merges each gate with an earlier gate's
% rules into it, so that each is compiled once (src/ground/aig.rs,
% `reduce`); a queried or observed gate stays.
0.3::x0. 0.6::x1. 0.7::y0. 0.2::y1. 0.5::z.

% s = x + y: bits s0 and s1, carry out s2.
c0 :- x0, y0.
s0 :- x0, \+y0.
s0 :- y0, \+x0.
h1 :- x1, \+y1.
h1 :- y1, \+x1.
s1 :- h1, \+c0.
s1 :- c0, \+h1.
s2 :- x1, y1.
s2 :- h1, c0.
% t = x + y again, gate for gate.
d0 :- x0, y0.
t0 :- x0, \+y0.
t0 :- y0, \+x0.
k1 :- x1, \+y1.
k1 :- y1, \+x1.
t1 :- k1, \+d0.
t1 :- d0, \+k1.
t2 :- x1, y1.
t2 :- k1, d0.
% Whether the two sums differ, and readers of both.
differ :- s1, \+t1.
differ :- t1, \+s1.
differ :- s2, \+t2.
differ :- t2, \+s2.
odd_and_big :- t0, s2.
both :- s1, t2, z.
either :- t1, z.
either :- s2.

evidence(odd_and_big, false).
query(s1).
query(t1).
query(differ).
query(both).
query(either).
query(s0).
