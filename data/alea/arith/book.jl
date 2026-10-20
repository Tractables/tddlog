# Ported from experiments/dicejl/book_mod.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: book.
using Alea

const QUERY = :pr

model() = @alea begin
curValue = uniform(DistInt{14}, 1, 501)
tgtValue = uniform(DistInt{14}, 1, 501)



delta = tgtValue - curValue

for i=1:1
    d = ifelse((delta < DistInt{14}(5)) & (delta > DistInt{14}(-5)),
        DistInt{14}(0),
        ifelse(delta < DistInt{14}(0),

            ifelse(delta > DistInt{14}(-25),
            delta + uniform(DistInt{14}, -10, 11),
            ifelse(delta > DistInt{14}(-50),
            delta + uniform(DistInt{14}, -20, 21),
            ifelse(delta > DistInt{14}(-100),
            delta + uniform(DistInt{14}, -50, 51),
            delta + uniform(DistInt{14}, -100, 101)))),

            ifelse(delta < DistInt{14}(25),
            delta + uniform(DistInt{14}, -10, 11),
            ifelse(delta < DistInt{14}(50),
            delta + uniform(DistInt{14}, -20, 21),
            ifelse(delta < DistInt{14}(100),
            delta + uniform(DistInt{14}, -50, 51),
            delta + uniform(DistInt{14}, -100, 101))))
        )
    )

    curValue = curValue + d

    curValue = ifelse(curValue > DistInt{14}(500),
                    DistInt{14}(500),
                ifelse(curValue < DistInt{14}(1),
                    DistInt{14}(1),
                    curValue))

    delta = tgtValue - curValue
end

(delta < DistInt{14}(5)) && (delta > DistInt{14}(-5))
end
