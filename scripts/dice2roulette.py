#!/usr/bin/env python3
"""Translate a Dice program (Holtzen, Van den Broeck, Millstein, OOPSLA 2020)
into a program of Roulette's language (Moy, Czenszak, Li, Marshall and
Holtzen, PLDI 2025; `#lang roulette/example/disrupt`, the language of the
paper's benchmarks), which compiles a program's distribution to a BDD
(rsdd) in the order its flips are created.

The program is read by dice2problog.py's parser and written as Racket over
Roulette's values: a Boolean is a Racket Boolean (symbolic where it
depends on a flip), an int(w) a list of w Booleans, least significant bit
first, as the artifact's caesar and network programs hold their bit
vectors, and a tuple a list of two (Rosette merges lists of one length
element by element). `flip p` is Roulette's `flip`. `discrete` and
`uniform` draw their bits as Dice does (gen_discrete in Dice's
lib/Passes.ml, dice2problog.py's `choice`): from the most significant bit
down, bit k is a flip whose probability is the mass of the values with bit
k set among those that agree on the bits above it, over their mass, one
flip for each distinct ratio, chosen by the bits above. `&&`, `||` and `!`
are Rosette's; `<=>`, `==` and `!=` on Booleans are `<=>` and its
negation, `^` its negation. On integers, `+` is a ripple-carry adder as in
the artifact's `bvadd`, `-` adds the complement and one, `*` adds x
shifted by each bit of y, all modulo 2^w as Dice's; `==` compares bit by
bit, `<` (and `<=`, `>`, `>=`) is an unsigned comparator from bit 0 up,
`<<` and `>>` shift by a constant.

`let` binds in program order, so Roulette creates the flips, which order
its BDD's variables, in the order Dice does. `if c then a else b` is
Racket's `if`, which Roulette (through Rosette) evaluates on both branches
when c is symbolic, as Dice does; `observe e` is `(observe! e)`, which
conjoins e to the evidence on the path it is on. A Dice function is a
Racket function, `iterate(f, x, k)` a loop of k calls; a function that
calls itself takes its depth as a first argument and is cut at Dice's
recursion limit as in dice2problog.py: the call past it observes false and
returns its type's default value.

The program's value is queried with Roulette's `query`, one component of a
returned tuple at a time, as Alea.jl is asked them (dice2alea.py). The
program prints `roulette K VALUE PROBABILITY` for every value of the K-th
component (true and false, or 0 to 2^w - 1); with `--joint` a value of N
Booleans is one component, an int N whose most significant bit is the
first Boolean (the joint distribution a circuit's `joint` program asks).
Before them it prints the time of
running the program and the queries as Racket's `time` prints it (`cpu
time: ... real time: ... gc time: ...`), which leaves out Racket's start
and loading. The program's statements are the module's own forms rather
than one function's: Racket compiles a module too large for its compiler
as forms it interprets, where one function of a large network's
statements took minutes to compile. With the
argument `--no-complement` it queries with `(query e #f)`: Roulette's
default takes one value's probability as one minus the others', which
loses that value once the evidence's probability falls below about 1e-16
(data/roulette/README.md).

Supported: what dice2problog.py supports. Not supported: lists, /, %,
nth_bit, sample, functions that call each other in a cycle.

Usage: dice2roulette.py program.dice [--joint] > program.rkt
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from dice2problog import (Parser, RECURSION_LIMIT, bits_for, calls, deep,  # noqa: E402
                          discrete_probs, tokenize)

BOOL = ('bool',)
# A variable, a constant, or an expression that creates nothing: evaluating
# it again creates no flip and observes nothing.
TRIVIAL = re.compile(r"#t|#f|[a-z][a-z0-9_-]*|'\((#t|#f)( (#t|#f))*\)")


def literal(p):
    """A probability as Racket reads the same double."""
    text = repr(float(p))
    return text if any(c in text for c in '.e') else text + '.0'


def bits_of(k, w):
    return "'(" + ' '.join('#t' if k >> i & 1 else '#f' for i in range(w)) + ')'


def same(a, b, what):
    if a != b:
        raise TypeError(f'{what}: {show(a)} and {show(b)}')
    return a


def show(t):
    if t[0] == 'bool':
        return 'bool'
    if t[0] == 'int':
        return f'int({t[1]})'
    return f'({show(t[1])}, {show(t[2])})'


def block(statements, value):
    """Statements and a value as one Racket expression."""
    if not statements:
        return value
    return '(let ()\n  ' + '\n  '.join(statements) + f'\n  {value})'


class Translator:
    def __init__(self):
        self.n = 0
        self.defs = []       # the Racket definitions, in order
        self.funs = {}       # Dice name: (Racket name, parameter types, return type, recursive)
        self.current = None  # the recursive function being translated

    def fresh(self, hint):
        self.n += 1
        return f'x{self.n}-{re.sub(r"[^a-z0-9]", "-", hint.lower())[:16]}'

    def value(self, e, env, out, hint):
        """e's value as a variable or a constant, its statements in out."""
        code, t = self.expr(e, env, out, hint)
        if TRIVIAL.fullmatch(code):
            return code, t
        return self.make(code, out, hint), t

    def make(self, code, out, hint):
        """A statement for an expression that creates flips or observes: it
        runs at this point of the program, whatever reads it."""
        v = self.fresh(hint)
        out.append(f'(define {v} {code})')
        return v

    def expr(self, e, env, out, hint):
        """Racket for expression e under env (Dice names to Racket
        variables and constants, with their types), and its type; its
        statements are appended to out. An expression returned inline
        creates no flip and observes nothing."""
        if e[0] == 'let':
            env = dict(env)
            while e[0] == 'let':
                env[e[1]] = self.value(e[2], env, out, e[1])
                e = e[3]
        kind = e[0]
        if kind == 'var':
            if e[1] not in env:
                raise NameError(f'unbound variable {e[1]}')
            return env[e[1]]
        if kind == 'bool':
            return ('#t' if e[1] else '#f'), BOOL
        if kind == 'flip':
            p = e[1]
            if p <= 0:
                return '#f', BOOL
            if p >= 1:
                return '#t', BOOL
            return self.make(f'(flip {literal(p)})', out, hint), BOOL
        if kind == 'discrete':
            ps = discrete_probs(e[1])
            w = bits_for(len(ps))
            return self.make(f'(discrete {w} (list {" ".join(literal(p) for p in ps)}))', out, hint), ('int', w)
        if kind == 'uniform':
            width, lo, hi = e[1:]
            if not 0 <= lo < hi <= 2 ** width:
                raise ValueError(f'uniform({width}, {lo}, {hi}): empty or out of range')
            return self.make(f'(uniform {width} {lo} {hi})', out, hint), ('int', width)
        if kind == 'int':
            width, k = e[1:]
            if not 0 <= k < 2 ** width:
                raise ValueError(f'integer constant {k} out of range for {2 ** width} values')
            return bits_of(k, width), ('int', width)
        if kind == 'tuple':
            a, ta = self.expr(e[1], env, out, hint)
            b, tb = self.expr(e[2], env, out, hint)
            return f'(list {a} {b})', ('tuple', ta, tb)
        if kind in ('fst', 'snd'):
            c, t = self.expr(e[1], env, out, hint)
            if t[0] != 'tuple':
                raise TypeError(f'{kind} of {show(t)}')
            return (f'(car {c})', t[1]) if kind == 'fst' else (f'(cadr {c})', t[2])
        if kind == 'not':
            c, t = self.expr(e[1], env, out, hint)
            same(t, BOOL, '!')
            return {'#t': '#f', '#f': '#t'}.get(c, f'(! {c})'), BOOL
        if kind == 'shift':
            c, t = self.expr(e[2], env, out, hint)
            if t[0] != 'int':
                raise TypeError(f'{e[1]} of {show(t)}')
            return f'({"bv-shl" if e[1] == "<<" else "bv-shr"} {c} {e[3]})', t
        if kind == 'bin':
            a, ta = self.expr(e[2], env, out, hint)
            b, tb = self.expr(e[3], env, out, hint)
            return self.binary(e[1], a, ta, b, tb)
        if kind == 'observe':
            c, t = self.expr(e[1], env, out, hint)
            same(t, BOOL, 'observe')
            out.append(f'(observe! {c})')
            return '#t', BOOL
        if kind == 'if':
            return self.ite(e, env, out, hint)
        if kind == 'call':
            args = [self.expr(x, env, out, f'{e[1]}-arg') for x in e[2]]
            return self.call(e[1], args, out, hint)
        if kind == 'iterate':
            init, t = self.value(e[2], env, out, hint)
            name, params, ret = self.signature(e[1])
            if len(params) != 1:
                raise TypeError(f'iterate: {e[1]} takes {len(params)} arguments')
            same(params[0], t, f'iterate({e[1]})')
            same(ret, t, f'iterate({e[1]})')
            s = self.fresh('state')
            call = self.apply(e[1], [s])
            return self.make(f'(for/fold ([{s} {init}]) ([_ (in-range {e[3]})]) {call})', out, hint), t
        raise ValueError(f'unknown expression {kind}')

    def binary(self, op, a, ta, b, tb):
        if op in ('/', '%', '::'):
            raise NotImplementedError(f'operator {op} is not supported')
        if op in ('&&', '||'):
            same(ta, BOOL, op)
            same(tb, BOOL, op)
            return f'({op} {a} {b})', BOOL
        if ta == BOOL and op in ('^', '<=>', '==', '!='):
            # Dice reads == and != on integers only, the translators on
            # Booleans too.
            same(tb, BOOL, op)
            iff = f'(<=> {a} {b})'
            return (f'(! {iff})' if op in ('^', '!=') else iff), BOOL
        if ta[0] != 'int':
            raise TypeError(f'{op} on {show(ta)}')
        same(ta, tb, op)
        if op in ('+', '-', '*'):
            return f'({ {"+": "bv-add", "-": "bv-sub", "*": "bv-mul"}[op] } {a} {b})', ta
        if op in ('==', '!='):
            eq = f'(bv-eq {a} {b})'
            return (eq if op == '==' else f'(! {eq})'), BOOL
        if op == '<':
            return f'(bv-ult {a} {b})', BOOL
        if op == '>':
            return f'(bv-ult {b} {a})', BOOL
        if op == '<=':
            return f'(! (bv-ult {b} {a}))', BOOL
        if op == '>=':
            return f'(! (bv-ult {a} {b}))', BOOL
        raise ValueError(f'unknown operator {op}')

    def ite(self, e, env, out, hint):
        c, t = self.value(e[1], env, out, hint)
        same(t, BOOL, 'if')
        if c == '#t':
            return self.expr(e[2], env, out, hint)
        if c == '#f':
            return self.expr(e[3], env, out, hint)
        then, other = [], []
        a, ta = self.value(e[2], env, then, hint)
        b, tb = self.value(e[3], env, other, hint)
        t = same(ta, tb, 'if')
        code = f'(if {c} {block(then, a)} {block(other, b)})'
        if not then and not other:
            return code, t
        return self.make(code, out, hint), t

    def signature(self, fname):
        if fname not in self.funs:
            raise NameError(f'unknown function {fname}')
        return self.funs[fname][:3]

    def apply(self, fname, args):
        """The call of fname on argument codes: a function that calls
        itself takes its depth first, 0 from outside it."""
        name, _, _, recursive = self.funs[fname]
        if recursive:
            depth = '(add1 depth)' if self.current == fname else '0'
            args = [depth] + args
        return f'({" ".join([name] + args)})'

    def call(self, fname, args, out, hint):
        name, params, ret = self.signature(fname)
        if len(params) != len(args):
            raise TypeError(f'{fname}: {len(params)} parameters, {len(args)} arguments')
        for p, (_, t) in zip(params, args):
            same(p, t, f'{fname}')
        return self.make(self.apply(fname, [c for c, _ in args]), out, hint), ret

    def default(self, t, fname):
        if t is None:
            raise TypeError(f'{fname}: a recursive function needs a return type')
        if t[0] == 'bool':
            return '#f'
        if t[0] == 'int':
            return bits_of(0, t[1])
        return f'(list {self.default(t[1], fname)} {self.default(t[2], fname)})'

    def function(self, fname, params, types, ret, body, recursive):
        if None in types:
            raise TypeError(f'{fname}: a parameter without a type')
        name = 'f-' + re.sub(r'[^a-z0-9]', '-', fname.lower())
        while any(name == f[0] for f in self.funs.values()):
            name += '-'
        # A recursive function's return type is its declared one; the
        # others' is their body's.
        self.funs[fname] = (name, types, ret, recursive)
        args = [self.fresh(p) for p in params]
        out = []
        self.current = fname if recursive else None
        value, t = self.value(body, {p: (a, ty) for p, a, ty in zip(params, args, types)}, out, fname)
        self.current = None
        if ret is not None:
            same(ret, t, f'{fname} returns')
        self.funs[fname] = (name, types, t, recursive)
        if not recursive:
            self.defs.append(f'(define ({" ".join([name] + args)})\n  ' + '\n  '.join(out + [value]) + ')')
            return
        self.defs.append(
            f'(define ({" ".join([name, "depth"] + args)})\n'
            f'  (cond\n'
            f'    [(> depth {RECURSION_LIMIT}) (observe! #f) {self.default(ret, fname)}]\n'
            f'    [else\n     ' + '\n     '.join(out + [value]) + ']))')

    def components(self, t, code):
        """The leaves of a value of type t: (code, type) each."""
        if t[0] == 'tuple':
            return self.components(t[1], f'(car {code})') + self.components(t[2], f'(cadr {code})')
        return [(code, t)]

    def translate(self, text, source, joint=False):
        parser = Parser(tokenize(text))
        funs, body = parser.program()
        names = [f[0] for f in funs]
        for i, (fname, params, ret, fbody) in enumerate(funs):
            # A function calls only those defined before it, and itself.
            for other in names[i + 1:]:
                if calls(fbody, other):
                    raise NotImplementedError(f'{fname} calls {other}, defined after it')
            self.function(fname, params, parser.param_types[fname], ret, fbody, calls(fbody, fname))
        # The program's own statements are the module's, in order, between
        # the clock's start and the queries: Racket compiles a module too
        # large for its compiler (PLT_CS_COMPILE_LIMIT) as forms it
        # interprets, where one function of them all took minutes.
        out = []
        value, t = self.value(body, {}, out, 'result')
        comps = self.components(t, value)
        if joint:   # the Booleans as one int, least significant bit first
            if any(ct != BOOL for _, ct in comps):
                raise NotImplementedError("--joint asks a value of Booleans, and the value is " + show(t))
            comps = [(f"(list {' '.join(c for c, _ in reversed(comps))})", ("int", len(comps)))]
        queries = ' '.join(f'(query {c} complement?)' for c, _ in comps)
        reports = []
        for k, (_, ct) in enumerate(comps, 1):
            reports.append(f'(report-bool {k} (list-ref pmfs {k - 1}))' if ct == BOOL else
                           f'(report-int {k} (list-ref pmfs {k - 1}) {ct[1]})')
        main = ['(define clock (start-clock))'] + out + [f'(define pmfs (list {queries}))', '(print-time clock)']
        return PRELUDE.format(source=source) + '\n\n'.join(self.defs) + '\n\n' + '\n'.join(main + reports) + '\n'


PRELUDE = '''\
#lang roulette/example/disrupt
;; {source}, translated by tddlog's scripts/dice2roulette.py.

(define complement?
  (not (member "--no-complement" (vector->list (current-command-line-arguments)))))

;; Integers: lists of Booleans, least significant bit first.
(define (bxor a b) (! (<=> a b)))
(define (bv-add xs ys [c #f])
  (if (null? xs)
      '()
      (let* ([x (car xs)] [y (car ys)] [s (bxor x y)])
        (cons (bxor s c) (bv-add (cdr xs) (cdr ys) (|| (&& x y) (&& c s)))))))
(define (bv-sub xs ys) (bv-add xs (map ! ys) #t))
(define (bv-mul xs ys)
  (define w (length xs))
  (for/fold ([acc (make-list w #f)]) ([y (in-list ys)] [i (in-naturals)])
    (if (eq? y #f)
        acc
        (bv-add acc (append (make-list i #f) (for/list ([x (in-list (take xs (- w i)))]) (&& y x)))))))
(define (bv-eq xs ys) (apply && (map <=> xs ys)))
(define (bv-ult xs ys)
  (for/fold ([lt #f]) ([x (in-list xs)] [y (in-list ys)])
    (|| (&& (! x) y) (&& (<=> x y) lt))))
(define (bv-shl xs k)
  (define w (length xs))
  (take (append (make-list (min k w) #f) xs) w))
(define (bv-shr xs k)
  (define j (min k (length xs)))
  (append (drop xs j) (make-list j #f)))

;; Dice's categorical (gen_discrete): from the most significant bit down,
;; bit k is a flip whose probability is the mass of the values with bit k
;; set among those that agree on the bits above, over their mass; one flip
;; for each distinct ratio, chosen by the bits above, settings of no mass
;; left out.
(define (discrete w ps)
  (define size (expt 2 w))
  (define given (list->vector ps))
  (define masses
    (for/vector ([i (in-range size)])
      (if (< i (vector-length given)) (inexact->exact (vector-ref given i)) 0)))
  (define (mass lo hi) (for/sum ([i (in-range lo hi)]) (vector-ref masses i)))
  (let level ([k (sub1 w)] [above '()])
    (cond
      [(< k 0) (reverse above)]
      [else
       (define span (expt 2 (add1 k)))
       (define flips (make-hash))
       (define table
         (for/vector ([u (in-range (quotient size span))])
           (define lo (* u span))
           (define total (mass lo (+ lo span)))
           (cond
             [(zero? total) 'none]
             [else
              (define r (exact->inexact (/ (mass (+ lo (quotient span 2)) (+ lo span)) total)))
              (hash-ref! flips r (λ () (cond [(= r 0) #f] [(= r 1) #t] [else (flip r)])))])))
       (level (sub1 k) (append above (list (choose above table))))])))
;; The entry of table for the setting of the bits `above` (most significant
;; first), constant where every setting of mass agrees.
(define (choose above table)
  (define vals (for/list ([x (in-vector table)] #:unless (eq? x 'none)) x))
  (cond
    [(null? vals) #f]
    [(andmap (λ (x) (eq? x (car vals))) vals) (car vals)]
    [else
     (define half (quotient (vector-length table) 2))
     (if (car above)
         (choose (cdr above) (vector-copy table half))
         (choose (cdr above) (vector-copy table 0 half)))]))
(define (uniform w lo hi)
  (discrete w (for/list ([i (in-range hi)]) (if (< i lo) 0.0 (exact->inexact (/ 1 (- hi lo)))))))

(define (bits->natural bs)
  (for/sum ([b (in-list bs)] [i (in-naturals)]) (if b (expt 2 i) 0)))
(define (real p) (if (exact? p) (exact->inexact p) p))
;; Racket's `time` around the program's statements and its queries.
(define (start-clock)
  (vector (current-process-milliseconds) (current-inexact-milliseconds) (current-gc-milliseconds)))
(define (print-time c)
  (printf "cpu time: ~a real time: ~a gc time: ~a\\n"
          (- (current-process-milliseconds) (vector-ref c 0))
          (inexact->exact (round (- (current-inexact-milliseconds) (vector-ref c 1))))
          (- (current-gc-milliseconds) (vector-ref c 2))))
(define (report-bool k pm)
  (printf "roulette ~a true ~a\\n" k (real (pm #t)))
  (printf "roulette ~a false ~a\\n" k (real (pm #f))))
(define (report-int k pm w)
  (define by-value (make-hash))
  (for ([(v p) (in-pmf pm)])
    (hash-set! by-value (bits->natural v) p))
  (for ([i (in-range (expt 2 w))])
    (printf "roulette ~a ~a ~a\\n" k i (real (hash-ref by-value i 0)))))

'''


def translate(text, source='a Dice program', joint=False):
    """The Roulette program for a Dice program; `source` names it in the
    program's second line. With `joint` the value, a tuple of Booleans, is
    asked as one int whose bits they are, the most significant first."""
    return deep(lambda: Translator().translate(text, source, joint))


if __name__ == '__main__':
    argv = [a for a in sys.argv[1:] if a != '--joint']
    if len(argv) != 1:
        sys.exit(__doc__)
    sys.stdout.write(translate(open(argv[0]).read(), os.path.basename(argv[0]), '--joint' in sys.argv[1:]))
