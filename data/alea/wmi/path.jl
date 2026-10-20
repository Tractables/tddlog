# Weighted model integration on a path (Zeng, Morettin, Yan, Passerini and
# Van den Broeck, "Scaling up Hybrid Probabilistic Inference with Logical
# and Arithmetic Constraints via Message Passing", ICML 2020, Section 6;
# the path-shaped primal graphs of Zeng and Van den Broeck, "Efficient
# Search-Based Weighted Model Integration", UAI 2019): n real variables
# uniform on [0, 1], each pair of neighbours constrained by
# x_i + x_{i+1} <= 1, and the query x_1 <= 1/2. The continuous volume of
# the constraints is E_n / n!, E_n the n-th Euler zigzag number; the
# answer of the discretized program converges to that of the continuous
# one as the bits after the binary point grow. Each x_i is a uniform
# fixed-point number with `bits` bits after the binary point, so that a
# constraint compares a sum of two numbers with a constant; `n` is the
# length of the path.
using Alea

const QUERY = :pr
const n = 16
const bits = 8
const DFiP = DistFix{2 + bits, bits}

model() = @alea begin
    x = [uniform(DFiP, 0.0, 1.0) for i in 1:n]
    for i in 1:n-1
        observe(!(DFiP(1.0) < x[i] + x[i+1]))
    end
    !(DFiP(0.5) < x[1])
end
