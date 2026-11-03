% examples/car.net with the evidence of examples/car.inst (a cold morning:
% bright headlights, the fuel gauge low, and the car does not start) and
% three queries, as `tddlog --program --query fuel,plugs,starter --evidence
% examples/car.inst examples/car.net` prints it. Run with `tddlog
% examples/bn_car_evidence.pl`; the guide works through it:
% https://tractables.github.io/tddlog/guide/#bayesian-network
0.3::weather(cold); 0.5::weather(mild); 0.2::weather(hot).
0.7::battery_age(new); 0.3::battery_age(old).
0.1::lights_left_on(yes); 0.9::lights_left_on(no).
0.2::battery(charged); 0.4::battery(weak); 0.4::battery(flat) :- battery_age(new), lights_left_on(yes).
0.9::battery(charged); 0.08::battery(weak); 0.02::battery(flat) :- battery_age(new), lights_left_on(no).
0.05::battery(charged); 0.35::battery(weak); 0.6::battery(flat) :- battery_age(old), lights_left_on(yes).
0.6::battery(charged); 0.3::battery(weak); 0.1::battery(flat) :- battery_age(old), lights_left_on(no).
0.95::headlights(bright); 0.04::headlights(dim); 0.01::headlights(off) :- battery(charged).
0.15::headlights(bright); 0.75::headlights(dim); 0.1::headlights(off) :- battery(weak).
0.1::headlights(dim); 0.9::headlights(off) :- battery(flat).
0.97::starter(ok); 0.03::starter(faulty).
0.97::cranks(yes); 0.02::cranks(slowly); 0.01::cranks(no) :- battery(charged), starter(ok).
0.1::cranks(slowly); 0.9::cranks(no) :- battery(charged), starter(faulty).
0.3::cranks(yes); 0.6::cranks(slowly); 0.1::cranks(no) :- battery(weak), starter(ok).
0.05::cranks(slowly); 0.95::cranks(no) :- battery(weak), starter(faulty).
0.05::cranks(slowly); 0.95::cranks(no) :- battery(flat), starter(ok).
cranks(no) :- battery(flat), starter(faulty).
0.55::fuel(full); 0.35::fuel(low); 0.1::fuel(empty).
0.9::fuel_gauge(full); 0.08::fuel_gauge(low); 0.02::fuel_gauge(empty) :- fuel(full).
0.1::fuel_gauge(full); 0.8::fuel_gauge(low); 0.1::fuel_gauge(empty) :- fuel(low).
0.3::fuel_gauge(low); 0.7::fuel_gauge(empty) :- fuel(empty).
0.8::plugs(ok); 0.2::plugs(fouled) :- weather(cold).
0.95::plugs(ok); 0.05::plugs(fouled) :- weather(mild).
0.97::plugs(ok); 0.03::plugs(fouled) :- weather(hot).
0.99::spark(yes); 0.01::spark(no) :- plugs(ok), battery(charged).
0.9::spark(yes); 0.1::spark(no) :- plugs(ok), battery(weak).
spark(no) :- plugs(ok), battery(flat).
0.4::spark(yes); 0.6::spark(no) :- plugs(fouled), battery(charged).
0.2::spark(yes); 0.8::spark(no) :- plugs(fouled), battery(weak).
spark(no) :- plugs(fouled), battery(flat).
0.98::fires(yes); 0.02::fires(no) :- spark(yes), fuel(full).
0.95::fires(yes); 0.05::fires(no) :- spark(yes), fuel(low).
fires(no) :- spark(yes), fuel(empty).
fires(no) :- spark(no), fuel(full).
fires(no) :- spark(no), fuel(low).
fires(no) :- spark(no), fuel(empty).
0.99::starts(yes); 0.01::starts(no) :- cranks(yes), fires(yes).
starts(no) :- cranks(yes), fires(no).
0.6::starts(yes); 0.4::starts(no) :- cranks(slowly), fires(yes).
starts(no) :- cranks(slowly), fires(no).
starts(no) :- cranks(no), fires(yes).
starts(no) :- cranks(no), fires(no).
evidence(weather(cold)).
evidence(headlights(bright)).
evidence(fuel_gauge(low)).
evidence(starts(no)).
query(fuel(full)).
query(fuel(low)).
query(fuel(empty)).
query(plugs(ok)).
query(plugs(fouled)).
query(starter(ok)).
query(starter(faulty)).
