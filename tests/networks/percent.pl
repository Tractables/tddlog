% gen_bn.py's program for percent.hugin --rescale: every row divided by its
% sum, a row of one certain value a plain rule.
0.2::temp(v__12_); 0.5::temp(v_12_20_); 0.3::temp(v__20_).
0.9::ice(yes); 0.1::ice(no) :- temp(v__12_).
0.255::ice(yes); 0.745::ice(no) :- temp(v_12_20_).
ice(no) :- temp(v__20_).
query(temp(v__12_)).
query(temp(v_12_20_)).
query(temp(v__20_)).
query(ice(yes)).
query(ice(no)).
