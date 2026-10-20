#lang racket/base
;; racket hardware.rkt FUEL: benchmark/scaling/hardware.rkt of the
;; Roulette artifact under rsdd with bit width 2 and n = FUEL - 2 (`-m rsdd
;; -p (2 n FUEL)`, the artifact's fuel ladder), its distribution printed
;; in full (print.rkt): res(T) for a total T, res(4) for #f (out of fuel).
(require (submod roulette/benchmark/scaling/hardware rsdd)
         roulette/example/private/pmf
         "print.rkt")
(define fuel (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (main 2 (max 0 (- fuel 2)) fuel))
(for ([(v p) (in-pmf result)])
  (emit (format "res(~a)" (or v 4)) p))
