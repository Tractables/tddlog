# Weighted model integration on a star (Zeng, Morettin, Yan, Passerini and
# Van den Broeck, ICML 2020, Section 6; the star-shaped primal graphs of
# Zeng and Van den Broeck, UAI 2019): a centre x_0 and n leaves, real
# variables uniform on [0, 1], each leaf constrained by x_0 + x_i <= 1,
# and the query x_1 <= 1/2. The constraints have volume 1/(n+1), and the
# continuous answer is (n+1)/(2n) (1 - 2^-n) + 2^-(n+1); the answer of the
# discretized program converges to it as the bits after the binary point
# grow. Each variable is a uniform fixed-point number with `bits` bits
# after the binary point; `n` is the number of leaves.
using Alea

const QUERY = :pr
const n = 16
const bits = 8
const DFiP = DistFix{2 + bits, bits}

model() = @alea begin
    x0 = uniform(DFiP, 0.0, 1.0)
    x = [uniform(DFiP, 0.0, 1.0) for i in 1:n]
    for i in 1:n
        observe(!(DFiP(1.0) < x0 + x[i]))
    end
    !(DFiP(0.5) < x[1])
end
