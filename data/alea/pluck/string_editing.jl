# Ported from PluckArtifact.jl/src/table1/sequence_models/dice/string_editing.jl of
# the artifact of Bowers et al., "Stochastic Lazy Knowledge Compilation for
# Inference in Discrete Probabilistic Programs" (PLDI 2025), Zenodo
# 10.5281/zenodo.15051140 (CC-BY-4.0): the Dice.jl baseline of the paper's
# Table 1, row String Editing. Mechanical port: Dice.List, Dice.Nil and Dice.Cons ->
# Alea's List, Nil and Cons, @dice_ite -> @alea_ite, the add_benchmark!
# harness dropped, `pr(E)` -> E, the query moved to QUERY and model().
using Alea

const QUERY = :pr

@inductive PerturbChar a_() b_() c_() d_() e_()

function random_char()
    @alea_ite if flip(0.2)
        a_()
    elseif flip(0.25)
        b_()
    elseif flip(0.33333)
        c_()
    elseif flip(0.5)
        d_()
    else
        e_()
    end
end

function random_string(p, max_length)
    if max_length == 0
        return Nil(PerturbChar)
    else 
        @alea_ite if flip(p)
            Cons(PerturbChar, random_char(), random_string(p, max_length-1))
        else
            Nil(PerturbChar)
        end
    end
end

function list_append(l1, l2)
    match(l1, [:Nil => () -> l2, :Cons => (hd, tl) -> Cons(PerturbChar, hd, list_append(tl, l2))])
end

function perturb(s, fuel, max_insert_length)

  fuel == 0 && return s

  insertion = random_string(0.01, max_insert_length)

  match(s, [:Nil => () -> insertion, 
            :Cons => (hd, tl) -> let perturbed_tail = perturb(tl, fuel-1, max_insert_length); (@alea_ite if flip(0.99); list_append(insertion, Cons(PerturbChar, hd, perturbed_tail)); else; perturbed_tail; end); end])

end


function list_symbols_equal(l1::List{PerturbChar}, l2::List{PerturbChar}, fuel::Int)
    fuel == 0 && return false
    match(l1, [:Nil => () -> match(l2, [:Nil => () -> true, :Cons => (hd, tl) -> false]), :Cons => (hd1, tl1) -> match(l2, [:Nil => () -> false, :Cons => (hd2, tl2) -> (@alea_ite if prob_equals(hd1, hd2); list_symbols_equal(tl1, tl2, fuel-1); else; false; end)])])
end

function make_string_for_perturb(s)
    if s == ""
        return Nil(PerturbChar)
    else
        chars = Dict(["a" => a_(), "b" => b_(), "c" => c_(), "d" => d_(), "e" => e_()])
        return Cons(PerturbChar, chars[s[1:1]], make_string_for_perturb(s[2:end]))
    end
end

long_string_1 = "edccdadbedadbbacddbecaedddaabdccbadadeaecbadcacedbdeeaadadcbeadedbdaebccdaeaaadabecdcadebaddbbdbceea"
long_string_2 = "edddecbcadccbeecedbabbcbbcaeebeebaabddbcbcaccdaddceeddedddbabdabceceabaccbbbcbcacceceaecacdddbbdaedb"

function perturb_example(n, m)
    before_string = make_string_for_perturb(long_string_1[1:n])
    after_string =  make_string_for_perturb(long_string_2[1:m])
    return list_symbols_equal(perturb(before_string, n+1, m+1), after_string, max(n+1, m+1))
end

model() = perturb_example(4, 5)
