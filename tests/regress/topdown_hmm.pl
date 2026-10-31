%Expected outcome:
% obs([big]) 0.59
% obs([small]) 0.41
% obs([big, big]) 0.3005
% obs([big, small]) 0.2895
% obs([small, big]) 0.1565
% obs([small, small]) 0.2535
% obs([big, big, big]) 0.151115
% obs([big, big, small]) 0.149385
% obs([big, small, big]) 0.104235
% obs([big, small, small]) 0.185265
% obs([small, big, big]) 0.076235
% obs([small, big, small]) 0.080265
% obs([small, small, big]) 0.085515
% obs([small, small, small]) 0.167985
%
% A hidden Markov model whose transitions and emissions take the time step
% from their callers, so that no demand makes their annotated disjunctions
% safe, and a query rule that asks for the sequences of each length with
% length/2. It is resolved top-down. The expected outcomes are the forward
% algorithm's, worked out by hand.
0.7::start(hot) ; 0.3::start(cold).
0.6::trans(T, hot, hot) ; 0.4::trans(T, hot, cold).
0.3::trans(T, cold, hot) ; 0.7::trans(T, cold, cold).
0.8::emit(T, hot, big) ; 0.2::emit(T, hot, small).
0.1::emit(T, cold, big) ; 0.9::emit(T, cold, small).

obs([O|Os]) :- start(S), emit(0, S, O), go(S, Os, 0).
go(_, [], _).
go(S, [O|Os], T) :- trans(T, S, S2), T1 is T + 1, emit(T1, S2, O), go(S2, Os, T1).

query(obs(L)) :- between(1, 3, N), length(L, N).
