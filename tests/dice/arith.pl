0.7::x_f1.
0.6666666666666666::x_f0.
0.5714285714285714::x_f0_1.
x_b0 :- \+x_f1, x_f0.
x_b0 :- x_f1, x_f0_1.
0.6666666666666666::y_f1.
0.5::y_f0.
y_b0 :- \+y_f1.
y_b0 :- y_f1, y_f0.
s_b :- x_b0, \+y_b0.
s_b :- \+x_b0, y_b0.
s_b_1 :- x_b0, y_b0.
s_b_2 :- x_f1, \+y_f1.
s_b_2 :- \+x_f1, y_f1.
s_b_3 :- s_b_1, \+s_b_2.
s_b_3 :- \+s_b_1, s_b_2.
d_b :- \+y_b0, \+x_b0.
d_b :- y_b0, x_b0.
d_b_1 :- x_b0, \+y_b0.
d_b_1 :- d_b.
d_b_2 :- \+y_f1, \+x_f1.
d_b_2 :- y_f1, x_f1.
d_b_3 :- d_b_1, \+d_b_2.
d_b_3 :- \+d_b_1, d_b_2.
m_b :- y_b0, x_b0.
m_b_1 :- y_b0, x_f1.
m_b_2 :- y_f1, x_b0.
m_b_3 :- m_b_1, \+m_b_2.
m_b_3 :- \+m_b_1, m_b_2.
z_c_b :- \+d_b, \+s_b.
z_c_b :- d_b, s_b.
z_c_b_1 :- d_b_3, \+s_b_3.
z_c_b_1 :- \+d_b_3, s_b_3.
z_c :- \+s_b, \+d_b.
z_c_1 :- \+s_b_3, d_b_3.
z_c_1 :- \+z_c_b_1, z_c.
z_b :- z_c_1, m_b.
z_b_1 :- z_c_1, m_b_3.
z_b_1 :- \+z_c_1, s_b.
0.3::w_c.
w_else_b :- z_b, \+z_b_1.
w_else_b :- \+z_b, z_b_1.
w_b :- w_c, z_b_1.
w_b :- \+w_c, z_b.
w_b_1 :- \+w_c, w_else_b.
tmp :- \+w_b, \+w_b_1.
tmp_1 :- \+x_b0, y_b0.
tmp_2 :- \+x_f1, y_f1.
tmp_2 :- \+s_b_2, tmp_1.
tmp_3 :- \+tmp.
tmp_3 :- \+tmp_2.
obs :- tmp_3.
evidence(obs, true).
w(0) :- \+w_b, \+w_b_1.
query(w(0)).
w(1) :- w_b, \+w_b_1.
query(w(1)).
w(2) :- \+w_b, w_b_1.
query(w(2)).
w(3) :- w_b, w_b_1.
query(w(3)).
