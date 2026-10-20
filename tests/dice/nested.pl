0.3333333333333333::a_f1.
0.5::a_f0.
a_b0 :- \+a_f1, a_f0.
b_c :- \+a_b0, \+a_f1.
0.1::b_then_f0.
b_c_1 :- a_b0, \+a_f1.
b_g :- \+b_c, b_c_1.
0.6::b_then_1_f0.
b_g_1 :- \+b_c, \+b_c_1.
0.8::b_else_f0.
b_b :- b_c, b_then_f0.
b_b :- b_g, b_then_1_f0.
b_b :- b_g_1, b_else_f0.
c_then_c :- \+a_b0, a_f1.
0.2::c_then_then_f1.
0.375::c_then_then_f0.
c_then_then_b0 :- \+c_then_then_f1, c_then_then_f0.
0.3::c_then_else_f1.
0.8571428571428571::c_then_else_f0.
c_then_else_b0 :- \+c_then_else_f1, c_then_else_f0.
c_then_b :- c_then_c, c_then_then_b0.
c_then_b :- \+c_then_c, c_then_else_b0.
c_then_b_1 :- c_then_c, c_then_then_f1.
c_then_b_1 :- \+c_then_c, c_then_else_f1.
0.3333333333333333::c_else_f1.
0.5::c_else_f0.
c_else_b0 :- \+c_else_f1, c_else_f0.
c_b :- \+b_b, c_then_b.
c_b :- b_b, c_else_b0.
c_b_1 :- \+b_b, c_then_b_1.
c_b_1 :- b_b, c_else_f1.
c(0) :- \+c_b, \+c_b_1.
query(c(0)).
c(1) :- c_b, \+c_b_1.
query(c(1)).
c(2) :- \+c_b, c_b_1.
query(c(2)).
