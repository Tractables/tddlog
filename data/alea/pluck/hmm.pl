% hmm: ported to ProbLog from PluckArtifact.jl/src/table1/sequence_models/
% pluck/hmm.jl of the artifact of Bowers, Lew, Tenenbaum, Solar-Lezama and
% Mansinghka, "Stochastic Lazy Knowledge Compilation for Inference in
% Discrete Probabilistic Programs" (PLDI 2025), Zenodo
% 10.5281/zenodo.15051140 (CC-BY-4.0): the Pluck program of the paper's
% Table 1, row HMM. The query is the probability that the first 50
% emissions are all true, of the chain
%   hmm z0 = let z1 = flip (if z0 then 0.7 else 0.1)
%            in Cons (flip (if z1 then 0.3 else 0.6)) (hmm z1)
% from z0 false. z(T) is the state after step T, x(T) its emission, and
% seen(T) that the first T emissions are true.
steps(50).
step(T) :- steps(N), between(1, N, T).

0.7::stays(T) :- step(T).
0.1::rises(T) :- step(T).
0.3::shows_on(T) :- step(T).
0.6::shows_off(T) :- step(T).

z(T) :- step(T), P is T - 1, z(P), stays(T).
z(T) :- step(T), P is T - 1, \+z(P), rises(T).
x(T) :- z(T), shows_on(T).
x(T) :- step(T), \+z(T), shows_off(T).
seen(0).
seen(T) :- step(T), P is T - 1, seen(P), x(T).

emitted :- steps(N), seen(N).
query(emitted).
