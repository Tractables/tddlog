# Genfer's programs

`population.jl` is the population model of Kevin Winner, Daniel Sheldon,
"Probabilistic Inference with Generating Functions for Poisson Latent
Variable Models", NeurIPS 2016, as Genfer (Fabian Zaiser, Andrzej S.
Murawski, C.-H. Luke Ong, "Exact Bayesian Inference on Discrete Models
via Probability Generating Functions: A Probabilistic Programming
Approach", NeurIPS 2023) has it in
`benchmarks/neurips2023/approx/population/population.sgcl` of
github.com/fzaiser/genfer at d132c76 (MIT License). Its counts have
unbounded support, which the bit-blasted program truncates at 1023 (the
file's header says how the binomial thinnings are bit blasted). `bench.py
--suite bitblast` runs it as `bb_population`, laddered by the number of
seasons, Genfer's four repeated in turn.

Genfer's other programs are PSI's (in the suite through the bit-blasting
paper's Table 2, or discrete and answered in milliseconds) or use
geometric rates in Poisson likelihoods (hmm, mixture, switchpoint), which
are not written here.

Reference: the posterior mean of the final population by a forward
filter over 0 to 1023 on the program as written (the truncated Poissons
normalized, binomial thinnings, the observations' binomial likelihood at
the counts): 241.919232 after one season (Alea.jl's answer, to 1e-15),
485.650048 after 2, 416.220542 after 3, 194.275228 after 4, 492.032394
after 6, 194.561421 after 8, 194.561994 after 12 and 16. Alea.jl runs
out of memory from two seasons on (16 GB); tddlog's answers on the
circuit are checked against these.
