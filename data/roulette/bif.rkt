#lang roulette/example/disrupt
;; racket bif.rkt BIF QUERIES: a network as the Roulette artifact runs its
;; own (bayes.rkt), under `make-run` of neuppl/roulette's
;; roulette/example/bn with rsdd, on a BIF file that gen_bn.py writes
;; (bif_programs): for each line `Q V...` of the file QUERIES, the
;; distribution of the variable Q with the variables V evaluated in that
;; order, the file read and the network evaluated inside the query's `time`
;; once a variable, printed after the `time` line as `roulette_b Q VALUE P`.
(require racket/file
         racket/string
         (only-in roulette/example/bn make-run)
         "blocks.rkt"
         "print.rkt")

(define ARGS (current-command-line-arguments))
(define BIF (vector-ref ARGS 0))
(statement
 (for ([line (in-list (file->lines (vector-ref ARGS 1)))]
       #:unless (equal? (string-trim line) ""))
   (define words (map string->symbol (string-split line)))
   (define result
     (time (query ((make-run (bif-blocks BIF)) (list (car words)) (cdr words)))))
   (for ([(v p) (in-pmf result)])
     (printf "roulette_b ~a ~a ~a\n" (car words) v (if (exact? p) (exact->inexact p) p)))))
