# trueskill (Gehr et al. 2016): two players' skills, standard normal, two
# observed matches with normal performance noise, and the probability
# that the first wins a third. Bit-blasting paper (Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024), Table 2: 10 bits, 16 pieces. From
# Tractables/Alea.jl hybrid/benchmarks/trueskill/trueskill.jl at cf5e5b8
# (Apache License 2.0); DFiP moved out of the block, the body otherwise
# unchanged.
using Alea, Distributions

const QUERY = :pr
const bits = 4
const pieces = 16
const DFiP = DistFix{bits + 6, bits}

model() = @alea begin
    alice_skill = bitblast(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    bob_skill = bitblast(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    match = Vector(undef, 3)
    for i = 1:3
        match[i] = (bitblast(DFiP, Normal(0, 2), pieces, -16.0, 16.0) + alice_skill) < (bitblast(DFiP, Normal(0, 2), pieces, -16.0, 16.0) + bob_skill)
    end
    observe(match[1])
    observe(!match[2])
    match[3]
end
