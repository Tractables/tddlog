# conjugate gaussians (Jordan 2010): a standard normal mean given two
# observed points, each normal around it with variance 1; the expectation
# of the mean (the paper's measure for a program that returns a number).
# Bit-blasting paper (Garg, Holtzen, Van den Broeck, Millstein, PLDI 2024),
# Table 2: 23 bits, 16 pieces. From Tractables/Alea.jl branch hybit
# (96b69a5, the paper's artifact) benchmarks/conjugate_gaussians/
# conjugate_gaussians.jl (Apache License 2.0), with Dice.jl's names
# changed to Alea's (Dice to Alea, @dice to @alea), and the observations 1
# and 2 rather than the file's 8 and 9: the expectations the artifact
# records for its runs (benchmarks/conjugate_gaussians/results.txt) are
# near 1, the posterior mean of 1 and 2 (17/3 is that of 8 and 9), and its
# ground truth for the table is 1.0 (benchmarks/plotting.ipynb). The 8 and
# 9 are the paper's Figure 5, data/alea/bitblast/conjugate_ex3.jl. The
# body is otherwise unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 15
const pieces = 16
const DFiP = DistFix{8 + bits, bits}
const data = DFiP.([1.0, 2.0])
const add_arg = true

model() = @alea begin
    a = bitblast(DFiP, Normal(0, 1), pieces, -16.0, 16.0)
    for datapt in data
        gaussian_observe(DFiP, pieces, -16.0, 16.0, a, 1.0, datapt, add=add_arg, exp=false)
    end
    a
end
