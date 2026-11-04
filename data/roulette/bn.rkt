#lang roulette/example/disrupt
;; racket bn.rkt NETWORK ASKS: a Bayesian network in Roulette's own BN
;; language, neuppl/roulette's `roulette/example/bn` (its reader and its
;; `make-run`, which the package's tests and the artifact's
;; benchmark/bayes.rkt run), on the BIF file NETWORK, asked what the file
;; ASKS asks, a line each:
;;
;;   order V ...   the variables evaluated, in this order (make-run's
;;                 var-order); every variable below and its parents are
;;                 among them
;;   observe V K   V observed at its K-th value (from 0, in the order the
;;                 file declares its values)
;;   query V       the distribution of V given the observations
;;   holds V K     with no query line: the probability that every holds
;;                 line's V takes its K-th value, nothing observed (P(e))
;;
;; The network is evaluated once at the module's level, the observations
;; observed there (`observe!`), and each query is asked as the language
;; asks an expression of a module (`display-pmf`: every random choice
;; summed out under the observations), its distribution read back from the
;; table it prints and printed in full, `roulette_b V VALUE P` for each
;; value of V of positive probability (VALUE as `write` writes it, as the
;; notes of Roulette's tddlog engine write the values of a call of
;; `infer`); with no query, `roulette_e P` for the holds lines'
;; conjunction. Reading the file is part of the run, as the artifact reads
;; its .bif inside its `time`.
(require racket/file
         racket/list
         racket/match
         racket/port
         racket/string
         (only-in roulette/example/bn make-run)
         (only-in (submod roulette/example/bn reader) [read bn-read])
         "print.rkt")

;; Every form of the module is a definition or a `statement` (print.rkt):
;; the language asks each expression of a module as a query of its own, two
;; calls of `infer` (its value's and its evidence's), and the circuit of
;; Roulette's tddlog engine numbers the calls, so that the K-th query here,
;; or P(e), is the (2K-1)-th call, as bench.py reads them.
(define ARGS (current-command-line-arguments))
(statement
 (unless (= (vector-length ARGS) 2)
   (raise-user-error 'bn.rkt "usage: racket bn.rkt NETWORK ASKS")))
(define NETWORK (vector-ref ARGS 0))

;; The blocks of the BIF file as `#lang roulette/example/bn`'s reader reads
;; them and `make-run` takes them: the reader's module is
;; (module _ LANG ... (define run (make-run 'BLOCKS)) ...).
(define BLOCKS
  (or (for/first ([f (in-list (call-with-input-file NETWORK bn-read))]
                  #:when (and (pair? f) (eq? (car f) 'define) (eq? (cadr f) 'run)))
        (cadr (cadr (caddr f))))
      (raise-user-error 'bn.rkt "~a: no blocks" NETWORK)))
;; Each variable's values, in the order the file declares them.
(define DOMAINS
  (for/fold ([h (hash)]) ([b (in-list BLOCKS)])
    (match b
      [`(variable ,v (type discrete (,_) ,vals)) (hash-set h v vals)]
      [_ h])))

(define (variable s)
  (define v (string->symbol s))
  (unless (hash-has-key? DOMAINS v)
    (raise-user-error 'bn.rkt "~a has no variable ~a" NETWORK v))
  v)
(define (value-of v k)
  (define vals (hash-ref DOMAINS v))
  (define i (string->number k))
  (unless (and (exact-nonnegative-integer? i) (< i (length vals)))
    (raise-user-error 'bn.rkt "~a has no value ~a of its ~a" v k (length vals)))
  (list-ref vals i))

;; The asks: the order, the observations and the holds lines as (variable
;; . value), and the queried variables.
(define-values (ORDER OBSERVED QUERIED HELD)
  (for/fold ([order #f] [observed '()] [queried '()] [held '()]
             #:result (values order (reverse observed) (reverse queried) (reverse held)))
            ([line (in-list (file->lines (vector-ref ARGS 1)))]
             #:unless (string=? (string-trim line) ""))
    (match (string-split line)
      [(cons "order" vs)
       (when order (raise-user-error 'bn.rkt "two order lines"))
       (values (map variable vs) observed queried held)]
      [(list "observe" v k) (values order (cons (cons (variable v) (value-of (variable v) k)) observed) queried held)]
      [(list "query" v) (values order observed (cons (variable v) queried) held)]
      [(list "holds" v k) (values order observed queried (cons (cons (variable v) (value-of (variable v) k)) held))]
      [_ (raise-user-error 'bn.rkt "not an ask: ~a" line)])))
(statement
 (unless ORDER
   (raise-user-error 'bn.rkt "no order line"))
 (unless (if (null? QUERIED) (null? OBSERVED) (null? HELD))
   (raise-user-error 'bn.rkt "holds lines ask P(e) alone: no observe or query line beside them")))

;; The network evaluated once: the value of each variable asked.
(define ASKED (remove-duplicates (append (map car OBSERVED) QUERIED (map car HELD))))
(define VALUE
  (for/hash ([v (in-list ASKED)]
             [x (in-list (call-with-values (λ () ((make-run BLOCKS) ASKED ORDER)) list))])
    (values v x)))

(statement
 (for ([o (in-list OBSERVED)])
   (observe! (equal? (hash-ref VALUE (car o)) (cdr o)))))

;; The distribution of x, a list of (value . probability): Roulette's query
;; at the module's level (`display-pmf`), read back from the table it
;; prints, or from the one value it prints alone (as dice2roulette.py's
;; programs read it). A row of the table is a line that starts with the
;; table's frame, split at that character only: a value that reads as a
;; number prints between bars ('|0|), in a row and alone.
(define (distribution x)
  (define text (with-output-to-string (λ () (display-pmf x))))
  (define (datum s)
    (define d (read (open-input-string s)))
    (if (and (pair? d) (eq? (car d) 'quote)) (cadr d) d))
  (define rows
    (for*/list ([line (in-list (string-split text "\n"))]
                [l (in-value (string-trim line))]
                #:when (and (positive? (string-length l)) (memv (string-ref l 0) '(#\│ #\|))))
      (filter (λ (s) (not (string=? s "")))
              (map string-trim (string-split l (string (string-ref l 0)) #:trim? #f)))))
  (if (null? rows)
      (list (cons (datum text) 1))
      (for/list ([r (in-list rows)]
                 #:unless (equal? r '("Value" "Probability")))
        (cons (datum (car r)) (string->number (cadr r))))))
(define (real p) (if (exact? p) (exact->inexact p) p))

(statement
 (for ([q (in-list QUERIED)])
   (for ([e (in-list (distribution (hash-ref VALUE q)))])
     (unless (member (car e) (hash-ref DOMAINS q))
       (raise-user-error 'bn.rkt "~a takes ~s, which it does not declare" q (car e)))
     (printf "roulette_b ~a ~s ~a\n" q (car e) (real (cdr e)))))
 (when (null? QUERIED)
   (define e (apply && (for/list ([h (in-list HELD)]) (equal? (hash-ref VALUE (car h)) (cdr h)))))
   (printf "roulette_e ~a\n" (real (or (for/first ([d (in-list (distribution e))] #:when (eq? (car d) #t)) (cdr d)) 0)))))
