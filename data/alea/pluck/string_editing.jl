# string_editing: ported to Alea.jl from PluckArtifact.jl/src/table1/
# sequence_models/pluck/string_editing.jl of the artifact of Bowers et al.,
# "Stochastic Lazy Knowledge Compilation for Inference in Discrete
# Probabilistic Programs" (PLDI 2025), Zenodo 10.5281/zenodo.15051140
# (CC-BY-4.0): the Pluck program of the paper's Table 1, row String
# Editing, with the perturb its benchmark defines (perturb_defs, "the one
# we're using"). The query is the probability that perturb turns INPUT
# into OUTPUT:
#   perturb Nil         = random_string 0.99
#   perturb (Cons c cs) = let delete = flip 0.01, insertion = random_string 0.99,
#                             rest = perturb cs
#                         in if delete then rest
#                            else insertion ++ Cons (if flip 0.01 then random_char else c) rest
#   random_string p     = if flip p then Nil else Cons random_char (random_string p)
# with random_char uniform over a to e. The artifact's Dice.jl baseline
# (sequence_models/dice/string_editing.jl) is another model and bounds
# its recursion by fuel; this port is Pluck's model, bounded by the two
# strings alone.
#
# The perturbation is read against OUTPUT left to right: perturbs(k, i)
# is that perturb of INPUT from position k is OUTPUT from position i to
# its end, and inserts(k, i, j) that the string inserted before input
# position k (k = length(INPUT) + 1: after the input) is OUTPUT's
# characters i to j - 1. A choice is keyed by the input position it
# perturbs and, within an inserted string, by the output position it
# writes, which gives the original's distribution where no perturbation
# reads one key twice. The input positions are perturbed once each, in
# order, and an inserted string writes each output position once, its end
# read at the position after its last character; an empty insertion, or a
# deleted input character, leaves the output position to the next input
# position, a key of its own. No perturbation then reads one key twice, it
# has the original's probability, and the choices at its keys fix it, so
# that two perturbations exclude each other.
using Alea

const QUERY = :pr

const INPUT = "edcc"
const OUTPUT = "eddde"

function string_editing_perturbs(input, output)
    n, m = length(input), length(output)
    code(c) = DistUInt{3}(c - 'a')
    random_char() = uniform(DistUInt{3}, 0, 5)
    deleted = [flip(0.01) for _ in 1:n]
    replaced = [flip(0.01) for _ in 1:n]
    replacement = [random_char() for _ in 1:n]
    ends = [flip(0.99) for _ in 1:n+1, _ in 1:m+1]
    inserted = [random_char() for _ in 1:n+1, _ in 1:m]
    any_of(xs) = reduce(|, xs; init = false)
    memo_ins = Dict{Tuple{Int, Int, Int}, AnyBool}()
    function inserts(k, i, j)
        haskey(memo_ins, (k, i, j)) && return memo_ins[(k, i, j)]
        g = if i == j
            ends[k, i]
        elseif i > m || j < i
            false
        else
            !ends[k, i] & prob_equals(inserted[k, i], code(output[i])) & inserts(k, i + 1, j)
        end
        memo_ins[(k, i, j)] = g
    end
    written(k, d) = (!replaced[k] & (input[k] == d)) | (replaced[k] & prob_equals(replacement[k], code(d)))
    memo = Dict{Tuple{Int, Int}, AnyBool}()
    function perturbs(k, i)
        haskey(memo, (k, i)) && return memo[(k, i)]
        g = if k == n + 1
            inserts(k, i, m + 1)
        else
            kept = any_of([inserts(k, i, j) & written(k, output[j]) & perturbs(k + 1, j + 1) for j in i:m])
            (deleted[k] & perturbs(k + 1, i)) | (!deleted[k] & kept)
        end
        memo[(k, i)] = g
    end
    perturbs(1, 1)
end

model() = @alea begin
    string_editing_perturbs(INPUT, OUTPUT)
end
