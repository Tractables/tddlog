#!/usr/bin/env python3
"""A TPC-H program of gen_tpch.py (or any ProbLog program of probabilistic and
deterministic facts, non-recursive conjunctive rules with comparisons, and
`query(atom)` lines) as SQL for ProvSQL, the PostgreSQL extension.

usage: tpch2provsql.py PROGRAM.pl OUT.sql

OUT.sql creates a table per fact predicate (columns c1, c2, ..., and `prob`
where the facts carry a probability), loads the facts, gives the tables
with probabilities a provenance token and the probability (`add_provenance`,
`set_prob`), and creates a view per rule predicate (a UNION of the rule's
clauses, SELECT DISTINCT for the set semantics). A query q is then
`SELECT probability_evaluate(provenance()) FROM q`; the script prints one
`name<TAB>SQL` line per query atom.
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


def parse(text):
    """The facts (by predicate: probability or None, arguments), the rules
    (head predicate, head terms, body) and the query atoms of a program."""
    facts, rules, queries = {}, [], []
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
                else:
                    c = COMPARISON.match(b)
                    items.append(('cmp', OPS[c.group(2)], term(c.group(1)), term(c.group(3))))
            rules.append((m.group(1), hargs, items))
            continue
        m = re.fullmatch(r'query\((\w+)\)\.', line)
        if m:
            queries.append(m.group(1))
            continue
        m = FACT.match(line)
        if not m:
            sys.exit(f'not a fact, rule or query: {line}')
        prob, pred, args = m.groups()
        facts.setdefault(pred, []).append((None if prob is None else prob, [term(a) for a in split_top(args or '')]))
    return facts, rules, queries


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


def translate(text):
    """(the SQL that loads the program, [(query, SQL of its probability)])."""
    facts, rules, queries = parse(text)
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
    return '\n'.join(out) + '\n', [(q, f'SELECT probability_evaluate(provenance()) FROM {q}') for q in queries]


def main():
    sql, qs = translate(open(sys.argv[1]).read())
    open(sys.argv[2], 'w').write(sql)
    for q, s in qs:
        print(f'{q}\t{s}')


if __name__ == '__main__':
    main()
