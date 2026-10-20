# Fairness of a decision tree (FairSquare, Albarghouthi, D'Antoni, Drews and
# Nori, OOPSLA 2017; SPPL, Saad, Rinard and Mansinghka, PLDI 2021, Table 3):
# a population model of the Adult census data and a decision tree trained on
# it. The program returns two Booleans: that the tree's decision is the
# fairness target (t = 0) and the person is female, and that it is the
# target and the person is male. The tree's fairness ratio is
# (q1 / 0.3307) / (q2 / 0.6693). The population model is FairSquare's
# second Bayes net (`M_BNc_...`), and `dt` picks the tree: 4, 14, 16 or 44
# (`F_DT_V2_D2_N4`, `F_DT_V2_D3_N14`, `F_DT_V2_D2_N16`, `F_DT_V3_D2_N44`),
# from oopsla/noqual of github.com/sedrews/fairsquare at bd27437 (MIT
# License). Each tree's population model draws only the variables the tree
# reads; drawing the others too, as here, leaves its answers unchanged.
#
# The translation: FairSquare's `gaussian(m, v)` takes a variance and is
# bit blasted here as a normal of standard deviation sqrt(v), in
# exponential pieces (bitblast_exponential), on an interval of more than 4
# standard deviations on each side; its `step` over unit intervals,
# compared only with integers, is a categorical; a capital gain is in
# units of 1024 dollars, its thresholds divided alike; `if education_num >
# age: age = education_num` is an `ifelse`. `bits` is the bits after the
# binary point of every number.
using Alea, Distributions

const QUERY = :pr
const dt = 4
const bits = 6
const pieces = 32
const DFiP = DistFix{8 + bits, bits}
const K = 1024.0

gauss(m, v, lo, hi) = bitblast_exponential(DFiP, Normal(m, sqrt(v)), pieces, lo, hi)
categorical(W, p) = discrete(DistUInt{W}, p ./ sum(p))
below(x, c) = x < DFiP(c)
is(r, k) = prob_equals(r, DistUInt{3}(k))

# The population (FairSquare's M_BNc_F_DT_V3_D2_N44, with the
# education_num of its other trees).
function population()
    female = flip(0.3307)
    cg_f = gauss(568.4105 / K, 24248365.5428 / K^2, -64.0, 64.0)
    cg_m = gauss(1329.3700 / K, 69327473.1006 / K^2, -64.0, 64.0)
    capital_gain = ifelse(female, cg_f, cg_m)
    low = ifelse(female, below(cg_f, 7298.0000 / K), below(cg_m, 5178.0000 / K))
    pick(f_low, f_high, m_low, m_high) = ifelse(female, ifelse(low, f_low, f_high), ifelse(low, m_low, m_high))
    age = pick(gauss(38.4208, 184.9151, -32.0, 96.0), gauss(38.8125, 193.4918, -32.0, 96.0),
               gauss(38.6361, 187.2435, -32.0, 96.0), gauss(38.2668, 187.2747, -32.0, 96.0))
    education = pick(
        categorical(4, [0.1638, 0.2308, 0.0354, 0.3230, 0.0173, 0.0321, 0.0412, 0.0156,
                        0.0200, 0.0112, 0.0528, 0.0050, 0.0290, 0.0119, 0.0092, 0.0017]),
        categorical(4, [0.1916, 0.2000, 0.0500, 0.3542, 0.0208, 0.0125, 0.0375, 0.0125,
                        0.0292, 0.0042, 0.0541, 0.0000, 0.0250, 0.0042, 0.0042, 0.0000]),
        categorical(4, [0.1670, 0.2239, 0.0358, 0.3267, 0.0159, 0.0320, 0.0426, 0.0155,
                        0.0198, 0.0121, 0.0518, 0.0047, 0.0287, 0.0125, 0.0096, 0.0014]),
        categorical(4, [0.1569, 0.2205, 0.0417, 0.3071, 0.0255, 0.0302, 0.0409, 0.0155,
                        0.0178, 0.0147, 0.0619, 0.0062, 0.0317, 0.0139, 0.0139, 0.0016]))
    education_num = pick(gauss(10.0827, 6.5096, -6.0, 26.0), gauss(10.1041, 6.1522, -6.0, 26.0),
                         gauss(10.0817, 6.4841, -6.0, 26.0), gauss(10.0974, 7.1793, -6.0, 26.0))
    relationship = pick(
        categorical(3, [0.0491, 0.1556, 0.4012, 0.2589, 0.0294, 0.1058]),
        categorical(3, [0.0416, 0.1667, 0.4583, 0.2292, 0.0166, 0.0876]),
        categorical(3, [0.0497, 0.1545, 0.4021, 0.2590, 0.0294, 0.1053]),
        categorical(3, [0.0417, 0.1624, 0.3976, 0.2606, 0.0356, 0.1021]))
    age = ifelse(age < education_num, education_num, age)
    (female, capital_gain, age, education, relationship)
end

# The trees: whether t = 1.
function tree(capital_gain, age, education, relationship)
    if dt == 4
        ifelse(below(capital_gain, 7073.5 / K), true, below(age, 20.0))
    elseif dt == 14
        ifelse(is(relationship, 0), true,
        ifelse(is(relationship, 1), ifelse(below(age, 21.5), true, below(age, 47.5)),
        ifelse(is(relationship, 2), true,
        ifelse(is(relationship, 3), below(age, 50.5),
        ifelse(is(relationship, 4), below(age, 49.5), true)))))
    elseif dt == 16
        ifelse(is(relationship, 0), below(capital_gain, 5095.5 / K),
        ifelse(is(relationship, 1), below(capital_gain, 4718.5 / K),
        ifelse(is(relationship, 2), below(capital_gain, 5095.5 / K),
        ifelse(is(relationship, 3), below(capital_gain, 8296.0 / K),
        ifelse(is(relationship, 4), true, below(capital_gain, 4668.5 / K))))))
    elseif dt == 44
        # t by education (0 to 15) where relationship is 0 and where it is 2
        t0 = [0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1]
        t2 = [0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1]
        by_education(t) = reduce(|, [prob_equals(education, DistUInt{4}(e)) for e in 0:15 if t[e + 1] == 1])
        ifelse(is(relationship, 0), by_education(t0),
        ifelse(is(relationship, 1), below(capital_gain, 4718.5 / K),
        ifelse(is(relationship, 2), by_education(t2),
        ifelse(is(relationship, 3), below(capital_gain, 8296.0 / K),
        ifelse(is(relationship, 4), true, below(capital_gain, 4668.5 / K))))))
    else
        error("no tree $dt")
    end
end

model() = @alea begin
    (female, capital_gain, age, education, relationship) = population()
    target = !tree(capital_gain, age, education, relationship)
    (female & target, !female & target)
end
