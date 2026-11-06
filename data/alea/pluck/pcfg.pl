% pcfg: ported to ProbLog from PluckArtifact.jl/src/table1/sequence_models/
% pluck/pcfg.jl of the artifact of Bowers, Lew, Tenenbaum, Solar-Lezama
% and Mansinghka, "Stochastic Lazy Knowledge Compilation for Inference in
% Discrete Probabilistic Programs" (PLDI 2025), Zenodo
% 10.5281/zenodo.15051140 (CC-BY-4.0): the Pluck program of the paper's
% Table 1, row PCFG. The query is the probability that the grammar
%   S  -> XX YY | YY XX     (by a flip 0.5)
%   XX -> a | b XX XX       (a by a flip 0.5)
%   YY -> c | c XX | c S    (c alone by a flip 0.5, then XX or S by another)
% generates the program's 23-symbol string from S.
%
% The generation reads the string left to right: s(I, J), x(I, J) and
% y(I, J) hold where S, XX or YY, generated at position I, is the
% string's symbols I to J - 1. Each choice is keyed by its symbol and its
% position, which gives the original's distribution where no derivation
% expands one symbol twice at one position. The grammar has neither left
% recursion nor an empty production: S's productions start with XX or YY,
% and XX's and YY's with a terminal, so a derivation expands at a position
% at most S and then one of XX and YY, each once. No generation then
% reads one key twice, a derivation has the original's probability, and
% the choices at its keys fix it, so that two derivations exclude each
% other.
w(0, c). w(1, b). w(2, b). w(3, b). w(4, a). w(5, b). w(6, b). w(7, a).
w(8, b). w(9, a). w(10, b). w(11, b). w(12, a). w(13, b). w(14, a). w(15, a).
w(16, a). w(17, a). w(18, a). w(19, b). w(20, a). w(21, a). w(22, a).
symbols(23).

0.5::x_ends(I) :- w(I, _).     % XX at I is a (otherwise b XX XX)
0.5::s_order(I) :- w(I, _).    % S at I is XX YY (otherwise YY XX)
0.5::y_ends(I) :- w(I, _).     % YY at I is c alone
0.5::y_xx(I) :- w(I, _).       % YY at I goes on with XX (otherwise S)

x(I, J) :- w(I, a), x_ends(I), J is I + 1.
x(I, J) :- w(I, b), \+x_ends(I), I1 is I + 1, x(I1, K), x(K, J).
y(I, J) :- w(I, c), y_ends(I), J is I + 1.
y(I, J) :- w(I, c), \+y_ends(I), y_xx(I), I1 is I + 1, x(I1, J).
y(I, J) :- w(I, c), \+y_ends(I), \+y_xx(I), I1 is I + 1, s(I1, J).
s(I, J) :- s_order(I), x(I, K), y(K, J).
s(I, J) :- \+s_order(I), y(I, K), x(K, J).

generated :- symbols(N), s(0, N).
query(generated).
