# Examples

Each file's header says what it shows and how to run it. `cargo test`
checks every program's answers (`tests/it/examples.rs`), against
ProbLog's or, for the Dice translations, Dice's, and for the Alea.jl
program the ones computed by hand in its header. The
[guide](https://tractables.github.io/tddlog/guide/) works through four of
them, and the [playground](https://tractables.github.io/tddlog/) opens
those and a few more, the larger programs below among them.

## Start here

| file | what it shows |
|---|---|
| [alarm.pl](alarm.pl) | facts, rules, evidence and queries; the program the README and [docs/design.md](../docs/design.md) follow |
| [bn_asia_evidence.pl](bn_asia_evidence.pl) | a Bayesian network, one annotated disjunction per table row, with evidence ([guide](https://tractables.github.io/tddlog/guide/#bayesian-network)) |
| [cycle_graph.pl](cycle_graph.pl) | reachability in a graph with cycles ([guide](https://tractables.github.io/tddlog/guide/#reachability)) |
| [adder.pl](adder.pl) | two integers in binary added bit by bit, as Dice compiles them ([guide](https://tractables.github.io/tddlog/guide/#binary-integers)) |
| [win.pl](win.pl), [gearwheels.pl](gearwheels.pl), [unsound.pl](unsound.pl) | negation through recursion under the well-founded semantics, and a program without a distribution ([guide](https://tractables.github.io/tddlog/guide/#negation)) |
| [smokers.pl](smokers.pl) | probabilistic rules and positive recursion |
| [dice.pl](dice.pl) | a die thrown until it shows 6: negation under recursion, safe under the query's demand |
| [uwcse.pl](uwcse.pl) | a relational program with annotated disjunctions and negation |

## More programs

| file | what it shows |
|---|---|
| [asia.net](asia.net), [cancer.net](cancer.net) | Bayesian networks in Hugin format, which tddlog reads as programs (`tddlog examples/asia.net`); `tddlog --program examples/asia.net` prints the program |
| [bn_cancer.pl](bn_cancer.pl) | cancer.net as `tddlog --query leaves` reads it and `tddlog --program --query leaves` prints it |
| [bn_split.pl](bn_split.pl) | a table that the indicator rewrite splits on a parent |
| [negcycle_sound.pl](negcycle_sound.pl) | negation through a cycle that is two-valued in every world, which ProbLog refuses |
| [dice_evidence.pl](dice_evidence.pl), [dice_murder.pl](dice_murder.pl) | Dice programs as tddlog's Dice reader reads them (`tddlog --program`) |
| [alea_dice.jl](alea_dice.jl), [alea_dice.pl](alea_dice.pl) | an Alea.jl program and the circuit Alea.jl builds for it, which `scripts/alea2problog.jl` writes |
| [query_beside_evidence.pl](query_beside_evidence.pl) | a query's formula beside evidence it does not read |

## Larger programs

In [larger](larger): sizes that ProbLog does not answer within five
minutes, and tddlog's browser build in one to three seconds. Their
answers are checked against exact ones computed independently, which
agree with ProbLog's on the smaller sizes it answers.

| file | what it shows |
|---|---|
| [grid_reliability.pl](larger/grid_reliability.pl) | two-terminal reliability of an 8 by 8 grid of routers: reachability over 112 uncertain links |
| [dice_sum.pl](larger/dice_sum.pl) | the sum of 100 dice, built die by die with arithmetic in the rules |
| [blood_types.pl](larger/blood_types.pl) | ABO blood types in a pedigree of 7 generations full of loops, with evidence on the youngest |
| [mastermind.pl](larger/mastermind.pl) | a code of 12 pegs in 12 colours after 10 scored guesses: counting with negation, and evidence of probability 2.5e-10 |
| [stones.pl](larger/stones.pl) | a game whose winning positions are defined through their own negation, over 4000 heaps |
| [robot.pl](larger/robot.pl) | a hidden Markov model: a robot on a ring of 12 cells after 400 sensor readings |

## One grounding step each

Each of these shows a step of [Grounding](../docs/design.md#grounding-srcground)
in docs/design.md; `tddlog --ground FILE` prints what the step made of it.

| file | step |
|---|---|
| [linear_path.pl](linear_path.pl), [linear_target.pl](linear_target.pl) | 2: right-linear recursion grounded from its bound end |
| [equal_atoms.pl](equal_atoms.pl) | 5: atoms the grounder made merged into atoms with the same rules |
| [views.pl](views.pl) | 6: views replaced by their bodies |
| [equal_gates.pl](equal_gates.pl), [equal_functions.pl](equal_functions.pl) | 6: equal gates merged, by structure and by function |
| [row_classes.pl](row_classes.pl) | 7: row classes |
| [shared_rows.pl](shared_rows.pl) | 7: switch sharing |
| [cover.pl](cover.pl) | 8: atoms certain by cover |
| [cycle_graph.pl](cycle_graph.pl) | 12: linear recursion written without recursion (`--ground --vtree closed`) |

## From Rust

| file | what it shows |
|---|---|
| [solve.rs](solve.rs) | answers a program, Dice or network file as the command line does: `cargo run --release --example solve -- examples/alarm.pl` |
| [sprinkler.rs](sprinkler.rs) | a network written as a program from Rust data: `cargo run --release --example sprinkler` |

The Python and C bindings have examples of their own, in
[bindings/python/examples](../bindings/python/examples) and
[bindings/c/examples](../bindings/c/examples).
