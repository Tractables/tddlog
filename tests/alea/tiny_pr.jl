# A test program for scripts/alea_bench.jl: a Boolean and a number under an
# observation, answered by `pr`. By hand: P(a or b) = 0.72; q1 = P(a & !b |
# e) = 0.12 / 0.72; x + 1 is 2 when a, else 1 to 4 uniformly, so given e
# its values 1, 3 and 4 have 0.105 / 0.72 each and 2 has 0.405 / 0.72.
using Alea

const QUERY = :pr

model() = @alea begin
    a = flip(0.3)
    b = flip(0.6)
    x = ifelse(a, DistUInt{3}(1), uniform(DistUInt{3}, 0, 4))
    observe(a | b)
    (a & !b, x + DistUInt{3}(1))
end
