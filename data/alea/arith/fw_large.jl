# Ported from experiments/dicejl/fw_large.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: floydwarshall-large.
using Alea

const QUERY = :pr

model() = @alea begin
    num_nodes = 8
    MAX = num_nodes * 3 * num_nodes
    w = Int(floor(log2(MAX))) + 2
    edge_list = [[0, 2],
    [0, 4],
    [0, 5],
    [0, 6],
    [1, 4],
    [1, 6],
    [2, 3],
    [2, 7],
    [3, 6],
    [3, 7],
    [4, 7],
    [5, 6],
    [6, 7]]
    edges = Dict()

    for edge in edge_list
        diff = edge[2] - edge[1]
        half = Int(floor((diff/2)))
        edges[(edge[1]+1, edge[2]+1)] = uniform(DistUInt{w}, 1, 2*diff+1)
    end


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

    distance[1][8]
end
