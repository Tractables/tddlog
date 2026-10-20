# Hierarchical Markov switching model (SPPL, Saad, Rinard and Mansinghka,
# PLDI 2021, Table 2 and examples/hierarchcial-markov-switching.ipynb of
# github.com/probsys/sppl at e537b0c, Apache License 2.0): a hidden
# Boolean `separated` (probability 0.4) picks the means of a two-state
# Markov chain's emissions; state z_i moves to 1 with probability 0.2
# from 0 and 0.8 from 1; at each step a normal X_i of variance 1 and a
# Poisson Y_i are observed, their means mu_x[s][z] and mu_y[s][z]. The
# queries are `separated` and the last state z_n given the n
# observations. The observations are a draw from the model with
# `separated` false (means 5 and 7, which overlap), by the linear
# congruential generator below (seeded with 2021) and Distributions'
# quantiles, so that they are the same on every Julia; SPPL's notebook
# draws them with numpy at n = 100.
#
# Bit blasting: each X_i is observed as g + mean == x_i, g a standard
# normal in `pieces` exponential pieces on [-8, 8] (gaussian_observe's
# form), the mean the constant its state picks; each Y_i is a categorical
# over 0 to 31 (the Poisson truncated there) per state, compared with
# y_i. `bits` is the bits after the binary point of the numbers, `n` the
# number of steps.
using Alea, Distributions

const QUERY = :pr
const n = 20
const bits = 4
const pieces = 16
const DFiP = DistFix{6 + bits, bits}
const Y = DistUInt{5}
const p_transition = [0.2, 0.8]
const mu_x = [[5.0, 7.0], [5.0, 15.0]]
const mu_y = [[5.0, 8.0], [3.0, 8.0]]

function lcg(seed)
    state = Ref(UInt64(seed))
    () -> (state[] = state[] * 0x5851f42d4c957f2d + 0x14057b7ef767814f; ((state[] >> 11) + 0.5) * 2.0^-53)
end

# The observations (x_i, y_i), i = 1 .. 2000, drawn with s = 1 (not separated).
const DATA = let u = lcg(2021), s = 1, xs = Float64[], ys = Int[], z = u() < 0.5
    for i in 1:2000
        i > 1 && (z = u() < p_transition[z + 1])
        push!(xs, quantile(Normal(mu_x[s][z + 1], 1.0), u()))
        push!(ys, min(Int(quantile(Poisson(mu_y[s][z + 1]), u())), 31))
    end
    (xs, ys)
end

poisson(mu) = discrete(Y, (p = [pdf(Poisson(mu), k) for k in 0:31]; p ./ sum(p)))

function emit(s, z, i)
    m(a, b) = ifelse(z, DFiP(b), DFiP(a))
    mean = ifelse(s, m(mu_x[2]...), m(mu_x[1]...))
    g = bitblast_exponential(DFiP, Normal(0, 1), pieces, -8.0, 8.0)
    observe(prob_equals(g + mean, DFiP(DATA[1][i])))
    y = ifelse(s, ifelse(z, poisson(mu_y[2][2]), poisson(mu_y[2][1])),
                  ifelse(z, poisson(mu_y[1][2]), poisson(mu_y[1][1])))
    observe(prob_equals(y, Y(DATA[2][i])))
end

model() = @alea begin
    s = flip(0.4)
    z = flip(0.5)
    emit(s, z, 1)
    for i in 2:n
        z = ifelse(z, flip(p_transition[2]), flip(p_transition[1]))
        emit(s, z, i)
    end
    (s, z)
end
