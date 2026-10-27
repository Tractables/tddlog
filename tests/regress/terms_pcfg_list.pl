%Expected outcome:
% sentence([the, man, saw, the, dog, with, the, telescope]) 0.2432
% sentence([the, dog, saw, the, man]) 0.448
% sentence([the, dog, with, the, telescope, saw, the, man, with, the, dog]) 0.04864
% sentence([the, dog, saw]) 0
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% A probabilistic context-free grammar over a list of words, read with
% difference lists. Each nonterminal chooses its expansion once per place
% in the sentence: an annotated disjunction over the rest of the words.
0.8::np_rule(L, base) ; 0.2::np_rule(L, pp).
0.7::vp_rule(L, base) ; 0.3::vp_rule(L, pp).

s(L0, L) :- np(L0, L1), vp(L1, L).
np(L0, L) :- np_rule(L0, base), det(L0, L1), noun(L1, L).
np(L0, L) :- np_rule(L0, pp), det(L0, L1), noun(L1, L2), pp(L2, L).
vp(L0, L) :- vp_rule(L0, base), verb(L0, L1), np(L1, L).
vp(L0, L) :- vp_rule(L0, pp), verb(L0, L1), np(L1, L2), pp(L2, L).
pp(L0, L) :- prep(L0, L1), np(L1, L).

det([the|L], L).
noun([dog|L], L).
noun([man|L], L).
noun([telescope|L], L).
verb([saw|L], L).
prep([with|L], L).

sentence(S) :- s(S, []).

query(sentence([the, man, saw, the, dog, with, the, telescope])).
query(sentence([the, dog, saw, the, man])).
query(sentence([the, dog, with, the, telescope, saw, the, man, with, the, dog])).
query(sentence([the, dog, saw])).
