#lang roulette/example/disrupt
;; benchmark/modular/ladder.rkt of the Roulette artifact (Zenodo
;; 10.5281/zenodo.15253254), its last line changed to print the two
;; marginals o1(N) = P((#t #f)) and o2(N) = P((#f #t)) in full (print.rkt).
(require "print.rkt")

(define (main n)
  (let go ([n n])
    (cond
      [(zero? n) (list #t #f)]
      [else
       (define a (flip 0.001))
       (define b (flip 0.5))
       (match-define (list s1 s2)
         (go (sub1 n)))
       (cond
         [(and s1 a)
          (list #f #f)]
         [(or s1 s2)
          (list b (not b))]
         [else
          (list #f #f)])])))

(define SIZE (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (time (query (main SIZE))))
(emit (format "o1(~a)" SIZE) (result (list #t #f)))
(emit (format "o2(~a)" SIZE) (result (list #f #t)))
