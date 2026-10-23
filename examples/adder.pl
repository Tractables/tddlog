% Two random integers in binary, as Dice compiles them: x and y take the
% values 0 to 7, bit I of each a probabilistic fact of its own (bit 0 the
% least significant), so a number costs three facts rather than one per
% value. The rules add them bit by bit as a ripple-carry adder does, and
% a comparison with a constant reads the sum's bits. The evidence says the
% sum is even; the queries ask for the sum's other bits and for the sum
% being 8 or more, which is its bit 3.
% Run with `tddlog --evidence-probability examples/adder.pl`; the guide
% works through it: https://tractables.github.io/tddlog/guide/#binary-integers
bit(0). bit(1). bit(2).
0.7::x(0). 0.3::x(1). 0.8::x(2).
0.4::y(0). 0.6::y(1). 0.2::y(2).

% exactly one of x(I) and y(I)
half(I) :- x(I), \+ y(I).
half(I) :- y(I), \+ x(I).
% the carry into bit I + 1
carry(J) :- x(I), y(I), J is I + 1.
carry(J) :- half(I), carry(I), J is I + 1.
% bit I of the sum
sum(I) :- half(I), \+ carry(I).
sum(I) :- bit(I), \+ half(I), carry(I).
sum(3) :- carry(3).
at_least_8 :- sum(3).

evidence(sum(0), false).
query(sum(1)).
query(sum(2)).
query(at_least_8).
query(x(0)).
