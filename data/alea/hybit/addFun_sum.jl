# Addfun/sum (Gehr et al. 2016): the sum of two of three standard normal
# numbers. Bit-blasting paper (Garg, Holtzen, Van den Broeck, Millstein,
# PLDI 2024), Table 2: 23 bits, 16 pieces. From Tractables/Alea.jl
# hybrid/benchmarks/addFun_sum/addFun_sum.jl at cf5e5b8 (Apache License
# 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 18
const pieces = 16
const DFiP = DistFix{5+bits, bits}
const indices = [1, 0, 1]

model() = @alea begin
    x = Vector(undef, 3)
    for i=1:3
        x[i] = bitblast(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    end
    sum = DFiP(0.0)
    for i=1:3
        sum = ifelse(indices[i] == 1, sum + x[i], sum)
    end
    sum
end
