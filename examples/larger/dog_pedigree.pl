% A gene in a breeder's line of dogs, 7 generations of 7. The parents of
% dog J of a generation are dogs J and J + 1 of the generation before (the
% last one's are the last and the first), so the pedigree is full of loops.
% The gene has three alleles: a and b both show, and o shows only in a dog
% with two copies of it, as in ABO blood types. Each founder's two alleles
% follow the breed's frequencies; a puppy inherits one of its dam's
% alleles and one of its sire's, each with probability 1/2. Given what the
% youngest generation shows, what did the founders carry?
% Run with `tddlog examples/larger/dog_pedigree.pl`. `--evidence-probability` adds P(e).
width(7).
generations(7).
dog(d(I, J)) :- generations(G), width(W), between(1, G, I), between(1, W, J).
parents(d(I, J), d(H, J), d(H, K)) :-
    dog(d(I, J)), I > 1, H is I - 1, width(W), K is J mod W + 1.
side(dam).
side(sire).
0.28::allele(d(1, J), S, a); 0.07::allele(d(1, J), S, b); 0.65::allele(d(1, J), S, o) :-
    dog(d(1, J)), side(S).
% from(C, S, T): the allele C has from its parent on side S is that
% parent's allele on side T.
0.5::from(C, S, dam); 0.5::from(C, S, sire) :- parents(C, _, _), side(S).
allele(C, dam, A) :- parents(C, M, _), from(C, dam, T), allele(M, T, A).
allele(C, sire, A) :- parents(C, _, F), from(C, sire, T), allele(F, T, A).
has(D, A) :- allele(D, _, A).
shows(D, ab) :- has(D, a), has(D, b).
shows(D, a) :- has(D, a), \+ has(D, b).
shows(D, b) :- has(D, b), \+ has(D, a).
shows(D, o) :- dog(D), \+ has(D, a), \+ has(D, b).
evidence(shows(d(7, 1), a)).
evidence(shows(d(7, 2), a)).
evidence(shows(d(7, 3), a)).
evidence(shows(d(7, 4), o)).
evidence(shows(d(7, 5), a)).
evidence(shows(d(7, 6), a)).
evidence(shows(d(7, 7), a)).
query(shows(d(1, 1), T)).
query(has(d(1, J), b)) :- width(W), between(1, W, J).
