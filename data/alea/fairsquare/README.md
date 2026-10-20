# FairSquare's decision trees

`fairness.jl` is the fairness verification problems of FairSquare
(Aws Albarghouthi, Loris D'Antoni, Samuel Drews, Aditya V. Nori,
"FairSquare: Probabilistic Verification of Program Fairness", OOPSLA 2017,
PACMPL 1, article 80, DOI 10.1145/3133904) with decision-tree classifiers,
which SPPL (Feras A. Saad, Martin C. Rinard, Vikash K. Mansinghka, "SPPL:
Probabilistic Programming with Fast Exact Symbolic Inference", PLDI 2021,
DOI 10.1145/3453483.3454078) solves exactly in its Table 3: DT4, DT14,
DT16 and DT44 (`dt = 4, 14, 16, 44`) under the population model of
FairSquare's second Bayes net, from `oopsla/noqual/M_BNc_F_DT_*.fr` of
github.com/sedrews/fairsquare at bd27437 (MIT License). The file's header
says how FairSquare's language is translated. The program returns the
probabilities that the tree's decision is the fairness target and the
person is female, and that it is the target and the person is male, from
which the fairness ratio follows (P(female) = 0.3307). `bench.py --suite
bitblast` runs each tree as a family `bb_fair_dtT`, laddered by the bits
after the binary point.

Reference: Alea.jl's answer on the discretized program; tddlog's answers
on the circuit are checked against it in every run.
