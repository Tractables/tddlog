# noDice benchmark programs

The purely probabilistic benchmark programs of noDice (Gürtler and
Kaminski, "noDice: Inference for Discrete Probabilistic Programs with
Nondeterminism and Conditioning", OOPSLA 2026, DOI 10.1145/3798215,
arXiv 2602.20049), in Dice's syntax, from the paper's artifact (Zenodo
10.5281/zenodo.18739789, record licensed CC-BY-4.0; the repository's
LICENSE is the Apache License 2.0), directory `benchmarks/nodice/`.
Copied unchanged, except coupon2-16-obs, coupon2-18-obs and coupon2-20-obs
(below).

| file | artifact path | paper's name |
|---|---|---|
| coupon2-10-obs.dice ... coupon2-14-obs.dice | coupon/ | CouponProb-10 ... CouponProb-14 |
| coupon2-16-obs.dice, coupon2-18-obs.dice, coupon2-20-obs.dice | written here | CouponProb-16, -18, -20 |
| survey.dice, hepar2.dice, insurance.dice, water.dice, alarm.dice | bayesian/ | Survey, Hepar2, Insurance, Water, Alarm |
| 3sat-0.dice | 3sat/ | 3sat-0 |

CouponProb-N is a coupon collector with N coupons: each of N rounds
draws two coupons uniformly, observing that they differ, and the program
returns whether every coupon is held at the end. CouponProb-16, -18 and
-20 are written in the artifact's pattern with more coupons: the pairs of
`fst` and `snd` that hold N flags, coupons drawn from `uniform(B, 0, N)`
with B the bits that hold N - 1 (the artifact's files for 10 to 14 come
out of the same pattern byte for byte), as larger sizes of the series, by
`awk -v N=COUPONS -f scripts/gen_coupon.awk`.
The networks are written as nested `discrete` choices and return one
variable (Alarm returns BP, Hepar2 bleeding, Insurance PropCost, Water
CBODD_12_45, Survey T). 3sat-0 is a 3-SAT formula of 40 variables and 115 clauses
(the paper says 114) from the instance uf50-0999 of Hoos and Stützle's
SATLIB; it returns whether an assignment drawn uniformly satisfies it.

The paper's other classes are left out: Runway, CouponNdet, Network,
Rabin, the `-Ndet` networks and 3sat-25 to 3sat-100 choose
nondeterministically (`nflip`), which Dice, Alea.jl and ProbLog do not
have. The paper's Table 1 times Dice on the programs here (ms, Apple M2
Pro): CouponProb-10 to -14 721, 1 958, 5 798, 20 691, 63 418; Survey 1,
Hepar2 99, Insurance 580, Water 9 241, Alarm 160 104; 3sat-0 19 511.
