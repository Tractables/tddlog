# GPA (Wu et al. 2018): whether a student is from the first of two
# countries given a GPA of 0, each country's GPA a mixture of a scaled beta
# distribution and two point masses. Bit-blasting paper (Garg, Holtzen, Van
# den Broeck, Millstein, PLDI 2024), Table 2: 25 bits, 4096 pieces. From
# Tractables/Alea.jl hybrid/benchmarks/GPA/GPA.jl at cf5e5b8 (Apache
# License 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :pr
const bits = 20
const pieces = 4096
const DFiP = DistFix{bits + 5, bits}

model() = @alea begin
    d1 = bitblast_exponential(DFiP, 4*Beta(8, 2), pieces, 0.0, 4.0)
    d2 = bitblast_exponential(DFiP, 8*Beta(5, 5), pieces, 0.0, 8.0)
    gpa1 = if flip(0.95) d1 else
            if flip(0.15) DFiP(4.0) else
                DFiP(0.0)
            end
        end
    gpa2 = if flip(0.99) d2 else
                if flip(0.1) DFiP(8.0) else
                    DFiP(0.0)
                end
            end
    n = flip(0.25)
    final = if n gpa1 else gpa2 end
    observe(prob_equals(final, DFiP(0.0)))
    n
end
