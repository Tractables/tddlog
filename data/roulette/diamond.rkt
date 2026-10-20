#lang roulette/example/disrupt
;; benchmark/modular/diamond.rkt of the Roulette artifact (Zenodo
;; 10.5281/zenodo.15253254), its last line changed to print P(s(N)) in
;; full (print.rkt).
(require "print.rkt")

(define (main n)
  (let go ([n n])
    (cond
      [(zero? n) #t]
      [else
       (define drop (flip 0.0001))
       (define route (flip 0.5))
       (define s1 (go (sub1 n)))
       (define s2 (if route s1 #f))
       (define s3 (if route #f s1))
       (or s2 (and s3 (not drop)))])))

(define SIZE (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (time (query (main SIZE))))
(emit (format "s(~a)" SIZE) (result #t))
