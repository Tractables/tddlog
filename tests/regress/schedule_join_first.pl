%Expected outcome:
% hit(t1,1) 0.5
% hit(t1,2) 0.3
% hit(t2,3) 0.4
% both 0.15
%
% The expected outcomes are computed by hand: t1 reads 1, which e links
% to 2 and to itself, and s(1, 2) and s(2, 1) hold with 0.5 and 0.3; t2
% reads 3, linked to 4 by s(3, 4), 0.4. hit's atom s(B, C) shares no
% variable with l(T, A) before it, so the grounder reads e(A, C) first,
% which binds C; both hits of t1 are independent, 0.5 * 0.3.
0.5::s(1, 2).
0.4::s(3, 4).
0.3::s(2, 1).
l(t1, 1).
l(t2, 3).
e(1, 1).
e(1, 2).
e(3, 3).
e(3, 4).
hit(T, B) :- l(T, A), s(B, C), e(A, C).
both :- hit(t1, 1), hit(t1, 2).
query(hit(t1, 1)).
query(hit(t1, 2)).
query(hit(t2, 3)).
query(both).
