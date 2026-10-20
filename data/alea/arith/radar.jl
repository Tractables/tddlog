# Ported from experiments/dicejl/radar.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: radar1.
using Alea

const QUERY = :pr

model() = @alea begin
    w = 10

    function T(mu, l, r)
        x = uniform(DistInt{w}, -l, 1) + uniform(DistInt{w}, 0, l+1)
        y = uniform(DistInt{w}, -r, 1) + uniform(DistInt{w}, 0, r+1)
        observe(x <= DistInt{w}(0) && y > DistInt{w}(0))

        return ifelse(flip(l/(l+r)), x, y) + mu
    end

    b1 = flip(0.2)
    b2 = ifelse(b1, true, flip(0.2))
    x1 = uniform(DistInt{w}, 0, 11)
    x2 = uniform(DistInt{w}, -2, 3) + x1

    o1 = x1 + ifelse(b1, T(x1, 5, 1), T(x1, 5, 5))
    o2 = x2 + ifelse(b2, T(x2, 5, 1), T(x2, 5, 5))

    observe(o1 == DistInt{w}(5));
    observe(b1)

    x2

    # T(DistInt{w}(0), 5, 5)
end
