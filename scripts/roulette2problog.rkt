#lang racket/base
;; roulette2problog.rkt: the Boolean circuit Roulette builds for a program,
;; written as a ProbLog program that tddlog reads.
;;
;;     racket roulette2problog.rkt PROGRAM.rkt [ARG ...] > PROGRAM.pl
;;     racket roulette2problog.rkt --tddlog TDDLOG PROGRAM.rkt [ARG ...]
;;     racket roulette2problog.rkt --make PROGRAM.rkt
;;
;; Roulette (Moy, Czenszak, Li, Marshall and Holtzen, "Roulette: A Language
;; for Expressive, Exact, and Efficient Discrete Probabilistic Programming",
;; PLDI 2025) evaluates a Racket program symbolically with Rosette: every
;; `flip` is a symbolic Boolean constant, and every Boolean the program
;; computes from flips is a Rosette term over them, a DAG of !, &&, ||, =>
;; and <=>. Its `query` flattens the program's value into the values it can
;; take, each with the Boolean term (its guard) under which it does, and
;; compiles each guard, conjoined with the observations, to a BDD with rsdd,
;; its variables the flips in the order they were created. The script was
;; written against Roulette 54d6185 (the PLDI 2025 artifact) on Racket 8.16
;; with Rosette 4.1, run with Roulette's packages installed (PLTUSERHOME and
;; PLTADDONDIR naming the installation, as its README says).
;;
;; The script runs PROGRAM.rkt, with ARG ... as its command line, in
;; Roulette's language `#lang roulette/example/disrupt` (the language of the
;; paper's programs), with the language's module
;; `roulette/example/disrupt/rsdd` replaced by this file's submodule
;; `language`, which is the same language whose `query` hands its value and
;; the observations to this script instead of rsdd. A module compiled
;; against Roulette's language (by `raco make`) is compiled again in this
;; one; the others, the artifact's drivers among them, run compiled. rsdd is
;; never loaded. --make does that compilation ahead of the runs, as `raco
;; make` does Roulette's: it compiles PROGRAM.rkt's modules, and Roulette's
;; language module with them, into compiled/roulette2problog/ beside their
;; sources, and a run loads them from there while this file and their
;; sources are unchanged; without it, each run compiles them anew. It
;; refuses a program none of whose modules `raco make` has compiled
;; against Roulette's language.
;;
;; Without --tddlog, each query's guards are recorded and the query returns
;; an empty distribution, the program's own output goes to standard error,
;; and once the program has run the script writes one ProbLog program for
;; all its queries to standard output: a flip is a probabilistic fact
;; `P::fN.` (the N-th flip of the circuit in the order Roulette created
;; them), an && a rule, an || a rule for each operand, a ! a negated
;; literal, a <=> two rules and a => two rules; the observations are
;; `evidence(...)`, one for each conjunct of their conjunction. The K-th
;; query of the program is asked as `qK(I)` for its I-th value, in the order
;; of `datum-order`, the value written in a comment beside it. The rules come
;; in the order Rosette created the terms, every gate after the gates it
;; reads. tddlog's answer for qK(I) is the probability of the value given the
;; observations, which Roulette's query computes (without its
;; `#:complement-of` shortcut, which takes one value's probability as one
;; minus the others'). The queries must be asked under the same
;; observations, which one ProbLog program holds.
;;
;; With --tddlog TDDLOG (a command, split at spaces), each query is written
;; on its own to a temporary file and answered by TDDLOG on it, and the query
;; returns tddlog's distribution: Roulette with tddlog in place of rsdd. The
;; program's output is its own; each query's sizes and the seconds of
;; writing its circuit and of the tddlog run go to standard error.
;;
;; `define-compile`, Roulette's staged procedures, compiles to BDDs before
;; any query and is refused, as are flips whose probability is symbolic and
;; measures that are not probabilities.
;;
;; The language submodule follows Roulette's example/disrupt/rsdd.rkt
;; (Apache License 2.0), its `query` replaced.

(module export racket/base
  (require racket/list
           racket/match
           racket/port
           racket/string
           racket/system
           racket/file
           data/ddict
           data/order
           rosette/base/core/term
           (only-in rosette/base/core/bool @! @&& @|| @=> @<=>)
           roulette/private/measure
           (only-in roulette/private/util flatten-symbolic)
           roulette/example/private/pmf)

  (provide tddlog-command
           program-label
           export-query
           recorded-queries
           write-circuit)

  ;; The command that answers each query, or #f to record the queries.
  (define tddlog-command (make-parameter #f))
  ;; The program's name and arguments, for the circuit's first line.
  (define program-label (make-parameter "a Roulette program"))

  (define (fail fmt . args)
    (raise-user-error 'roulette2problog (apply format fmt args)))

  ;; The values of `e` with their guards, in datum order: a list of (value
  ;; . guard), a guard #t, #f or a Boolean term.
  (define (value-guards e)
    (define entries (hash->list (flatten-symbolic e)))
    (with-handlers ([exn:fail? (λ (_) (sort entries string<? #:key (λ (x) (format "~s" (car x)))))])
      (sort entries (order-<? datum-order) #:key car)))

  ;; The queries recorded so far, the first first: (k entries evidence).
  (define recorded '())
  (define (recorded-queries) (reverse recorded))

  ;; Roulette's `query` of `e` under the observations `evidence`.
  (define (export-query e evidence)
    (define k (add1 (length recorded)))
    (define entries (value-guards e))
    (set! recorded (cons (list k entries evidence) recorded))
    (define cmd (tddlog-command))
    (if cmd (answer cmd k entries evidence) (pmf (hash))))

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

  ;; A flip's probability as the double rsdd is given.
  (define (prob-text c)
    (define m (ddict-ref measures c #f))
    (unless m
      (fail "a guard holds the symbolic constant ~a, which is no flip (define-compile is not supported)" c))
    (define p (measure-value m))
    (unless (and (real? p) (< 0 p 1))
      (fail "the flip ~a has the measure ~s, not a probability strictly between 0 and 1" c p))
    (number->string (exact->inexact p)))

  (define (negated? n) (match n [(expression (== @!) _) #t] [_ #f]))

  ;; The ProbLog program of `queries`, a list of (k entries evidence) that
  ;; share their evidence, to `out`.
  (define (write-circuit out queries)
    (define evidence (third (first queries)))
    (for ([q (in-list (cdr queries))])
      (unless (eq? (third q) evidence)
        (fail "query ~a is asked under other observations than query 1, and one ProbLog program holds one set of evidence"
              (first q))))
    (define obs (conjuncts evidence))
    (define guards (for*/list ([q (in-list queries)] [e (in-list (second q))]) (cdr e)))
    (define nodes (circuit-nodes (append guards obs)))
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
             (length flips) (length gates) (length obs) n-queries)
    (for ([f (in-list flips)])
      (fprintf out "~a::~a.\n" (prob-text f) (hash-ref names f)))
    (for ([g (in-list gates)])
      (define head (hash-ref names g))
      (match g
        [(expression (== @&&) xs ...) (rule head (map lit xs))]
        [(expression (== @||) xs ...) (apply rule head (for/list ([x (in-list xs)]) (list (lit x))))]
        [(expression (== @<=>) x y) (rule head (list (lit x) (lit y)) (list (lit x #f) (lit y #f)))]
        [(expression (== @=>) x y) (rule head (list (lit x #f)) (list (lit y)))]))
    (for ([o (in-list obs)])
      (match o
        [#f (fprintf out "evidence(false_observation, true).\n")]
        [(expression (== @!) x) (fprintf out "evidence(~a, false).\n" (lit x))]
        [_ (fprintf out "evidence(~a, true).\n" (lit o))]))
    (for* ([q (in-list queries)])
      (for ([e (in-list (second q))] [i (in-naturals)])
        (define atom (format "q~a(~a)" (first q) i))
        (fprintf out "% ~a: ~s\n" atom (car e))
        (match (cdr e)
          [#t (fprintf out "~a.\n" atom)]
          [#f (fprintf out "~a :- fail.\n" atom)]
          [g (fprintf out "~a :- ~a.\n" atom (lit g))])
        (fprintf out "query(~a).\n" atom)))
    (values (length flips) (length gates) (length obs) n-queries))

  (define (seconds-since t) (/ (- (current-inexact-milliseconds) t) 1000.0))

  (define ANSWER #px"^q([0-9]+)\\(([0-9]+)\\):\t(.*)$")

  ;; The distribution of the k-th query, of values and guards `entries`
  ;; under `evidence`, as tddlog (the command `cmd`) answers its circuit.
  (define (answer cmd k entries evidence)
    (define file (make-temporary-file "roulette2problog~a.pl"))
    (define t0 (current-inexact-milliseconds))
    (define-values (nf ng no nq)
      (call-with-output-file file #:exists 'truncate
        (λ (out) (write-circuit out (list (list k entries evidence))))))
    (define written (seconds-since t0))
    (define words (string-split cmd))
    (define exe (or (find-executable-path (car words)) (fail "no command ~a" (car words))))
    (define t1 (current-inexact-milliseconds))
    (define text
      (with-output-to-string
        (λ ()
          (unless (apply system* exe (append (cdr words) (list "--precision" "17" (path->string file))))
            (fail "~a failed on query ~a (~a)" cmd k file)))))
    (define ran (seconds-since t1))
    (delete-file file)
    (eprintf "roulette2problog: query ~a: ~a values, ~a flips, ~a gates, ~a observations; circuit ~a s, tddlog ~a s\n"
             k nq nf ng no written ran)
    (cond
      [(regexp-match? #rx"^evidence is inconsistent" text) (pmf (hash))]
      [else
       (define probs (make-hash))
       (for ([line (in-list (string-split text "\n"))])
         (match (regexp-match ANSWER line)
           [(list _ kk i p) #:when (equal? (string->number kk) k)
            (hash-set! probs (string->number i) (string->number p))]
           [_ (void)]))
       (pmf (for/hash ([e (in-list entries)] [i (in-naturals)])
              (values (car e)
                      (hash-ref probs i (λ () (fail "tddlog gave no answer for q~a(~a)" k i))))))])))

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
        (set! evidence old)))))

(module+ main
  ;; The language submodule is required here so that it is declared (a
  ;; compiled file's submodules are loaded on demand) and is the instance
  ;; the program's requires of Roulette's language are sent to.
  (require racket/file
           racket/list
           racket/match
           racket/path
           racket/string
           compiler/compilation-path
           file/sha1
           syntax/modread
           (submod ".." export)
           (only-in (submod ".." language)))

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

  ;; Roulette's language module, and this file's that takes its place.
  (define original-resolver (current-module-name-resolver))
  (define rsdd-name (original-resolver 'roulette/example/disrupt/rsdd #f #f #f))
  (define language-name
    (module-path-index-resolve
     (module-path-index-join '(submod ".." language) (variable-reference->module-path-index (#%variable-reference)))))
  ;; A module compiled against Roulette's module names the modules that one
  ;; requires by relative paths ("../private/pmf.rkt"), resolved against
  ;; Roulette's file, not this one.
  (define redirecting-resolver
    (case-lambda
      [(r ns) (original-resolver r ns)]
      [(mp rel stx load?)
       (define base (if (and (string? mp) (equal? rel language-name)) rsdd-name rel))
       (define r (original-resolver mp base stx #f))
       (if (equal? r rsdd-name) language-name (original-resolver mp base stx load?))]))

  ;; A module compiled against Roulette's language refers to its variables
  ;; by the names its compilation gave them, which this file's need not
  ;; have (`with-observe`'s `evidence`): such a module (the program, a
  ;; driver of the artifact's) is loaded from source, in this file's
  ;; language. Its compiled dependencies say which it is.
  (define disrupt-files
    (map (λ (m) (resolved-module-path-name (original-resolver m #f #f #f)))
         '(roulette/example/disrupt roulette/example/disrupt/rsdd)))
  (define (dependency-file d)
    (match d
      [(cons 'indirect d) (dependency-file d)]
      [(list 'collects (? bytes? c) ... (? bytes? f))
       (with-handlers ([exn:fail? (λ (_) #f)])
         (apply collection-file-path (bytes->path f) (map bytes->path c)))]
      [(? bytes? p) (bytes->path p)]
      [_ #f]))
  (define (dep-file file) (path-replace-extension (get-compilation-bytecode-file file) #".dep"))
  (define (compiled-against-disrupt? file)
    (define dep (dep-file file))
    (and (file-exists? dep)
         (let ([deps (call-with-input-file dep read)])
           (for/or ([d (in-list (if (and (list? deps) (> (length deps) 3)) (drop deps 3) '()))])
             (define f (dependency-file d))
             (and f (member (simplify-path f) disrupt-files))))))
  ;;
  ;; --make compiles each such module in this file's language once, as
  ;; `raco make` compiles it in Roulette's, into compiled/roulette2problog/
  ;; beside its source, keyed by the SHA-1 of this file's source, the
  ;; module's, its .dep from `raco make` (which holds the SHA-1s of its
  ;; dependencies) and Racket's version; a run loads it from there while
  ;; the key holds, and compiles it from source otherwise.
  (define self-file
    (let ([n (resolved-module-path-name (variable-reference->resolved-module-path (#%variable-reference)))])
      (if (pair? n) (car n) n)))
  (define (cache-files file)
    (define-values (dir name _dir?) (split-path file))
    (define zo (build-path dir "compiled" "roulette2problog" (path-add-extension name #".zo")))
    (values zo (path-add-extension zo #".sha1")))
  (define (cache-key file)
    (sha1 (open-input-bytes (bytes-append (file->bytes self-file) #"\0" (file->bytes file) #"\0"
                                          (file->bytes (dep-file file)) #"\0" (string->bytes/utf-8 (version))))))
  ;; The program's modules that are compiled in this file's language, and
  ;; those of them --make compiled; Roulette's language module, compiled
  ;; with them, is not counted.
  (define found 0)
  (define made '())
  (define (module-code file)
    (define-values (zo key-file) (cache-files file))
    (define key (cache-key file))
    (define program-module? (not (member (simplify-path file) disrupt-files)))
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
      [(not (compiled-against-disrupt? file)) (original-load file name)]
      ;; A submodule alone, without its enclosing module: only from compiled code.
      [(and (pair? name) (not (car name))) (void)]
      [else
       (define-values (dir _name _dir?) (split-path file))
       (parameterize ([current-load-relative-directory dir])
         (eval (module-code file)))]))

  (define stdout (current-output-port))
  (define (seconds-since t) (/ (- (current-inexact-milliseconds) t) 1000.0))
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
  (parameterize ([current-command-line-arguments (list->vector (cdr argv))]
                 [current-module-name-resolver redirecting-resolver]
                 [current-load/use-compiled loading]
                 [current-output-port (if tddlog stdout (current-error-port))]
                 [tddlog-command tddlog]
                 [program-label (string-join (cons (path->string (file-name-from-path program)) (cdr argv)) " ")])
    (dynamic-require program-mp #f)
    (when (module-declared? `(submod ,program-mp main) #t)
      (dynamic-require `(submod ,program-mp main) #f))
    (unless tddlog
      (define queries (recorded-queries))
      (when (null? queries)
        (raise-user-error 'roulette2problog "~a asks no query" (car argv)))
      (define ran (seconds-since t0))
      (define t1 (current-inexact-milliseconds))
      ;; Written whole at the end, so that a pipe takes it in one piece.
      (define out (open-output-bytes))
      (define-values (nf ng no nq) (write-circuit out queries))
      (write-bytes (get-output-bytes out #t) stdout)
      (flush-output stdout)
      (eprintf "roulette2problog: ~a queries, ~a values, ~a flips, ~a gates, ~a observations; program ~a s, circuit ~a s\n"
               (length queries) nq nf ng no ran (seconds-since t1)))))
