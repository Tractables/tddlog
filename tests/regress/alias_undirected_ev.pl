% alias_undirected.pl with evidence on an alias, conn(2,1), which sets
% edge(1,2): 1 reaches 3 with 1 - (1 - 0.4)(1 - 0.5) = 0.7, and 4 with
% 0.7 * 0.7 = 0.49; ProbLog 2.3.0 agrees.
0.6::edge(1,2). 0.5::edge(2,3). 0.4::edge(1,3). 0.7::edge(3,4).
conn(X,Y) :- edge(X,Y).
conn(X,Y) :- edge(Y,X).
path(X,Y) :- conn(X,Y).
path(X,Y) :- conn(X,Z), path(Z,Y).
evidence(conn(2,1), true).
query(path(1,4)).
query(path(4,1)).
query(conn(3,2)).
