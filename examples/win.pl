% Win/move game on a probabilistic DAG: negation through recursion,
% acyclic once ground.
% Run with `tddlog examples/win.pl`; `tddlog --ground examples/win.pl`
% prints the program compiled. The guide works through it:
% https://tractables.github.io/tddlog/guide/#negation
0.5::move(1,2). 0.5::move(1,3). 0.5::move(2,3). 0.5::move(3,4).
win(X) :- move(X,Y), \+ win(Y).
query(win(1)).
query(win(2)).
query(win(3)).
