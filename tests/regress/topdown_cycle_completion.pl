%Expected outcome:
% q([1]) 1
% q([2]) 0.5
% q([3]) 0.3
% r([1]) 1
% r([2]) 0.5
% r([3]) 0.3
%
% Three calls in one cycle of the top-down route, computed by hand: a/1,
% b/1 and c/1 hold for the same values, 1 for certain, 2 where d(1, 2)
% holds and 3 where d(2, 3) holds too. wrap/2 answers with a variable
% that only its caller binds, so no demand makes the program safe and it
% is resolved top-down. The first pass makes the tables of a, b and c in
% that order and leaves b and c open in a's cycle. The second pass calls
% c first, from a's first clause, and c calls b again: b read only c,
% which was being resolved, gained no answer and was taken for the
% first call of a cycle of its own. b and c were completed before c read
% a's new answers, and b kept the one answer of the first pass. A table
% found in a cycle now stays in it until the cycle's first call
% completes.
s(1).
0.5::d(1, 2).
0.6::d(2, 3).
a(X) :- a(_), c(X).
a(X) :- b(X).
b(X) :- c(X).
c(X) :- b(X).
c(X) :- a(Y), d(Y, X).
c(X) :- s(X).
wrap(X, [X]).
q(L) :- wrap(X, L), a(X).
r(L) :- wrap(X, L), b(X).
query(q([_])).
query(r([_])).
