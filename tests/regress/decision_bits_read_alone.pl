% coin_bias_mux.pl's trie of flips and decision list, whose bias bits are
% each read alone with one flip outside the tree, observed: the readers do
% not grow when the bits' names are composed away, so settling expands the
% tree's cut (tp::settle). Queries: the bias bits; answers ProbLog 2.3.0's.
0.353746::f1.
0.638973::f2.
0.605510::f3.
g1 :- f1, f2.
g1 :- f3, \+f1.
0.402028::f4.
0.498174::f5.
0.479796::f6.
0.560637::f7.
g2 :- g1, f4.
g2 :- f5, \+g1.
g3 :- g1, f6.
g3 :- f7, \+g1.
g4 :- f1, g2.
g4 :- g3, \+f1.
g5 :- f1, g1, g4.
0.673234::f8.
g6 :- g5, f8.
g7 :- f1, g1, \+g4.
0.256316::f9.
g8 :- g7.
g8 :- g5, \+g7.
g9 :- g7.
g9 :- g5, \+g7.
g10 :- g5, \+g7.
g11 :- g7, f9.
g11 :- g6, \+g7.
g12 :- f1, \+g1, g4.
0.217008::f10.
g13 :- g12.
g13 :- g8, \+g12.
g14 :- g9, \+g12.
g15 :- g12.
g15 :- g10, \+g12.
g16 :- g12, f10.
g16 :- g11, \+g12.
g17 :- f1, \+g1, \+g4.
0.701459::f11.
g18 :- g17.
g18 :- g13, \+g17.
g19 :- g14, \+g17.
g20 :- g15, \+g17.
g21 :- g17, f11.
g21 :- g16, \+g17.
g22 :- \+f1, g1, g4.
0.459660::f12.
g23 :- g18, \+g22.
g24 :- g22.
g24 :- g19, \+g22.
g25 :- g22.
g25 :- g20, \+g22.
g26 :- g22, f12.
g26 :- g21, \+g22.
g27 :- \+f1, g1, \+g4.
0.657368::f13.
g28 :- g23, \+g27.
g29 :- g27.
g29 :- g24, \+g27.
g30 :- g25, \+g27.
g31 :- g27, f13.
g31 :- g26, \+g27.
g32 :- \+f1, \+g1, g4.
0.201264::f14.
g33 :- g28, \+g32.
g34 :- g29, \+g32.
g35 :- g32.
g35 :- g30, \+g32.
g36 :- g32, f14.
g36 :- g31, \+g32.
g37 :- \+f1, \+g1, \+g4.
0.467232::f15.
g38 :- g33, \+g37.
g39 :- g34, \+g37.
g40 :- g35, \+g37.
g41 :- g37, f15.
g41 :- g36, \+g37.
0.500000::f16.
0.500000::f17.
0.500000::f16.
0.300000::f17.
0.600000::f18.
0.450000::f19.
x0 :- g38, f16.
x1 :- g39, \+f17.
x1 :- \+g39, f17.
x2 :- g40, f18.
x2 :- \+g40, \+f18.
x3 :- g41.
x3 :- f19.
b(0) :- g38.
query(b(0)).
b(1) :- g39.
query(b(1)).
b(2) :- g40.
query(b(2)).
b(3) :- g41.
query(b(3)).
evidence(x0, false).
evidence(x1, true).
evidence(x2, true).
evidence(x3, true).
