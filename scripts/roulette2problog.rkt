#lang racket/base
;; roulette2problog.rkt: the Boolean circuit Roulette builds for a program,
;; written as a ProbLog program that tddlog reads.
;;
;;     racket roulette2problog.rkt PROGRAM.rkt [ARG ...] > PROGRAM.pl
;;     racket roulette2problog.rkt --tddlog TDDLOG PROGRAM.rkt [ARG ...]
;;     racket roulette2problog.rkt --make PROGRAM.rkt
;;
;; It runs from its source, not compiled by `raco make`.
;;
;; Roulette (Moy, Czenszak, Li, Marshall and Holtzen, "Roulette: A Language
;; for Expressive, Exact, and Efficient Discrete Probabilistic Programming",
;; PLDI 2025) evaluates a Racket program symbolically with Rosette: every
;; random choice is a symbolic Boolean constant, and every Boolean the
;; program computes from them is a Rosette term over them, a DAG of !, &&,
;; ||, => and <=>. A query flattens a value into the values it can take,
;; each with the Boolean term (its guard) under which it does, and rsdd
;; compiles each guard to a BDD, its variables the random choices in the
;; order they were created, and counts its models.
;;
;; The script reads the programs of two versions of Roulette, whichever is
;; installed, and is compiled for that one:
;;
;; - Roulette 54d6185, the PLDI 2025 artifact, on Racket 8.16 with Rosette
;;   4.1, run with its packages installed as its README says (PLTUSERHOME
;;   and PLTADDONDIR naming the installation): programs in its language
;;   `#lang roulette/example/disrupt`, the language of the paper's
;;   programs, whose `query` conjoins each guard with the observations
;;   (`observe!`) and normalizes.
;; - Roulette 60f77cb, the package roulette-lib of Racket's package catalog,
;;   on Racket 9.3: programs in `#lang roulette` or `roulette/safe`, whose
;;   `infer` hands a value to an engine, rsdd's (roulette/engine/rsdd) by
;;   default, and returns a measure that gives each value the program asks
;;   about its weight: the weighted model count of its guard conjoined with
;;   Rosette's assumptions, not normalized. With the package `roulette`
;;   beside it, also programs in its example language `#lang
;;   roulette/example/disrupt` (the paper's programs, their `query` written
;;   as this version's), whose `query` asks `infer` twice, for the value
;;   under the observations and for the observations, and normalizes.
;;
;; The script runs PROGRAM.rkt, with ARG ... as its command line, with
;; Roulette's modules that answer queries replaced by this file's
;; submodules, which hand the queries to this script instead: the
;; artifact's `roulette/example/disrupt/rsdd` by `language`, the same
;; language with the exporter's `query`, and the package's `roulette`,
;; `roulette/safe` and `roulette/engine/rsdd` by `engine-roulette`,
;; `engine-safe` and `engine-rsdd`, the same modules with the exporter's
;; engine as `infer`'s default engine and as `rsdd-engine`'s engines. A
;; module compiled against one of the replaced modules (by `raco make`) is
;; compiled again against this file's; the others, Roulette's own and the
;; artifact's drivers among them, run compiled. rsdd answers no query. --make
;; does that compilation ahead of the runs, as `raco make` does Roulette's:
;; it compiles PROGRAM.rkt's modules, and the artifact's language module
;; with them, into compiled/roulette2problog/ beside their sources, and a run
;; loads them from there while this file and their sources are unchanged;
;; without it, each run compiles them anew. It refuses a program none of
;; whose modules `raco make` has compiled against the replaced modules.
;;
;; Without --tddlog, the queries are recorded and answer nothing (the
;; artifact's an empty distribution, the package's measures a weight of 0
;; for every value; 1 for a value a call can take where the program runs
;; on the package's example language `roulette/example/disrupt`, whose
;; `query` divides each value's weight by their sum and stops on a sum of
;; 0), the program's own output goes to standard error, and
;; once the program has run the script writes one ProbLog program for all
;; its queries to standard output: a random choice is a probabilistic fact
;; `P::fN.` (the N-th of the circuit in the order Roulette created them),
;; an && a rule, an || a rule for each operand, a ! a negated literal, a <=>
;; two rules and a => two rules; the evidence is `evidence(...)`, one for
;; each of its conjuncts. The rules come in the order Rosette created the
;; terms, every gate after the gates it reads. The K-th query of the
;; program is asked as `qK(I)` for its I-th value, in the order of
;; `datum-order`, the value written in a comment beside it.
;;
;; - The artifact's evidence is the observations, and every value of a
;;   query is asked, qK(I) its guard: tddlog's answer for qK(I) is the
;;   probability of the value given the observations, which Roulette's
;;   query computes (without its `#:complement-of` shortcut, which takes one
;;   value's probability as one minus the others'). The queries must be
;;   asked under the same observations, which one ProbLog program holds.
;; - The package's queries are the calls of `infer`, and a value is asked
;;   where the program asks its weight (of the measure, its density or its
;;   support). Each asked value's guard, conjoined with the assumptions, is a
;;   conjunction: the conjuncts that every one shares are the evidence, and
;;   qK(I) is the conjunction of the others. The value's weight is P(e)
;;   times tddlog's answer for qK(I), as `tddlog --evidence-probability`
;;   prints them. A value asked that the call cannot take comes after the
;;   call's values, in the order asked, as `qK(I) :- fail.`: its weight is
;;   0. A note counts the calls of `infer`, whether or not they asked a
;;   value.
;;
;; With --tddlog TDDLOG (a command, split at spaces), each query is written
;; on its own to a temporary file and answered by TDDLOG on it, and the query
;; returns tddlog's distribution, or its weights, the package's one value at
;; a time: Roulette with tddlog in place of rsdd. The program's output is its
;; own; each query's sizes and the seconds of writing its circuit and of the
;; tddlog run go to standard error.
;;
;; The artifact's `define-compile`, Roulette's staged procedures, compiles to
;; BDDs before any query and is refused, as are flips whose probability is
;; symbolic and measures that are not probabilities. The package's
;; `rsdd-engine` is refused a semiring other than numbers' (#:semiring), its
;; `infer` variables kept symbolic (#:keep), and its measures weights of #f
;; and #t that do not sum to one; a random choice of probability 0 or 1 is
;; written as such, `0.0::fN.` or `1.0::fN.`.
;;
;; The language submodule follows Roulette's example/disrupt/rsdd.rkt
;; (Apache License 2.0), its `query` replaced.

;; `(roulette-case [NAME FORM ...] ...)`: the forms of the first clause whose
;; Roulette is installed, `disrupt` the artifact's (its language
;; roulette/example/disrupt) and `engine` the package's (its engines,
;; roulette/private/core.rkt). Where neither is, the script does not compile.
(module installed racket/base
  (require (for-syntax racket/base))
  (provide roulette-case)
  (begin-for-syntax
    (define FILES '((disrupt "rsdd.rkt" "roulette" "example" "disrupt")
                    (engine "core.rkt" "roulette" "private")))
    (define (installed? name)
      (define where (cdr (assq name FILES)))
      (define p (apply collection-file-path (car where) (cdr where) #:fail (λ (_) #f)))
      (and p (file-exists? p))))
  (define-syntax (roulette-case stx)
    (syntax-case stx ()
      [(_ clause ...)
       (or (for/first ([c (in-list (syntax->list #'(clause ...)))]
                       #:when (installed? (syntax-e (car (syntax->list c)))))
             #`(begin #,@(cdr (syntax->list c))))
           (raise-syntax-error
            'roulette2problog
            "finds no Roulette: neither the PLDI 2025 artifact's (roulette/example/disrupt) nor the package roulette-lib (roulette/private/core.rkt) is installed"
            stx))])))
(require 'installed)

;; The circuit of the queries as a ProbLog program, and tddlog's answers.
(module circuit racket/base
  (require racket/list
           racket/match
           racket/port
           racket/string
           racket/system
           racket/file
           data/order
           rosette/base/core/term
           (only-in rosette/base/core/bool @! @&& @|| @=> @<=>)
           (only-in roulette/private/util flatten-symbolic))

  (provide tddlog-command
           program-label
           fail
           seconds-since
           value-guards
           conjuncts
           write-circuit
           tddlog-answers)

  ;; The command that answers each query, or #f to record the queries.
  (define tddlog-command (make-parameter #f))
  ;; The program's name and arguments, for the circuit's first line.
  (define program-label (make-parameter "a Roulette program"))

  (define (fail fmt . args)
    (raise-user-error 'roulette2problog (apply format fmt args)))

  (define (seconds-since t) (/ (- (current-inexact-milliseconds) t) 1000.0))

  ;; The values of `e` with their guards, in datum order: a list of (value
  ;; . guard), a guard #t, #f or a Boolean term.
  (define (value-guards e)
    (define entries (hash->list (flatten-symbolic e)))
    (with-handlers ([exn:fail? (λ (_) (sort entries string<? #:key (λ (x) (format "~s" (car x)))))])
      (sort entries (order-<? datum-order) #:key car)))

  ;; The conjuncts of a Boolean, the && of their conjunction flattened.
  (define (conjuncts e)
    (define seen (make-hasheq))
    (define out '())
    (let walk ([e e])
      (match e
        [#t (void)]
        [(expression (== @&&) xs ...) (for-each walk xs)]
        [_ (unless (hash-ref seen e #f)
             (hash-set! seen e #t)
             (set! out (cons e out)))]))
    (reverse out))

  (define BOOLEAN-OPS (list @! @&& @|| @=> @<=>))

  (define (children n)
    (match n
      [(expression op xs ...)
       (unless (memq op BOOLEAN-OPS)
         (fail "a guard holds ~a, which is not a Boolean connective" op))
       (for ([x (in-list xs)])
         (unless (term? x)
           (fail "a guard holds the constant ~s inside ~a" x op)))
       xs]
      [(? constant?) '()]
      [_ (fail "a guard holds ~s, which is not a Boolean term" n)]))

  ;; The terms under `roots`, each once, in the order Rosette created them
  ;; (a term's operands before it).
  (define (circuit-nodes roots)
    (define seen (make-hasheq))
    (let loop ([stack (filter term? roots)])
      (unless (null? stack)
        (define n (car stack))
        (cond
          [(hash-ref seen n #f) (loop (cdr stack))]
          [else
           (hash-set! seen n #t)
           (loop (append (children n) (cdr stack)))])))
    (sort (hash-keys seen) < #:key term-id))

  (define (negated? n) (match n [(expression (== @!) _) #t] [_ #f]))

  ;; The ProbLog program of `queries` under `evidence`, to `out`: `queries`
  ;; a list of (k entries), each entry (i value body) with `body` #f for a
  ;; value never taken or a list of terms, its conjuncts; `evidence` a list
  ;; of terms, its conjuncts, #f where it is false; `probability` gives each
  ;; random choice's probability; `notes`, comment lines after the sizes.
  (define (write-circuit out queries evidence probability #:notes [notes '()])
    (define bodies (for*/list ([q (in-list queries)] [e (in-list (second q))] #:when (third e))
                     (third e)))
    (define nodes (circuit-nodes (append (append* bodies) evidence)))
    (define names (make-hasheq))
    (define-values (flips gates)
      (for/fold ([fs '()] [gs '()] [nf 0] [ng 0] #:result (values (reverse fs) (reverse gs)))
                ([n (in-list nodes)])
        (cond
          [(constant? n)
           (hash-set! names n (format "f~a" (add1 nf)))
           (values (cons n fs) gs (add1 nf) ng)]
          [(negated? n) (values fs gs nf ng)]
          [else
           (hash-set! names n (format "g~a" (add1 ng)))
           (values fs (cons n gs) nf (add1 ng))])))
    (define (lit n [pos #t])
      (match n
        [(expression (== @!) x) (lit x (not pos))]
        [_ (define name (hash-ref names n))
           (if pos name (string-append "\\+" name))]))
    (define (rule head . bodies)
      (for ([body (in-list bodies)])
        (fprintf out "~a :- ~a.\n" head (string-join body ", "))))
    (define n-queries (for/sum ([q (in-list queries)]) (length (second q))))
    (fprintf out "% ~a: the circuit Roulette builds, written by tddlog's scripts/roulette2problog.rkt.\n"
             (program-label))
    (fprintf out "% ~a flips, ~a gates, ~a observations, ~a queries.\n"
             (length flips) (length gates) (length evidence) n-queries)
    (for ([note (in-list notes)])
      (fprintf out "% ~a\n" note))
    (for ([f (in-list flips)])
      (fprintf out "~a::~a.\n" (number->string (exact->inexact (probability f))) (hash-ref names f)))
    (for ([g (in-list gates)])
      (define head (hash-ref names g))
      (match g
        [(expression (== @&&) xs ...) (rule head (map lit xs))]
        [(expression (== @||) xs ...) (apply rule head (for/list ([x (in-list xs)]) (list (lit x))))]
        [(expression (== @<=>) x y) (rule head (list (lit x) (lit y)) (list (lit x #f) (lit y #f)))]
        [(expression (== @=>) x y) (rule head (list (lit x #f)) (list (lit y)))]))
    (for ([o (in-list evidence)])
      (match o
        [#f (fprintf out "evidence(false_observation, true).\n")]
        [(expression (== @!) x) (fprintf out "evidence(~a, false).\n" (lit x))]
        [_ (fprintf out "evidence(~a, true).\n" (lit o))]))
    (for* ([q (in-list queries)])
      (for ([e (in-list (second q))])
        (match-define (list i value body) e)
        (define atom (format "q~a(~a)" (first q) i))
        (fprintf out "% ~a: ~s\n" atom value)
        (match body
          [#f (fprintf out "~a :- fail.\n" atom)]
          ['() (fprintf out "~a.\n" atom)]
          [_ (fprintf out "~a :- ~a.\n" atom (string-join (map lit body) ", "))])
        (fprintf out "query(~a).\n" atom)))
    (values (length flips) (length gates) (length evidence) n-queries))

  (define ANSWER #px"^q([0-9]+)\\(([0-9]+)\\):\t(.*)$")
  (define EVIDENCE #px"^P\\(e\\):\t([^\t]*)\t")

  ;; tddlog's answers (the command `cmd`) for the k-th query, whose program
  ;; `writer` writes to a port (returning its sizes, as `write-circuit`): a
  ;; hash of each value's index to its probability, #f where the evidence is
  ;; inconsistent, and with `evidence-probability?` P(e), #f otherwise.
  (define (tddlog-answers cmd k writer #:evidence-probability? [evidence-probability? #f])
    (define file (make-temporary-file "roulette2problog~a.pl"))
    (define t0 (current-inexact-milliseconds))
    (define-values (nf ng no nq) (call-with-output-file file #:exists 'truncate writer))
    (define written (seconds-since t0))
    (define words (string-split cmd))
    (define exe (or (find-executable-path (car words)) (fail "no command ~a" (car words))))
    (define t1 (current-inexact-milliseconds))
    (define text
      (with-output-to-string
        (λ ()
          (unless (apply system* exe (append (cdr words)
                                             (if evidence-probability? '("--evidence-probability") '())
                                             (list "--precision" "17" (path->string file))))
            (fail "~a failed on query ~a (~a)" cmd k file)))))
    (define ran (seconds-since t1))
    (delete-file file)
    (eprintf "roulette2problog: query ~a: ~a values, ~a flips, ~a gates, ~a observations; circuit ~a s, tddlog ~a s\n"
             k nq nf ng no written ran)
    (define probs (and (not (regexp-match? #rx"^evidence is inconsistent" text)) (make-hash)))
    (define pe #f)
    (for ([line (in-list (string-split text "\n"))])
      (match (regexp-match ANSWER line)
        [(list _ kk i p) #:when (and probs (equal? (string->number kk) k))
         (hash-set! probs (string->number i) (string->number p))]
        [_ (match (regexp-match EVIDENCE line)
             [(list _ p) (set! pe (string->number p))]
             [_ (void)])]))
    (values probs pe)))

(roulette-case
 [disrupt
  ;; The artifact's queries, recorded or answered by tddlog.
  (module export racket/base
    (require racket/list
             racket/match
             data/ddict
             roulette/private/measure
             roulette/example/private/pmf
             (submod ".." circuit))

    (provide MARKERS
             REPLACED
             ROULETTE-DIRS
             recorded-count
             write-program
             export-query)

    ;; The modules a module compiled against which is compiled again.
    (define MARKERS '(roulette/example/disrupt roulette/example/disrupt/rsdd))
    ;; Each module the program's requires are sent from, and this file's
    ;; submodule they are sent to.
    (define REPLACED '((roulette/example/disrupt/rsdd . language)))
    ;; The directories of Roulette's modules whose requires are not sent.
    (define ROULETTE-DIRS '())

    ;; The queries recorded so far, the last first: (k entries evidence).
    (define recorded '())
    (define (recorded-count) (length recorded))

    ;; Roulette's `query` of `e` under the observations `evidence`.
    (define (export-query e evidence)
      (define k (add1 (length recorded)))
      (define entries (value-guards e))
      (set! recorded (cons (list k entries evidence) recorded))
      (define cmd (tddlog-command))
      (if cmd (answer cmd k entries evidence) (pmf (hash))))

    ;; A flip's probability, its measure.
    (define (flip-probability c)
      (define m (ddict-ref measures c #f))
      (unless m
        (fail "a guard holds the symbolic constant ~a, which is no flip (define-compile is not supported)" c))
      (define p (measure-value m))
      (unless (and (real? p) (< 0 p 1))
        (fail "the flip ~a has the measure ~s, not a probability strictly between 0 and 1" c p))
      p)

    ;; The ProbLog program of `queries`, a list of (k entries evidence) that
    ;; share their evidence, to `out`.
    (define (write-queries out queries)
      (define evidence (third (first queries)))
      (for ([q (in-list (cdr queries))])
        (unless (eq? (third q) evidence)
          (fail "query ~a is asked under other observations than query 1, and one ProbLog program holds one set of evidence"
                (first q))))
      (write-circuit out
                     (for/list ([q (in-list queries)])
                       (list (first q)
                             (for/list ([e (in-list (second q))] [i (in-naturals)])
                               (list i (car e) (match (cdr e) [#t '()] [#f #f] [g (list g)])))))
                     (conjuncts evidence)
                     flip-probability))

    (define (write-program out) (write-queries out (reverse recorded)))

    ;; The distribution of the k-th query, of values and guards `entries`
    ;; under `evidence`, as tddlog (the command `cmd`) answers its circuit.
    (define (answer cmd k entries evidence)
      (define-values (probs _pe)
        (tddlog-answers cmd k (λ (out) (write-queries out (list (list k entries evidence))))))
      (pmf (if probs
               (for/hash ([e (in-list entries)] [i (in-naturals)])
                 (values (car e) (hash-ref probs i (λ () (fail "tddlog gave no answer for q~a(~a)" k i)))))
               (hash)))))

  ;; Roulette's language `#lang roulette/example/disrupt`, as its
  ;; example/disrupt/rsdd.rkt defines it, with `query` the exporter's.
  (module language roulette
    (provide
     (except-out (all-from-out roulette) #%module-begin #%top-interaction)
     (rename-out [module-begin #%module-begin]
                 [top-interaction #%top-interaction])
     query
     flip
     observe!
     with-observe
     define-compile
     pmf
     pmf?
     pmf-support
     in-pmf
     for/pmf)

    (require (for-syntax racket/base)
             (only-in rosette/base/core/bool @boolean?)
             roulette/private/measure
             roulette/example/private/pmf
             (submod ".." export))

    (define-syntax module-begin (make-wrapping-module-begin #'wrap))
    (define-syntax top-interaction (make-wrapping-top-interaction #'wrap))

    (define (wrap e)
      (if (symbolic? e) (query e) e))

    (define (query e [opt? #t])
      (export-query e evidence))

    (define (real-measure r)
      (measure r @boolean?))

    (define (flip pr)
      (cond
        [(= pr 0) #f]
        [(= pr 1) #t]
        [else
         (for/all ([pr pr])
           (define-symbolic* x (real-measure pr))
           x)]))

    (define-syntax (define-compile stx)
      (raise-syntax-error #f "staged procedures compile to BDDs and are not supported by roulette2problog.rkt" stx))

    (define evidence #t)

    (define (observe! e)
      (set! evidence (&& evidence e)))

    (define-syntax-rule (with-observe body0 body ...)
      (let ([old evidence])
        (begin0
          (begin body0 body ...)
          (set! evidence old)))))]
 [engine
  ;; The package's engine, which records the values `infer` hands it, or
  ;; has tddlog weigh them.
  (module engine racket/base
    (require racket/class
             racket/list
             racket/match
             racket/set
             (only-in racket/contract/base any/c)
             (only-in rosette vc vc-assumes)
             (only-in rosette/base/core/bool @&&)
             (only-in roulette [infer roulette-infer] immutable-set/c)
             (only-in roulette/engine/rsdd number-semiring)
             (only-in roulette/private/engine engine<%>)
             (only-in roulette/private/measure measure measure-density in-measures)
             (submod ".." circuit))

    (provide MARKERS
             REPLACED
             ROULETTE-DIRS
             recorded-count
             write-program
             rsdd-engine
             infer)

    ;; The modules a module compiled against which is compiled again.
    (define MARKERS '(roulette roulette/safe roulette/engine/rsdd))
    ;; Each module the program's requires are sent from, and this file's
    ;; submodule they are sent to.
    (define REPLACED '((roulette . engine-roulette)
                       (roulette/safe . engine-safe)
                       (roulette/engine/rsdd . engine-rsdd)))
    ;; The directories of Roulette's modules whose requires are not sent:
    ;; roulette-lib's, whose core makes its default engine with rsdd's
    ;; `rsdd-engine`.
    (define ROULETTE-DIRS
      (let-values ([(dir _name _dir?) (split-path (collection-file-path "main.rkt" "roulette"))])
        (list (simplify-path (path->complete-path dir)))))

    (define WEIGHTS
      "A value's weight is P(e) times its query's probability, both of which tddlog --evidence-probability prints.")

    ;; A call of `infer`: its number `k`, its value's values and their guards
    ;; in datum order (a vector of (value . guard)), each value's index, the
    ;; values the program asked that the call cannot take, the last first,
    ;; each with the guard #f and the next index, Rosette's assumptions at
    ;; the call, the indices of the values whose weight the program asked,
    ;; and the weights tddlog gave them.
    (struct query (k entries index [others #:mutable] assumes asked weights))

    ;; The calls so far, the last first.
    (define recorded '())
    (define calls 0)

    ;; The calls with a value asked.
    (define (recorded-count)
      (for/sum ([q (in-list recorded)]) (if (hash-empty? (query-asked q)) 0 1)))

    (define (record! val)
      (set! calls (add1 calls))
      (define entries (list->vector (value-guards val)))
      (define q (query calls
                       entries
                       (make-hash (for/list ([e (in-vector entries)] [i (in-naturals)]) (cons (car e) i)))
                       '()
                       (vc-assumes (vc))
                       (make-hasheqv)
                       (make-hasheqv)))
      (set! recorded (cons q recorded))
      q)

    ;; The i-th (value . guard) of the call `q`.
    (define (query-entry q i)
      (define n (vector-length (query-entries q)))
      (if (< i n)
          (vector-ref (query-entries q) i)
          (list-ref (reverse (query-others q)) (- i n))))

    ;; A value's weight while the queries are recorded: 0, or 1 where the
    ;; program runs on the example language `roulette/example/disrupt`,
    ;; whose `query` normalizes the weights of a call's values (and of the
    ;; evidence's) by their sum and stops where it is 0. A program on
    ;; `infer` alone, Circulette's, sees 0 and prints no posterior.
    (define record-weight
      (let ([w #f])
        (λ ()
          (or w
              (begin
                (set! w (if (with-handlers ([exn:fail? (λ (_) #f)])
                              (module-declared? 'roulette/example/disrupt/core #f))
                            1
                            0))
                w)))))

    ;; The measure of the call `q`: a value's weight, once the value is
    ;; recorded as asked, is the record weight (0 for a value the call cannot
    ;; take), or with --tddlog tddlog's. A value the call cannot take is
    ;; recorded too, with the guard #f.
    (define (query-measure q)
      (define (density v)
        (define i (hash-ref! (query-index q) v
                             (λ ()
                               (set-query-others! q (cons (cons v #f) (query-others q)))
                               (hash-count (query-index q)))))
        (hash-set! (query-asked q) i #t)
        (define cmd (tddlog-command))
        (cond
          [cmd (hash-ref! (query-weights q) i (λ () (weight cmd q i)))]
          [(cdr (query-entry q i)) (record-weight)]
          [else 0]))
      (measure (λ (vs) (apply + (set-map vs density)))
               (λ () (for/set ([e (in-vector (query-entries q))]
                               #:unless (zero? (density (car e))))
                       (car e)))
               density
               (immutable-set/c any/c)
               number-semiring))

    (define recording-engine%
      (class* object% (engine<%>)
        (super-new)
        (define/public (domain)
          (immutable-set/c any/c))
        (define/public (infer val kept-vars)
          (unless (set-empty? kept-vars)
            (fail "infer's #:keep is not supported: a ProbLog program's answers are numbers"))
          (query-measure (record! val)))))

    ;; rsdd's `rsdd-engine`, for numbers' semiring alone.
    (define (rsdd-engine #:semiring [semiring number-semiring])
      (unless (eq? semiring number-semiring)
        (fail "rsdd-engine's #:semiring ~a is not supported: a ProbLog program's weights are probabilities (number-semiring)"
              semiring))
      (new recording-engine%))

    (define default-engine (new recording-engine%))
    (define unsupplied (string->uninterned-symbol "unsupplied"))

    ;; Roulette's `infer`, its default engine this file's.
    (define (infer val #:engine [engine default-engine] #:keep [keep unsupplied])
      (if (eq? keep unsupplied)
          (roulette-infer val #:engine engine)
          (roulette-infer val #:engine engine #:keep keep)))

    ;; Each random choice's probability, from the measure that defines it.
    (define (flip-probabilities)
      (define measures (make-hasheq))
      (for ([(x m _pc) (in-measures)])
        (hash-set! measures x m))
      (λ (c)
        (define m (hash-ref measures c (λ () (fail "a guard holds the symbolic constant ~a, which no measure defines" c))))
        (define density (measure-density m))
        (define f (density #f))
        (define t (density #t))
        (unless (and (real? f) (real? t) (<= 0 f) (<= 0 t) (< (abs (- (+ f t) 1)) 1e-9))
          (fail "the measure of ~a gives #f the weight ~s and #t ~s, which are not probabilities that sum to one"
                c f t))
        t))

    ;; The queries `items`, each (k assumes entries) with an entry (i value
    ;; guard), for `write-circuit`: each value's guard conjoined with the
    ;; assumptions, the conjuncts every one shares as the evidence, and each
    ;; value's other conjuncts its body, #f for a value never taken.
    (define (factor items)
      (define queries
        (for/list ([q (in-list items)])
          (match-define (list k assumes entries) q)
          (list k (for/list ([e (in-list entries)])
                    (match-define (list i v g) e)
                    (define c (@&& assumes g))
                    (list i v (and c (conjuncts c)))))))
      (define bodies (for*/list ([q (in-list queries)] [e (in-list (second q))] #:when (third e))
                       (third e)))
      (define shared
        (if (null? bodies)
            '()
            (for/fold ([s (car bodies)]) ([b (in-list (cdr bodies))])
              (filter (λ (x) (memq x b)) s))))
      (values (for/list ([q (in-list queries)])
                (list (first q)
                      (for/list ([e (in-list (second q))])
                        (match-define (list i v b) e)
                        (list i v (and b (filter (λ (x) (not (memq x shared))) b))))))
              shared))

    ;; The calls with a value asked, each with its asked values.
    (define (asked-items)
      (for/list ([q (in-list (reverse recorded))]
                 #:unless (hash-empty? (query-asked q)))
        (list (query-k q)
              (query-assumes q)
              (for/list ([e (in-sequences (query-entries q) (reverse (query-others q)))]
                         [i (in-naturals)]
                         #:when (hash-ref (query-asked q) i #f))
                (list i (car e) (cdr e))))))

    (define (write-program out)
      (define-values (queries evidence) (factor (asked-items)))
      (write-circuit out queries evidence (flip-probabilities)
                     #:notes (list WEIGHTS (format "infer was called ~a times; query K is its K-th call." calls))))

    ;; The weight of the i-th value of the call `q`, from tddlog (the
    ;; command `cmd`): P(e) times its answer, the value's guard the evidence;
    ;; 0 for a value the call cannot take.
    (define (weight cmd q i)
      (define e (query-entry q i))
      (cond
        [(not (cdr e)) 0]
        [else
         (define-values (queries evidence)
           (factor (list (list (query-k q) (query-assumes q) (list (list i (car e) (cdr e)))))))
         (define probability (flip-probabilities))
         (define-values (probs pe)
           (tddlog-answers cmd (query-k q)
                           (λ (out) (write-circuit out queries evidence probability #:notes (list WEIGHTS)))
                           #:evidence-probability? #t))
         (if probs
             (* (or pe (fail "tddlog gave no P(e) for query ~a" (query-k q)))
                (hash-ref probs i (λ () (fail "tddlog gave no answer for q~a(~a)" (query-k q) i))))
             0)])))

  ;; `roulette`, `roulette/safe` and `roulette/engine/rsdd` with the
  ;; exporter's engine.
  (module engine-roulette racket/base
    (require (except-in roulette infer)
             (only-in (submod ".." engine) infer))
    (provide (all-from-out roulette) infer))

  (module engine-safe racket/base
    (require (except-in roulette/safe infer)
             (only-in (submod ".." engine) infer))
    (provide (all-from-out roulette/safe) infer))

  (module engine-rsdd racket/base
    (require (except-in roulette/engine/rsdd rsdd-engine)
             (only-in (submod ".." engine) rsdd-engine))
    (provide (all-from-out roulette/engine/rsdd) rsdd-engine))])

(module+ main
  ;; The submodules that take the place of Roulette's are required here so
  ;; that they are declared (a compiled file's submodules are loaded on
  ;; demand) and are the instances the program's requires are sent to.
  (require racket/file
           racket/list
           racket/match
           racket/path
           racket/string
           compiler/compilation-path
           file/sha1
           syntax/modread
           (submod ".." circuit))
  (roulette-case
   [disrupt (require (submod ".." export)
                     (only-in (submod ".." language)))]
   [engine (require (submod ".." engine)
                    (only-in (submod ".." engine-roulette))
                    (only-in (submod ".." engine-safe))
                    (only-in (submod ".." engine-rsdd)))])

  (define USAGE "usage: racket roulette2problog.rkt [--make | --tddlog TDDLOG] PROGRAM.rkt [ARG ...]")

  (define-values (mode tddlog argv)
    (match (vector->list (current-command-line-arguments))
      [(list "--tddlog" cmd args ...) (values 'run cmd args)]
      [(list "--make" args ...) (values 'make #f args)]
      [args (values 'run #f args)]))
  (when (or (null? argv) (regexp-match? #rx"^-" (car argv)) (and (eq? mode 'make) (pair? (cdr argv))))
    (eprintf "~a\n" USAGE)
    (exit 2))

  (define program (path->complete-path (car argv)))
  (define program-mp `(file ,(path->string program)))

  (define self-file
    (let ([n (resolved-module-path-name (variable-reference->resolved-module-path (#%variable-reference)))])
      (if (pair? n) (car n) n)))
  ;; The script runs from its source. Compiled (`raco make` of this file),
  ;; its submodules' re-exports of Roulette's bindings resolve to the
  ;; submodules themselves, and a program fails to instantiate; compiled
  ;; by another Racket, it does not load.
  (let ([zo (get-compilation-bytecode-file self-file)])
    (when (file-exists? zo)
      (raise-user-error 'roulette2problog "~a is compiled (~a); it runs from its source: delete the compiled file"
                        (file-name-from-path self-file) zo)))

  ;; Roulette's modules that the program's requires are sent from, each to
  ;; this file's submodule that takes its place.
  (define original-resolver (current-module-name-resolver))
  (define self (variable-reference->module-path-index (#%variable-reference)))
  (define replacements
    (for/hash ([r (in-list REPLACED)])
      (values (original-resolver (car r) #f #f #f)
              (module-path-index-resolve (module-path-index-join `(submod ".." ,(cdr r)) self)))))
  (define originals (for/hash ([(from to) (in-hash replacements)]) (values to from)))
  ;; This file's modules and Roulette's own require Roulette's modules.
  (define (own? rel)
    (define n (and rel (resolved-module-path-name rel)))
    (define file (if (pair? n) (car n) n))
    (and (path? file)
         (or (equal? file self-file)
             (for/or ([d (in-list ROULETTE-DIRS)])
               (string-prefix? (path->string file) (path->string d))))))
  ;; A module compiled against Roulette's module names the modules that one
  ;; requires by relative paths ("../private/pmf.rkt"), resolved against
  ;; Roulette's file, not this one.
  (define redirecting-resolver
    (case-lambda
      [(r ns) (original-resolver r ns)]
      [(mp rel stx load?)
       (define base (if (string? mp) (hash-ref originals rel rel) rel))
       (define r (original-resolver mp base stx #f))
       (or (and (not (own? rel)) (hash-ref replacements r #f))
           (original-resolver mp base stx load?))]))

  ;; A module compiled against a replaced module refers to its variables by
  ;; the names its compilation gave them, which this file's need not have
  ;; (`with-observe`'s `evidence`): such a module (the program, a driver of
  ;; the artifact's) is loaded from source, against this file's. Its
  ;; compiled dependencies say which it is.
  (define marker-files
    (map (λ (m) (resolved-module-path-name (original-resolver m #f #f #f))) MARKERS))
  (define (dependency-file d)
    (match d
      [(cons 'indirect d) (dependency-file d)]
      [(list 'collects (? bytes? c) ... (? bytes? f))
       (with-handlers ([exn:fail? (λ (_) #f)])
         (apply collection-file-path (bytes->path f) (map bytes->path c)))]
      [(? bytes? p) (bytes->path p)]
      [_ #f]))
  (define (dep-file file) (path-replace-extension (get-compilation-bytecode-file file) #".dep"))
  (define (compiled-against-roulette? file)
    (define dep (dep-file file))
    (and (file-exists? dep)
         (let ([deps (call-with-input-file dep read)])
           (for/or ([d (in-list (if (and (list? deps) (> (length deps) 3)) (drop deps 3) '()))])
             (define f (dependency-file d))
             (and f (member (simplify-path f) marker-files))))))
  ;;
  ;; --make compiles each such module against this file's once, as `raco
  ;; make` compiles it against Roulette's, into compiled/roulette2problog/
  ;; beside its source, keyed by the SHA-1 of this file's source, the
  ;; module's, its .dep from `raco make` (which holds the SHA-1s of its
  ;; dependencies) and Racket's version; a run loads it from there while
  ;; the key holds, and compiles it from source otherwise.
  (define (cache-files file)
    (define-values (dir name _dir?) (split-path file))
    (define zo (build-path dir "compiled" "roulette2problog" (path-add-extension name #".zo")))
    (values zo (path-add-extension zo #".sha1")))
  (define (cache-key file)
    (sha1 (open-input-bytes (bytes-append (file->bytes self-file) #"\0" (file->bytes file) #"\0"
                                          (file->bytes (dep-file file)) #"\0" (string->bytes/utf-8 (version))))))
  ;; The program's modules that are compiled against this file's, and those
  ;; of them --make compiled; Roulette's own, compiled with them, are not
  ;; counted.
  (define found 0)
  (define made '())
  (define (module-code file)
    (define-values (zo key-file) (cache-files file))
    (define key (cache-key file))
    (define program-module? (not (member (simplify-path file) marker-files)))
    (when program-module? (set! found (add1 found)))
    (cond
      [(and (file-exists? zo) (file-exists? key-file) (equal? (file->string key-file) key))
       (parameterize ([read-accept-compiled #t]) (call-with-input-file zo read))]
      [else
       (define stx
         (with-module-reading-parameterization
           (λ () (call-with-input-file file (λ (in) (port-count-lines! in) (read-syntax file in))))))
       (define code (compile (check-module-form stx 'ignored file)))
       (when (eq? mode 'make)
         (make-directory* (path-only zo))
         ;; Renamed into place, the code before its key: a run that reads
         ;; the key reads the code it was written with.
         (call-with-atomic-output-file zo (λ (out _tmp) (write code out)))
         (call-with-atomic-output-file key-file (λ (out _tmp) (write-string key out)))
         (when program-module? (set! made (cons file made))))
       code]))
  (define original-load (current-load/use-compiled))
  (define (loading file name)
    (cond
      [(not (compiled-against-roulette? file)) (original-load file name)]
      ;; A submodule alone, without its enclosing module: only from compiled code.
      [(and (pair? name) (not (car name))) (void)]
      [else
       (define-values (dir _name _dir?) (split-path file))
       (parameterize ([current-load-relative-directory dir])
         (eval (module-code file)))]))

  (define stdout (current-output-port))
  (define t0 (current-inexact-milliseconds))
  (when (eq? mode 'make)
    (parameterize ([current-module-name-resolver redirecting-resolver]
                   [current-load/use-compiled loading])
      (dynamic-require program-mp (void)))
    (when (zero? found)
      (raise-user-error 'roulette2problog "~a has no module compiled against Roulette's language (raco make it first)"
                        (car argv)))
    (eprintf "roulette2problog: compiled ~a of ~a modules in this file's language~a (~a s)\n"
             (length made) found
             (string-append* (for/list ([f (in-list (reverse made))]) (format " ~a" (file-name-from-path f))))
             (seconds-since t0))
    (exit 0))
  ;; The modules the program requires (at phase 0), instantiated, so that
  ;; the clock starts at the program's own body: Racket's start and the
  ;; load of its modules are not the program's run.
  (define (instantiate-imports! mp)
    (define root (module-path-index-join mp #f))
    (define (rooted mpi)
      (define-values (name base) (module-path-index-split mpi))
      (cond
        [(not name) root]
        [(module-path-index? base) (module-path-index-join name (rooted base))]
        [else mpi]))
    (for* ([phase+imports (in-list (module->imports mp))]
           #:when (eqv? (car phase+imports) 0)
           [mpi (in-list (cdr phase+imports))])
      (dynamic-require (rooted mpi) #f)))
  (parameterize ([current-command-line-arguments (list->vector (cdr argv))]
                 [current-module-name-resolver redirecting-resolver]
                 [current-load/use-compiled loading]
                 [current-output-port (if tddlog stdout (current-error-port))]
                 [tddlog-command tddlog]
                 [program-label (string-join (cons (path->string (file-name-from-path program)) (cdr argv)) " ")])
    (unless (module-declared? program-mp #t)
      (raise-user-error 'roulette2problog "~a declares no module" (car argv)))
    (instantiate-imports! program-mp)
    ;; From here to the circuit, the program's run (`program ~a s`).
    (define t-run (current-inexact-milliseconds))
    (dynamic-require program-mp #f)
    (when (module-declared? `(submod ,program-mp main) #t)
      (dynamic-require `(submod ,program-mp main) #f))
    (unless tddlog
      (when (zero? (recorded-count))
        (raise-user-error 'roulette2problog "~a asks no query" (car argv)))
      (define ran (seconds-since t-run))
      (define t1 (current-inexact-milliseconds))
      ;; Written whole at the end, so that a pipe takes it in one piece.
      (define out (open-output-bytes))
      (define-values (nf ng no nq) (write-program out))
      (write-bytes (get-output-bytes out #t) stdout)
      (flush-output stdout)
      (eprintf "roulette2problog: ~a queries, ~a values, ~a flips, ~a gates, ~a observations; program ~a s, circuit ~a s\n"
               (recorded-count) nq nf ng no ran (seconds-since t1)))))
