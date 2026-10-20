#lang racket/base
;; Printing the answers of the drivers here: one `roulette_q ATOM P` a
;; line, P in full (Roulette's own printing rounds to six digits).
(provide emit)
(define (emit atom p)
  (printf "roulette_q ~a ~a\n" atom (if (exact? p) (exact->inexact p) p)))
