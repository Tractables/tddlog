%Expected outcome:
% o(3) 0.405
%
% A chain over a time index whose steps are bounded by a relation read
% before the recursive call. Grounded from the bound end (`linear`), the
% magic rewrite asks o(T+1) of every o(T) it asks, step(T) coming after
% the call, and the demand grew without end until the demands were held
% to the grounding's bound. o(3) holds where ((f(0), f(1)) or g(0)) and
% f(2), or f(0) and g(1): 0.275 + 0.2 - 0.07, which ProbLog 2.3.0 gives.

0.5::f(T) :- step(T).
0.4::g(T) :- step(T).
step(0). step(1). step(2).
o(0).
o(T1) :- step(T), T1 is T + 1, o(T), f(T).
o(T2) :- step(T), T2 is T + 2, o(T), g(T).
query(o(3)).
