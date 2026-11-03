#lang roulette
;; A program of Roulette's package roulette-lib whose circuit
;; Roulette's tddlog engine writes to diagnosis.pl, with its weights by hand
;; (tests/it/examples.rs): a diagnosis of faults in a circuit. The inputs
;; x and y are 1 and 0, random choices of probability 1 and 0; an and gate
;; and an or gate each flip their output where they fail, f1 (0.1) and f2
;; (0.2): out = ((x and y) xor f1 or y) xor f2, 0 without a fault and here
;; f1 xor f2. Observed: out is 1, of probability 0.1 * 0.8 + 0.9 * 0.2 =
;; 0.26. Every guard asked is a conjunction with it, which is therefore the
;; circuit's evidence, and each weight is 0.26 times its query's
;; probability.
;;
;;   q1, the observation: #t 0.26.
;;   q2, f1 where it holds: #t 0.1 * 0.8 = 0.08.
;;   q3, f2 where it holds: #t 0.9 * 0.2 = 0.18.
;;   q4, f1 and f2 where it holds: #t 0.
(require racket/set
         roulette/engine/rsdd)
(struct bottom ())
(define ⊥ (bottom))
(define-measurable* x (bernoulli-measure 0.0 1.0))
(define-measurable* y (bernoulli-measure 1.0 0.0))
(define-measurable* f1 (bernoulli-measure 0.9 0.1))
(define-measurable* f2 (bernoulli-measure 0.8 0.2))
(define (gate f v) (if f (not v) v))
(define out (gate f2 (or (gate f1 (and x y)) y)))
(define eng (rsdd-engine))
(define (weight v) ((infer (if out v ⊥) #:engine eng) (set #t)))
(printf "~a\n" ((infer out #:engine eng) (set #t)))
(printf "~a ~a ~a\n" (weight f1) (weight f2) (weight (and f1 f2)))
