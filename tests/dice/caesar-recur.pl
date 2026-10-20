0.5::key_f1.
0.5::key_f0.
0.25::gen_f1.
0.3333333333333333::gen_f0.
0.5::gen_f0_1.
gen_b0 :- \+gen_f1, gen_f0.
gen_b0 :- gen_f1, gen_f0_1.
enc_b :- gen_b0, \+key_f0.
enc_b :- \+gen_b0, key_f0.
enc_b_1 :- key_f0, gen_b0.
enc_b_2 :- gen_f1, \+key_f1.
enc_b_2 :- \+gen_f1, key_f1.
enc_b_3 :- enc_b_1, \+enc_b_2.
enc_b_3 :- \+enc_b_1, enc_b_2.
tmp :- \+enc_b, enc_b_3.
obs :- tmp.
0.25::gen_1_f1.
0.3333333333333333::gen_1_f0.
0.5::gen_1_f0_1.
gen_1_b0 :- \+gen_1_f1, gen_1_f0.
gen_1_b0 :- gen_1_f1, gen_1_f0_1.
enc_b_4 :- gen_1_b0, \+key_f0.
enc_b_4 :- \+gen_1_b0, key_f0.
enc_b_5 :- key_f0, gen_1_b0.
enc_b_6 :- gen_1_f1, \+key_f1.
enc_b_6 :- \+gen_1_f1, key_f1.
enc_b_7 :- enc_b_5, \+enc_b_6.
enc_b_7 :- \+enc_b_5, enc_b_6.
tmp_1 :- \+enc_b_4, enc_b_7.
obs_1 :- tmp_1.
0.25::gen_2_f1.
0.3333333333333333::gen_2_f0.
0.5::gen_2_f0_1.
gen_2_b0 :- \+gen_2_f1, gen_2_f0.
gen_2_b0 :- gen_2_f1, gen_2_f0_1.
enc_b_8 :- gen_2_b0, \+key_f0.
enc_b_8 :- \+gen_2_b0, key_f0.
enc_b_9 :- key_f0, gen_2_b0.
enc_b_10 :- gen_2_f1, \+key_f1.
enc_b_10 :- \+gen_2_f1, key_f1.
enc_b_11 :- enc_b_10, \+enc_b_9.
enc_b_11 :- \+enc_b_10, enc_b_9.
tmp_2 :- \+enc_b_8, enc_b_11.
obs_2 :- tmp_2.
0.25::gen_3_f1.
0.3333333333333333::gen_3_f0.
0.5::gen_3_f0_1.
gen_3_b0 :- \+gen_3_f1, gen_3_f0.
gen_3_b0 :- gen_3_f1, gen_3_f0_1.
enc_b_12 :- gen_3_b0, \+key_f0.
enc_b_12 :- \+gen_3_b0, key_f0.
enc_b_13 :- key_f0, gen_3_b0.
enc_b_14 :- gen_3_f1, \+key_f1.
enc_b_14 :- \+gen_3_f1, key_f1.
enc_b_15 :- enc_b_13, \+enc_b_14.
enc_b_15 :- \+enc_b_13, enc_b_14.
tmp_3 :- \+enc_b_12, enc_b_15.
obs_3 :- tmp_3.
evidence(obs, true).
evidence(obs_1, true).
evidence(obs_2, true).
evidence(obs_3, true).
key(0) :- \+key_f0, \+key_f1.
query(key(0)).
key(1) :- key_f0, \+key_f1.
query(key(1)).
key(2) :- \+key_f0, key_f1.
query(key(2)).
key(3) :- key_f0, key_f1.
query(key(3)).
