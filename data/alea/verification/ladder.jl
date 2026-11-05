# The ladder network of the Dice paper's network verification benchmark
# (Holtzen, Van den Broeck and Millstein, OOPSLA 2020; SHoltzen/dice
# bench/dicebench.ml, gen_ladder), ported to Alea.jl, whose own programs
# have no ladder: N rungs over two wires, N the program's argument
# (`alea_bench.jl --set N=...`). From (true, false), a rung gives
# (false, false) where the first wire is up and the rung breaks (flip
# 0.001), else (f, !f), f a flip 0.5, where either wire is up, and
# (false, false) where neither is; the value is the two wires after the
# last rung.
#
# The port follows the paper's text: each rung makes its flips when it
# runs, in the text's order (the break, then a flip 0.5 for each branch
# that sets the wires), and `@alea_ite` merges the branches as in the
# programs of Alea.jl's tour.
using Alea

const QUERY = :pr
const N = 5

function ladder(s1, s2)
    @alea_ite if s1
        @alea_ite if flip(0.001)
            (false, false)
        else
            f = flip(0.5)
            (f, !f)
        end
    else
        @alea_ite if s2
            g = flip(0.5)
            (g, !g)
        else
            (false, false)
        end
    end
end

function network()
    x = (true, false)
    for _ in 1:N
        x = ladder(x[1], x[2])
    end
    x
end

model() = network()
