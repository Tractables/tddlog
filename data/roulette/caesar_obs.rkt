#lang racket/base
;; racket caesar_obs.rkt N: the first N observations of Roulette's caesar
;; benchmark (benchmark/modular/caesar.rkt of the artifact), one a line:
;; (random 0 26 rng) from a generator seeded with (random-seed 0), as the
;; benchmark draws them.
(define rng (make-pseudo-random-generator))
(parameterize ([current-pseudo-random-generator rng]) (random-seed 0))
(define n (string->number (vector-ref (current-command-line-arguments) 0)))
(for ([_ (in-range n)])
  (displayln (random 0 26 rng)))
