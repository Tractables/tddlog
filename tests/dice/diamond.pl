diamond(0).
step(0).
step(T1) :- step(T), T < 99, T1 is T + 1.
0.5::route(T) :- step(T).
s2(T) :- route(T), diamond(T).
s3(T) :- diamond(T), \+route(T).
0.0001::drop(T) :- step(T).
result(T) :- s3(T), \+drop(T).
result_1(T) :- s2(T).
result_1(T) :- result(T).
diamond(T1) :- step(T), T1 is T + 1, result_1(T).
result_2 :- diamond(100).
query(result_2).
