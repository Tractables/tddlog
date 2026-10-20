# ClickGraph (Gehr et al. 2016): the similarity of two search results
# given ten observed clicks. Bit-blasting paper (Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024), Table 2: 10 bits. From Tractables/Alea.jl
# hybrid/benchmarks/clickGraph/clickGraph.jl at cf5e5b8 (Apache License
# 2.0); the vectors sim0 and sim1 made within the block, the body
# otherwise unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 8
const DFiP = DistFix{2+bits, bits}
const clicks0 = [true, true, true, false, false]
const clicks1 = [true, true, true, false, false]

model() = @alea begin
    sim0 = Vector(undef, 5)
    sim1 = Vector(undef, 5)
    similarityAll = uniform(DFiP, 0.0, 1.0)
    for i=1:5
        sim = parametrised_flip(similarityAll)
        beta1 = uniform(DFiP, 0.0, 1.0)
        beta2 = ifelse(sim, beta1, uniform(DFiP, 0.0, 1.0))
        sim0[i] = parametrised_flip(beta1)
        sim1[i] = parametrised_flip(beta2)
    end
    for i=1:5
        observe(sim0[i] == clicks0[i])
        observe(sim1[i] == clicks1[i])
    end
    similarityAll
end
