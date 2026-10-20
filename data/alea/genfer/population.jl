# Population ecology (Winner and Sheldon, "Probabilistic Inference with
# Generating Functions for Poisson Latent Variable Models", NeurIPS 2016),
# as Genfer has it (Zaiser, Murawski and Ong, "Exact Bayesian Inference on
# Discrete Models via Probability Generating Functions", NeurIPS 2023,
# benchmarks/neurips2023/approx/population/population.sgcl of
# github.com/fzaiser/genfer at d132c76, MIT License): an initial population
# Poisson(51.4); each season each animal survives with probability 0.2636
# (a binomial thinning), new animals arrive (Poisson, rates 232.6, 420.8,
# 300.8, 85.6), and the number counted is a binomial thinning of the
# population with probability 0.2 (observed 45, 98, 73, 38). The query
# is the expectation of the last population. `steps` seasons are run, the
# four seasons' rates and counts repeated in turn (Genfer's program is
# `steps = 4`).
#
# Bit blasting: the population is an unsigned integer of 10 bits; a
# Poisson is a categorical over 0 to 1023, the distribution truncated
# there; a binomial thinning of a random n with probability p is the sum,
# over the bits j of n that are set, of a categorical Binomial(2^j, p).
# A population past 1023 (probability below 1e-60 here) wraps around:
# the program runs with `ignore_errors`.
using Alea, Distributions

const QUERY = :expectation
const IGNORE_ERRORS = true
const steps = 4
const W = 10
const N = DistUInt{W}
const arrivals = [232.6, 420.8, 300.8, 85.6]
const counts = [45, 98, 73, 38]

truncated_pmf(d, k) = (p = [pdf(d, i) for i in 0:k]; p ./ sum(p))
poisson(rate) = discrete(N, truncated_pmf(Poisson(rate), 2^W - 1))

function thin(n, p)
    total = N(0)
    for j in 0:W-1
        b = discrete(N, truncated_pmf(Binomial(2^j, p), 2^j))
        total = total + ifelse(n.bits[W - j], b, N(0))
    end
    total
end

model() = @alea begin
    population = discrete(N, truncated_pmf(Poisson(51.4), 255))
    for k in 1:steps
        population = thin(population, 0.2636) + poisson(arrivals[mod1(k, 4)])
        observe(prob_equals(thin(population, 0.2), N(counts[mod1(k, 4)])))
    end
    population
end
