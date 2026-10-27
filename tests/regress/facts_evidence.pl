%Expected outcome:
% a 0.408450704225352
% b 0.704225352112676
% k 0
%
% The expected outcomes are computed by hand. a, k and b are given by
% their facts alone, a and k each by two: a has probability
% 1 - 0.7 * 0.6 = 0.58 and b 0.5, and k, observed false, leaves c the
% rules that read a and b, which hold with probability
% 0.58 * 0.5 + 0.42 = 0.71. Given c, a is 0.29 / 0.71 and b is
% (0.29 + 0.42 * 0.5) / 0.71.
0.3::a. 0.4::a.
0.5::b.
0.2::k. 0.5::k.
c :- a, b.
c :- \+a.
c :- k.
evidence(c).
evidence(k, false).
query(a). query(b). query(k).
