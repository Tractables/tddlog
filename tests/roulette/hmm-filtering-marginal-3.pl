% gen_roulette.py hmm-filtering-marginal 3
% Roulette hmm.rkt, filtering-marginal, 3 steps.
0.5::h(0).
0.3::t1(1).
0.65::t0(1).
0.65::e1(1).
0.3::e0(1).
h(1) :- h(0), t1(1).
h(1) :- \+h(0), t0(1).
v(1) :- h(1), e1(1).
v(1) :- \+h(1), e0(1).
0.3::t1(2).
0.65::t0(2).
0.65::e1(2).
0.3::e0(2).
h(2) :- h(1), t1(2).
h(2) :- \+h(1), t0(2).
v(2) :- h(2), e1(2).
v(2) :- \+h(2), e0(2).
0.3::t1(3).
0.65::t0(3).
0.65::e1(3).
0.3::e0(3).
h(3) :- h(2), t1(3).
h(3) :- \+h(2), t0(3).
v(3) :- h(3), e1(3).
v(3) :- \+h(3), e0(3).
evidence(v(1), true).
evidence(v(2), true).
evidence(v(3), true).
c(3) :- h(3).
q :- c(3).
query(q).
