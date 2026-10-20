# A mixture of two Laplace distributions observed under normal noise: a
# location a, Laplace around 0 or 1 (probability 1/2 each) of scale
# 1/64, and four points observed, each a + g with g standard normal; the
# query is the expectation of a. From Tractables/Alea.jl
# hybrid/benchmarks/laplace_scaling/laplace_scaling.jl at cf5e5b8 (Apache
# License 2.0; the bit-blasting paper's repository, Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024), at its commented setting (10 bits after
# the binary point, 1024 pieces, scale 0.015625): the sizes it reads from
# the command line are constants and the location is drawn inside the
# block; the body is otherwise unchanged. The Laplace distributions are
# Alea's `laplace`, two exponentials bit blasted exactly; each observation
# is gaussian_observe's g + a == datapt, g in `pieces` linear pieces.
using Alea, Distributions

const QUERY = :expectation
const bits = 10
const pieces = 1024
const scale = 0.015625
const DFiP = DistFix{7 + bits, bits}
const data = [-0.5, -0.75, 1.75, 1.5]

model() = @alea begin
    a = ifelse(flip(0.5),
               laplace(DFiP, 0.0, scale, -8.0, 8.0),
               laplace(DFiP, 1.0, scale, -7.0, 9.0))
    for datapt in data
        gaussian_observe(DFiP, pieces, -8.0, 8.0, a, 1.0, DFiP(datapt), add=true, exp=true)
    end
    a
end
