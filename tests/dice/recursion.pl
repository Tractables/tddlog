0.5::result_else.
0.5::result_else_else.
0.5::result_else_else_else.
result_else_else_1 :- result_else_else, result_else_else_else.
result_else_1 :- result_else, result_else_else_1.
query(result_else_1).
