# The Yao-Vehtari-Gelman model (bit-blasting paper, Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024, Section 2.2, Figures 3 and 4): two means
# uniform on [-16, 16], nine observations, each normal around the first
# mean with probability 2/3 and around the second otherwise; the posterior
# of the first mean is bimodal. The paper plots that posterior at 6 bits
# after the binary point and 256 pieces; here the program is queried for
# its expectation. From Tractables/Alea.jl
# hybrid/benchmarks/multimodal/multimodal.jl at cf5e5b8 (Apache License
# 2.0): the distributions are built inside the block, the plotting and
# results file are dropped, and the program is run with `ignore_errors`,
# as the source's `pr` is; the body is otherwise unchanged.
using Alea, Distributions

const QUERY = :expectation
const IGNORE_ERRORS = true
const bits = 6
const pieces = 256
const DFiP = DistFix{7 + bits, bits}
const datapt = [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, -5.0, -5.0, -5.0]

model() = @alea begin
    mu1 = uniform(DFiP, -16.0, 16.0)
    mu2 = uniform(DFiP, -16.0, 16.0)
    for data in datapt
        y = ifelse(flip(2/3),
                   bitblast_exponential(DFiP, Normal(0, 1), pieces, -8.0, 8.0) + mu1,
                   bitblast_exponential(DFiP, Normal(0, 1), pieces, -8.0, 8.0) + mu2)
        observe(prob_equals(y, DFiP(data)))
    end
    (mu1)
end
