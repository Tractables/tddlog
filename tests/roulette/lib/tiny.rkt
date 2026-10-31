#lang roulette
;; A program of Roulette's package roulette-lib whose circuit
;; scripts/roulette2problog.rkt writes to tiny.pl
;; (tests/test_roulette2problog.py), with its weights by hand
;; (tests/it/examples.rs). Three random choices, a (0.3), b (0.6) and c
;; (0.9), and d, which is c where a holds and b otherwise. No conjunct is
;; common to the guards asked, so the circuit has no evidence, and each
;; weight is its query's probability.
;;
;;   q1, d through rsdd's engine: #f 0.31, #t 0.3 * 0.9 + 0.7 * 0.6 = 0.69.
;;   q2, (list a (not b)) through infer's default engine, each value its
;;       support asks about: (#f #f) 0.7 * 0.6 = 0.42, (#f #t) 0.28,
;;       (#t #f) 0.18, (#t #t) 0.12.
;;   q3, (and a b): #t 0.18; 'none, a value it never takes, weighs 0 and
;;       is not written.
;;   The fourth call's measure is not asked about, and it is not written.
(require racket/set
         roulette/engine/rsdd)
(define-measurable* a (bernoulli-measure 0.7 0.3))
(define-measurable* b (bernoulli-measure 0.4 0.6))
(define-measurable* c (bernoulli-measure 0.1 0.9))
(define d (if a c b))
(define eng (rsdd-engine))
(define m1 (infer d #:engine eng))
(printf "~a ~a\n" (m1 (set #f)) (m1 (set #t)))
(define m2 (infer (list a (not b))))
(for ([v (in-list '((#f #f) (#f #t) (#t #f) (#t #t)))]
      #:when (set-member? (support m2) v))
  (printf "~a ~a\n" v (m2 (set v))))
(define m3 (infer (and a b) #:engine eng))
(printf "~a ~a\n" (m3 (set 'none)) (m3 (set #t)))
(void (infer (or a b) #:engine eng))
