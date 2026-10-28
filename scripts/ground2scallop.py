#!/usr/bin/env python3
"""Convert a ground ProbLog program, as `tddlog --ground` prints it, to Scallop.

    tddlog --ground prog.pl | ground2scallop.py > prog.scl

The input is ground and its probabilities sit on independent facts only:
`--ground` has already turned every annotated disjunction and every
probabilistic rule into switch facts and plain rules. That matters for
Scallop, whose rule-level probabilities are not ProbLog's (a probability on
a rule is a fresh random variable per derivation there, `docs/baselines.md`),
so a translation from the source program would change the answers.

Every relation is renamed `r_<name>` so that no ProbLog predicate collides
with a Scallop keyword, every constant becomes a string, and every relation
gets a `type` declaration so that one with no facts still exists. Each query
atom becomes a nullary relation `q<i>`, and with evidence two more, `e()`
and `qe<i>()` for the query and the evidence together: Scallop does not
condition, so the harness divides. The query names are listed in a comment
at the top, `// query q<i> <atom>`, which is how the harness maps them back.
"""
import re
import sys

import source_tree  # noqa: F401  (the source tree's converters)
from lpad2problog import split_clauses, split_top, strip_comments
from problog2pita import split_neck


def name(functor):
    if functor.startswith("'") and functor.endswith("'"):
        functor = functor[1:-1]
    return 'r_' + re.sub(r'[^A-Za-z0-9_]', '_', functor)


def const(c):
    c = c.strip()
    if c.startswith("'") and c.endswith("'"):
        c = c[1:-1]
    return '"' + c.replace('\\', '\\\\').replace('"', '\\"') + '"'


def atom(text, arities):
    """A ground atom as a Scallop atom, recording its relation's arity."""
    text = text.strip()
    m = re.match(r"^('(?:[^']|\\')*'|[^(]+)\s*(\((.*)\))?$", text)
    if not m:
        raise ValueError(f'not a ground atom: {text}')
    rel = name(m.group(1).strip())
    args = [const(a) for a in split_top(m.group(3), ',')] if m.group(2) else []
    # A name of two arities is two predicates (hmm/1 and hmm/4): the one
    # met second is named with its arity.
    if arities.setdefault(rel, len(args)) != len(args):
        rel = f'{rel}__{len(args)}'
        arities.setdefault(rel, len(args))
    return f"{rel}({', '.join(args)})"


def literal(text, arities):
    text = text.strip()
    if text.startswith('\\+'):
        return 'not ' + atom(text[2:], arities)
    return atom(text, arities)


def translate(text):
    arities, facts, rules, queries, evidence = {}, [], [], [], []
    for clause in split_clauses(strip_comments(text)):
        clause = ' '.join(clause.split())
        m = re.fullmatch(r'query\((.*)\)', clause)
        if m:
            queries.append(m.group(1).strip())
            continue
        m = re.fullmatch(r'evidence\((.*)\)', clause)
        if m:
            parts = split_top(m.group(1), ',')
            truth = parts[-1].strip() if len(parts) > 1 else 'true'
            lit = ','.join(parts[:-1]) if len(parts) > 1 else parts[0]
            evidence.append(literal(lit, arities) if truth == 'true' else 'not ' + atom(lit, arities))
            continue
        head, body = split_neck(clause)
        parts = split_top(head, ':')
        if len(parts) == 3 and parts[1] == '':
            if body is not None:
                raise ValueError(f'a probabilistic rule in a ground program: {clause}')
            facts.append(f'rel {float(parts[0]):.17g}::{atom(parts[2], arities)}')
        elif body is None:
            facts.append(f'rel {atom(head, arities)}')
        else:
            lits = [literal(b, arities) for b in split_top(body, ',')]
            rules.append(f"rel {atom(head, arities)} = {', '.join(lits)}")
    out = [f'// query q{i} {q}' for i, q in enumerate(queries)]
    qs = [(f'q{i}', atom(q, arities)) for i, q in enumerate(queries)]
    out += [f"type {r}({', '.join(['String'] * n)})" for r, n in sorted(arities.items())]
    out += ['type ' + q + '()' for q, _ in qs]
    out += facts + rules
    for q, a in qs:
        out.append(f'rel {q}() = {a}')
    if evidence:
        out += ['type e()', f"rel e() = {', '.join(evidence)}"]
        for q, a in qs:
            out += [f'type {q}e()', f"rel {q}e() = {a}, {', '.join(evidence)}"]
        out += ['query e'] + [f'query {q}e' for q, _ in qs]
    else:
        out += [f'query {q}' for q, _ in qs]
    return '\n'.join(out) + '\n'


def main():
    sys.stdout.write(translate(open(sys.argv[1]).read() if len(sys.argv) > 1 else sys.stdin.read()))


if __name__ == '__main__':
    main()
