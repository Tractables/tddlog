% Evidence that propagation finds inconsistent: b makes a true, and a makes
% c true.
0.5::a.
b :- a.
c :- a.
evidence(b,true).
evidence(c,false).
query(a).
