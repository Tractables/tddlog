# spacex (canyon289 2022): the expected completion time of a rocket, a sum
# of three normal durations. Bit-blasting paper (Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024), Table 2: 19 bits, 32 pieces. From
# Tractables/Alea.jl hybrid/benchmarks/spacex/spacex.jl at cf5e5b8 (Apache
# License 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 11
const pieces = 32
const DFiP = DistFix{8+bits, bits}

model() = @alea begin
    engines = bitblast(DFiP, Normal(5, 1), pieces, 0.0, 64.0)
    first_stage = bitblast_exponential(DFiP, Normal(10, 3), pieces, 0.0, 64.0)
    completed_first_stage = engines + first_stage
    second_stage = bitblast_exponential(DFiP, Normal(15, 3), pieces, 0.0, 64.0)
    completed_rocket = completed_first_stage + second_stage
    completed_rocket
end
