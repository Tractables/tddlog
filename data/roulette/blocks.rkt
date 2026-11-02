#lang racket/base
;; The blocks of a BIF file, as the reader of `#lang roulette/example/bn`
;; (neuppl/roulette's package `roulette`) reads them and its `make-run`
;; takes them. With `#:lang? #t` the file's first line is a `#lang` line,
;; as in the package's networks (roulette/test/bn/NAME.rkt).
(require (only-in (submod roulette/example/bn reader) [read bn-read]))
(provide bif-blocks)
(define (bif-blocks file #:lang? [lang? #f])
  (call-with-input-file file
    (λ (in)
      (when lang? (read-line in))
      ;; The reader's module: (module _ LANG ... (define run (make-run 'BLOCKS)) ...).
      (or (for/first ([f (in-list (bn-read in))]
                      #:when (and (pair? f) (eq? (car f) 'define) (eq? (cadr f) 'run)))
            (cadr (cadr (caddr f))))
          (error 'blocks.rkt "~a: no blocks" file)))))
