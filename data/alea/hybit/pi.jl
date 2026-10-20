# Pi (@10kdiver): the probability that round(x / y) of two uniform numbers
# in (0, 1] is even, which tends to (5 - pi) / 4. Bit-blasting paper (Garg,
# Holtzen, Van den Broeck, Millstein, PLDI 2024), Table 2: 14 bits.
# From Tractables/Alea.jl hybrid/benchmarks/pi/pi.jl at cf5e5b8 (Apache
# License 2.0). Alea's DistFix calls its field `mantissa`, not `number`,
# and has no `round`: round_fix is the `round` of Alea.jl commit f5eb564e
# (branch table1); the hybit branch's pi.jl calls a `round` that its
# source does not define.
using Alea, Distributions

const QUERY = :pr
const IGNORE_ERRORS = true
const b = 12

# Round to the nearest integer, a half up.
function round_fix(x::DistFix{W, F}) where {W, F}
    F == 0 && return x
    t = convert(DistFix{W - F, 0}, x)
    ifelse(x.mantissa.number.bits[W - F + 1], t + DistFix{W - F, 0}(1.0), t)
end

model() = @alea begin
    x = uniform(DistFix{b+2, b}, 1/2^b, 1.0 + 1/2^b)
    y = uniform(DistFix{b+2, b}, 1/2^b, 1.0 + 1/2^b)
    x = DistFix{b+2, 1}(x.mantissa)
    y = DistFix{b+2, 1}(y.mantissa)
    t = round_fix(x/y)
    !t.mantissa.number.bits[b+1]
end
