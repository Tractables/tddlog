# weekend (Gehr et al. 2016): whether it is a weekend given an hour drawn
# from one of two normal distributions. Bit-blasting paper (Garg, Holtzen,
# Van den Broeck, Millstein, PLDI 2024), Table 2: 24 bits, 4096 pieces.
# From Tractables/Alea.jl hybrid/benchmarks/weekend/weekend.jl at cf5e5b8
# (Apache License 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :pr
const precision = 20
const num_pieces = 4096
const DFiP = DistFix{4+precision, precision}

model() = @alea begin
    isWeekend = flip(2/7)
    hour = if isWeekend
        bitblast_exponential(DFiP, Normal(5, 4), num_pieces, 0.0, 8.0)
    else
        bitblast_exponential(DFiP, Normal(2, 4), num_pieces, 0.0, 8.0)
    end
    observe(hour == DFiP(6.0))
    isWeekend
end
