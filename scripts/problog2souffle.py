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
  as ProbLog does; `\\+ A` is `!A`, the comparisons and `is` their Souffle
  forms, and a rule whose body holds `fail` is left out;
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
  body over certain facts is its ground queries. OUTDIR/queries.txt lists
  the queries as tddlog prints their atoms.

An argument position holds Souffle numbers where every constant that
reaches it, through the variables the clauses share, is an integer, and
symbols otherwise. A program that PSouffle cannot be given exits with
status 3 (bench.py's REFUSED) and the reason: function symbols or lists,
a disjunction or a directive in a body, arithmetic on symbols, an
annotated disjunction whose body depends on its heads, a negation through
recursion, a non-ground fact.
"""
import gzip
import os
import re
import sys
from decimal import Decimal

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
  | (?P<op>::|:-|\\\+|\\==|\\=|=\\=|=:=|==|=<|>=|//|[-+*/=<>(),;\[\]|])
""", re.S | re.X)
COMPARISONS = {'=': '=', '==': '=', '=:=': '=', '\\=': '!=', '\\==': '!=', '=\\=': '!=',
               '<': '<', '>': '>', '=<': '<=', '>=': '>='}
ORDERS = {'<', '>', '=<', '>='}
ARITH = {'+': '+', '-': '-', '*': '*', '//': '/', 'mod': '%'}


def tokens(text):
    out, i = [], 0
    while i < len(text):
        m = TOKEN.match(text, i)
        if not m:
            raise Refused(f'cannot read {text[i:i + 40]!r}')
        i = m.end()
        if m.lastgroup != 'ws':
            out.append((m.lastgroup, m.group()))
    return out


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
        while self.peek()[0] is not None:
            yield self.clause()

    def clause(self):
        if self.peek()[1] == ':-':
            raise Refused('a directive')
        heads = [self.head()]
        while self.peek()[1] == ';':
            self.take(';')
            heads.append(self.head())
        body = []
        if self.peek()[1] == ':-':
            self.take(':-')
            body = [self.literal()]
            while self.peek()[1] in (',', ';'):
                if self.take()[1] == ';':
                    raise Refused('a disjunction in a body')
                body.append(self.literal())
        self.take('.')
        return heads, body

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
        if self.peek()[1] == '\\+':
            self.take('\\+')
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


def fmt_prob(p):
    return format(Decimal(repr(float(p))), 'f')


def translate(text):
    """(compute.dl's text, {relation: [rows]}, {relation: [probabilities]},
    [(Souffle relation, ProbLog name or a query's atom, arity)], [each query as
    tddlog prints its atom, `_` for a variable])."""
    p = Parser(tokens(text))
    facts = {}      # (name, arity) -> {args: probability}
    rules = []      # (prob, head, body)
    ads_fact, ads_rule = [], []
    queries, evidence, query_rules = [], [], []
    for heads, body in p.clauses():
        if len(heads) == 1 and heads[0][0] is None and heads[0][1][0] == 'query' and len(heads[0][1][1]) == 1 and body:
            query_rules.append((as_atom(heads[0][1][1][0]), body))
            continue
        if len(heads) == 1 and not body:
            prob, (name, args) = heads[0]
            if name == 'query' and len(args) == 1 and prob is None:
                queries.append(as_atom(args[0]))
                continue
            if name == 'evidence' and len(args) in (1, 2) and prob is None:
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
            rows = facts.setdefault((head[0], len(head[1])), {})
            q = 1.0 if prob is None else prob
            key = tuple(head[1])
            rows[key] = 1 - (1 - rows[key]) * (1 - q) if key in rows else q
            continue
        rules.append((prob, head, body if prob is None or prob >= 1 else named(body)))
    queries += ground_queries(query_rules, facts, rules, ads_fact + ads_rule)
    return emit(facts, rules, ads_fact, ads_rule, queries, evidence)


def ground_queries(query_rules, facts, rules, ads):
    """The ground queries of each `query(A) :- B.`, where B is one atom of a
    relation of certain facts and no rules (smokers' `query(smokes(X)) :-
    person(X).`), as ProbLog's grounding finds them."""
    derived = {(h[0], len(h[1])) for _, h, _ in rules} | {(h[0], len(h[1])) for alts, _ in ads for _, h in alts}
    out = []
    for atom, body in query_rules:
        lits = [lit for lit in body if lit[0] != 'true']
        if len(lits) != 1 or lits[0][0] != 'pos':
            raise Refused('a query whose body is not one atom')
        name, args = lits[0][1]
        rows = facts.get((name, len(args)), {})
        if (name, len(args)) in derived or any(q != 1.0 for q in rows.values()):
            raise Refused(f'a query whose body {name}/{len(args)} is not a relation of certain facts')
        for row in rows:
            bind = {}
            if all(match(t, c, bind) for t, c in zip(args, row)):
                ground = [bind.get(t[1]) if t[0] == 'var' else t for t in atom[1]]
                if any(g is None or g[0] not in ('int', 'sym', 'num') for g in ground):
                    raise Refused(f'a query of {atom[0]}/{len(atom[1])} whose variables its body does not bind')
                out.append((atom[0], ground))
    return out


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
    arities = {}
    for name, n in preds:
        arities.setdefault(name, set()).add(n)
    rel = {}
    for name, n in sorted(preds):
        base = name if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', name) else 'q_' + re.sub(r'\W', '_', name)
        if len(arities[name]) > 1:
            base = f'{base}__a{n}'
        elif base in RESERVED:
            base = base + '_'
        rel[(name, n)] = base
    if len(set(rel.values())) != len(rel) or any('__' in r and not re.search(r'__a\d+$', r) for r in rel.values()):
        raise Refused('a predicate name that collides with the translation\'s own names')
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
    # annotated disjunctions of facts, by shape: their choices and heads are rows
    shapes = {}
    for alts, _ in ads_fact:
        for _, h in alts:
            if any(t[0] not in ('int', 'sym', 'num') for t in h[1]):
                raise Refused(f'an annotated disjunction of facts that are not ground: {h[0]}/{len(h[1])}')
        total = sum(pr for pr, _ in alts)
        shape = (tuple((h[0], len(h[1])) for _, h in alts), total >= 1 - 1e-9)
        shapes.setdefault(shape, []).append(alts)
    for j, ((heads, certain), group) in enumerate(sorted(shapes.items(), key=lambda s: s[0])):
        c = f'ad{j}_c'
        own(c, ('ad', j, 'id'), ('ad', j, 'k'))
        mark(('ad', j, 'id'), 'int')
        mark(('ad', j, 'k'), 'int')
        inputs[c] = {}
        n = len(heads)
        for k, (name, ar) in enumerate(heads, 1):
            a = f'ad{j}_a{k}'
            own(a, ('ad', j, 'id'), *[(name, ar, i) for i in range(ar)])
            inputs[a] = {}
        for ident, alts in enumerate(group):
            left = 1.0
            for k, (pr, h) in enumerate(alts, 1):
                inputs[f'ad{j}_a{k}'][(('int', ident), *h[1])] = 1.0
                if k < n or not certain:
                    inputs[c][(('int', ident), ('int', k))] = min(1.0, pr / left) if left > 0 else 0.0
                    left -= pr
        for k, (name, ar) in enumerate(heads, 1):
            xs = [('var', f'x{i}') for i in range(ar)]
            body = [('pos', (f'ad{j}_a{k}', [('var', 'id')] + xs))]
            body += [('not', (c, [('var', 'id'), ('int', i)])) for i in range(1, k)]
            if k < n or not certain:
                body.append(('pos', (c, [('var', 'id'), ('int', k)])))
            out_rules.append((None, (rel[(name, ar)], xs), body))
    # annotated disjunctions with a body: a chain of choices over the clause's variables
    deps = dependencies(rules, ads_rule)
    for j, (alts, body) in enumerate(ads_rule, len(shapes)):
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

    # types: the positions joined through each clause's variables
    for relation, cls in positions.items():
        for i, cl in enumerate(cls):
            uf.union((relation, i), cl)
    for relation, rows in inputs.items():
        for args in rows:
            for i, t in enumerate(args):
                mark(uf.find((relation, i)), t[0])
    for ci, (_, head, body) in enumerate(out_rules):
        def at(atom):
            for i, t in enumerate(atom[1]):
                walk(t, (atom[0], i))

        def walk(t, node, kind=None):
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
    for atom in list(queries) + [a for a, _ in evidence]:
        for i, t in enumerate(atom[1]):
            if t[0] in ('int', 'sym', 'num'):
                mark(uf.find((rel[(atom[0], len(atom[1]))], i)), t[0])
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
    types = {r: [type_of(r, i) for i in range(len(cls))] for r, cls in positions.items()}
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
        rows_out[r] = ['\t'.join(const(t, types[r][i]) for i, t in enumerate(args)) for args in rows]
        probs_out[r] = [fmt_prob(q) for q in rows.values()]   # 1.0 for a certain row, as PSouffle's own inputs
    table = [(rel[k], k[0], k[1]) for k in sorted(preds)] + [(name, text, 0) for name, text in aux]
    return '\n'.join(lines) + '\n', rows_out, probs_out, table, asked


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
            f.write(''.join(line + '\n' for line in lines))
        with open(os.path.join(outdir, 'input', r + '.prob'), 'w') as f:
            f.write(''.join(q + '\n' for q in probs[r]))
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
