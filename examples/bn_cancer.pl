% The cancer network (examples/cancer.net) written by scripts/gen_bn.py:
% one annotated disjunction per CPT row, the leaf marginals queried.
0.9::pollution(low); 0.1::pollution(high).
0.3::smoker(true); 0.7::smoker(false).
0.03::cancer(true); 0.97::cancer(false) :- pollution(low), smoker(true).
0.001::cancer(true); 0.999::cancer(false) :- pollution(low), smoker(false).
0.05::cancer(true); 0.95::cancer(false) :- pollution(high), smoker(true).
0.02::cancer(true); 0.98::cancer(false) :- pollution(high), smoker(false).
0.9::xray(positive); 0.1::xray(negative) :- cancer(true).
0.2::xray(positive); 0.8::xray(negative) :- cancer(false).
0.65::dyspnoea(true); 0.35::dyspnoea(false) :- cancer(true).
0.3::dyspnoea(true); 0.7::dyspnoea(false) :- cancer(false).
query(xray(positive)).
query(xray(negative)).
query(dyspnoea(true)).
query(dyspnoea(false)).
