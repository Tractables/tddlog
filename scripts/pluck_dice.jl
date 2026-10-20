# pluck_dice.jl: Pluck (Bowers et al., PLDI 2025) on a program of
# dice2pluck.py, a Dice program translated into Pluck's language, as
# bench.py's `--pluck` runs it on the Dice programs:
#
#   julia --project=ART/PluckArtifact.jl/Pluck.jl pluck_dice.jl ART FILE
#
# ART is the artifact's directory (Zenodo 10.5281/zenodo.15051140, installed
# by docs/baselines.md). The file is loaded with Pluck's load_pluck_file,
# which runs each query (cK, component K of the Dice program's value) and
# prints its distribution, a line `VALUE PROBABILITY` per value of nonzero
# probability. It runs in a task with a large stack: Pluck's evaluator
# recurses as deep as the program's bindings and loops nest. Then
#
#   pluck_seconds S      load_pluck_file's time, Pluck's compilation in it
using Pluck

art, file = ARGS
task = Task(() -> load_pluck_file(file), 1 << 30)
t = @elapsed begin
    schedule(task)
    wait(task)
end
println("pluck_seconds ", t)
