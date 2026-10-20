% Detached evidence that no assignment of p1 and p2 satisfies, while unit
% propagation alone derives nothing from it: the four observations are the
% four clauses over p1 and p2. ProbLog 2.3.0 reports the evidence
% inconsistent.
0.3::a.
q :- a.
query(q).
0.5::p1. 0.5::p2.
o1 :- p1.
o1 :- p2.
o2 :- p1.
o2 :- \+p2.
o3 :- \+p1.
o3 :- p2.
o4 :- \+p1.
o4 :- \+p2.
evidence(o1). evidence(o2). evidence(o3). evidence(o4).
