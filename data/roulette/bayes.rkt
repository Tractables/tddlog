#lang roulette/example/disrupt
;; racket bayes.rkt NAME: the Bayesian network NAME of the Roulette
;; artifact's benchmark/main.rkt, as neuppl/roulette's package `roulette`
;; holds it (roulette/test/bn/NAME.rkt, `#lang roulette/example/bn`), with
;; the queried variable and the variable order of main.rkt
;; (networks.rktd), evaluated by `make-run` of roulette/example/bn, the
;; artifact's benchmark/bayes.rkt in the package. The file is read inside
;; the query's `time`, as the artifact's `main` reads its .bif there. The
;; distribution is printed in full: `roulette_v VALUE P` for each value of
;; the queried variable.
(require racket/file
         racket/match
         (only-in roulette/example/bn make-run)
         "blocks.rkt"
         "print.rkt")

(define NAME (string->symbol (vector-ref (current-command-line-arguments) 0)))
(define HERE
  (let-values ([(dir _name _dir?) (split-path (variable-reference->module-source (#%variable-reference)))])
    dir))
(define-values (QUERY ORDER)
  (match (assq NAME (file->value (build-path HERE "networks.rktd")))
    [(list _ q order) (values q order)]
    [_ (error 'bayes.rkt "no network ~a in networks.rktd" NAME)]))
(define FILE (collection-file-path (format "~a.rkt" NAME) "roulette" "test" "bn"))

(define result
  (time (query ((make-run (bif-blocks FILE #:lang? #t)) (list QUERY) ORDER))))
(statement
 (for ([(v p) (in-pmf result)])
   (printf "roulette_v ~a ~a\n" v (if (exact? p) (exact->inexact p) p))))
