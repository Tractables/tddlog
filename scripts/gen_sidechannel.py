#!/usr/bin/env python3
"""Write a side-channel analysis of PSouffle as tddlog and ProbLog read it.

usage: gen_sidechannel.py PROGRAM FACTS [--souffle DIR]

The side-channel benchmarks of PSouffle: Li, Xia, Adnan, Wang, "PSouffle:
Scaling Exact Probabilistic Logic Inference for Program Analysis" (FMCAD
2026). Each of its 19 cases (P1, P3 to P20) is a Souffle program of 20 to
80 rules, most of them probabilistic, that labels the variables of a
cryptographic routine as key-sensitive, key-independent or random from
their assignments and from bit-vector helper relations, and the case's
input relations, from 56 to 147,000 tuples, some of them probabilistic.
It asks every tuple of its output relations. PROGRAM is the case's
compute.souffle.dl and FACTS its compute.problog.dl, from the PSouffle
repository (evaluation/full/side_channel/full/CASE/, MIT licence), which
data/fetch.txt names, gzipped or not. FACTS is the authors' ProbLog
version of the case, and only its facts are read: they are the tuples of
the input relations (input/R.facts) with their probabilities
(input/R.prob), relation by relation in the order of the files, a
probability of 1 written as a plain fact.

The rules are translated from PROGRAM, since Souffle does not read them
as ProbLog does:

- in Souffle `x = e` evaluates e, where ProbLog unifies x with the term
  e: an equation whose one side is a variable not yet bound and whose
  other side is arithmetic is `X is E` (band, bor and bxor are /\\, \\/
  and xor); between two bound sides it is a comparison (=:= where a side
  is arithmetic, = otherwise);
- the order of a body does not matter in Souffle, and ProbLog evaluates
  its built-ins from left to right: the atoms come first, then the
  equations in the order in which their variables are bound, then the
  negations (`!(x = 0)` is `X \\= 0`, `!R(x)` is `\\+ r(X)`) and the
  comparisons;
- relations are lowercase and variables uppercase;
- a relation with no tuple and no rule, which ProbLog cannot call, is
  declared by a clause that derives nothing, `r(0, 0) :- fail.`, as in
  gen_taint.py;
- an equivalence relation (`eqrel`) is the relation its rules define:
  every case states its symmetry and transitivity as rules, which give
  its reflexive pairs too, and a program that does not is refused;
- each output relation is asked for all its tuples, `query(r(_, _))`.

A construct outside this (strings, functors, aggregates, other operators)
is refused (exit 3).

With --souffle DIR, DIR holds the case as PSouffle reads it: DIR/compute.dl
is PROGRAM, unchanged; DIR/input/ holds the R.facts and R.prob of every
input relation, written back from FACTS (the repository's files, byte for
byte, but that a relation with no tuples has empty files); DIR/names.tsv
maps each output relation to the predicate it is here, and
DIR/queries.txt lists the queries as tddlog prints them.
"""
import gzip
import os
import re
import sys

REFUSED = 3


class Refused(Exception):
    pass


def read(path):
    with (gzip.open if path.endswith('.gz') else open)(path, 'rt') as f:
        return f.read()


def strip_comments(text):
    return re.sub(r'//[^\n]*', '', re.sub(r'/\*.*?\*/', '', text, flags=re.S))


def split_top(s, sep=','):
    """s split at the separators outside parentheses."""
    out, depth, cur = [], 0, ''
    for ch in s:
        if ch == '(':
            depth += 1
        elif ch == ')':
            depth -= 1
        if ch == sep and depth == 0:
            out.append(cur)
            cur = ''
        else:
            cur += ch
    out.append(cur)
    return [p.strip() for p in out]


def unwrap(s):
    """s without the parentheses around the whole of it."""
    s = s.strip()
    while s.startswith('(') and s.endswith(')'):
        depth = 0
        for i, ch in enumerate(s):
            depth += ch == '('
            depth -= ch == ')'
            if depth == 0 and i < len(s) - 1:
                return s
        s = s[1:-1].strip()
    return s


# --- the Souffle program ----------------------------------------------------

def parse(text):
    """(declarations {relation: (arity, eqrel)}, inputs, outputs, clauses
    [(probability or None, head, body literals)])."""
    decls, inputs, outputs, clauses, cur = {}, [], [], [], ''
    for line in strip_comments(text).splitlines():
        line = line.strip()
        if not line:
            continue
        if not cur and line.startswith('.'):
            words = line.split()
            if words[0] == '.decl':
                m = re.fullmatch(r'\.decl\s+(\w+)\s*\((.*)\)\s*(eqrel)?', line)
                if not m:
                    raise Refused(f'a declaration tddlog does not read: {line}')
                decls[m[1]] = (len(split_top(m[2])) if m[2].strip() else 0, bool(m[3]))
            elif words[0] in ('.input', '.output'):
                for r in words[1:]:
                    (inputs if words[0] == '.input' else outputs).append(r)
            elif words[0] != '.type':
                raise Refused(f'a directive tddlog does not read: {line}')
            continue
        cur += ' ' + line
        if cur.endswith('.'):
            clauses.append(clause(cur.strip()[:-1]))
            cur = ''
    if cur:
        raise Refused(f'an unfinished clause: {cur.strip()}')
    for r in inputs + outputs:
        if r not in decls:
            raise Refused(f'an undeclared relation: {r}')
    return decls, list(dict.fromkeys(inputs)), list(dict.fromkeys(outputs)), clauses


def clause(text):
    m = re.fullmatch(r'([0-9.eE+-]+)\s*::\s*(.*)', text, re.S)
    p, text = (m[1], m[2]) if m else (None, text)
    head, _, body = text.partition(':-')
    return p, atom(head), [literal(b) for b in split_top(body)] if body.strip() else []


def atom(text):
    m = re.fullmatch(r'\s*(\w+)\s*\((.*)\)\s*', text, re.S)
    if not m:
        raise Refused(f'not an atom: {text.strip()}')
    args = split_top(m[2]) if m[2].strip() else []
    for a in args:
        if not re.fullmatch(r'-?\d+|[A-Za-z_]\w*', a):
            raise Refused(f'an argument that is neither a variable nor a number: {a}')
    return m[1], args


COMPARISONS = ['!=', '<=', '>=', '=', '<', '>']


def literal(text):
    """('atom', (rel, args)), ('not', (rel, args)), ('cmp', (op, lhs, rhs))
    or ('ncmp', (op, lhs, rhs)): a constraint and its negation."""
    neg = text.startswith('!')
    if neg:
        text = text[1:]
    text = unwrap(text)
    if re.fullmatch(r'\w+\s*\(.*\)', text, re.S) and not any(f' {op} ' in text for op in COMPARISONS):
        return ('not' if neg else 'atom', atom(text))
    for op in COMPARISONS:
        parts = split_op(text, op)
        if parts:
            return ('ncmp' if neg else 'cmp', (op, tokens(unwrap(parts[0])), tokens(unwrap(parts[1]))))
    raise Refused(f'a literal tddlog does not read: {text}')


def split_op(text, op):
    """(lhs, rhs) where `op` is the comparison at the top of text, else
    None."""
    depth = 0
    for i in range(len(text)):
        depth += text[i] == '('
        depth -= text[i] == ')'
        if depth == 0 and text.startswith(op, i):
            before, after = text[i - 1:i], text[i + len(op):i + len(op) + 1]
            if before in '!<>=' and before or after == '=':
                continue
            return text[:i].strip(), text[i + len(op):].strip()
    return None


ARITH = {'+': '+', '-': '-', '*': '*', 'band': '/\\', 'bor': '\\/', 'bxor': 'xor'}


def tokens(text):
    out = re.findall(r'\d+|[A-Za-z_]\w*|[-+*()]|\S', text)
    for t in out:
        if not re.fullmatch(r'\d+|[A-Za-z_]\w*|[-+*()]', t):
            raise Refused(f'an expression tddlog does not read: {text}')
    return out


def variables(toks):
    return {t for t in toks if re.fullmatch(r'[A-Za-z_]\w*', t) and t not in ARITH and t != '_'}


def arithmetic(toks):
    ops = {t for t in toks if t in ARITH}
    if len(ops) > 1:
        raise Refused(f'an expression of two operators: {" ".join(toks)}')
    return bool(ops)


# --- the ProbLog program ----------------------------------------------------

def translate(program, facts_text):
    """(the ProbLog program, the PSouffle files {input relation: (facts
    lines, prob lines)}, names.tsv rows, queries)."""
    decls, inputs, outputs, clauses = parse(program)
    pred = {}
    for r in decls:
        if r.lower() in pred.values():
            raise Refused(f'two relations named {r.lower()} in lowercase')
        pred[r] = r.lower()
    rel_of = {p: r for r, p in pred.items()}
    facts, rows = [], {r: ([], []) for r in inputs}
    for line in facts_text.splitlines():
        m = re.fullmatch(r'(?:([0-9.eE+-]+)::)?([a-z_]\w*)\(([-0-9, ]*)\)\.', line.strip())
        if not m:
            continue
        r = rel_of.get(m[2])
        if r not in rows:
            raise Refused(f'a fact of {m[2]}, which is not an input relation')
        values = [v.strip() for v in m[3].split(',')]
        if len(values) != decls[r][0]:
            raise Refused(f'a fact of {m[2]} of arity {len(values)}')
        facts.append(line.strip())
        rows[r][0].append('\t'.join(values))
        rows[r][1].append(m[1] or '1.0')
    defined = {r for r in rows if rows[r][0]} | {head[0] for _, head, _ in clauses}
    for r, (arity, eqrel) in decls.items():
        if eqrel and not equivalence(r, clauses):
            raise Refused(f'the equivalence relation {r}, whose rules do not state its symmetry and transitivity')
    lines, called = [], set(pred[r] for r in outputs)
    for p, head, body in clauses:
        text, used = rule(p, head, body, pred)
        lines.append(text)
        called |= used
    stubs = [f'{pred[r]}({", ".join("0" * decls[r][0])}) :- fail.' for r in decls
             if pred[r] in called and r not in defined]
    queries = [f'{pred[r]}({",".join("_" * decls[r][0])})' for r in outputs]
    text = ''.join(f + '\n' for f in facts) + '\n' + ''.join(s + '\n' for s in lines + stubs) + '\n' + \
        ''.join(f'query({q}).\n' for q in queries)
    return text, rows, [(r, pred[r], decls[r][0]) for r in outputs], queries


def equivalence(r, clauses):
    """Whether rules without a probability state that r is symmetric and
    transitive."""
    sym = trans = False
    for p, (h, ha), body in clauses:
        atoms = [b[1] for b in body if b[0] == 'atom']
        if p is not None or h != r or len(atoms) != len(body) or any(a[0] != r for a in atoms):
            continue
        if len(atoms) == 1 and atoms[0][1] == ha[::-1]:
            sym = True
        if len(atoms) == 2 and atoms[0][1][0] == ha[0] and atoms[1][1][1] == ha[1] and \
                atoms[0][1][1] == atoms[1][1][0] and atoms[0][1][1] not in ha:
            trans = True
    return sym and trans


def rule(p, head, body, pred):
    """The ProbLog clause of a Souffle clause and the predicates its body
    calls."""
    names = {}

    def var(v):
        if v == '_':
            return '_'
        u = v.upper()
        if names.setdefault(u, v) != v:
            raise Refused(f'the variables {names[u]} and {v} of one clause')
        return u

    def term(toks):
        return ' '.join(ARITH.get(t, var(t) if t in variables([t]) else t) for t in toks) \
            .replace('( ', '(').replace(' )', ')')

    def at(rel, args):
        if not args:
            return pred[rel]
        return pred[rel] + '(' + ', '.join(a if re.fullmatch(r'-?\d+', a) else var(a) for a in args) + ')'
    for kind, x in body:
        if kind in ('atom', 'not') and x[0] not in pred:
            raise Refused(f'an undeclared relation: {x[0]}')
    if head[0] not in pred:
        raise Refused(f'an undeclared relation: {head[0]}')
    out, bound, used = [], set(), set()
    for kind, (rel, args) in [b for b in body if b[0] == 'atom']:
        out.append(at(rel, args))
        bound |= {a for a in args if a != '_' and not re.fullmatch(r'-?\d+', a)}
        used.add(pred[rel])
    pending = [b for b in body if b[0] == 'cmp' and b[1][0] == '=']
    while pending:
        for b in pending:
            _, lhs, rhs = b[1]
            free = (variables(lhs) | variables(rhs)) - bound
            if not free:
                op = '=:=' if arithmetic(lhs) or arithmetic(rhs) else '='
                out.append(f'{term(lhs)} {op} {term(rhs)}')
            elif len(lhs) == 1 and free == {lhs[0]} or len(rhs) == 1 and free == {rhs[0]}:
                v, e = (lhs, rhs) if free == set(lhs) else (rhs, lhs)
                out.append(f'{term(v)} {"is" if arithmetic(e) else "="} {term(e)}')
                bound |= free
            else:
                continue
            pending.remove(b)
            break
        else:
            raise Refused(f'an equation whose variables nothing binds: {pending[0][1]}')
    for kind, x in body:
        if kind == 'not':
            if set(a for a in x[1] if a != '_' and not re.fullmatch(r'-?\d+', a)) - bound:
                raise Refused(f'a negation of {x[0]} over a variable nothing binds')
            out.append(f'\\+ {at(*x)}')
            used.add(pred[x[0]])
        elif kind == 'ncmp' or kind == 'cmp' and x[0] != '=':
            op, lhs, rhs = x
            if (variables(lhs) | variables(rhs)) - bound:
                raise Refused(f'a comparison over a variable nothing binds: {" ".join(lhs)} {op} {" ".join(rhs)}')
            ar = arithmetic(lhs) or arithmetic(rhs)
            if kind == 'ncmp':
                op = {'=': '!=', '!=': '=', '<': '>=', '<=': '>', '>': '<=', '>=': '<'}[op]
            pl = {'=': '=:=' if ar else '=', '!=': '=\\=' if ar else '\\=', '<': '<', '<=': '=<', '>': '>',
                  '>=': '>='}[op]
            out.append(f'{term(lhs)} {pl} {term(rhs)}')
    if set(a for a in head[1] if a != '_' and not re.fullmatch(r'-?\d+', a)) - bound:
        raise Refused(f'a head variable of {head[0]} that nothing binds')
    text = (f'{p}::' if p is not None else '') + at(*head) + (' :- ' + ', '.join(out) if out else '') + '.'
    return text, used


def main(argv):
    args = argv[1:]
    souffle = None
    if '--souffle' in args:
        i = args.index('--souffle')
        souffle = args[i + 1]
        del args[i:i + 2]
    if len(args) != 2:
        sys.exit(__doc__.split('\n\n')[1])
    program = read(args[0])
    try:
        text, rows, names, queries = translate(program, read(args[1]))
    except Refused as x:
        print(f'gen_sidechannel.py: {x}', file=sys.stderr)
        sys.exit(REFUSED)
    case = os.path.basename(args[0]).split('.')[0]
    sys.stdout.write(f'% PSouffle\'s side-channel case {case} (FMCAD 2026; MIT licence), from its\n'
                     '% compute.souffle.dl and the facts of its compute.problog.dl by gen_sidechannel.py.\n\n'
                     + text)
    if souffle:
        os.makedirs(os.path.join(souffle, 'input'), exist_ok=True)
        with open(os.path.join(souffle, 'compute.dl'), 'w') as f:
            f.write(program)
        for r, (facts, probs) in rows.items():
            for ext, lines in [('facts', facts), ('prob', probs)]:
                with open(os.path.join(souffle, 'input', f'{r}.{ext}'), 'w') as f:
                    f.write(''.join(line + '\n' for line in lines))
        with open(os.path.join(souffle, 'names.tsv'), 'w') as f:
            f.write(''.join(f'{r}\t{n}\t{a}\n' for r, n, a in names))
        with open(os.path.join(souffle, 'queries.txt'), 'w') as f:
            f.write(''.join(q + '\n' for q in queries))


if __name__ == '__main__':
    main(sys.argv)
