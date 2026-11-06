# pcfg: ported to Alea.jl from PluckArtifact.jl/src/table1/sequence_models/
# pluck/pcfg.jl of the artifact of Bowers et al., "Stochastic Lazy
# Knowledge Compilation for Inference in Discrete Probabilistic Programs"
# (PLDI 2025), Zenodo 10.5281/zenodo.15051140 (CC-BY-4.0): the Pluck
# program of the paper's Table 1, row PCFG. The query is the probability
# that the grammar
#   S  -> XX YY | YY XX     (by a flip 0.5)
#   XX -> a | b XX XX       (a by a flip 0.5)
#   YY -> c | c XX | c S    (c alone by a flip 0.5, then XX or S by another)
# generates TARGET from S. The artifact's Dice.jl baseline
# (sequence_models/dice/pcfg.jl) is another grammar (YY -> c XX by 0.5)
# and bounds its recursion by fuel; this port is Pluck's grammar, bounded
# by the target alone.
#
# The generation reads the target left to right: generates(sym, i, j)
# is that sym, generated at position i, is the target's symbols i to
# j - 1. Each choice is keyed by its symbol and its position, which gives
# the original's distribution where no derivation expands one symbol twice
# at one position. The grammar has neither left recursion nor an empty
# production: S's productions start with XX or YY, and XX's and YY's with
# a terminal, so a derivation expands at a position at most S and then one
# of XX and YY, each once. No generation then reads one key twice, a
# derivation has the original's probability, and the choices at its keys
# fix it, so that two derivations exclude each other.
using Alea

const QUERY = :pr

const TARGET = [:c, :b, :b, :b, :a, :b, :b, :a, :b, :a, :b, :b, :a, :b, :a, :a, :a, :a, :a, :b, :a, :a, :a]

function pcfg_generates(target)
    n = length(target)
    x_ends = [flip(0.5) for _ in 1:n]     # XX at i is a (otherwise b XX XX)
    s_order = [flip(0.5) for _ in 1:n]    # S at i is XX YY (otherwise YY XX)
    y_ends = [flip(0.5) for _ in 1:n]     # YY at i is c alone
    y_xx = [flip(0.5) for _ in 1:n]       # YY at i goes on with XX (otherwise S)
    memo = Dict{Tuple{Symbol, Int, Int}, AnyBool}()
    any_of(xs) = reduce(|, xs; init = false)
    function generates(sym, i, j)
        haskey(memo, (sym, i, j)) && return memo[(sym, i, j)]
        g = if i > n || j <= i
            false
        elseif sym == :XX
            a = target[i] == :a && j == i + 1 ? x_ends[i] : false
            b = target[i] == :b ? !x_ends[i] & any_of([generates(:XX, i + 1, k) & generates(:XX, k, j) for k in i+2:j-1]) : false
            a | b
        elseif sym == :YY
            if target[i] != :c
                false
            else
                alone = j == i + 1 ? y_ends[i] : false
                on = !y_ends[i] & ((y_xx[i] & generates(:XX, i + 1, j)) | (!y_xx[i] & generates(:S, i + 1, j)))
                alone | on
            end
        else
            xy = any_of([generates(:XX, i, k) & generates(:YY, k, j) for k in i+1:j-1])
            yx = any_of([generates(:YY, i, k) & generates(:XX, k, j) for k in i+1:j-1])
            (s_order[i] & xy) | (!s_order[i] & yx)
        end
        memo[(sym, i, j)] = g
    end
    generates(:S, 1, n + 1)
end

model() = @alea begin
    pcfg_generates(TARGET)
end
