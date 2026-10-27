#!/usr/bin/env python3
"""A TPC-H program of gen_tpch.py, a probabilistic database of gen_pdb.py or a
noisy-or network of gen_noisyor.py (any ProbLog program of probabilistic and
deterministic facts, non-recursive conjunctive rules with comparisons,
`query(atom)` lines and `evidence(atom, true|false)` lines over ground
atoms) as SQL for ProvSQL, the PostgreSQL extension.

usage: tpch2provsql.py PROGRAM.pl OUT.sql

OUT.sql creates a table per fact predicate (columns c1, c2, ..., and `prob`
where the facts carry a probability), loads the facts, gives the tables
with probabilities a provenance token and the probability (`add_provenance`,
`set_prob`), and creates a view per rule predicate (a UNION of the rule's
clauses, SELECT DISTINCT for the set semantics). A query of a predicate
without arguments q is then `SELECT probability_evaluate(provenance()) FROM
q`. A ground atom p(a, b) is the Boolean view `pq_atomK`, the rows of p with
c1 = a and c2 = b. Evidence is the view `pq_ev`: the product of its true
atoms' views (a conjunction) EXCEPT the union of its false atoms' (ProvSQL's
monus, a negation in the Boolean semiring); under evidence a query q is the
view `pq_qK` of q's atom times the evidence's, and its answer P(q | e) is
P(pq_qK) / P(pq_ev). The script prints one `name<TAB>SQL` line per query
atom, the SQL of P(q and e), and `evidence<TAB>SQL` for P(e).
"""
import re
import sys

FACT = re.compile(r'^(?:([0-9.eE+-]+)::)?(\w+)(?:\((.*)\))?\.\s*$')
OPS = {'=<': '<=', '>=': '>=', '<': '<', '>': '>', '\\=': '<>', '=:=': '=', '=': '='}
COMPARISON = re.compile(r'^(.+?)\s*(=<|>=|=:=|\\=|<|>|=)\s*(.+)$')


def split_top(text, sep=','):
    """Split at the separators outside parentheses and quotes."""
    out, depth, quote, cur = [], 0, False, ''
    for ch in text:
        if ch == "'":
            quote = not quote
        elif not quote and ch == '(':
            depth += 1
        elif not quote and ch == ')':
            depth -= 1
        if ch == sep and depth == 0 and not quote:
            out.append(cur.strip())
            cur = ''
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def term(t):
    """('var', name), ('anon',), ('int', n) or ('text', s)."""
    t = t.strip()
    if t == '_':
        return ('anon',)
    if re.fullmatch(r'-?\d+', t):
        return ('int', int(t))
    if t[0] == "'" and t[-1] == "'":
        return ('text', t[1:-1])
    if t[0].isupper() or t[0] == '_':
        return ('var', t)
    return ('text', t)


GROUND = r'(\w+)(?:\((.*?)\))?'


def ground(pred, args):
    """(the atom as tddlog prints it, predicate, argument terms) of a ground atom."""
    terms = [term(a) for a in split_top(args)] if args else []
    if any(t[0] in ('var', 'anon') for t in terms):
        sys.exit(f'not a ground atom: {pred}({args})')
    return (f'{pred}({",".join(a.strip() for a in split_top(args))})' if args else pred), pred, terms


def parse(text):
    """The facts (by predicate: probability or None, arguments), the rules
    (head predicate, head terms, body), the query atoms and the evidence
    ((atom, truth value)) of a program, each atom (its text, predicate,
    argument terms)."""
    facts, rules, queries, evidence = {}, [], [], []
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith('%'):
            continue
        if ':-' in line:
            head, body = line.rstrip('.').split(':-', 1)
            m = re.fullmatch(r'(\w+)(?:\((.*)\))?', head.strip())
            hargs = [term(a) for a in split_top(m.group(2))] if m.group(2) else []
            items = []
            for b in split_top(body):
                m2 = re.fullmatch(r'(\w+)\((.*)\)', b)
                if m2:
                    items.append(('atom', m2.group(1), [term(a) for a in split_top(m2.group(2))]))
                elif re.fullmatch(r'[a-z]\w*', b):
                    items.append(('atom', b, []))   # an atom of no arguments (liftsep's h30, rgraph's)
                else:
                    c = COMPARISON.match(b)
                    items.append(('cmp', OPS[c.group(2)], term(c.group(1)), term(c.group(3))))
            rules.append((m.group(1), hargs, items))
            continue
        m = re.fullmatch(r'query\(' + GROUND + r'\)\.', line)
        if m:
            queries.append(ground(*m.groups()))
            continue
        m = re.fullmatch(r'evidence\(' + GROUND + r'(?:,\s*(true|false))?\)\.', line)
        if m:
            evidence.append((ground(m.group(1), m.group(2)), m.group(3) != 'false'))
            continue
        m = FACT.match(line)
        if not m:
            sys.exit(f'not a fact, rule, query or evidence: {line}')
        prob, pred, args = m.groups()
        facts.setdefault(pred, []).append((None if prob is None else prob, [term(a) for a in split_top(args or '')]))
    return facts, rules, queries, evidence


def lit(t):
    return str(t[1]) if t[0] == 'int' else "'" + t[1].replace("'", "''") + "'"


def clause_sql(head_terms, body, view_of):
    """SELECT DISTINCT of one rule clause; `view_of` gives each predicate's
    arity (columns c1..)."""
    tables, where, bound = [], [], {}
    for item in body:
        if item[0] != 'atom':
            continue
        alias = f't{len(tables)}'
        tables.append(f'{item[1]} {alias}')
        for j, t in enumerate(item[2], 1):
            col = f'{alias}.c{j}'
            if t[0] == 'var':
                if t[1] in bound:
                    where.append(f'{col} = {bound[t[1]]}')
                else:
                    bound[t[1]] = col
            elif t[0] != 'anon':
                where.append(f'{col} = {lit(t)}')
    for item in body:
        if item[0] == 'cmp':
            _, op, a, b = item
            side = [bound[x[1]] if x[0] == 'var' else lit(x) for x in (a, b)]
            where.append(f'{side[0]} {op} {side[1]}')
    cols = [bound[t[1]] if t[0] == 'var' else lit(t) for t in head_terms] or ['1']
    sel = ', '.join(f'{c} AS c{i}' for i, c in enumerate(cols, 1))
    return f'SELECT DISTINCT {sel} FROM {", ".join(tables)}' + (f' WHERE {" AND ".join(where)}' if where else '')


def atom_view(pred, terms):
    """The rows of `pred` whose columns are the ground terms, as a Boolean view's SELECT."""
    where = [f't0.c{j} = {lit(t)}' for j, t in enumerate(terms, 1)]
    return 'SELECT DISTINCT 1 AS c1 FROM ' + pred + ' t0' + (f' WHERE {" AND ".join(where)}' if where else '')


def translate(text):
    """(the SQL that loads the program, [(query, SQL of the probability of the
    query and the evidence)], the SQL of the evidence's probability or None
    without evidence)."""
    facts, rules, queries, evidence = parse(text)
    names = set(facts) | {head for head, _, _ in rules}
    if any(n.startswith('pq_') for n in names):
        sys.exit('a predicate named pq_*: the names of the Boolean views')
    out = []
    for pred, rows in facts.items():
        types = ['bigint' if a[0] == 'int' else 'text' for a in rows[0][1]]
        probabilistic = rows[0][0] is not None
        cols = [f'c{i} {t}' for i, t in enumerate(types, 1)] + (['prob float8'] if probabilistic else [])
        out.append(f'CREATE TABLE {pred} ({", ".join(cols)});')
        out.append(f'COPY {pred} FROM STDIN;')
        for p, args in rows:
            out.append('\t'.join([str(a[1]) for a in args] + ([p] if probabilistic else [])))
        out.append('\\.')
        if probabilistic:
            out.append(f"SELECT add_provenance('{pred}');")
            out.append(f'SELECT set_prob(provenance(), prob) FROM {pred};')
    by_head = {}
    for head, hargs, body in rules:
        by_head.setdefault(head, []).append((hargs, body))
    done = set(facts)

    def create(head):
        if head in done:
            return
        done.add(head)
        for _, body in by_head[head]:
            for item in body:
                if item[0] == 'atom' and item[1] in by_head:
                    create(item[1])
        out.append(f'CREATE VIEW {head} AS ' + ' UNION '.join(clause_sql(h, b, None) for h, b in by_head[head]) + ';')
    for head in by_head:
        create(head)
    for (_, pred, _), _ in evidence:
        if pred not in names:
            sys.exit(f'evidence on {pred}, which no fact or rule defines')
    prob = 'SELECT probability_evaluate(provenance()) FROM {}'.format
    atoms = {}

    def view(atom):
        """The Boolean view of a ground atom, created once."""
        text, pred, terms = atom
        if text not in atoms:
            atoms[text] = f'pq_atom{len(atoms)}'
            out.append(f'CREATE VIEW {atoms[text]} AS {atom_view(pred, terms)};')
        return atoms[text]
    if not evidence:
        return '\n'.join(out) + '\n', [(q, prob(q if not terms else view((q, pred, terms))))
                                       for q, pred, terms in queries], None
    true = [view(a) for a, v in evidence if v]
    false = [view(a) for a, v in evidence if not v]
    if not true:
        sys.exit('evidence with no true atom: the conjunction would need a row without provenance')
    if false:
        out.append('CREATE VIEW pq_evfalse AS ' + ' UNION '.join(f'SELECT c1 FROM {v}' for v in false) + ';')
    minus = ' EXCEPT SELECT c1 FROM pq_evfalse' if false else ''

    def conj(vs):
        return 'SELECT DISTINCT 1 AS c1 FROM ' + ', '.join(f'{v} e{k}' for k, v in enumerate(vs)) + minus
    out.append(f'CREATE VIEW pq_ev AS {conj(true)};')
    qs = []
    for k, q in enumerate(queries):
        out.append(f'CREATE VIEW pq_q{k} AS {conj([view(q)] + true)};')
        qs.append((q[0], prob(f'pq_q{k}')))
    return '\n'.join(out) + '\n', qs, prob('pq_ev')


def main():
    sql, qs, ev = translate(open(sys.argv[1]).read())
    open(sys.argv[2], 'w').write(sql)
    for q, s in qs:
        print(f'{q}\t{s}')
    if ev:
        print(f'evidence\t{ev}')


if __name__ == '__main__':
    main()
