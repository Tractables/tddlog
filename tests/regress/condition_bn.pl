% A network with deterministic tables whose evidence fixes most of it, as in
% Ace's blockmap and mastermind networks. The evidence on w rules out x(1)
% and x(2), so x(3) holds; y then holds, and z's evidence fixes u.
0.2::x(1); 0.5::x(2); 0.3::x(3).
w(true) :- x(1).
w(true) :- x(2).
w(false) :- x(3).
y(true) :- x(3).
y(false) :- x(1).
y(false) :- x(2).
0.6::u(true); 0.4::u(false).
z(true) :- y(true), u(true).
z(false) :- y(true), u(false).
z(false) :- y(false).
0.7::v(true); 0.3::v(false) :- z(true).
0.1::v(true); 0.9::v(false) :- z(false).
0.25::t.
s :- u(false), t.
s :- x(1).
evidence(w(false)).
evidence(z(false)).
query(x(3)).
query(u(true)).
query(v(true)).
query(s).
query(t).
