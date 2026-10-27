%Expected outcome:
% reach(a,c,[]) 0.42
% reach(c,a,[]) 0
%
% The expected outcomes are computed by hand: a path from a to c takes
% both edges, 0.6 * 0.7, and none leads back. The third argument of link
% is a variable its clause mentions nowhere else, so it does not decide
% which atoms of link hold, and the grounder drops it; reach keeps the
% list of the nodes it passes, which its second clause reads.
0.6::edge(a, b).
0.7::edge(b, c).
link(X, Y, _Via) :- edge(X, Y).
reach(X, Y, D) :- link(X, Y, D).
reach(X, Z, D) :- link(X, Y, [Y|D]), reach(Y, Z, [Y|D]).
query(reach(a, c, [])).
query(reach(c, a, [])).
