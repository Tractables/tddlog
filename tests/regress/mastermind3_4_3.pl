% mastermind: 3 colours, 4 pegs, 3 guesses, seed 1
% scripts/lit_exact.py mastermind: the answers are in src/auto/tests.rs, a_trial_that_waits_past_the_ceiling_resumes_at_its_retry_reach.
0.333333333333::code(1,c0); 0.333333333333::code(1,c1); 0.333333333333::code(1,c2).
0.333333333333::code(2,c0); 0.333333333333::code(2,c1); 0.333333333333::code(2,c2).
0.333333333333::code(3,c0); 0.333333333333::code(3,c1); 0.333333333333::code(3,c2).
0.333333333333::code(4,c0); 0.333333333333::code(4,c1); 0.333333333333::code(4,c2).
peg(0,1).
peg(1,2).
peg(2,3).
peg(3,4).
colour(0,c0).
colour(1,c1).
colour(2,c2).
round(g0).
guess(g0,1,c0).
guess(g0,2,c1).
guess(g0,3,c1).
guess(g0,4,c1).
guessed(g0,c0,1).
guessed(g0,c1,3).
guessed(g0,c2,0).
round(g1).
guess(g1,1,c2).
guess(g1,2,c1).
guess(g1,3,c0).
guess(g1,4,c0).
guessed(g1,c0,2).
guessed(g1,c1,1).
guessed(g1,c2,1).
round(g2).
guess(g2,1,c1).
guess(g2,2,c0).
guess(g2,3,c1).
guess(g2,4,c1).
guessed(g2,c0,1).
guessed(g2,c1,3).
guessed(g2,c2,0).
black(G,0,0) :- round(G).
black(G,K,B) :- peg(J,K), black(G,J,B0), guess(G,K,X), code(K,X), B is B0+1.
black(G,K,B) :- peg(J,K), black(G,J,B), guess(G,K,X), code(K,Y), X \= Y.
count(X,0,0) :- colour(_,X).
count(X,K,N) :- peg(J,K), count(X,J,N0), code(K,X), N is N0+1.
count(X,K,N) :- peg(J,K), count(X,J,N), code(K,Y), X \= Y.
common(G,X,N) :- guessed(G,X,M), count(X,4,N), N =< M.
common(G,X,M) :- guessed(G,X,M), count(X,4,N), N > M.
matched(G,0,0) :- round(G).
matched(G,I,T) :- colour(J,X), I is J+1, matched(G,J,T0), common(G,X,M), T is T0+M.
white(G,W) :- matched(G,3,T), black(G,4,B), W is T-B.
evidence(black(g0,4,2)).
evidence(white(g0,0)).
evidence(black(g1,4,1)).
evidence(white(g1,3)).
evidence(black(g2,4,1)).
evidence(white(g2,1)).
query(code(1,c0)).
query(code(1,c1)).
query(code(1,c2)).
query(code(2,c0)).
query(code(2,c1)).
query(code(2,c2)).
query(code(3,c0)).
query(code(3,c1)).
query(code(3,c2)).
query(code(4,c0)).
query(code(4,c1)).
query(code(4,c2)).
