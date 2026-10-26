%Expected outcome:
% vp_rule([saw, the, dog, with, the, telescope],pp) 0.63157895
% np_rule([the, dog, with, the, telescope],pp) 0.36842105
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% The grammar of terms_pcfg_list.pl, asked which reading of an ambiguous
% sentence it took: the prepositional phrase attaches to the verb phrase
% or to the noun phrase before it.
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

evidence(s([the, man, saw, the, dog, with, the, telescope], [])).
query(vp_rule([saw, the, dog, with, the, telescope], pp)).
query(np_rule([the, dog, with, the, telescope], pp)).
