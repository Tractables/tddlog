% A program where a derived atom depends on itself, which conditioning leaves
% as it is. b holds exactly when sb does or sa and eab do, so the evidence
% has probability 0.8 * (1 - 0.3 * 0.5) = 0.68; a holds with it exactly when
% sa does and eab does not, 0.3 * 0.5 * 0.8 = 0.12, and eab exactly when sa
% does not, 0.5 * 0.7 * 0.8 = 0.28.
0.3::sa.
0.2::sb.
0.5::eab.
0.4::eba.
a :- sa.
a :- b, eba.
b :- sb.
b :- a, eab.
evidence(b,false).
query(a).
query(sb).
query(eab).
