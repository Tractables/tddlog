% ABO blood types in a family of 7 generations of 7 people. The parents
% of person J of a generation are persons J and J + 1 of the generation
% before (the last one's are the last and the first), so cousins marry and
% the family tree is full of loops. Each founder's two alleles follow the
% population's frequencies; a child inherits one of its mother's alleles
% and one of its father's, each with probability 1/2. Given the blood types
% of the youngest generation, what were the founders'?
% Run with `tddlog examples/larger/blood_types.pl`. `--evidence-probability` adds P(e).
width(7).
generations(7).
person(p(I, J)) :- generations(G), width(W), between(1, G, I), between(1, W, J).
parents(p(I, J), p(H, J), p(H, K)) :-
    person(p(I, J)), I > 1, H is I - 1, width(W), K is J mod W + 1.
side(m).
side(f).
0.28::allele(p(1, J), S, a); 0.07::allele(p(1, J), S, b); 0.65::allele(p(1, J), S, o) :-
    person(p(1, J)), side(S).
% from(C, S, T): the allele C has from its parent on side S is that
% parent's allele on side T.
0.5::from(C, S, m); 0.5::from(C, S, f) :- parents(C, _, _), side(S).
allele(C, m, A) :- parents(C, M, _), from(C, m, T), allele(M, T, A).
allele(C, f, A) :- parents(C, _, F), from(C, f, T), allele(F, T, A).
has(P, A) :- allele(P, _, A).
blood(P, ab) :- has(P, a), has(P, b).
blood(P, a) :- has(P, a), \+ has(P, b).
blood(P, b) :- has(P, b), \+ has(P, a).
blood(P, o) :- person(P), \+ has(P, a), \+ has(P, b).
evidence(blood(p(7, 1), a)).
evidence(blood(p(7, 2), a)).
evidence(blood(p(7, 3), a)).
evidence(blood(p(7, 4), o)).
evidence(blood(p(7, 5), a)).
evidence(blood(p(7, 6), a)).
evidence(blood(p(7, 7), a)).
query(blood(p(1, 1), T)).
query(has(p(1, J), b)) :- width(W), between(1, W, J).
