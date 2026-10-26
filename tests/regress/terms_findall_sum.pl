%Expected outcome:
% total(12) 1
% most(5) 1
% children(ann,2) 1
% children(bob,1) 1
% children(cid,0) 1
% children(dan,0) 1
% big(b) 0.6
% big(c) 0.5
% happy(ann) 0.4
% costs([a-3, b-4, c-5]) 1
% grandchildren(ann,[dan]) 1
% grandchildren(bob,[]) 1
% grandchildren(cid,[]) 1
% grandchildren(dan,[]) 1
% spend(7) 0.5
% spend(8) 0.6
% spend(9) 0.3
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% findall/3 over a deterministic goal, read by sum_list/2, length/2 and
% max_list/2, and its result used under probabilistic facts.
:- use_module(library(lists)).
cost(a, 3). cost(b, 4). cost(c, 5).
person(ann). person(bob). person(cid). person(dan).
parent(ann, bob). parent(ann, cid). parent(bob, dan).
0.3::buy(a). 0.6::buy(b). 0.5::buy(c).

total(S) :- findall(C, cost(_, C), L), sum_list(L, S).
most(M) :- findall(C, cost(_, C), L), max_list(L, M).
children(P, N) :- person(P), findall(C, parent(P, C), L), length(L, N).
big(I) :- buy(I), cost(I, C), total(T), C * 3 >= T.
0.4::happy(P) :- children(P, N), N >= 2.
costs(L) :- findall(I-C, cost(I, C), L).
grandchildren(P, L) :- person(P), findall(G, (parent(P, C), parent(C, G)), L).
spend(S) :- buy(I), findall(C, (cost(J, C), J \== I), L), sum_list(L, S).

query(total(_)).
query(most(_)).
query(children(_, _)).
query(big(_)).
query(happy(_)).
query(costs(_)).
query(grandchildren(_, _)).
query(spend(_)).
