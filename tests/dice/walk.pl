step_b0(0).
step_b1(0).
step(0).
step(T1) :- step(T), T < 9, T1 is T + 1.
x_c(T) :- step(T), \+step_b0(T), \+step_b1(T), \+step_b2(T).
x_c_1(T) :- step_b0(T), step_b1(T), step_b2(T).
x_c_2(T) :- x_c(T).
x_c_2(T) :- x_c_1(T).
0.4::x_c_3(T) :- step(T).
x_g(T) :- x_c_3(T), \+x_c_2(T).
x_then_b(T) :- step_b0(T), \+step_b1(T).
x_then_b(T) :- step_b1(T), \+step_b0(T).
x_then_b_1(T) :- step_b0(T), step_b1(T).
x_then_b_2(T) :- step_b2(T), \+x_then_b_1(T).
x_then_b_2(T) :- x_then_b_1(T), \+step_b2(T).
x_g_1(T) :- step(T), \+x_c_2(T), \+x_c_3(T).
x_else_b(T) :- step(T), \+step_b1(T), \+step_b0(T).
x_else_b(T) :- step_b1(T), step_b0(T).
x_else_b_1(T) :- step_b1(T).
x_else_b_1(T) :- step_b0(T), \+step_b1(T).
x_else_b_2(T) :- step(T), \+step_b2(T), \+x_else_b_1(T).
x_else_b_2(T) :- step_b2(T), x_else_b_1(T).
x_b(T) :- x_c_2(T), step_b0(T).
x_b(T) :- x_g(T), \+step_b0(T).
x_b(T) :- x_g_1(T), \+step_b0(T).
x_b_1(T) :- x_c_2(T), step_b1(T).
x_b_1(T) :- x_g(T), x_then_b(T).
x_b_1(T) :- x_g_1(T), x_else_b(T).
x_b_2(T) :- x_c_2(T), step_b2(T).
x_b_2(T) :- x_g(T), x_then_b_2(T).
x_b_2(T) :- x_g_1(T), x_else_b_2(T).
step_b0(T1) :- step(T), T1 is T + 1, x_b(T).
step_b1(T1) :- step(T), T1 is T + 1, x_b_1(T).
step_b2(T1) :- step(T), T1 is T + 1, x_b_2(T).
tmp :- \+step_b0(10), \+step_b1(10), \+step_b2(10).
obs :- \+tmp.
evidence(obs, true).
x(0) :- \+step_b0(10), \+step_b1(10), \+step_b2(10).
query(x(0)).
x(1) :- step_b0(10), \+step_b1(10), \+step_b2(10).
query(x(1)).
x(2) :- \+step_b0(10), step_b1(10), \+step_b2(10).
query(x(2)).
x(3) :- step_b0(10), step_b1(10), \+step_b2(10).
query(x(3)).
x(4) :- \+step_b0(10), \+step_b1(10), step_b2(10).
query(x(4)).
x(5) :- step_b0(10), \+step_b1(10), step_b2(10).
query(x(5)).
x(6) :- \+step_b0(10), step_b1(10), step_b2(10).
query(x(6)).
x(7) :- step_b0(10), step_b1(10), step_b2(10).
query(x(7)).
