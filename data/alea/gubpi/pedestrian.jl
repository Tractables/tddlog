# The pedestrian (Mak, Ong, Paquet and Wagner, "Densities of Almost Surely
# Terminating Probabilistic Programs are Differentiable Almost Everywhere",
# ESOP 2021, Example 1.1; Beutner, Ong and Zaiser, "Guaranteed Bounds for
# Posterior Inference in Universal Probabilistic Programming", PLDI 2022,
# Section 7): a pedestrian starts at a distance uniform on [0, 3] from
# home and walks steps of length uniform on [0, 1], each towards home or
# away from it with probability 1/2, until reaching home; the total
# distance walked is observed as 1.1 under normal noise of standard
# deviation 0.1. The query is the expectation of the starting point. The
# walk is unrolled to `steps` steps, and the program observes that it has
# reached home by then, which conditions the posterior on a walk of at
# most that many steps; the posterior converges to the unbounded one as
# `steps` grows.
#
# Bit blasting: the start and the steps are uniform fixed-point numbers
# with `bits` bits after the binary point; the noise is a normal in
# `pieces` exponential pieces on [-1, 1], observed as noise + distance ==
# 1.1 (gaussian_observe's form).
using Alea, Distributions

const QUERY = :expectation
const steps = 8
const bits = 4
const pieces = 16
const DFiP = DistFix{7 + bits, bits}

model() = @alea begin
    start = uniform(DFiP, 0.0, 3.0)
    position = start
    distance = DFiP(0.0)
    for k in 1:steps
        walking = DFiP(0.0) < position
        step = uniform(DFiP, 0.0, 1.0)
        position = ifelse(walking, ifelse(flip(0.5), position + step, position - step), position)
        distance = ifelse(walking, distance + step, distance)
    end
    observe(!(DFiP(0.0) < position))
    noise = bitblast_exponential(DFiP, Normal(0, 0.1), pieces, -1.0, 1.0)
    observe(prob_equals(noise + distance, DFiP(1.1)))
    start
end
