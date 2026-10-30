# TensorInference.jl (exact tensor network contraction) on a UAI network,
# for `bench.py run --tensorinference`. Usage:
#   julia --project=DIR tensorinference_uai.jl MODEL.uai EVID OUT MAR|PR [I,J,...|@FILE]
# OUT is in Merlin's format (scripts/bn_uai_run.py describes it); MAR gives
# the marginals of the listed variables, inline or in FILE (all by
# default); the environment
# variable BN_VARIANT is `treesa` (default) or `greedy`. Julia is timed warm,
# as alea_bench.jl times Alea.jl: a small model goes
# through the same calls first, so that their common methods are compiled
# before the instance is timed; but the package compiles methods anew for
# the instance's tensors, which can take most of the time, and so the
# instance is solved a second time where the first took less than
# BN_WARM_BELOW seconds (bench.py passes a third of the limit; every time
# where it is not set). The last lines are `inference_seconds X`, the
# instance's time from reading its files to the answers (the contraction
# order's search included), `compile_seconds C`, Julia's compilation within
# it, `warm_seconds Y`, the second solve's time, where there was one, and
# `warmup_seconds W`, the small model's.
using TensorInference

# The instance's work, from reading its files to its answers.
function solve(model_path, evid_path, task, optimizer)
    tok = split(read(evid_path, String))
    k = isempty(tok) ? 0 : parse(Int, tok[1])
    evid = Dict{Int,Int}(parse(Int, tok[2 + 2i]) + 1 => parse(Int, tok[3 + 2i]) for i in 0:k-1)
    model = read_model_file(model_path)
    tn = TensorNetworkModel(model; evidence = evid, optimizer = optimizer)
    lz = log_probability(tn)
    lz = lz isa AbstractArray ? only(lz) : lz
    ok = isfinite(lz)
    mar = (task == "MAR" && ok) ? marginals(tn) : nothing
    return model.cards, evid, lz, ok, mar
end

# The warm-up's model: a Markov network of three binary variables, the
# last one observed.
const WARMUP_UAI = """MARKOV
3
2 2 2
3
1 0
2 0 1
2 1 2
2
0.4 0.6
4
0.1 0.2 0.3 0.4
4
0.5 0.25 0.125 0.125
"""
const WARMUP_EVID = "1 2 1\n"

function warm_up(dir, task, optimizer)
    model, evid = joinpath(dir, "warmup.uai"), joinpath(dir, "warmup.evid")
    write(model, WARMUP_UAI)
    write(evid, WARMUP_EVID)
    solve(model, evid, task, optimizer)
    rm(model)
    rm(evid)
end

function main(model_path, evid_path, out_path, task, vars)
    # BN_VARIANT: the contraction order by simulated annealing (the package's default) or greedily
    optimizer = get(ENV, "BN_VARIANT", "treesa") == "greedy" ? GreedyMethod() : TreeSA()
    warm = @elapsed warm_up(dirname(abspath(out_path)), task, optimizer)
    run1 = @timed solve(model_path, evid_path, task, optimizer)
    sizes, evid, lz, ok, mar = run1.value
    below = parse(Float64, get(ENV, "BN_WARM_BELOW", "Inf"))
    again = run1.time < below ? (@timed solve(model_path, evid_path, task, optimizer)).time : nothing
    listed = startswith(vars, "@") ? strip(read(vars[2:end], String)) : vars
    qvars = isempty(listed) ? collect(1:length(sizes)) : [parse(Int, x) + 1 for x in split(listed, ",")]
    open(out_path, "w") do io
        println(io, "PR")
        println(io, "$(lz) ($(ok ? exp(lz) : 0.0))")
        println(io, "STATUS")
        println(io, ok ? "true" : "false")
        println(io, "MAR")
        if mar === nothing
            println(io, 0)
        else
            println(io, length(qvars))
            for v in qvars
                row = if haskey(evid, v)
                    [u == evid[v] + 1 ? 1.0 : 0.0 for u in 1:sizes[v]]
                elseif haskey(mar, [v])
                    p = mar[[v]]; p ./ sum(p)
                else
                    fill(1.0 / sizes[v], sizes[v])
                end
                println(io, sizes[v], " ", join(row, " "))
            end
        end
        println(io, "inference_seconds $(run1.time)")
        println(io, "compile_seconds $(get(run1, :compile_time, NaN))")
        again === nothing || println(io, "warm_seconds $(again)")
        println(io, "warmup_seconds $(warm)")
    end
end

main(ARGS[1], ARGS[2], ARGS[3], ARGS[4], length(ARGS) > 4 ? ARGS[5] : "")
