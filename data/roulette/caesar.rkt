#lang roulette/example/disrupt
;; benchmark/modular/caesar.rkt of the Roulette artifact (Zenodo
;; 10.5281/zenodo.15253254), its query written as neuppl/roulette's
;; `(query (main SIZE))` (the artifact's took a second argument, whether to
;; take one value's probability as one minus the others'), its seeding a
;; statement (print.rkt), and its last line changed to print the posterior
;; of the key, key(K) for K = 0..25, in full.
(require "print.rkt")

(define rng (make-pseudo-random-generator))
(statement (parameterize ([current-pseudo-random-generator rng]) (random-seed 0)))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(require rosette/lib/destruct)

(define (natural->bv k size)
  (cond
    [(zero? size) '()]
    [else (cons (odd? k) (natural->bv (truncate (/ k 2)) (sub1 size)))]))

(define (bv->natural xs)
  (destruct xs
    [(list) 0]
    [(cons x xt) (+ (if x 1 0) (* 2 (bv->natural xt)))]))

(define (bvadd xs ys)
  (let go ([xs xs] [ys ys] [cin #f])
    (match* (xs ys)
      [('() '()) '()]
      [((cons x xt) (cons y yt))
       (define x⊕y (xor x y))
       (cons (xor x⊕y cin) (go xt yt (or (and x y) (and cin x⊕y))))])))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(require (only-in rosette assert))

(define (discrete . xs)
  (bin-cat (map cons (range (length xs)) xs)
           (inexact->exact (expt 2 (ceiling (log (length xs) 2))))))

(define (bin-cat xs size)
  (match xs
    [(list) (assert #f)]
    [(list (cons x _)) (natural->bv x size)]
    [_
     (define-values (left right)
       (split-at xs (floor (/ (length xs) 2))))
     (define left-sum (foldl + 0 (map cdr left)))
     (if (flip left-sum)
         (bin-cat (renormalize left left-sum) size)
         (bin-cat (renormalize right (- 1 left-sum)) size))]))

(define (renormalize xs n)
  (for/list ([x+y (in-list xs)])
    (cons (car x+y) (/ (cdr x+y) n))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(define (draw-char key observation)
  (define drawn-char
    (discrete 0.08167 0.01492 0.02782
              0.04253 0.04378 0.02228
              0.02015 0.06094 0.06966
              0.0153 0.0772 0.04025
              0.02406 0.06749 0.07507
              0.01929 0.00095 0.05987
              0.06327 0.09056 0.02758
              0.00978 0.02360 0.00150
              0.01974 0.00074))
  (define encrypted (bvadd key drawn-char))
  (define fail (flip 0.0001))
  (if fail
      (equal? observation drawn-char)
      (equal? observation encrypted)))

(define (main n)
  (define key1
    (discrete 0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538 0.038461538
              0.038461538 0.038461538))
  (define bools
    (for/list ([_ (in-range n)])
      (draw-char key1 (natural->bv (random 0 26 rng) 32))))
  (observe! (andmap values bools))
  (bv->natural key1))

(define SIZE (string->number (vector-ref (current-command-line-arguments) 0)))
(define result (time (query (main SIZE))))
(statement (for ([k (in-range 26)]) (emit (format "key(~a)" k) (result k) k)))
