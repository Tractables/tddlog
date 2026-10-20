# pluck_bench.jl: Pluck (Bowers et al., PLDI 2025) on a sequence model of the
# paper's artifact, as bench.py's `--pluck` runs it:
#
#   julia --project=ART/PluckArtifact.jl/Pluck.jl pluck_bench.jl ART MODEL
#
# ART is the artifact's directory (Zenodo 10.5281/zenodo.15051140, installed
# by docs/baselines.md), MODEL one of hmm, pcfg, string_editing,
# sorted_list. The program is the artifact's own, src/table1/
# sequence_models/pluck/MODEL.jl, included as the artifact's harness
# includes it; only the harness's registry (add_benchmark!, PluckBenchmark)
# is replaced by the few lines below, so that the artifact's plotting and
# profiling packages and Dice.jl are not needed. It runs the artifact's
# `pluck_default` benchmark as the artifact's `ours` strategy does:
# `bdd_forward` of the parsed query, and prints
#
#   pluck_v VALUE PROBABILITY     for each value of the query
#   pluck_seconds S               the first run, with Julia's compilation
#   pluck_warm_seconds S          a second run, when the first took under 30 s
using Pluck

mutable struct PluckBenchmark
    query::String
    normalize::Bool
    pre::Union{Function, Nothing}
    PluckBenchmark(query; normalize=false, pre=nothing, kwargs...) = new(query, normalize, pre)
end
const BENCHMARKS = Dict{Tuple{String,String}, PluckBenchmark}()
add_benchmark!(baseline, group, b) = (BENCHMARKS[(baseline, group)] = b)
macro bbtime(args...)
    :(error("@bbtime is the artifact's harness's"))
end

art, model = ARGS
include(joinpath(art, "PluckArtifact.jl", "src", "table1", "sequence_models", "pluck", model * ".jl"))
b = BENCHMARKS[(model, "pluck_default")]
b.pre === nothing || b.pre()
expr = Pluck.parse_expr(b.query)
run_once() = b.normalize ? Pluck.bdd_normalize(Pluck.bdd_forward(expr)) : Pluck.bdd_forward(expr)

t1 = @elapsed res = run_once()
for (v, p) in res
    name = hasproperty(v, :constructor) ? string(v.constructor) : string(v)
    println("pluck_v ", name, " ", p)
end
println("pluck_seconds ", t1)
if t1 < 30
    GC.gc()
    println("pluck_warm_seconds ", @elapsed run_once())
end
