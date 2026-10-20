0.4::x_c.
0.4::x_c_1.
x_b :- x_c_1, \+x_c.
x_b :- \+x_c_1, x_c.
x_b_1 :- x_c_1, x_c.
0.4::x_c_2.
x_then_b :- x_b, \+x_b_1.
x_then_b :- \+x_b, x_b_1.
x_else_b :- x_b.
x_else_b :- \+x_b.
x_else_b_1 :- \+x_b_1, \+x_else_b.
x_else_b_1 :- x_b_1, x_else_b.
x_b_2 :- x_c_2, \+x_b.
x_b_2 :- \+x_c_2, x_b.
x_b_3 :- x_c_2, x_then_b.
x_b_3 :- \+x_c_2, x_else_b_1.
x_c_3 :- \+x_b_2, \+x_b_3.
0.4::x_c_4.
x_g :- \+x_c_3, x_c_4.
x_g_1 :- \+x_c_3, \+x_c_4.
x_else_b_2 :- \+x_b_3, \+x_b_2.
x_else_b_2 :- x_b_3, x_b_2.
x_b_4 :- x_g.
x_b_4 :- x_g_1.
x_b_5 :- x_g, x_b_2.
x_b_5 :- x_g_1, \+x_b_2.
x_b_6 :- x_g, x_b_3.
x_b_6 :- x_g_1, x_else_b_2.
x_c_5 :- \+x_b_4, \+x_b_5, \+x_b_6.
x_c_6 :- x_b_4, x_b_5, x_b_6.
x_c_7 :- x_c_5.
x_c_7 :- x_c_6.
0.4::x_c_8.
x_g_2 :- \+x_c_7, x_c_8.
x_then_b_1 :- x_b_5, \+x_b_6.
x_then_b_1 :- \+x_b_5, x_b_6.
x_g_3 :- \+x_c_7, \+x_c_8.
x_else_b_3 :- x_b_5.
x_else_b_3 :- \+x_b_5.
x_else_b_4 :- \+x_b_6, \+x_else_b_3.
x_else_b_4 :- x_b_6, x_else_b_3.
x_b_7 :- x_c_7, x_b_4.
x_b_8 :- x_c_7, x_b_5.
x_b_8 :- x_g_2, \+x_b_5.
x_b_8 :- x_g_3, x_b_5.
x_b_9 :- x_c_7, x_b_6.
x_b_9 :- x_g_2, x_then_b_1.
x_b_9 :- x_g_3, x_else_b_4.
x_c_9 :- \+x_b_7, \+x_b_8, \+x_b_9.
x_c_10 :- x_b_7, x_b_8, x_b_9.
x_c_11 :- x_c_9.
x_c_11 :- x_c_10.
0.4::x_c_12.
x_g_4 :- \+x_c_11, x_c_12.
x_g_5 :- \+x_c_11, \+x_c_12.
x_else_b_5 :- \+x_b_9, \+x_b_8.
x_else_b_5 :- x_b_9, x_b_8.
x_b_10 :- x_c_11, x_b_7.
x_b_10 :- x_g_4.
x_b_10 :- x_g_5.
x_b_11 :- x_c_11, x_b_8.
x_b_11 :- x_g_4, x_b_8.
x_b_11 :- x_g_5, \+x_b_8.
x_b_12 :- x_c_11, x_b_9.
x_b_12 :- x_g_4, x_b_9.
x_b_12 :- x_g_5, x_else_b_5.
x_c_13 :- \+x_b_10, \+x_b_11, \+x_b_12.
x_c_14 :- x_b_10, x_b_11, x_b_12.
x_c_15 :- x_c_13.
x_c_15 :- x_c_14.
0.4::x_c_16.
x_g_6 :- \+x_c_15, x_c_16.
x_then_b_2 :- x_b_11, \+x_b_12.
x_then_b_2 :- \+x_b_11, x_b_12.
x_g_7 :- \+x_c_15, \+x_c_16.
x_else_b_6 :- x_b_11.
x_else_b_6 :- \+x_b_11.
x_else_b_7 :- \+x_b_12, \+x_else_b_6.
x_else_b_7 :- x_b_12, x_else_b_6.
x_b_13 :- x_c_15, x_b_10.
x_b_14 :- x_c_15, x_b_11.
x_b_14 :- x_g_6, \+x_b_11.
x_b_14 :- x_g_7, x_b_11.
x_b_15 :- x_c_15, x_b_12.
x_b_15 :- x_g_6, x_then_b_2.
x_b_15 :- x_g_7, x_else_b_7.
x_c_17 :- \+x_b_13, \+x_b_14, \+x_b_15.
x_c_18 :- x_b_13, x_b_14, x_b_15.
x_c_19 :- x_c_17.
x_c_19 :- x_c_18.
0.4::x_c_20.
x_g_8 :- \+x_c_19, x_c_20.
x_g_9 :- \+x_c_19, \+x_c_20.
x_else_b_8 :- \+x_b_15, \+x_b_14.
x_else_b_8 :- x_b_15, x_b_14.
x_b_16 :- x_c_19, x_b_13.
x_b_16 :- x_g_8.
x_b_16 :- x_g_9.
x_b_17 :- x_c_19, x_b_14.
x_b_17 :- x_g_8, x_b_14.
x_b_17 :- x_g_9, \+x_b_14.
x_b_18 :- x_c_19, x_b_15.
x_b_18 :- x_g_8, x_b_15.
x_b_18 :- x_g_9, x_else_b_8.
x_c_21 :- \+x_b_16, \+x_b_17, \+x_b_18.
x_c_22 :- x_b_16, x_b_17, x_b_18.
x_c_23 :- x_c_21.
x_c_23 :- x_c_22.
0.4::x_c_24.
x_g_10 :- \+x_c_23, x_c_24.
x_then_b_3 :- x_b_17, \+x_b_18.
x_then_b_3 :- \+x_b_17, x_b_18.
x_g_11 :- \+x_c_23, \+x_c_24.
x_else_b_9 :- x_b_17.
x_else_b_9 :- \+x_b_17.
x_else_b_10 :- \+x_b_18, \+x_else_b_9.
x_else_b_10 :- x_b_18, x_else_b_9.
x_b_19 :- x_c_23, x_b_16.
x_b_20 :- x_c_23, x_b_17.
x_b_20 :- x_g_10, \+x_b_17.
x_b_20 :- x_g_11, x_b_17.
x_b_21 :- x_c_23, x_b_18.
x_b_21 :- x_g_10, x_then_b_3.
x_b_21 :- x_g_11, x_else_b_10.
x_c_25 :- \+x_b_19, \+x_b_20, \+x_b_21.
x_c_26 :- x_b_19, x_b_20, x_b_21.
x_c_27 :- x_c_25.
x_c_27 :- x_c_26.
0.4::x_c_28.
x_g_12 :- \+x_c_27, x_c_28.
x_g_13 :- \+x_c_27, \+x_c_28.
x_else_b_11 :- \+x_b_21, \+x_b_20.
x_else_b_11 :- x_b_21, x_b_20.
x_b_22 :- x_c_27, x_b_19.
x_b_22 :- x_g_12.
x_b_22 :- x_g_13.
x_b_23 :- x_c_27, x_b_20.
x_b_23 :- x_g_12, x_b_20.
x_b_23 :- x_g_13, \+x_b_20.
x_b_24 :- x_c_27, x_b_21.
x_b_24 :- x_g_12, x_b_21.
x_b_24 :- x_g_13, x_else_b_11.
tmp :- \+x_b_22, \+x_b_23, \+x_b_24.
obs :- \+tmp.
evidence(obs, true).
x(0) :- \+x_b_22, \+x_b_23, \+x_b_24.
query(x(0)).
x(1) :- x_b_22, \+x_b_23, \+x_b_24.
query(x(1)).
query(x(2)).
x(3) :- x_b_22, x_b_23, \+x_b_24.
query(x(3)).
query(x(4)).
x(5) :- x_b_22, \+x_b_23, x_b_24.
query(x(5)).
query(x(6)).
x(7) :- x_b_22, x_b_23, x_b_24.
query(x(7)).
