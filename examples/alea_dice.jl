# Two dice in Alea.jl (https://github.com/Tractables/Alea.jl), each a number
# of four bits uniform on 1 to 6, and the observation that the first shows
# at least 4. The program's value is two Booleans, the queries q1 and q2:
# that the dice sum to 7 (one value of the second die for each of the
# first's, 1/6) and that the first shows more (in 3, 4 or 5 of the
# second's 6 values, (3 + 4 + 5) / 18 = 2/3).
# scripts/alea2problog.jl writes the circuit Alea.jl builds for it, which
# is alea_dice.pl:
#   julia --project=ALEA scripts/alea2problog.jl examples/alea_dice.jl > alea_dice.pl
#   tddlog alea_dice.pl
using Alea

const QUERY = :pr

model() = @alea begin
    x = uniform(DistUInt{4}, 1, 7)
    y = uniform(DistUInt{4}, 1, 7)
    observe(DistUInt{4}(3) < x)
    (prob_equals(x + y, DistUInt{4}(7)), y < x)
end
