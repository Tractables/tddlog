#!/usr/bin/env python3
"""Translate a ProbLog program of the Datalog fragment into Scallop, first
order as it is written.

usage: problog2scallop.py PROGRAM > NAME.scl      (PROGRAM gzipped or not)

Scallop (Li, Huang and Naik, "Scallop: A Language for Neurosymbolic
Programming", PLDI 2023; github.com/scallop-lang/scallop) evaluates a
Datalog program bottom up over tagged tuples; under its top-k-proofs
provenance (`scli -p topkproofs -k K`) a tuple's tag is a set of proofs over
independent probabilistic facts, and its probability is the weighted model
count of their disjunction. This writes the Scallop program of a ProbLog
program (problog2souffle.py's parse reads it):

- each predicate is a relation of its name (a Scallop keyword or type with
  `_` appended, a name of two arities with `__a<arity>`), declared with the
  types of its arguments: i64 where every constant that reaches the
  argument through the clauses' variables is an integer, String otherwise;
- the ground facts of a predicate are one set, `rel edge = {0.3::(1, 2),
  (2, 3)}`, a tuple given twice one of probability 1 - (1 - p)(1 - q);
- a rule is a rule, `\\+ A` and `not(A)` are `not A`, `X is E` is
  `x == E`, and the comparisons are Scallop's;
- between(L, H, X), ProbLog's built-in where the program does not define
  between/3, is `L <= x and x <= H` where another literal binds X, and
  otherwise Scallop's foreign predicate `range<i64>(L, H + 1, x)`, which
  enumerates L to H (its end is exclusive); a predicate that no clause
  defines, where ProbLog stops on an unknown clause, is refused
  (problog2souffle.py's builtins);
- a probabilistic rule p::H :- B is drawn once per ground instance of the
  clause, every variable V of the clause bound, as ProbLog draws it.
  Scallop's own probabilistic rule, `rel "p"::H = B` (its probability a
  string, which Scallop reads as an f64; as a number it reads an f32),
  draws once per tuple of
  the rule's first atom, and again each time that tuple's proofs grow in a
  recursion, so it is written so where B has one positive atom, of a
  relation and not range's, and H is not
  recursive. Otherwise the draw is a relation of its own: where V is empty
  a row of the switch facts `choice__` (j, 1); where it is not `rel
  p::sw__j(V) = dom__j(V)`, over the tuples of V for which B can hold,
  `dom__j(V) = B'`, and then H :- sw__j(V), B. B' is B with every relation
  that is not certain read as its copy R__d, the relation as if each of
  its facts were certain and each of its clauses always chose its head,
  and with the negations of such relations dropped, so that sw__j holds
  one independent draw for every ground instance that has one;
- an annotated disjunction p1::h1; ..; pn::hn :- B chooses its head in
  order: the choice c_k, of probability p_k / (1 - p1 - .. - p(k-1)), is
  drawn once per ground clause, and h_k holds where B, c_k and none of
  c_1 .. c(k-1) do (the last choice is certain where the p's sum to 1). The
  annotated disjunctions of facts are rows of the relations of their
  shape, ad__<j>_c (the choices) and ad__<j>_a<k> (the heads); one with a
  body draws its choices as rows of `choice__` (j, k) where V is empty, and
  as `rel q_k::ch__j(k, V) = dom__j(V)` otherwise;
- each query is a relation q__i, of no arguments where the query is
  ground, of the query's arguments where it has variables; a query with a
  body over certain relations is its ground queries. With evidence, e__()
  holds where the evidence does, and qe__i where the query and the
  evidence do, and the harness divides: Scallop does not condition. The
  comments `// query q__i ATOM` and `// open q__i NAME` at the top map the
  relations back (`answers`).

A program Scallop cannot be given exits with status 3 (bench.py's REFUSED)
and the reason. Scallop's own language refuses a negation through
recursion (it stratifies negation by relation), a fact that is not ground
or a rule whose head or negation no positive literal binds (range
restriction), arithmetic or an order on an argument that also holds
symbols (each argument has one type), and a constant with a backslash (its
string literals read one back ambiguously). A reason that begins
`translator:` is this translator's limit and not Scallop's: function
symbols and lists (Scallop's algebraic data types could hold them), a
disjunction in a body, flexible probabilities, and the like.
"""
import gzip
import re
import sys

import problog2souffle as p2s
from problog2souffle import Refused, REFUSED

# Scallop's keywords, types and aggregators (its grammar.lalrpop and its
# foreign aggregates), which a relation or a variable must not be named.
RESERVED = set('''and as bool bound case char const else exists extern false forall free if implies import in
is isize new not or query rel relation then true type usize where i8 i16 i32 i64 i128 u8 u16 u32 u64 u128
f32 f64 count sum prod min max avg unique top categorical uniform normalize softmax argmax argmin argsort rank
sort enumerate disjunct string_join weighted_sum weighted_avg range'''.split())
IDENT = r'[A-Za-z][A-Za-z0-9_]*'
TYPES = {'number': 'i64', 'symbol': 'String'}
COMPARE = {'=': '==', '!=': '!=', '<': '<', '>': '>', '<=': '<=', '>=': '>='}
# ProbLog's term equality and its negation, under which `X + 1` is a compound term
TERM_EQ = {'=', '==', '\\=', '\\=='}
# The reason of every program that negates through recursion, whichever its
# relations, so that the records of such programs give one reason.
NEGATION_CYCLE = 'a negation through recursion, which Scallop cannot express (it stratifies negation by relation)'
# Refusals of Scallop's own language; every other one is the translator's.
SCALLOP_OWN = re.compile(r'a negation through recursion|that are not ground|a fact that is not ground'
                         r'|that no positive literal binds|arithmetic or an order on symbols|a backslash')


def key(atom):
    return (atom[0], len(atom[1]))


# Scallop's foreign predicate range<i64>(begin, end, i), which binds i to begin .. end - 1
# (its core/src/common/foreign_predicates/range.rs), as an atom of the rules
RANGE = 'range<i64>'
RANGE_KEY = (RANGE, 3)


def scallop_range(lo, hi, x, body, k):
    """enumerate_ for p2s.between: range<i64>(L, H + 1, X), H + 1 a variable
    of its own where H is one."""
    if hi[0] == 'int':
        return [('pos', (RANGE, [lo, ('int', hi[1] + 1), x]))]
    end = ('var', f'between#{k}')
    return [('is', end, ('op', '+', hi, ('int', 1))), ('pos', (RANGE, [lo, end, x]))]


def components(deps):
    """{node: its strongly connected component's id} of the graph deps
    (node -> nodes), by Tarjan's algorithm, iteratively."""
    index, low, comp, stack, on = {}, {}, {}, [], set()
    nodes = set(deps) | {w for ws in deps.values() for w in ws}
    for root in sorted(nodes):
        if root in index:
            continue
        index[root] = low[root] = len(index)
        stack.append(root)
        on.add(root)
        work = [(root, iter(sorted(deps.get(root, ()))))]
        while work:
            v, it = work[-1]
            w = next(it, None)
            if w is None:
                work.pop()
                if work:
                    low[work[-1][0]] = min(low[work[-1][0]], low[v])
                if low[v] == index[v]:
                    while True:
                        x = stack.pop()
                        on.discard(x)
                        comp[x] = v
                        if x == v:
                            break
            elif w not in index:
                index[w] = low[w] = len(index)
                stack.append(w)
                on.add(w)
                work.append((w, iter(sorted(deps.get(w, ())))))
            elif w in on:
                low[v] = min(low[v], index[w])
    return comp


def no_compound(lit):
    """Refuses arithmetic where ProbLog reads it as a compound term: as an
    atom's argument and under term equality."""
    terms = lit[1][1] if lit[0] in ('pos', 'not') else [lit[2], lit[3]] if lit[0] == 'cmp' and lit[1] in TERM_EQ \
        else []
    if any(t[0] in ('op', 'neg') for t in terms):
        raise Refused('translator: arithmetic as a term (a compound term in ProbLog)')


def emit(facts, rules, ads_fact, ads_rule, queries, evidence):
    rules, ads_rule = p2s.builtins(facts, rules, ads_fact, ads_rule, queries, evidence, scallop_range)
    preds = set(facts)
    for _, head, body in rules:
        preds.add(key(head))
        no_compound(('pos', head))
        for lit in body:
            no_compound(lit)
            if lit[0] in ('pos', 'not') and key(lit[1]) != RANGE_KEY:
                preds.add(key(lit[1]))
    for alts, body in ads_rule + ads_fact:
        for _, h in alts:
            no_compound(('pos', h))
        for lit in body:
            no_compound(lit)
            if lit[0] in ('pos', 'not') and key(lit[1]) != RANGE_KEY:
                preds.add(key(lit[1]))
    for atom in list(queries) + [a for a, _ in evidence]:
        preds.add(key(atom))
    rel = p2s.relation_names(preds, RESERVED, IDENT)

    # the predicates' dependencies, their recursive components, and the predicates that are certain
    deps = {}
    for prob, head, body in rules:
        if prob != 0:
            deps.setdefault(key(head), set()).update(key(lit[1]) for lit in body if lit[0] in ('pos', 'not'))
    for alts, body in ads_rule:
        for _, h in alts:
            deps.setdefault(key(h), set()).update(key(lit[1]) for lit in body if lit[0] in ('pos', 'not'))
    comp = components(deps)
    size = {}
    for k, c in comp.items():
        size[c] = size.get(c, 0) + 1
    recursive = {k for k in comp if size[comp[k]] > 1 or k in deps.get(k, ())}
    uncertain = {k for k, rows in facts.items() if rows and min(rows.values()) < 1}
    uncertain |= {key(h) for alts, _ in ads_fact + ads_rule for _, h in alts}
    uncertain |= {key(h) for prob, h, _ in rules if prob is not None and 0 < prob < 1}
    changed = True
    while changed:
        changed = False
        for k, ds in deps.items():
            if k not in uncertain and ds & uncertain:
                uncertain.add(k)
                changed = True

    out_rules = []                 # (prob, head atom (relation, args), body literals with relation names)
    inputs = {}                    # relation -> {args: probability}
    positions = {}                 # relation -> [the ProbLog position (name, arity, i) or the relation's own]
    uf, kinds = p2s.UnionFind(), {}

    def own(relation, *classes):
        positions[relation] = list(classes)

    def fresh(relation, n):
        own(relation, *[(relation, i) for i in range(n)])

    for k in preds:
        own(rel[k], *[(k[0], k[1], i) for i in range(k[1])])
    for k, rows in facts.items():
        if rows:
            inputs[rel[k]] = rows
    fresh(RANGE, 3)
    for i in range(3):
        kinds[(RANGE, i)] = {'int'}    # range's arguments are integers

    def lits(body):
        return [(lit[0], (rel[key(lit[1])], lit[1][1])) if lit[0] in ('pos', 'not') and key(lit[1]) != RANGE_KEY
                else lit for lit in body]

    copies = []                    # the predicates whose copy R__d a domain reads, in order

    def relaxed(body):
        """B': B as the relations' copies read it (the module's docstring)."""
        out = []
        for lit in body:
            if lit[0] in ('pos', 'not') and key(lit[1]) in uncertain:
                if lit[0] == 'pos':
                    if key(lit[1]) not in copies:
                        copies.append(key(lit[1]))
                    out.append(('pos', (rel[key(lit[1])] + '__d', lit[1][1])))
            else:
                out.extend(lits([lit]))
        return out

    def choice(j, k, prob):
        """The atom of choice__ (j, k), a switch fact of probability prob."""
        if 'choice__' not in inputs:
            inputs['choice__'] = {}
            fresh('choice__', 2)
        inputs['choice__'][(('int', j), ('int', k))] = prob
        return ('choice__', [('int', j), ('int', k)])

    def domain(j, vs, body):
        name = f'dom__{j}'
        fresh(name, len(vs))
        out_rules.append((None, (name, vs), relaxed(body)))
        return ('pos', (name, vs))

    j = 0                          # the probabilistic clauses with a body, numbered
    for prob, head, body in rules:
        if prob == 0:
            continue
        h = (rel[key(head)], head[1])
        if prob is None or prob >= 1:
            out_rules.append((None, h, lits(body)))
            continue
        vs = [('var', v) for v in p2s.clause_variables([head], body)]
        pos = [lit[1] for lit in body if lit[0] == 'pos']
        if key(head) not in recursive and len(pos) == 1 and key(pos[0]) != RANGE_KEY:
            out_rules.append((prob, h, lits(body)))         # Scallop's own probabilistic rule
        elif not vs:
            out_rules.append((None, h, lits(body) + [('pos', choice(j, 1, prob))]))
        else:
            sw = f'sw__{j}'
            fresh(sw, len(vs))
            out_rules.append((prob, (sw, vs), [domain(j, vs, body)]))
            out_rules.append((None, h, [('pos', (sw, vs))] + lits(body)))
        j += 1
    p2s.ad_fact_shapes(ads_fact, rel, own, inputs, out_rules, lambda i, part: f'ad__{i}_{part}')
    for alts, body in ads_rule:
        vs = [('var', v) for v in p2s.clause_variables([h for _, h in alts], body)]
        dom = domain(j, vs, body) if vs else None
        certain = sum(pr for pr, _ in alts) >= 1 - 1e-9
        left, chosen = 1.0, []
        for k, (pr, head) in enumerate(alts, 1):
            q = min(1.0, pr / left) if left > 0 else 0.0
            left -= pr
            if q <= 0:
                continue               # a head that never holds, and a choice never made
            last = k == len(alts) and certain
            c = None
            if not last and vs:
                c = (f'ch__{j}', [('int', k)] + vs)
                if f'ch__{j}' not in positions:
                    fresh(f'ch__{j}', len(vs) + 1)
                out_rules.append((None if q >= 1 else q, c, [dom]))
            elif not last:
                c = choice(j, k, q)
            out_rules.append((None, (rel[key(head)], head[1]),
                              lits(body) + [('not', x) for x in chosen] + ([('pos', c)] if c else [])))
            if c:
                chosen.append(c)
        j += 1
    # the copies R__d that the domains read, and those that the copies' rules read
    i = 0
    while i < len(copies):
        k = copies[i]
        i += 1
        d = rel[k] + '__d'
        own(d, *positions[rel[k]])
        rows = {args: 1.0 for args in facts.get(k, {})}
        for alts, _ in ads_fact:
            rows.update({tuple(h[1]): 1.0 for pr, h in alts if pr > 0 and key(h) == k})
        if rows:
            inputs[d] = rows
        for prob, head, body in rules:
            if key(head) == k and prob != 0:
                out_rules.append((None, (d, head[1]), relaxed(body)))
        for alts, body in ads_rule:
            for pr, head in alts:
                if key(head) == k and pr > 0:
                    out_rules.append((None, (d, head[1]), relaxed(body)))

    # the queries and the evidence
    ev = []
    for atom, value in evidence:
        if any(t[0] not in ('int', 'sym', 'num') for t in atom[1]):
            raise Refused('translator: evidence on an atom that is not ground')
        ev.append(('pos' if value else 'not', (rel[key(atom)], atom[1])))
    comments, asked = [], []
    for n, atom in enumerate(queries):
        q = f'q__{n}'
        if all(t[0] in ('int', 'sym', 'num') for t in atom[1]):
            comments.append(f'// query {q} {p2s.atom_text(atom)}')
            args, hargs = atom[1], []
        else:
            named = [t[1] for t in atom[1] if t[0] == 'var']
            if len(set(named)) < len(named):
                raise Refused(f'translator: a query with a repeated variable: {atom[0]}/{len(atom[1])}')
            args = [('var', f'_Q{i}') if t[0] == 'anon' else t for i, t in enumerate(atom[1])]
            hargs = args
            comments.append(f'// open {q} {atom[0]}')
        q = f'qe__{n}' if ev else q
        fresh(q, len(hargs))
        out_rules.append((None, (q, hargs), [('pos', (rel[key(atom)], args))] + ev))
        asked.append(q)
    if ev:
        fresh('e__', 0)
        out_rules.append((None, ('e__', []), ev))
        asked.insert(0, 'e__')
        comments.append('// evidence e__')

    try:
        p2s.stratified(out_rules)
    except Refused:
        raise Refused(NEGATION_CYCLE) from None
    for _, head, body in out_rules:
        p2s.safe_vars(head, body)
    types = p2s.infer_types(uf, kinds, positions, inputs, out_rules, [])
    return render(comments, types, inputs, out_rules, asked)


def render(comments, types, inputs, out_rules, asked):
    def const(t, ty):
        if ty == 'number':
            if t[0] != 'int':
                raise Refused(f'translator: the constant {t[1]} in a numeric argument')
            return str(t[1])
        text = str(t[1])
        if '\\' in text:
            raise Refused('a constant with a backslash, which Scallop reads back ambiguously')
        return '"' + text.replace('"', '\\"').replace('\n', '\\n').replace('\t', '\\t').replace('\r', '\\r') + '"'

    def term(t, ty, names):
        if t[0] == 'var':
            return names[t[1]]
        if t[0] == 'anon':
            return '_'
        if t[0] in ('int', 'sym', 'num'):
            return const(t, ty)
        if t[0] == 'neg':
            return f'(0 - {term(t[1], ty, names)})'
        if t[0] == 'op':
            if t[1] == '/':
                raise Refused("translator: the division / (ProbLog's on integers is a float's)")
            return f'({term(t[2], ty, names)} {p2s.ARITH[t[1]]} {term(t[3], ty, names)})'
        raise Refused(f'translator: the term {t}')

    def atom(a, names):
        return f'{a[0]}({", ".join(term(x, types[a[0]][i], names) for i, x in enumerate(a[1]))})'

    def literal(lit, names, vtypes):
        if lit[0] == 'pos':
            return atom(lit[1], names)
        if lit[0] == 'not':
            return 'not ' + atom(lit[1], names)
        a, b = (lit[2], lit[3]) if lit[0] == 'cmp' else (lit[1], lit[2])
        op = COMPARE[p2s.COMPARISONS[lit[1]]] if lit[0] == 'cmp' else '=='
        ty = next((vtypes[t[1]] for t in (a, b) if t[0] == 'var' and t[1] in vtypes),
                  'number' if any(t[0] in ('int', 'op', 'neg') for t in (a, b)) else 'symbol')
        # a constraint stands bare: Scallop reads `(x < 3)` as an expression
        # in parentheses, which a comparison must then follow
        return f'{term(a, ty, names)} {op} {term(b, ty, names)}'

    lines = ['// translated from a ProbLog program by problog2scallop.py'] + comments
    for r in sorted(types):
        if r != RANGE:                 # a foreign predicate of Scallop's, declared by Scallop
            lines.append(f'type {r}({", ".join(TYPES[ty] for ty in types[r])})')
    for r in sorted(inputs):
        rows = inputs[r]
        if not rows:
            continue
        if not types[r]:
            q = rows[()]
            lines.append(f'rel {"" if q >= 1 else p2s.fmt_prob(q) + "::"}{r}()')
            continue
        tags = [('' if q >= 1 else t + '::') for q, t in zip(rows.values(), p2s.prob_texts(rows.values()))]
        tuples = [f'  {tag}({args}),' for tag, args in zip(tags, p2s.row_texts(rows, types[r], const, ', '))]
        tuples[-1] = tuples[-1][:-1]
        lines.append(f'rel {r} = {{')
        lines += tuples + ['}']
    for prob, head, body in out_rules:
        if not body:
            raise Refused(f'translator: a rule of {head[0]} with no body')
        vtypes = {}
        for a in [head] + [lit[1] for lit in body if lit[0] in ('pos', 'not')]:
            for i, t in enumerate(a[1]):
                if t[0] == 'var':
                    vtypes.setdefault(t[1], types[a[0]][i])
        names = variable_names(p2s.clause_variables([head], body))
        # a rule's probability as a string, which Scallop's tag predicate parses
        # as f64; a number there is an f32 (0.3 reads 0.30000001192092896, and
        # one below about 1e-45 reads 0), and a constant declared f64 is a cast
        # from F64 to F64 that its runtime does not implement
        pre = '' if prob is None or prob >= 1 else f'"{p2s.fmt_prob(prob)}"::'
        lines.append(f'rel {pre}{atom(head, names)} = {" and ".join(literal(b, names, vtypes) for b in body)}')
    lines += [f'query {q}' for q in asked]
    return '\n'.join(lines) + '\n'


def variable_names(variables):
    """{ProbLog variable: Scallop variable}: lower case, as Scallop's programs
    write them, an identifier that begins with a letter, and no keyword."""
    out, used = {}, set()
    for v in variables:
        base = re.sub(r'\W', '', v).lstrip('_').lower() or 'v'
        if not base[0].isalpha():
            base = 'v' + base
        name, n = base, 1
        while name in used or name in RESERVED:
            name, n = f'{base}_{n}', n + 1
        used.add(name)
        out[v] = name
    return out


def translate(text):
    """The Scallop program of the ProbLog program `text`."""
    return emit(*p2s.parse(text))


# --- the answers -------------------------------------------------------------

COMMENT = re.compile(r'^// (query|open|evidence) (\w+)(?: (.*))?$')
RESULT = re.compile(r'^(\w+): \{(.*)\}\s*$')
STRING = r'"(?:[^"\\]|\\.)*"'
# an element of a set scli prints: its probability and its tuple, `(v, ..)` or one value
ELEMENT = re.compile(rf'\s*([0-9.eE+-]+)::(?:\(((?:{STRING}|[^()"])*)\)|({STRING}|[^,\s{{}}()"]+))\s*,?')
VALUE = re.compile(r'\s*(?:"((?:[^"\\]|\\.)*)"|([^,\s]+))\s*,?')
ESCAPES = {'n': '\n', 't': '\t', 'r': '\r', '0': '\0'}


def value(text, quoted):
    """A value scli prints (a string as Rust's Debug writes it) as a term."""
    if quoted is not None:
        s = re.sub(r'\\(.)', lambda m: ESCAPES.get(m.group(1), m.group(1)), quoted)
    else:
        s = text
    return ('int', int(s)) if re.fullmatch(r'-?\d+', s) else ('sym', s)


def tuples(text):
    """{(values): probability} of the set scli prints, `0.5::("a", 1), ...`."""
    out, i = {}, 0
    while text[i:].strip():
        m = ELEMENT.match(text, i)
        if not m:
            raise ValueError(f'scli output: cannot read {text[i:i + 60]!r}')
        i = m.end()
        inner = m.group(2) if m.group(3) is None else m.group(3)
        args, k = [], 0
        while inner[k:].strip():
            v = VALUE.match(inner, k)
            args.append(value(v.group(2), v.group(1)))
            k = v.end()
        out[tuple(args)] = float(m.group(1))
    return out


def answers(out, scl):
    """{each query's atom as tddlog prints it: its probability} from scli's
    output `out` and the program `scl` (its comments): with evidence the
    query's and the evidence's together divided by the evidence's, and
    'inconsistent' where the evidence has no proof. A ground query scli
    lists no tuple of has probability 0; a query with variables is answered
    by each tuple scli lists."""
    asked, observed = [], False
    for line in scl.splitlines():
        if not line.startswith('//'):
            break
        m = COMMENT.match(line)
        if m and m.group(1) == 'evidence':
            observed = True
        elif m:
            asked.append(m.groups())
    printed = {}
    for line in out.splitlines():
        m = RESULT.match(line)
        if m:
            printed[m.group(1)] = tuples(m.group(2))
    e = 1.0
    if observed:
        if 'e__' not in printed:
            raise ValueError('scli output: no e__, the evidence')
        e = printed['e__'].get((), 0.0)
        if e == 0:
            return 'inconsistent'
    res = {}
    for kind, q, text in asked:
        rows = printed.get('qe__' + q[3:] if observed else q, {})
        if kind == 'query':
            res[text] = rows.get((), 0.0) / e
        else:
            for args, p in rows.items():
                res[p2s.atom_text((text, list(args)))] = p / e
    return res


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__.split('\n\n')[1])
    path = sys.argv[1]
    with (gzip.open if path.endswith('.gz') else open)(path, 'rt') as f:
        text = f.read()
    try:
        program = translate(text)
    except Refused as x:
        reason = str(x)
        if not reason.startswith('translator:') and not SCALLOP_OWN.search(reason):
            reason = 'translator: ' + reason
        print(f'problog2scallop: {reason}', file=sys.stderr)
        sys.exit(REFUSED)
    sys.stdout.write(program)


if __name__ == '__main__':
    main()
