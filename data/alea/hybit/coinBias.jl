# coinBias (Gehr et al. 2016): a coin's bias, Beta(2, 5), given five
# observed tosses. Bit-blasting paper (Garg, Holtzen, Van den Broeck,
# Millstein, PLDI 2024), Table 2: 22 bits, 4096 pieces. From
# Tractables/Alea.jl hybrid/benchmarks/coinBias/coinBias.jl at cf5e5b8
# (Apache License 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 20
const pieces = 4096
const DFiP = DistFix{bits+2, bits}
const y = [1, 1, 0, 1, 0]

model() = @alea begin
    a = bitblast_exponential(DFiP, Beta(2, 5), pieces, 0.0, 1.0)
    for i = 1:5
        b = parametrised_flip(a)
        observe((y[i]!=1) | b)
        observe((y[i]==1) | !b)
    end
    return a
end
