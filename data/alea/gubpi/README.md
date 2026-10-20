# The pedestrian

`pedestrian.jl` is the pedestrian random walk of Carol Mak, C.-H. Luke
Ong, Hugo Paquet, Dominik Wagner, "Densities of Almost Surely Terminating
Probabilistic Programs are Differentiable Almost Everywhere", ESOP 2021,
which GuBPI (Raven Beutner, C.-H. Luke Ong, Fabian Zaiser, "Guaranteed
Bounds for Posterior Inference in Universal Probabilistic Programming",
PLDI 2022) bounds. The walk is unrolled to a fixed number of steps and
conditioned on ending by then (the file's header says how); `bench.py
--suite bitblast` runs it as `bb_pedestrian`, laddered by the steps.

Reference: Alea.jl's answer on the discretized program; tddlog's answers
on the circuit are checked against it in every run.
