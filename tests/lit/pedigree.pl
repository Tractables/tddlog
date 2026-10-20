% blood type on a random pedigree: 3 generations of 3, observed 0.3, seed 1
% rules of cplint's bloodtype example (data/cplint/bloodtype.pl)
0.9::mchrom(Person,a); 0.05::mchrom(Person,b); 0.05::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,a   ), mchrom(Mother,a   ).
0.49::mchrom(Person,a); 0.49::mchrom(Person,b); 0.02::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,b   ), mchrom(Mother,a   ).
0.49::mchrom(Person,a); 0.02::mchrom(Person,b); 0.49::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,null), mchrom(Mother,a   ).
0.49::mchrom(Person,a); 0.49::mchrom(Person,b); 0.02::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,a   ), mchrom(Mother,b   ).
0.05::mchrom(Person,a); 0.9::mchrom(Person,b); 0.05::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,b   ), mchrom(Mother,b   ).
0.02::mchrom(Person,a); 0.49::mchrom(Person,b); 0.49::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,null), mchrom(Mother,b   ).
0.49::mchrom(Person,a); 0.02::mchrom(Person,b); 0.49::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,a   ), mchrom(Mother,null).
0.02::mchrom(Person,a); 0.49::mchrom(Person,b); 0.49::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,b   ), mchrom(Mother,null).
0.05::mchrom(Person,a); 0.05::mchrom(Person,b); 0.9::mchrom(Person,null) :- mother(Mother,Person), pchrom(Mother,null), mchrom(Mother,null).
0.9::pchrom(Person,a); 0.05::pchrom(Person,b); 0.05::pchrom(Person,null) :- father(Father,Person), pchrom(Father,a   ), mchrom(Father,a   ).
0.49::pchrom(Person,a); 0.49::pchrom(Person,b); 0.02::pchrom(Person,null) :- father(Father,Person), pchrom(Father,b   ), mchrom(Father,a   ).
0.49::pchrom(Person,a); 0.02::pchrom(Person,b); 0.49::pchrom(Person,null) :- father(Father,Person), pchrom(Father,null), mchrom(Father,a   ).
0.49::pchrom(Person,a); 0.49::pchrom(Person,b); 0.02::pchrom(Person,null) :- father(Father,Person), pchrom(Father,a   ), mchrom(Father,b   ).
0.05::pchrom(Person,a); 0.9::pchrom(Person,b); 0.05::pchrom(Person,null) :- father(Father,Person), pchrom(Father,b   ), mchrom(Father,b   ).
0.02::pchrom(Person,a); 0.49::pchrom(Person,b); 0.49::pchrom(Person,null) :- father(Father,Person), pchrom(Father,null), mchrom(Father,b   ).
0.49::pchrom(Person,a); 0.02::pchrom(Person,b); 0.49::pchrom(Person,null) :- father(Father,Person), pchrom(Father,a   ), mchrom(Father,null).
0.02::pchrom(Person,a); 0.49::pchrom(Person,b); 0.49::pchrom(Person,null) :- father(Father,Person), pchrom(Father,b   ), mchrom(Father,null).
0.05::pchrom(Person,a); 0.05::pchrom(Person,b); 0.9::pchrom(Person,null) :- father(Father,Person), pchrom(Father,null), mchrom(Father,null).
0.9::bloodtype(Person,a); 0.03::bloodtype(Person,b); 0.03::bloodtype(Person,ab); 0.04::bloodtype(Person,null) :- pchrom(Person,a   ),mchrom(Person,a   ).
0.03::bloodtype(Person,a); 0.03::bloodtype(Person,b); 0.9::bloodtype(Person,ab); 0.04::bloodtype(Person,null) :- pchrom(Person,b   ),mchrom(Person,a   ).
0.9::bloodtype(Person,a); 0.04::bloodtype(Person,b); 0.03::bloodtype(Person,ab); 0.03::bloodtype(Person,null) :- pchrom(Person,null),mchrom(Person,a   ).
0.03::bloodtype(Person,a); 0.03::bloodtype(Person,b); 0.9::bloodtype(Person,ab); 0.04::bloodtype(Person,null) :- pchrom(Person,a   ),mchrom(Person,b   ).
0.04::bloodtype(Person,a); 0.9::bloodtype(Person,b); 0.03::bloodtype(Person,ab); 0.03::bloodtype(Person,null) :- pchrom(Person,b   ),mchrom(Person,b   ).
0.03::bloodtype(Person,a); 0.09::bloodtype(Person,b); 0.04::bloodtype(Person,ab); 0.03::bloodtype(Person,null) :- pchrom(Person,null),mchrom(Person,b   ).
0.9::bloodtype(Person,a); 0.03::bloodtype(Person,b); 0.03::bloodtype(Person,ab); 0.04::bloodtype(Person,null) :- pchrom(Person,a   ),mchrom(Person,null).
0.03::bloodtype(Person,a); 0.9::bloodtype(Person,b); 0.04::bloodtype(Person,ab); 0.03::bloodtype(Person,null) :- pchrom(Person,b   ),mchrom(Person,null).
0.03::bloodtype(Person,a); 0.04::bloodtype(Person,b); 0.03::bloodtype(Person,ab); 0.9::bloodtype(Person,null) :- pchrom(Person,null),mchrom(Person,null).
0.3::mchrom(i0_0,a); 0.3::mchrom(i0_0,b); 0.4::mchrom(i0_0,null).
0.3::pchrom(i0_0,a); 0.3::pchrom(i0_0,b); 0.4::pchrom(i0_0,null).
0.3::mchrom(i0_1,a); 0.3::mchrom(i0_1,b); 0.4::mchrom(i0_1,null).
0.3::pchrom(i0_1,a); 0.3::pchrom(i0_1,b); 0.4::pchrom(i0_1,null).
0.3::mchrom(i0_2,a); 0.3::mchrom(i0_2,b); 0.4::mchrom(i0_2,null).
0.3::pchrom(i0_2,a); 0.3::pchrom(i0_2,b); 0.4::pchrom(i0_2,null).
father(i0_0,i1_0).
mother(i0_1,i1_0).
father(i0_2,i1_1).
mother(i0_1,i1_1).
father(i0_2,i1_2).
mother(i0_1,i1_2).
father(i1_2,i2_0).
mother(i1_1,i2_0).
father(i1_0,i2_1).
mother(i1_1,i2_1).
father(i1_2,i2_2).
mother(i1_1,i2_2).
query(bloodtype(i2_0,a)).
query(bloodtype(i2_0,b)).
query(bloodtype(i2_0,ab)).
query(bloodtype(i2_0,null)).
query(bloodtype(i2_1,a)).
query(bloodtype(i2_1,b)).
query(bloodtype(i2_1,ab)).
query(bloodtype(i2_1,null)).
query(bloodtype(i2_2,a)).
query(bloodtype(i2_2,b)).
query(bloodtype(i2_2,ab)).
query(bloodtype(i2_2,null)).
