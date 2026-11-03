#lang roulette/example/disrupt
;; A program of Roulette's language whose circuit Roulette's tddlog engine
;; wrote to tiny_path.pl on
;; the PLDI 2025 artifact's Roulette, with its answers by hand
;; (tests/it/examples.rs). b is observed on the path where
;; a holds: the evidence is (not a) or b, of probability 0.5 + 0.125.
;;
;;   q1, a: #f 0.5 / 0.625 = 0.8, #t 0.125 / 0.625 = 0.2.
;;   q2, a constant: 7, probability 1.
(define a (flip 0.5))
(define b (flip 0.25))
(when a (observe! b))
(query a)
(query 7)
