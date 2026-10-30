# Design

This page is for people who change tddlog. It follows one program through
the pipeline, then describes each stage in the order it runs: what it
does, why, and where the code is. The cases measured behind the stages'
rules are at the end, in [Measured cases](#measured-cases), under the
same headings.

To use tddlog, start with the [README](../README.md) and the
[guide](https://tractables.github.io/tddlog/guide/). tddlog answers
ProbLog queries by Tp-compilation (Vlasselaer et al.) into the structural
tree decision diagrams (TDDs) of
[tididi](https://github.com/Tractables/tididi), whose
[data model](https://tractables.github.io/tididi/tididi/guide/model/index.html)
explains vtrees, levels, nodes and pairs. [papers/](papers/) has notes on
the papers the stages come from: [Tp-compilation](papers/tp-compilation.md)
(the compile, negation through recursion), [Fierens et
al.](papers/fierens-tplp15.md) (ProbLog's semantics, the relevant ground
program, the weights) and [flip-hoisting](papers/cheng-probprog21.md)
(switch sharing). The constants named here are documented with their
reasons in the source. Sizes in pairs, nodes and engine work units are
machine independent. The results are in
[benchmarks.md](https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md),
on the baselines branch.

## How a program is answered

Take `examples/alarm.pl` (Fierens et al. 2015, Example 1):

```prolog
0.1::burglary.
0.2::earthquake.
0.7::hears_alarm(john).
0.4::hears_alarm(mary).
alarm :- burglary.
alarm :- earthquake.
calls(X) :- alarm, hears_alarm(X).
evidence(calls(john), true).
query(burglary).
query(earthquake).
query(calls(mary)).
```

The four *probabilistic facts* are independent: a choice of their values
is a *world*, whose probability is the product of the facts'. Every other
atom is *derived*: in each world it holds where the rules derive it (the
well-founded model of the world). `P(q | e)` is the probability of the
worlds where the query `q` and the evidence `e` hold over that of the
worlds where `e` holds, and tddlog computes both as weighted model counts
of TDDs. `solve` (`src/solve/`) runs six stages, for the binary and for a
library caller alike.

1. **Parse** (`src/syntax`). The text becomes a `Program`: its clauses,
   queries and evidence. A network file is read as a program first
   ([Network files](#network-files-srcnetwork)).
2. **Ground** (`src/ground`). The grounder writes the *ground program*:
   rules without variables over the atoms the queries and the evidence
   depend on, simplified by the passes listed under
   [Grounding](#grounding-srcground). `tddlog --ground` prints it as a
   ProbLog program, and under a strategy other than `auto` the program
   that layout compiles (after stages 3 and 4):

   ```text
   $ tddlog --ground examples/alarm.pl
   0.1::burglary.
   0.2::earthquake.
   0.4::hears_alarm(mary).
   alarm :- burglary.
   alarm :- earthquake.
   calls(mary) :- hears_alarm(mary).
   evidence(alarm,true).
   query(burglary).
   query(earthquake).
   query(calls(mary)).
   ```

   The evidence `calls(john)` holds exactly where `alarm` and
   `hears_alarm(john)` do, so the conditioning on the evidence (step 9)
   fixes both: `hears_alarm(john)` becomes certain, its 0.7 a factor of
   `P(q, e)` and `P(e)` alike that the ground program's `weight` keeps;
   `alarm` is observed in place of `calls(john)`; and `calls(mary)` reads
   `alarm` as true. An annotated disjunction would have become a chain of
   *switches*, facts that choose one of its heads (step 1). A table-shaped
   program (a Bayesian network, one annotated disjunction per row) skips
   the steps up to simplification. Where the queries' own parts of the
   program are estimated much smaller than it, the queries are split into
   groups, each run on its part from stage 3 on
   ([Queries in groups](#queries-in-groups-srcsolvesplitrs-split_check)).
3. **Rewrite.** Linear recursion is written without recursion
   (`src/ground/eliminate.rs`, `ground::without_linear_recursion`) and the
   acyclic result finished again. Every layout compiles the rewrite
   (`Kind::eliminated`, the first step of `Kind::rewrite`); `auto` chooses
   its trials from the program as grounded and writes it once for all of
   them. A layout may then read derived atoms through *indicators*
   (`src/indicators/`): each atom another part of the program reads is
   read there through a fact of its own, a *copy*, tied to the atom by a
   *link* the program observes. alarm.pl has no recursion, and the layout
   that answers it reads no indicators.
4. **Choose a layout** (`src/vtree/`, `src/auto/`). The TDDs' variables are
   the facts (and the copies), and a *vtree*, a binary tree whose leaves
   are those variables, shapes every diagram: one function can take a few
   pairs on one vtree and millions on another. A *layout*, a `vtree::Kind`,
   builds a vtree for the program. `auto`, the default `Strategy`, tries a
   few layouts in order, each a *trial* with the program as it rewrites it
   (`Kind::rewrite`: through every indicator for `families`, through none
   for the others), each through compilation, evidence and queries, in
   turns under bounds in pairs and work that double each turn. The first
   trial to finish answers. Any other `--vtree` strategy is one layout run
   once without bounds (`Strategy::Trial`). On alarm.pl `auto`'s first
   trial, `td`, finishes in its first attempt.
5. **Compile** (`src/tp`). Each derived atom gets a *formula*, a TDD over
   the facts that holds in exactly the worlds where the atom does, by
   Tp-compilation: the rules applied to diagrams rather than to truth
   values. The atoms go by strongly connected components in dependency
   order, and a cyclic component is iterated until no formula changes.
   Here `alarm` is `burglary ∨ earthquake` and `calls(mary)` is
   `hears_alarm(mary)`. A formula is stored as the negation of the atom's
   (the *dual*), `¬burglary ∧ ¬earthquake` for `alarm`, but for an atom of
   one rule, stored as its body's conjunction. Each component compiles on
   the vtree projected to its ancestor cone.
6. **Evidence and queries** (`src/query/`, `src/eval/`). The evidence
   factors, here `alarm`'s formula, are conjoined bottom up along the
   vtree. A query on a variable of the evidence formula (`burglary`,
   `earthquake`) is a literal of it, and one backward pass counts them
   all; any other (`calls(mary)`) is counted against the evidence in dense
   tables. A fact weighs `p` true and `1 − p` false, so `P(burglary | e)
   = WMC(burglary ∧ e) / WMC(e) = 0.1 / 0.28 = 0.357143`, and `P(e)` is
   `0.28 × 0.7 = 0.196`, with the 0.7 the conditioning took out.

The size of a diagram is counted in *pairs*: a TDD node at a vtree node is
a list of (left, right) pairs, a child on each side. tididi's engine
counts the work of its operations in *work units*. Every bound tddlog
sets is in pairs or work, never in seconds, so a run takes the same
decisions on every machine. `tddlog --stats` prints what each stage did.

## Where the code is

`solve` runs the stages under `Options`; `solve_ground` runs a program
already ground, whole. A `Strategy` (`src/strategy.rs`) is `auto` or one
layout, a `vtree::Kind`, which fixes the grounder's options, the rewrite,
the formulation and the projection besides its vtree. The modules, in
`src/`:

| module | what it holds |
|---|---|
| `bin/tddlog/` | the binary: `main.rs`, the command line, options and answers; `stats.rs`, the `--stats` lines; `alloc.rs`, the allocator |
| `lib.rs` | `Run`, `Error`, the public items |
| `input.rs` | `Format` and `read`: every input (a ProbLog program or an LPAD, a Dice program, a network file with its evidence file) read as the program `solve` takes, the one route of the binary and the bindings |
| `solve/` | `mod.rs`: `Options`, `solve` (the one answer path: ground, what `auto` is handed ready, the split, each part's run), `solve_ground` (one program's run); `split.rs`: queries in groups |
| `strategy.rs` | `Strategy` (`auto` or one layout), what a `Kind` decides besides its vtree (grounding, rewrite, formulation, projection), `Prepared`: a layout's program (with `td`'s separator copies, or the names of `program` and `decision`) and vtree |
| `evaluation.rs` | `Evaluation`: a program's compile and query phase on one vtree, resumed across `auto`'s attempts |
| `auto/` | `auto`'s trials: `mod.rs`, the run of the trials in turns, and what it is handed ready (`Ready`: the program with its views in place, conditioned for `decision`, and `families:0`'s layout); `policy.rs`, which trials a program gets, in order, their weights, and when the searched layout takes `families:0`'s place; `trial.rs`, a trial and what it holds |
| `syntax/` | lexer, parser, AST; `lpad.rs`, an LPAD written for cplint read as the program it holds; `dice.rs`, a Dice program read as the program its symbolic evaluation writes |
| `network/` | network files read as programs: `mod.rs`, `read` and the program written from a network's tables; `bif.rs`, `hugin.rs`, `xdsl.rs` (with Ace's `.inst` evidence) and `uai.rs` (with `.evid` evidence and Markov networks), the formats; `xml.rs` and `scan.rs`, what they are read with; `rows.rs`, the rows' rescaling and the probabilities' text, which `syntax/lpad.rs` and `syntax/dice.rs` write too |
| `ground/` | `mod.rs`: the pipeline, `GroundOptions`, `ground`, `finish`; `program.rs`: `GroundProgram`, `GroundStats`, the shared helpers (`Lists`, `named_atoms`, `mix`); `encoding.rs`: `Encoding`, what a rewrite through indicators records of its copies, links, codes and sibling groups; the passes in order: `terms.rs`, `table.rs`, `noisy_or.rs`, `normalize.rs`, `unread.rs`, `chain.rs`, `linear.rs`, `magic.rs`, `engine.rs` (with `schedule.rs`, body order), `simplify.rs`, `inline.rs`, `aig.rs` (the and-inverter graph of the gates, structural hashing and SAT sweeping), `structure.rs` (the `Structure` kept with the program) over `exclusive.rs` (the exclusive sets) and `tables.rs` (`Tables`), `contexts.rs`, `classes.rs`, `share.rs`, `cover.rs`, `condition.rs` (with `propagation.rs`, the unit propagation it and `detach.rs` drive, and `renormalize.rs`), `preprocess/` (`backbone.rs` and `sat.rs`, a CDCL solver; `implied.rs`, `defined.rs` with `support.rs`), `detach.rs`; the layouts' rewrites: `eliminate.rs` (linear recursion), `guards.rs` and `steps.rs` (`decision`'s), `fact_copies.rs` (`td`'s separator copies); `deps.rs`: derived SCCs, cyclicity, internal negation, merged components; `shape.rs`: the program's shape, which `auto` picks its trials by |
| `indicators/` | `mod.rs`: `introduce`, which fills the program's `Encoding` through `name` in steps (what the program reads, `Reads`, with the binary reads, `Bits`; the copies, `Copies`; the rules' bodies over them, `Bodies`; the bits' rules; the links, with the tables' rows as their readers' clauses), `clause_links`, and `name` and `name_once`, the copies that the names of `program` and `decision` are read through; `tables.rs`: clause tables; `splits.rs`: table splits and units |
| `vtree/` | `mod.rs`: `Kind`, `Layout`, `build`; `graph.rs`: interaction graph, min-fill, fact orders; `forest.rs`, `cost.rs`: the forest and its layout cost; `td.rs` with `separators.rs` (its separator copies), `closed.rs` with `order.rs`, `nest.rs`, `families.rs`, `decision.rs` (with the names of its tree's contexts and of the atoms read outside it): the strategies' layouts; `naming.rs`: `program`'s narrow cuts and the atoms named across them |
| `tp/` | `mod.rs`: `CompileError`, `Projection`, `Formulation`; `compile.rs`: `Compilation` (in parts: the fixpoint's state, the encoding by variable, the projection's scopes, the dying formulas, the dynamic search's schedule) and `Compiled`, with `ForQueries`, what the query phase takes over; `factoring.rs`, `clauses.rs`, `none_of.rs`, `rows.rs`: rule application, the clause memo, the level-by-level builder, a table's link from its models; `tie.rs`: readers of a clause table tied; `template.rs`: queries answered by another query's formula; `aft.rs`: negation through recursion; `cone.rs`: ancestor cones; `backend.rs`: diagram operations, minimization; `settle.rs`: which of the cuts of `program` and `decision` keep their names, and the copies of the others composed away; `scope.rs`, `projector.rs`: the scopes formulas live on (`VtreeScope`, `ScopeTable`) and the projection of the vtree that makes them; `stats.rs`: the compile's counters; `tests/`: the dynamic search, dying formulas, resumption, the primal's update, the clause memo and projection on vtrees of their own |
| `query/` | `mod.rs`: `QueryEvaluation`, the query counts; `walk.rs`: the evidence walk; `sum_out.rs`: its finished subtrees summed out where only P(e) is wanted |
| `eval/` | `mod.rs`: `Value`; `number.rs`: `Count`; `weights.rs`: the weights of facts and indicators; `count.rs`: a count's two passes, the count of a conjunction; `grid.rs` and `kernel.rs`: count passes over levels read as grids, and their vector kernels |
| `cubes.rs` | cubes merged on complementary literals |
| `disjoint.rs` | the union-find |
| `multisets.rs` | literal lists numbered as multisets, for `tp`'s templates and `vtree`'s families |

Items are `pub` only where the binary, the bindings or a file under
`tests/` uses them, together with the types their signatures and fields
name.

## Language (`src/syntax`)

The parser reads ProbLog without cut:

- constants, integers, floats, variables and compound terms, lists
  `[a, b | T]` among them, with ProbLog's operators (`N - 1` is the term
  `-(N,1)`, and an operator alone is an atom, as in `[+, -]`);
- probabilistic facts `p::a.`, probabilistic rules `p::h :- b.`,
  annotated disjunctions `p1::h1; p2::h2 :- b.`, and negative heads
  `\+h :- b.` (ProbLog's `p$pos`/`p$neg` reading);
- body disjunctions `;` and parentheses, split into clauses that share
  one switch group;
- negation as failure, `\+` or `not`, also of an atom with variables
  of its own (`\+ p(X, _)`: no `p(X, Y)` for any `Y`);
- unification `=` and `\=`, the comparisons of terms `== \== @< @> @=<
  @>=` in ProbLog's standard order, the arithmetic comparisons `< > =< >=
  =:= =\=`, and `is` with ProbLog's arithmetic, which is Python's: `//`
  rounds down, `mod` takes the divisor's sign, `/` divides in floating
  point, and a float a term holds is rounded to 15 decimal places, as
  ProbLog's constants round theirs;
- the list predicates of ProbLog's library, built in: member/2,
  memberchk/2, append/3, length/2, nth0/3, nth1/3, last/2, reverse/2,
  sum_list/2, max_list/2, min_list/2, sort/2 and between/3, in ProbLog's
  modes and order of solutions;
- `findall/3` over a deterministic goal that is not recursive, its
  solutions in ProbLog's order;
- `query/1` (also `query(\+a)`), `query(pattern) :- body.`, and
  `evidence/1,2`.

There are no cut, if-then-else or strings.

Every input reaches the parser through `read` (`input.rs`): a ProbLog
program is its own text, and an LPAD, a Dice program and a network file
are read by their modules as the program `tddlog --program` prints. The
command line takes the format from the file's extension
(`Format::of_path`), and the bindings take its name (`problog`, `dice`,
`bif`, `hugin`, `xdsl`, `uai`), with a network's evidence file as its
contents; a network's options given for another format are refused.

A source that holds the directive `:- begin_lpad.` (or cplint's other
spelling, `:- begin_plp.`) is an LPAD written for cplint and PITA, and
`syntax/lpad.rs` reads it as a ProbLog program, which the parser then
reads and `tddlog --program` prints (`read` finds the directive in a
ProbLog input). The LPAD is the clauses between
`:- begin_lpad.` and `:- end_lpad.`: `h1:p1 ; h2:p2 :- b.` becomes
`p1::h1; p2::h2 :- b.` and `a:p.` becomes `p::a.`, a head without a
probability taking what the others leave (a probability may be a
fraction, `1/4`). The clauses outside the markers are cplint's database
clauses, which the LPAD calls through `db(Goal)` or the goal of a
built-in such as `findall/3`: those the LPAD's bodies name, and those
they name in turn, are kept, and `db(Goal)` becomes `Goal`; the rest
(plots and other programs over the LPAD, in Prolog that tddlog does not
read) is left out with the directives and goals. The file's example
queries become queries: `?- prob(Q, P).` is `query(Q).`, and `?- prob(Q,
E, P).` is `query(Q).` with an `evidence(L).` for each literal `L` of the
conjunction `E`. A program has one set of evidence, that of the first
example query not under an intervention `do(L)`; a query under other
evidence or under an intervention is named in a comment of the program
and on standard error, and left out. A clause the program cannot write
(a disjunctive head with no probability or with two heads without one, a
probability that is not a finite number) is refused on its line. The
reader finds the clauses in the text and writes the program that the
Python script it replaced (`lpad2problog.py`, now on the baselines
branch) wrote: the clauses end at a `.` before white space outside
quotes and brackets, the names a body calls are its words outside
quotes that start with a lower-case letter, and the probabilities are
written as Python writes a float (tests/lpad holds LPADs and the
script's programs for them).

A file with the extension `.dice` is a Dice program, and `syntax/dice.rs`
reads it as the ProbLog program its symbolic evaluation writes, which the
parser then reads and `tddlog --program` prints. Every `let` binds a
Boolean literal, an integer or a tuple; an integer of type `int(w)` is `w`
bits, each a literal; a `flip` is a probabilistic fact, and a `discrete`
or a `uniform` is drawn bit by bit from the most significant, as Dice
draws it, each bit a flip whose probability is a ratio of exact sums of
the masses. Boolean operators and conditionals are rules over fresh
atoms; `+`, `-` and `*` are adders over the bits, `==` a conjunction, `<`
a comparator. The values an integer can take are tracked where their
pairs are few, so that a comparison every pair decides is a constant, and
within a branch of a conditional a variable the guard compares with a
constant takes only the values the guard allows. An `observe` within a
branch holds where the branch is not taken. A function is inlined at each
call, a recursive one up to Dice's limit of ten nested calls, the call
past it an observation of false. `iterate(f, init, k)` is recursion over
a step index that the grounder unrolls: f's body once, its atoms indexed
by the step, the state before step `T + 1` the body's at step `T`. The
program's result is queried, every value of an integer as the
conjunction of its bits. Lists, `/`, `%`, `nth_bit`, `sample`, `head`,
`tail` and `length` are refused. The reader writes the program that the
Python script it replaced (`dice2problog.py`, now on the baselines
branch) wrote, byte for byte (tests/dice holds Dice programs and the
script's programs for them).

A predicate without clauses is not read as false: a query or an
observation of one, and a call of one that is not a built-in above in a
clause the queries and the evidence reach, is refused, as ProbLog refuses
it ("No clauses found"), so that a misspelt name or a built-in of
ProbLog's that tddlog does not evaluate (`write/1`, `call/1`) stops the
run rather than give a wrong probability. ProbLog refuses a call only
once its resolution reaches it, and answers where the call comes after a
literal that never holds; tddlog, which checks predicates, refuses that
program too.

The grounder lowers the rest
to the Datalog the passes after it read (`ground/terms.rs`, before
normalization): a compound with variables in a clause becomes a variable
and a unification with one functor in the body, which builds the term
where its arguments are bound and takes it apart where the term is; an
argument that is a compound of one name and arity in every head and call
of its predicate is read as the compound's arguments, so that a demand
binds the parts a call binds (a count passed in, another passed out); the
goal of a findall becomes an auxiliary predicate whose tuples are its
solutions, and a negated atom with variables that nothing else in its
clause reads becomes the negation of an auxiliary predicate over its
other variables, defined by the atom. A ground compound is a value: the
engine interns each one once, its name and its arguments' values end to
end in one buffer as a relation's tuples are, and builds the term only
where an atom is printed or terms are compared; the ground program's
atoms hold them as arguments, so that nothing after grounding sees a
term's structure and the compile does not change. A grounding that builds terms deeper than 10 000 does not end and
is refused, as is a findall whose answer ProbLog gives otherwise
than tddlog could: over a probabilistic goal, which is a distribution over
lists, or where a solution has two proofs, which ProbLog lists twice.
Rules must be range restricted, but for variables a demand binds; a body
is reordered so that binding literals come first and switches last, as
they bind nothing and reject no binding (`ground/schedule.rs`, shared by
the grounder and the magic rewrite). Where the grounder joins a body, the
next atom is one whose variables are all bound, else the one that shares
the most bound variables, the first in source order among equals: an
atom whose variables are all unbound after some are bound would pair
each of its tuples with each binding so far, and one that shares only a
variable most tuples bind alike would read most of its relation for each
binding; the magic rewrite passes the bindings in the order
of the source, as ProbLog's resolution does, which a clause such as
`walk([], _, _)` relies on to be asked bound. Predicate names and
constants are `Name`s, an `Arc<str>` shared from the lexer on, one per
distinct spelling.

## Network files (`src/network`)

A Bayesian or Markov network file is read as a program, which
`tddlog --program FILE` prints and the other systems of the benchmarks
read, and that text is parsed as any program is: one route, whose
answers are those of the program. The reader replaced a Python script
(`gen_bn.py`, now on the baselines branch) and writes its programs byte
for byte; `tests/it/networks.rs` checks the reader's text line for line
against the script's programs for the files of `tests/networks/`,
`tests/regress/` and `examples/`. The extension names the format: BIF
(`.bif`), Hugin (`.net`, `.hugin`), GeNIe (`.xdsl`, whose `cpt` nodes
are tables; another kind of node is refused) and UAI (`.uai`). BIF and
Hugin are read as the script's regular expressions read them, GeNIe's
XML into a tree (`xml.rs`: the encoding the declaration names, comments,
entities). A file that is not UTF-8 is read as ISO-8859-1, as Ace's
water, pigs and mildew are written. A compressed file is refused.

Each variable gets one annotated disjunction per row of its table,
`p1::x(v1); ...; pk::x(vk) :- p1(u1), ..., pm(um).`, in the order of a
depth-first walk from each variable in turn to its parents. A name is
lowercased, every other character than a letter, a digit or `_` becomes
`_`, and `v_` goes before one that does not start with a letter; a clash
is an error. A zero head is dropped and a row of one certain value is a
plain rule. The queries (`--query`) are every value of every variable, of
those without children or of those named, an observed one left out; the
evidence (`--evidence`) is an Ace `.inst` file or, for a UAI network, a
`.evid` one; under `--no-queries` the program asks nothing.

**Rows** (`rows.rs`). A row sums to one, and a file's decimals round it.
A sum within 1e-9 under one (`math.fsum`'s, the exact sum rounded once)
keeps the row. A sum further from one
divides the row where the rounding of its decimals explains the gap. Each
entry is the decimal it means: the shortest of at most 12 significant
digits within two units in the last place of its double (the file's own
text for an entry written so; 0.00099404 for the 0.0009940400000000005 of
Ace's pathfinder.xdsl, whose doubles a program printed in full), and
otherwise the shortest decimal that reads back as the double. Each entry
may be off by the larger of half a unit in the row's finest place and half
a unit in the most significant digits any entry of the row has
(pathfinder's `0.3333333` three times; `0.9999, 9.999e-05`, which are
10000/10001 and 1/10001 to four digits), and the gap of the decimals'
exact sum is compared with the sum of these. An excess may equal that
slack, entries rounded up by half a unit each (`0.160598, 0.839403` in
the UAI 2006 networks); a shortfall must be under it. An excess also
divides the row where it is at most `(n + 1) 2^-24` for `n` entries, the
error of a row computed in single precision, its entries rounded to
floats and divided by their sum of `n - 1` rounded additions: the UAI
classics write such rows to eight decimals, diagnose_b's `0.94297521,
0.05702495` over one by 1.6e-7. Otherwise a shortfall is the
table's own, its mass left to no value as in ProbLog, and an excess is an
error. The script decided by the same rule, but for the excess of single
precision, which it refused past 1e-9, and an excess equal to the slack,
which it refused, and the two decide alike on every
row of every network the script read. `--rescale` divides
every row by its sum first, for Ace's tables in percent or as weights.

**Markov networks** (`uai.rs`). A `MARKOV` file whose functions are each
a conditional table of one variable of its scope, one function per
variable and the child relation acyclic (Promedus, Pedigree), is that
Bayesian network. In any other, every variable gets a uniform prior times
its unary factors, normalized, and every other factor `f` becomes a
variable `fJ` of one value, observed, whose row for each assignment of the
factor's scope is its entry over its largest; a constant factor is left
out. The program's `P(e)` times `10^c`, where `c` sums the base-10
logarithms of the priors' sums, the factors' largest entries and the
constants, is the partition function given the evidence: the program
states `c` on its first line, and the binary prints `Z:` for such a
file where it would print `P(e):` ([The probability of the
evidence](#evidence-and-queries-srcquery-srceval)). Under `--no-queries`
without observations, a network of Potts tables over pairs of variables of
`k` values each (a graph colouring) has its colour symmetry broken: one
variable of largest degree is fixed, a neighbour takes two values with
weights 1 and `k - 1`, and a common neighbour the values used so far and
the next; `Z` is then a `k`-th of the network's, and `c` holds `log10 k`
besides.

## Grounding (`src/ground`)

The grounder makes a `GroundProgram` (`program.rs`): atoms, ground rules,
the facts' weights, the queries and the evidence, with `weight`, the
factor that steps 9 and 11 and `td`'s separator copies take out of
`P(q, e)` and `P(e)` alike, and `stats`, what each step did, which
`--stats` prints. The ground program prints as ProbLog that ProbLog
accepts (`tddlog --ground`), which is how grounding is checked apart from
compilation.

Steps 0 to 4 are the front end. `finish` runs steps 5 to 11, after the
front end and again after each layout rewrite (step 12, `decision`'s
guards, `td`'s separator copies); where unit propagation settles the
evidence without the exclusive sets (`condition::settle`), steps 6 to 9
are skipped. Several steps have a program in `examples/` that shows them.

| step | files | what it does |
|---|---|---|
| 0 | `terms.rs`, `table.rs`, `noisy_or.rs` | compound terms, lists and findall lowered to Datalog with built-in literals; grounds a table-shaped program from its rows; writes noisy-or tables as noisy-ors |
| 1 | `normalize.rs`, `unread.rs`, `chain.rs` | annotated disjunctions and probabilistic rules to switches and rules; arguments no clause reads dropped; transitive closures to linear steps |
| 2 | `linear.rs` | right-linear recursion grounded from its bound end |
| 3 | `magic.rs` | magic sets: only what the queries and evidence demand is ground |
| 4 | `engine.rs` | stratified semi-naive bottom-up grounding |
| 5 | `simplify.rs` | certain atoms propagated, equal atoms merged, relevance, facts read as variables |
| 6 | `exclusive.rs`, `inline.rs`, `aig.rs`, `contexts.rs` | exclusive sets; views; equal gates; choices read under exclusive contexts |
| 7 | `classes.rs`, `share.rs` | row classes; switch sharing |
| 8 | `cover.rs` | atoms certain by cover |
| 9 | `condition.rs`, `preprocess/backbone.rs`, `renormalize.rs` | conditioning on the evidence: unit propagation, backbones, gates, renormalized rows |
| 10 | `detach.rs` | evidence no query depends on dropped |
| 11 | `preprocess/` | in a CNF over facts: implied clauses dropped, defined facts eliminated |
| 12 | `eliminate.rs` | linear recursion without recursion, after the grounder |

0. **Table-shaped programs** (`table.rs`). A program whose clauses each
   have ground heads of one predicate and a body of positive ground
   atoms, with ground queries and evidence, no query patterns and no
   cycle among its predicates, is a Bayesian network written one
   annotated disjunction per row (`tddlog --program` of a network;
   `examples/bn_cancer.pl`). What steps 1 to 4 make of it is known from
   the rows, so `table::ground_rows` builds that output directly, the
   same atoms in the same order, and simplification onward is shared
   (`finish`). The shape (`Shape`) is decided before anything is built;
   any other program takes the generic steps. Equality with the generic
   grounding is tested on the table-shaped examples.

   **Noisy-or tables** (`noisy_or.rs`) are then written as noisy-ors, from
   the same shape. A table of two values over parents of two values whose
   rows partition the parents' values, and whose value `c` has
   `P(c | u) = L · ∏ q_i` over the parents at which `u` differs from the
   row `u*` where `c` is likeliest, becomes `(1-L)::x(b).` and
   `(1-q_i)::x(b) :- y_i(a_i).`, where nothing reads, asks or observes
   `x(c)` once the readers taken are written so too, and an observed table
   only with its parents (the conditioning reads a table's clauses, not a
   noisy-or's). A chain of such tables is then a chain of views that
   inline (step 6) into one rule per cause. The program written is not
   built: its shape is made from the source's (`Shape::rewritten`).
1. **Normalize** (`normalize.rs`). Each probabilistic clause
   `p1::h1;...;pn::hn :- body` with clause variables V becomes rules
   `hi :- body, swi(V), \+sw1(V), ..., \+sw_{i-1}(V)`. With `S_i` the
   mass of heads `i` on and of no head, switch `i` weighs `pi / S_i` true
   and `S_{i+1} / S_i` false, each weight its own number (`Prob`,
   `chain_weights`), never `1 - q`: a tail after a head of nearly all the
   mass left keeps its mass. Switch predicates have no rules; their
   ground atoms are the random variables. A value that takes all the mass
   left has no switch, and a head of probability zero none either.
   A predicate given by ground facts alone, `p::f(a,b).` or `f(a,b).`,
   none of them in a disjunction and none read by a findall, is kept as
   its facts (`Facts`): the grounder makes each tuple an atom, a fact of
   its switch's weights or a certain one, without a switch or a rule per
   fact, and two facts of one tuple make it true where either is. A
   database's tables are such facts: TPC-H's line items at scale factor
   0.1 are 600 000 of them. An atom keeps its arguments in its tuple
   alone, a fact's as a derived atom's, and the ground program has only the atoms of facts that
   a rule, a query or the evidence names: a query reads few of a
   table's tuples.

   **Arguments no clause reads** (`unread.rs`). An argument that every
   clause of a predicate leaves as a variable the clause mentions nowhere
   else (cplint's left-corner grammar: `lc('S', a, _Der, rule('S', [a]))`)
   does not decide which atoms hold, and is dropped from the predicate's
   heads and from every literal that reads it, until none qualifies. The
   non-ground facts become ground ones, which step 3 reads as they are,
   and their negations no longer depend on the derivation. A switch
   predicate, a predicate asked, observed or read by a findall, and one
   whose name is in use with the smaller arity keep their arguments.

   **Transitive closures** (`chain.rs`). A predicate `p/2` whose rules are
   exits that do not read it and the chain rule `p(X,Y) :- p(X,Z),
   p(Z,Y)` holds the transitive closure of its exits' relation in every
   world, which the chain rule leaves to the compile's Tp iteration over
   every pair of nodes. Each exit `p(s,t) :- B` then gives a linear step,
   `p(s,W) :- B, p(t,W)`, or `p(W,t) :- p(W,s), B` where every use of `p`
   binds its second argument and some use leaves the first unbound, so
   that step 2 finds the bound end and step 12 writes the recursion
   without recursion (Zhang, Yu and Troy, TODS 1990).
2. **Right-linear recursion from the bound end** (`linear.rs`;
   `examples/linear_path.pl`, `examples/linear_target.pl`). A predicate
   whose recursion is right-linear and whose every other use binds the
   remaining positions to constants is also rewritten to chains from those
   ends (`$r_p`; Naughton, Ramakrishnan, Sagiv and Ullman, VLDB 1989).
   Both programs are ground and the one with fewer rules plus body
   literals is kept. As written, `path(1,Y)` over a right-recursive `path`
   grounds `path(X,Y)` for every `X` on a path to some `Y`, one recursive
   component per target. A predicate whose rules have no variables is
   ground as written and not rewritten: the rewrite would ground its atoms
   again, once per bound end.
3. **Magic sets** (`magic.rs`), wherever the program has variables:
   adorned copies `h :- $m_h(bound args), body` plus demand rules, seeded
   from the queries and evidence. A demand rule reads the literals before
   its atom that bear on the atom's bound arguments, those that share a
   variable with them or with a literal that does: another only asks that
   some values of its own satisfy it (on PSouffle's side-channel analyses
   the demand of `equal_assign` with no argument bound was derived again
   from each of 437 million pairs before it). The rewrite only restricts which ground
   atoms are produced; the control predicates' tuples have no atoms and do
   not reach the compiler. A
   recursive component with internal negation is ground as it is, its
   negative literals left to the compiler; the program as written is the
   fallback where the rewritten one cannot be ground. A bound head argument
   that the body reads only as the result of an `is` binds nothing: the
   rule is copied under the pattern without it, whose demand is projected
   from the bound one (`'$m_s_bf'(K) :- '$m_s_bb'(K,S)`), so that a sum is
   not ground once per demanded value. A predicate demanded both with
   every argument free and with some bound is demanded free only, in a
   second rewrite: its free demand derives every atom a bound one would,
   and a bound demand whose rule reads the predicate's own atoms before
   the bound argument (`r(Y) :- r(X), a(Y, X)`) calls `a` for every pair
   of a bound `Y` and an atom of `r`. A given predicate, of ground facts
   (with or without probabilities) or of rules that read only given
   predicates and bind their heads without a demand, is not demanded: its
   facts and rules are read once, rather
   than copied under a guard and checked against every new demand of a
   recursive stratum (PSouffle's side-channel analyses: 83 571 facts in
   each of 1 350 rounds). Its atoms are all known before any demand is,
   so a negation of it filters a demand as a positive literal does:
   cplint's left-corner grammar tries a goal's third clause only where
   `\+ lc(G, B, rule(G, [B|_]))` holds, which is nowhere, and without the
   filter demanded every derivation of that clause. A program whose
   recursive strata all have rising levels (below) is not rewritten but
   grounded level by
   level: the demand of a loop over a time index runs down the levels
   (`T0 is T - 1`) and makes the whole chain one stratum without levels.
4. **Bottom-up grounding** (`engine.rs`). Predicates are stratified by the
   SCCs of the dependency graph and each stratum is evaluated semi-naively
   with every switch assumed true, which yields exactly the atoms true in
   some world. Every instantiation is a ground rule, but for those of a
   head that a rule whose body atoms are all true in every world (the
   deterministic facts, and what such rules derive from them) has made so:
   they add nothing, and a closure of deterministic facts (PSouffle's
   `equal_assign`) would keep a rule for each atom between two of its
   atoms. A semi-naive round walks a rule from the literal that reads
   the round's new tuples, the rest of the body joined to them
   (`schedule.rs`'s `Order::From`): in the rule's own order, a round
   walked every binding of the literals before that one to find the few
   that meet a new tuple (pcfg24's 368 518 demands in each of 43
   rounds). It does so where that is expected to walk fewer bindings: the
   new tuples times the tuples the atom joined next matches for each,
   against the product of the tuples each atom before that literal in
   the own order matches, a relation expected to hold as many tuples for
   a value of an argument as it holds for each of its values there. The
   own order stays where the next atom is reached only by a value every
   tuple shares (DeepProbLog's handwritten formulas, whose label is the
   same in every demand). A literal with bound arguments, the delta's included, takes
   its candidate tuples from a per-position index of its relation, each
   position indexed when a join first binds it. A negative literal on an impossible
   atom is dropped as true, and one on an atom true in every world fails
   the rule, a demand's rule too; one inside its own stratum (negation through
   recursion) is kept for the compiler's well-founded fixpoint
   (`examples/win.pl` shows both). The atoms and the rules grow through
   fallible reservations: where the allocator refuses one, the grounding
   is given up with `GroundError::OutOfMemory`, as a diagram operation is
   refused, rather than aborted.

   A recursive stratum has *rising levels* where the last argument of each
   of its predicates is a level that no rule lowers: every literal of the
   stratum is read at its head's level (the same variable, or constant
   levels h >= b) or below it (`H is B + c`, c >= 1, or a literal of a
   relation outside the stratum whose clauses all rise from B to H, as
   `next(B, H)` given by its facts), and some below it,
   an XY-stratification (Zaniolo, Arni and Ong, DOOD 1993). Within a
   level, the components of the graph of the literals read at their
   head's level rank the predicates. A program with variables whose
   recursive strata all have rising levels grounds each of them key by
   key, (level, rank) in order (`evaluate_by_levels`): a rule instance
   whose head is at a later key waits for it, and a rank whose predicates
   read each other at their own level takes semi-naive rounds within its
   key. A key's atoms are then complete before a rule reads them at a
   later key, so a negated atom of an earlier key is resolved where it is
   read, and the gates of an acyclic key's atoms are built in one
   and-inverter graph over those of the keys before (`aig.rs`'s
   `Pruner`), whose sweep proves some of them constant: the rules
   grounded after pass over an atom proved false and drop one proved true
   from their bodies. On a loop written as recursion over its steps
   (Dice's `iterate` as the Dice reader writes it) the dead branches
   of each step are never grounded, and the program grounds fewer atoms
   than the loop unrolled.
5. **Simplify** (`simplify.rs`): propagate certain atoms; merge each atom
   the grounder introduced into another atom with the same rule bodies
   (atoms the program writes wait for the exclusive sets, below;
   `examples/equal_atoms.pl`); restrict to what the queries and evidence
   depend on; read `a :- sw` as the variable `a`. Where every atom that
   reads a fact has that fact alone as its one rule, the first of them
   becomes the fact and the others are read as it, to a fixpoint: an
   undirected edge `conn(a,b) :- edge(a,b)`, `conn(b,a) :- edge(a,b)` is
   one variable.
6. **Exclusive sets** (`exclusive.rs`). Two derived atoms are exclusive
   when every pair of their rules has clashing bodies. The sets are the
   values of a network variable; row classes, the cover, the indicators
   and the families layout read them. Where the cover leaves the program as
   it is, the grounder keeps them with it, with the rules by head, the
   derived components and the units (`GroundProgram::structure`), for the
   indicators and `auto`'s `nothing_to_order` to read.

   Two rules keep the comparisons few. An atom is compared only with the
   sets that no body of its rules rules out: a set rules out the bodies of
   its atoms' rules and those that do not clash with one of them. A body
   here leaves out the facts that no other rule reads, which clash with
   nothing, so that the rules of atoms that differ in such facts alone, as
   a choice read through a fact per exclusive context, stand for each
   other. Nor is an atom compared with every set of a key that holds two
   or more: an atom joins a set only where each of its rules clashes with
   the first rule of the set's first atom, so such a key's sets are listed
   by the literals that clash with that rule, and an atom is compared with
   the sets listed under the literals of one of its rules, the one with
   the fewest.

   Then **views** (`inline.rs`; `examples/views.pl`). A derived atom that
   no query or evidence names, of a predicate that does not depend on
   itself, in no exclusive set, read only positively, and whose rules read
   facts alone, is replaced in its readers by its rules' bodies where it
   is read once or has one rule, and the literals that adds are at most
   twice those of its rules and its readers (`GROWTH`). The atoms go in
   dependency order, so a view over views reads facts alone in its turn.
   Tp-compilation gives every derived atom a formula and the reader one
   negated term per body on the reader's scope, where a structural TDD
   keeps a node at every level; a query written through views paid for
   each view's term on its whole scope. The atoms of a recursive predicate
   stay even where the ground program is acyclic: a path in a directed
   grid is a part that the paths into its first node share. An atom of one
   rule read once is replaced by its body whatever the body reads, which
   copies nothing; so is an exclusive set of such atoms of several
   predicates as a whole, the branches of a choice. A set of one
   predicate's values stays: they are a network variable, whose copies
   inlined cost more work than the variable. A loop unrolled step by step
   names each step's intermediate values this way: the Dice paper's
   diamond, unrolled, keeps one atom a step of four. Written as recursion
   over a step index, as the Dice reader writes Dice's `iterate`, they
   are a recursive predicate's atoms and stay.

   Then **equal gates** (`aig::reduce`; `examples/equal_gates.pl`,
   `examples/equal_functions.pl`). The acyclic derived atoms with rules,
   the *gates*, are built in dependency order into an and-inverter graph:
   each the OR of its bodies, each body the AND of its literals, from
   two-input ANDs with complemented edges, hashed on creation, the
   literals of a body and the bodies of a gate sorted. The inputs are the
   facts and the atoms of cyclic components; a derived atom without
   rules is false. A gate built as an earlier gate's node, the same
   bodies over equal gates, is merged into it: its rules go and its
   readers read the earlier atom, so a gate is compiled once however
   often the program writes it (structural hashing). Alea.jl writes a
   gate again wherever its program computes a value again.

   A gate alone in its predicate may also compute an earlier such gate's
   function, or a fact's, by other structure, or the complement of one,
   or a constant. Those are found as an and-inverter graph package finds
   them (Mishchenko et al.'s fraiging): every node carries its values on
   four words of 64 random assignments of the inputs, computed
   bit-parallel as it is created, and a gate whose values are an earlier
   candidate's, or their complement, or constant, is proved equal to it
   by one incremental solver (`sat.rs`) on the Tseitin clauses of the two
   cones, each node's clauses given once, with a limit of conflicts a
   call and of steps a program (`STEPS_PER_LITERAL` a body literal). A
   model that tells two apart is simulated with the others as a word of
   its own, which splits the candidates; the search stops when 64 such
   models come with no gate proved among them, and while calls still
   prove it goes on within its steps, past 16 words no longer simulating
   them. A proved gate is merged into the earlier one, read through a
   negated literal where it is the complement, or keeps an empty body or
   none where it is constant; a later gate alone in its predicate built as
   its node is merged the same way.

   A body that then reads an atom both ways is dropped, and so is a rule
   its head already has. An atom a query or the evidence names is not
   merged away, nor a value of an exclusive set of a predicate with
   several atoms, a network variable's: merged into another variable's
   value, it splits its variable. Only atoms alone in their predicates and
   in no exclusive set of several atoms are merged into a complement, a
   fact or a constant: merged into a switch fact or another value, a
   table's values take the table's structure apart. The survivors keep
   their rules, over the atoms merged into. A program conditioned on its
   guards (`decision`'s, finished again) is not merged: the conditioning
   put each gate under the decisions that select it, and which of two
   equal gates survives decides whether `decision` compiles. The graph is
   built only where a gate that may be merged away has a constant's values
   or another atom's, or their complement, on the four words simulated
   over the rules first (`may_merge`): a Bayesian network's gates are all
   its variables' values, where the graph would find nothing.

   Then **choices read under exclusive contexts** (`contexts.rs`). A
   *choice* is a table whose atoms each have one rule over facts only the
   table reads: an annotated disjunction without a body, normalized to a
   chain of switches. Where every rule that reads an atom of a choice
   reads one, positively, and the rest of every two such bodies clash
   (each reads a different atom of one exclusive set, or the bodies are
   compared two by two up to `MOST_COMPARED` readers), each reading rule
   reads a fact of its own in place of the atom, of the atom's
   probability, and the choice goes. At most one context holds in a
   world, and where one does, the atom it read depends on facts nothing
   else reads, so the fact has the same effect. A choice is read so where
   it saves at least as many literals as it adds facts. Dice's `iterate`
   writes a step that draws a value and observes a function of it and a
   state as such a choice read under each value of the state: a step
   under one key is then one fact, not the chain of switches before its
   letter, and switch sharing then makes the facts of one probability
   one. The exclusive sets are found again after it.
7. **Row classes** (`classes.rs`; `examples/row_classes.pl`). The rows of
   a table that hold one distribution read it through a class atom, one
   rule per row, and the table's rules for the class read it and the
   switches of its first row. A table is rewritten where it has fewer than
   half as many classes as rows (`CLASS_GAIN`) and some class has
   switches. Tables of one parent are left alone, since a split on the
   parent already reads them a row at a time. A table whose rows lie on
   long spines (*Bodies of facts alone*, under Compilation) is rewritten
   too, with up to `MOST_SELECTORS` classes and one per row if need be
   (`SPINE_ROWS`, `OWN_VALUE_FROM`): a class atom then selects its rows,
   and the table over the classes is split on them, its parts side by
   side. A noisy-or over facts is left alone: its class atoms would be
   copies of the facts and split one component into many.

   Then **switch sharing** (`share.rs`, local flip-hoisting, Theorem 1 of
   Cheng et al.): in a table whose rows exclude each other, the k-th switch
   of probability p of every row is one fact. A table shares where it has
   one parent (its rows are the parent's values or its row classes), at
   least `MIN_ROWS` rows, a quarter of its switches merged away
   (`MIN_MERGED`), and, where the parent reads no fact, at least two rules
   per row in the parent (`MIN_RULES_PER_ROW`). A shared fact read in two
   parts of a split table keeps them on a spine instead of side by side,
   so elsewhere sharing costs more than it saves. Rows that can hold
   together keep switches of their own (`examples/shared_rows.pl`).
8. **Certain by cover** (`cover.rs`; `examples/cover.pl`). A derived atom
   whose rule bodies together are valid is certain and leaves the bodies
   that read it. Validity is decided by merging bodies equal but for one
   literal, those literals being an exhaustive set. A table whose rows all
   hold one distribution then loses its parents, as ProbLog would read
   such a CPT.
9. **Conditioning on the evidence** (`condition.rs`, on the propagation of
   `propagation.rs`). Unit propagation over the completion, seeded by the
   evidence and the certain atoms, with each exclusive set read as at most
   one true atom and each exhaustive one (from the cover) as exactly one.
   A fact it fixes becomes certain, its weight a factor of `P(q, e)` and
   `P(e)` alike, which `GroundProgram::weight` keeps for
   `--evidence-probability`. A derived atom it fixes is observed and read
   as its value by every rule, which keeps each world's model where no
   derived atom depends on itself; a cyclic program is left alone. A
   derived atom it makes false whose rule has two open literals on facts
   (directly, or through an atom with one live rule) is a clause, and the
   facts of a strongly connected component of the clauses' implications
   become copies `g :- f` or `g :- \+f` of one, whose weight takes theirs
   (`Propagation::equal_facts`). This is what Ace gets from compiling with
   its evidence and c2d's unit resolution.

   Before the copies, a **SAT search** (`backbone.rs`, on the CDCL solver
   of `sat.rs`) reads the clauses the propagation leaves over facts, one
   solver variable per class of equal facts. Every world the evidence
   allows satisfies them, so a fact all their models fix (the backbone) is
   fixed as the propagation's facts are, and a clause of two literals the
   solver ends with joins the implications whose components become copies.
   The search is Janota, Lynce and Marques-Silva's with cadiback's
   flipping, in rounds of growing conflict limits (`CONFLICT_LIMITS`), then
   SAT sweeping over the literals that agree in the first 64 models, within
   a budget of propagation steps per literal (larger once a backbone is
   proved: they come in a cascade). It does not fix the facts of the gates
   (below) that are summed out. `closed` lays copied facts out below the
   clauses that read them, so `auto` puts `td` before `closed` for a CNF
   over its facts (`auto::candidates`).

   Then the **gates** of the clauses left: a fact `g` of probability one
   half with a clause `g ∨ \+l_1 ∨ ... ∨ \+l_k`, k ≥ 2, and the clauses
   `\+g ∨ l_i` is the OR of the `l_i` (an AND where the long clause reads
   `\+g`), a Tseitin gate. Since `Σ_g ½ [g = f(l)] R(g) = ½ R(f(l))`, the
   definition's clauses go, `g` becomes a derived atom with the gate's
   rules, and the half goes to `GroundProgram::weight`. Gates whose inputs
   reach them again are refused (the least model of `g1 :- g2. g2 :- g1.`
   is not `g1 ⇔ g2`), and only gates that nothing reads but other gates
   taken are taken: a gate read elsewhere would make a weighted CNF a
   program that reads a derived atom, which `td` lays out right-linear
   instead of folded (`ground::shape::reads_facts_alone`).

   Last, the **renormalization** (`renormalize.rs`). A row of a table, an
   annotated disjunction's chain of switches under one context (one
   assignment of its parents), that the propagation leaves some heads
   false and others not keeps the mass `m` of the heads it keeps and of
   what it leaves to no head. It becomes a chain over the heads kept with
   weights `θ_h / m`, and `m` a fact `w` that the context requires, a rule
   `bad :- context, \+w` with `bad` observed false; a row without a
   context puts `m` into `GroundProgram::weight`. Each world counted keeps
   its weight, `θ_h / m · m`, and the false heads leave the program, so
   the evidence walk reads the table's level without them: Ace's
   restriction of each table to the evidence, on the switch chains. A
   table whose contexts exclude each other pairwise (a network's table)
   shares its new switches and masses' facts across its rows, as switch
   sharing does. Where a context of the table reads a set of
   `ground::BITS_FROM` values or more, which the indicators read in
   binary, `w` goes first in each row's chain of the heads it keeps, and
   the masses' rules go on one of the table's own false heads, which stays
   observed false, in place of an atom of their own: each row's false
   heads merged into one, in the table's own unit. Read through an atom
   of their own, the masses of a table over a set read in binary can
   exceed memory where a copy per value of the set compiles; elsewhere the
   atom of their own compiles smaller. A row that keeps a head the
   propagation made true, an observed value, is not renormalized: the
   head's observation carries the row's evidence, and with the mass's
   constraint beside it an observed child of two sets read in binary
   (tests/it/binary.rs) does not compile in memory. In a table whose
   contexts exclude each other pairwise and read no set in binary, a row
   that keeps that one head `h` becomes `h :- context, t`, `t` a fact of
   weight `θ_h` shared by the rows of one `θ_h`, since `h` holds where the
   context does through that row alone; elsewhere the row stays as it is.
   The renormalization pays on Ace's networks, whose zeros make values
   false; `GroundOptions::renormalize` turns it off.
10. **Dropping detached evidence** (`detach.rs`). The components of the
    graph of rule heads and body atoms share no fact, so the count of a
    component holding observations and no query is a factor of `P(q, e)`
    and `P(e)` and cancels, unless it is zero; `--evidence-probability`
    grounds with this step off (`GroundOptions::detach`). A DPLL over the
    completion, on the conditioning's propagation with a trail, looks for
    a world in which the component's observations hold, checked by
    evaluating the rules forward. A component with such a world loses its
    observations, and the relevance pass its rules; one with none found
    within `VISITS_PER_LITERAL` keeps them. A cyclic program is left alone.
11. **Defined facts** (`defined.rs`, `implied.rs`, `support.rs`), where
    the program is then a CNF over its facts
    (`ground::shape::clauses_over_facts`: every derived atom a copy of a
    fact or an atom observed false that nothing reads).

    First the clauses of four literals or more that the others imply
    leave, the longest first, each found where `F \ C ∧ ¬C` has no model:
    by unit propagation first, then by a search of at most 100 conflicts
    on those propagation keeps, under a budget of propagation steps, with
    guards on a window of clauses at a time. The models do not change, and
    the primal graph loses the cliques of long clauses, which can hold
    `td`'s width far above the rest's. A clause is tried only where it
    joins two variables that no clause known to stay joins (those of three
    literals or fewer, and those found not implied): one that leaves the
    primal graph as it is stays, since an implied clause can help the
    compile.

    Then an independent support of its clauses, by Arjun's backward search
    on Padoa's theorem (Soos and Meel, ICCAD 2022) on the solver of
    `sat.rs`: the CNF twice, `F(x) ∧ F(x')`, with an equality
    `x_i = x'_i` guarded by an assumption per candidate, and a candidate
    `y` leaves the support where `y ∧ \+y'` has no model with the others'
    equalities assumed. The candidates are the facts of probability one
    half that no query reads, tried in Arjun's order, the fewest
    occurrences of the rarer sign first; the other facts stay in the
    support. A candidate outside it is a function of the facts in it, and
    is eliminated by resolution where its resolvents leave a minor of the
    primal graph: every pair of facts a resolvent joins that no clause
    joined has one fact in common, into which the eliminated one is
    contracted, so that the treewidth does not grow (a bound on the count
    of clauses, as SatELite's, lets it grow). Each model of `∃y F` extends
    to one of `F`, whose `y` weighs one half, so
    `Σ_y ½ F(x, y) R(x) = ½ (∃y F)(x) R(x)` for any `R` that does not read
    `y`, and the half goes to `GroundProgram::weight`. The gates step 9
    takes are the syntactic case; a gate it leaves because another clause
    reads it can go here. The resolvents and clauses that the others imply
    then leave as before the search. The clauses that stay keep their
    atoms; each resolvent is an atom `$resolvent(k)` observed false.
    `auto` then chooses its trials and vtrees on the program with fewer
    facts and clauses.
12. **Linear recursion without recursion** (`eliminate.rs`;
    `examples/cycle_graph.pl`), after the grounder, through
    `without_linear_recursion`. A recursive component whose rules each
    read at most one of its atoms, positively, is reachability: a node per
    atom and a source `s`, an edge `b → h` labelled `L` for a rule
    `h :- b, L`, and `s → h` for a rule that reads no atom of the
    component; `h` holds exactly when a path from `s` to `h` has every
    label true. Eliminating node `k` adds `label(i → k) ∧ label(k → j)` to
    `label(i → j)` (Floyd and Warshall, one world at a time), and then
    `v_k ⇔ A(k) ∨ ⋁_{j after k} v_j ∧ L(j, k)`: each atom reads only atoms
    eliminated after it, so the program is acyclic, and the atoms keep
    their names (a label of several bodies becomes an atom `$e(n)`). The
    order is greedy on fill, the atoms read outside the component last;
    past `MAX_PRODUCTS` label products the rest stays recursive. Steps 5
    and 8 to 11 then run again. This is Gaussian elimination for the
    algebraic path problem (Tarjan, JACM 1981) used as cycle breaking, for
    linear positive recursion only: unlike Janhunen's translation of
    normal programs (ECAI 2004) it adds no rank variables, and unlike
    ProbLog 2's proof-based cycle breaking no per-context copies. It is
    faster than the compile's Tp iteration on the same program on the
    random graphs and grids, which the iteration can run out of memory on.
    `tddlog --ground --vtree closed examples/cycle_graph.pl` prints the
    acyclic program.

## Queries in groups (`src/solve/split.rs`, `split_check`)

The compile builds the evidence and every query over the relevant program,
and one layout serves them all. `P(q | e)` needs only what `q` and the
evidence depend on: in a Bayesian network that queries every variable,
`P(x | e)` reads the ancestors of `x` and of the evidence, while the whole
network ties together the parents of every unobserved child.

`solve` splits the queries (`src/solve/split.rs`) and runs each group,
from the rewrites on, on its own part: the program restricted to what the
group's queries and the evidence depend on, finished as the grounder
finishes a program. The answers go back in query order; `P(e)` is the
first part's. A query that another query or the evidence depends on goes
with that query, or with the first group where the evidence reads it; each
other query heads a group with the queries of its strongly connected
component. A group joins a larger one that reads every fact it reads (the
values of a network's variable): the facts are the layout's variables, and
the join adds none. A program with more than `MAX_GROUPS` heads, one read
through indicators or one observing an atom that cannot hold is not split.

Three bounds decide (`SplitBounds`), once the queries make two groups or
more. First the floor: the program's `families:0` layout must be
estimated to hold 2^40 pairs or more (`auto::families_estimate`; below,
it compiles in seconds). Under `auto`, whose first trial on most programs
is `families:0`, that trial's program and layout are built before the
check, which reads their estimate, and a program kept whole runs with
them. Then a filter: the min-fill weight of the atoms' interaction graph,
log2 of the sum of 2^k over its eliminations (`vtree::min_fill_weight`),
must be 2^40 or more and the parts together at most a quarter of it.
Last the parts' searched layouts, each charged 2^20 pairs for the fixed
cost of a run, must be estimated at a quarter of the program's pairs or
less, since min-fill over atoms misleads where the searched decomposition
is much narrower. The floor comes before the weights, which cost more
where it refuses the split. The tests in `src/solve/tests.rs` check the
parts' answers.

## Indicators and clause tables (`src/indicators/`)

Tp-compilation composes every derived atom away: an atom's formula is over
the facts alone, and an atom read by two components has its formula
conjoined into both. In a Bayesian network a node's formula is then a
function of every switch among its ancestors, and no vtree bounded by the
network's treewidth holds it. An indicator rewrite reads an atom `k` in
each other unit `u` through a fact `v_{k,u}` of its own, of weight one on
both sides, tied to the atom by a link

```text
d_{k,u} :- v_{k,u}, k.
d_{k,u} :- \+v_{k,u}, \+k.
```

which the program observes true. In each world of the facts the links
admit one value of the copies, the atoms' values, so the count of
`q ∧ e ∧ ⋀d` is `P(q ∧ e)` and the query ratio is unchanged.
`examples/bn_split.pl` shows a split table (below).

- **Units** are the merged components of `ground::deps::components`,
  joined again where they hold atoms of one exclusive set. A rule that
  reads a member of a set positively also reads the other members' copies
  negatively; without it a table's formula separates rows that two values
  of one parent would switch on together.
- **Which atoms.** `introduce` gives a copy to every atom another unit
  reads; the families layouts read the program so (`Kind::indicators`),
  the others read it as grounded.
- **Table splits** (`table_splits`). A table over its parents' copies pays
  about r²/2 pairs for r rows. Where every rule of a unit reads exactly one
  value of a parent P, each value's rules read the other parents through
  copies of their own: |P|² + Σ r_a² against r², taken where it costs at
  most half (`SPLIT_GAIN`), on the cheapest P. Parts are split again where
  that pays, each copy priced at `COPY_COST` pairs.
- **Parts side by side.** Where P has `OWN_VALUE_FROM` values or more, at
  most one table of the unit is read from outside and no fact is read in
  two parts, a part reads P's copy of its value alone, and the layout puts
  the parts in a balanced tree, |P| log₂|P| instead of a spine. The layout
  (`side_by_side` in `nest`) puts parts of a block side by side where each
  has a copy that all its rows read positively and no row of another part
  does, and every copy below them is read in one sign by every row. A node
  over some of the parts then has a class for whether one of them holds
  and what it chose, where on a spine it had one for every part still to
  come. Parts that read the other values' copies negated stay on the
  spine.
- **Wide shared parents.** A table that holds no switch (a unit of row
  classes) or whose rows read P alone is not split on a P of `BITS_FROM`
  values or more that two units or more read; it reads P in binary
  (`Unit::read_in_binary`): ⌈log₂(|P|+1)⌉ bits instead of a copy per part.
  A table whose rows hold switches and read other parents is still split.
- **Chained links.** The copies of one atom in the parts of a split table
  after the first are tied to the previous part's copy, an equality of two
  variables, rather than each by a link to the atom.
- **Sets read in binary** (`Bits`, `Encoding::codes`). A unit or part
  that reads a set of k ≥ `BITS_FROM` values only positively, and not as
  its own value, reads ⌈log₂(k+1)⌉ copies of derived bits of the set's
  unit instead of k copies. Value i has the code i + 1 and 0 is none of
  them; bit j's rules are those of the values whose code has it, and a
  reader reads a value as the cube of its code. A bit's rules read the
  sets they read without the other copies negated: its link's loops of
  clauses hold their at-most-one. Two rules of a bit that differ only in
  one bit of the code of a value they read in binary are one rule without
  it (`BitMerge`), where that leaves at most half of the bit's rules; a
  merged rule keeps the place and literal order of its first row, which
  the families layout reads. The families layout puts the bits' formulas
  and copies with the set's values.

**Clause tables** (`clause_tables`). A table whose rows provably partition
its inputs (`partition`, with `covers` splitting on an input every row
fixes) is tied to each reader by one link holding, as clauses, each row
with the reader's copy of its value false, exactly one of the reader's
copies, and at most one member of each input set (exactly one where the
set is a table whose every value is read); the program observes the link
false. The at-most-one of three copies or more is one rule of pairs
(`GroundRule::pairs`). A row reads a set's copy of its value without the
other copies negated, which the link's at-most-one implies, unless another
rule reads the set's copies; then the links read them too, since copies
read both ways are laid apart. The dual builds the link's negation as a
conjunction of clauses with no negation, and the table's own rules are not
compiled. The exactly-one clauses change no answer and are needed: without
them the query phase separates impossible input combinations. Every
`auto` trial with indicators uses clause tables. A table of more than
`MAX_CLAUSE_RULES` rules keeps per-copy links to its rules' formulas
unless its rules times values are at most `MAX_CLAUSE_CELLS`.

**Readers tied** (`tp/tie.rs`). The rows partition the inputs, so two
readers' links together hold where one of them holds and the two readers'
copies are equal: `link_A ∧ link_B ≡ link_A ∧ ⋀_v (c_B,v ⇔ c_A,v) ∧
exactly-one(c_A)`. A reader whose link the evidence walk puts at or above
the node of another reader's link gets that tie in place of its link, at
its link's node, and the link is not built; the walk weighs the tie as the
link it stands on. A link of fewer than `TIE_FROM` clauses, or at the
root, is built. Put where the two readers' copies meet instead, the tie
would meet the copies below that free of the inputs.

**Network files.** A network's row whose gap from one the rounding of its
entries explains (`0.3333333` three times) is rescaled
([Network files](#network-files-srcnetwork)); a row like `0.5, 0.499` is
kept, and its missing mass goes to no value as in ProbLog. Rows that do
not sum to one fall outside the clause encoding.

## Vtrees (`src/vtree/`)

The vtree's leaves are the probabilistic facts (with indicators, the
copies too). A structural TDD keeps a node at every vtree level whether or
not the function depends on the variables below it, and a node at vtree
node `t` partitions the assignments to the variables under `t` by the
function they leave outside. Two consequences decide the strategies.

**Direction.** Over a right-linear vtree, the level under leaves
`x_i..x_n` has one node per cofactor with those fixed, so a right-linear
TDD over an order is the quasi-reduced OBDD over the reversed order. A
formula compiled bottom up wants its leaves in reverse dependency order;
the forward order can be far larger. At a vtree node whose facts are read
by atoms that also read atoms decided outside it, a class is a function
from the outside values to the inside ones, doubly exponential in the
atoms read across the cut the wrong way (McMillan's bound for BDDs of
circuits, read at every vtree node). A tree decomposition of the
undirected interaction graph does not see the direction. So `td` and
`dynamic`'s start are right-linear over an order, `closed` branches only
where no subtree has inputs, and `families` branches over a program whose
indicators give each group its inputs as leaves of its own. Where every
rule reads facts alone, or copies of facts (`reads_facts_alone`), no
formula composes a derived atom and no class reads an atom decided outside
its node: there `td` and `dynamic`'s start are the decomposition folded.

**TDDs are not SDDs.** An SDD needs only the primes at a vtree node to be
disjoint; a TDD partitions both sides of every node. A vtree that suits an
SDD does not suit a TDD.

**Tree decompositions** come from goatd, a single-threaded
tree-decomposition library (a dependency): min-fill and min-degree
elimination, and a portfolio (FlowCutter separators, nested dissection,
refinement) that runs under one budget and keeps the narrowest. goatd
knows nothing of vtrees; `vtree/` builds the graphs, reads a fact order or
a folded tree off the bags, and sets the budgets. goatd's budgets are
deadlines; tddlog arms its work meter (`goatd::meter::arm`), so a budget
is graph work in goatd's units and the decomposition is the same on every
machine. An elimination order stops being exact at
`ELIMINATION_BUDGET_UNITS`, since exact elimination is cubic in a dense
graph's degree.

The layouts (`Kind`, `build`), each a trial of `auto`:

- **`td`** (`td_forest`, `fold`): a tree decomposition of the interaction
  graph, the facts read off it into a right-linear order: each fact at the
  topmost bag holding it, a bag's facts after its subtrees', rooted at the
  bag holding the last-consumed fact. Where rules read derived atoms, the
  bushy fold the decomposition argues for compiles far slower than the
  right-linear order.
  Where every rule reads facts alone, or copies of facts, the fold is the
  vtree (`td_layout`): a node's classes then follow the facts its subtree
  shares with the rest, at most a bag, where a cut of the right-linear
  order holds every fact alive on a root path.

  There the fold also copies separator facts (`separators.rs`,
  `ground/fact_copies.rs`), and the program `td` compiles is the one with
  the copies (`Prepared::lay_out`; `tddlog --ground --vtree td` prints
  it). The atoms that cross a unit of the fold (a bag's subtree, or a join
  of some of its children) read the facts outside it, its separator, and
  the evidence's level at the unit's root holds a node per function of
  the separator that an assignment of the unit leaves, which can be many
  more than the separator's points. A copy of a separator fact, of
  probability one half, laid atop the unit, is read in the fact's place by
  the crossing atoms and tied to the fact by two clauses observed false,
  whose weight `P(e)` takes back out; the unit's root level then holds at
  most a node per point of the copies. The rule (`CopyRule::ON`) copies a
  unit where the facts its crossing atoms read inside it, and the parts of
  their rules outside it, are each at least twice its separator, and where
  the unit holds at most four times as many facts as the separator:
  copying every unit lost instances of circuits' weighted CNFs that the
  rule answers.

  The decomposition is a min-degree elimination where that reaches the
  graph's lower bound on the width (`minor_min_width`), else a min-fill
  one where that does, and only then goatd's portfolio
  (`portfolio_to_bound`), whose floor of work a small program would
  otherwise pay, under a budget that scales with the graph between
  `MIN_TD_BUDGET_UNITS` and `MAX_TD_BUDGET_UNITS`, seed 0; the narrowest
  is kept. The two eliminations share goatd's reductions of the graph
  (`goatd::elimination::Prepared`). Min-degree, the cheapest, goes first,
  and where it does not finish within `ELIMINATION_BUDGET_UNITS` the
  vertices it left are one bag, not a decomposition: `td` cannot help,
  `build` returns an error, and `auto` disables the trial. The portfolio
  runs only where its hard cutoff, twice its budget, is more than the
  reductions and min-degree charged; below that its own eliminations stop
  part way and leave their vertices in one bag.
- **`closed`** (`closed_forest`): components in the order of a local
  search on a size proxy (`src/vtree/order.rs`: simulated annealing over
  dependency orders, components sharing switches merged first; skipped
  where 64 facts or fewer bound every formula, and where its budget of
  steps pays for fewer than four moves per component). The search starts
  from the compiler's order taken root by root, and keeps that order
  where it is skipped: where the roots' cones (a root is a component
  nothing reads) share no component, the components are grouped by
  root, the roots ordered so that each next adds the fewest facts to
  those of the roots before it among those that share one with them.
  Observations of one parameter are then read together rather than
  interleaved with another's. Where cones share components the
  compiler's order stays, since it builds each shared component among
  its first reader's own and moving them apart changes which blocks
  absorb which (zeroone's compile took up to 13 times the work). Each
  component is a block with its new facts on a right-linear spine above
  a balanced join of the blocks it reads. Blocks that share nothing and
  are read by one component are siblings, the only place the tree
  branches. A join whose rules pair up the blocks' atoms one to one
  (UW-CSE's `advised_by(S,P), student_nb_publications(S,_)`) is not
  taken.
- **`families`** and **`families:0`** (`families_forest`, `nest`): for a
  program rewritten with indicators. A *family* is a formula the query
  phase conjoins (a link, a derived query or evidence atom); *groups* are
  families merged by the facts and copies they share. A fact that the
  rules of two atoms or more read positively, a *hub*, merges no families
  where its readers are apart without it; the hubs one rule reads together
  are a group of facts alone. A group whose rows fall into `MIN_BRANCHES`
  or more branches, rows that share neither a fact nor a group they read,
  is a chain of its branches, each placed with the groups it reads, so
  that a disjunction over independent parts does not make them one
  clique. The moral graph of the groups is decomposed and each group's
  block goes in the highest bag holding it with its parents. Within a bag
  the items are joined pairwise by the fewest values left open, then the
  fewest leaves.

  Each of two group numberings is decomposed by min-fill, by goatd's
  portfolio where min-fill's width is `FAMILIES_SEARCH_FROM_WIDTH` or
  more, above the graph's lower bound, and min-fill charged less than the
  search's hard cutoff (twice its budget; past it the search's own
  eliminations stop part way), and by eliminating the groups in numbering
  order where that is narrower (a grid's anti-diagonals). The first
  numbering is also decomposed by min-fill weighted by the groups' values
  (`weighted_min_fill`: a fill edge weighs the product of its ends'
  values), where the values differ, and, where the portfolio runs and the
  cheapest layout of both numberings pays for it (`anneal_budget`: a unit
  of work for every 8 pairs of its cost, up to twice an elimination's
  budget, and at least 32 moves a vertex), by simulated annealing over the
  elimination order (`annealed`), from the better of the two min-fills by
  their states, the sum over the eliminations of the product of the
  clique's values. The layout with the least `Interfaces::layout_cost` is
  kept. `families:0` lays out the min-fill decompositions alone. Within a
  block (`nest`) the copies every row reads go at the bottom, the rows
  split into parts by shared facts, and where every head reads a part's
  facts under one context, as a row's switches, those facts go in a
  balanced tree beside the copies. The layout cost doubles as an estimate
  of the evidence's pairs (`Estimate`), which `auto` reads.
- **`dynamic`**, or `dynamic:minfill` (`facts_by_minfill`,
  `Compilation::dynamic`, tididi `Engine::pool_search`): right-linear over
  a pre-order of the min-fill elimination tree of the interaction graph (a
  vertex per ground atom, a clique per rule, the head left out of the
  clique of an atom observed false that nothing else reads), changed
  during the compile. The pre-order has the reverse elimination order's
  fill but finishes each subtree before the next, where the reverse order
  put every disease of a noisy-or network above every link. Where every
  rule reads facts alone, the start is the elimination tree folded
  (`min_fill_forest`).

  Between steps (a component, or a round of a cyclic component's
  worklist) a greedy search of rotations runs over every stored formula on
  the shared global vtree. A move is a left or right rotation, optionally
  crossed (the promoted node's children swapped first); a TDD level is
  symmetric in its two sides, so a swap alone changes no size, and the
  crossed rotations are the moves that change the leaf order. The first
  search runs at `DYNAMIC_FIRST_PAIRS` live pairs (CUDD's first reordering
  is at 4 004 nodes), the next whenever the live pairs have doubled. A
  search sweeps until a sweep keeps nothing. Its budget is the work the
  compile spent since the last one, and no more than its work so far per
  finished component times the components left; what the search spends
  past it, the compile spends before the next search runs, unless the
  live pairs double again first, so searching costs at most twice what
  compiling does, plus the last search's excess. A budget that stopped
  searches part way made the outcome depend on where they stopped (a 2%
  change of budget made single programs several times slower or faster).
  A search that finds little ends within a sweep or two, and what it spent
  past the budget holds off the next search. A formula is dropped once the
  last component whose rules read it is finished (`dying_atoms`), so the search
  does not fit the vtree to formulas the compile is done with. A dynamic
  compile is unprojected, since a projected scope stops being a projection
  after the first move. A stopped compile resumes inside a cyclic
  component at the update that stopped.
- **`program`** (`auto`'s first trial where views were inlined, its last
  elsewhere): right-linear over the facts in the reverse of the order the
  program states them. A right-linear vtree over x1 .. xn is read as an
  OBDD reading xn first, so the diagram reads the facts in program order.
  A bit-blasted circuit exported with its flips in creation order
  (Alea's) is then laid out in the variable order its BDD package built it
  in, and on some circuits this is the one layout that compiles. On
  programs with something to order it is `auto`'s first trial where the
  grounder inlined views, which the exported circuits all have and the
  networks, grids and graphs do not, and its last elsewhere, where it
  takes its share of the work until another trial wins.

  Where the order has narrow cuts, `program` and `program:fanin` name the
  derived atoms that cross them (`vtree/naming.rs`). The width of the
  boundary after a position is the number of atoms at or before it that a
  rule of a later atom reads. From the last cut, a window opens at the
  first boundary after a derived atom whose width is at most twice the
  facts since the cut, and runs until those facts double; the cut is the
  narrowest boundary in the window, and a window whose narrowest boundary
  is wider than `WIDEST` (20) has none. A reader after a cut reads the
  atoms crossing it through indicator copies, the segments between the
  cuts being the units, and each segment's copies are laid out at its
  start, before its facts. A formula then holds its own segment's facts
  and the copies at its start, and the evidence holds the links, the
  history, once. On a Markov chain such as GeNI's weather the cut is the
  day's frontier, the 11 state bits and the goal of 42 facts a day.
- **`program:fanin`** (`auto`'s second trial where views were inlined,
  absent elsewhere): right-linear as `program`, over the facts in the
  fan-in order of BDD variable ordering (Malik, Wang, Brayton and
  Sangiovanni-Vincentelli, ICCAD 1988), interleaved over the roots, the
  observed and queried atoms, as Fujii, Ootomo and Hori interleave it
  (ICCAD 1993; `deps::fan_in`). Each root's cone is walked depth first,
  each atom's rules' reads entered as their bodies list them, and an atom
  is placed once the atoms it reads are, a fact as the walk meets it. An
  atom an earlier root's walk placed is not entered again but marks where
  the walk is, and the next atom placed goes right after it. The roots go
  deepest first, by the longest chain of rules below each. Where a front
  end states each number's bits together, as Alea's export does, the walk
  puts the bits of the operands a gate combines side by side, an adder's
  or a comparator's bit by bit, and those of an operand that several
  operations read beside each of the others' (conjugate_gaussians adds μ
  to two noises; a single walk on from the first sum put the second noise
  after both). The walk loses where the file states the facts in the
  order of a model's time (a hidden Markov model, trueskill, a grammar, a
  grid of link failures). Since neither order wins everywhere, `auto`
  tries both where views were inlined. Every atom comes after the atoms
  it reads, which the names along the order need (above): an atom that
  reads one this walk placed after its anchor and one an earlier walk
  placed before it moves to right after the last atom it reads. The
  grounder sorts each ground body by atom id, so the walk reads a rule's
  atoms in the order they were made rather than as the rule writes them.
- **`decision`** (`src/vtree/decision.rs`): the tree of the decisions the
  conditioning on guards read facts in (`ground::Decisions`, `auto`'s
  `decision` trial, below), each fact above the facts it decides between,
  with the facts the decisions leave out right-linear below the root's
  right, in the fan-in order of the shallowest roots first
  (`deps::fan_in`, above): a data point's facts then stay together on the
  spine, where deepest first put the data points' noises beside the mean
  at every bit. A program the grounder read no fact of as a guard has no
  such tree, and the layout fails.

  The layout names two kinds of atoms, each read through a variable of its
  own tied to it by an observed link, as `program`'s names are: each
  context of the tree that a rule body holds the cube of, and the contexts
  above it, `c_S :- c_S', l`, its name at the top of its subtree, read in
  place of the cube; and the atoms of the tree that atoms reading facts
  outside it read (the bits a mux tree selects), their names above the
  lowest node holding their cones. Without the context names an
  if-then-else over two values in disjoint subtrees holds their product,
  `4^k` pairs for a mux tree over `k` selector bits. The names of the
  tree's atoms above each node are a cut, which settling keeps or composes
  away (below); the contexts keep their names. Where nothing outside the
  tree reads it, nothing is named: the names are then variables the
  compile holds apart until the evidence conjoins their links. Alea.jl's
  caesar is such a program, its key and letters trees of decisions that
  only their sums read.

`ground::shape::nothing_to_order` holds where the derived atoms are one
merged component, or one recursive component reads at least half the
facts (a reachability relation grounded from its source). There no
indicator cuts anything the component's diagram does not hold, and no
component order matters.

## `auto` (`src/auto/`)

`auto` does not choose a layout ahead of the compile: it runs a few as
trials and keeps the first that finishes. A trial is a layout, a
`vtree::Kind`, compiled on the program as that layout rewrites it
(`Kind::rewrite`): `families` and `families:0` read it through
indicators, the others do not. The trials take turns: each turn lets one
trial's compile or query phase go on until it has spent a target of
engine work or would complete a formula larger than a watch in pairs, and
a trial's target doubles once it has spent it. A trial wins only after
its compile, evidence and all queries complete; the first complete trial
answers. This section gives which trials a program gets, the two trials
that rewrite the program further (`program`'s and `decision`'s), how the
turns go, and what happens when memory runs out.

A program gets these trials (`candidates`), in order:

| program | trials |
|---|---|
| `nothing_to_order`, an atom recursive | `dynamic` |
| `nothing_to_order`, every rule reads facts alone | `td`, `dynamic`, `closed` |
| `nothing_to_order`, otherwise | `dynamic`, `closed`, `td` |
| a CNF over facts the conditioning copied | `td`, `closed` |
| otherwise | `program` and `program:fanin` where views were inlined, `families:0`, `closed`, `td` where no view was inlined, `families`, `program` where no view was inlined |

The CNF over facts is the conditioning's (Grounding, step 9). `closed`
comes before `families:0` where the program has `FEW_FACTS` facts or
fewer and `DERIVED_PER_FACT` derived atoms per fact or more (the families
rewrite then copies many atoms per fact for a trial that `closed`
beats). A trial with indicators ties tables by clauses.

**The `program` trials.** A right-linear vtree over the facts in the order
the program states them, as an OBDD reads them (Vtrees). `program` reads
the program ground with its views in place (`Kind::ground_options`;
`auto` keeps the grounding before its views are inlined and finishes it
again for them, `Programs::of`): a view inlined into its readers moves
its facts to them, and a grid's boundary chains, views over facts, are
then read by every later row, which widens the cuts and leaves fewer of
them to name. `program:fanin`, whose order comes from the rules, reads
the inlined program, which serves it better. `program` comes first where
the grounder inlined views (Grounding: atoms of one rule read once), the
gates of a circuit built gate by gate, and last elsewhere: first on every
program, it costs the networks and grids, which have no views, several
times their time. Where it comes first, `program:fanin`, the facts in the
order the gates read them, comes second. A front end that bit-blasts a
program states the bits of each variable together and the variables in
the source's order, the order Alea and Dice hand CUDD and their own
diagrams. On circuits whose every other layout runs out of memory it may
be small, and `program:fanin` compiles Alea's adders in a hundredth of
`program`'s work (Vtrees). Their layouts cost nothing to build; as the
last trial `program` has its turn after the others, and as the first the
two and the trial after them start with short turns (`LEAD_SHARE`,
Turns). Where they come first, `td` is no trial: its order comes from a
decomposition of the undirected interaction graph, which does not see the
gates' direction (Vtrees, direction), where the two orders the program
states and `closed`'s search over dependency orders follow it.

**The `decision` trial.** A program answered whole with at least
`DECISION_DERIVED` (4 096) derived atoms, to which the conditioning on
guards applies as ground (`may_decide`: acyclic, no pairs rules, and a
rule reads a fact within the range of one of its derived literals) and
on which it gets under way (the pass, run on the program as ground with
an eighth of its budgets, `TEST_BUDGET`, makes a quarter as many atoms as
the program has derived ones, `TEST_MADE`), is rewritten for `decision`
from the program ground with its views in place, which `program` takes
too (`auto::ready`). Where that conditioning makes at
least as many atoms as the program has derived ones, the rewrite is a
trial of its own, before the others, and shares its program with none of
them; it takes its turns with the weight of a families trial, and the
program is answered whole, without the split check, which would build the
`families:0` trial's layout first. A circuit that compiles under
`decision` alone makes many more atoms than it has derived ones; the
arithmetic circuits make fewer than they have, and their `decision`
compile is worse than the other trials' or runs out of memory. The test
keeps a program from being ground again for no trial, and the pass is
also charged for the cube literals it copies, at most `COPIED` (1 024)
per literal of the program. `--stats` prints a `decision:` line wherever
the program was ground again.

Where the conditioning records decisions, the conditioned program goes
to `decision` where it is a trial. Where it is not, it goes to the
trials whose layout the rules' text does not set only where the pass
made at least a quarter as many atoms as the program had derived ones
before the merge of equal gates (`auto::takes_conditioned`): `program`,
whose order is the one the program states its facts in, and `closed`
and `td`, which search theirs on the program they get. On the zeroone
circuits the pass makes an eighth as many atoms as derived ones, and the
conditioned program holds 45% more derived atoms than the program as
ground (bb_zeroone_w2_w25: 14 499 against 9 972); taken there, `auto`
took 25 to 28% longer on three of them. The other trials compile the
program as ground (`auto::Programs`; a `--vtree` layout compiles the program as
ground). `closed` and `program` take less work on the conditioned
program at about the same sizes. `program:fanin` reads its order off the
rules, a walk through the atoms they read, and the conditioning rewrites
what they read: a test of a number's bits becomes the cube of the flips
on its path, so the walk no longer meets each number's flips together.
`families:0` takes more work on the conditioned program, its indicators
on every atom the conditioning made. Beside `decision` the order layouts
compile the program as ground: on the conditioned program they get
further and hold their diagrams between turns, which takes the memory
`decision`'s query phase needs.

The conditioning on guards (`src/ground/guards.rs`, `ground::guarded`):
a rule `h :- f, x` reads `x` only where the fact `f` holds, so it may read
`x` conditioned on `f` in its place. A bit-blasted circuit reads its
selector flips in nested if-then-elses whose branches read the same flips
again, through shared sub-circuits and derived tests; conditioned on the
selectors above them, the branches no longer read those flips, and a
branch's diagram need not keep their values alive. Each derived atom is
conditioned once per cube of fact literals, the cube restricted to the
atom's cover: the range of facts its support lies in, less the widest
run of facts within it that the covers of its rules' literals leave out,
built bottom up. Keyed by the range alone, two frames in five on the
bit-blasted circuits evaluated an atom again to a value an earlier frame
had found under a fact the support does not reach. A definition under a
cube other than the empty one is an atom of its own (`$gd`). The pass
leaves the program
as it was past `GROWTH` times its size or `SPEND` literal visits per
literal. Chains of steps, a decision list folding a sequence of tests
into one value, are then written as balanced trees of their compositions
(`src/ground/steps.rs`), so that a chain costs its length times the
tree's depth rather than its square. What the conditioning read as
guards gives the `decision` layout (`ground::Decisions`).

**Turns.** Each trial has a weight, 8 (`FAMILIES_WEIGHT`) for
`families:0`, a `decision` trial, or the searched families trial in
`families:0`'s place (below), 1 otherwise. Where `program` leads,
`families:0` and `decision` hold it only where `families:0`'s rewrite
ties tables by clauses (`families_weighted`), which reads the rewrite
alone, so the layout waits for `families:0`'s first attempt. Weighted
otherwise, on the circuits that `program` leads, layouts over the
families took the weight's share for nothing. Each trial also has a target, the engine work
it may have spent by the end of its turn, at first its weight times
`FIRST_PAIRS` times `WORK_PER_PAIR` (2^28) units. Each attempt goes to
the trial whose target is least for its weight, the first in the
candidates' order on a tie, so the trials take their first turns in
order and later ones in proportion to their weights. The attempt may
spend the target less what the trial has spent; once the trial has spent
its target, the target doubles.

An attempt may complete formulas of up to its watch in pairs: a sixteenth
of the target (`WORK_PER_HELD_PAIR`), `FIRST_PAIRS` at least, or more
where a formula past the watch stopped an earlier attempt, since under a
smaller watch it would build the formula again and be stopped again. Such
a stop raises the watch, by powers of two, until it holds the formula
(`Trial::hold`), and leaves the target alone, so the trial goes on where
it is still the least: a formula's pairs need not follow its work, and a
watch tied to the target would give the other trials turns for nothing.
An operation in flight may output twice the watch in nodes, and a raw
conjunction sixteen times it in pairs. Where `program` goes first, it,
`program:fanin` and the trial after them start at a 256th of their first
targets (`LEAD_SHARE`): the circuits whose views were inlined are won by
one of the three. The trial left alone runs without bounds (`attempt`).
Every bound is in pairs or work, never seconds, so the winner is the same
on every machine. At equal shares the run spends on each other trial
about its weight's share of twice the winner's work, which costs most
where a late winner shares the run with trials that keep going.

**Turns by projected finish.** After each attempt that leaves a trial in a
phase, the trial records, at the attempt's target, the share of the phase
it has finished: in its compile, its components; in its query phase, its
steps, which are the evidence walk's joins (each makes one formula of
two, so the walk makes one of the observed factors in one fewer) and then
the queries answered. The records of its compile are dropped as its query
phase starts, but for the target of the last, at which the query phase
had finished none of its steps: the phase began since, so its first
record measures it over at least a doubling, and a trial that finished
its compile in a turn has a projection at the next. The share it
finished over the last doubling of its target projects the doublings it
still needs, (1 − f)/Δf, and so the work at which it finishes the phase
(`Trial::projected`). A trial that has not yet doubled its target in its
phase makes no projection, and one that finished no step over its last
doubling is projected never to finish. The records are by target rather
than by the work spent, which an attempt stopped by work overruns by a
little: by the work, a record a hair past half the last went unread and
the projection waited a doubling more.

Each turn goes to the trial projected to finish first for its weight
(`Trial::key`): its projected finish, counted from its target, where it
is projected to finish within the next turn, to `PROJECTED_DOUBLINGS`
(three) doublings past it; a projection further out, and none, count
three. Without projections the trials take equal shares for their
weights, as the doubling has them; a trial projected to finish soon
takes its turns ahead of the others, up to three doublings ahead. The
projections of a compile hold that far and no further: the chance that a
trial finishes in its next doubling falls with the doublings it is
projected out, down to that of a trial with no projection at about three
to five doublings. The bound is also what a wrong projection costs: no
trial goes more than three doublings ahead of another by its projection,
so the run spends at most eight times what equal shares spend, and a
trial that does not finish where it was projected to measures a slower
pace over that doubling and falls back.

The projections of a query phase hold less well. The walk's last joins,
at the root, are its largest, so a trial stopped near the end of its walk
has many joins over its last doubling and few left. A query phase is
measured only once it has run: where a trial projected to finish its
compile goes ahead, the query phase that follows runs within that turn
whatever its length. A projection is counted from the target to three
doublings past it, and one further out counts as none: given its weight
however far out, a trial in its compile at a few components per doubling
would go ahead of a winner that has reached its query phase.

**Turns that end on their projection.** Within a turn, a trial's compile
reads its projection at the end of each component (`tp::Horizon`): the
work the trial has spent times all the components for each one finished.
Where that is past `PROJECTED_DOUBLINGS` (three) doublings of its target,
the turn cannot finish the compile, and it ends there as one that spent
its target: the target doubles, and the other trials take their turns
before it goes on. A stop between components loses nothing that the next
attempt builds again. The projection is read once the trial has spent
the work of its first watch, 2^24 units, about a tenth of a second:
before it the components done are too few to project from. This
projection is linear in the work, where the one between turns is by
doublings: early in a compile whose components cost about alike the
share finished per doubling grows, so that by doublings such a compile
is projected dozens of doublings out, where the linear projection holds
it close to its finish.

**The searched layout.** The searched `families` layout is weighed
against `families:0`'s once in a run, when it is built (`weigh_searched`):
where it is estimated `SEARCHED_GAP` (4) times smaller, its trial moves
just before `families:0` with the weight, at the first target of that
weight, and has the next attempt, and `families:0` loses its weight, the
bet now being on the searched layout; otherwise `families:0` goes on, and
the searched trial weighs 1. Estimated about as large, the searched
layout makes no bet of its own. It is built at its own turn, after
`closed`'s and `td`'s first, or sooner where `families:0`'s layout is
estimated past what its turn may hold and its rewrite ties tables by
clauses (a Bayesian network's rows, `ties_tables_by_clauses`): after the
attempt of `families:0` that spends what the search charges
(`search_charge`: goatd's search units over `SEARCH_UNITS_PER_WORK`). So
`families:0` may still finish for less than the search costs, as in
renting until the rent paid equals the price. An attempt that would
spend past twice the charge stops at the charge (`search_rent`);
otherwise rent is paid in whole attempts, so that it is at most twice the
price: an attempt stopped at the charge loses the operation under way,
and the rest of its turn may not hold it again. The relational programs
with as large estimates (cora500, uwcse10) tie no table by clauses and
are won by `closed` or `td`.

**Outgrown estimates.** A families trial that has spent more than
`WORK_PER_ESTIMATED_PAIR` (64) units of work for each pair of its
layout's estimate loses its weight, tested after each attempt: the
weight bets on the estimate, and the families trials that win spend far
fewer units a pair than those that lose. Their formulas need not reach
the watch, which grows with the target.

**Refusals.** Memory goes to the trial that has spent the most. An
operation refused memory, by the allocator or tididi's byte budget,
while other trials hold what their attempts built, frees that
(`Trial::release`) where the refused trial has spent more work than each
of them, once a trial: each of them begins its phase again at its next
turn, with its layout, target, watch and work spent, and the refused
trial resumes at its next turn. Refused otherwise (again, behind a
trial that has spent as much, while no other trial holds anything, or
past tididi's 32-bit indices), the trial is retired (`Trial::retire`): a
later attempt would resume at the same operation with larger bounds and
no more memory to take. What it built is freed, and the run goes on with
the other trials; it fails with the refusal once none is left. So a
refusal discards less work than the refused trial has spent from each
other trial, and once: freed for whichever trial was refused, the others
paid for an early refusal of a cheap trial. The compile and query phases
copy a formula they keep wherever an operation would consume it, through
tididi's fallible copy (`TddBackend::copy`), so a copy the allocator
cannot serve is a refusal too: `Tdd`'s `Clone` would abort the process.

**Resumption.** A stopped compile keeps every finished component, and
within a cyclic component its rounds so far (`tp::Compilation`). The
component it stopped in starts over, but a loop of clauses over facts
alone that the watch refused is kept, reduced in full, and the trial's
next attempt, however many turns later, takes it rather than build it
again (`clauses::Refused`); the loop is no larger than the attempt's node
cap, twice its watch. This serves a component whose formula is one loop
over a whole lineage (TPC-H's q18). A stopped query phase keeps its
finished conjunctions, a pending reduction or count, and its answers
(`QueryEvaluation`); a resumable conjunction keeps its operands until
`and` succeeds, since `and` consumes them on failure. Both phases go
through `Evaluation` (`src/evaluation.rs`), which an attempt calls under
its limits and a single layout (`Strategy::Trial`) once without them.
Rewrites and vtrees are built lazily, and equal layouts over the same
rewrite are compiled once. A layout that cannot help disables its trial
when its turn comes to be laid out: a `td` whose min-degree elimination
does not finish (`build` returns an error), and a searched `families`
where `families:0`'s estimate says no search would run
(`Estimate::search_units` zero), whose layout would be `families:0`'s.

**Why this order.** `families:0` comes first and needs no search; with
`closed` first the networks pay for a trial `families` then beats. The
searched layouts come last because goatd's search has a floor of work.
`closed` and `td` win the relational programs (uwcse, cora). Each trial
has its own budget, so a trial that never wins first costs only its work.
Where nothing is left to order but no atom is recursive, the one
component is acyclic atoms that `ground::deps::components` merged because
they read the same facts; they still have a direction, which `dynamic`'s
min-fill start does not see and `closed` follows (Vtrees, direction), so
`closed` follows `dynamic`, and `td` follows `closed` (tuple-independent
databases). Where every rule reads facts alone (`reads_facts_alone`:
noisy-or networks, tuple-independent databases, a weighted CNF with its
clauses observed true), `td` comes first: both `td` and `dynamic` start
from a fold, but `td` compiles projected and the walk conjoins each
observation at the lowest node over its facts, where `dynamic` compiles
every formula on the global scope and conjoins every observation at the
root. The trials are chosen from the program as grounded and each compiles
the program written without its linear recursion (Grounding, step 12), on
which `dynamic` takes less work than `closed` on the random graphs and
grids and stays the one trial of a recursive component.

## Compilation (`src/tp`)

The TDD variables are the facts (and copies). Ground derived atoms are
split into SCCs (`ground/deps.rs`) and compiled in dependency order
(`compile.rs`). For each atom `Tcp(a) = OR over rules (cube of fact
literals AND formulas of derived body atoms)`; a negative derived literal
reads the finished formula of a lower component. An acyclic component
takes one application; a cyclic one is iterated until no formula changes.
[papers/tp-compilation.md](papers/tp-compilation.md) gives the operator
and its fixpoint as the papers state them.

**Worklist** (`Worklist`). A round takes the atoms whose bodies changed in
reverse postorder of the component's dependency graph, and a reader ranked
after an atom that changes joins the current round. Every atom carries a
version and every rule the versions of its body atoms at its last
application; a rule whose body has not changed is skipped. The fixpoint
test compares pair counts before it calls `equivalent`.

**Formulation.** The dual (`Formulation::Dual`) maintains `μ_a = ¬λ_a` and
applies `μ_a' = μ_a ∧ ⋀_r ¬body_r`: one negation per body with a derived
literal, none for a body of facts alone. Nothing is negated back: a query
counts `μ_q` and complements the count. A positive derived literal reads
`λ_b`, built on `b`'s own scope once per version of `b`. An acyclic atom
of one rule is stored as `λ_a`, its body's conjunction, under either
formulation (`conjoined`, `Compiled::positive`): stored as `μ_a`, it cost
a negation of the conjunction and one back for each positive read; stored
so, only a negative read negates it, once, into the same cache. A body of
one derived literal reads the literal of the other sign as stored or
cached rather than negating the literal's formula. On a bit-blasted
circuit, two thirds of whose atoms are gates of one rule, the two cut the
negations by almost three quarters. A read is a copy of the stored or
cached formula, since the operations consume their operands. In a run
without a watch (a single layout, or `auto`'s trial left alone) the last
read of a formula, in the last component that reads it, takes it and its
cached negation instead (`last_reads`): the reads are counted per body of
that component's rules, and a read for several bodies at once (a factored
literal) counts for each; the copies for the earlier readers remain.

The primal is the default under `dynamic` (`Kind::formulation`), where the
search ends on smaller vtrees; it applies `λ_a' = λ_a ∨ ¬⋀_r ¬body_r` as
`¬(¬λ_a ∧ ⋀_r ¬body_r)`, keeping the conjunction as `¬λ_a'` for the next
update. tididi's overlay `or` would disjoin the bodies without negation;
the compile does not use it, since the work clock charges it below its
cost and it is slower on the graphs.

**Rule application.** An atom's changed rules are applied together
(`Factoring`, `factoring.rs`):

- **Factoring.** With `L` the literal in most bodies and `M` every literal
  the bodies with `L` share, `⋀_r ¬body_r = ¬(L ∧ M ∧ ⋁_{r∋L} (body_r \ L
  \ M)) ∧ ⋀_{r∌L} ¬body_r`, recursively, the recursion returning the
  disjunction directly rather than a double negation. A literal shared by
  k bodies is conjoined once. A branch whose bodies are facts alone is
  built on its parent's scope projected to those facts where that keeps at
  most nine tenths of the parent's variables (`NARROW_TENTHS`).
- **Bodies of facts alone** (`clauses_of_facts`, `one_by_one`,
  `clauses.rs`) are not factored, since each factored term costs two
  negations: they are conjoined as clauses. A loop of two bodies or more is
  looked up first in the `ClauseMemo`, keyed by its bodies in the scope's
  local variables, and a hit is copied (`TddBackend::transport`): a table
  read by several nodes is the same loop once per reader. A miss that is a
  table's link with no more models than literals is built from its models
  in one pass (`rows::table_models`, `TddBackend::of_models`); any other
  is built level by level by `TddBackend::none_of` (`none_of.rs`): that
  none of the cubes holds and at most one of each exclusive group, in one
  pass over the scope's levels, within `NONE_OF_STEPS_PER_LITERAL`. Where
  the builder gives up, the clauses are conjoined one at a time
  (`and_clause`, which negates nothing), with a contraction every
  `CLAUSES_PER_CONTRACTION` clauses, doubling while a contraction keeps
  more than nine tenths.

  The builder first merges each two cubes that differ in one literal's
  sign alone, `c ∧ x` and `c ∧ ¬x` into `c` (`merge_complements`): a set
  read in binary names each value by the cube of its code, so a sum over
  digits pairs every code with every digit. The cubes are held as lists
  with a hash summed over their literals (`CubeMerge`), so the cube a
  flipped literal makes is looked up in constant time. At a node whose
  left child is a leaf of no group and whose cubes there all have a
  literal on the right, a class of the right that none of those cubes
  reaches passes up as it is (a *lift*). A table's switches lie so on a
  spine above its parents' copies, about r²/2 pairs for r rows. A TDD's
  nodes at a level are disjoint classes of its assignments, so a node over
  the switches of several rows without the copies that select them has a
  class per function from those rows to the head's values, and a balanced
  tree of the rows is much larger. A spine is shorter only where each row
  has a selector of its own: the row classes (Grounding, step 7).
- **The or-route** (`Factoring::or_route`). A body of two derived parts
  `a`, `b` goes into the head's formula `acc` as `(acc ∧ ¬a) ∨ (acc ∧ ¬(b ∧
  facts))`, the disjunction as `¬(¬x ∧ ¬y)`, rather than through the
  product `a ∧ b`, which can be far larger than the formula it is negated
  into: at once where `acc` holds at most half the pairs of either part
  (`OR_ROUTE_SHARE`), otherwise once the product outgrows a work bound
  (`PRODUCT_WORK_PER_PAIR`, `PRODUCT_WORK_PER_FORMULA_PAIR`).

**Projection** (`cone.rs`, `Projection::Cones`). A structural TDD over a
few facts of a program of thousands is nearly all padding: cora500's
largest formula is 16 186 pairs for a function of 13 of its 2 216 facts,
and 101 pairs on a vtree over those 13. Each component compiles on the
global vtree projected to its ancestor cone, scopes shared by cone while
a formula on one lives (`ScopeTable`, which the query phase uses too),
formulas moved up with `embed` (a copy, no apply). The `Projector`
(`projector.rs`) projects in time proportional to the variable set, from
lowest common ancestors in a range-minimum table over pre-order depths. A
component whose cone holds `DEFAULT_MAX_CONE` (half) of the facts or more
keeps the global vtree; cones grow along dependencies, so no formula is
embedded downwards.

A cone over-approximates the support, and in a bit-blasted circuit by
far: a gate the evidence fixed or a term that cancels leaves its readers'
cones holding thousands of facts nothing depends on. So a finished
component whose scope holds `NARROW_FROM` (1024) variables or more, and
whose formulas depend on half of them or fewer, has its formulas moved
onto the scope of their support (`Compilation::narrow`,
`TddBackend::restrict`, the inverse of the embed), and its cone becomes
that support (`Cones::narrow`). Narrowing every scope that could be made
smaller lost: the support walk and the restrict copy each formula, and a
narrowed component's vtree is one its readers no longer share.

**Minimization** (`backend.rs`, `TddBackend::minimize`). tididi's full
reduction prunes unreachable pairs before contracting twins, and the prune
walks the whole diagram whether or not it frees anything. So between
operations (`Reduce::Between`) the full plan runs on a schedule: after a
full reduction that freed at most 1/2^`PRUNE_KEPT` of its pairs the next
1, 3, 7, 15, then `STRIDE_CAP` calls only contract, and one that frees
more resets the stride. The evidence walk's schedule is the other way
round: its joins are pruned but not contracted between full plans
(`Reduce::Prune`). A result that meets a target (Batches, below) is
fully reduced off the schedule where that costs less than the join it
precedes: the last result of a given-up batch's chain, and a batch's
product that holds fewer pairs than its target. The twins a prune leaves
multiply in the join with the target and in every join after it; a full
reduction costs as much as its result is large, and a product larger than
its target is reduced on the schedule. A formula an atom settles on
(`Reduce::Settled`) is always fully reduced, since it is measured, stored
and bounded. Between the clauses of one conjunction only inner twins are
contracted (`Reduce::Contract`). Answers do not depend on it: a
contracted diagram is count-correct, and `equivalent` minimizes its
operands itself.

**Negations by source** (`NegationSource`). Every negation is counted by
source: `Lambda`, `Term` and `Disjunction` (the two of a factored term),
`LinkTerm`, `Primal` and `WellFounded` in the compile, `Evidence` and
`Query` in the query phase. `--stats` prints calls, operand pairs and work
units per source; `tests/it/negations.rs` checks the counts in both
formulations.

**Negation through recursion** (`aft.rs`). A component whose negative
literals stay inside it is compiled with the approximation fixpoint of
Bogaerts and Van den Broeck (ICLP 2015): each atom carries a lower and an
upper formula, a negative literal swaps them, application steps run to
stability, then an unfoundedness step recomputes the upper bounds with the
lower frozen, until nothing changes. If lower and upper agree, the program
is two-valued in every world; otherwise compilation stops naming an atom
undefined in some world. ProbLog rejects every such program syntactically;
tddlog accepts the sound ones, e.g. `a :- \+b. b :- \+a. a :- f. b :- \+f.`
(`examples/negcycle_sound.pl`). `examples/gearwheels.pl` needs the
unfoundedness step, and `examples/unsound.pl` is refused.

**Settling the names** (`settle.rs`). Between the compile and the
queries, `program`'s cuts are settled one by one in the order's
direction, from the diagrams, and so are `decision`'s, the names above
each node of its tree, up the tree. Named, the evidence holds the joint
`J` of the cut's links, built on the scope over them. Expanded, each
formula whose diagram depends on the cut's copies is conjoined with the
links it reads and the copies are summed out
(`∃x (G ∧ (x ⇔ F)) = G[x := F]`, one `and_exists` per group of links, the
groups built on the links' scope and shared by the readers that read the
same links), and the links leave the evidence. The readers that are not
queries grow from `R` pairs to `K`, and the cut keeps its names where
`J < K − R`: the joint is weighed first, up to `4·L` for links of `L`
pairs, and where it is that small the readers are composed only until
they gain more than it; otherwise they are composed up to a gain of
`4·(L + R)`, and the joint then up to their gain. The queries are settled
after them, against the evidence `E` the other readers leave: a query's
count walks every pair of its nodes against the evidence's at each level,
so its names pay where `R·(E + J) < (R + L)·E`, that is where
`J < L·E/R`.

On a Markov chain each day's readers are the next day's links and would
carry two days' joint composed, so the weather and the link grids keep
their cuts. Where the readers compress what they read, as the bits of a
sum or a coupon collector's counts do, the cut is expanded. A cut's links
may read its own copies (a split table's tie to the part before, an atom
that reads a clause table's value through the table's first reader's
copy): a reader is composed with the links it reads and those they read
in turn, each link before the links of the copies it reads. A reader
composed with links that read a later cut's copies joins that cut's
readers. A cut whose link a tie stands on (`tp/tie.rs`) keeps its names.

Settling spends at most the compile's work units, and the cuts it does
not reach keep their names. Expanding a cut whose joint holds more than
`4·L` pairs is not charged to that budget: kept, the cut's names would
hold the joint in the evidence, and the work composes formulas the
compile would have built without them. Charged, it spent the budget
before settling reached the cuts at the root of `decision`'s tree. Under
`auto` that work is not charged to the trial's turn: the run's stop moves
later by what settling spends (`TddBackend::stop_later`), so that it
never takes the trial into another round, where every other trial spends
a turn too.

## Evidence and queries (`src/query/`, `src/eval/`)

`P(q | e) = WMC(q ∧ e ∧ L) / WMC(e ∧ L)` with `L` the indicator links.

**The walk** (`walk.rs`). Each evidence factor (an observed atom's formula
or its negation, a link) goes to the lowest vtree node over its scope
(`TddBackend::lowest_over`), and the vtree is walked bottom up: a node
conjoins what its children built with the factors put there. Conjoined in
any other order, factors at a node read variables decided outside it
without the factors deciding them, and their product tells apart
assignments the whole conjunction rules out. At a node the factors are
conjoined pairwise in balanced rounds, so that thousands of observations
do not cost the square of the evidence's size, and every result is
reduced on the walk's schedule (Minimization, above). Each pair is
brought together on the union of the two scopes, or on the scope over
all of a node's factors where those hold at most `SHARED_UP_TO`
variables. The walk's conjunctions take tididi's sparse route at a lower
density than the compile's (`QUERY_ROUTE`).

- **Put-offs.** Where a node's items other than the largest total under
  1/`PUT_OFF_BELOW` of the largest, charged `LEVEL_PAIRS` pairs per node of
  its scope's vtree since a join walks every level of its scope, the
  largest passes up and the rest are put off; the first node where that
  fails conjoins them. On a right-linear vtree the observations otherwise
  each join a growing result.
- **Batches.** The factors a child put off are conjoined among themselves
  and their product meets the child's result. They go in groups that merge
  as a binary counter carries, each a balanced tree, the groups' products
  meeting two smallest first. Factors without the formula that ties them
  can grow with each other, so each join of a batch is capped at the
  larger of `DEFER_GROWTH` times its operands and the target's charge,
  what putting its operands into the target instead would walk; past the
  cap what is left goes into the target one at a time, in the order the
  factors were put off.
- **The largest child aside.** Below the root, a node puts its own factors
  off into the batch of its largest child, as one more group, conjoins
  every other child's result, and joins the largest once, two smallest
  first (`Join::Smallest`). The own factors read variables the child's
  result decides: data points of a bit-blasted circuit, observed bit by
  bit, read a shared number's lower bits, and conjoined among themselves
  their product holds every combination of the data points' carries. At
  the root the own factors may span the whole vtree, so the root sets its
  largest child aside only where the child's put-offs weigh at least the
  own factors, or where the child has none.
- **Waiting joins.** A node's last join below the root, of two operands of
  `DEFER_FROM` pairs or more, is tried capped; past the cap it waits, and
  where the walk's last join reads it with a smaller operand, that operand
  goes into the larger of the two first.
- **Parts under a chain.** Where the vtree's top is a chain (a node whose
  one child is a leaf, over another such node; `decision` puts the
  decisions every part reads there), a batch whose factors read the
  chain's leaves and two parts or more below it is grouped by part, so
  that the factors of one part meet first (`EvidenceWalk::by_part`). The
  factors of two parts agree only through the chain's leaves, and their
  product holds every pair of what each keeps apart.

**Summing out** (`sum_out.rs`). Where no query reads the evidence formula
(`--no-queries`), the walk's result is only counted, and a variable can
be summed out of a join's result once no later join can read it. A join
sums out every finished subtree of its scope as it builds it
(`Engine::and_marginalizing`): a subtree whose variables no other slot of
the walk holds, found with a Fenwick tree over the vtree's leaves. The
weights are tididi's scaled floats (`Arithmetic::ScaledFloat`), and the
walk reads P(e) from its last result's weighted value. Before, every join
kept its full structure, and on Markov networks of high width the last
joins ran out of memory: nine partition functions of the UAI 2022 PR set
answer with it, 1bxv and 1jer at a peak of 2.1 and 4.3 GB. Programs with
queries keep the walk above.

**Queries.** A query on an atom with an indicator in the evidence's scope
is a literal of the evidence formula, and one backward pass
(`Counted::down`, `Counted::with_literal`) gives all of them. Any other
query is counted against the evidence without a product
(`eval::conjunction_count`): the evidence formula is counted once, up and
down, and each query's formula, on its own cone vtree, against it in one
dense table per level, a row per evidence node and a column per query
node; a memoized walk would reach nearly every pair at a hash entry each.
The dense count is charged to the work clock at `MULTIPLY_ADDS_PER_UNIT`
multiply-adds per unit. A query is counted in the polarity asked where
that costs nothing: a literal of the evidence reads its own leaves, and a
fact's literal is built as asked. A query of the polarity the formulation
does not store is `1 - m` from the stored formula's count `m`, except
where `1 - m` is under `eval::COMPLEMENT_FLOOR` (1e-4): the subtraction
keeps the count's absolute error, some 1e-14, and a posterior of 2.8e-121
came out as 2.6e-14. There the stored formula is negated and counted
(`eval::complement`), in the query phase and where the compile counts a
query's formula it drops.

**Templates** (`src/tp/template.rs`). A *leaf* is a derived atom that no
rule reads and nothing observes, and a fact is *private* where only
leaves' rules read it. Where every rule of a query leaf `y` has one
private literal, a positive `t`, and the rules of another query leaf `x`
are those of `y` with each `t` taken by a cube `C_t` of private literals,
the cubes of distinct `t` over distinct facts, `x` is answered by `y`'s
formula counted with each `t` at `P(C_t)`, and its own formula is not
built (`ForQueries::templates`). The values of a table's leaf variable
have this shape: value `i` of a row is `¬s_0 ∧ … ∧ ¬s_i-1 ∧ s_i` over the
row's own switches.

**Counts** (`src/eval/`). A count is taken in `f64` and, where that is
not exact, in `eval::Count`s, a float with an exponent of its own in
steps of 2^256; an answer is the ratio of two counts. A thousand
observations have a probability below the least positive `f64`. One
generic implementation over `eval::Value` serves both types: the passes
up and down of `eval::count`, `conjunction_count`'s tables and the
engine's evaluation (`eval::Weighing`).

Each level, column and table notes the least nonzero and the largest of
its values (`eval::Range`; a grid's plan notes them where it stores each
value for the last time), and a product of two such sets is exact when
the product of their least values is a normal float and neither largest
is infinite. Where it would not be, the passes keep the operands under a
power of two per level and per column (`Scaled`), exact on normal floats:
each operand centred on one, and both moved down where their products
could overflow (`eval::capping`). The test takes the products of the
operands' extremes, which no pair need take: a level whose own values
span a few hundred powers of two can read children that span a thousand
each. A level or column that fails it is exact as computed where its
values are all `2^-960` or more (`eval::exact_as_computed`): a product
that leaves the normal floats is off by under `2^-1075`, and a sum of
fewer than `2^63` of them by under one rounding of such a value; a zero
is exact only where no product could round to zero. A level whose values
come out smaller is moved up and computed again (`eval::raising`).

A level or column still not exact is computed in `Count`s from its
operands and kept in floats under a power of its own where its values fit
(`Counted::recount_level`, `eval::fitted`); a column of the pass down that
does not fit is kept in `Count`s (`Down::wide`), each value normalized, and
read so by the columns under it. A count up still not exact goes on in
`Count`s from the level that failed, and a query that reads a column it
cannot fit takes the count over in `Count`s from the float counts up
(`Counts::widen`); every later query reads that count
(`QuerySizes::recounts`). With queries, the evidence's mass is the pass
up; without them (`--no-queries`) it is the walk's last result's
weighted value (Summing out, below). Neither is charged to the engine's
work clock,
so a trial that has built its evidence formula within its bounds counts it;
a formula counted without evidence is the engine's evaluation
(`eval::mass`), one range for the whole diagram, which the work clock is
charged for. Only what a count reads is checked: a query formula's other
levels overflow with many indicators and are not read. The exponent is per
value, not per level: a cause observed 2 200 times for and 2 199 against
has two nodes at one level 2^2200 apart.

**Grids** (`eval/grid.rs`, `eval/kernel.rs`). A level of `GRID_MIN_PAIRS`
pairs or more whose nodes each hold their first pair shifted by one list of
offsets, the first pairs affine in the digits of the node index in a mixed
radix, is read as a grid: the large levels of a dense network's evidence
are such tables. An implicit level (tididi's description of one,
`Grid::implicit`) is read without enumerating its pairs; any other is
fitted by one pass (`Grid::fit`). The passes are then contractions
`out[o] += x[a] · y[b]` with `o`, `a`, `b` affine in the digits, each run
in an order chosen against a model of cache misses (`Plan`), through AVX2
kernels where the processor has them and scalar loops otherwise. Each
output takes its terms in the order the passes over pairs add them, so
the answers are theirs to the bit.

**The probability of the evidence.** `--evidence-probability` prints
`P(e)` after the answers (`Run::evidence_probability`): the count of the
evidence formula, `WMC(e ∧ L)`, which the query phase takes anyway, times
what conditioning took out of the program (`GroundProgram::weight`).
Dropping detached evidence would leave out factors of `P(e)`, so the
binary grounds with it off under the option. The line is
`P(e):<tab>p<tab>log10 l`; below the least normal float the mantissa and
exponent of `p` are read off the logarithm. Inconsistent evidence prints
`0` and `-inf`, none `1` and `0`. For a UAI `MARKOV` file the binary
prints the partition function given the evidence instead,
`Z:<tab>z<tab>log10 l`: `P(e)` times ten to the offset the reader computes
([Network files](#network-files-srcnetwork)), the UAI PR task. The
program the network is read as states the offset, `log10 Z - log10 P(e)`,
on its first line. `--no-queries` drops the queries before grounding
(`syntax::Program::without_queries`), so the program is ground for its
evidence alone and the `P(e)` line is all the run prints.
tests/it/evidence_probability.rs checks `P(e)` against ProbLog's under every
strategy.

## Allocator (`src/bin/tddlog/alloc.rs`)

jemalloc instead of glibc malloc, with transparent huge pages for its
extents, one arena for blocks of every size, no purging while requests
succeed, and every kept free extent open to every request that fits in
it (`_rjem_malloc_conf =
"thp:always,oversize_threshold:0,dirty_decay_ms:-1,muzzy_decay_ms:-1,lg_extent_max_active_fit:64"`;
`_RJEM_MALLOC_CONF` overrides each option): the query phase of the large
networks faults in gigabytes of fresh levels, and memory the program frees
is reused rather than given back to the kernel and faulted in again. By
default jemalloc splits a kept extent only for a request of at least a
64th of its size; freed blocks join into extents of gigabytes, the
smaller requests after them took fresh pages, and with no limit on the
address space the kept pages grew without bound. Kept free pages count
against a limit on the address space as live ones do, and jemalloc joins
the small pieces among them only when it gives them back, so a request
jemalloc refuses is made again once after a purge of every arena
(`purge`): a trial `auto` retired leaves its memory to the trials that
follow. With jemalloc's default, blocks of 8 MiB and more come from an
arena of their own, whose free address space no other request can take.

tddlog is refused memory only where the allocator refuses it, which under
a limit on the address space (`ulimit -v`) is at the limit. A cgroup's
`memory.max` bounds resident memory instead, and the kernel kills a
process that reaches it without refusing anything first, so `auto` could
not give up one trial and keep another. The binary therefore lowers its
address-space limit to the room its cgroups leave, `memory.max` less
`memory.current`, the least over its cgroup and those above it
(`src/bin/tddlog/budget.rs`), and is refused there as under `ulimit -v`:
resident memory never exceeds the address space that holds it.

Huge pages go on once the process has asked for `HUGE_FROM` (16 MB), and
off for good where the kernel stalls (`kernel_stalls`, looked at every
`HUGE_FROM` bytes asked for): where it has taken more than a tenth of a
second of system time and more than a nanosecond for each byte of the
process's peak resident set, several times what supplying memory costs it
in either page size. Where free memory is fragmented, as with several
large runs side by side, a fault in an advised mapping compacts memory at
once (the `madvise` defrag setting), the compaction fails, and the fault
takes a small page after all. The binary leaves the `Solution`, which
holds the parsed and normalized programs and the last run, to the
process's exit rather than free it (`std::mem::forget`).

## The browser build (`bindings/wasm`)

The playground at tractables.github.io/tddlog runs the library in the
browser. `bindings/wasm` is a crate of its own that calls `tddlog::solve`
and writes the answers, `P(e)` and what each part's run did as JSON;
nothing else of the library is wrapped, and the page's examples are
programs the repository tests against ProbLog. It is built for
`wasm32-unknown-emscripten`, as goatd's page is, because goatd's vendored
FlowCutter is C++ that throws and catches: Emscripten compiles it with its
libc++ and links it with the Rust code, whose standard library has a clock
on that target. The binary's jemalloc is left out (its dependencies are
gated to targets other than WebAssembly), and Emscripten's malloc returns
null where the heap cannot grow past what the browser grants, at most
4 GB: an operation tididi can refuse is refused as on the command line,
and `auto` retires its trial; any other failed allocation aborts the
module, which the page reports before loading it again. The run goes in a
worker, which Cancel ends.

## The Python and C bindings (`bindings/python`, `bindings/c`)

`bindings/python` (PyO3, built with maturin) and `bindings/c` (a C ABI,
its header generated by cbindgen) are crates of their own, each over
`solve` and nothing else: they take the program's text and the command
line's options (`--vtree`, `--evidence-probability`, `--no-queries`),
copy the answers and `P(e)` out of the `Solution`, and free it at once.
The query's text is `QueryResult`'s `Display`, as the binary prints it.
`Error`'s variants become exceptions or status codes: `Options` the
caller's mistake (`ValueError`, `TDDLOG_ERROR_INVALID_INPUT`), `Parse`,
`Ground`, and `Vtree`, `Compile` and `Trials` together as a compile
error. They run under the host's allocator: the binary's jemalloc set-up
(above) is not theirs.

## Testing

- `cargo test`: unit tests in the modules; `tests/it/examples.rs` checks
  `examples/*.pl` against ProbLog's answers under `auto`, `closed`,
  `dynamic` and `families`, and the literature families in `tests/lit/`;
  `tests/it/problog_suite.rs` runs the ProbLog system tests copied into
  `tests/problog/` under `auto`, `closed`, `dynamic`, `program` and
  `families`. The other modules of `tests/it/` check a mechanism each
  against an exact sum, enumeration or ProbLog: indicators, binary
  reads, the clause memo, table models, ties, templates, switch sharing,
  projection, the layouts (`layouts/`), the or-route, negation counts,
  the evidence walk, the query and dense counts, implicit levels, counts
  beyond a float's range (`underflow`), `P(e)` (`evidence_probability`),
  the command line (`cli`, which also checks this page's walk through
  `examples/alarm.pl` against the binary), network files (`networks`:
  the programs of the files in `tests/networks/`, `tests/regress/` and
  `examples/` against the programs written for them, and the binary's
  answers and partition functions on them against enumeration and
  ProbLog), and regressions (`tests/regress/`). Queries in groups are
  tested in `src/solve/tests.rs`; `src/tp/tests/` compiles every example
  with a dynamic search after every step, checks formulas dropped as
  they die, resumes stopped cyclic components, and checks the primal's
  update in a cyclic component against enumeration.
- The Python tests (`tests/test_*.py`) check `scripts/alea2problog.jl`
  (where Julia is installed) and the release metadata check.
- `bindings/wasm/tests/examples.cjs` runs the playground's examples
  through the WebAssembly build against ProbLog's answers and `P(e)`, and
  `bindings/wasm/tests/browser.cjs` the page in a headless browser.
- `bindings/python/tests` (pytest, against the installed package) and
  `bindings/c/tests/answers.c` (CMake's test, run by
  `bindings/c/check.py`) check 14 of the ProbLog system tests under five
  strategies, the errors, and `P(e)` by hand.
- `tests/problog_suite.py` runs all of ProbLog's system tests through
  the binary and classifies pass, fail, unsupported and expected error.
- `tests/compare.py` diffs one program's probabilities against
  ProbLog's; `tests/fuzz_problog.py` diffs random programs in the
  supported fragment against ProbLog.
- `scripts/bn_exact.py` (on the baselines branch) answers a generated
  network by clique-tree calibration with ProbLog's semantics, the
  reference for the networks ProbLog does not finish.
- The benchmark driver, on the repository's `baselines` branch, runs the
  benchmark families against the baselines and checks every answer
  against ProbLog or the exact references
  ([benchmarks.md](https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md)).

## Measured cases

The cases behind the rules above, by section, for weighing a change to a
rule. The names are benchmark instances of the results page. Sizes in
pairs and work units are machine independent; seconds and gigabytes
compare only within the run they come from.

### Grounding

- **Step 4, rising levels.** A loop written as recursion over its steps
  grounds no dead branch: dice_pmc1 is 1 506 atoms once simplified,
  against 8 350 unrolled, and 3.2 G instructions against 3.8 G; the 27
  loop programs 166 G against 219 G unrolled.
- **Step 6, exclusive sets.** Without the facts no other rule reads left
  out of a body, caesar-iter's 3000 letters were compared pairwise.
  Without a key's sets listed by the literals that clash with its first
  rule, the bits of caesar-iter in binary, two sets a step under one key,
  were each compared with every set before them.
- **Views.** bn_munin's copies of a variable cost `auto` twice the work
  inlined. The Dice paper's diamond, unrolled, keeps one atom a step of
  four, its work at 1 600 steps 3.98 M units to 1.23 M.
- **Equal gates.** Of pi's 6 635 gates at 12 bits, 4 941 merge by
  structural hashing, which held three quarters of the pairs the gates'
  formulas hold, and the compile's work falls to a fourth. Before the
  sweep stopped at 64 models with no gate proved, bb_weekend_w30 made
  7 762 calls and proved nothing, a second of its ground time. The sweep
  takes 306 of conjugate_gaussians_w19's 1 334 gates and half its compile
  work, and 1 417 of pi_w16's 3 093. Merged into another variable's
  value, bn_hailfinder's `scnrelplfcst(b)` split its variable and grew the
  families layout's largest formula tenfold. Merged into a switch fact or
  another value, uwcse4's values and dice_digitRecognition's took the
  tables' structure apart (twice the work and more). Merged in a program
  conditioned on its guards, bb_altermu2_w11 takes 181 M work units with
  the later gate kept, and runs out of 8 GB with the earlier one, 27 000
  atoms before it. Built without `may_merge`, the graph took a fourth of
  bn_diabetes's ground time to find nothing.
- **Choices under exclusive contexts.** simpleCaesar at 1 000 steps, 32
  letters, goes from 3.87 M pairs to 161 k.
- **Renormalization.** bn_mildew_ev's `dm_4`, 210 rows over the 100 values
  of `dm_3`, does not compile in memory with its masses read through an
  atom of their own, once a stronger propagation rules out most of
  `dm_4`'s values; with `dm_3` read through a copy per value it compiles.
  On Ace's networks munin1's work is 0.45 of what it was without the
  renormalization.
- **Step 11, defined facts.** Three implied clauses of 36 literals held
  `td`'s width on one instance at 41; without them it is 23. On another
  the search finds what propagation does not, and takes it from 41 to 16.
  Dropping implied clauses whatever they join lost an instance. Bounded by
  the count of clauses instead, as SatELite does, the eliminations took
  `td`'s width on one CNF from 19 to 37.
- **Step 12.** The compile's Tp iteration runs out of memory on g30_100.

### Vtrees

- **Direction.** On the sachs `akt` marginal the reverse dependency order
  gives 2.1 k pairs; the forward order does not finish. On bn_child one
  formula reaches 31 M pairs under a tree decomposition's vtree, 6.9 k
  under a dependency order.
- **TDDs are not SDDs.** ProbLog's ground formula for uwcse4 under PySDD's
  own vtree is a TDD with about as many nodes as the SDD has elements and
  56 times the pairs; a right-linear vtree over the reverse compile order
  gives 13 k pairs.
- **`td`.** The bushy fold does not compile uwcse4, uwcse6 or bn_child in
  minutes, where the right-linear order takes about a second. Below its
  hard cutoff the portfolio's eliminations left 1.3 M atoms in one bag on
  Roulette's caesar at 20 000 steps, where min-degree finished at width
  14.
- **`families`.** The layout cost estimates the evidence's pairs within
  0.85 to 1.8 times on the grids, uwcse and bn_link, and over it on the
  other networks.
- **`program`.** On the bit-blasting benchmarks it is the one layout that
  compiles trueskill (largest formula 63.6 M pairs, against 3.4 M nodes
  in the package's own diagram for the evidence and query); forward
  program order and every other `auto` trial run out of memory there. On
  weekend scaled down to 256 pieces it is the one layout that finishes
  (131 k pairs at most; `td`, `dynamic` and `families:0` run out of
  memory), though the largest formula grows with the square of the
  pieces, and the full program, at 4096, runs out of memory under it.
  With names at its narrow cuts, nine days of GeNI's weather take 174 M
  work units against 24 204 M without, and the 12 by 12 link grid 254 M
  against 2 984 M.
- **`program:fanin`.** On conjugate_gaussians_w21 the compile takes 78 M
  units of work under `program:fanin` against 7.2 G under `program`; on
  addFun_sum at 19 bits the largest formula is 58 k pairs and the compile
  43 M units, against 50 M pairs and 4.0 G units. Where the file states
  the facts in the order of a model's time, pluck_hmm compiles in 1 286
  pairs under `program` and runs out of memory (25 GB) under the walk,
  and so do both trueskills (Alea's at 8 bits and hybit's); pluck_pcfg
  compiles in 60 s under `program` and not in 600 s under the walk. On
  GeNI's 12 by 12 grid of link failures the walk takes 0.66 G units of
  work, `program` 0.11 G.
- **`decision`.** With the roots deepest first, altermu2_w11's 40 data
  points' noises lay beside the mean at every bit, and its query phase
  took 1.52 G units of work against 247 M. On caesar_huge, where nothing
  outside the tree reads it, the contexts' names alone took the compile
  from 0.47 G to 5.36 G units.

### `auto`

- **The `program` trials.** On geni_link_16 the inlined views take the
  widest cut from 17 atoms to 30 and the cuts the naming finds from 67 to
  21, and the compile reaches a formula of 113 M pairs in place of 3 022.
  The circuits `program:fanin` wins took a third longer with the views in
  place. First on every program, `program` costs the networks and grids 3
  to 60 times. On circuits whose every other layout runs out of memory
  `program` may be small: pluck_hmm (199 flips, 346 gates) compiles in
  1 286 pairs, pluck_pcfg in 1.1 M, Dice's triangle_large in 29 M and
  trueskill in 64 M; of these `program:fanin` compiles triangle_large
  alone (9.2 M pairs). Over the 157 bit-blasted circuits that both
  finished, `auto` with the walk in place of `program`'s order took 0.895
  of `auto`'s time in geometric mean and answered 15 programs more and
  one fewer. In the timed runs of the circuits, GeNI's networks and
  Roulette's programs, all built with views, `td` won none, and its turns
  took a fifth of what the losing trials spent on the circuits of a
  second or more.
- **The `decision` trial.** The full weekend circuit of the bit-blasting
  paper runs out of memory under every other trial and compiles under
  `decision`; its conditioning makes 578 k atoms for 160 k derived.
  Without the test, Roulette's ladder of 3 000 rungs was ground again for
  no trial at 48 s and 13 GB, its pass copying cubes that held the other
  column's facts.
- **The conditioned program.** On bb_altermu2_w11 `closed` takes 2.29 G
  units on it against 14.4 G on the program as ground. Under `auto` it
  takes bb_normal_mixture_theta_w9, which `closed` wins, to 0.66 of the
  time it takes on the program as ground and mu2_w7, which `program`
  wins, to 0.83. On addFun_max_w17 the order `program:fanin` reads off the
  conditioned program takes 29 times the recursive calls, in CUDD's
  model, of the order it reads off the program as ground, and under
  `auto` every order trial on the conditioned program ran out of memory.
  `families:0` takes 1.07 G units over its turns on bb_normal_mixture_mu2_w7
  conditioned against 133 M. With the order layouts on the conditioned
  program beside `decision`, at 25 GB a process they starve
  bb_coinBias_w16's `decision` of memory in its query phase, and the run
  runs out of time at 600 s, where beside the order layouts on the
  program as ground `decision` answers in 286 s. That gives up
  bb_altermu2_w11, which `program` wins on the conditioned program in
  0.32 to 0.38 of the time.
- **Turns.** dice_pmc1, which `program` answers in 0.3 s, has a
  `families:0` layout of width 345 that takes 3.8 s to build. On the
  circuits that `program` leads, layouts over the families estimated at
  up to 1e14 pairs took the weight's share for nothing (pi_w12 8 s to
  57). objectdetection_13's `families:0` completes 2^30 pairs on 1.15 G
  units, and a watch tied to the target gave the other trials three turns
  there. Against the rounds the doubling targets replaced, whose size
  stops ended a trial's round, Alea's arith_book (won by `closed`, third
  in order, at 0.6 G units of 4.4 G spent) went from 16 s to 47 s, GeNI's
  router_18 from 3.7 s to 7.4 (`program` and `closed` run on past size
  stops before the searched layout's turn), and Pluck's pcfg, which
  `program` wins after 9 G units, past 600 s from 479 (in 56 GB, against
  the 1.7 GB of `program` alone). Over the three benchmark sets the two
  take about the same time (0.84, 1.03 and 1.00 of the rounds' time by
  suite).
- **Turns by projected finish.** With every compile record dropped as the
  query phase starts, trueskill_w10's `program` stopped with its compile
  done and two of its four query steps, then the four other trials took
  a turn each, 90 s, before the second it needed. Over the circuits, in
  two timed runs of the finishing turns this rule replaces, a trial in
  its compile projected to finish within one doubling did so 70% of the
  time; one projected one to two doublings out finished within two 71 to
  80%, three to four out within three 62 to 70%, four to six out within
  four 49 to 55%; one projected past six doublings, or making no
  projection, finished within four 16 to 43% of the time. The chance of
  finishing in the next doubling per unit of work spent on it, by which a
  turn is worth giving, so falls about as 2^−e with the doublings e a
  trial is projected out, down to that of a trial with no projection at
  about three to five doublings, and is flat past it. The finishing turns
  (twice the weight for a turn where the trial was projected to finish
  within it) are the rule with one doubling, a projection within it
  counted as none. On the logged curves of the circuits of 0.5 s and more
  in the two runs, replayed under each rule, the three doublings took
  0.79 to 0.87 of the finishing turns' time by set of circuits; four and
  five doublings gained little more, and lost more where a projection was
  wrong (trueskill_w8, whose `program` stalls at a few components a
  doubling and then finishes). Projected to finish within one doubling, a
  query phase did so 22% of the time. GeNI's weather_h400, whose
  `program` stops 0.2 G units short of its walk's end at its target of
  2^31, is projected at 2^31.0 and takes its next turn ahead of the
  others', where at equal shares every other trial took a turn first, 9 G
  units in all. addFun_max_w17's `program` runs its seventeen queries, of
  0.14 G units each, within the turn its compile was projected to finish
  in. On GeNI's weather and normal_mixture_theta_w9, a trial in its
  compile at a few components per doubling would, given its weight
  however far out, go ahead of a winner that has reached its query
  phase.
- **Turns that end on their projection.** By doublings, the circuits
  `decision` wins were projected dozens of doublings out at 2^24 units
  (GPA_w31 at 6.8% of its components, 2.8% over the last doubling). The
  linear projection held them within 2^31.4 at their first turn's target
  of 2^31, and SPPL's markov chains, whose `decision` finishes two
  components per doubling of work (the separated flag decides between
  every step's emission trees), at 2^34.3 to 2^38.7 at 2^24.9 units:
  their first turn of 2^31 units, 15 to 17 s, ends after 0.2 s.
- **The searched layout.** On GeNI's weather_n7 and n8, which `program`
  wins, a searched layout estimated about as large that bet at weight 8
  spent 2.1 G units in its first attempt. GeNI's link_12, at a gap of
  4.15, is won by the searched layout at its weight, and by `program` in
  27 s otherwise. On geni_router_24 the searched layout is estimated 88
  times smaller, and `families:0` at its weight otherwise spent 4.3 G
  units beside it before it was refused memory. On bn_munin1 the searched
  layout wins before `closed` and `td` spend their turns. mastermind's one
  query operation takes 6.9e7 units, against a charge of 2e8 met halfway
  through a turn: stopped at the charge, the rest of the turn would not
  hold it again.
- **Outgrown estimates.** Alone, the families trials that win have spent
  at most 7 units a pair on the networks and up to 150 on small programs
  they finish in their first turn (mnist_sum96); those that lose have
  spent 300 and more (GeNI's links and weather networks) or never finish
  (Dice's simpleCaesar).
- **Refusals.** Freed for whichever trial was refused, the others paid
  for an early refusal of a cheap trial: on bb_spacex_w17 `program`,
  freed for `families:0`, did its 2 G units again (195 s against 124),
  and on geni_link_18 `families` took the others' formulas twice and still
  did not fit. On geni_router_24 in 30 GB of address space `families` was
  refused at the last step of its query phase while `program`,
  `program:fanin`, `closed` and `families:0` held their stopped compiles,
  and the run failed after 472 s; alone it peaks at 22.6 GB. On
  spacex_w19 in 33 GB `closed` and then `program` were refused beside
  each other's formulas, where `program` alone answers in 113 s.

### Compilation

- **Formulation.** On the HyBit weekend, storing gates of one rule as
  their conjunction and reading the other sign as stored cut the
  negations from 724 k to 265 k, and the compile work from 836 M units to
  533 M. On Geni's weather factory, whose formulas have many readers, the
  last reads take a third of the pairs copied.
- **Projection.** weekend_w30's scopes held 11.1 M variables in all, the
  facts and supports its components read 7.2 M. Narrowing every scope
  that could be made smaller took addFun_max_w17 from 36 s to 114 s.
- **Minimization.** normal_mixture_theta under `closed`: a batch's product
  pruned to 475 745 pairs where the same factors' product minimized held
  37 188, and the target's join 17.8 M pairs against 5.5 M. GeNI's
  weather_h400 under `program`: a product of 202 M pairs, its target 23 M,
  minimized in full to 200 pairs fewer for 143 M units of work.
- **Settling the names.** The weather keeps all its cuts, and so do the
  link grids (31 of 31 at side 12). 20 of addFun_max_w17's 26 cuts are
  expanded, which takes it from 12 210 M work units with every name kept
  to 7 502 M (11 597 M without names). With the expansion of a large
  joint charged to the budget, the genes programs' cuts spent it before
  settling reached the cuts at the root of `decision`'s tree, and
  genes5_w21's evidence held 94 M pairs of their joints (93 k with them
  expanded). Weather by factories at 13 takes 3.9 G work units against
  7.1 G with settling charged to the turn, every cut keeping its names.
  Over 178 programs of the timed suites (GeNI, the standard and large
  sets, lit, indicators, noDice, TPC-H), names and settling take `auto` to
  0.591 of the time without them, and answer 17 programs that `auto`
  without them does not answer in 600 s.

### Allocator

- With jemalloc's default extent fit and no limit on the address space,
  coinBias_w16's kept pages grew past 200 GB. With one arena,
  triangle_large answers at 16 GB, where it was refused with blocks of
  8 MiB and more in an arena of their own.
- Supplying memory costs the kernel about 0.1 ns a byte in huge pages and
  0.5 ns in small ones. Where a fault in an advised mapping compacts
  memory and the compaction fails, the run pays 1.4 to 10 ns a byte.
