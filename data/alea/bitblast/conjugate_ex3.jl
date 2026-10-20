# Conjugate Gaussians with low-probability observations (bit-blasting
# paper, Garg, Holtzen, Van den Broeck, Millstein, PLDI 2024, Section 2.3,
# Figures 5 and 6): a standard normal mean and two observations, 8 and 9,
# each normal around it with variance 1; the posterior is normal with mean
# 17/3 and variance 1/3. The paper plots it at 16 bits after the binary
# point and 2048 pieces; here the program is queried for its expectation.
# From Tractables/Alea.jl hybrid/benchmarks/conjugate_gaussians/
# conjugate_gaussians.jl at cf5e5b8 (Apache License 2.0): the bits and
# pieces, read from the command line there, are constants, and the
# plotting and results file are dropped; the body is otherwise unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 16
const pieces = 2048
const DFiP = DistFix{8 + bits, bits}
const data = DFiP.([8.0, 9.0])
const add_arg = true

model() = @alea begin
    a = bitblast(DFiP, Normal(0, 1), pieces, -16.0, 16.0)
    for datapt in data
        gaussian_observe(DFiP, pieces, -16.0, 16.0, a, 1.0, datapt, add=add_arg, exp=false)
    end
    a
end
