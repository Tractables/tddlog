ladder_1(0).
step(0).
step(T1) :- step(T), T < 99, T1 is T + 1.
0.001::result_then_c(T) :- step(T).
0.5::f(T) :- step(T).
result_then_1(T) :- f(T), \+result_then_c(T).
result_then_2(T) :- step(T), \+result_then_c(T), \+f(T).
result_g(T) :- ladder_2(T), \+ladder_1(T).
0.5::f_1(T) :- step(T).
result_g_1(T) :- step(T), \+ladder_1(T), \+ladder_2(T).
result_1(T) :- ladder_1(T), result_then_1(T).
result_1(T) :- result_g(T), f_1(T).
result_2(T) :- ladder_1(T), result_then_2(T).
result_2(T) :- result_g(T), \+f_1(T).
ladder_1(T1) :- step(T), T1 is T + 1, result_1(T).
ladder_2(T1) :- step(T), T1 is T + 1, result_2(T).
result_1_1 :- ladder_1(100).
query(result_1_1).
result_2_1 :- ladder_2(100).
query(result_2_1).
