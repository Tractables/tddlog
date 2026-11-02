#lang roulette/example/disrupt
;; racket hmm.rkt QUERY N: the hidden Markov model of the Roulette
;; artifact's benchmark/hmm.rkt (Zenodo 10.5281/zenodo.15253254), its
;; Roulette implementation alone (the Dice and dynamic programming ones run
;; other systems), N steps, QUERY one of its five patterns. The model, the
;; queries and the parameters are the artifact's; its query is written as
;; neuppl/roulette's `(query e)` (the artifact's was
;; `(with-observe (query e #f))`, the observations' normalization and no
;; complement shortcut), its printing of the answer replaced by print.rkt's
;; `emit` of q, P(#t), and its `time` kept around the run.
(require "print.rkt")

;; start : [0,1]
;; step : bool -> [0,1]
;; obs : bool -> [0,1]
(define-struct hmm-parameters (start step obs) #:transparent)

;; n-steps : nat
;; parameters : hmm-parameters
;; query : hmm-query(n-steps)
(define-struct configuration (n-steps parameters query) #:transparent)

;; variable(n-steps) = hidden {0..n-steps} + visible {1..n-steps}
(define-struct hidden (id) #:transparent)
(define-struct visible (id) #:transparent)

;; event(n-steps) = variable(n-steps) -assoclist-> bool
;; outcomes(n-steps) : event(n-steps)
;; conditions(n-steps) : event(n-steps)
;; NOTE: assoclist, and not hash, ensures iteration happens in ascending order
;; Random access is never used
(define-struct hmm-query (outcomes conditions) #:transparent)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Query patterns

;; n-steps:nat -> event(n-steps)
(define (all-visible-variables-true n-steps)
  (for/list ([i (in-range n-steps)]) (cons (visible (add1 i)) #t)))
(define (last-hidden-variable-true n-steps) (list (cons (hidden n-steps) #t)))
(define (first-hidden-variable-true n-steps) (list (cons (hidden 0) #t)))
(define (first-half-hidden-variables-true n-steps)
  (for/list ([i (in-range 0 (floor (/ n-steps 2)))])
    (cons (hidden i) #t)))
(define (second-half-hidden-variables-true n-steps)
  (for/list ([i (in-range (floor (/ n-steps 2)) (add1 n-steps))])
    (cons (hidden i) #t)))

;; n-steps:nat -> hmm-query(n-steps)
(define (smoothing-marginal n-steps)
  (hmm-query (first-hidden-variable-true n-steps)
             (all-visible-variables-true n-steps)))
(define (smoothing-joint n-steps)
  (hmm-query (first-half-hidden-variables-true n-steps)
             (all-visible-variables-true n-steps)))
(define (filtering-marginal n-steps)
  (hmm-query (last-hidden-variable-true n-steps)
             (all-visible-variables-true n-steps)))
(define (filtering-joint n-steps)
  (hmm-query (second-half-hidden-variables-true n-steps)
             (all-visible-variables-true n-steps)))
(define (marginal-likelihood n-steps)
  (hmm-query (all-visible-variables-true n-steps)
             (list)))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;; Roulette implementation

;; configuration -> float
(define (run-hmm-roulette config)
  ;; roulette-model = (variable -hash-> random bool)
  ;; nat parameters -> roulette-model
  (define (make-model n-steps parameters)
    (match-define (hmm-parameters start step obs) parameters)
    (define initial-state (flip start))
    (for/fold ([state initial-state]
               [model (hash (hidden 0) initial-state)]
               #:result model)
              ([i (in-range n-steps)])
      (define state- (if state (flip (step #t)) (flip (step #f))))
      (define observation (if state- (flip (obs #t)) (flip (obs #f))))
      (define i- (add1 i))
      (define model- (hash-set (hash-set model (hidden i-) state-) (visible i-) observation))
      (values state- model-)))
  ;; roulette-model (cons variable bool) -> random bool
  (define (interpret-literal model xb)
    (define v (hash-ref model (car xb)))
    (if (cdr xb) v (! v)))
  ;; (n-steps:nat) roulette-model hmm-query(n-steps) -> random bool
  (define (query-model n-steps model q)
    (match-define (hmm-query outcomes conditions) q)
    (for ([xb conditions])
      (observe! (interpret-literal model xb)))
    (for/fold ([res #t]) ([xb (reverse outcomes)])
      (&& (interpret-literal model xb) res))
    #;(for/fold ([res #t]) ([xb outcomes])
      (&& res (interpret-literal model xb))))
  (match-define (configuration n-steps params q) config)
  ((query (query-model n-steps (make-model n-steps params) q)) #t))

;; main

(define example-hmm-parameters
  (hmm-parameters
   0.5
   (λ (b) (if b 0.3 0.65))
   (λ (b) (if b 0.65 0.3))))

(define QUERIES
  (hash 'marginal-likelihood marginal-likelihood
        'filtering-marginal filtering-marginal
        'smoothing-marginal smoothing-marginal
        'filtering-joint filtering-joint
        'smoothing-joint smoothing-joint))

(define ARGS (current-command-line-arguments))
(define QUERY (hash-ref QUERIES (string->symbol (vector-ref ARGS 0))))
(define N-STEPS (string->number (vector-ref ARGS 1)))
(define result
  (time (run-hmm-roulette (configuration N-STEPS example-hmm-parameters (QUERY N-STEPS)))))
(statement (emit "q" result #t))
