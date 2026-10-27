% gen_bn.py's program for noisy.xdsl, rows of Ace's pathfinder.xdsl whose doubles
% carry binary noise (0.00099404 written 0.0009940400000000005): each row is
% off one by 1e-8, the rounding of its eight decimals, and is divided by its sum.
0.3::mantle(present); 0.7::mantle(absent).
0.9940357800596422::f5(s1); 0.004970179950298201::f5(s2); 0.0009940399900596007::f5(s3) :- mantle(present).
0.99800399998004::f5(s1); 0.0009980000099800005::f5(s2); 0.0009980000099800005::f5(s3) :- mantle(absent).
0.9927370800726293::fault(s1); 0.007262919927370802::fault(s2).
query(mantle(present)).
query(mantle(absent)).
query(f5(s0)).
query(f5(s1)).
query(f5(s2)).
query(f5(s3)).
query(fault(s0)).
query(fault(s1)).
query(fault(s2)).
