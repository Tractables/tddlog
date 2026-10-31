%Expected outcome:
% urns(1) 0.4580152672
% urns(2) 0.2977099237
% urns(3) 0.2442748092
% A findall whose goal shares a variable with its clause, in a clause that
% calls select_uniform/4 as the clause that binds that variable does. The
% demand of both calls of select_uniform/4 closes a loop through the
% findall, which the program does not have. By hand: with n urns, two
% draws pick the same urn with probability 1/n, so both are green with
% probability 0.3/n + 0.09 (1 - 1/n); each n has prior 1/3.
:- use_module(library(lists)).

urns(N) :- findall(K, between(1, 3, K), Ks), select_uniform(urns, Ks, N, _).
0.7::colour(U, blue) ; 0.3::colour(U, green).
draw(D, C) :-
    urns(N), findall(V, between(1, N, V), Us), select_uniform(D, Us, U, _),
    colour(U, C).

evidence(draw(0, green)).
evidence(draw(1, green)).
query(urns(_)).
