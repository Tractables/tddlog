%Expected outcome:
% state([b,a],[]) 0.28
% state([b,b,a,a],[]) 0.0336
% state([a,b],[]) 0
%
% The expected outcomes are computed by hand. A word is generated left
% to right, kept reversed in the first argument, with the symbols still
% to expand in the second. "ab" has one derivation, S -> O M (0.4), O ->
% a, M -> b (0.7): 0.28. "aabb" has one, S -> O M (0.4), O -> a, M -> S C
% (0.3), S -> O M (0.4), O -> a, M -> b (0.7), C -> b: 0.0336. No
% derivation ends in a. Each semi-naive round grounds the recursive rules
% from their recursive literal, which comes after the demand that the
% magic rewrite puts first.
state([],[s]).
0.4::state(W,[o,m|S]) ; 0.6::state(W,[s,s|S]) :- state(W,[s|S]).
state([a|W],S) :- state(W,[o|S]).
0.7::state([b|W],S) ; 0.3::state(W,[s,c|S]) :- state(W,[m|S]).
state([b|W],S) :- state(W,[c|S]).
query(state([b,a],[])).
query(state([b,b,a,a],[])).
query(state([a,b],[])).
