# Ported from experiments/dicejl/ranking_large.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: ranking-large.
using Alea

const QUERY = :pr

model() = @alea begin
    uniq_count = 4
    bits = Int(floor(log2(uniq_count)) + 4)

    rankings = [[3, 2, 1, 0],
                [1, 3, 2, 0],
                [1, 2, 3, 0],
                [3, 1, 2, 0],
                [3, 2, 1, 0]]

    ranks = [uniform(DistInt{bits}, 0, uniq_count) for i in 1:uniq_count]
    for i in 1:uniq_count
        for j in i+1:uniq_count
            observe(!prob_equals(ranks[i], ranks[j]))
        end
    end


    for data in rankings
        obs_rank = [x + uniform(DistInt{bits}, -uniq_count, uniq_count+1) for x in ranks]
        for i in 1:uniq_count-1
            a, b = data[i]+1, data[i+1] + 1
            observe(obs_rank[a] < obs_rank[b])
        end
    end

    (ranks[1], ranks[2], ranks[3], ranks[4])
end
