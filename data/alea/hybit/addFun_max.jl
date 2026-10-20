# addfun/max (Gehr et al. 2016): the maximum of two standard normal
# numbers. Bit-blasting paper (Garg, Holtzen, Van den Broeck, Millstein,
# PLDI 2024), Table 2: 23 bits, 128 pieces. From Tractables/Alea.jl
# hybrid/benchmarks/addFun_max/addFun_max.jl at cf5e5b8 (Apache License
# 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 18
const pieces = 128
const DFiP = DistFix{5+bits, bits}

model() = @alea begin
    x = bitblast(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    y = bitblast(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    m = if (x < y) y else x end
    m
end
