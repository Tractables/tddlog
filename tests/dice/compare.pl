0.25::x_f2.
0.6::x_f1.
x_b1 :- \+x_f2, x_f1.
0.3333333333333333::x_f0.
x_b0 :- \+x_f2, x_f0.
0.6::y_f2.
0.3333333333333333::y_f1.
y_b1 :- \+y_f2.
y_b1 :- y_f2, y_f1.
0.5::y_f0.
y_b0 :- \+y_f2, y_f0.
y_b0 :- y_f2, \+y_b1, y_f0.
a_b :- x_b0, \+y_b0.
a_b :- \+x_b0, y_b0.
a_b_1 :- x_b1, \+y_b1.
a_b_1 :- \+x_b1, y_b1.
a_b_2 :- x_f2, \+y_f2.
a_b_2 :- \+x_f2, y_f2.
a :- \+y_b0, x_b0.
a_1 :- \+y_b1, x_b1.
a_1 :- \+a_b_1, a.
a_2 :- \+y_f2, x_f2.
a_2 :- \+a_b_2, a_1.
b_b :- y_b0, \+y_b1.
b_b :- \+y_b0, y_b1.
b_b_1 :- y_b0, y_b1.
b_b_2 :- b_b_1, \+y_f2.
b_b_2 :- \+b_b_1, y_f2.
b_b_3 :- \+y_b0, \+x_b0.
b_b_3 :- y_b0, x_b0.
b_b_4 :- b_b, \+x_b1.
b_b_4 :- \+b_b, x_b1.
b_b_5 :- b_b_2, \+x_f2.
b_b_5 :- \+b_b_2, x_f2.
b :- y_b0, x_b0.
b_1 :- \+b_b, x_b1.
b_1 :- \+b_b_4, b.
b_2 :- \+b_b_2, x_f2.
b_2 :- \+b_b_5, b_1.
0.6::c.
tmp :- \+a_2, c.
tmp :- a_2, \+c.
obs :- \+tmp.
result_1 :- \+a_2, b_2.
result_1 :- a_2, \+b_2.
result_2_1_b :- x_b0, \+y_b0.
result_2_1_b :- b_b_3.
result_2_1_b_1 :- \+y_b1, \+x_b1.
result_2_1_b_1 :- y_b1, x_b1.
result_2_1_b_2 :- result_2_1_b, \+result_2_1_b_1.
result_2_1_b_2 :- \+result_2_1_b, result_2_1_b_1.
result_2_1_b_3 :- x_b1, \+y_b1.
result_2_1_b_3 :- result_2_1_b, result_2_1_b_1.
result_2_1_b_4 :- \+y_f2, \+x_f2.
result_2_1_b_4 :- y_f2, x_f2.
result_2_1_b_5 :- result_2_1_b_3, \+result_2_1_b_4.
result_2_1_b_5 :- \+result_2_1_b_3, result_2_1_b_4.
result_2_2_b :- \+x_b0.
result_2_2_b :- x_b0.
result_2_2_b_1 :- \+x_b1, \+result_2_2_b.
result_2_2_b_1 :- x_b1, result_2_2_b.
result_2_2_b_2 :- result_2_2_b, \+x_b1.
result_2_2_b_3 :- result_2_2_b_2, \+x_f2.
result_2_2_b_3 :- \+result_2_2_b_2, x_f2.
evidence(obs, true).
query(result_1).
result_2_1(0) :- b_b_3, \+result_2_1_b_2, \+result_2_1_b_5.
query(result_2_1(0)).
result_2_1(1) :- \+b_b_3, \+result_2_1_b_2, \+result_2_1_b_5.
query(result_2_1(1)).
result_2_1(2) :- b_b_3, result_2_1_b_2, \+result_2_1_b_5.
query(result_2_1(2)).
result_2_1(3) :- \+b_b_3, result_2_1_b_2, \+result_2_1_b_5.
query(result_2_1(3)).
result_2_1(4) :- b_b_3, \+result_2_1_b_2, result_2_1_b_5.
query(result_2_1(4)).
result_2_1(5) :- \+b_b_3, \+result_2_1_b_2, result_2_1_b_5.
query(result_2_1(5)).
result_2_1(6) :- b_b_3, result_2_1_b_2, result_2_1_b_5.
query(result_2_1(6)).
result_2_1(7) :- \+b_b_3, result_2_1_b_2, result_2_1_b_5.
query(result_2_1(7)).
query(result_2_2(0)).
result_2_2(1) :- \+x_b0, \+result_2_2_b_1, \+result_2_2_b_3.
query(result_2_2(1)).
result_2_2(2) :- x_b0, result_2_2_b_1, \+result_2_2_b_3.
query(result_2_2(2)).
result_2_2(3) :- \+x_b0, result_2_2_b_1, \+result_2_2_b_3.
query(result_2_2(3)).
result_2_2(4) :- x_b0, \+result_2_2_b_1, result_2_2_b_3.
query(result_2_2(4)).
result_2_2(5) :- \+x_b0, \+result_2_2_b_1, result_2_2_b_3.
query(result_2_2(5)).
