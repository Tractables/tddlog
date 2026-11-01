# Programs from ProbLog

Copies of example programs from ProbLog's repository
(<https://github.com/ML-KULeuven/problog>, `test/`, commit
a294fa24c3cfb4bae3a2acea3a128386113889f4), under the Apache License 2.0
(`LICENSE`, ProbLog's). They are the programs of the `problogex` benchmark
family (`scripts/gen_problogex.py`), each as it is: the system tests 3 to 12
(coin tosses, Bayesian networks, an HMM, a probabilistic graph, smokers,
packing, cards, ads, holidays) and the examples `coin`, `smokers_or`,
`advars_smokers`, `advars_smokers_alt`, `tc_1` to `tc_3`, `negation`, `swap`,
`add` and `generated`. Each states its answers in a comment,
`%Expected outcome:`, which is the family's reference (`scripts/lit_exact.py
expected`). ProbLog's tests of single features (`00_*`, `findall*`, `cut*`
and the like) and those whose expected outcome is an error are not copied.
