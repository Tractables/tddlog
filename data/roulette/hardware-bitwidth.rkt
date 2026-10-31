#lang racket/base
;; racket hardware-bitwidth.rkt BW: benchmark/scaling/hardware.rkt of the
;; Roulette artifact under rsdd with bit width BW, n = 2 and fuel 3 (`-m
;; rsdd -p (BW 2 3)`, the artifact's bit-width ladder), its distribution
;; printed in full (print.rkt): res(T) for each total T, and for #f (out
;; of fuel) res(L), L one past the largest total printed, as
;; gen_roulette.py labels it.
(require (submod roulette/benchmark/scaling/hardware rsdd)
         roulette/example/private/pmf
         "print.rkt")
(define bw (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (main bw 2 3))
(define top (for/fold ([m -1]) ([(v p) (in-pmf result)] #:when v) (max m v)))
(for ([(v p) (in-pmf result)])
  (emit (format "res(~a)" (or v (add1 top))) p))
