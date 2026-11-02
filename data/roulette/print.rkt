#lang racket/base
;; Printing the answers of the drivers here: one `roulette_q ATOM P VALUE`
;; a line, P in full (Roulette's own printing rounds to six digits) and
;; VALUE the program's value of probability P, as `write` prints it, by
;; which bench.py reads the answers to the circuit of Roulette+tddlog.
;;
;; `statement` runs its forms as a definition. The language
;; `roulette/example/disrupt` runs each expression of a module as a query
;; of its own (it prints its distribution), and the drivers' printing is no
;; query.
(provide emit statement)
(define (emit atom p value)
  (printf "roulette_q ~a ~a ~s\n" atom (if (exact? p) (exact->inexact p) p) value))
(define-syntax-rule (statement e ...)
  (define-values () (begin e ... (values))))
