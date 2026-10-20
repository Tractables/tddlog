#!/usr/bin/env python3
"""Convert a ProbLog program to a cplint/PITA LPAD.

    problog2pita.py in.pl > out.pl
    problog2pita.py in.pl --driver > out.pl   # with a main/0 that asks every query

The inverse of `lpad2problog.py`, whose clause splitter it uses: `p::a.`
becomes `a:p.`, `p1::h1 ; p2::h2 :- Body.` becomes `h1:p1 ; h2:p2 :- Body.`,
and the `query/1` and `evidence/1,2` directives leave the program and become
the arguments of PITA's `prob/2` and `prob/3`. Everything else is passed
through: PITA reads Prolog, so bodies, negation and comparisons need no
translation.

Evidence is one conjunction for the whole program, as ProbLog's is, and
`evidence(a, false)` becomes `\\+ a`. A program with evidence is asked with
`prob(Query, Evidence, P)`, which is PITA's conditional probability.
"""
import argparse
import re
import sys
from fractions import Fraction

from lpad2problog import split_clauses, split_top, strip_comments


def split_neck(clause):
    """(head, body) at the first `:-` outside quotes and parentheses, or
    (clause, None) when there is none."""
    depth, quote, i = 0, None, 0
    while i < len(clause):
        c = clause[i]
        if quote:
            if c == quote:
                quote = None
        elif c in "'\"":
            quote = c
        elif c in '([':
            depth += 1
        elif c in ')]':
            depth -= 1
        elif c == ':' and depth == 0 and clause[i + 1:i + 2] == '-':
            return clause[:i], clause[i + 2:]
        i += 1
    return clause, None


def head_disjuncts(head):
    """`p1::h1 ; p2::h2` -> `h1:p1 ; h2:p2`. A disjunct without a probability
    is passed through, which is how a rule with a plain head stays plain."""
    out = []
    for d in split_top(head, ';'):
        d = d.strip()
        parts = split_top(d, ':')
        # `p::a` splits into three pieces with an empty one in the middle.
        if len(parts) == 3 and parts[1] == '':
            out.append(f'{parts[2].strip()}:{parts[0].strip()}')
        else:
            out.append(d)
    return ' ; '.join(out)


def certain_fact(head):
    """`a:1` -> `a`: a fact of probability one written as a plain fact. PITA
    (cplint 2026-09, SWI-Prolog 9.2.9) reads the fact `a:1.` as false, so a
    path through an edge of probability one was dropped: on the Biomine toy
    program (`tests/lit/biomine.pl`) PITA gave 0.297 for a query whose
    answer is 0.930, and 0 for every query of the Biomine suite. A rule
    `a:1 :- Body.` it reads right."""
    parts = split_top(head, ':')
    if len(parts) == 2 and ';' not in head:
        try:
            if float(Fraction(parts[1].strip())) == 1.0:
                return parts[0].strip()
        except (ValueError, ZeroDivisionError):
            pass
    return head


def functor(term):
    """`p(a,b)` -> `p/2`, `p` -> `p/0`, and None for anything that is not a
    plain compound term (a comparison, a number, a variable)."""
    term = term.strip()
    m = re.match(r'^([a-z][a-zA-Z0-9_]*)\s*(\(|$)', term)
    if not m:
        return None
    if not m.group(2):
        return f'{term}/0'
    args = term[m.end(1):].strip()
    if not (args.startswith('(') and args.endswith(')')):
        return None
    return f"{m.group(1)}/{len(split_top(args[1:-1], ','))}"


def body_functors(body):
    """The predicates a body calls, negation and disjunction seen through."""
    out = []
    for conjunct in split_top(body, ','):
        for goal in split_top(conjunct, ';'):
            goal = goal.strip()
            while goal.startswith('\\+'):
                goal = goal[2:].strip()
            f = functor(goal)
            if f:
                out.append(f)
    return out


def recursive(rules):
    """The predicates that can reach themselves through `rules`, which is
    what PITA wants tabled: without a `table` declaration a cyclic program
    loops, and the smokers programs are cyclic by construction."""
    edges = {}
    for head, body in rules:
        edges.setdefault(head, set()).update(body)
    out = set()
    for start in edges:
        seen, stack = set(), list(edges[start])
        while stack:
            p = stack.pop()
            if p in seen:
                continue
            seen.add(p)
            stack += list(edges.get(p, ()))
        if start in seen:
            out.add(start)
    return out


def translate(text):
    """(clauses, queries, evidence) of a ProbLog program, as PITA wants them.

    `clauses` are LPAD clauses without the `begin_lpad` markers, led by a
    `table` declaration for every predicate that can reach itself, `queries`
    the query terms in program order, `evidence` a list of goals to
    conjoin."""
    clauses, queries, evidence, rules, facts = [], [], [], [], {}
    for clause in split_clauses(strip_comments(text)):
        clause = ' '.join(clause.split())
        if clause.startswith(':-'):
            continue  # a directive of the ProbLog program, not of the LPAD
        head, body = split_neck(clause)
        m = re.fullmatch(r'query\((.*)\)', head.strip())
        if m:
            # `query(Q) :- Body.` asks Q for every solution of Body over the
            # deterministic part. PITA has no such directive, so the driver
            # enumerates the body itself over a plain copy of the facts it
            # reads (`generators` below).
            queries.append((m.group(1).strip(), body.strip() if body else None))
            continue
        m = re.fullmatch(r'evidence\((.*)\)', clause)
        if m:
            parts = split_top(m.group(1), ',')
            if len(parts) > 1 and parts[-1].strip() in ('true', 'false'):
                atom, truth = ','.join(parts[:-1]).strip(), parts[-1].strip()
            else:
                atom, truth = m.group(1).strip(), 'true'
            evidence.append(atom if truth == 'true' else f'\\+ ({atom})')
            continue
        if body is None and ':' not in split_top(head, ':')[0:1][0][:0] + '':
            pass
        if body is not None:
            for d in split_top(head, ';'):
                parts = split_top(d, ':')
                f = functor(parts[0] if len(parts) != 3 or parts[1] != '' else parts[2])
                if f:
                    rules.append((f, body_functors(body)))
        if body is None and len(split_top(head, ':')) == 1:
            f = functor(head)
            if f:
                facts.setdefault(f.split('/')[0], []).append(head.strip())
        head = head_disjuncts(head).strip()
        if body is None:
            head = certain_fact(head)
        clauses.append(f'{head}.' if body is None else f'{head} :- {body.strip()}.')
    tables = [f':- table {p}.' for p in sorted(recursive(rules))]
    return tables + clauses, queries, evidence, facts


GEN = 'query_gen_'


def generator(body, facts):
    """(renamed body, copies of the facts it reads) for the body of a
    `query(Q) :- Body.` directive. The copies sit outside the LPAD under a
    prefixed name: PITA rewrites every predicate inside the block, arguments
    and all, and the driver only needs to enumerate ground facts."""
    copies, renamed = [], body
    for f in dict.fromkeys(body_functors(body)):
        name = f.split('/')[0]
        if name not in facts:
            continue
        copies += [re.sub(rf'^{re.escape(name)}\b', GEN + name, fact) + '.' for fact in facts[name]]
        renamed = re.sub(rf'\b{re.escape(name)}\s*\(', GEN + name + '(', renamed)
    return renamed, copies


def program(text, driver=False):
    clauses, queries, evidence, facts = translate(text)
    lines = [':- use_module(library(pita)).', ':- pita.', ':- begin_lpad.']
    lines += clauses
    lines.append(':- end_lpad.')
    if driver:
        goal = 'prob(Q, P)' if not evidence else f"prob(Q, ({', '.join(evidence)}), P)"
        ask = f'({goal}, format("~q:\\t~w~n", [Q, P]))'
        enumerators = []
        for term, body in queries:
            if body is None:
                enumerators.append(f'Q = {term}')
            else:
                renamed, copies = generator(body, facts)
                lines += copies
                enumerators.append(f'{renamed}, Q = {term}')
        lines.append('main :-')
        lines += [f'    forall(({e}), {ask}),' for e in enumerators[:-1]]
        lines.append(f'    forall(({enumerators[-1]}), {ask}).' if enumerators else '    true.')
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('file')
    ap.add_argument('--driver', action='store_true', help='add a main/0 that asks every query')
    args = ap.parse_args()
    sys.stdout.write(program(open(args.file).read(), args.driver))


if __name__ == '__main__':
    main()
