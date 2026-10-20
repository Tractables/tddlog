#lang roulette/example/disrupt
;; benchmark/modular/motivating.rkt of the Roulette artifact (Zenodo
;; 10.5281/zenodo.15253254), its last line changed to print P(x(N)) in
;; full (print.rkt).
(require "print.rkt")

(define (main n)
  (let go ([n n])
    (cond
      [(zero? n) (flip 0.1)]
      [else
       (define x (flip 0.2))
       (define y (flip 0.3))
       (if (go (sub1 n)) x y)])))

(define SIZE (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (time (query (main SIZE))))
(emit (format "x(~a)" SIZE) (result #t))
