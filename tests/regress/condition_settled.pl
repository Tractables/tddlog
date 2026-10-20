% A network with deterministic tables but for its roots, whose evidence
% settles every atom by unit propagation alone, the exclusive sets unread,
% as in the UAI 2022 blockmaps. w(false) holds only through x(3), which
% leaves x(1), x(2), w(true) and y(false) false and y(true) true; z(false)
% then holds only through u(false), and z(true) is false. P(e) = 0.3 * 0.4
% = 0.12, and given the evidence x(3) and u(false) are certain.
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
evidence(w(false)).
evidence(z(false)).
query(x(3)).
query(u(false)).
query(y(false)).
query(z(true)).
