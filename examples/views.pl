% Views over a tuple-independent database (src/ground/inline.rs): orders
% with line items, customers who placed one, suppliers of the home nation.
% `ordered`, `local_order` and `shipped` are read once and `home_supplier`
% has one rule over facts, read twice: all are replaced by their bodies,
% and one body `shipped` passes to `second_only` meets its complement
% there. `big(1,a)`, `big(2,a)` and then `buyer(a)` are read once;
% `big(3,b)` is queried, and `buyer(b)`, which reads it, has one rule and
% is read once, so it goes too. `busy` is read
% negatively, `colour` is an annotated disjunction's set of values,
% `reach` is recursive, and `many` is read once by a rule of six other
% literals, where its twelve bodies would take six copies each: all of
% these stay.
0.4::order(1,a). 0.7::order(2,a). 0.5::order(3,b). 0.6::order(4,c).
0.3::item(1,1). 0.8::item(1,2). 0.5::item(2,1). 0.6::item(2,2). 0.2::item(3,1). 0.9::item(3,2). 0.4::item(3,3).
0.6::customer(a). 0.5::customer(b). 0.7::customer(c).
0.8::supplier(s1,home). 0.4::supplier(s2,home). 0.9::supplier(s3,away).
0.5::nation(home). 0.6::nation(away).
0.3::supplies(s1,1). 0.6::supplies(s2,1). 0.5::supplies(s1,3). 0.7::supplies(s3,2).
ordered(O) :- item(O,_).
big(O,C) :- order(O,C), ordered(O).
buyer(C) :- customer(C), big(_,C).
q :- buyer(_).
home_supplier(S) :- supplier(S,N), nation(N), N == home.
local_order(O) :- supplies(S,O), home_supplier(S).
busy(C) :- order(O,C), local_order(O).
quiet(C) :- customer(C), \+busy(C).
shipped(O) :- item(O,L), L =< 2.
second_only :- \+item(1,1), shipped(1).
0.3::colour(red); 0.5::colour(blue).
warm :- colour(red), customer(a).
cold :- colour(blue), customer(b).
0.5::edge(1,2). 0.6::edge(2,3). 0.7::edge(3,1).
reach(X) :- edge(1,X).
reach(Y) :- reach(X), edge(X,Y).
looped :- reach(1), customer(b).
0.5::f(1). 0.5::f(2). 0.5::f(3). 0.5::f(4). 0.5::f(5). 0.5::f(6).
0.1::g(1). 0.2::g(2). 0.3::g(3). 0.4::g(4). 0.5::g(5). 0.6::g(6).
0.7::g(7). 0.8::g(8). 0.9::g(9). 0.15::g(10). 0.25::g(11). 0.35::g(12).
many :- g(_).
wide :- f(1), f(2), f(3), f(4), f(5), f(6), many.
query(q).
query(big(3,b)).
query(quiet(a)).
query(quiet(b)).
query(quiet(c)).
query(second_only).
query(warm).
query(cold).
query(looped).
query(wide).
