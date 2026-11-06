#!/usr/bin/env python3
"""Translate a ProbLog program of the Datalog fragment into PSouffle's
probabilistic Datalog.

usage: problog2souffle.py PROGRAM OUTDIR      (PROGRAM gzipped or not)

PSouffle (Li, Xia, Adnan, Wang, "PSouffle: Scaling Exact Probabilistic
Logic Inference for Program Analysis", FMCAD 2026;
github.com/psouffle-lang/psouffle) compiles a Souffle program whose rules
may carry a probability (`0.7::path(x,y) :- edge(x,y).`), reads each input
relation R from R.facts, a tuple a line, and its probabilities from R.prob,
line by line, and writes the probability of every tuple its `query(...)`
lines ask to facts.prob. This writes OUTDIR/compute.dl, OUTDIR/input/ and
OUTDIR/names.tsv (each Souffle relation, the ProbLog predicate it is and
its arity) from a ProbLog program:

- a ground fact, probabilistic or not, is a row of an input relation; a
  tuple given twice is one row of probability 1 - (1 - p)(1 - q), as
  ProbLog's two independent facts are; a predicate with facts and rules
  reads its facts from an input relation of its own, R__f, through the rule
  R(x, ..) :- R__f(x, ..);
- a rule keeps its probability, which PSouffle draws once per ground rule,
  as ProbLog does; `\\+ A` and `not(A)` are `!A`, the comparisons and `is`
  their Souffle forms, and a rule whose body holds `fail` is left out;
- between(L, H, X), ProbLog's built-in where the program does not define
  between/3, is `L <= X, X <= H` where another literal binds X, and
  otherwise an atom of the input relation between, whose rows (L, H, X)
  are those of the values L and H take, integers or arguments of atoms of
  certain relations;
- an annotated disjunction p1::h1; ..; pn::hn :- B chooses its head in
  order: the choice c_k, of probability p_k / (1 - p1 - .. - p(k-1)), is
  drawn once per ground clause, and h_k holds where B, c_k and none of
  c_1 .. c(k-1) do (the last choice is certain where the p's sum to 1).
  Each annotated disjunction of facts is a row of the input relations of
  its shape, ad<j>_c (its choices) and ad<j>_a<k> (its heads); one with a
  body has the rules q_k::ad<j>_c(k, V) :- B over the clause's variables V;
- `query(A)` and `evidence(A, true|false)` are PSouffle's own, one of each
  a relation (PSouffle names their code after it): a second ground atom is
  asked through a relation of no arguments, q__N() :- A; a query with a
  body over certain relations is its ground queries. OUTDIR/queries.txt lists
  the queries as tddlog prints their atoms.

An argument position holds Souffle numbers where every constant that
reaches it, through the variables the clauses share, is an integer, and
symbols otherwise. A program that PSouffle cannot be given exits with
status 3 (bench.py's REFUSED) and the reason: function symbols or lists,
a directive, a disjunction in the body of a probabilistic clause (that of a
certain clause is a clause for each disjunct), arithmetic on symbols, an
annotated disjunction whose body depends on its heads, a negation through
recursion, a non-ground fact, a predicate that no clause defines (where
ProbLog stops on an unknown clause), between/3 of other bounds.
"""
import gzip
import os
import re
import sys
from decimal import Decimal
from itertools import islice, repeat
from operator import itemgetter

REFUSED = 3


class Refused(Exception):
    pass


# --- the ProbLog text --------------------------------------------------------

TOKEN = re.compile(r"""
    (?P<ws>\s+|%[^\n]*|/\*.*?\*/)
  | (?P<num>\d+\.\d+(?:[eE][-+]?\d+)?|\d+[eE][-+]?\d+|\d+)
  | (?P<var>[A-Z_][A-Za-z0-9_]*)
  | (?P<name>[a-z][A-Za-z0-9_]*)
  | (?P<quoted>'(?:[^'\\]|''|\\.)*')
  | (?P<string>"(?:[^"\\]|\\.)*")
  | (?P<end>\.(?=\s|%|$))
  | (?P<op>::|:-|<-|\\\+|\\==|\\=|=\\=|=:=|==|=<|>=|//|[-+*/=<>(),;\[\]|])
""", re.S | re.X)
COMPARISONS = {'=': '=', '==': '=', '=:=': '=', '\\=': '!=', '\\==': '!=', '=\\=': '!=',
               '<': '<', '>': '>', '=<': '<=', '>=': '>='}
ORDERS = {'<', '>', '=<', '>='}
ARITH = {'+': '+', '-': '-', '*': '*', '//': '/', 'mod': '%'}


# A ground fact of plain constants, `P::name(c, ..).` or `name(c, ..).`,
# where a clause starts begins a run of them that is read whole, a window of
# the text at a time (fact_run), each window one token ('facts', (the
# probabilities, the names, the rows of terms)) that parse adds at once. A
# window is read by one findall of FACT_RUN, each fact's arguments as one
# text, and each distinct argument text is read once (Constants); the run
# ends before a fact with an argument that is no CONSTANT, which is read
# token by token. A database of millions of rows (TPC-H at SF 1) then reads
# in seconds, where token by token it took more than ten minutes.
NUMBER = r'\d+(?:\.\d+)?(?:[eE][-+]?\d+)?'
FACT_TEXT = (rf'(?:({NUMBER})\s*::\s*)?(?!query\(|evidence\()([a-z][A-Za-z0-9_]*)'
             r'\(([^()]*)\)\s*\.(?=\s|%|$)')
FACT_START = re.compile(FACT_TEXT)
# a fact and the spaces after it, or the rest of the window, which ends the run
FACT_RUN = re.compile(rf'{FACT_TEXT}\s*|([\s\S]+)')
# an argument of a fact: an integer, a number that is no integer, or a name,
# as the parser reads each (Parser.primary and unary)
CONSTANT = re.compile(r'\s*(?:(-?)(?:(\d+)|(\d+\.\d+(?:[eE][-+]?\d+)?|\d+[eE][-+]?\d+))'
                      r'|([a-z][A-Za-z0-9_]*))\s*')


def tokens(text):
    out, i, start = [], 0, True
    constants = Constants()
    while i < len(text):
        if start and FACT_START.match(text, i):
            j = fact_run(text, i, out, constants)
            if j > i:
                i = j
                continue
        m = TOKEN.match(text, i)
        if not m:
            raise Refused(f'cannot read {text[i:i + 40]!r}')
        i = m.end()
        if m.lastgroup != 'ws':
            # `H <- B`, ProbLog's other neck, is `H :- B`
            out.append((m.lastgroup, ':-' if m.group() == '<-' else m.group()))
            start = m.lastgroup == 'end'
    return out


def fact_run(text, i, out, constants):
    """Reads the run of facts at i into tokens of out, the windows growing
    from 256 bytes to 4 MB as the run goes on, so that a short run reads
    little past its end; the position after the run."""
    size = 256
    while i < len(text):
        end = text.find('\n', i + size) + 1 or len(text)
        got = FACT_RUN.findall(text, i, end)
        rest = got.pop()[3] if got and got[-1][3] else ''
        j = end - len(rest)
        facts = read_facts(got, constants)
        if facts is None:
            # a fact whose arguments are not all constants: the facts before it, one at a time
            got, j = [], i
            while (m := FACT_RUN.match(text, j, end)) and m[2] and read_facts([m.groups('')], constants):
                got.append(m.groups(''))
                j = m.end()
            rest, facts = 'the run ends', read_facts(got, constants)
        if facts:
            out.append(('facts', facts))
            i = j
        if rest or not facts:
            break
        size = min(2 * size, 1 << 22)
    return i


def read_facts(got, constants):
    """(probabilities, names, rows of terms) of facts, each FACT_RUN's
    groups, as the parser reads each fact; None if one has an argument that
    is no CONSTANT, and () for no facts."""
    if not got:
        return ()
    texts = list(map(itemgetter(0), got))
    probs = list(map(float, texts)) if '' not in texts else [float(p) if p else 1.0 for p in texts]
    argss = list(map(itemgetter(2), got))
    try:
        terms = list(map(constants.__getitem__, ','.join(argss).split(',')))
    except NotConstant:
        return None
    commas = list(map(str.count, argss, repeat(',', len(argss))))
    if min(commas) == max(commas):
        rows = list(zip(*[iter(terms)] * (commas[0] + 1)))
    else:
        it = iter(terms)
        rows = [tuple(islice(it, c + 1)) for c in commas]
    return probs, list(map(itemgetter(1), got)), rows


class NotConstant(Exception):
    pass


class Constants(dict):
    """{the text of an argument of a fact (with the spaces around it): its
    term, as the parser reads it}, filled as they are met; NotConstant for
    a text that is no CONSTANT."""

    def __missing__(self, text):
        m = CONSTANT.fullmatch(text)
        if not m:
            raise NotConstant(text)
        sign, digits, number, name = m.groups()
        if name:
            t = ('sym', name)
        elif digits:
            t = ('int', -int(digits) if sign else int(digits))
        else:
            t = ('num', -float(number) if sign else float(number))
        self[text] = t
        return t


class Parser:
    """Clauses of ProbLog's Datalog fragment. A term is ('var', name),
    ('anon',), ('int', n), ('sym', text), ('num', x) (a number that is no
    integer, a constant only), or ('op', op, a, b) and ('neg', a) in
    arithmetic; an atom is (name, [terms]); a literal ('pos', atom),
    ('not', atom), ('cmp', op, a, b), ('is', a, expr), ('true',) or
    ('fail',)."""

    def __init__(self, toks):
        self.t, self.i, self.anon = toks, 0, 0

    def peek(self, k=0):
        return self.t[self.i + k] if self.i + k < len(self.t) else (None, None)

    def take(self, text=None):
        kind, val = self.peek()
        if kind is None or (text is not None and val != text):
            raise Refused(f'expected {text or "more"} at {val!r}')
        self.i += 1
        return kind, val

    def clauses(self):
        """Each clause, (heads, body), and each run of facts read whole, the
        token ('facts', [...]). A certain clause whose body is a disjunction,
        H :- B1 ; B2, is the clauses H :- B1 and H :- B2, as Prolog reads
        it; a probabilistic one draws once for the whole body, which two
        clauses would not, and is refused."""
        while self.peek()[0] is not None:
            if self.peek()[0] == 'facts':
                yield self.take()
                continue
            heads, bodies = self.clause()
            if len(bodies) > 1 and (len(heads) > 1 or heads[0][0] is not None and heads[0][0] < 1):
                raise Refused('a disjunction in the body of a probabilistic clause')
            for body in bodies:
                yield heads, body

    def clause(self):
        """(heads, the body's disjuncts), `,` binding tighter than `;`."""
        if self.peek()[1] == ':-':
            raise Refused('a directive')
        heads = [self.head()]
        while self.peek()[1] == ';':
            self.take(';')
            heads.append(self.head())
        bodies = [[]]
        if self.peek()[1] == ':-':
            self.take(':-')
            bodies[0].append(self.literal())
            while self.peek()[1] in (',', ';'):
                if self.take()[1] == ';':
                    bodies.append([])
                bodies[-1].append(self.literal())
        self.take('.')
        return heads, bodies

    def head(self):
        prob = None
        save = self.i
        try:
            expr = self.expr()
            if self.peek()[1] == '::':
                self.take('::')
                prob = evaluate(expr)
            else:
                self.i = save
        except Refused:
            self.i = save
        return prob, self.atom()

    def literal(self):
        # \+ G, and not(G), ProbLog's not/1, the same negation
        if self.peek()[1] == '\\+' or self.peek() == ('name', 'not') and self.peek(1)[1] == '(':
            self.take()
            if self.peek()[1] == '(':
                self.take('(')
                lit = self.literal()
                self.take(')')
            else:
                lit = ('pos', self.atom())
            if lit[0] == 'pos':
                return ('not', lit[1])
            if lit[0] == 'cmp' and COMPARISONS[lit[1]] in ('=', '!='):
                return ('cmp', '\\=' if COMPARISONS[lit[1]] == '=' else '=', lit[2], lit[3])
            raise Refused('a negation of a comparison other than = and \\=')
        left = self.expr()
        kind, val = self.peek()
        if val in COMPARISONS:
            self.take()
            return ('cmp', val, left, self.expr())
        if (kind, val) == ('name', 'is'):
            self.take()
            return ('is', left, self.expr())
        if left[0] == 'sym' and left[1] in ('true', 'fail', 'false'):
            return ('true',) if left[1] == 'true' else ('fail',)
        if left[0] == 'atom':
            return ('pos', left[1])
        if left[0] == 'sym':
            return ('pos', (left[1], []))
        raise Refused(f'not a literal: {left}')

    def atom(self):
        kind, val = self.take()
        if kind not in ('name', 'quoted'):
            raise Refused(f'not an atom: {val!r}')
        name = unquote(val) if kind == 'quoted' else val
        args = []
        if self.peek()[1] == '(':
            self.take('(')
            # query(A) and evidence(A, V) hold an atom; any other argument is a constant,
            # a variable or arithmetic
            outer = kind == 'name' and name in ('query', 'evidence')
            args = [self.arg(outer)]
            while self.peek()[1] == ',':
                self.take(',')
                args.append(self.arg(outer))
            self.take(')')
        return (name, args)

    def arg(self, outer=False):
        if outer and self.peek()[1] == '\\+':   # evidence(\+A): A observed false, as ProbLog reads it
            self.take()
            return ('negated', ('atom', self.atom()))
        e = self.expr()
        if e[0] == 'atom' and not outer:
            raise Refused(f'a function symbol: {e[1][0]}/{len(e[1][1])}')
        return e

    # arithmetic, with Prolog's precedences: + - below * // mod, unary minus above
    def expr(self):
        a = self.mul()
        while self.peek()[1] in ('+', '-'):
            op = self.take()[1]
            a = ('op', op, a, self.mul())
        return a

    def mul(self):
        a = self.unary()
        while self.peek()[1] in ('*', '/', '//') or self.peek() == ('name', 'mod'):
            op = self.take()[1]
            a = ('op', op, a, self.unary())
        return a

    def unary(self):
        if self.peek()[1] == '-':
            self.take()
            a = self.unary()
            if a[0] == 'int':
                return ('int', -a[1])
            if a[0] == 'num':
                return ('num', -a[1])
            return ('neg', a)
        return self.primary()

    def primary(self):
        kind, val = self.peek()
        if val == '(':
            self.take('(')
            e = self.expr()
            self.take(')')
            return e
        if val in ('[', '|'):
            raise Refused('a list')
        if kind == 'num':
            self.take()
            return ('int', int(val)) if re.fullmatch(r'\d+', val) else ('num', float(val))
        if kind == 'var':
            self.take()
            if val == '_':
                self.anon += 1
                return ('anon',)
            return ('var', val)
        if kind == 'string':
            self.take()
            return ('sym', re.sub(r'\\(.)', r'\1', val[1:-1]))
        if kind in ('name', 'quoted'):
            if self.peek(1)[1] == '(':
                return ('atom', self.atom())
            self.take()
            return ('sym', unquote(val) if kind == 'quoted' else val)
        raise Refused(f'unexpected {val!r}')


def unquote(q):
    return q[1:-1].replace("''", "'")


def evaluate(e):
    """A probability: a number or arithmetic on numbers."""
    if e[0] in ('int', 'num'):
        return float(e[1])
    if e[0] == 'neg':
        return -evaluate(e[1])
    if e[0] == 'op':
        a, b = evaluate(e[2]), evaluate(e[3])
        if e[1] == '/':
            return a / b
        return {'+': a + b, '-': a - b, '*': a * b}.get(e[1]) if e[1] in '+-*' else _bad(e)
    raise Refused(f'a probability that is not a number: {e}')


def _bad(e):
    raise Refused(f'a probability that is not a number: {e}')


def variables(term, out):
    if term[0] == 'var':
        out.setdefault(term[1], None)
    elif term[0] == 'op':
        variables(term[2], out)
        variables(term[3], out)
    elif term[0] == 'neg':
        variables(term[1], out)
    return out


def literal_terms(lit):
    if lit[0] in ('pos', 'not'):
        return lit[1][1]
    if lit[0] == 'cmp':
        return [lit[2], lit[3]]
    if lit[0] == 'is':
        return [lit[1], lit[2]]
    return []


def clause_variables(atoms, body):
    out = {}
    for a in atoms:
        for t in a[1]:
            variables(t, out)
    for lit in body:
        for t in literal_terms(lit):
            variables(t, out)
    return list(out)


# --- the translation ---------------------------------------------------------

# Souffle's keywords, functors and types, which a relation must not be named.
RESERVED = set('''max min mean cat ord fold range strlen sum count substr match contains nil true false as
bnot band bor bxor bshl bshr bshru lnot land lor lxor to_float to_number to_string to_unsigned itou utoi
itof ftoi ftou utof autoinc inline magic brie btree btree_delete eqrel number symbol unsigned float
query evidence input output printsize decl type comp init override functor plan choice subsumes
lt le gt ge eq ne'''.split())


class UnionFind:
    def __init__(self):
        self.parent = {}

    def find(self, x):
        self.parent.setdefault(x, x)
        while self.parent[x] != x:
            self.parent[x] = self.parent[self.parent[x]]
            x = self.parent[x]
        return x

    def union(self, a, b):
        a, b = self.find(a), self.find(b)
        if a != b:
            self.parent[a] = b


def positional(text):
    """A float's repr in positional notation: unchanged without an exponent."""
    return text if 'e' not in text and 'n' not in text else format(Decimal(text), 'f')


def fmt_prob(p):
    """A probability in positional notation, the shortest that reads back as the same double."""
    return positional(repr(float(p)))


def prob_texts(probs):
    """[fmt_prob(p) for p in probs]: only the reprs with an exponent go through Decimal."""
    return list(map(positional, map(repr, map(float, probs))))


def row_texts(rows, types, const, sep):
    """sep.join(const(t, type) for each term t of a row) for each of the
    rows (tuples of terms) of a relation whose arguments' types are `types`:
    by column, each distinct term of a column converted once."""
    if not types:
        return [''] * len(rows)
    columns = []
    for column, ty in zip(zip(*rows), types):
        text = {t: const(t, ty) for t in dict.fromkeys(column)}
        columns.append(map(text.__getitem__, column))
    return list(map(sep.join, zip(*columns)))


def translate(text):
    """(compute.dl's text, {relation: [rows]}, {relation: [probabilities]},
    [(Souffle relation, ProbLog name or a query's atom, arity)], [each query as
    tddlog prints its atom, `_` for a variable])."""
    return emit(*parse(text))


def parse(text):
    """The program's clauses, as emit and problog2scallop.py read them:
    (facts, rules, annotated disjunctions of facts, annotated disjunctions
    with a body, queries, evidence). facts is {(name, arity): {args:
    probability}}; a rule is (probability or None, head, body), its body's
    anonymous variables named where it is probabilistic (`named`); an
    annotated disjunction is ([(probability, head)], body); a query is a
    ground atom or one with variables, a query with a body its ground
    queries (`ground_queries`); a piece of evidence is (atom, value)."""
    p = Parser(tokens(text))
    facts = {}      # (name, arity) -> {args: probability}
    rules = []      # (prob, head, body)
    ads_fact, ads_rule = [], []
    queries, evidence, query_rules = [], [], []

    def fact(name, args, prob):
        """A ground fact, a row of its predicate; a row given twice, both
        independent, is a noisy-or."""
        if not 0 <= prob <= 1:
            raise Refused(f'the probability {prob}')
        rows = facts.setdefault((name, len(args)), {})
        rows[args] = 1 - (1 - rows[args]) * (1 - prob) if args in rows else prob

    def run(probs, names, rows):
        """A token of facts (read_facts), as fact() reads each in turn: at
        once where they are rows of one relation, each new and in range."""
        if names.count(names[0]) == len(names) and len(set(map(len, rows))) == 1 and \
                0 <= min(probs) and max(probs) <= 1:
            have = facts.setdefault((names[0], len(rows[0])), {})
            new = dict(zip(rows, probs))
            if len(new) == len(rows) and have.keys().isdisjoint(new.keys()):
                have.update(new)
                return
        for name, args, prob in zip(names, rows, probs):
            fact(name, args, prob)

    for clause in p.clauses():
        if clause[0] == 'facts':
            run(*clause[1])
            continue
        heads, body = clause
        if len(heads) == 1 and heads[0][0] is None and heads[0][1][0] == 'query' and len(heads[0][1][1]) == 1 and body:
            query_rules.append((as_atom(heads[0][1][1][0]), body))
            continue
        if len(heads) == 1 and not body:
            prob, (name, args) = heads[0]
            if name == 'query' and len(args) == 1 and prob is None:
                queries.append(as_atom(args[0]))
                continue
            if name == 'evidence' and len(args) in (1, 2) and prob is None:
                if args[0][0] == 'negated':
                    if len(args) == 2:
                        raise Refused('evidence on a negation with a value')
                    evidence.append((as_atom(args[0][1]), False))
                    continue
                atom, value = as_atom(args[0]), True
                if len(args) == 2:
                    if args[1] not in (('sym', 'true'), ('sym', 'false')):
                        raise Refused(f'evidence with the value {args[1]}')
                    value = args[1] == ('sym', 'true')
                evidence.append((atom, value))
                continue
        if any(t[0] == 'atom' for _, h in heads for t in h[1]):
            raise Refused(f'a function symbol in the head of a clause of {heads[0][1][0]}')
        body = [lit for lit in body if lit[0] != 'true']
        if any(lit[0] == 'fail' for lit in body):
            for _, (name, args) in heads:
                facts.setdefault((name, len(args)), {})
            continue
        if len(heads) > 1:
            for _, (name, args) in heads:
                facts.setdefault((name, len(args)), {})
            probs = [hp if hp is not None else 1.0 for hp, _ in heads]
            if sum(probs) > 1 + 1e-9 or min(probs) < 0:
                raise Refused(f'an annotated disjunction of probabilities {probs}')
            (ads_rule if body else ads_fact).append(([(pr, h) for pr, h in zip(probs, [h for _, h in heads])], named(body)))
            continue
        prob, head = heads[0]
        if prob is not None and not 0 <= prob <= 1:
            raise Refused(f'the probability {prob}')
        if not body:
            if any(t[0] not in ('int', 'sym', 'num') for t in head[1]):
                raise Refused(f'a fact that is not ground: {head[0]}/{len(head[1])}')
            fact(head[0], tuple(head[1]), 1.0 if prob is None else prob)
            continue
        rules.append((prob, head, body if prob is None or prob >= 1 else named(body)))
    queries += ground_queries(query_rules, facts, rules, ads_fact + ads_rule)
    return facts, rules, ads_fact, ads_rule, queries, evidence


def ground_queries(query_rules, facts, rules, ads):
    """The ground queries of each `query(A) :- B.`, as ProbLog's grounding
    finds them: B a conjunction of atoms of certain relations (`certain`),
    `between(L, H, V)` of integer bounds and comparisons (smokers'
    `query(smokes(X)) :- person(X).`, graph's `query(path(1,Y)) :-
    between(2,N,Y).`, the Circulette reader's `query(faults(A, B)) :-
    gate(A), gate(B), A < B.` over `gate(G) :- gate_and(G, _, _).`)."""
    rel = certain(facts, rules, ads, [lit[1] for _, body in query_rules for lit in body if lit[0] == 'pos'])
    out = []
    for atom, body in query_rules:
        for bind in solutions([lit for lit in body if lit[0] != 'true'], rel, {}):
            ground = [bind.get(t[1]) if t[0] == 'var' else t for t in atom[1]]
            if any(g is None or g[0] not in ('int', 'sym', 'num') for g in ground):
                raise Refused(f'a query of {atom[0]}/{len(atom[1])} whose variables its body does not bind')
            out.append((atom[0], ground))
    return out


BETWEEN = ('between', 3)


def key(atom):
    return (atom[0], len(atom[1]))


def builtins(facts, rules, ads_fact, ads_rule, queries, evidence, enumerate_):
    """(rules, annotated disjunctions with a body) with between/3, ProbLog's
    built-in where the program does not define it, written by `between`
    (enumerate_ its literals that enumerate a range). Any other predicate
    that a clause, a query or the evidence reads and that the program
    neither states facts of nor defines by a clause (`r(_) :- fail.`
    included) is refused, as ProbLog stops on it (an unknown clause): a
    relation of no rows in its place would answer where ProbLog does not,
    and a built-in left as one would answer wrongly."""
    defined = set(facts) | {key(h) for _, h, _ in rules} | {key(h) for alts, _ in ads_fact + ads_rule for _, h in alts}
    builtin = {BETWEEN} - defined
    read = {key(lit[1]) for body in [b for _, _, b in rules] + [b for _, b in ads_fact + ads_rule]
            for lit in body if lit[0] in ('pos', 'not')}
    asked = {key(a) for a in queries} | {key(a) for a, _ in evidence}
    for k in sorted(((read - builtin) | asked) - defined):
        raise Refused(f'{k[0]}/{k[1]}, which no clause defines (ProbLog stops on an unknown clause)')
    if not builtin:
        return rules, ads_rule
    return ([(prob, head, between(body, enumerate_)) for prob, head, body in rules],
            [(alts, between(body, enumerate_)) for alts, body in ads_rule])


def between(body, enumerate_):
    """The body with each between(L, H, X) written as a translation computes
    it: L =< X, X =< H where another literal binds X (an atom's argument,
    the variable an `is` sets, an earlier between's X), and otherwise the
    literals enumerate_(L, H, X, body, k) returns, which bind X to L .. H
    (k, the place they take, names any variable of their own). L and H are
    integers or variables so bound, as ProbLog requires of them."""
    bound = {t[1] for lit in body if lit[0] == 'pos' and key(lit[1]) != BETWEEN
             for t in lit[1][1] if t[0] == 'var'}
    bound |= {lit[1][1] for lit in body if lit[0] == 'is' and lit[1][0] == 'var'}
    out = []
    for lit in body:
        if lit[0] not in ('pos', 'not') or key(lit[1]) != BETWEEN:
            out.append(lit)
            continue
        if lit[0] == 'not':
            raise Refused('a negated between/3')
        lo, hi, x = lit[1][1]
        if not all(t[0] == 'int' or t[0] == 'var' and t[1] in bound for t in (lo, hi)):
            raise Refused('between/3 whose bounds are not bound integers')
        if x[0] == 'int' or x[0] == 'var' and x[1] in bound:
            out += [('cmp', '=<', lo, x), ('cmp', '=<', x, hi)]
            continue
        if x[0] != 'var':
            raise Refused('between/3 of a value that is no integer')
        out += enumerate_(lo, hi, x, body, len(out))
        bound.add(x[1])
    return out


def between_rows(facts, rules, ads, rows):
    """enumerate_ for `between` in PSouffle: between(L, H, X) an atom of the
    input relation between/3, whose rows (L, H, X) of probability 1 it adds
    to `rows` for each value of L and H, an integer or a variable of the
    clause's atoms of certain relations (`certain`, which reads the
    program's rules as ProbLog's grounding does)."""
    def enumerate_(lo, hi, x, body, k):
        refused = 'between/3 whose bounds are not integers of certain relations'
        vs = {t[1] for t in (lo, hi) if t[0] == 'var'}
        src = [lit for lit in body if lit[0] == 'pos' and key(lit[1]) != BETWEEN and
               vs & {t[1] for t in lit[1][1] if t[0] == 'var'}]
        try:
            rel = certain(facts, rules, ads, [lit[1] for lit in src])
            pairs = {(resolve(lo, b), resolve(hi, b)) for b in solutions(src, rel, {})}
        except Refused:
            raise Refused(refused) from None
        for a, b in pairs:
            if a is None or b is None or a[0] != 'int' or b[0] != 'int':
                raise Refused(refused)
            for i in range(a[1], b[1] + 1):
                rows[(a, b, ('int', i))] = 1.0
        return [('pos', ('between', [lo, hi, x]))]
    return enumerate_


def certain(facts, rules, ads, atoms):
    """{(name, arity): {row: None}}: the rows of the relations of `atoms`
    and of those they read, each a relation the program states for certain
    (facts of probability one, rules without a probability whose bodies are
    atoms of such relations and comparisons), evaluated bottom up. A
    relation that is not certain, or has no clause, is refused; between/3
    is the builtin where the program does not define it."""
    uncertain = {(h[0], len(h[1])) for alts, _ in ads for _, h in alts}
    uncertain |= {k for k, rows in facts.items() if any(q != 1.0 for q in rows.values())}
    by = {}
    for prob, head, body in rules:
        k = (head[0], len(head[1]))
        if prob is not None and prob < 1:
            uncertain.add(k)
        by.setdefault(k, []).append((head, body))
    need, todo = set(), [(name, len(args)) for name, args in atoms]
    while todo:
        k = todo.pop()
        if k in need or k == BETWEEN and k not in facts and k not in by:
            continue
        if k not in facts and k not in by:
            raise Refused(f'a query whose body {k[0]}/{k[1]} has no facts')
        if k in uncertain or any(lit[0] not in ('pos', 'cmp') for _, body in by.get(k, ()) for lit in body):
            raise Refused(f'a query whose body {k[0]}/{k[1]} is not a relation of certain facts')
        need.add(k)
        todo += [(lit[1][0], len(lit[1][1])) for _, body in by.get(k, ()) for lit in body if lit[0] == 'pos']
    rel = {k: dict.fromkeys(facts.get(k, {})) for k in need}
    changed = True
    while changed:
        changed = False
        for k in need:
            for head, body in by.get(k, ()):
                for bind in list(solutions(body, rel, {})):
                    row = tuple(resolve(t, bind) for t in head[1])
                    if any(t is None for t in row):
                        raise Refused(f'a rule of {k[0]}/{k[1]} whose head its body does not bind')
                    if row not in rel[k]:
                        rel[k][row] = None
                        changed = True
    return rel


def solutions(body, rel, bind):
    """The bindings that extend `bind` under which each literal of `body`
    holds: an atom of a relation of `rel` (`certain`), between/3 of integer
    bounds where `rel` has no such relation, a comparison of constants."""
    if not body:
        yield bind
        return
    lit, rest = body[0], body[1:]
    if lit[0] == 'pos':
        name, args = lit[1]
        if (name, len(args)) == BETWEEN and BETWEEN not in rel:
            lo, hi = resolve(args[0], bind), resolve(args[1], bind)
            if lo is None or hi is None or lo[0] != 'int' or hi[0] != 'int':
                raise Refused('a query whose body between/3 has no integer bounds')
            rows = [(lo, hi, ('int', i)) for i in range(lo[1], hi[1] + 1)]
        else:
            rows = rel[(name, len(args))]
        for row in rows:
            b = dict(bind)
            if all(match(t, c, b) for t, c in zip(args, row)):
                yield from solutions(rest, rel, b)
    elif lit[0] == 'cmp':
        a, c = resolve(lit[2], bind), resolve(lit[3], bind)
        if a is None or c is None:
            raise Refused(f'a query whose body compares by {lit[1]} a term that is no bound constant')
        if compare(lit[1], a, c):
            yield from solutions(rest, rel, bind)
    else:
        raise Refused('a query whose body is not atoms and comparisons')


def resolve(t, bind):
    """The constant term `t` is under `bind`; None where it is unbound or no constant."""
    if t[0] == 'var':
        return bind.get(t[1])
    return t if t[0] in ('int', 'sym', 'num') else None


def compare(op, a, b):
    """ProbLog's comparison `op` of two constants: = and == and their
    negations on terms, the orders, =:= and =\\= on numbers."""
    if op in ('=', '==', '\\=', '\\=='):
        return (a == b) == (op in ('=', '=='))
    if a[0] == 'sym' or b[0] == 'sym':
        raise Refused(f'a query whose body compares a symbol by {op}')
    x, y = a[1], b[1]
    return {'=:=': x == y, '=\\=': x != y, '<': x < y, '>': x > y, '=<': x <= y, '>=': x >= y}[op]


def match(t, c, bind):
    if t[0] == 'anon':
        return True
    if t[0] == 'var':
        return bind.setdefault(t[1], c) == c
    return t == c


def atom_text(atom):
    """An atom as tddlog prints it and bench.py keys it, `_` for a variable."""
    def text(t):
        if t[0] in ('var', 'anon'):
            return '_'
        if t[0] == 'sym':
            return t[1] if re.fullmatch(r'[a-z][A-Za-z0-9_]*', t[1]) else "'" + t[1].replace("'", "''") + "'"
        if t[0] in ('int', 'num'):
            return str(t[1])
        raise Refused(f'a query of the term {t}')
    return atom[0] + (f'({",".join(text(t) for t in atom[1])})' if atom[1] else '')


def named(body):
    """The body with each anonymous variable a variable of its own: a
    probabilistic clause draws its choice once per ground instance, every
    variable of the clause bound, as ProbLog draws it."""
    count = [0]

    def term(t):
        if t[0] == 'anon':
            count[0] += 1
            return ('var', f'_A{count[0]}')
        if t[0] == 'op':
            return ('op', t[1], term(t[2]), term(t[3]))
        if t[0] == 'neg':
            return ('neg', term(t[1]))
        return t

    def lit(x):
        if x[0] == 'pos':       # in a negated literal `_` stays: \+ p(X, _) is no p(X, Y) for any Y
            return ('pos', (x[1][0], [term(t) for t in x[1][1]]))
        return x
    return [lit(x) for x in body]


def as_atom(t):
    if t[0] == 'atom':
        return t[1]
    if t[0] == 'sym':
        return (t[1], [])
    raise Refused(f'a query of {t}')


def emit(facts, rules, ads_fact, ads_rule, queries, evidence):
    rows = {}
    rules, ads_rule = builtins(facts, rules, ads_fact, ads_rule, queries, evidence,
                               between_rows(facts, rules, ads_fact + ads_rule, rows))
    if rows:
        facts = {**facts, BETWEEN: rows}
    preds = set(facts)
    for _, head, body in rules:
        preds.add((head[0], len(head[1])))
        for lit in body:
            if lit[0] in ('pos', 'not'):
                preds.add((lit[1][0], len(lit[1][1])))
    for alts, body in ads_rule + ads_fact:
        for lit in body:
            if lit[0] in ('pos', 'not'):
                preds.add((lit[1][0], len(lit[1][1])))
    for atom in [a for a in queries] + [a for a, _ in evidence]:
        preds.add((atom[0], len(atom[1])))
    rel = relation_names(preds, RESERVED, r'[A-Za-z_][A-Za-z0-9_]*')
    ruled = {(h[0], len(h[1])) for _, h, _ in rules} | {(h[0], len(h[1])) for alts, _ in ads_rule + ads_fact
                                                        for _, h in alts}

    # the rules of the annotated disjunctions, and their input relations
    out_rules = []                 # (prob, head atom (relation, args), body literals with relation names)
    inputs = {}                    # relation -> {args: probability}
    positions = {}                 # relation -> [the ProbLog position (name, arity, i) or a fresh class]
    uf = UnionFind()
    kinds = {}                     # class -> set of 'int', 'sym', 'num', 'arith', 'order'

    def mark(node, kind):
        kinds.setdefault(node, set()).add(kind)

    def own(relation, *classes):
        positions[relation] = list(classes)

    for key in preds:
        own(rel[key], *[(key[0], key[1], i) for i in range(key[1])])
    for key, rows in facts.items():
        if not rows:
            continue                    # declared, and never read
        if key[1] == 0:
            # a fact of no arguments is the row 0 of an input relation of one
            target = rel[key] + '__f'
            own(target, ('nullary', key[0]))
            mark(('nullary', key[0]), 'int')
            inputs[target] = {(('int', 0),): rows[()]}
            out_rules.append((None, (rel[key], []), [('pos', (target, [('int', 0)]))]))
            continue
        target = rel[key] + '__f' if key in ruled else rel[key]
        inputs[target] = rows
        if key in ruled:
            own(target, *positions[rel[key]])
            xs = [('var', f'x{i}') for i in range(key[1])]
            out_rules.append((None, (rel[key], xs), [('pos', (target, xs))]))

    def lit_rel(lit):
        if lit[0] in ('pos', 'not'):
            return (lit[0], (rel[(lit[1][0], len(lit[1][1]))], lit[1][1]))
        return lit

    for prob, head, body in rules:
        out_rules.append((prob, (rel[(head[0], len(head[1]))], head[1]), [lit_rel(b) for b in body]))
    shapes = ad_fact_shapes(ads_fact, rel, own, inputs, out_rules, lambda j, part: f'ad{j}_{part}')
    # annotated disjunctions with a body: a chain of choices over the clause's variables
    deps = dependencies(rules, ads_rule)
    for j, (alts, body) in enumerate(ads_rule, shapes):
        heads = {h[0] for _, h in alts}
        below = reach(deps, {lit[1][0] for lit in body if lit[0] in ('pos', 'not')})
        if heads & below:
            raise Refused('an annotated disjunction whose body depends on its own heads (a negation through recursion)')
        vs = [('var', v) for v in clause_variables([h for _, h in alts], body)]
        c = f'ad{j}_c'
        own(c, ('ad', j, 'k'), *[('adv', j, v[1]) for v in vs])
        mark(('ad', j, 'k'), 'int')
        b = [lit_rel(x) for x in body]
        certain = sum(pr for pr, _ in alts) >= 1 - 1e-9
        left = 1.0
        for k, (pr, h) in enumerate(alts, 1):
            last = k == len(alts) and certain
            if not last:
                out_rules.append((min(1.0, pr / left) if left > 0 else 0.0, (c, [('int', k)] + vs), b))
            left -= pr
            hb = b + [('not', (c, [('int', i)] + vs)) for i in range(1, k)]
            if not last:
                hb.append(('pos', (c, [('int', k)] + vs)))
            out_rules.append((None, (rel[(h[0], len(h[1]))], h[1]), hb))
    stratified(out_rules)
    types = infer_types(uf, kinds, positions, inputs, out_rules,
                        [(rel[(a[0], len(a[1]))], a[1]) for a in list(queries) + [a for a, _ in evidence]])
    for r in list(types):
        if not any(r == x for x in inputs) and not any(r == h[0] for _, h, _ in out_rules) and \
                r not in [rel[k] for k in preds]:
            del types[r]

    def const(t, ty):
        if ty == 'number':
            if t[0] != 'int':
                raise Refused(f'the constant {t[1]} in a numeric argument')
            return str(t[1])
        text = str(t[1])
        if '\t' in text or '\n' in text:
            raise Refused('a constant with a tab or a newline')
        return text

    def term(t, ty, names):
        if t[0] == 'var':
            return names[t[1]]
        if t[0] == 'anon':
            return '_'
        if t[0] in ('int', 'sym', 'num'):
            return const(t, ty) if ty == 'number' else '"' + const(t, ty).replace('\\', '\\\\').replace('"', '\\"') + '"'
        if t[0] == 'neg':
            return f'-({term(t[1], ty, names)})'
        if t[0] == 'op':
            if t[1] == '/':
                raise Refused("the division / (tddlog's and ProbLog's on integers is not Souffle's)")
            return f'({term(t[2], ty, names)} {ARITH[t[1]]} {term(t[3], ty, names)})'
        raise Refused(f'the term {t}')

    def literal(lit, names):
        if lit[0] in ('pos', 'not'):
            r, args = lit[1]
            s = f'{r}({", ".join(term(a, types[r][i], names) for i, a in enumerate(args))})'
            return s if lit[0] == 'pos' else '!' + s
        a, b = (lit[2], lit[3]) if lit[0] == 'cmp' else (lit[1], lit[2])
        op = COMPARISONS[lit[1]] if lit[0] == 'cmp' else '='
        ty = cmp_type(a, b, names)
        return f'{term(a, ty, names)} {op} {term(b, ty, names)}'

    def cmp_type(a, b, names):
        for t in (a, b):
            if t[0] == 'var' and t[1] in names.types:
                return names.types[t[1]]
        return 'number' if any(t[0] in ('int', 'op', 'neg') for t in (a, b)) else 'symbol'

    class Names(dict):
        pass

    lines = ['// translated from a ProbLog program by problog2souffle.py']
    for r in sorted(types):
        cols = ', '.join(f'a{i}:{ty}' for i, ty in enumerate(types[r]))
        lines.append(f'.decl {r}({cols})')
        if inputs.get(r):
            lines.append(f'.input {r}')
    for prob, head, body in out_rules:
        if prob == 0:
            continue
        names = Names()
        names.types = {}
        for atom in [head] + [lit[1] for lit in body if lit[0] in ('pos', 'not')]:
            for i, t in enumerate(atom[1]):
                if t[0] == 'var':
                    names.types.setdefault(t[1], types[atom[0]][i])
        for v in clause_variables([head], body):
            names[v] = 'u' + v if v.startswith('_') else v
        safe_vars(head, body)
        pre = '' if prob is None or prob >= 1 else fmt_prob(prob) + '::'
        lines.append(f'{pre}{literal(("pos", head), names)} :- {", ".join(literal(b, names) for b in body)}.')
    # PSouffle names its code for a query or the evidence after the relation, so a relation
    # takes one query and one piece of evidence: a second ground atom of it is asked through a
    # relation of no arguments of its own, aux() :- atom, and several queries with variables
    # of one relation (or one beside ground ones) are one query of every tuple of it
    aux, asked = [], []

    def single(atom, kind):
        r = rel[(atom[0], len(atom[1]))]
        lit = literal(('pos', (r, atom[1])), Names())
        name = f'{kind}__{len(aux)}'
        aux.append((name, atom_text(atom)))
        lines.append(f'{name}() :- {lit}.')
        return f'{name}()'

    on = {}
    for atom, _ in evidence:
        if any(t[0] not in ('int', 'sym', 'num') for t in atom[1]):
            raise Refused('evidence on an atom that is not ground')
        on[(atom[0], len(atom[1]))] = on.get((atom[0], len(atom[1])), 0) + 1
    for atom, value in evidence:
        key = (atom[0], len(atom[1]))
        lit = single(atom, 'e') if on[key] > 1 else literal(('pos', (rel[key], atom[1])), Names())
        lines.append(f'evidence({lit}, {"true" if value else "false"}).')
    by = {}
    for atom in queries:
        named = [t[1] for t in atom[1] if t[0] == 'var']
        if len(set(named)) < len(named):
            raise Refused(f'a query with a repeated variable: {atom[0]}/{len(atom[1])}')
        by.setdefault((atom[0], len(atom[1])), []).append(atom)
        asked.append(atom_text(atom))
    for key, atoms in by.items():
        open_ = [a for a in atoms if any(t[0] in ('var', 'anon') for t in a[1])]
        if open_:
            args = open_[0][1] if len(atoms) == 1 else [('anon',)] * key[1]
            args = [('anon',) if t[0] == 'var' else t for t in args]
            lines.append(f'query({literal(("pos", (rel[key], args)), Names())}).')
        elif len(atoms) == 1:
            lines.append(f'query({literal(("pos", (rel[key], atoms[0][1])), Names())}).')
        else:
            seen = set()
            for a in atoms:
                if atom_text(a) not in seen:
                    seen.add(atom_text(a))
                    lines.append(f'query({single(a, "q")}).')
    for name, _ in aux:
        lines.append(f'.decl {name}()')
    rows_out, probs_out = {}, {}
    for r, rows in inputs.items():
        if not rows:
            continue
        rows_out[r] = row_texts(rows, types[r], const, '\t')
        probs_out[r] = prob_texts(rows.values())   # 1.0 for a certain row, as PSouffle's own inputs
    table = [(rel[k], k[0], k[1]) for k in sorted(preds)] + [(name, text, 0) for name, text in aux]
    return '\n'.join(lines) + '\n', rows_out, probs_out, table, asked


def relation_names(preds, reserved, ident):
    """{(name, arity): relation}: each predicate a relation of its name where
    that is an identifier (`ident`) and not `reserved`, with `_` appended
    where it is, `q_` before a name that is no identifier, and `__a<arity>`
    after a name of two arities. A translation's own relations have `__` in
    their names, which no predicate's relation may then have."""
    arities = {}
    for name, n in preds:
        arities.setdefault(name, set()).add(n)
    rel = {}
    for name, n in sorted(preds):
        base = name if re.fullmatch(ident, name) else 'q_' + re.sub(r'\W', '_', name)
        if len(arities[name]) > 1:
            base = f'{base}__a{n}'
        elif base in reserved:
            base = base + '_'
        rel[(name, n)] = base
    if len(set(rel.values())) != len(rel) or any('__' in r and not re.search(r'__a\d+$', r) for r in rel.values()):
        raise Refused('a predicate name that collides with the translation\'s own names')
    return rel


def ad_fact_shapes(ads_fact, rel, own, inputs, out_rules, name):
    """The annotated disjunctions of facts, by shape (their heads'
    predicates, and whether their probabilities sum to 1), each shape's
    choices the rows of an input relation name(j, 'c') (an annotated
    disjunction's number and k: the choice c_k, as for a body) and its heads
    the rows of name(j, 'a<k>'), and a rule for each head. Adds the input
    relations' positions (`own`), rows (`inputs`) and rules (`out_rules`);
    returns the number of shapes."""
    shapes = {}
    for alts, _ in ads_fact:
        for _, h in alts:
            if any(t[0] not in ('int', 'sym', 'num') for t in h[1]):
                raise Refused(f'an annotated disjunction of facts that are not ground: {h[0]}/{len(h[1])}')
        total = sum(pr for pr, _ in alts)
        shape = (tuple((h[0], len(h[1])) for _, h in alts), total >= 1 - 1e-9)
        shapes.setdefault(shape, []).append(alts)
    for j, ((heads, certain), group) in enumerate(sorted(shapes.items(), key=lambda s: s[0])):
        c = name(j, 'c')
        own(c, ('ad', j, 'id'), ('ad', j, 'k'))
        inputs[c] = {}
        n = len(heads)
        for k, (pred, ar) in enumerate(heads, 1):
            a = name(j, f'a{k}')
            own(a, ('ad', j, 'id'), *[(pred, ar, i) for i in range(ar)])
            inputs[a] = {}
        for ident, alts in enumerate(group):
            left = 1.0
            for k, (pr, h) in enumerate(alts, 1):
                inputs[name(j, f'a{k}')][(('int', ident), *h[1])] = 1.0
                if k < n or not certain:
                    inputs[c][(('int', ident), ('int', k))] = min(1.0, pr / left) if left > 0 else 0.0
                    left -= pr
        for k, (pred, ar) in enumerate(heads, 1):
            xs = [('var', f'x{i}') for i in range(ar)]
            body = [('pos', (name(j, f'a{k}'), [('var', 'id')] + xs))]
            body += [('not', (c, [('var', 'id'), ('int', i)])) for i in range(1, k)]
            if k < n or not certain:
                body.append(('pos', (c, [('var', 'id'), ('int', k)])))
            out_rules.append((None, (rel[(pred, ar)], xs), body))
    return len(shapes)


def infer_types(uf, kinds, positions, inputs, out_rules, atoms):
    """{relation: ['number' or 'symbol' for each argument]}: the positions
    (relation -> [the class of each argument], joined in `uf`) joined
    through each rule's variables, a class numbers where every constant that
    reaches it is an integer or arithmetic or an order applies to it, and
    symbols otherwise. kinds holds the classes' marks so far; the rows of
    the inputs and the constants of `atoms` ([(relation, args)], the
    queries and the evidence) mark theirs. Arithmetic or an order on
    symbols is refused."""
    def mark(node, kind):
        kinds.setdefault(node, set()).add(kind)

    for relation, cls in positions.items():
        for i, cl in enumerate(cls):
            uf.union((relation, i), cl)
    for relation, rows in inputs.items():
        for i, column in enumerate(zip(*rows)):
            for kind in set(map(itemgetter(0), column)):
                mark(uf.find((relation, i)), kind)
    for ci, (_, head, body) in enumerate(out_rules):
        def at(atom):
            for i, t in enumerate(atom[1]):
                walk(t, (atom[0], i))

        def walk(t, node):
            if t[0] == 'var':
                uf.union(('v', ci, t[1]), node)
            elif t[0] in ('int', 'sym', 'num'):
                mark(uf.find(node), t[0])
            elif t[0] == 'op':
                mark(uf.find(node), 'arith')
                walk(t[2], node)
                walk(t[3], node)
            elif t[0] == 'neg':
                mark(uf.find(node), 'arith')
                walk(t[1], node)
        at(head)
        for lit in body:
            if lit[0] in ('pos', 'not'):
                at(lit[1])
            elif lit[0] in ('cmp', 'is'):
                a, bb = (lit[2], lit[3]) if lit[0] == 'cmp' else (lit[1], lit[2])
                node = ('cmp', ci, id(lit))
                walk(a, node)
                walk(bb, node)
                if lit[0] == 'is' or (lit[0] == 'cmp' and lit[1] in ORDERS):
                    mark(uf.find(node), 'arith' if lit[0] == 'is' else 'order')
    for relation, args in atoms:
        for i, t in enumerate(args):
            if t[0] in ('int', 'sym', 'num'):
                mark(uf.find((relation, i)), t[0])
    merged = {}
    for node, ks in kinds.items():
        merged.setdefault(uf.find(node), set()).update(ks)

    def type_of(relation, i):
        ks = merged.get(uf.find((relation, i)), set())
        if 'sym' in ks or 'num' in ks:
            if 'arith' in ks or 'order' in ks:
                raise Refused(f'arithmetic or an order on symbols (argument {i + 1} of {relation})')
            return 'symbol'
        return 'number' if ks & {'int', 'arith', 'order'} else 'symbol'
    return {r: [type_of(r, i) for i in range(len(cls))] for r, cls in positions.items()}


def safe_vars(head, body):
    """Every variable of the head or of a negated literal is bound by a
    positive literal or an equation, as Souffle requires."""
    bound = {}
    for lit in body:
        if lit[0] == 'pos':
            for t in lit[1][1]:
                variables(t, bound)
    changed = True
    while changed:
        changed = False
        for lit in body:
            if lit[0] in ('is', 'cmp') and (lit[0] == 'is' or lit[1] == '='):
                a, b = (lit[1], lit[2]) if lit[0] == 'is' else (lit[2], lit[3])
                for x, y in ((a, b), (b, a)):
                    if x[0] == 'var' and x[1] not in bound and all(v in bound for v in variables(y, {})):
                        bound[x[1]] = None
                        changed = True
    need = {}
    for t in head[1]:
        variables(t, need)
    for lit in body:
        if lit[0] != 'pos':
            for t in literal_terms(lit):
                variables(t, need)
    free = [v for v in need if v not in bound]
    if free:
        raise Refused(f'the variable {free[0]} of a rule of {head[0]} that no positive literal binds')


def dependencies(rules, ads_rule):
    deps = {}
    for _, head, body in rules:
        deps.setdefault(head[0], set()).update(lit[1][0] for lit in body if lit[0] in ('pos', 'not'))
    for alts, body in ads_rule:
        for _, h in alts:
            deps.setdefault(h[0], set()).update(lit[1][0] for lit in body if lit[0] in ('pos', 'not'))
    return deps


def reach(deps, start):
    seen, stack = set(), list(start)
    while stack:
        x = stack.pop()
        if x not in seen:
            seen.add(x)
            stack.extend(deps.get(x, ()))
    return seen


def stratified(rules):
    """Refuses a negation inside a recursive component (Souffle compiles
    stratified negation only)."""
    deps = {}
    for _, head, body in rules:
        for lit in body:
            if lit[0] in ('pos', 'not'):
                deps.setdefault(head[0], set()).add(lit[1][0])
    for _, head, body in rules:
        for lit in body:
            if lit[0] == 'not' and head[0] in reach(deps, {lit[1][0]}):
                raise Refused(f'a negation through recursion ({head[0]} and {lit[1][0]})')


def write(outdir, text):
    dl, rows, probs, table, asked = translate(text)
    os.makedirs(os.path.join(outdir, 'input'), exist_ok=True)
    with open(os.path.join(outdir, 'compute.dl'), 'w') as f:
        f.write(dl)
    for r, lines in rows.items():
        with open(os.path.join(outdir, 'input', r + '.facts'), 'w') as f:
            f.write('\n'.join(lines) + '\n')
        with open(os.path.join(outdir, 'input', r + '.prob'), 'w') as f:
            f.write('\n'.join(probs[r]) + '\n')
    with open(os.path.join(outdir, 'names.tsv'), 'w') as f:
        f.write(''.join(f'{r}\t{n}\t{a}\n' for r, n, a in table))
    with open(os.path.join(outdir, 'queries.txt'), 'w') as f:
        f.write(''.join(q + '\n' for q in asked))


ANSWER = re.compile(r'^(\w+)\((.*)\)\s*:\s*(\S+)\s*$')


def answers(facts_prob, names_tsv):
    """{ProbLog atom as tddlog prints it: probability} from PSouffle's
    facts.prob and the translation's names.tsv."""
    back = {}
    for line in names_tsv.splitlines():
        r, n, a = line.split('\t')
        back[r] = n
    out = {}
    for line in facts_prob.splitlines():
        m = ANSWER.match(line.strip())
        if not m:
            continue
        r, args, p = m.groups()
        if r not in back:
            continue
        vals = [a[1:-1].replace('\\"', '"').replace('\\\\', '\\') if a.startswith('"') else a
                for a in split_args(args)]
        out[back[r] + (f'({",".join(vals)})' if vals else '')] = float(p)
    return out


def split_args(s):
    out, cur, quote, esc = [], '', False, False
    for ch in s:
        if esc:
            cur += ch
            esc = False
        elif ch == '\\' and quote:
            cur += ch
            esc = True
        elif ch == '"':
            quote = not quote
            cur += ch
        elif ch == ',' and not quote:
            out.append(cur.strip())
            cur = ''
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__.split('\n\n')[1])
    path, outdir = sys.argv[1:]
    with (gzip.open if path.endswith('.gz') else open)(path, 'rt') as f:
        text = f.read()
    try:
        write(outdir, text)
    except Refused as x:
        print(f'problog2souffle: {x}', file=sys.stderr)
        sys.exit(REFUSED)


if __name__ == '__main__':
    main()
