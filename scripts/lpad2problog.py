#!/usr/bin/env python3
"""Convert a cplint/PITA LPAD program to ProbLog syntax.

    lpad2problog.py in.pl > out.pl

Takes the clauses between `:- begin_lpad.` and `:- end_lpad.` (the whole file
when the markers are absent), rewrites `h1:p1 ; h2:p2 :- body.` to
`p1::h1; p2::h2 :- body.` and `a:p.` to `p::a.`, drops directives, and turns
the example queries `?- prob(Q, P).` in the file into `query(Q).` (a
`prob(Q, E, P)` becomes `query(Q).` and `evidence(E).` when E is one atom).
Everything else is passed through unchanged, so a program using features
outside ProbLog's language still fails, downstream and visibly.
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


def convert(text):
    """The ProbLog text of an LPAD program text."""
    # The example queries sit in `/** <examples> ... */` blocks, so they are
    # collected from the raw text, before comments go.
    queries = []
    for line in text.splitlines():
        goal = line.strip()
        if not goal.startswith('?-'):
            continue
        m = re.search(r'\bprob\(', goal)
        if not m:
            continue
        args = split_top(balanced(goal, m.end()), ',')
        if len(args) == 2:
            queries.append(f'query({args[0].strip()}).')
        elif len(args) == 3:
            queries.append(f'evidence({args[1].strip()}).')
            queries.append(f'query({args[0].strip()}).')
    raw = strip_comments(text)
    clauses = split_clauses(raw)
    out = []
    inside = 'begin_lpad' not in raw
    for c in clauses:
        cs = c.strip()
        if cs.startswith('?-'):
            continue
        if cs.startswith(':-'):
            if 'begin_lpad' in cs:
                inside = True
            elif 'end_lpad' in cs:
                inside = False
            continue
        if not inside:
            continue
        out.append(convert_clause(cs))
    seen = set()
    for q in queries:
        if q not in seen:
            seen.add(q)
            out.append(q)
    return '\n'.join(out)


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
