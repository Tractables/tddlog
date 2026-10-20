# Ported from experiments/dicejl/fw_small.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: floydwarshall-small.
using Alea

const QUERY = :pr

model() = @alea begin
    num_nodes = 3
    MAX = 14
    w = Int(floor(log2(MAX))) + 2

    edges = Dict(
        (1, 2) => uniform(DistUInt{w}, 1, 3),
        (1, 3) => uniform(DistUInt{w}, 3, 5),
        (2, 3) => uniform(DistUInt{w}, 2, 4)
    )

    distance = [[DistUInt{w}(MAX) for j in 1:num_nodes] for i in 1:num_nodes]


    for edge in edges
        coord, dist = edge[1], edge[2]
        distance[coord[1]][coord[2]] = dist
        distance[coord[2]][coord[1]] = dist
    end

    for i in 1:num_nodes
        distance[i][i] = DistUInt{w}(0)
    end

    for k in 1:num_nodes
        for i in 1:num_nodes
            for j in 1:num_nodes
                distance[i][j] = ifelse(distance[i][j] > distance[i][k]+distance[k][j], distance[i][k]+distance[k][j], distance[i][j])
            end
        end
    end

    distance[1][3]
end
