% Dice (cplint dice example): throw a die until it shows 6; non-stratified
% through the rewrite (\+ on(X1,6) under recursion) and safe only under the
% demand of the query. Evidence on the first two throws.
0.16666666666666666::on(0,1); 0.16666666666666666::on(0,2); 0.16666666666666666::on(0,3); 0.16666666666666666::on(0,4); 0.16666666666666666::on(0,5); 0.16666666666666666::on(0,6).
0.16666666666666666::on(X,1); 0.16666666666666666::on(X,2); 0.16666666666666666::on(X,3); 0.16666666666666666::on(X,4); 0.16666666666666666::on(X,5); 0.16666666666666666::on(X,6) :- X1 is X-1,X1>=0,on(X1,_),
  \+ on(X1,6).
evidence :- on(0,1),
  on(1,1).
query(on(0,1)).
query(on(1,1)).
query(on(2,1)).
evidence(on(0,1)).
evidence(evidence).
evidence(on(1,1)).
