# zeroone (w1) (Bissiri et al. 2016): a linear classifier's weights
# w1 and w2, uniform, under a zero-one loss on 20 observed points; the
# expectation of w1. Bit-blasting paper (Garg, Holtzen, Van den Broeck,
# Millstein, PLDI 2024), Table 2: 16 bits. From Tractables/Alea.jl
# hybrid/benchmarks/zeroone/zeroone.jl at cf5e5b8 (Apache License 2.0),
# its body unchanged, run with flag 1.
using Alea, Distributions

const QUERY = :expectation
const IGNORE_ERRORS = true
const bits = 8
const flag = 1
const DFiP = DistFix{8+bits, bits}
const ys = [1, -1, 1, -1, -1, -1, 1, 1, 1, 1, 1, -1, 1, -1, -1, -1, 1, 1, 1, 1]
const xs = DFiP.([6, 8, -1, 0, 5, 1.2, -2, 9.8, 4, 12, 1, 10, 1, 2.2, -6, 9.8, 1, 1, 1, 1])

model() = @alea begin
    w1 = uniform(DFiP, -8.0, 8.0)
    w2 = uniform(DFiP, -8.0, 8.0)
    for (x, y) in zip(xs, ys)
        temp = x*w1 + w2
        isneg = temp < DFiP(0.0)
        if y == 1
            observe(!isneg | (isneg & flip(1/ℯ)))
        else
            observe(isneg | (!isneg & flip(1/ℯ)))
        end
    end
    if flag == 1
        w1
    else
        w2
    end
end
