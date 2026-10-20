% The fact edge(2,1) is used by one rule only, but a query names it: the
% aliasing pass must not absorb it into path(2,1). Found by the fuzzer.
0.4::edge(2,1).
path(X,Y) :- edge(X,Y).
query(edge(X,Y)).
query(path(2,1)).
query(\+ edge(2,1)).
