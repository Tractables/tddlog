#!/usr/bin/env python3
"""Convert a cplint/PITA LPAD program to ProbLog syntax.

The harness's copy: tddlog reads LPADs itself since this script left the
source tree (tddlog --program prints the same program); the generators
(gen_cplint.py, gen_uwcse.py, gen_pedigree.py, uwcse_exact.py) and the
converters that use its clause splitter import it from here.

    lpad2problog.py in.pl > out.pl

The LPAD is the clauses between `:- begin_lpad.` and `:- end_lpad.` (or
`begin_plp`/`end_plp`), the whole file when the markers are absent. Its
clauses `h1:p1 ; h2:p2 :- body.` are written `p1::h1; p2::h2 :- body.`, and
`a:p.` is written `p::a.`; a head without a probability takes what the
others leave. The clauses outside the markers are cplint's database
clauses, plain Prolog, which an LPAD body calls through `db(Goal)` or the
goal of a built-in such as `findall/3`: those the LPAD's bodies name, and
those they name in turn, are kept as they are, and `db(Goal)` is written
`Goal`. The rest (plots, samplers and other programs over the LPAD) is
left out, as are the directives. The example queries `?- prob(Q, P).` in
the file become `query(Q).`, and `?- prob(Q, E, P).` becomes `query(Q).`
and an `evidence(L).` for each literal L of the conjunction E. One
ProbLog program holds one set of evidence: the queries asked under the
evidence of the file's first example query are written, and the others
are named in a comment and on standard error, as is a query under an
intervention `do(L)`, which ProbLog does not read. Everything else is passed
through unchanged, so a program using features outside ProbLog's language
still fails, downstream and visibly.
"""
import math
import re
import sys


def strip_comments(text):
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.S)
    return '\n'.join(re.sub(r'(?<!\\)%.*$', '', line) for line in text.splitlines())


def split_clauses(text):
    """Split on `.` followed by whitespace, outside quotes and parentheses."""
    out, cur, depth, quote = [], [], 0, None
    i = 0
    while i < len(text):
        c = text[i]
        if quote:
            cur.append(c)
            if c == quote:
                quote = None
        elif c in "'\"":
            quote = c
            cur.append(c)
        elif c in '([':
            depth += 1
            cur.append(c)
        elif c in ')]':
            depth -= 1
            cur.append(c)
        elif c == '.' and depth == 0 and (i + 1 == len(text) or text[i + 1].isspace()):
            s = ''.join(cur).strip()
            if s:
                out.append(s)
            cur = []
        else:
            cur.append(c)
        i += 1
    s = ''.join(cur).strip()
    if s:
        out.append(s)
    return out


def split_top(s, sep):
    parts, cur, depth, quote = [], [], 0, None
    for c in s:
        if quote:
            cur.append(c)
            if c == quote:
                quote = None
            continue
        if c in "'\"":
            quote = c
        elif c in '([':
            depth += 1
        elif c in ')]':
            depth -= 1
        if c == sep and depth == 0:
            parts.append(''.join(cur))
            cur = []
        else:
            cur.append(c)
    parts.append(''.join(cur))
    return parts


def number(expr):
    expr = expr.strip()
    m = re.fullmatch(r'(\d+(?:\.\d+)?(?:[eE]-?\d+)?)\s*/\s*(\d+(?:\.\d+)?)', expr)
    if m:
        return float(m.group(1)) / float(m.group(2))
    return float(expr)


def head_prob(head):
    """`atom:p` -> (atom, p); `atom` -> (atom, None). The colon must be at top level."""
    parts = split_top(head, ':')
    if len(parts) == 2:
        try:
            return parts[0].strip(), number(parts[1])
        except ValueError:
            pass
    return head.strip(), None


def fmt(p):
    return repr(p) if p != int(p) else f'{p:.1f}'


def convert_clause(clause):
    if ':-' in clause:
        head, body = clause.split(':-', 1)
        body = ' ' + body.strip()
    else:
        head, body = clause, ''
    heads = [head_prob(h) for h in split_top(head, ';')]
    if all(p is None for _, p in heads):
        if len(heads) == 1:
            return f'{heads[0][0]}{" :-" + body if body else ""}.'
        raise ValueError('disjunctive head without probabilities')
    # `math.fsum`: see the note in scripts/gen_bn.py -- `sum` over floats
    # differs between Python versions and the remainder below is printed.
    total = math.fsum(p for _, p in heads if p is not None)
    missing = [i for i, (_, p) in enumerate(heads) if p is None]
    if len(missing) > 1:
        raise ValueError('several heads without probability')
    if missing:
        heads[missing[0]] = (heads[missing[0]][0], max(0.0, 1.0 - total))
    lhs = '; '.join(f'{fmt(p)}::{a}' for a, p in heads)
    return f'{lhs}{" :-" + body if body else ""}.'


def main():
    print(convert(open(sys.argv[1]).read()))


MARKER = re.compile(r':-\s*(begin|end)_(lpad|plp)\s*$')
NAME = re.compile(r'\b[a-z]\w*')
INTERVENTION = re.compile(r'do\s*\(')


def convert(text):
    """The ProbLog text of an LPAD program text."""
    raw = strip_comments(text)
    # (clause, in the LPAD) in the order of the file, directives and goals
    # left out.
    clauses = []
    inside = not any(MARKER.match(c) for c in split_clauses(raw))
    for c in split_clauses(raw):
        m = MARKER.match(c)
        if m:
            inside = m.group(1) == 'begin'
        elif not c.startswith((':-', '?-')):
            clauses.append((without_db(c) if inside else c, inside))
    # The database clauses the LPAD's bodies name, and those they name.
    named = set()
    for c, lpad in clauses:
        if lpad:
            named |= mentioned(body(c))
    kept = [lpad for _, lpad in clauses]
    grew = True
    while grew:
        grew = False
        for k, (c, lpad) in enumerate(clauses):
            if not kept[k] and head_name(c) in named:
                kept[k] = grew = True
                named |= mentioned(body(c))
    out = [convert_clause(c) for (c, _), keep in zip(clauses, kept) if keep]
    # One program holds one set of evidence: that of the first query not
    # asked under an intervention.
    asked = list(dict.fromkeys(examples(text)))
    observed = [e for _, e in asked if not any(INTERVENTION.match(lit) for lit in e)]
    under = observed[0] if observed else ()
    out += [f'evidence({lit}).' for lit in under]
    out += [f'query({q}).' for q, e in asked if e == under]
    for q, e in asked:
        if e != under:
            given = ', '.join(e)
            goal = f'prob({q}, ({given}), P)' if e else f'prob({q}, P)'
            why = 'an intervention' if e not in observed else 'other evidence'
            note = f'left out, asked under {why}: {goal}'
            out.append(f'% {note}')
            print(f'lpad2problog.py: {note}', file=sys.stderr)
    return '\n'.join(out)


def examples(text):
    """The example queries `?- prob(Q, P)` and `?- prob(Q, E, P)` of the
    text, in its order, each as (Q, the literals of the conjunction E)."""
    # They sit in `/** <examples> ... */` blocks, so they are read from the
    # raw text, comments and all.
    asked = []
    for line in text.splitlines():
        goal = line.strip()
        if not goal.startswith('?-'):
            continue
        m = re.search(r'\bprob\(', goal)
        if not m:
            continue
        args = [a.strip() for a in split_top(balanced(goal, m.end()), ',')]
        if len(args) == 2:
            asked.append((args[0], ()))
        elif len(args) == 3:
            e = tuple(lit.strip() for lit in split_top(unwrapped(args[1]), ','))
            asked.append((args[0], e))
    return asked


def unwrapped(s):
    """`s` without the parentheses around the whole of it."""
    while s.startswith('(') and len(balanced(s, 1)) == len(s) - 2:
        s = s[1:-1].strip()
    return s


def without_db(clause):
    """`clause` with each call `db(Goal)` of a database goal written `Goal`."""
    while (m := re.search(r'\bdb\(', clause)):
        goal = balanced(clause, m.end())
        clause = clause[:m.start()] + goal + clause[m.end() + len(goal) + 1:]
    return clause


def body(clause):
    return clause.split(':-', 1)[1] if ':-' in clause else ''


def head_name(clause):
    m = NAME.match(clause.split(':-', 1)[0].strip())
    return m.group(0) if m else None


def mentioned(text):
    """The names `text` mentions outside quotes, which hold those of the
    predicates it calls."""
    return set(NAME.findall(re.sub(r"'[^']*'|\"[^\"]*\"", '', text)))


def balanced(s, start):
    """The text from `start` up to the parenthesis closing the one before it."""
    depth = 1
    for i in range(start, len(s)):
        if s[i] in '([':
            depth += 1
        elif s[i] in ')]':
            depth -= 1
            if depth == 0:
                return s[start:i]
    return s[start:]


if __name__ == '__main__':
    main()
