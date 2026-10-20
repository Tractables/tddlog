% A growing head of 8 atoms: a_i is an annotated disjunction over
% a0..a_{i-1} caused by a_i, and a 0.5 fact. The heads share each
% disjunction's switches, which merges the acyclic atoms into one
% component. P(a0) = 1 - 2^-7 up to the 1e-10 each disjunction leaves
% out: a0 holds once any of a1..a7 holds as a fact, since every
% disjunction picks a lower atom. tests/it/examples.rs and src/auto/tests.rs.
a0 :- a1.
0.5::a1.
0.4999999999::a0; 0.4999999999::a1 :- a2.
0.5::a2.
0.3333333332::a0; 0.3333333332::a1; 0.3333333332::a2 :- a3.
0.5::a3.
0.2499999999::a0; 0.2499999999::a1; 0.2499999999::a2; 0.2499999999::a3 :- a4.
0.5::a4.
0.1999999999::a0; 0.1999999999::a1; 0.1999999999::a2; 0.1999999999::a3; 0.1999999999::a4 :- a5.
0.5::a5.
0.1666666666::a0; 0.1666666666::a1; 0.1666666666::a2; 0.1666666666::a3; 0.1666666666::a4; 0.1666666666::a5 :- a6.
0.5::a6.
0.1428571428::a0; 0.1428571428::a1; 0.1428571428::a2; 0.1428571428::a3; 0.1428571428::a4; 0.1428571428::a5; 0.1428571428::a6 :- a7.
0.5::a7.
query(a0).
