# Ported from PluckArtifact.jl/src/table1/sequence_models/dice/hmm.jl of
# the artifact of Bowers et al., "Stochastic Lazy Knowledge Compilation for
# Inference in Discrete Probabilistic Programs" (PLDI 2025), Zenodo
# 10.5281/zenodo.15051140 (CC-BY-4.0): the Dice.jl baseline of the paper's
# Table 1, row HMM. Mechanical port: Dice.List, Dice.Nil and Dice.Cons ->
# Alea's List, Nil and Cons, @dice_ite -> @alea_ite, the add_benchmark!
# harness dropped, `pr(E)` -> E, the query moved to QUERY and model().
using Alea

const QUERY = :pr


function dice_hmm_example(N)

    function make_bool_list(n)
        n == 0 && return Nil(AnyBool)
        Cons(AnyBool, true, make_bool_list(n-1))
    end

    function hmm(z0, num_steps)

        if num_steps == 0
            return Nil(AnyBool)
        end

        z1 = @alea_ite if z0 
            flip(0.7)
        else
            flip(0.1)
        end

        x1 = @alea_ite if z1 
            flip(0.3)
        else
            flip(0.6)
        end

        return Cons(AnyBool, x1, hmm(z1, num_steps - 1))
    end
        
    function bool_lists_equal(l1::List{AnyBool}, l2::List{AnyBool})
        match(l1, [:Nil => () -> match(l2, [:Nil => () -> true, :Cons => (hd, tl) -> false]), :Cons => (x, xs) -> match(l2, [:Cons => (y, ys) -> (@alea_ite if (prob_equals(x, y)); bool_lists_equal(xs, ys); else; false; end), :Nil => () -> false])])
    end 

    return bool_lists_equal(hmm(false, N), make_bool_list(N))
end

model() = dice_hmm_example(50)
