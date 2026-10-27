% `tddlog --program --query leaves --evidence sprinkler.inst
% sprinkler.xdsl`.
0.5::cloudy(t); 0.5::cloudy(f).
0.1::sprinkler(t); 0.9::sprinkler(f) :- cloudy(t).
0.5::sprinkler(t); 0.5::sprinkler(f) :- cloudy(f).
0.8::rain(t); 0.2::rain(f) :- cloudy(t).
0.2::rain(t); 0.8::rain(f) :- cloudy(f).
0.99::wetgrass(t); 0.01::wetgrass(f) :- sprinkler(t), rain(t).
0.9::wetgrass(t); 0.1::wetgrass(f) :- sprinkler(t), rain(f).
0.9::wetgrass(t); 0.1::wetgrass(f) :- sprinkler(f), rain(t).
wetgrass(f) :- sprinkler(f), rain(f).
evidence(sprinkler(t)).
query(wetgrass(t)).
query(wetgrass(f)).
