%Expected outcome:
% similar(ale, stout) 0.5
% similar(ale, gin) 0.25
% similar(ale, soda) 0.2
%
% ProbLog's tutorial (taxonomy): the paths between two nodes of a tree,
% walked both ways along its edges, and a similarity of one over the
% path's length. `\+ member(B, L)` keeps a node off the path that goes on
% from it. Under the magic rewrite the demand of member/2 reads d/3's
% answers, so the negation sits inside d/3's recursion, is read as
% unknown, and the paths grew without end until the grounding ran out of
% memory. The program is resolved top-down, as ProbLog grounds it. The
% expected outcomes are ProbLog's answers.
e(drink,alcoholic).
e(drink,nonalcoholic).
e(alcoholic,beer).
e(alcoholic,wine).
e(alcoholic,spirit).
e(nonalcoholic,soda).
e(nonalcoholic,coffee).
e(beer,ale).
e(beer,stout).
e(beer,lager).
e(wine,redwine).
e(wine,whitewine).
e(spirit,gin).
e(spirit,whiskey).

member(X,[X|_]).
member(X,[_|T]) :- member(X,T).

d(B,B,[B]).
d(B,E,[B|L]) :- e(B,M), d(M,E,L),\+member(B,L).
d(B,E,[M|L]) :- e(M,B), d(M,E,L),\+member(B,L).

P::similar(B,E) :- d(B,E,L), length(L,LL), (LL=1, P is 1; LL>1, P is 1/(LL-1)).

query(similar(ale,stout)).
query(similar(ale,gin)).
query(similar(ale,soda)).
