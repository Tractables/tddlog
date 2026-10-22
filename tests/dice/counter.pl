step(0).
step(T1) :- step(T), T < 5, T1 is T + 1.
0.3::h(T) :- step(T).
n2_then_b(T) :- step_1_b0(T), \+step_1_b1(T).
n2_then_b(T) :- step_1_b1(T), \+step_1_b0(T).
n2_then_b_1(T) :- step_1_b0(T), step_1_b1(T).
n2_then_b_2(T) :- n2_then_b_1(T), \+step_1_b2(T).
n2_then_b_2(T) :- step_1_b2(T), \+n2_then_b_1(T).
n2_b(T) :- h(T), \+step_1_b0(T).
n2_b(T) :- step_1_b0(T), \+h(T).
n2_b_1(T) :- h(T), n2_then_b(T).
n2_b_1(T) :- step_1_b1(T), \+h(T).
n2_b_2(T) :- h(T), n2_then_b_2(T).
n2_b_2(T) :- step_1_b2(T), \+h(T).
tmp(T) :- n2_b(T), n2_b_2(T), \+n2_b_1(T).
obs(T) :- step(T), \+tmp(T).
result_2(T) :- n2_b(T), n2_b_1(T), \+n2_b_2(T).
result_2_1(T) :- step_2(T).
result_2_1(T) :- result_2(T).
step_1_b0(T1) :- step(T), T1 is T + 1, n2_b(T).
step_1_b1(T1) :- step(T), T1 is T + 1, n2_b_1(T).
step_1_b2(T1) :- step(T), T1 is T + 1, n2_b_2(T).
step_2(T1) :- step(T), T1 is T + 1, result_2_1(T).
failed :- step(T), \+obs(T).
evidence(failed, false).
n2(0) :- \+step_1_b0(6), \+step_1_b1(6), \+step_1_b2(6).
query(n2(0)).
n2(1) :- step_1_b0(6), \+step_1_b1(6), \+step_1_b2(6).
query(n2(1)).
n2(2) :- \+step_1_b0(6), step_1_b1(6), \+step_1_b2(6).
query(n2(2)).
n2(3) :- step_1_b0(6), step_1_b1(6), \+step_1_b2(6).
query(n2(3)).
n2(4) :- \+step_1_b0(6), \+step_1_b1(6), step_1_b2(6).
query(n2(4)).
n2(5) :- step_1_b0(6), \+step_1_b1(6), step_1_b2(6).
query(n2(5)).
n2(6) :- \+step_1_b0(6), step_1_b1(6), step_1_b2(6).
query(n2(6)).
n2(7) :- step_1_b0(6), step_1_b1(6), step_1_b2(6).
query(n2(7)).
result_2_2 :- step_2(6).
query(result_2_2).
