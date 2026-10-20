# Ported from experiments/dicejl/linext_large.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: linext-large.
using Alea

const QUERY = :pr

model() = @alea begin

    uniq_count = 8

    bits = Int(floor(log2(uniq_count)) + 4)
    partial = [[1, 2],
        [3, 2],
        [1, 4],
        [5, 8],
        [6, 5],
        [3, 4]]

    ranks = [uniform(DistInt{bits}, 0, uniq_count) for i in 1:uniq_count]
    for i in 1:uniq_count
        for j in i+1:uniq_count
            observe(!prob_equals(ranks[i], ranks[j]))
        end
    end


    for comp in partial
        observe(ranks[comp[1]] < ranks[comp[2]])
    end

    (ranks[1], ranks[2], ranks[3], ranks[4], ranks[5], ranks[6], ranks[7], ranks[8])
end
