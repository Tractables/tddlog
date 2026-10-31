%Expected outcome:
% parts(f(a,b),[f, a, b]) 0.3
% parts(g,[g]) 0.3
% parts(7,[7]) 0.3
% built(h(1,c)) 0.6
% built(k) 0.6
% built(2) 0.6
% same(f(a,b)) 0.3
%
% =../2 both ways, on ground terms: the list of a term's name and
% arguments, an atom's or a number's the list of itself; and the term of
% such a list. Computed by hand: each answer holds where its fact does.
0.3::a.
0.6::b.
t(f(a,b)). t(g). t(7).
l([h,1,c]). l([k]). l([2]).
parts(T, L) :- t(T), a, T =.. L.
built(T) :- l(L), b, T =.. L.
same(T) :- t(T), T =.. [F, X|R], a, T2 =.. [F, X|R], T2 == T.
query(parts(_,_)).
query(built(_)).
query(same(_)).
