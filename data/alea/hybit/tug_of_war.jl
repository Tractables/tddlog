# Tug of war (Huang et al. 2021): two players' skills, each a normal
# distribution, four observed matches where a player may be lazy, and the
# probability that the first wins a fifth. Bit-blasting paper (Garg,
# Holtzen, Van den Broeck, Millstein, PLDI 2024), Table 2: 22 bits, 16
# pieces. From Tractables/Alea.jl hybrid/benchmarks/tug_of_war/tug_of_war.jl
# at cf5e5b8 (Apache License 2.0); DFiP moved out of the block, the body
# otherwise unchanged (it draws each skill twice, as the source does).
using Alea, Distributions

const QUERY = :pr
const bits = 17
const pieces = 16
const DFiP = DistFix{bits + 5, bits}

model() = @alea begin
    alice_skill = bitblast_exponential(DFiP, Normal(5, 1), pieces, -3.0, 13.0)
    bob_skill = bitblast_exponential(DFiP, Normal(5, 1), pieces, -3.0, 13.0)
    alice_skill = bitblast_exponential(DFiP, Normal(5, 1), pieces, -3.0, 13.0)
    bob_skill = bitblast_exponential(DFiP, Normal(5, 1), pieces, -3.0, 13.0)
    lazy_lag = DFiP(2.0)
    match = Vector(undef, 5)
    for i = 1:5
        match[i] = (ifelse(flip(1/3), alice_skill - lazy_lag, alice_skill) < (ifelse(flip(1/3), bob_skill - lazy_lag, bob_skill)))
    end
    observe(match[1])
    observe(!match[2])
    observe(match[3])
    observe(!match[4])
    match[5]
end
