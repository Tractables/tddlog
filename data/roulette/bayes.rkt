#lang racket/base
;; racket bayes.rkt NAME: the Bayesian network NAME of the Roulette
;; artifact's benchmark/main.rkt (its .bif, queried variable and variable
;; order, read from main.rkt as the artifact's driver builds them) under
;; benchmark/bayes.rkt with rsdd, its distribution printed in full:
;; `roulette_v VALUE P` for each value of the queried variable.
(require (submod roulette/benchmark/bayes rsdd)
         roulette/example/private/pmf
         racket/match
         racket/path)
(define name (vector-ref (current-command-line-arguments) 0))
(define dir (path-only (collection-file-path "main.rkt" "roulette" "benchmark")))
(define body
  (let* ([mod (parameterize ([read-accept-reader #t])
                (call-with-input-file (build-path dir "main.rkt") read))]
         [b (cadddr mod)])
    (if (eq? (car b) '#%module-begin) (cdr b) (cdddr mod))))
(define def
  (for/first ([f body] #:when (and (pair? f) (eq? (car f) 'define)
                                   (eq? (cadr f) (string->symbol (string-upcase name)))))
    (caddr f)))
(define params
  (match def
    [(list 'quasiquote (list (list 'unquote (list 'bif n)) q order))
     (list (path->string (build-path dir "bayesian-networks" (format "~a.bif" n))) q order)]
    [_ (error 'bayes.rkt "no network ~a in main.rkt" name)]))
(define result (apply main params))
(for ([(v p) (in-pmf result)])
  (printf "roulette_v ~a ~a\n" v (if (exact? p) (exact->inexact p) p)))
