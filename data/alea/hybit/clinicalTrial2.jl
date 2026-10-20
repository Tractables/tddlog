# clinicaltrial2 (Gehr et al. 2016): the success probability of the
# control group in a trial of five patients a group, given that the
# treatment is effective. Bit-blasting paper (Garg, Holtzen, Van den
# Broeck, Millstein, PLDI 2024), Table 2: 12 bits. From Tractables/Alea.jl
# hybrid/benchmarks/clinicalTrial2/clinicalTrial2.jl at cf5e5b8 (Apache
# License 2.0), its body unchanged.
using Alea, Distributions

const QUERY = :expectation
const bits = 10
const DFiP = DistFix{2+bits, bits}
const controlGroup = [false, false, true, false, false]
const treatedGroup = [true, false, true, true, true]

model() = @alea begin
    isEffective = flip(0.5)
    probIfTreated = uniform(DFiP, 0.0, 1.0)
    probIfControl = if isEffective uniform(DFiP, 0.0, 1.0) else probIfTreated end
    for i=1:5
        controlFlip = parametrised_flip(probIfControl)
        observe(controlFlip == controlGroup[i])
        treatedFlip = parametrised_flip(probIfTreated)
        observe(treatedFlip == treatedGroup[i])
    end
    observe(isEffective)
    probIfControl
end
