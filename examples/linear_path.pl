% A path relation written right-recursively and queried from bound
% sources: grounded from those sources (src/ground/linear.rs), it needs
% the nodes reachable from 1 and 2 rather than path(X,Y) for every X.
% A step may not pass through a closed node.
% Run with `tddlog examples/linear_path.pl`; `tddlog --ground examples/linear_path.pl`
% prints the program the grounder writes.
0.6::edge(1,2). 0.5::edge(1,3). 0.4::edge(2,4). 0.7::edge(3,4).
0.3::edge(4,1). 0.5::edge(4,5). 0.6::edge(5,6). 0.2::edge(6,3).
0.8::edge(2,7). 0.4::edge(7,8). 0.5::edge(8,5). 0.3::edge(6,8).
0.9::edge(3,7).
0.2::closed(4). 0.1::closed(7).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), \+closed(Z), path(Z,Y).
evidence(path(1,7), true).
query(path(1,1)).
query(path(1,4)).
query(path(1,6)).
query(path(1,8)).
query(\+path(1,5)).
query(path(2,6)).
