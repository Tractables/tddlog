% string_editing: ported to ProbLog from PluckArtifact.jl/src/table1/
% sequence_models/pluck/string_editing.jl of the artifact of Bowers, Lew,
% Tenenbaum, Solar-Lezama and Mansinghka, "Stochastic Lazy Knowledge
% Compilation for Inference in Discrete Probabilistic Programs" (PLDI
% 2025), Zenodo 10.5281/zenodo.15051140 (CC-BY-4.0): the Pluck program of
% the paper's Table 1, row String Editing, with the perturb its benchmark
% defines (perturb_defs, "the one we're using"). The query is the
% probability that perturb turns "edcc" into "eddde":
%   perturb Nil         = random_string 0.99
%   perturb (Cons c cs) = let delete = flip 0.01, insertion = random_string 0.99,
%                             rest = perturb cs
%                         in if delete then rest
%                            else insertion ++ Cons (if flip 0.01 then random_char else c) rest
%   random_string p     = if flip p then Nil else Cons random_char (random_string p)
% with random_char uniform over a to e.
%
% The perturbation is read against the target left to right: pert(K, I)
% holds where perturb of the input from position K is the target from
% position I to its end, and ins(K, I, J) where the string inserted before
% input position K (K = 4: after the input) is the target's symbols I to
% J - 1. A choice is keyed by the input position it perturbs and, within
% an inserted string, by the target position it writes, which gives the
% original's distribution where no perturbation reads one key twice. The
% input positions are perturbed once each, in order, and an inserted
% string writes each target position once, its end read at the position
% after its last symbol; an empty insertion, or a deleted input
% character, leaves the target position to the next input position, a
% key of its own. No perturbation then reads one key twice, it has the
% original's probability, and the choices at its keys fix it, so that two
% perturbations exclude each other.
in(0, e). in(1, d). in(2, c). in(3, c).
input(4).
out(0, e). out(1, d). out(2, d). out(3, d). out(4, e).
target(5).
place(K) :- input(N), between(0, N, K).

0.01::deleted(K) :- in(K, _).
0.01::replaced(K) :- in(K, _).
0.2::replacement(K, a); 0.2::replacement(K, b); 0.2::replacement(K, c);
  0.2::replacement(K, d); 0.2::replacement(K, e) :- in(K, _).
0.99::ends(K, I) :- place(K), target(M), between(0, M, I).
0.2::inserted(K, I, a); 0.2::inserted(K, I, b); 0.2::inserted(K, I, c);
  0.2::inserted(K, I, d); 0.2::inserted(K, I, e) :- place(K), out(I, _).

ins(K, I, I) :- ends(K, I).
ins(K, I, J) :- out(I, C), \+ends(K, I), inserted(K, I, C), I1 is I + 1, ins(K, I1, J).
written(K, C, C) :- in(K, C), \+replaced(K).
written(K, C, D) :- in(K, C), replaced(K), replacement(K, D).
pert(N, I) :- input(N), target(M), ins(N, I, M).
pert(K, I) :- in(K, _), deleted(K), K1 is K + 1, pert(K1, I).
pert(K, I) :- in(K, C), \+deleted(K), ins(K, I, J), out(J, D), written(K, C, D),
  J1 is J + 1, K1 is K + 1, pert(K1, J1).

perturbed :- pert(0, 0).
query(perturbed).
