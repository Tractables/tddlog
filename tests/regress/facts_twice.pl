%Expected outcome:
% a 0.58
% b 1
% c 1
% d 1
% f 0
% e(1,2) 0.8
% path(1,3) 0.4
% path(3,1) 0
%
% The expected outcomes are computed by hand. Every predicate here but
% path is given by ground facts alone, which the grounder makes its atoms
% without a rule each. An atom given by two facts is true where either is:
% a is 1 - 0.7 * 0.6 and e(1,2) is 1 - 0.5 * 0.4. A fact without a
% probability makes its atom certain, before or after a probabilistic one
% (b, c); d and f are facts of probability one and zero. path(1,3) needs
% e(1,2) and e(2,3), 0.8 * 0.5, and nothing leaves 3.
0.3::a. 0.4::a.
0.3::b. b.
c. 0.2::c.
1.0::d.
0.0::f.
0.5::e(1,2). 0.5::e(2,3). 0.6::e(1,2).
path(X,Y) :- e(X,Y).
path(X,Y) :- e(X,Z), path(Z,Y).
query(a). query(b). query(c). query(d). query(f).
query(e(1,2)). query(path(1,3)). query(path(3,1)).
