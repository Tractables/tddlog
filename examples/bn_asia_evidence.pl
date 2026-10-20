% The asia network (examples/asia.net) written by scripts/gen_bn.py
% --query all --evidence 2 --seed 3: both leaves observed in a forward sample.
0.01::asia(yes); 0.99::asia(no).
0.05::tub(yes); 0.95::tub(no) :- asia(yes).
0.01::tub(yes); 0.99::tub(no) :- asia(no).
0.5::smoke(yes); 0.5::smoke(no).
0.1::lung(yes); 0.9::lung(no) :- smoke(yes).
0.01::lung(yes); 0.99::lung(no) :- smoke(no).
0.6::bronc(yes); 0.4::bronc(no) :- smoke(yes).
0.3::bronc(yes); 0.7::bronc(no) :- smoke(no).
either(yes) :- lung(yes), tub(yes).
either(yes) :- lung(yes), tub(no).
either(yes) :- lung(no), tub(yes).
either(no) :- lung(no), tub(no).
0.98::xray(yes); 0.02::xray(no) :- either(yes).
0.05::xray(yes); 0.95::xray(no) :- either(no).
0.9::dysp(yes); 0.1::dysp(no) :- bronc(yes), either(yes).
0.8::dysp(yes); 0.2::dysp(no) :- bronc(yes), either(no).
0.7::dysp(yes); 0.3::dysp(no) :- bronc(no), either(yes).
0.1::dysp(yes); 0.9::dysp(no) :- bronc(no), either(no).
evidence(xray(yes)).
evidence(dysp(no)).
query(asia(yes)).
query(asia(no)).
query(tub(yes)).
query(tub(no)).
query(smoke(yes)).
query(smoke(no)).
query(lung(yes)).
query(lung(no)).
query(bronc(yes)).
query(bronc(no)).
query(either(yes)).
query(either(no)).
