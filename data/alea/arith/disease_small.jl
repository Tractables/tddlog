# Ported from experiments/dicejl/disease_small.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: disease-small.
using Alea

const QUERY = :pr

model() = @alea begin
    function binomial(n::DistUInt{W}, p, max::Int) where W
        output = DistUInt{W}(0)
        for i in 0:max-1
            output += ifelse((DistUInt{W}(i)<n) & flip(p), DistUInt{W}(1), DistUInt{W}(0))
        end
        return output
    end


    param = uniform(DistUInt{12}, 50, 101)
    nummet = binomial(param, 0.5, 100)
    numinfected = binomial(nummet, 0.3, 100)
    numinfected
end
