% Grounded from its source (src/ground/linear.rs), the closure gets a
% chain atom $r_path_2(1,Y) for every path(1,Y), with the same rules
% wherever Y is not 1, and grounding merges each chain atom into its path
% atom. A queried atom is not merged away: p1 and q1 have the same bodies
% and both stay, so t is false in every world.
% Run with `tddlog examples/equal_atoms.pl`; `tddlog --ground examples/equal_atoms.pl`
% prints the program the grounder writes.
0.6::edge(1,2). 0.5::edge(2,3). 0.4::edge(3,1). 0.7::edge(3,4).
0.3::edge(2,4). 0.5::edge(4,5).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
0.3::a. 0.6::b.
p1 :- a, path(1,4).
p1 :- b.
q1 :- b.
q1 :- path(1,4), a.
t :- p1, \+q1.
evidence(path(1,3), true).
query(path(1,1)).
query(path(1,5)).
query(path(1,4)).
query(p1).
query(q1).
query(t).
