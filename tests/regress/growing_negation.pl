% A growing negated body of 10 atoms: a_i holds with probability 0.5 by
% its rule for the first a_j, j > i, that holds, read through the
% negations of the atoms between. P(a0) = 1/4: a9 holds with
% probability 1/2, a_i for i < 9 holds only if some atom above it holds,
% and then by one rule's 0.5 fact. tests/it/examples.rs and src/auto/tests.rs.
0.5::a0 :- a1.
0.5::a0 :- \+a1, a2.
0.5::a0 :- \+a1, \+a2, a3.
0.5::a0 :- \+a1, \+a2, \+a3, a4.
0.5::a0 :- \+a1, \+a2, \+a3, \+a4, a5.
0.5::a0 :- \+a1, \+a2, \+a3, \+a4, \+a5, a6.
0.5::a0 :- \+a1, \+a2, \+a3, \+a4, \+a5, \+a6, a7.
0.5::a0 :- \+a1, \+a2, \+a3, \+a4, \+a5, \+a6, \+a7, a8.
0.5::a0 :- \+a1, \+a2, \+a3, \+a4, \+a5, \+a6, \+a7, \+a8, a9.
0.5::a1 :- a2.
0.5::a1 :- \+a2, a3.
0.5::a1 :- \+a2, \+a3, a4.
0.5::a1 :- \+a2, \+a3, \+a4, a5.
0.5::a1 :- \+a2, \+a3, \+a4, \+a5, a6.
0.5::a1 :- \+a2, \+a3, \+a4, \+a5, \+a6, a7.
0.5::a1 :- \+a2, \+a3, \+a4, \+a5, \+a6, \+a7, a8.
0.5::a1 :- \+a2, \+a3, \+a4, \+a5, \+a6, \+a7, \+a8, a9.
0.5::a2 :- a3.
0.5::a2 :- \+a3, a4.
0.5::a2 :- \+a3, \+a4, a5.
0.5::a2 :- \+a3, \+a4, \+a5, a6.
0.5::a2 :- \+a3, \+a4, \+a5, \+a6, a7.
0.5::a2 :- \+a3, \+a4, \+a5, \+a6, \+a7, a8.
0.5::a2 :- \+a3, \+a4, \+a5, \+a6, \+a7, \+a8, a9.
0.5::a3 :- a4.
0.5::a3 :- \+a4, a5.
0.5::a3 :- \+a4, \+a5, a6.
0.5::a3 :- \+a4, \+a5, \+a6, a7.
0.5::a3 :- \+a4, \+a5, \+a6, \+a7, a8.
0.5::a3 :- \+a4, \+a5, \+a6, \+a7, \+a8, a9.
0.5::a4 :- a5.
0.5::a4 :- \+a5, a6.
0.5::a4 :- \+a5, \+a6, a7.
0.5::a4 :- \+a5, \+a6, \+a7, a8.
0.5::a4 :- \+a5, \+a6, \+a7, \+a8, a9.
0.5::a5 :- a6.
0.5::a5 :- \+a6, a7.
0.5::a5 :- \+a6, \+a7, a8.
0.5::a5 :- \+a6, \+a7, \+a8, a9.
0.5::a6 :- a7.
0.5::a6 :- \+a7, a8.
0.5::a6 :- \+a7, \+a8, a9.
0.5::a7 :- a8.
0.5::a7 :- \+a8, a9.
0.5::a8 :- a9.
0.5::a9.
query(a0).
