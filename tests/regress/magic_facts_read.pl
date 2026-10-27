%Expected outcome:
% path(1,4) 0.8496
% path(5,4) 0
%
% The expected outcomes are computed by hand: path(3,4) is 0.8, path(2,4)
% is 1 - (1 - 0.8)(1 - 0.9 * 0.8) = 0.944, each rule instance a choice of
% its own, and path(1,4) is 0.9 * 0.944; nothing leads from 5 to 4.
% A relation of ground facts, read by the magic rewrite as it is rather
% than demanded: the rules of path are copied under their demands, and
% the facts of edge are in the rewritten program once.
edge(1,2). edge(2,3). edge(3,4). edge(2,4). edge(5,6).
0.8::path(X,Y) :- edge(X,Y).
0.9::path(X,Z) :- edge(X,Y), path(Y,Z).
query(path(1,4)).
query(path(5,4)).
