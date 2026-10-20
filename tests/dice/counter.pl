0.3::h.
obs.
0.3::h_1.
n2_b :- h_1, \+h.
n2_b :- \+h_1, h.
n2_b_1 :- h_1, h.
obs_1.
0.3::h_2.
n2_then_b :- n2_b, \+n2_b_1.
n2_then_b :- \+n2_b, n2_b_1.
n2_then_b_1 :- n2_b, n2_b_1.
n2_b_2 :- h_2, \+n2_b.
n2_b_2 :- \+h_2, n2_b.
n2_b_3 :- h_2, n2_then_b.
n2_b_3 :- \+h_2, n2_b_1.
n2_b_4 :- h_2, n2_then_b_1.
obs_2.
result_2 :- n2_b_2, n2_b_3, \+n2_b_4.
0.3::h_3.
n2_then_b_2 :- n2_b_2, \+n2_b_3.
n2_then_b_2 :- \+n2_b_2, n2_b_3.
n2_then_b_3 :- n2_b_2, n2_b_3.
n2_then_b_4 :- n2_b_4, \+n2_then_b_3.
n2_then_b_4 :- \+n2_b_4, n2_then_b_3.
n2_b_5 :- h_3, \+n2_b_2.
n2_b_5 :- \+h_3, n2_b_2.
n2_b_6 :- h_3, n2_then_b_2.
n2_b_6 :- \+h_3, n2_b_3.
n2_b_7 :- h_3, n2_then_b_4.
n2_b_7 :- \+h_3, n2_b_4.
obs_3.
result_2_1 :- n2_b_5, n2_b_6, \+n2_b_7.
result_2_2 :- result_2.
result_2_2 :- result_2_1.
0.3::h_4.
n2_then_b_5 :- n2_b_5, \+n2_b_6.
n2_then_b_5 :- \+n2_b_5, n2_b_6.
n2_then_b_6 :- n2_b_5, n2_b_6.
n2_then_b_7 :- n2_b_7, \+n2_then_b_6.
n2_then_b_7 :- \+n2_b_7, n2_then_b_6.
n2_b_8 :- h_4, \+n2_b_5.
n2_b_8 :- \+h_4, n2_b_5.
n2_b_9 :- h_4, n2_then_b_5.
n2_b_9 :- \+h_4, n2_b_6.
n2_b_10 :- h_4, n2_then_b_7.
n2_b_10 :- \+h_4, n2_b_7.
tmp :- n2_b_8, \+n2_b_9, n2_b_10.
obs_4 :- \+tmp.
result_2_3 :- n2_b_8, n2_b_9, \+n2_b_10.
result_2_4 :- result_2_2.
result_2_4 :- result_2_3.
0.3::h_5.
n2_then_b_8 :- n2_b_8, \+n2_b_9.
n2_then_b_8 :- \+n2_b_8, n2_b_9.
n2_then_b_9 :- n2_b_8, n2_b_9.
n2_then_b_10 :- n2_b_10, \+n2_then_b_9.
n2_then_b_10 :- \+n2_b_10, n2_then_b_9.
n2_b_11 :- h_5, \+n2_b_8.
n2_b_11 :- \+h_5, n2_b_8.
n2_b_12 :- h_5, n2_then_b_8.
n2_b_12 :- \+h_5, n2_b_9.
n2_b_13 :- h_5, n2_then_b_10.
n2_b_13 :- \+h_5, n2_b_10.
tmp_1 :- n2_b_11, \+n2_b_12, n2_b_13.
obs_5 :- \+tmp_1.
result_2_5 :- n2_b_11, n2_b_12, \+n2_b_13.
result_2_6 :- result_2_4.
result_2_6 :- result_2_5.
evidence(obs_4, true).
evidence(obs_5, true).
n2(0) :- \+n2_b_11, \+n2_b_12, \+n2_b_13.
query(n2(0)).
n2(1) :- n2_b_11, \+n2_b_12, \+n2_b_13.
query(n2(1)).
n2(2) :- \+n2_b_11, n2_b_12, \+n2_b_13.
query(n2(2)).
n2(3) :- n2_b_11, n2_b_12, \+n2_b_13.
query(n2(3)).
n2(4) :- \+n2_b_11, \+n2_b_12, n2_b_13.
query(n2(4)).
n2(5) :- n2_b_11, \+n2_b_12, n2_b_13.
query(n2(5)).
n2(6) :- \+n2_b_11, n2_b_12, n2_b_13.
query(n2(6)).
query(result_2_6).
