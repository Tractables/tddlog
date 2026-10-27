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

import source_tree  # noqa: F401  (the source tree's converters)
from lpad2problog import split_clauses, split_top, strip_comments


# What PITA keeps for itself (cplint 4.5.0, SWI-Prolog 9.2.9): the
# predicates its user module defines before any clause of the program,
# PITA's exports, and `db/1`, which PITA reads in a body as a call to plain
# Prolog. PITA adds two arguments to every predicate of the program, so a
# program's `v/1` is PITA's own `v/3`, and a program's `db(X)` is not a
# predicate at all: on Roulette's HMM programs, which name their
# observations v/1, PITA answered 0 for every length, and on its caesar
# programs, which name a bit db/2, it printed nothing. These names are
# renamed with a prefix, and the driver prints the queries without it.
RESERVED = frozenset((
    'v av dec db goal_n rule_n tabled zero_clauses local_pita_setting rule_by_num query_rule expand_query '
    'expand_answer pita_on exception resource message_hook thread_message_hook portray portray_message '
    'prolog_list_goal prolog_load_file library_directory file_search_path prolog_file_type message_property '
    'term_expansion goal_expansion '
    # the exports of library(pita)
    'probability_dd end_ex or map get_abd_var_n dt_bdd_json_file std_dev to_atom symmetric_dirichlet_sample '
    'init_em gamma_sample prob_meta key_pair create_dot_string andcnf density equalityc histogram add_var '
    'compute_optimal_strategy_sp bar dt_solve_sp swi_builtin bdd_not add_decision_var set_pita get_dec_var_n '
    'variance one dirichlet_sample beta abd_bdd_dot_string rand_seed init_ex abd_prob prob to_pair create_dot '
    'or_list load_file bdd_notc add_abd_var ret_strategy onec parse ret_prob end load initial_values '
    'setting_pita agg_val zero add_sum em bin bdd_dot_file init uniform_sample or_listc density2d densities '
    'orc bar1 zeroc msw ret_abd_prob get_var_n dt_solve and discrete_sample equality add_prod gauss_sample '
    'average create_json_util end_em map_bdd_dot_string bdd_dot_string value_pair make_query_var ret_probc '
    'argbar add_query_var debug_cudd_var andc').split())
PREFIX = 'problog2pita_'
_NAME = re.compile(r"(?<![\w$'])([a-z][a-zA-Z0-9_]*)(?![\w'])")


def rename(text):
    """`text` with every atom that PITA keeps for itself (RESERVED) given
    PREFIX, and whether there was one. Atoms are renamed wherever they
    appear, as predicates and as constants alike, which keeps the program's
    meaning; the driver strips PREFIX from the queries it prints."""
    hit = False

    def sub(m):
        nonlocal hit
        if m.group(1) in RESERVED:
            hit = True
            return PREFIX + m.group(1)
        return m.group(1)
    return _NAME.sub(sub, text), hit


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
    loops, and the smokers programs are cyclic by construction. They are
    the strongly connected components with a cycle (Tarjan's algorithm,
    iterative), found in time linear in the program: a circuit's program
    has a predicate per gate, hundreds of thousands of them."""
    edges = {}
    for head, body in rules:
        edges.setdefault(head, set()).update(body)
    index, low, stack, on, out = {}, {}, [], set(), set()

    def visit(p):
        index[p] = low[p] = len(index)
        stack.append(p)
        on.add(p)
        return p, iter(edges.get(p, ()))

    for root in edges:
        if root in index:
            continue
        work = [visit(root)]
        while work:
            p, succ = work[-1]
            q = next(succ, None)
            if q is not None:
                if q not in index:
                    work.append(visit(q))
                elif q in on:
                    low[p] = min(low[p], index[q])
                continue
            work.pop()
            if work:
                low[work[-1][0]] = min(low[work[-1][0]], low[p])
            if low[p] == index[p]:
                comp = []
                while not comp or comp[-1] != p:
                    comp.append(stack.pop())
                    on.discard(comp[-1])
                if len(comp) > 1 or p in edges.get(p, ()):
                    out.update(comp)
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
    text, renamed = rename(text)
    clauses, queries, evidence, facts = translate(text)
    lines = [':- use_module(library(pita)).', ':- pita.', ':- begin_lpad.']
    lines += clauses
    lines.append(':- end_lpad.')
    if driver:
        goal = 'prob(Q, P)' if not evidence else f"prob(Q, ({', '.join(evidence)}), P)"
        show = 'format("~q:\\t~w~n", [Q, P])'
        if renamed:
            # one argument of forall/2
            show = (f"(term_to_atom(Q, Q0), atomic_list_concat(Qs, '{PREFIX}', Q0), atomic_list_concat(Qs, Q1), "
                    'format("~w:\\t~w~n", [Q1, P]))')
        enumerators = []
        for term, body in queries:
            if body is None:
                enumerators.append(f'Q = {term}')
            else:
                renamed, copies = generator(body, facts)
                lines += copies
                enumerators.append(f'{renamed}, Q = {term}')
        lines.append('main :-')
        # prob/2 backtracks over the instances of a query with variables:
        # each is printed, as ProbLog asks every instance of such a query.
        lines += [f'    forall(({e}, {goal}), {show}),' for e in enumerators[:-1]]
        lines.append(f'    forall(({enumerators[-1]}, {goal}), {show}).' if enumerators else '    true.')
    return '\n'.join(lines) + '\n'


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('file')
    ap.add_argument('--driver', action='store_true', help='add a main/0 that asks every query')
    args = ap.parse_args()
    sys.stdout.write(program(open(args.file).read(), args.driver))


if __name__ == '__main__':
    main()
