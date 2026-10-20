# PSI's programs

`linear_regression.jl` is Bayesian linear regression as R2 (Aditya V.
Nori, Chung-Kil Hur, Sriram K. Rajamani, Selva Samuel, "R2: An Efficient
MCMC Sampler for Probabilistic Programs", AAAI 2014) and PSI (Timon Gehr,
Sasa Misailovic, Martin Vechev, "PSI: Exact Symbolic Inference for
Probabilistic Programs", CAV 2016) have it, `test/r2/linearRegression.psi`
of github.com/eth-sri/psi at 622f00e (Boost Software License 1.0), with
R2's data (`test/r2/data/LinearRegression/dataX1.csv`, `dataY1.csv`
there). The noise has standard deviation 1 rather than a gamma prior on
its precision, so that the posterior of the slope and intercept is the
conjugate normal one: with prior means m = (0, 5), prior covariance
S = diag(1, 1.8257) and design rows (x_k, 1), the posterior mean is
(S^-1 + X'X)^-1 (S^-1 m + X'y). `bench.py --suite bitblast` runs it as
`bb_linreg` with 10 observations, laddered by the bits after the binary
point.

PSI's other programs with continuous variables are in the suite already
(coinBias, clickGraph, clinicalTrial, trueskill, addFun, through the
bit-blasting paper's Table 2), are skipped by PSI itself for their size
(LearningGaussian, the BPM, conference, Hiv), or are discrete and answered
in milliseconds (burglarAlarm, grass, noisyOr, twoCoins, murderMystery,
evidence), below the 0.1 s the tables count.

Reference: Alea.jl's answer on the discretized program; tddlog's answers
on the circuit are checked against it in every run. Alea.jl's posterior
means approach the continuous one, (-0.77899, 2.62779): (-0.8293, 2.5806)
at 2 bits, (-0.7939, 2.6346) at 4, (-0.7888, 2.6299) at 5, (-0.7881,
2.6293) at 6.
