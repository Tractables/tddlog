# Ported from PluckArtifact.jl/src/table1/sequence_models/dice/sorted_list.jl of
# the artifact of Bowers et al., "Stochastic Lazy Knowledge Compilation for
# Inference in Discrete Probabilistic Programs" (PLDI 2025), Zenodo
# 10.5281/zenodo.15051140 (CC-BY-4.0): the Dice.jl baseline of the paper's
# Table 1, row Sorted List. Mechanical port: Dice.List, Dice.Nil and Dice.Cons ->
# Alea's List, Nil and Cons, @dice_ite -> @alea_ite, the add_benchmark!
# harness dropped, `pr(E)` -> E, the query moved to QUERY and model().
using Alea

const QUERY = :pr



function gen_sorted_nat(min_val, fuel)
    fuel == 0 && return min_val
    @alea_ite if flip(0.5)
        min_val
    else
        Nat.S(gen_sorted_nat(min_val, fuel-1))
    end
end

function gen_sorted_list(max_size, min_val, fuel)
    max_size == 0 && return Nil(Nat.t)
    @alea_ite if flip(0.5)
        Nil(Nat.t)
    else
        let hd = gen_sorted_nat(min_val, fuel)
            Cons(Nat.t, hd, gen_sorted_list(max_size-1, hd, fuel))
        end
    end
end

function nats_equal(n1::Nat.t, n2::Nat.t)
    match(n1, [
        :Z => () -> match(n2, [:Z => () -> true, :S => (n2pred) -> false]),
        :S => (n1pred) -> match(n2, [:S => (n2pred) -> nats_equal(n1pred, n2pred), :Z => () -> false]),
    ])
end

function lists_equal(l1::List{Nat.t}, l2::List{Nat.t})
    match(l1, [
        :Nil => () -> match(l2, [:Nil => () -> true, :Cons => (hd, tl) -> false]),
        :Cons => (x, xs) -> match(l2, [:Cons => (y, ys) -> (@alea_ite if (nats_equal(x, y)); lists_equal(xs, ys); else; false; end), :Nil => () -> false]),
    ])
end


make_nat(n) = if n == 0; Nat.Z(); else Nat.S(make_nat(n-1)); end

make_list(l) = if isempty(l); Nil(Nat.t); else Cons(Nat.t, make_nat(l[1]), make_list(l[2:end])); end

l = [0, 3, 7, 12, 13, 15, 16, 20]
test_list = make_list(l);

model() = lists_equal(gen_sorted_list(9, Nat.Z(), 6), make_list(l))
