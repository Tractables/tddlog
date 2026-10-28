#lang racket/base
;; racket bif.rkt BIF QUERIES: a network as the Roulette artifact runs its
;; own, under benchmark/bayes.rkt's `main` with rsdd, on a BIF file that
;; gen_bn.py writes (bif_programs): for each line `Q V...` of the file
;; QUERIES, the distribution of the variable Q with the variables V
;; evaluated in that order, one `main` a variable, printed after main's
;; `time` line as `roulette_b Q VALUE P`.
(require (submod roulette/benchmark/bayes rsdd)
         roulette/example/private/pmf
         racket/file
         racket/string)
(define args (current-command-line-arguments))
(define bif (vector-ref args 0))
(for ([line (in-list (file->lines (vector-ref args 1)))]
      #:unless (equal? (string-trim line) ""))
  (define words (map string->symbol (string-split line)))
  (define result (main bif (car words) (cdr words)))
  (for ([(v p) (in-pmf result)])
    (printf "roulette_b ~a ~a ~a\n" (car words) v (if (exact? p) (exact->inexact p) p))))
