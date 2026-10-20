% Win/move game on a probabilistic DAG: negation through recursion,
% acyclic once ground.
0.5::move(1,2). 0.5::move(1,3). 0.5::move(2,3). 0.5::move(3,4).
win(X) :- move(X,Y), \+ win(Y).
query(win(1)).
query(win(2)).
query(win(3)).
