%Expected outcome:
% b(2,0) 0.58
% b(2,1) 0.58
%
% A bit flipped at each hop: b(K, B) is b(K - 1, B) exclusive-or flip(K, B).
% The level is not the last argument, so the program is grounded under the
% magic rewrite. The demand of `\+ b(P, B)` read only `P is K - 1`, which
% the schedule placed before `hop(K)`, and asked b at every level below the
% one queried, without end, until the grounding ran out of memory. b(1, B)
% holds where flip(1, B) does not, 0.7, and b(2, B) is 0.7 * 0.7 + 0.3 *
% 0.3.

hop(1). hop(2).
pos(0). pos(1).
0.3::flip(K, B) :- hop(K), pos(B).
b(0, B) :- pos(B).
b(K, B) :- hop(K), P is K - 1, b(P, B), \+ flip(K, B).
b(K, B) :- hop(K), pos(B), P is K - 1, \+ b(P, B), flip(K, B).
query(b(2, 0)).
query(b(2, 1)).
