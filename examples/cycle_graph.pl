% Reachability from node 1 over a graph with cycles. The paths are one
% recursive component whose rules each read one of its atoms, positively,
% which every strategy compiles written without recursion by vertex
% elimination (src/ground/eliminate.rs): the atoms path(1,Y) are eliminated
% one at a time, as in Floyd and Warshall's algorithm, and each is defined
% from the atoms eliminated after it. Where several paths join and go on,
% the disjunction becomes an atom of its own, named for what it means:
% '$via'(path(1,X),path(1,Y),K) holds where a path leads from X to Y whose
% inner nodes are all among the first K eliminated, and
% '$via'(true,path(1,Y),K) where such a path leads from 1 to Y.
% ProbLog 2.3.0's answers are in tests/it/examples.rs.
% Run with `tddlog examples/cycle_graph.pl`; `tddlog --ground --vtree closed
% examples/cycle_graph.pl` prints the program written without recursion,
% the order of the elimination first.
% The guide works through it:
% https://tractables.github.io/tddlog/guide/#reachability
0.6::edge(1,2). 0.5::edge(2,3). 0.7::edge(3,1). 0.4::edge(3,4). 0.8::edge(4,5).
0.3::edge(5,3). 0.6::edge(5,6). 0.5::edge(6,4). 0.4::edge(2,6). 0.3::edge(6,1).
0.5::edge(4,7). 0.6::edge(7,2). 0.4::edge(7,5). 0.7::edge(1,5).
path(X,Y) :- edge(X,Y).
path(X,Y) :- edge(X,Z), path(Z,Y).
query(path(1,2)).
query(path(1,3)).
query(path(1,4)).
query(path(1,5)).
query(path(1,6)).
query(path(1,7)).
