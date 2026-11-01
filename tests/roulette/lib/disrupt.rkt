#lang roulette/example/disrupt
;; A program in the example language of Roulette's package `roulette`,
;; whose circuit scripts/roulette2problog.rkt writes to disrupt.pl
;; (tests/test_roulette2problog.py), with its weights by hand
;; (tests/it/examples.rs). Two random choices, a (0.3) and b (0.6), the
;; observation a or b, and the value x where a holds, y where b alone
;; does, z otherwise. Its `query` calls `infer` twice: the value where the
;; observation holds, ⊥ otherwise, and the observation.
;;
;;   q1: x 0.3, y 0.7 * 0.6 = 0.42, ⊥ 0.7 * 0.4 = 0.28; z, which only a
;;       failed observation takes, is not asked.
;;   q2: #t 0.72, #f 0.28.
;;
;; The query divides each value's weight by the sum of the values other
;; than ⊥: x 0.3 / 0.72 = 5/12, y 7/12, z 0, which the program prints.
(define (model)
  (define a (flip 0.3))
  (define b (flip 0.6))
  (observe! (or a b))
  (if a 'x (if b 'y 'z)))
(define m (query (model)))
;; A definition, which the language does not run as a query of its own.
(define-values ()
  (begin (printf "~a ~a ~a\n" (m 'x) (m 'y) (m 'z))
         (values)))
