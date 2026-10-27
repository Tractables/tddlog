# A test program for scripts/alea2problog.jl: the expectation of a signed
# number under an observation. By hand: y is uniform on -3 .. 2, and given
# y != 0 its expectation is (-3 - 2 - 1 + 1 + 2) / 5 = -0.6; with
# `const lo = -1`, y is uniform on -1 .. 2 and the expectation (-1 + 1 + 2)
# / 3 = 2/3.
using Alea

const QUERY = :expectation
const lo = -3

model() = @alea begin
    y = uniform(DistInt{4}, lo, 3)
    observe(!prob_equals(y, DistInt{4}(0)))
    y
end
