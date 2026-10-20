% An undirected graph written as two directed edges per link: conn(a,b) and
% conn(b,a) both read edge(a,b) alone, and are read as one fact. By hand:
% 1 reaches 3 directly or through 2, 1 - (1 - 0.4)(1 - 0.6 * 0.5) = 0.58,
% and 4 through 3, 0.58 * 0.7 = 0.406, either way round; ProbLog 2.3.0
% agrees. A query names conn(3,2), which keeps a rule.
0.6::edge(1,2). 0.5::edge(2,3). 0.4::edge(1,3). 0.7::edge(3,4).
conn(X,Y) :- edge(X,Y).
conn(X,Y) :- edge(Y,X).
path(X,Y) :- conn(X,Y).
path(X,Y) :- conn(X,Z), path(Z,Y).
query(path(1,4)).
query(path(4,1)).
query(conn(3,2)).
