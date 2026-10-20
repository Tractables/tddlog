# TensorInference.jl (exact tensor network contraction) on a UAI network,
# for `bench.py run --tensorinference`. Usage:
#   julia --project=DIR tensorinference_uai.jl MODEL.uai EVID OUT MAR|PR [I,J,...]
# OUT is in Merlin's format (scripts/bn_uai_run.py describes it); MAR gives
# the marginals of the listed variables (all by default); the environment
# variable BN_VARIANT is `treesa` (default) or `greedy`. The last line is
# `inference_seconds X`: building the contraction order and contracting,
# after the packages are loaded.
using TensorInference

function main(model_path, evid_path, out_path, task, vars)
    tok = split(read(evid_path, String))
    k = isempty(tok) ? 0 : parse(Int, tok[1])
    evid = Dict{Int,Int}(parse(Int, tok[2 + 2i]) + 1 => parse(Int, tok[3 + 2i]) for i in 0:k-1)
    model = read_model_file(model_path)
    sizes = model.cards
    qvars = isempty(vars) ? collect(1:length(sizes)) : [parse(Int, x) + 1 for x in split(vars, ",")]
    t0 = time()
    # BN_VARIANT: the contraction order by simulated annealing (the package's default) or greedily
    optimizer = get(ENV, "BN_VARIANT", "treesa") == "greedy" ? GreedyMethod() : TreeSA()
    tn = TensorNetworkModel(model; evidence = evid, optimizer = optimizer)
    lz = log_probability(tn)
    lz = lz isa AbstractArray ? only(lz) : lz
    ok = isfinite(lz)
    mar = (task == "MAR" && ok) ? marginals(tn) : nothing
    secs = time() - t0
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
        println(io, "inference_seconds $(secs)")
    end
end

main(ARGS[1], ARGS[2], ARGS[3], ARGS[4], length(ARGS) > 4 ? ARGS[5] : "")
