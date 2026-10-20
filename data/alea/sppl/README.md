# SPPL's programs

`markov_switching.jl` is the hierarchical Markov switching model of SPPL
(Feras A. Saad, Martin C. Rinard, Vikash K. Mansinghka, "SPPL:
Probabilistic Programming with Fast Exact Symbolic Inference", PLDI 2021,
DOI 10.1145/3453483.3454078; Table 2, "Hierarchical HMM"), from
`examples/hierarchcial-markov-switching.ipynb` of github.com/probsys/sppl
at e537b0c (Apache License 2.0). The model is unchanged; its observations
are a fixed draw from it (the file's header says how), and its normal
emissions are observed at fixed-point precision, its Poisson ones as
categoricals over 0 to 31. The queries are the hierarchical variable
`separated` and the last state. `bench.py --suite bitblast` runs it as `bb_markov`, laddered
by the number of steps (SPPL's notebook has 100).

SPPL's other benchmarks are in the suite already (clickGraph, trueskill,
clinicalTrial and GPA, through the bit-blasting paper's Table 2) or are
discrete with unbounded support (student interviews, the Poisson ones),
which Alea.jl represents only truncated.

Reference: Alea.jl's answer on the discretized program; tddlog's answers
on the circuit are checked against it in every run.
