#!/usr/bin/env python3
"""Convert a ground ProbLog program, as `tddlog --ground` prints it, to Scallop.

    tddlog --ground prog.pl | ground2scallop.py > prog.scl

It is Scallop's reading of the circuits Alea.jl exports, an evaluation of
their own that every system reads as the circuit or a translation of it:
a circuit has no first-order source, and its ground program is the
circuit, a relation per gate. Every other program is written first order,
by `problog2scallop.py`.

The input is ground and its probabilities sit on independent facts only:
`--ground` has already turned every annotated disjunction and every
probabilistic rule into switch facts and plain rules, so Scallop's
rule-level probabilities (a fresh random variable per derivation,
`docs/baselines.md`) never arise.

Every relation is renamed `r_<name>` so that no ProbLog predicate collides
with a Scallop keyword, every constant becomes a string, and every relation
gets a `type` declaration so that one with no facts still exists. Each query
atom becomes a nullary relation `q<i>`, and with evidence two more, `e()`
and `qe<i>()` for the query and the evidence together: Scallop does not
condition, so the harness divides. The query names are listed in a comment
at the top, `// query q<i> <atom>`, which is how the harness maps them back.

Scallop stratifies negation by relation, and refuses a program where a
relation reads its own negation through its rules ("Cannot stratify
program"), although the ground atoms may form no such cycle: a loop's step
`s(2) :- \\+ s(1), ...` reads the step before it. Every relation on a cycle
of the relations' dependencies that runs through a negation is therefore
written atom by atom, each of its ground atoms a nullary relation of its
own (`r_s__g<k>()`), so that Scallop's relations depend on each other as the
ground atoms do. The program means the same: its rules are ground, and the
split is a renaming of their atoms.
"""
import re
import sys
from decimal import Decimal

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
    """A ground atom as its relation and its arguments, Scallop constants,
    recording the relation's arity."""
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
    return rel, tuple(args)


def literal(text, arities):
    """A body literal as (negated, atom)."""
    text = text.strip()
    if text.startswith('\\+'):
        return True, atom(text[2:], arities)
    return False, atom(text, arities)


def negative_cycles(rules):
    """The relations on a cycle of the dependencies of rules' heads on their
    bodies that runs through a negation: every relation of a strongly
    connected component inside which a body literal is negated."""
    graph = {}
    for (head, _), body in rules:
        graph.setdefault(head, set()).update(rel for _, (rel, _) in body)
    # Tarjan's components, iteratively (a long loop is a deep recursion)
    index, low, comp, stack, on, n = {}, {}, {}, [], set(), 0
    for root in list(graph):
        if root in index:
            continue
        work = [(root, iter(graph.get(root, ())))]
        index[root] = low[root] = n
        n += 1
        stack.append(root)
        on.add(root)
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
                index[w] = low[w] = n
                n += 1
                stack.append(w)
                on.add(w)
                work.append((w, iter(graph.get(w, ()))))
            elif w in on:
                low[v] = min(low[v], index[w])
    cyclic = {comp[head] for (head, _), body in rules for neg, (rel, _) in body if neg and comp[head] == comp[rel]}
    return {r for r, c in comp.items() if c in cyclic}



def probability(text):
    """A fact's probability as Scallop reads it: a decimal without an exponent
    (Scallop's grammar has no `1e-08`), of the fewest digits that give the
    same float back."""
    return format(Decimal(repr(float(text))), 'f')

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
            neg, a = literal(lit, arities)
            evidence.append((neg != (truth != 'true'), a))
            continue
        head, body = split_neck(clause)
        parts = split_top(head, ':')
        if len(parts) == 3 and parts[1] == '':
            if body is not None:
                raise ValueError(f'a probabilistic rule in a ground program: {clause}')
            facts.append((probability(parts[0]) + '::', atom(parts[2], arities)))
        elif body is None:
            facts.append(('', atom(head, arities)))
        else:
            lits = [literal(b, arities) for b in split_top(body, ',')]   # the body first, as before
            rules.append((atom(head, arities), lits))
    qatoms = [atom(q, arities) for q in queries]
    split, names = negative_cycles(rules), {}

    def show(a):
        """An atom as Scallop reads it: of a split relation, its own nullary relation."""
        rel, args = a
        if rel not in split:
            return f"{rel}({', '.join(args)})"
        if a not in names:
            k = len(names)
            while f'{rel}__g{k}' in arities:
                k += 1
            names[a] = f'{rel}__g{k}'
        return names[a] + '()'

    shown = lambda lits: ', '.join(('not ' if neg else '') + show(a) for neg, a in lits)
    body = [f'rel {p}{show(a)}' for p, a in facts] + [f'rel {show(h)} = {shown(b)}' for h, b in rules]
    qs = [(f'q{i}', show(a)) for i, a in enumerate(qatoms)]
    ev = shown(evidence)
    out = [f'// query q{i} {q}' for i, q in enumerate(queries)]
    out += [f"type {r}({', '.join(['String'] * n)})" for r, n in sorted(arities.items()) if r not in split]
    out += [f'type {r}()' for r in sorted(names.values())]
    out += ['type ' + q + '()' for q, _ in qs]
    out += body
    for q, a in qs:
        out.append(f'rel {q}() = {a}')
    if evidence:
        out += ['type e()', f'rel e() = {ev}']
        for q, a in qs:
            out += [f'type {q}e()', f'rel {q}e() = {a}, {ev}']
        out += ['query e'] + [f'query {q}e' for q, _ in qs]
    else:
        out += [f'query {q}' for q, _ in qs]
    return '\n'.join(out) + '\n'


def answers(out, scl):
    """The answers of Scallop's output `out` on the program `scl` that
    `translate` wrote: scli prints `<relation>: {<p>::()}` for a nullary
    relation, `{}` when it has no proof. The query names map back through
    the comment lines `// query q<i> ATOM` of `scl`, and with evidence the
    answer is `q<i>e / e`, 'inconsistent' where e has no proof."""
    names = dict(re.findall(r'^// query (q\d+) (.*)$', scl, re.M))
    probs = {}
    for line in out.splitlines():
        m = re.match(r'^(\w+): \{(?:([0-9.eE+-]+)::\(\))?\}\s*$', line)
        if m:
            probs[m.group(1)] = float(m.group(2)) if m.group(2) else 0.0
    if probs.get('e', 1) == 0:
        return 'inconsistent'
    res = {}
    for q, atom in names.items():
        key = re.sub(r'\s+', '', atom)
        if 'e' in probs:
            res[key] = probs[q + 'e'] / probs['e'] if probs['e'] else float('nan')
        elif q in probs:
            res[key] = probs[q]
    return res


def main():
    sys.stdout.write(translate(open(sys.argv[1]).read() if len(sys.argv) > 1 else sys.stdin.read()))


if __name__ == '__main__':
    main()
