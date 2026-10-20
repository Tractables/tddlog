#lang racket/base
;; racket network.rkt K: benchmark/scaling/network.rkt of the Roulette
;; artifact under rsdd (`-m rsdd -p (K)`), its distribution printed in
;; full (print.rkt): m(K,V) for a message whose bits, least significant
;; first, make V, m(K,16) for #f (lost). Roulette's distribution also holds #t
;; at a probability of 0 up to rounding; it is printed as other(#t).
(require (submod roulette/benchmark/scaling/network rsdd)
         roulette/example/private/pmf
         racket/format
         "print.rkt")
(define k (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (main k))
(for ([(v p) (in-pmf result)])
  (cond
    [(not v) (emit (format "m(~a,16)" k) p)]
    [(list? v) (emit (format "m(~a,~a)" k (for/sum ([b v] [i (in-naturals)]) (if b (expt 2 i) 0))) p)]
    [else (emit (format "other(~a)" (~s v)) p)]))
