%Expected outcome:
% observe([a, b, b, a, a]) 0.029564438
% observe([b, b]) 0.2235
% observe([]) 1
%
% The expected outcomes are ProbLog's answers (`-k ddnnf`), one query at a
% time.
% A hidden Markov model whose observations are a list: the walk down the
% list carries the time step and the state.
0.5::start(s1) ; 0.5::start(s2).
0.7::trans(T, s1, s1) ; 0.3::trans(T, s1, s2).
0.4::trans(T, s2, s1) ; 0.6::trans(T, s2, s2).
0.9::emit(T, s1, a) ; 0.1::emit(T, s1, b).
0.2::emit(T, s2, a) ; 0.8::emit(T, s2, b).

observe(Os) :- start(S), walk(Os, 0, S).
walk([], _, _).
walk([O|Os], T, S) :- emit(T, S, O), trans(T, S, S1), T1 is T + 1, walk(Os, T1, S1).

query(observe([a, b, b, a, a])).
query(observe([b, b])).
query(observe([])).
