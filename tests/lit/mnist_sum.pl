% MNIST sum-2: 2 digits, peak 0.7, seed 1, true digits 29
0.0488::digit(1,0); 0.0439::digit(1,1); 0.7150::digit(1,2); 0.0285::digit(1,3); 0.0258::digit(1,4); 0.0375::digit(1,5); 0.0454::digit(1,6); 0.0054::digit(1,7); 0.0016::digit(1,8); 0.0481::digit(1,9).
0.0288::digit(2,0); 0.0508::digit(2,1); 0.0001::digit(2,2); 0.0297::digit(2,3); 0.0481::digit(2,4); 0.0152::digit(2,5); 0.0630::digit(2,6); 0.0601::digit(2,7); 0.0020::digit(2,8); 0.7022::digit(2,9).
next(1,2).
partial(1,S) :- digit(1,S).
partial(K,S) :- next(J,K), partial(J,S0), digit(K,D), S is S0+D.
sum(S) :- partial(2,S).
query(sum(0)).
query(sum(1)).
query(sum(2)).
query(sum(3)).
query(sum(4)).
query(sum(5)).
query(sum(6)).
query(sum(7)).
query(sum(8)).
query(sum(9)).
query(sum(10)).
query(sum(11)).
query(sum(12)).
query(sum(13)).
query(sum(14)).
query(sum(15)).
query(sum(16)).
query(sum(17)).
query(sum(18)).
