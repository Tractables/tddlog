%Expected outcome:
% roll([1, 2, 3]) 0.11111111
% roll([1, 2]) 0.22222222
% roll([1, 3, 2]) 0.11111111
% roll([1, 3]) 0.22222222
% roll([1]) 0.33333333
%
% ProbLog's tutorial (dice, the seventh program): a die is rolled until a
% value comes up again, and the values seen are kept in a list. The walk's
% recursion is demanded with the list one value longer at each step, and
% only `\+ member(Now, Seen)`, a negation of a predicate below it, ends it.
% The magic rewrite cannot read that negation in the demand of the walk, so
% the grounding demanded lists without end and ran out of memory; the
% program is resolved top-down, where the negated call is complete before
% it is read and the walk stops where it holds in every world. The
% expected outcomes are ProbLog's answers.
1/3::dice(1,D); 1/3::dice(2,D); 1/3::dice(3,D) :- die(D).
die(X) :- between(1,3,X).

roll(L) :-
   next(1,[1],L).

next(N,Seen,Rev) :-
   dice(Now,N),
   member(Now,Seen),
   reverse(Seen,[],Rev).
next(N,Seen,List) :-
   dice(Now,N),
   \+ member(Now,Seen),
   next(Now,[Now|Seen],List).

member(X,[X|_]).
member(X,[_|Z]) :-
   member(X,Z).

reverse([],L,L).
reverse([H|T],A,L) :-
   reverse(T,[H|A],L).

query(roll(_)).
