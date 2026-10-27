%Expected outcome:
% gen([],[b,b,a]) 0.147
% gen([],[b,c,a]) 0
%
% The expected outcomes are computed by hand: each derivation D chooses
% stop(D) of its own, and [b,b,a] is two steps that do not stop and one
% that does, 0.7 * 0.7 * 0.3. No clause derives a word with c: its clause
% asks that kind(b, D, _) fail, and kind's facts hold for every D. The
% derivation is an argument of kind that no clause reads, which the
% grounder drops; the negation is then of a predicate given by facts
% alone, and filters the demand of the clause's recursive call.
0.3::stop(D).
kind(a, _D, leaf).
kind(b, _D, node).
gen(D, [a]) :- stop(D), kind(a, D, leaf).
gen(D, [b|W]) :- \+ stop(D), kind(b, D, node), length(D, N), N < 3, gen([x|D], W).
gen(D, [c|W]) :- \+ kind(b, D, _), length(D, N), N < 3, gen([y|D], W).
query(gen([],[b,b,a])).
query(gen([],[b,c,a])).
