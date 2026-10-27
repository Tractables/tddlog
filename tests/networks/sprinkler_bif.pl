% gen_bn.py's program for sprinkler.bif --query WetGrass: Rain's rows in the
% order the file writes them.
0.5::cloudy(true); 0.5::cloudy(false).
0.1::sprinkler(true); 0.9::sprinkler(false) :- cloudy(true).
0.5::sprinkler(true); 0.5::sprinkler(false) :- cloudy(false).
0.2::rain(true); 0.8::rain(false) :- cloudy(false).
0.8::rain(true); 0.2::rain(false) :- cloudy(true).
0.99::wetgrass(true); 0.01::wetgrass(false) :- sprinkler(true), rain(true).
0.9::wetgrass(true); 0.1::wetgrass(false) :- sprinkler(true), rain(false).
0.9::wetgrass(true); 0.1::wetgrass(false) :- sprinkler(false), rain(true).
wetgrass(false) :- sprinkler(false), rain(false).
query(wetgrass(true)).
query(wetgrass(false)).
