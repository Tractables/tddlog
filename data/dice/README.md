# Dice programs

The baseline programs of the Dice paper (Holtzen, Van den Broeck, Millstein,
"Scaling exact inference for discrete probabilistic programs", OOPSLA 2020),
copied from `benchmarks/baselines/` of <https://github.com/SHoltzen/dice>
(Apache License 2.0). They are the source of the `dice` benchmark family:
`scripts/dice2problog.py` translates each into a propositional ProbLog
program.
`digitRecognition.dice` is a naive Bayes digit classifier: a class prior
over ten digits and 7830 observed pixel flips per class. Its evidence
probability is far below double precision, which Dice reports as an
all-zero distribution; ProbLog normalizes to the posterior (class 2).

## Loops, integers and recursion

From the same repository (commit 0ea228e, Apache License 2.0), copied
unchanged; `iterate(f, init, k)` applies f k times:

- `caesar-iter.dice` (`resources/`): the Caesar cipher of Dice's README,
  a 4-letter alphabet (int(2)), 3000 observed characters by `iterate`.
- `simpleCaesar.dice` (`benchmarks/`): the same over 32 letters (int(5)),
  1000 characters. Dice underflows on both (all-zero tables).
- `pmc1.dice`, `pmc2.dice` (`resources/`): a contract signing protocol as a
  guarded-command step over 40 and 24 int(5) components, iterated 75 and
  51 times. Dice's test suite gives 1023/2048 and 31/64 (`test/Test.ml`,
  where both tests are commented out); tddlog gives the same.
- `caesar-recur.dice`, `recursion.dice` (`resources/`): loops written as
  recursion.

Written for tddlog from the generators of the Dice paper's network
verification benchmarks (`bench/dicebench.ml`, `gen_diamond` and
`gen_ladder`), their repeated calls as `iterate`, 100 steps:
`diamond.dice`, `ladder.dice`.

Written for tddlog to exercise each construct: `arith.dice` (+, -, *,
shifts, uniform), `compare.dice` (comparisons, ^, <=>), `walk.dice` (a
random walk by `iterate`), `counter.dice` (a tuple state with an
observation in the step), `geometric.dice` (recursion past Dice's limit
of ten nested calls), `nested.dice` (a network as nested `discrete`s,
rows of thirds that sum to 0.999999).

tests/dice/ holds copies of the programs and their translations, all but
caesar-iter, simpleCaesar and the pmc programs, so that main tests the
translator without this directory; they are checked against Dice's answers, or answers
computed by hand for nested.dice (tests/it/examples.rs), and against an
exact evaluation of the Dice program (tests/test_dice2problog.py).
