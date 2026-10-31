# noDice benchmark programs

The purely probabilistic benchmark programs of noDice (Gürtler and
Kaminski, "noDice: Inference for Discrete Probabilistic Programs with
Nondeterminism and Conditioning", OOPSLA 2026, DOI 10.1145/3798215,
arXiv 2602.20049), in Dice's syntax, from the paper's artifact (Zenodo
10.5281/zenodo.18739789, record licensed CC-BY-4.0; the repository's
LICENSE is the Apache License 2.0), directory `benchmarks/nodice/`.
Copied unchanged, the 3-SAT files renamed (`uf20-0990.cnf.dice` is
`uf20-0990.dice`), except the coupon collector at 16, 18 and 20 coupons
and the bounded retransmission protocol past 16 frames, which are
written here in the artifact's pattern (below).

| file | artifact path | paper's name |
|---|---|---|
| coupon2-10-obs.dice ... coupon2-14-obs.dice | coupon/ | CouponProb-10 ... CouponProb-14 |
| coupon2-2-obs.dice ... coupon2-9-obs.dice | coupon/ | (not in Table 1) |
| coupon2-16-obs.dice, coupon2-18-obs.dice, coupon2-20-obs.dice | written here | CouponProb-16, -18, -20 |
| survey.dice, hepar2.dice, insurance.dice, water.dice, alarm.dice | bayesian/ | Survey, Hepar2, Insurance, Water, Alarm |
| 3sat-0.dice | 3sat/ | 3sat-0 |
| uf50-0999.dice | 3sat/uf50-0999.cnf.dice | (not in Table 1) |
| uf20-0990.dice ... uf20-0999.dice | 3sat-20/ | (not in Table 1) |
| brp4-2.dice, brp6-2.dice, brp8-2.dice, brp10-2.dice, brp16-2.dice | brp/ | (not in Table 1) |
| brp64-2.dice, brp256-2.dice, brp1024-2.dice, brp4096-2.dice, brp16384-2.dice | written here | |
| diamond-N-p.dice, N = 1, 100 to 500, 700 to 1000, 2000 to 5000, 10000 | diamond-prob/ | (not in Table 1) |
| ladder-N-p.dice, N = 1, 10 to 100 by 10 | ladder-prob/ | (not in Table 1) |

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
uf50-0999 is that instance whole (50 variables, 218 clauses), and
uf20-0990 to uf20-0999 are SATLIB's uniform random 3-SAT instances of 20
variables and 91 clauses, written alike: the probability that a uniform
assignment satisfies the formula. None is a size of another.

brpN-2 is the bounded retransmission protocol (the PRISM model's N
frames, at most 2 retransmissions): each of N frames is sent up to three
times over a lossy channel (`flip 0.98`) and acknowledged over another
(`flip 0.99`), the frame counters `int(5)`, a step of `iterate` a frame;
it returns whether every frame was acknowledged. The artifact's files for
4 to 16 frames differ only in iterate's count. The sizes past 16 widen
the counters to hold N + 1:
`sed -e 's/int(5/int(W/g' -e 's/iterate(step, init, 16)/iterate(step, init, N)/' brp16-2.dice`
with (N, W) = (64, 7), (256, 9), (1024, 11), (4096, 13), (16384, 15).
The file's comments, the artifact's, are kept as they are.

diamond-N-p is N diamonds in a row, each routing its input one of two
ways (`flip 0.5`), one of them lossy (`flip 0.0001`), a function called N
times; it returns whether the signal arrives. ladder-N-p is a ladder of
N rungs over two wires, a function called N times, and returns the pair
of wires. They are the Dice paper's diamond and ladder networks, written
as the artifact writes them; data/dice's loops and Roulette's generator
(scripts/gen_roulette.py) write the same models otherwise.

The paper's other classes are left out: Runway, CouponNdet, Network,
Rabin, the `-Ndet` networks and 3sat-25 to 3sat-100 choose
nondeterministically (`nflip`), which Dice, Alea.jl and ProbLog do not
have; so do the `diamond/` directory's diamonds. The artifact's `toy/`
programs (small coupon collectors, Monty Hall, tests) are toys, and
`bayesian/reorder/` holds the five networks again with their variables in
another order. The paper's Table 1 times Dice on the programs here (ms,
Apple M2 Pro): CouponProb-10 to -14 721, 1 958, 5 798, 20 691, 63 418;
Survey 1, Hepar2 99, Insurance 580, Water 9 241, Alarm 160 104; 3sat-0
19 511.
