# alea_bench.jl: run an Alea.jl program, or write the Boolean circuit Alea
# builds for it as a ProbLog program and as Dice programs.
#
#     julia --project=ALEA alea_bench.jl run [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl [PREFIX]
#     julia --project=ALEA alea_bench.jl export [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl PREFIX
#
# PROGRAM.jl is a program in the form of data/alea/: Julia code, evaluated
# in a module of its own, that defines `model()`, returning what an
# `@alea begin ... end` block returns (a MetaDist) or a plain distribution,
# and `QUERY`, `:pr` or `:expectation`; `IGNORE_ERRORS = true` drops Alea's
# probabilistic errors from the inference, as `pr(...; ignore_errors =
# true)` does. The program's value is a Boolean, a number (DistUInt,
# DistInt, DistFix) or a tuple of them; each component is answered on its
# own, as `pr(c1, c2, ...)` answers them (a returned tuple's joint
# distribution is not computed). `--set NAME=VALUE` runs the program with
# its line `const NAME = ...` reading `const NAME = VALUE` (a size, such
# as the bits of a fixed-point number); the program must have exactly one
# such line.
#
# Alea represents every Boolean of a program as a DAG of flips, &, | and !,
# and a number as a vector of such Booleans (its bits, the most significant
# first); `pr` compiles the bits queried and the conjunction of the
# observations to one CUDD BDD, its variables the flips in the order they
# were created, and enumerates the worlds of the queried bits. The answers
# are named after ProbLog atoms, the same in every mode:
#
#   - a Boolean component k: `qK`, its probability of being true;
#   - under `:pr`, a number component k whose bits that are not constant
#     are b_1 ... b_n (most significant first): `qK(U)` for every
#     U = sum_i b_i 2^(n-i) in 0 .. 2^n - 1, the probability of that world
#     of its bits (U names a value of the number; `run` prints the value);
#   - under `:expectation`, a number component k: `eK(I)`, the probability
#     that its I-th bit is true, for each bit I that is not constant. The
#     expectation is linear in these, and Alea's `expectation` computes it
#     from them (`pr` of the bits).
#
# `run` builds the program and answers it with Alea's `pr`, as the program
# would, and prints `alea_q ATOM P` for every answer, `alea_value ATOM V`
# for the value a world names, `alea_expectation K E` for an expectation,
# then `alea_seconds T`, the time taken to build the program and run `pr`,
# and `alea_compile_seconds T`, Julia's compilation within it. Before it,
# a small program is run, so that the time leaves out the compilation of
# Alea's common methods; but under `@alea` Alea compiles its methods anew
# for the program, which can take most of the time. The program is then
# built and answered a second time, and `alea_warm_seconds T` is that
# time, without compilation, as the papers measure Alea (the median of
# BenchmarkTools' runs); under `--warm-below S`, only if the first took
# less than S seconds. Where the second took less than S/2, `run` then
# prints the sizes of the BDDs `pr` compiles, counted in a third
# compilation: `alea_bdd_evidence N` (the evidence's DAG),
# `alea_bdd_query_max N` (the largest DAG of a queried bit),
# `alea_bdd_joint_max N` (the largest of a queried bit and the evidence)
# and `alea_bdd_peak N` (the CUDD manager's peak node count). With PREFIX,
# `run` also writes the circuit of the first build, as `export` does. The
# work runs on a task with a 1 GiB stack, as Alea's compilation recurses
# as deep as the circuit.
#
# `export` builds the program and writes its circuit, without inference.
# PREFIX.pl is the ProbLog program that the source tree's
# scripts/alea2problog.jl writes of the program run with the `--set`s,
# whose first line names them: a flip is a probabilistic fact `P::fN.`
# (the N-th flip of the circuit in the order Alea created them), an `&` a
# rule, a `|` two rules, a `!` a negated literal, every observation
# `evidence(...)`, every answer a query, `qK(U)` defined by the
# conjunction of its world's bit literals. alea2problog.jl also builds the
# model that `run` answers.
# Under `--warm-below S`, `export` is timed as `run` times Alea, for the
# time a reader of the circuit pays for it: after the small program,
# `alea_seconds T` and `alea_compile_seconds T` are those of building the
# program and writing PREFIX.pl, and `alea_warm_seconds T` that of doing
# both a second time, without compilation, if the first took less than S.
# The Dice programs, a translation of the circuit for one family of
# systems, are written outside the time.
# The probabilistic errors of a program not run with IGNORE_ERRORS are
# left out: `run` stops (Alea throws) where an error has a positive
# probability.
#
# PREFIX__dJ.dice are Dice programs over the same circuit: the flips
# first, in the order Alea creates them (so Dice's BDD has Alea's variable
# order), then the gates and the observations as `let`s. A Dice program
# returns one expression and prints its distribution, a tuple's as a joint
# table, so each Boolean component is a program, each `:pr` number
# component a program returning its bits as a tuple, and each bit of an
# `:expectation` component a program. PREFIX.dicemap lists them, a line
# each: `FILE bool ATOM` (the probability of true answers ATOM) or `FILE
# joint PRED N` (N bits returned; the row of world U answers PRED(U)).

using Alea
import CUDD
using Alea: JointQuery, MetaDist, Flip, DistAnd, DistOr, DistNot, CondError,
    returnvalue, allobservations, tobits

# The program's model and its circuit, as the source tree's
# scripts/alea2problog.jl builds and writes them: beside this script on the
# baselines branch and in a flat copy, in tddlog/scripts/ in a checkout that
# holds the source tree there (as scripts/source_tree.py finds it).
include(let own = joinpath(@__DIR__, "alea2problog.jl")
    isfile(own) ? own : joinpath(dirname(@__DIR__), "tddlog", "scripts", "alea2problog.jl")
end)
using .Alea2ProbLog: Model, load, build, allbits, isbool, queried, MAX_WORLD_BITS,
    circuit, flip_numbers, flips, gates, literal, prob_text, write_problog

const USAGE = "usage: alea_bench.jl run [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl [PREFIX] | export [--warm-below S] [--set NAME=VALUE ...] PROGRAM.jl PREFIX"

# The value a world of a number's bits names, for display.
value(x, world) = Alea.frombits(x, world)

# The answers of every component, by Alea: `pr` with one joint query per
# component under `:pr`, and one query per bit under `:expectation`, as
# Alea's `pr(c1, c2, ...)` and `expectation` make them.
# The queries `pr` answers for a program: per component a JointQuery of a
# Boolean, of the bits of a `:pr` number that are not constant, or one per
# bit of an `:expectation` number; and the shape of each component.
function queries_of(model::Model, query)
    queries = JointQuery[]
    shape = []   # per component: (:bool | :pr | :bits, first query, count)
    for c in model.comps
        if isbool(c)
            push!(shape, (:bool, length(queries) + 1, 1))
            push!(queries, JointQuery(tobits(c)))
        elseif query == :pr
            push!(shape, (:pr, length(queries) + 1, 1))
            push!(queries, JointQuery(Dist{Bool}[b for b in allbits(c) if b isa Dist{Bool}]))
        else
            bits = allbits(c)
            push!(shape, (:bits, length(queries) + 1, length(bits)))
            append!(queries, [JointQuery(tobits(b)) for b in bits])
        end
    end
    queries, shape
end

function answer(model::Model, query)
    evidence = reduce(&, model.evidence; init = true)
    queries, shape = queries_of(model, query)
    worlds = pr(queries; evidence, errors = model.errors)
    shape, worlds
end

# The sizes of the BDDs `pr` compiles, counted in a compilation of the
# same roots in the same order (the queried bits, the error conditions,
# the evidence; the flips in creation order, no reordering): the
# evidence's DAG, the largest DAG of a queried bit, the largest of a
# queried bit and the evidence, and the manager's peak node count.
function bdd_sizes(model::Model, query)
    evidence = reduce(&, model.evidence; init = true)
    queries, _ = queries_of(model, query)
    bits = AnyBool[b for q in queries for b in q.bits]
    errors = AnyBool[e[1] for e in model.errors]
    c = Alea.BDDCompiler(vcat(bits, errors, AnyBool[evidence]))
    e = Alea.compile(c, evidence)
    qs = [Alea.compile(c, b) for b in bits]
    largest = maximum((CUDD.Cudd_DagSize(q) for q in qs); init = 0)
    joint = maximum((CUDD.Cudd_DagSize(CUDD.Cudd_bddAnd(c.mgr, e, q)) for q in qs); init = 0)
    for err in errors
        Alea.compile(c, err)
    end
    println("alea_bdd_evidence ", CUDD.Cudd_DagSize(e))
    println("alea_bdd_query_max ", largest)
    println("alea_bdd_joint_max ", joint)
    println("alea_bdd_peak ", CUDD.Cudd_ReadPeakNodeCount(c.mgr))
    flush(stdout)
end

prob_true(b::Bool, worlds) = b ? 1.0 : 0.0
prob_true(b::Dist{Bool}, worlds) = sum((p for (w, p) in worlds if w[b]); init = 0.0)

function report(model::Model, query, shape, worlds)
    for (k, c) in enumerate(model.comps)
        kind, i, n = shape[k]
        if kind == :bool
            println("alea_q q$k ", prob_true(c, worlds[i]))
        elseif kind == :pr
            bits = queried(c, query)
            dist = Dict{Int, Float64}()
            for (w, p) in worlds[i]
                u = 0
                for b in bits
                    u = 2u + (w[b] ? 1 : 0)
                end
                dist[u] = get(dist, u, 0.0) + p
                println("alea_value q$k($u) ", value(c, w))
            end
            for u in 0:(2^length(bits) - 1)
                println("alea_q q$k($u) ", get(dist, u, 0.0))
            end
        else
            bits = allbits(c)
            probs = [prob_true(b, worlds[i + j - 1]) for (j, b) in enumerate(bits)]
            seen = Set{Dist{Bool}}()
            for (j, b) in enumerate(bits)
                (b isa Dist{Bool} && !(b in seen)) || continue
                push!(seen, b)
                println("alea_q e$k($j) ", probs[j])
            end
            println("alea_expectation $k ", expectation_of(c, probs))
        end
    end
end

# The expectation from the bits' probabilities, as Alea computes it.
function expectation_of(c, probs)
    W = length(probs)
    e = sum(probs[j] * 2.0^(W - j) for j in 1:W)
    if c isa DistInt || c isa DistFix
        e -= probs[1] * 2.0^W
    end
    c isa DistFix ? e / 2.0^typeof(c).parameters[2] : e
end

function warmup()
    a = flip(0.7)
    x = discrete(DistUInt{2}, [0.1, 0.2, 0.3, 0.4])
    y = uniform(DistUInt{3}, 1, 6)
    z = ifelse(a, x + DistUInt{2}(1), x - DistUInt{2}(1))
    e = prob_equals(z, DistUInt{2}(2)) | (x < z) | xor(a, flip(0.2))
    pr(a, z, y; evidence = e)
    m = @alea begin
        b = flip(0.4)
        observe(b | flip(0.6))
        b
    end
    pr(m)
    expectation(uniform(DistFix{6, 2}, 0.0, 2.0))
    nothing
end

function run_program(query, ignore_errors)
    stats = @timed begin
        model = build(ignore_errors)
        shape, worlds = answer(model, query)
        (model, shape, worlds)
    end
    model, shape, worlds = stats.value
    report(model, query, shape, worlds)
    println("alea_seconds ", stats.time)
    println("alea_compile_seconds ", get(stats, :compile_time, NaN))
    flush(stdout)
    model, stats.time
end

# Dice: one program per answer group, each with the whole circuit the
# group and the observations need.
function write_dice(prefix, model::Model, query, source)
    flipno = flip_numbers(model, query)
    groups = []   # (kind, atom or pred, Booleans returned)
    for (k, c) in enumerate(model.comps)
        if isbool(c)
            push!(groups, (:bool, "q$k", c))
        elseif query == :pr
            bits = queried(c, query)
            length(bits) <= MAX_WORLD_BITS ||
                error("component $k has $(length(bits)) bits that are not constant")
            push!(groups, (:joint, "q$k", bits))
        else
            seen = Set{Dist{Bool}}()
            for (j, b) in enumerate(allbits(c))
                (b isa Dist{Bool} && !(b in seen)) || continue
                push!(seen, b)
                push!(groups, (:bool, "e$k($j)", b))
            end
        end
    end
    open(prefix * ".dicemap", "w") do map
        for (j, (kind, atom, ret)) in enumerate(groups)
            file = "$(basename(prefix))__d$j.dice"
            roots = Any[(ret isa AbstractVector ? ret : [ret])...]
            append!(roots, model.evidence)
            c = circuit(roots, flipno)
            lit(n) = begin
                name, neg = literal(c, n, false)
                neg ? "!$name" : name
            end
            open(joinpath(dirname(prefix), file), "w") do io
                println(io, "// $source: the circuit Alea.jl builds, written by tddlog's scripts/alea_bench.jl.")
                for f in flips(c)
                    println(io, "let ", c.names[f], " = flip ", prob_text(f.prob), " in")
                end
                for g in gates(c)
                    op = g isa DistAnd ? " && " : " || "
                    println(io, "let ", c.names[g], " = ", lit(g.x), op, lit(g.y), " in")
                end
                for o in model.evidence
                    o isa Bool ? println(io, "let _ = observe ", o, " in") : println(io, "let _ = observe ", lit(o), " in")
                end
                if ret isa AbstractVector
                    isempty(ret) && println(io, "true")
                    lits = [lit(b) for b in ret]
                    expr = lits[end]
                    for l in reverse(lits[1:end-1])
                        expr = "($l, $expr)"
                    end
                    isempty(ret) || println(io, expr)
                else
                    println(io, ret isa Bool ? string(ret) : lit(ret))
                end
            end
            if kind == :bool
                println(map, file, " bool ", atom)
            else
                println(map, file, " joint ", atom, " ", length(ret))
            end
        end
    end
end

function main()
    args = copy(ARGS)
    # `--warm-below S`: build and answer the program (`run`), or build it
    # and write its circuit (`export`), a second time only if the first
    # took less than S seconds; `export` is timed only under it.
    warm_below = nothing
    i = findfirst(==("--warm-below"), args)
    if i !== nothing
        i < length(args) || error(USAGE)
        warm_below = parse(Float64, args[i + 1])
        deleteat!(args, i:i+1)
    end
    # `--set NAME=VALUE`, any number of times.
    sets = Pair{String, String}[]
    while (i = findfirst(==("--set"), args)) !== nothing
        i < length(args) || error(USAGE)
        m = match(r"^([A-Za-z_][A-Za-z0-9_]*)=(.+)$", args[i + 1])
        m === nothing && error("--set takes NAME=VALUE, not $(args[i + 1])")
        push!(sets, m[1] => m[2])
        deleteat!(args, i:i+1)
    end
    length(args) in (2, 3) || error(USAGE)
    mode, src = args[1], abspath(args[2])
    prefix = length(args) == 3 ? abspath(args[3]) : nothing
    text = read(src, String)
    for (name, value) in sets
        line = Regex("^const $name = .*\$", "m")
        n = length(collect(eachmatch(line, text)))
        n == 1 || error("--set $name: $(basename(src)) has $n lines `const $name = ...`, not one")
        text = replace(text, line => "const $name = $value")
    end
    query, ignore_errors = load(text, src)
    source = join([basename(src); ["$name=$value" for (name, value) in sets]], " ")
    if mode == "run"
        warmup()
        model, seconds = run_program(query, ignore_errors)
        prefix === nothing || export_model(prefix, model, query, source)
        below = something(warm_below, Inf)
        if seconds < below
            stats = @timed answer(build(ignore_errors), query)
            println("alea_warm_seconds ", stats.time)
            # The BDDs' sizes take about the time of a warm run again: only
            # where that took less than half of S.
            2 * stats.time < below && bdd_sizes(build(ignore_errors), query)
        end
    elseif mode == "export" && prefix !== nothing && warm_below === nothing
        export_model(prefix, build(ignore_errors), query, source)
    elseif mode == "export" && prefix !== nothing
        warmup()
        circuit_of = () -> (m = build(ignore_errors); write_circuit(prefix, m, query, source); m)
        stats = @timed circuit_of()
        write_dice(prefix, stats.value, query, source)
        println("alea_seconds ", stats.time)
        println("alea_compile_seconds ", get(stats, :compile_time, NaN))
        stats.time < warm_below && println("alea_warm_seconds ", (@timed circuit_of()).time)
    else
        error(USAGE)
    end
end

write_circuit(prefix, model, query, source) =
    open(io -> write_problog(io, model, query, source), prefix * ".pl", "w")

function export_model(prefix, model, query, source)
    write_circuit(prefix, model, query, source)
    write_dice(prefix, model, query, source)
end

task = Task(main, 1 << 30)
schedule(task)
wait(task)
