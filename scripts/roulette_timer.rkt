#lang racket/base
;; racket roulette_timer.rkt PROGRAM.rkt [ARG ...]: runs a Roulette program
;; as `racket PROGRAM.rkt ARG ...` runs it (its module, then its main
;; submodule where it has one, under ARG ... as the command line) and prints
;;   roulette_timer: program S s
;; to stderr: the seconds of the program body's run, once Racket, Roulette's
;; modules (roulette, roulette/safe, roulette/engine/rsdd), the program's
;; declaration and the modules it imports are loaded. That is the span of
;; the `program S s` of Roulette's tddlog engine (`racket -l-
;; roulette/engine/tddlog`, the branch tddlog of Tractables/roulette),
;; which loads the same before its clock, so that Roulette's own time and
;; the charge of Roulette+tddlog's export pay the same part of a run:
;; Racket's start and the modules' load are its warm-up, as for the other
;; systems timed warm. bench.py keeps S as the record's `inproc_seconds`
;; (run_circulette).
(require racket/vector)

(define argv (current-command-line-arguments))
(when (zero? (vector-length argv))
  (raise-user-error 'roulette_timer "usage: racket roulette_timer.rkt PROGRAM.rkt [ARG ...]"))
(define mp `(file ,(path->string (path->complete-path (vector-ref argv 0)))))
(for ([m (in-list '(roulette roulette/safe roulette/engine/rsdd))])
  (dynamic-require m #f))
;; The program declared, and the modules it imports at phase 0 instantiated.
(define (instantiate-imports! mp)
  (define root (module-path-index-join mp #f))
  (define (rooted mpi)
    (define-values (name base) (module-path-index-split mpi))
    (cond [(not name) root]
          [(module-path-index? base) (module-path-index-join name (rooted base))]
          [else mpi]))
  (for* ([pi (in-list (module->imports mp))] #:when (eqv? (car pi) 0) [mpi (in-list (cdr pi))])
    (dynamic-require (rooted mpi) #f)))
(module-declared? mp #t)
(instantiate-imports! mp)
(define t0 (current-inexact-milliseconds))
(parameterize ([current-command-line-arguments (vector-drop argv 1)])
  (dynamic-require mp #f)
  (when (module-declared? `(submod ,mp main) #t)
    (dynamic-require `(submod ,mp main) #f)))
(flush-output (current-output-port))
(eprintf "roulette_timer: program ~a s\n" (/ (- (current-inexact-milliseconds) t0) 1000.0))
