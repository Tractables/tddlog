#lang roulette/example/disrupt
;; A program of Roulette's language whose circuit scripts/roulette2problog.rkt
;; writes to tiny_pr.pl (tests/test_roulette2problog.py), with its answers
;; by hand (tests/it/examples.rs). Observed: a or b, of probability
;; 1 - 0.7 * 0.4 = 0.72.
;;
;;   q1, c: #f 0.03 / 0.72 (a and not the flip 0.9), #t 0.69 / 0.72.
;;   q2, (list a (not b)): (#f #f) 0.42 / 0.72, (#f #t) 0,
;;       (#t #f) 0.18 / 0.72, (#t #t) 0.12 / 0.72.
;;   q3, an integer: 1 0.3 / 0.72, 2 0.42 / 0.72, 3 0.
;;   q4, (equal? a c): #f 0.45 / 0.72, #t 0.27 / 0.72.
(define a (flip 0.3))
(define b (flip 0.6))
(define c (if a (flip 0.9) b))
(observe! (or a b))
(query c)
(query (list a (not b)))
(query (if a 1 (if b 2 3)))
(query (equal? a c))
