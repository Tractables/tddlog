# The diamond network of the Dice paper's network verification benchmark
# (Holtzen, Van den Broeck and Millstein, OOPSLA 2020), as Alea.jl's own
# tour writes it: tutorial/tour_1_core.jl of Tractables/Alea.jl at
# cf5e5b8d6d34e63ae9ab614baf242d0d3520aa8b (Apache License 2.0), also
# test/tours/tour_1_core.jl there. The tour's loop runs 5 diamonds; here
# it runs N, the program's argument (`alea_bench.jl --set N=...`), and
# `model()` returns the network's signal, the tour's `pr(net)`. The body is
# the tour's.
using Alea

const QUERY = :pr
const N = 5

function diamond(s1)
    @alea_ite begin
        route = flip(0.5)
        s2 = if route s1 else false end
        s3 = if route false else s1 end
        drop = flip(0.0001)
        # Functions in Julia implicitly return the value of the last expression
        s2 | (s3 & !drop)
    end
end

function network()
    net = true
    for _ in 1:N
        net = diamond(net)
    end
    net
end

model() = network()
