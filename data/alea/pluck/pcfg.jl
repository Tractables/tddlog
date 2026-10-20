# Ported from PluckArtifact.jl/src/table1/sequence_models/dice/pcfg.jl of
# the artifact of Bowers et al., "Stochastic Lazy Knowledge Compilation for
# Inference in Discrete Probabilistic Programs" (PLDI 2025), Zenodo
# 10.5281/zenodo.15051140 (CC-BY-4.0): the Dice.jl baseline of the paper's
# Table 1, row PCFG. Mechanical port: Dice.List, Dice.Nil and Dice.Cons ->
# Alea's List, Nil and Cons, @dice_ite -> @alea_ite, the add_benchmark!
# harness dropped, `pr(E)` -> E, the query moved to QUERY and model().
using Alea

const QUERY = :pr


@inductive Nonterminal SS() XX() YY()
@inductive Terminal aa() bb() cc()

function prob_list_append(l1::List{Terminal}, l2::List{Terminal}, fuel::Int)
    fuel == 0 && return Nil(Terminal)
    match(l1, [:Nil => () -> l2, :Cons => (hd, tl) -> Cons(Terminal, hd, prob_list_append(tl, l2, fuel-1))])
end


function generate_from_grammar(sym::Union{Dist{Nonterminal}, Nonterminal}, fuel::Int, append_fuel::Int)
    fuel == 0 && return Nil(Terminal)

    match(sym, [:XX => () -> (@alea_ite if flip(0.5); Cons(Terminal, bb(), prob_list_append(generate_from_grammar(XX(), fuel-1, append_fuel), generate_from_grammar(XX(), fuel-1, append_fuel), append_fuel)); else; Cons(Terminal, aa(), Nil(Terminal)); end),
                :YY => () -> (@alea_ite if flip(0.5); Cons(Terminal, cc(), generate_from_grammar(XX(), fuel-1, append_fuel)); else; (@alea_ite if flip(0.5); Cons(Terminal, cc(), generate_from_grammar(SS(), fuel-1, append_fuel)); else; Cons(Terminal, cc(), Nil(Terminal)); end); end),
                :SS => () -> (@alea_ite if flip(0.5); prob_list_append(generate_from_grammar(XX(), fuel-1, append_fuel), generate_from_grammar(YY(), fuel-1, append_fuel), append_fuel); else; prob_list_append(generate_from_grammar(YY(), fuel-1, append_fuel), generate_from_grammar(XX(), fuel-1, append_fuel), append_fuel); end)])

end

function pcfg_equals(l1::List{Terminal}, l2::List{Terminal}, fuel::Int)
    fuel == 0 && return false
    match(l1, [:Nil => () -> match(l2, [:Nil => () -> true, :Cons => (hd, tl) -> false]), :Cons => (hd1, tl1) -> match(l2, [:Nil => () -> false, :Cons => (hd2, tl2) -> (@alea_ite if prob_equals(hd1, hd2); pcfg_equals(tl1, tl2, fuel-1); else; false; end)])])
end

SYMBOL_TO_TERM = Dict(:a => aa(), :b => bb(), :c => cc())
function observation_list_from_symbols(symbols)
    if symbols == []
        return Nil(Terminal)
    else
        return Cons(Terminal, SYMBOL_TO_TERM[symbols[1]], observation_list_from_symbols(symbols[2:end]))
    end
end

function generate_observation_list(julia_list)
    obs_list = Nil(Terminal)
    sym_to_term = Dict(:aa => aa(), :bb => bb(), :cc => cc())
    for symbol in reverse(julia_list)
        obs_list = Cons(Terminal, sym_to_term[symbol], obs_list)
    end
    return obs_list
end


function pcfg_example(example, fuel, append_fuel)
    start = SS()
    return pcfg_equals(generate_from_grammar(start, fuel, append_fuel), generate_observation_list(example), append_fuel)
end

len_23_fuel_11_example = [:cc, :bb, :bb, :bb, :aa, :bb, :bb, :aa, :bb, :aa, :bb, :bb, :aa, :bb, :aa, :aa, :aa, :aa, :aa, :bb, :aa, :aa, :aa]

model() = pcfg_example(len_23_fuel_11_example, 12, 24)
