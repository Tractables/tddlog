# Ported from experiments/dicejl/triangle_small.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: triangle-small.
using Alea

const QUERY = :pr

model() = @alea begin
    a = uniform(DistInt{15},-50, 51)
    b = uniform(DistInt{15},-50, 51)
    c = uniform(DistInt{15},-50, 51)



    zero = DistInt{15}(0)


    type =
        ifelse((a <= zero) | (b <= zero) | (c <= zero),
            DistUInt{2}(3),
        ifelse(
            (a+b<=c) | (a+c<=b) | (b+c <= a),
            DistUInt{2}(3),
        ifelse(
            prob_equals(a, b) & prob_equals(a, c) & prob_equals(b, c),
            DistUInt{2}(2),
        ifelse(
            prob_equals(a, b) | prob_equals(a, c) | prob_equals(b, c),
            DistUInt{2}(1),
            DistUInt{2}(0)
        ))))

    type

end
