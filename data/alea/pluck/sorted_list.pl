% sorted_list: ported to ProbLog from PluckArtifact.jl/src/table1/
% sequence_models/pluck/sorted_list.jl of the artifact of Bowers, Lew,
% Tenenbaum, Solar-Lezama and Mansinghka, "Stochastic Lazy Knowledge
% Compilation for Inference in Discrete Probabilistic Programs" (PLDI
% 2025), Zenodo 10.5281/zenodo.15051140 (CC-BY-4.0): the Pluck program of
% the paper's Table 1, row Sorted List. The query is the probability that
%   generate min = if flip 0.5 then Nil
%                  else let m = min + randnat in Cons m (generate m)
% with randnat geometric (0 by a flip 0.5, otherwise one more than
% randnat), generates [0, 3, 7, 12, 13, 15, 16, 20] from 0: 2^-37.
%
% The list is read against the target left to right: list(I, Min) holds
% where generate Min is the target from element I on, and step(I, K, D)
% where element I's randnat, at K, goes on to D. A choice is keyed by the
% element it draws and, in randnat, by its count, which gives the
% original's distribution where no generation reads one key twice. The
% elements are drawn once each, in order, and a randnat's count grows by
% one at each of its flips, so no generation reads one key twice, it has
% the original's probability, and the choices at its keys fix it, so that
% two generations exclude each other.
at(0, 0). at(1, 3). at(2, 7). at(3, 12). at(4, 13). at(5, 15). at(6, 16). at(7, 20).
elements(8).

0.5::ends(I) :- elements(N), between(0, N, I).
0.5::stops(I, K) :- at(I, V), between(0, V, K).

step(I, D, D) :- stops(I, D).
step(I, K, D) :- K < D, \+stops(I, K), K1 is K + 1, step(I, K1, D).
list(N, _) :- elements(N), ends(N).
list(I, Min) :- at(I, V), \+ends(I), V >= Min, D is V - Min, step(I, 0, D),
  I1 is I + 1, list(I1, V).

generated :- list(0, 0).
query(generated).
