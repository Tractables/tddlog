# alea2problog.jl: the Boolean circuit Alea.jl builds for a program, written
# as a ProbLog program that tddlog reads.
#
#     julia --project=ALEA alea2problog.jl PROGRAM.jl > PROGRAM.pl
#
# ALEA is a checkout of Alea.jl (https://github.com/Tractables/Alea.jl) set
# up as its README says; the script was written against Alea.jl at commit
# cf5e5b8 on Julia 1.11.
#
# PROGRAM.jl is Julia code, evaluated in a module of its own, that defines
#
#   - `model()`, which returns what an `@alea begin ... end` block returns,
#     or a plain distribution: a Boolean, a number (DistUInt, DistInt,
#     DistFix) or a tuple of them;
#   - `QUERY`, `:pr` or `:expectation`: what is asked of the numbers;
#   - optionally `IGNORE_ERRORS = true`, which drops Alea's probabilistic
#     errors from the inference, as `pr(...; ignore_errors = true)` does.
#
# For example (tests/alea/tiny_pr.jl):
#
#     using Alea
#
#     const QUERY = :pr
#
#     model() = @alea begin
#         a = flip(0.3)
#         b = flip(0.6)
#         x = ifelse(a, DistUInt{3}(1), uniform(DistUInt{3}, 0, 4))
#         observe(a | b)
#         (a & !b, x + DistUInt{3}(1))
#     end
#
# Alea represents every Boolean of a program as a DAG of flips, &, | and !,
# and a number as a vector of such Booleans (its bits, the most significant
# first); its `pr` compiles the queried bits and the conjunction of the
# observations into one BDD, its variables the flips in the order they were
# created. The script runs `model()` and writes that DAG, without inference:
# a flip is a probabilistic fact `P::fN.` (the N-th flip of the circuit in
# the order Alea created them), an `&` a rule, a `|` two rules, a `!` a
# negated literal, and every observation `evidence(...)`. The rules come in
# a depth-first post-order from the queries' literals and then the
# observations, every gate after the gates it reads.
#
# Each component K of the program's value is queried on its own, as
# `pr(c1, c2, ...)` answers them (the joint distribution of a returned
# tuple is not asked):
#
#   - a Boolean: `qK`, its probability of being true;
#   - under `:pr`, a number whose bits that are not constant are
#     b_1 ... b_n (at most 16): `qK(U)` for every U = sum_i b_i 2^(n-i) in
#     0 .. 2^n - 1, the probability of that world of its bits (Alea's
#     `frombits` gives the value a world names);
#   - under `:expectation`, a number: `eK(I)`, the probability that its I-th
#     bit is true, for each bit I that is not constant. The expectation is
#     linear in these, as Alea's `expectation` computes it: for W bits, the
#     sum of eK(I) 2^(W-I), less eK(1) 2^W for a signed number (DistInt,
#     DistFix), and divided by 2^F for a DistFix{W, F}.
#
# The probabilistic errors of a program run without IGNORE_ERRORS are left
# out of the circuit: Alea's `pr` stops where an error has a positive
# probability, and where none has, the circuit's answers are Alea's.
#
# Checked: tests/test_alea2problog.py, where TDDLOG_ALEA names Julia with
# Alea.jl, checks that the script writes tests/alea/*.pl from
# tests/alea/*.jl, and tests/it/examples.rs checks tddlog's answers on those
# against the ones computed by hand in the programs' comments. The
# baselines branch's tests check the circuits' answers against Alea's `pr`,
# and its benchmarks give tddlog the circuits this script writes.

module Alea2ProbLog

using Alea
using Alea: MetaDist, Flip, DistAnd, DistOr, DistNot, CondError, returnvalue

const USAGE = "usage: julia --project=ALEA alea2problog.jl PROGRAM.jl > PROGRAM.pl"

# The worlds of a `:pr` component are queried one atom each; past this many
# bits the script refuses.
const MAX_WORLD_BITS = 16

# The program's module.
module Program end

# Evaluate the program's text (read from `path`) in Program; its QUERY and
# whether it ignores errors.
function load(text, path)
    Base.include_string(Program, text, path)
    name = basename(path)
    isdefined(Program, :model) || error("$name defines no model()")
    isdefined(Program, :QUERY) || error("$name defines no QUERY (:pr or :expectation)")
    query = Program.QUERY
    query in (:pr, :expectation) || error("QUERY must be :pr or :expectation, not $query")
    query, isdefined(Program, :IGNORE_ERRORS) && Program.IGNORE_ERRORS
end

# The components of a program's value.
components(x::Tuple) = reduce(vcat, map(components, x); init = Any[])
components(x) = Any[x]

# A number's bits, the most significant first, constants included.
allbits(x::DistUInt) = x.bits
allbits(x::DistInt) = x.number.bits
allbits(x::DistFix) = x.mantissa.number.bits

isbool(x) = x isa Dist{Bool} || x isa Bool

struct Model
    comps::Vector{Any}
    evidence::Vector{AnyBool}
    errors::Vector{CondError}
end

# Run the loaded program's model().
function build(ignore_errors)
    m = Base.invokelatest(Program.model)
    if m isa MetaDist
        errors = ignore_errors ? CondError[] : m.errors
        Model(components(returnvalue(m)), m.observations, errors)
    else
        Model(components(m), AnyBool[], CondError[])
    end
end

# The Booleans queried for a number under `:pr`: its bits that are not
# constant.
function queried(c, query)
    isbool(c) && return c isa Bool ? Dist{Bool}[] : Dist{Bool}[c]
    bits = Dist{Bool}[b for b in allbits(c) if b isa Dist{Bool}]
    unique(bits)
end

# The nodes under `roots`, every node after its children; iterative, as a
# circuit can be as deep as a loop is long.
function topological(roots)
    order = Dist{Bool}[]
    seen = IdDict{Dist{Bool}, Bool}()
    for root in roots
        root isa Dist{Bool} || continue
        stack = Tuple{Dist{Bool}, Bool}[(root, false)]
        while !isempty(stack)
            node, expanded = pop!(stack)
            if expanded
                push!(order, node)
                continue
            end
            haskey(seen, node) && continue
            seen[node] = true
            push!(stack, (node, true))
            for child in children(node)
                haskey(seen, child) || push!(stack, (child, false))
            end
        end
    end
    order
end

children(n::DistAnd) = Dist{Bool}[n.x, n.y]
children(n::DistOr) = Dist{Bool}[n.x, n.y]
children(n::DistNot) = Dist{Bool}[n.x]
children(::Flip) = Dist{Bool}[]

prob_text(p) = repr(Float64(p))

struct Circuit
    order::Vector{Dist{Bool}}
    names::IdDict{Dist{Bool}, String}
end

# The flips of the model's circuit (its answers and observations),
# numbered from 1 in the order Alea created them, so that the names do not
# depend on what else the process built before.
function flip_numbers(model::Model, query)
    roots = Any[n for (_, conj) in query_atoms(model, query) if conj !== nothing for (n, _) in conj]
    append!(roots, model.evidence)
    fs = sort([n for n in topological(roots) if n isa Flip]; by = f -> f.global_id)
    IdDict{Dist{Bool}, Int}(f => i for (i, f) in enumerate(fs))
end

function circuit(roots, flipno)
    order = topological(roots)
    names = IdDict{Dist{Bool}, String}()
    k = 0
    for n in order
        if n isa Flip
            names[n] = "f$(flipno[n])"
        elseif !(n isa DistNot)
            k += 1
            names[n] = "g$k"
        end
    end
    Circuit(order, names)
end

flips(c::Circuit) = sort([n for n in c.order if n isa Flip]; by = f -> f.global_id)
gates(c::Circuit) = [n for n in c.order if n isa DistAnd || n isa DistOr]

# The literal of a node: its atom, or a negation's.
function literal(c::Circuit, n::Dist{Bool}, neg)
    while n isa DistNot
        n, neg = n.x, !neg
    end
    (c.names[n], neg)
end

# The queries of the program: (atom, conjunction of (node, polarity)), the
# conjunction empty for an answer that is constantly true and `nothing`
# for one constantly false.
function query_atoms(model::Model, query)
    atoms = []
    for (k, c) in enumerate(model.comps)
        if isbool(c)
            push!(atoms, ("q$k", c isa Bool ? (c ? [] : nothing) : [(c, true)]))
        elseif query == :pr
            bits = queried(c, query)
            length(bits) <= MAX_WORLD_BITS ||
                error("component $k has $(length(bits)) bits that are not constant; at most $MAX_WORLD_BITS are queried as worlds")
            n = length(bits)
            for u in 0:(2^n - 1)
                push!(atoms, ("q$k($u)", [(bits[i], (u >> (n - i)) & 1 == 1) for i in 1:n]))
            end
        else
            seen = Set{Dist{Bool}}()
            for (j, b) in enumerate(allbits(c))
                (b isa Dist{Bool} && !(b in seen)) || continue
                push!(seen, b)
                push!(atoms, ("e$k($j)", [(b, true)]))
            end
        end
    end
    atoms
end

# The ProbLog program of the model's circuit, to `io`; `source` names the
# program in its first line.
function write_problog(io, model::Model, query, source)
    atoms = query_atoms(model, query)
    roots = Any[n for (_, conj) in atoms if conj !== nothing for (n, _) in conj]
    append!(roots, model.evidence)
    c = circuit(roots, flip_numbers(model, query))
    lit(n, pos = true) = begin
        name, neg = literal(c, n, !pos)
        neg ? "\\+$name" : name
    end
    println(io, "% $source: the circuit Alea.jl builds, written by tddlog's scripts/alea2problog.jl.")
    fs, gs = flips(c), gates(c)
    println(io, "% $(length(fs)) flips, $(length(gs)) gates, $(length(model.evidence)) observations, $(length(atoms)) queries.")
    for f in fs
        println(io, prob_text(f.prob), "::", c.names[f], ".")
    end
    for g in gs
        if g isa DistAnd
            println(io, c.names[g], " :- ", lit(g.x), ", ", lit(g.y), ".")
        else
            println(io, c.names[g], " :- ", lit(g.x), ".")
            println(io, c.names[g], " :- ", lit(g.y), ".")
        end
    end
    for (k, o) in enumerate(model.evidence)
        if o isa Bool
            o || println(io, "evidence(false_observation, true).")
            continue
        end
        name, neg = literal(c, o, false)
        println(io, "evidence($name, $(neg ? "false" : "true")).")
    end
    for (atom, conj) in atoms
        if conj === nothing
            println(io, atom, " :- fail.")
        elseif isempty(conj)
            println(io, atom, ".")
        elseif !(length(conj) == 1 && conj[1][2] && lit(conj[1][1]) == atom)
            println(io, atom, " :- ", join([lit(n, pos) for (n, pos) in conj], ", "), ".")
        end
        println(io, "query(", atom, ").")
    end
end

function main(args)
    if length(args) != 1 || startswith(args[1], "-")
        println(stderr, USAGE)
        exit(2)
    end
    src = abspath(args[1])
    query, ignore_errors = load(read(src, String), src)
    # Written whole at the end: a pipe takes each `println` as a write.
    io = IOBuffer()
    write_problog(io, build(ignore_errors), query, basename(src))
    write(stdout, take!(io))
    flush(stdout)
end

end # module Alea2ProbLog

# Run as a script, on a task with a 1 GiB stack, for a program whose model()
# recurses deeply.
if abspath(PROGRAM_FILE) == @__FILE__
    task = Task(() -> Alea2ProbLog.main(ARGS), 1 << 30)
    schedule(task)
    wait(task)
end
