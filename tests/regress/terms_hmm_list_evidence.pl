%Expected outcome:
% state([a, b, b],2,s1) 0.094708277
% state([a, b, b],2,s2) 0.90529172
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% The model of terms_hmm_list.pl, asked for the state at the last step given
% the observations.
0.5::start(s1) ; 0.5::start(s2).
0.7::trans(T, s1, s1) ; 0.3::trans(T, s1, s2).
0.4::trans(T, s2, s1) ; 0.6::trans(T, s2, s2).
0.9::emit(T, s1, a) ; 0.1::emit(T, s1, b).
0.2::emit(T, s2, a) ; 0.8::emit(T, s2, b).

observe(Os) :- start(S), walk(Os, 0, S).
walk([], _, _).
walk([O|Os], T, S) :- emit(T, S, O), trans(T, S, S1), T1 is T + 1, walk(Os, T1, S1).
state(Os, T, S) :- start(S0), at(Os, 0, S0, T, S).
at(_, T, S, T, S).
at([_|Os], T0, S0, T, S) :- T0 < T, trans(T0, S0, S1), T1 is T0 + 1, at(Os, T1, S1, T, S).

evidence(observe([a, b, b])).
query(state([a, b, b], 2, s1)).
query(state([a, b, b], 2, s2)).
