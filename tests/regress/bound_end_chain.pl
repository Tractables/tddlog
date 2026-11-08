%Expected outcome:
% s(12) 0.999400164973
%
% The diamond network of the Dice paper's network verification benchmark,
% 12 diamonds, as a recursion over its steps: each passes with probability
% 0.5 + 0.5 * 0.9999 = 0.99995, so s(12) is 0.99995^12, by hand. step
% and s rise a level a step, and the grounder grounds them level by level;
% s's chain from s(12) runs down the same levels, and the bound-end
% rewrite, which would ground it from there, is not tried. Twelve diamonds
% keep every layout of the suite small: under `program` and
% `program:fanin`, forty diamonds take more than 16 GB.

diamonds(12).
step(0) :- diamonds(N), N > 0.
step(T1) :- step(T), diamonds(N), T1 is T + 1, T1 < N.
0.5::route(T) :- step(T).
0.0001::drop(T) :- step(T).
s(0).
s(T1) :- step(T), T1 is T + 1, s(T), route(T).
s(T1) :- step(T), T1 is T + 1, s(T), \+route(T), \+drop(T).
query(s(12)).
