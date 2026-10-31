% A program that takes its size, a probability and its query from the
% command line: the values after the file take the places $1, $2 and $3,
% each read as a term. A chain of $1 links, each up with probability $2,
% runs from node 0 to node $1, and the query asks whether node $3 is
% reached from node 0.
% Run with `tddlog examples/chain.pl 10 0.9 4`, which prints reach(4) =
% 0.9^4 = 0.656100; `tddlog --program examples/chain.pl 10 0.9 4` prints
% the program with the values in place.
$2::link(I) :- between(1, $1, I).
reach(0).
reach(I) :- between(1, $1, I), J is I - 1, reach(J), link(I).
query(reach($3)).
