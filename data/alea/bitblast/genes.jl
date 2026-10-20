# Gene expression (bit-blasting paper, Garg, Holtzen, Van den Broeck,
# Millstein, PLDI 2024, Section 2.1, Figures 1 and 2): T genes, each
# occurring with a probability drawn from a uniform prior; a patient has
# diabetes if any gene occurs, and two blood sugar readings, each normal
# around 135 or 80 depending on it, are observed; the expectation of the
# first gene's probability, for T from 5 to 50. The setting here, 20
# bits after the binary point and 1024 pieces, is that of the runs behind
# Figure 2 (Tractables/Alea.jl hybrid/benchmarks/or/results_T.txt at
# cf5e5b8: E = 0.4999995 at every T, the error of 4.7e-7 Appendix I.1
# reports, in 559 to 674 s). Section 2.1's text speaks of numbers 25 bits
# wide (16 after the point here), 4096 pieces and 53K flips; 16 bits and
# 1024 pieces give 53 588 flips at T = 5, 4096 pieces 183 680. From
# hybrid/benchmarks/or/or.jl at cf5e5b8 (Apache License 2.0): T, the bits
# and the pieces, read from the command line there, are constants, the
# distributions are built inside the block rather than before it, and the
# harness (timing, results file) is dropped; the body is otherwise
# unchanged.
using Alea, Distributions

const QUERY = :expectation
const T = 5
const bits = 20
const pieces = 1024
const DFiP = DistFix{9 + bits, bits}

model() = @alea begin
    z = Vector(undef, T)
    prior = Vector(undef, T)
    for i in 1:T
        prior[i] = uniform(DFiP, 0.0, 1.0)
        z[i] = parametrised_flip(prior[i])
    end
    y = reduce(|, z)
    d1 = ifelse(y,
                bitblast_exponential(DFiP, Normal(135, 8), pieces, 135-64.0, 135+64.0),
                bitblast_exponential(DFiP, Normal(80, 8), pieces, 80-64.0, 80+64.0))
    d2 = ifelse(y,
                bitblast_exponential(DFiP, Normal(135, 8), pieces, 135-64.0, 135+64.0),
                bitblast_exponential(DFiP, Normal(80, 8), pieces, 80-64.0, 80+64.0))
    observe(prob_equals(d1, DFiP(79.0)))
    observe(prob_equals(d2, DFiP(136.0)))
    prior[1]
end
