%Expected outcome:
% expression([i1,i2,i3],2) 0.375
% expression([i1,i2,i3],3) 0.25
% expression([i1,i2,i3],4) 0.25
%
% The expected outcomes are computed by hand. Each of the three images is
% read as one of two symbols with probability 0.5: 1 or 2, + or *, 1 or 2.
% The value 2 is read as 1+1, 1*2 or 2*1, 3 of 8 readings: 0.375; 3 as 1+2
% or 2+1: 0.25; 4 as 2+2 or 2*2: 0.25. A semi-naive round of parse/2 meets
% the demands parse([N1, *, N2|T], R) only through N3 is N1 * N2, and the
% label R is the same in each: the round walks the demands in the rule's
% own order rather than each demand for each new tuple of parse/2.
0.5::num(i1,1) ; 0.5::num(i1,2).
0.5::op(i2,+) ; 0.5::op(i2,*).
0.5::num(i3,1) ; 0.5::num(i3,2).
detect_all([N],[N2]) :- num(N,N2).
detect_all([N,O|T],[N2,O2|T2]) :- num(N,N2), op(O,O2), detect_all(T,T2).
expression(Images,R) :- detect_all(Images,S), parse(S,R).
parse([N],R) :- N =:= R.
parse([N1,+|T],R) :- parse_free(T,R2), N1 + R2 =:= R.
parse([N1,*,N2|T],R) :- N3 is N1 * N2, parse([N3|T],R).
parse_free([N],R) :- R is N.
parse_free([N1,+|T],R) :- parse_free(T,R2), R is N1 + R2.
parse_free([N1,*,N2|T],R) :- N3 is N1 * N2, parse_free([N3|T],R).
query(expression([i1,i2,i3],2)).
query(expression([i1,i2,i3],3)).
query(expression([i1,i2,i3],4)).
