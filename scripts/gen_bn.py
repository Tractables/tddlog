#!/usr/bin/env python3
"""Write a discrete Bayesian network (.bif, Hugin .net or .hugin, GeNIe
.xdsl, or the UAI format's BAYES) as a propositional ProbLog program, or as
a Dice program; or a Markov network (the UAI format's MARKOV) as a ProbLog
program with evidence.

Every variable X with parents P1..Pm becomes one annotated disjunction per
row of its conditional probability table,

    p1::x(v1); ...; pk::x(vk) :- p1(u1), ..., pm(um).

so the program is ground and every rule is a CPT row. Zero-probability heads
are dropped; a row with a single certain value becomes a plain rule.

Usage: gen_bn.py network.{bif,net,hugin,xdsl,uai}[.gz] [--query all | --query VAR[,VAR...] | --query leaves
                                                          | --query none]
                                   [--evidence K|half [--evidence-from leaves|internal|all] [--seed S]
                                    | --evidence-file FILE.inst | --evid FILE.evid] [--inst OUT.inst] [--uai OUT.uai]
                                   [--cnf OUT.cnf [--encoding enc1|chains|enc5]] [--rescale]
                                   [--format problog | --format dice]

`all` (default) queries every value of every variable, `leaves` every value
of the variables without children, VAR the values of the named variables,
and `none` nothing: the program for P(e) alone, and through the offset
below for the partition function (the UAI PR task).
`--evidence K` observes K leaf variables by default; `--evidence-from` can
select internal nodes or all nodes instead. `half` means half of the chosen
pool, at least one. Observations take their values from one world sampled
forward from the network (seed S, default 1), so that the evidence has
positive probability whatever the tables' zeros, and leaves the observed
variables out of the queries. `--evidence-file` takes the observations from
an Ace `.inst` file instead. `--inst OUT` writes the observations, whichever
way they were chosen and a Markov network's factors among them, as an
`.inst` file with the network's own names, for running Ace on the same
network and evidence. `--rescale`
divides every row by its sum, for a file that writes a table in percent or
as weights (`rescaled`); without it a row that sums to more than one beyond
rounding is an error.
A Markov network's variables get uniform priors, times their unary factors
normalized, and every other factor f becomes a variable of one value `ok`
whose row for each assignment of its scope holds the factor's entry divided
by its largest, observed: `w::f(ok) :- x1(u1), ..., xm(um).` and
`evidence(f(ok)).` The probability of the evidence is then the partition
function times a constant, and a query's probability given it is the
network's marginal. The program's first line holds the constant,
`% log10 Z - log10 P(e) = c`: the sum of the base-10 logarithms of each
variable's unary factors summed over its values (its number of values
where it has none), of each other factor's largest entry, and of the
entry of each factor that has one distinct entry, which is dropped
(`partition_offset`); so `log10 Z = log10 P(e) + c`, also given evidence
(`tddlog --evidence-probability` prints `log10 P(e)`). A factor row
within 1e-9 of its largest is written as certain, which moves `log10 Z`
by under 5e-10 for each factor with such a row. A Markov network that
is a Bayesian network as written, each factor the conditional table of
one variable of its scope and the child relation acyclic (the UAI
suite's Promedus and Pedigree), is written as that network instead
(`conditional_tables`): its partition function is one, so the offset is
0.0. Under `--query none`, a Markov network without evidence whose
variables all have the same k values and whose functions are all Potts
tables over two variables (one entry on the diagonal, one off it), such
as a graph colouring, is invariant under any permutation of the values:
the program breaks that symmetry (`colour_symmetry`), which leaves Z a
k-th of the network's, and the offset holds log10 k besides. Its
marginals are no longer the network's, so a program that asks queries
keeps the symmetry. A UAI file's variables
are named `x0`, `x1`, ... and its values `v_0`, `v_1`, ...; `--evid FILE` reads a UAI evidence file
(`N i1 u1 ... iN uN`), and `--uai OUT.uai` writes any network as a BAYES
file with its observations in OUT.uai.evid, for Merlin.
`--cnf OUT.cnf` writes a UAI network's weighted CNF with the
observations (`uai_wcnf.py`), whose weighted model count is Z given
them, from the file's own functions rather than the program's tables,
for a weighted model counter (Ganak): a parameter variable
per entry (ENC1, the default), or with `--encoding chains` the conditional
tables as Sang, Beame and Kautz's chains of switches (ENC2), or with
`--encoding enc5` a parameter per distinct entry of a table, implied by
the entries' merged cubes (ENC5). `--evidence K` samples
forward, which a Markov network does not allow.
Names are lowercased and non-alphanumerics become underscores; a clash is an
error. In the Dice format every variable is a `discrete` under nested
conditionals on its parents (the encoding of Dice's own bngen), whose
values are numbered in declaration order, and the program returns the
queried variable, or a tuple of the queried variables in the order the
ProbLog format queries them: Dice prints a tuple's joint distribution, so
Dice runs one program per variable (`--query VAR`) for the marginals.
"""
import argparse
from decimal import Decimal
import itertools
import math
import random
import re
import sys

import uai_wcnf


def parse_bif(text):
    text = re.sub(r'//.*', '', text)
    variables = {}     # name -> list of values, in declaration order
    order = []
    tables = {}        # name -> (parents, {parent values tuple: [probabilities]})
    for m in re.finditer(r'variable\s+(\S+)\s*\{\s*type\s+discrete\s*\[\s*(\d+)\s*\]\s*\{([^}]*)\}\s*;\s*\}', text):
        name = m.group(1)
        values = [v.strip() for v in m.group(3).split(',')]
        assert len(values) == int(m.group(2)), name
        variables[name] = values
        order.append(name)
    for m in re.finditer(r'probability\s*\(([^)]*)\)\s*\{([^}]*)\}', text):
        head, body = m.group(1), m.group(2)
        parts = [p.strip() for p in head.split('|')]
        name = parts[0]
        parents = [p.strip() for p in parts[1].split(',')] if len(parts) > 1 and parts[1].strip() else []
        rows = {}
        for line in body.strip().split(';'):
            line = line.strip()
            if not line:
                continue
            if line.startswith('table'):
                probs = [float(x) for x in line[len('table'):].replace(',', ' ').split()]
                rows[()] = probs
            else:
                pm = re.match(r'\(([^)]*)\)\s*(.*)', line, re.S)
                key = tuple(v.strip() for v in pm.group(1).split(','))
                probs = [float(x) for x in pm.group(2).replace(',', ' ').split()]
                rows[key] = probs
        tables[name] = (parents, rows)
    return order, variables, tables


def parse_net(text):
    """Hugin .net: `node X { states = ("a" "b"); }` and
    `potential (X | P1 P2) { data = (((...))); }`, the data nested with the
    first parent outermost and the values of X innermost."""
    # A comment runs from `%` to the end of the line, outside quotes: munin's
    # states include "<12%".
    text = re.sub(r'("[^"]*")|%.*', lambda m: m.group(1) or '', text)
    variables, order, tables = {}, [], {}
    for m in re.finditer(r'\bnode\s+(\S+)\s*\{(.*?)\}', text, re.S):
        sm = re.search(r'states\s*=\s*\(([^)]*)\)', m.group(2))
        if not sm:
            continue
        values = re.findall(r'"([^"]*)"', sm.group(1))
        variables[m.group(1)] = values
        order.append(m.group(1))
    for m in re.finditer(r'\bpotential\s*\(([^)]*)\)\s*\{(.*?)\}', text, re.S):
        parts = [p.strip() for p in m.group(1).split('|')]
        name = parts[0]
        parents = parts[1].split() if len(parts) > 1 else []
        dm = re.search(r'\bdata\s*=\s*(.*?);', m.group(2), re.S)
        numbers = [float(x) for x in re.findall(r'[-+]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][-+]?[0-9]+)?', dm.group(1))]
        k = len(variables[name])
        keys = list(itertools.product(*(variables[p] for p in parents)))
        assert len(numbers) == k * len(keys), (name, len(numbers), k, len(keys))
        rows = {key: numbers[i * k:(i + 1) * k] for i, key in enumerate(keys)}
        tables[name] = (parents, rows)
    return order, variables, tables


def parse_xdsl(data):
    """GeNIe/SMILE .xdsl: `<cpt id="X">` with its `<state id="..."/>`s, its
    `<parents>` and its `<probabilities>`, ordered as in a Hugin table. A node
    of another kind (noisy-max, deterministic, equation) is not a table, and
    the network is refused."""
    import xml.etree.ElementTree as ET
    nodes = ET.fromstring(data).find('nodes')
    if nodes is None:
        sys.exit('xdsl: no <nodes>')
    variables, order, tables, read = {}, [], {}, []
    for node in nodes:
        name = node.get('id')
        if node.tag != 'cpt':
            sys.exit(f'xdsl: {name} is a {node.tag} node, not a table')
        variables[name] = [s.get('id') for s in node.findall('state')]
        order.append(name)
        parents = node.find('parents')
        read.append((name, parents.text.split() if parents is not None and parents.text else [],
                     [float(x) for x in node.find('probabilities').text.split()]))
    for name, parents, numbers in read:
        k = len(variables[name])
        keys = list(itertools.product(*(variables[p] for p in parents)))
        assert len(numbers) == k * len(keys), (name, len(numbers), k, len(keys))
        tables[name] = (parents, {key: numbers[i * k:(i + 1) * k] for i, key in enumerate(keys)})
    return order, variables, tables


def read_uai(text):
    """The parts of a UAI model file: `BAYES` or `MARKOV`, the variables'
    domain sizes, the functions' scopes, then each function's table, the
    scope's last variable changing fastest. Returns the kind, the sizes,
    the scopes as lists of variable indices and the tables as lists of
    floats."""
    tok = text.split()
    kind, pos = tok[0].upper(), 1
    def take(k):
        nonlocal pos
        out = tok[pos:pos + k]
        if len(out) < k:
            sys.exit('UAI file ends early')
        pos += k
        return out
    if kind not in ('BAYES', 'MARKOV'):
        sys.exit(f'UAI file of kind {tok[0]}: BAYES or MARKOV')
    n = int(take(1)[0])
    sizes = [int(x) for x in take(n)]
    scopes = []
    for _ in range(int(take(1)[0])):
        k = int(take(1)[0])
        scopes.append([int(x) for x in take(k)])
        if any(not 0 <= v < n for v in scopes[-1]):
            sys.exit(f'UAI scope {scopes[-1]} names a variable beyond the {n} of the file')
    tables = []
    for scope in scopes:
        entries = int(take(1)[0])
        if entries != math.prod(sizes[v] for v in scope):
            sys.exit(f'UAI table of {entries} entries for a scope of sizes {[sizes[v] for v in scope]}')
        tables.append([float(x) for x in take(entries)])
    if pos != len(tok):
        sys.exit(f'UAI file has {len(tok) - pos} tokens after its tables')
    return kind, sizes, scopes, tables


def uai_variables(sizes):
    """The names of a UAI file's variables, `x0`, `x1`, ..., and of their
    values, `0`, `1`, ... (the program writes them `v_0`, `v_1`, ...)."""
    return [f'x{i}' for i in range(len(sizes))], {f'x{i}': [str(u) for u in range(k)] for i, k in enumerate(sizes)}


def parse_uai(text, colours=False):
    """A UAI model file (`read_uai`), the network the program is written
    from. A BAYES function is the table of its scope's last variable given
    the others. Returns the network, the observations a MARKOV network's
    factors make, and for a MARKOV network `log10 Z - log10 P(e)` of the
    program (`gen_bn.py` docstring), None for a BAYES one. With `colours`,
    for a program that asks no query and observes nothing, a MARKOV
    network's colour symmetry is broken where it has one
    (`colour_symmetry`)."""
    kind, sizes, scopes, tables = read_uai(text)
    n = len(sizes)
    order, variables = uai_variables(sizes)
    rows_of = lambda scope, table, width: {
        key: table[j * width:(j + 1) * width]
        for j, key in enumerate(itertools.product(*[variables[f'x{v}'] for v in scope]))}
    out = {}
    if kind == 'BAYES':
        for scope, table in zip(scopes, tables):
            child = f'x{scope[-1]}'
            if child in out:
                sys.exit(f'UAI BAYES file with two tables for {child}')
            out[child] = ([f'x{v}' for v in scope[:-1]], rows_of(scope[:-1], table, sizes[scope[-1]]))
        if len(out) != n:
            sys.exit(f'UAI BAYES file with tables for {len(out)} of {n} variables')
        return (order, variables, out), [], None
    network = conditional_tables(sizes, scopes, tables)
    if network is not None:
        for v, (scope, table) in enumerate(network):
            out[f'x{v}'] = ([f'x{u}' for u in scope[:-1]], rows_of(scope[:-1], table, sizes[v]))
        return (order, variables, out), [], 0.0
    prior = [[1.0] * sizes[i] for i in range(n)]
    factors = []
    # The constant factors, which the program leaves out: their entries are
    # factors of Z that P(e) does not hold.
    constants = []
    broken = colour_symmetry(sizes, scopes, tables) if colours else None
    if broken is not None:
        # The network's Z is k times that of the functions so broken.
        scopes, tables = broken
        constants.append(float(sizes[0]))
    for scope, table in zip(scopes, tables):
        if min(table) < 0:
            sys.exit('UAI factor with a negative entry')
        if len(scope) == 1:
            prior[scope[0]] = [a * b for a, b in zip(prior[scope[0]], table)]
        elif len(set(table)) > 1:
            factors.append((scope, table))
        elif table[0] == 0:
            sys.exit('UAI factor that is zero everywhere')
        else:
            constants.append(table[0])
    totals = []
    for i in range(n):
        total = math.fsum(prior[i])
        if total == 0:
            sys.exit(f'x{i}: its unary factors are zero everywhere')
        totals.append(total)
        out[f'x{i}'] = ([], {(): [w / total for w in prior[i]]})
    forced = []
    for j, (scope, table) in enumerate(factors):
        f, top = f'f{j}', max(table)
        order.append(f)
        variables[f] = ['ok']
        out[f] = ([f'x{v}' for v in scope], rows_of(scope, [w / top for w in table], 1))
        forced.append((f, 'ok'))
    return (order, variables, out), forced, partition_offset(totals, [max(t) for _, t in factors], constants)


def conditional_tables(sizes, scopes, tables):
    """A Markov network that is a Bayesian network as written: each
    function a conditional table of one variable of its scope (its child:
    for each assignment of the others the entries over the child's values
    sum to one within 1e-9), each variable the child of one function, and
    the child relation acyclic. Returns each variable's scope, its child
    last, and its table in that order; None for another network. A
    function that is a table of two of its variables, an identity say,
    takes the child a matching of functions to variables gives it."""
    n = len(sizes)
    if len(scopes) != n:
        return None
    candidates = []
    for scope, table in zip(scopes, tables):
        strides = [math.prod(sizes[u] for u in scope[i + 1:]) for i in range(len(scope))]
        candidates.append([
            i for i, (v, stride) in enumerate(zip(scope, strides))
            if all(abs(math.fsum(table[r + stride * x] for x in range(sizes[v])) - 1) <= 1e-9
                   for r in range(len(table)) if r // stride % sizes[v] == 0)])
        if not candidates[-1]:
            return None
    # Kuhn's augmenting paths, functions to the variables they may be the
    # table of; almost every function has one candidate.
    owner = [None] * n
    for j in range(n):
        seen, path, stack = set(), {}, [j]
        while stack:
            f = stack.pop()
            free = None
            for i in candidates[f]:
                v = scopes[f][i]
                if v in seen:
                    continue
                seen.add(v)
                path[v] = f
                if owner[v] is None:
                    free = v
                    break
                stack.append(owner[v])
            if free is not None:
                v = free
                while v is not None:
                    f = path[v]
                    previous = next((scopes[f][i] for i in candidates[f] if owner[scopes[f][i]] == f), None)
                    owner[v], v = f, previous
                break
        else:
            return None
    out = []
    for v in range(n):
        scope, table = scopes[owner[v]], tables[owner[v]]
        at = scope.index(v)
        order = [i for i in range(len(scope)) if i != at] + [at]
        strides = [math.prod(sizes[u] for u in scope[i + 1:]) for i in range(len(scope))]
        out.append(([scope[i] for i in order],
                    [table[sum(a * strides[i] for a, i in zip(key, order))]
                     for key in itertools.product(*[range(sizes[scope[i]]) for i in order])]))
    state = [0] * n   # 0 unseen, 1 on the path, 2 done
    for root in range(n):
        stack = [(root, iter(out[root][0][:-1]))] if state[root] == 0 else []
        if stack:
            state[root] = 1
        while stack:
            v, parents = stack[-1]
            u = next(parents, None)
            if u is None:
                state[v] = 2
                stack.pop()
            elif state[u] == 1:
                return None
            elif state[u] == 0:
                state[u] = 1
                stack.append((u, iter(out[u][0][:-1])))
    return out


def colour_symmetry(sizes, scopes, tables):
    """The colour symmetry of a Markov network, broken. In a network whose
    variables all have the same k >= 2 values and whose functions are all
    over two variables and Potts tables (one entry on the diagonal, one
    off it), any permutation of the values maps a world to one of the
    same weight. So without evidence Z is k times its value with v1, a
    variable of largest degree, at 0; given that, the values 1..k-1 are
    interchangeable, so v2, a neighbour of v1 of largest degree, takes 0
    with weight 1 and 1 with weight k - 1 and no other value; given v1
    and v2, v3 (a common neighbour of largest degree, else one of v2's
    other neighbours, if any) takes each value used so far with weight 1
    and the first unused one with the number of unused values as its
    weight. Returns the functions with v1's and v2's unary tables added
    and the table of v3 given v2 where there is a v3, whose Z is the
    network's divided by k; None for another network. On the UAI 2022 PR set's queen5_5_3
    (k = 3) the compile went from 5.08 G work units to 0.85 G."""
    k = sizes[0] if sizes else 0
    if k < 2 or any(c != k for c in sizes) or not scopes:
        return None
    neighbours = [set() for _ in sizes]
    for scope, table in zip(scopes, tables):
        if len(scope) != 2 or scope[0] == scope[1]:
            return None
        rows = [table[i * k:(i + 1) * k] for i in range(k)]
        if len({rows[i][i] for i in range(k)}) != 1 or \
                len({rows[i][j] for i in range(k) for j in range(k) if i != j}) != 1:
            return None
        a, b = scope
        neighbours[a].add(b)
        neighbours[b].add(a)
    largest = lambda vs: max(vs, key=lambda v: (len(neighbours[v]), -v))
    v1 = largest(range(len(sizes)))
    v2 = largest(neighbours[v1])
    common = neighbours[v1] & neighbours[v2]
    others = common or neighbours[v2] - {v1}
    scopes = list(scopes) + [[v1], [v2]]
    tables = list(tables) + [[1.0] + [0.0] * (k - 1), [1.0, float(k - 1)] + [0.0] * (k - 2)]
    if others:
        v3 = largest(others)
        table = []
        for c2 in range(k):
            row = [0.0] * k
            if c2 <= 1:
                used = c2 + 1   # v1 = 0 and v2 = c2
                row[:used] = [1.0] * used
                if used < k:
                    row[used] = float(k - used)
            table += row
        scopes.append([v2, v3])
        tables.append(table)
    return scopes, tables


def partition_offset(totals, tops, constants):
    """`log10 Z - log10 P(e)` of a Markov network's program: each variable's
    prior is its unary factors divided by their sum over its values
    (`totals`), each other factor is divided by its largest entry (`tops`),
    and a factor with one distinct entry is left out (`constants`). In each
    assignment the program's weight is the product of the factors divided
    by all of these, so `P(e) · 10^offset` is `Z` given the same evidence."""
    return math.fsum(math.log10(x) for x in totals + tops + constants)


def uai_cnf(path, evidence, encoding='enc1'):
    """The weighted CNF of the UAI file at `path` with the observations
    `evidence`, (variable, value) as `read_evid` gives them, and its counts
    (`uai_wcnf.encode`, its tables in `encoding`)."""
    _, sizes, scopes, tables = read_uai(read_bytes(path).decode())
    try:
        return uai_wcnf.encode(sizes, scopes, tables, [(int(v[1:]), int(u)) for v, u in evidence], encoding)
    except ValueError as x:
        sys.exit(f'{path}: {x}')


def read_evid(path, network):
    """The observations of a UAI evidence file, `N i1 u1 ... iN uN`, as
    (variable, value)."""
    order, variables, _ = network
    with open(path) as f:
        tok = f.read().split()
    if not tok or len(tok) != 1 + 2 * int(tok[0]):
        sys.exit(f'{path}: not a UAI evidence file of one sample')
    given = {}
    for i, u in zip(tok[1::2], tok[2::2]):
        v = f'x{i}'
        if v not in variables or u not in variables[v]:
            sys.exit(f'{path}: no value {u} of {v}')
        if v in given:
            sys.exit(f'{path}: {v} observed twice')
        given[v] = u
    return [(v, given[v]) for v in order if v in given]


def uai_observes(path):
    """Whether a UAI evidence file observes a variable (`read_evid` reads
    and checks the observations)."""
    with open(path) as f:
        tok = f.read().split()
    return bool(tok) and int(tok[0]) != 0


def read_bytes(path):
    """The bytes of a network file, gunzipped if its name ends in `.gz`."""
    import gzip
    with (gzip.open if path.endswith('.gz') else open)(path, 'rb') as f:
        return f.read()


def parse(path, colours=False):
    """The network of a file. `colours` breaks a UAI Markov network's
    colour symmetry where it has one (`parse_uai`)."""
    data = read_bytes(path)
    path = path.removesuffix('.gz')
    if path.endswith('.uai'):
        return parse_uai(data.decode(), colours)
    if path.endswith('.xdsl'):
        return parse_xdsl(data)   # XML: the declaration names the encoding
    try:
        text = data.decode()
    except UnicodeDecodeError:
        text = data.decode('latin-1')   # Ace's water, pigs and mildew .hugin files
    return parse_net(text) if path.endswith(('.net', '.hugin')) else parse_bif(text)


def read_inst(path, network):
    """The observations of an Ace `.inst` file, `<inst id="X" value="v"/>`
    under `<instantiation>`, as (variable, value) in declaration order. A
    variable or value the network does not have, or a variable observed
    twice, is an error."""
    import xml.etree.ElementTree as ET
    order, variables, _ = network
    root = ET.parse(path).getroot()
    if root.tag != 'instantiation':
        sys.exit(f'{path}: not an instantiation')
    given = {}
    for e in root:
        v, u = e.get('id'), e.get('value')
        if e.tag != 'inst' or v is None or u is None:
            sys.exit(f'{path}: not an observation: <{e.tag}>')
        if v not in variables:
            sys.exit(f'{path}: no variable {v}')
        if u not in variables[v]:
            sys.exit(f'{path}: {v} has no value {u}')
        if v in given:
            sys.exit(f'{path}: {v} observed twice')
        given[v] = u
    return [(v, given[v]) for v in order if v in given]


def write_inst(evidence):
    """The `.inst` file Ace reads for the observations `evidence`, with the
    network's own names (Ace reads the network file, not the program)."""
    from xml.sax.saxutils import quoteattr
    lines = ['<?xml version="1.0" encoding="UTF-8"?>', '<instantiation>']
    lines += [f'<inst id={quoteattr(v)} value={quoteattr(u)}/>' for v, u in evidence]
    return '\n'.join(lines + ['</instantiation>']) + '\n'


def write_uai(network, evidence):
    """The network as a UAI BAYES file and the observations as a UAI
    evidence file, the files Merlin reads: variable i is the i-th of the
    network's order and value u its u-th, and each table holds the rows as
    the program has them (`normalized`)."""
    order, variables, tables = network
    index = {v: i for i, v in enumerate(order)}
    lines = ['BAYES', str(len(order)), ' '.join(str(len(variables[v])) for v in order), str(len(order))]
    lines += [' '.join(str(x) for x in [len(tables[v][0]) + 1] + [index[p] for p in tables[v][0]] + [index[v]])
              for v in order]
    for v in order:
        parents, rows = tables[v]
        entries = []
        for key in itertools.product(*[variables[p] for p in parents]):
            if key not in rows:
                sys.exit(f'{v}: no row for {key}, which a UAI table needs')
            entries += normalized(rows[key], f'{v} {key}')
        lines += ['', str(len(entries)), ' '.join(repr(float(p)) for p in entries)]
    evid = [str(len(evidence))] + [f'{index[v]} {variables[v].index(u)}' for v, u in evidence]
    return '\n'.join(lines) + '\n', ' '.join(evid) + '\n'


def write_hugin(network):
    """The network as a Hugin .net file, for Ace on a network read from a
    UAI file. Each table holds the rows as the program has them
    (`normalized`). A variable of one value, as a Markov network's factor
    is (`parse_uai`), gets a second value `not_<value>` holding the rest
    of each row, so that every row sums to one as a Bayesian network's
    must; observing the first value then weighs each assignment of the
    factor's scope by its row as the program's evidence does."""
    order, variables, tables = network
    quote = lambda s: '"' + s.replace('"', '') + '"'
    lines = ['net', '{', '}']
    states = {v: variables[v] + ([f'not_{variables[v][0]}'] if len(variables[v]) == 1 else []) for v in order}
    for v in order:
        lines += [f'node {v}', '{', f'  states = ({" ".join(quote(u) for u in states[v])});', '}']
    for v in order:
        parents, rows = tables[v]

        def nest(depth, key):
            if depth == len(parents):
                if key not in rows:
                    sys.exit(f'{v}: no row for {key}, which a Hugin table needs')
                probs = normalized(rows[key], f'{v} {key}')
                if len(variables[v]) == 1:
                    probs = [probs[0], max(0.0, 1.0 - probs[0])]
                return '(' + ' '.join(repr(float(p)) for p in probs) + ')'
            return '(' + ' '.join(nest(depth + 1, key + (u,)) for u in variables[parents[depth]]) + ')'
        given = f' | {" ".join(parents)}' if parents else ''
        lines += [f'potential ({v}{given})', '{', f'  data = {nest(0, ())};', '}']
    return '\n'.join(lines) + '\n'


def ident(s):
    t = re.sub(r'[^a-z0-9_]', '_', s.lower())
    if not t or not t[0].isalpha():
        t = 'v_' + t
    return t


def rounding_slack(probs):
    """The row's sum in the decimals it is written with, and the most that
    rounding can have moved it. The files round both ways: to a number of
    decimals, so that sachs writes 0.0000000 and 0.001432665 in one row, and
    to a number of significant digits, so that bn_pathfinder's `0.9999,
    9.999e-05` are 10000/10001 and 1/10001 to four digits, 10⁻⁸ short of one.
    So each entry may be off by the larger of half a unit in the row's finest
    decimal place and half a unit in the most significant digits any entry
    of the row has, trailing zeros dropped (a zero entry by the first).
    Files written by a program print doubles in full, binary noise and all:
    Ace's pathfinder.xdsl writes 0.00099404 as 0.0009940400000000005, and its
    blockmap_10_01 writes 10/11 and 1 − 10/11 as two doubles that sum to
    1.00000000000000008. So an entry is read as the decimal it means
    (`meant_decimal`), and may be off by half a unit in the last place of
    one besides, the error of a complement."""
    written = [meant_decimal(float(p)) for p in probs]
    nonzero = [d for d in written if d]
    if not nonzero:
        return Decimal(0), Decimal(0)
    place = min(d.as_tuple().exponent for d in nonzero)
    digits = max(len(d.as_tuple().digits) for d in nonzero)
    half_place = Decimal(5).scaleb(place - 1)
    half_ulp = Decimal(math.ulp(1.0)) / 2
    slack = sum(max(half_place, Decimal(5).scaleb(d.adjusted() - digits), half_ulp) if d else half_place
                for d in written)
    return sum(written), slack


def meant_decimal(x):
    """The decimal a file means by the double `x`, trailing zeros dropped:
    the shortest one of at most 12 significant digits within two units in
    the last place of `x`, which strips the noise of the arithmetic that
    wrote the file (0.0009940400000000005 is 0.00099404), and otherwise `x`
    as written, so that 2/3 printed in full stays 0.6666666666666667."""
    for digits in range(1, 13):
        s = f'{x:.{digits}g}'
        if abs(float(s) - x) <= 2 * math.ulp(x):
            return Decimal(s).normalize()
    return Decimal(repr(x)).normalize()


def normalized(probs, where):
    # `math.fsum`, not `sum`: CPython 3.12 sums floats with compensation and
    # older versions do not, so `sum([0.55, 0.34, 0.11])` is 1.0 on one and
    # 1.0000000000000002 on another. That decides whether the row below is
    # rewritten, and a benchmark program whose numbers depend on the Python
    # that wrote it is not one two boxes can be compared on.
    #
    # A row of a table sums to one, and the file's decimals round it:
    # bn_pathfinder's rows of `0.3333333` three times sum to 0.9999999. Such
    # a row is rescaled when rounding its entries explains the gap, that is
    # when the gap is under the rounding slack; ProbLog would complete it
    # with a world where no value holds, and tddlog's clause tables
    # could not take it. A gap rounding does not
    # explain, `0.9, 0.099` say, is the table's own: short of one, the rest
    # is left to no value as written; over one, it is an error. A shortfall
    # below 10⁻⁹, the grounder's own tolerance for a row's last value, is
    # left as written, so that rows that sum to one but for binary rounding
    # keep their numbers.
    total = math.fsum(probs)
    exact, slack = rounding_slack(probs)
    if abs(exact - 1) >= slack:
        if exact > 1:
            sys.exit(f'{where}: probabilities sum to {exact}')
        return probs
    if total > 1 or total < 1 - 1e-9:
        probs = [p / total for p in probs]   # rounding in the file
    return probs


def rescaled(network):
    """The network with every row divided by its sum, as Ace's `-Xnormalize`
    does on reading. Ace's copies of diabetes and munin1 to munin4 write
    some tables in percent, and its water and mildew write a root as weights
    (`1 1 1 1`); scaling a row changes no posterior there. A row of zeros has
    no distribution to rescale to and is an error."""
    order, variables, tables = network
    out = {}
    for v in order:
        parents, rows = tables[v]
        for key, probs in rows.items():
            if not math.fsum(probs) > 0:
                sys.exit(f'{v} {key}: a row of zeros cannot be rescaled')
        out[v] = (parents, {key: [p / math.fsum(probs) for p in probs] for key, probs in rows.items()})
    return order, variables, out


def queried_variables(order, variables, tables, query):
    children = {v: set() for v in order}
    for v in order:
        for p in tables[v][0]:
            children[p].add(v)
    if query == 'all':
        return order
    if query == 'leaves':
        return [v for v in order if not children[v]]
    queried = query.split(',')
    for q in queried:
        if q not in variables:
            sys.exit(f'no variable {q}')
    return queried


def topological(order, tables):
    """Variables with parents before children (depth first from the
    declaration order); the order of the facts in the program."""
    done, topo = set(), []

    def visit(v):
        if v in done:
            return
        done.add(v)
        for p in tables[v][0]:
            visit(p)
        topo.append(v)

    for v in order:
        visit(v)
    return topo


def sampled_evidence(network, k, seed, pool='leaves'):
    """K variables from the requested pool, at their values in one world sampled forward, as
    (variable, value) in declaration order. Only `random.random` is drawn,
    whose sequence for a seed Python keeps from version to version."""
    order, variables, tables = network
    rng = random.Random(seed)
    world = {}
    for v in topological(order, tables):
        parents, rows = tables[v]
        probs = normalized(rows[tuple(world[p] for p in parents)], v)
        r, acc = rng.random() * math.fsum(probs), 0.0
        world[v] = next((u for u, p in zip(variables[v], probs) if (acc := acc + p) > r and p > 0),
                        [u for u, p in zip(variables[v], probs) if p > 0][-1])
    leaves = queried_variables(order, variables, tables, 'leaves')
    candidates = leaves if pool == 'leaves' else [v for v in order if pool == 'all' or v not in set(leaves)]
    if k == 'half':
        k = max(1, len(candidates) // 2)
    if k > len(candidates):
        sys.exit(f'--evidence {k}: the evidence pool {pool} has {len(candidates)} variables')
    keyed = sorted(candidates, key=lambda v: rng.random())
    chosen = set(keyed[:k])
    return [(v, world[v]) for v in order if v in chosen]


OFFSET = '% log10 Z - log10 P(e) = '


def offset_line(offset):
    """The program's first line for a Markov network, `% log10 Z - log10
    P(e) = c`, which `read_offset` reads; empty for another
    network."""
    return '' if offset is None else f'{OFFSET}{offset!r}\n'


def read_offset(text):
    """The `log10 Z - log10 P(e)` a program's first line states, or None."""
    first = text.split('\n', 1)[0]
    return float(first[len(OFFSET):]) if first.startswith(OFFSET) else None


def to_problog(network, query='all', evidence=()):
    order, variables, tables = network
    names = {}
    for v in order:
        n = ident(v)
        if n in names.values():
            sys.exit(f'name clash: {v} -> {n}')
        names[v] = n
    values = {v: {u: ident(u) for u in variables[v]} for v in order}
    for v in order:
        if len(set(values[v].values())) != len(variables[v]):
            sys.exit(f'value clash in {v}: {variables[v]}')
    out = []
    for v in topological(order, tables):
        parents, rows = tables[v]
        for key, probs in rows.items():
            assert len(probs) == len(variables[v]), (v, key)
            probs = normalized(probs, f'{v} {key}')
            body = ', '.join(f'{names[p]}({values[p][u]})' for p, u in zip(parents, key))
            heads = [(p, f'{names[v]}({values[v][u]})') for p, u in zip(probs, variables[v]) if p > 0]
            if not heads:
                continue   # no value under this row: no rule
            if len(heads) == 1 and abs(heads[0][0] - 1) < 1e-9:
                head = heads[0][1]
            else:
                head = '; '.join(f'{p!r}::{a}' for p, a in heads)
            out.append(f'{head} :- {body}.' if body else f'{head}.')
    observed = {v for v, _ in evidence}
    for v, u in evidence:
        out.append(f'evidence({names[v]}({values[v][u]})).')
    # None: the program of another one, which states its own queries;
    # 'none': the program for P(e) alone.
    if query is None or query == 'none':
        return '\n'.join(out) + '\n'
    queried = [v for v in queried_variables(order, variables, tables, query) if v not in observed]
    if not queried:
        sys.exit(f'--query {query}: every queried variable is observed, nothing is left to query')
    for v in queried:
        for u in variables[v]:
            out.append(f'query({names[v]}({values[v][u]})).')
    return '\n'.join(out) + '\n'


def to_dice(network, query):
    order, variables, tables = network
    queried = queried_variables(order, variables, tables, query)
    names = {v: ident(v).upper() for v in order}
    out = []

    def table(v, parents, rows, key):
        if not parents:
            probs = normalized(rows[key], f'{v} {key}')
            return 'discrete(' + ', '.join(repr(float(p)) for p in probs) + ')'
        p, rest = parents[0], parents[1:]
        vals = variables[p]
        bits = max(1, (len(vals) - 1).bit_length())   # int(bits, k): Dice's integer width
        branches = [table(v, rest, rows, key + (u,)) for u in vals]
        e = branches[-1]
        for k in range(len(vals) - 2, -1, -1):
            e = f'if ({names[p]} == int({bits}, {k})) then ({branches[k]}) else ({e})'
        return e

    for v in topological(order, tables):   # a let must follow its parents
        parents, rows = tables[v]
        out.append(f'let {names[v]} = {table(v, parents, rows, ())} in')
    result = names[queried[-1]]
    for v in reversed(queried[:-1]):   # Dice's tuples are pairs
        result = f'({names[v]}, {result})'
    out.append(result)
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('network')
    ap.add_argument('--query', default='all')
    ap.add_argument('--evidence-from', choices=['leaves', 'internal', 'all'], default='leaves')
    ap.add_argument('--evidence', type=lambda x: x if x == 'half' else int(x), default=0)
    ap.add_argument('--seed', type=int)
    ap.add_argument('--evidence-file', help='the observations of an Ace .inst file')
    ap.add_argument('--evid', help='the observations of a UAI evidence file')
    ap.add_argument('--inst', help='write the observations to this .inst file')
    ap.add_argument('--uai', help='write the network to this UAI file and the observations to it plus .evid')
    ap.add_argument('--cnf', help="write a UAI network's weighted CNF and the observations to this file "
                    '(uai_wcnf.py), for a weighted model counter')
    ap.add_argument('--encoding', choices=uai_wcnf.ENCODINGS,
                    help='with --cnf, how it writes the tables: a parameter per entry (enc1, the default), '
                    'the conditional tables as chains of switches (chains, ENC2), or a parameter per '
                    'distinct entry of a table (enc5)')
    ap.add_argument('--rescale', action='store_true', help='divide every row by its sum')
    ap.add_argument('--format', default='problog', choices=['problog', 'dice'])
    a = ap.parse_args()
    if a.evidence_from != 'leaves' and not a.evidence:
        sys.exit('--evidence-from chooses the evidence pool and needs --evidence')
    if a.seed is not None and not a.evidence:
        sys.exit('--seed chooses the evidence, and there is none (--evidence K)')
    if a.evidence and a.evidence_file:
        sys.exit('--evidence samples the observations and --evidence-file reads them: one of the two')
    if (a.evidence or a.evidence_file or a.inst or a.uai or a.cnf) and a.format == 'dice':
        sys.exit('the Dice format observes nothing')
    if a.query == 'none' and a.format == 'dice':
        sys.exit('--query none asks nothing, and a Dice program returns the queried variables')
    uai = a.network.removesuffix('.gz').endswith('.uai')
    if a.query == 'none' and uai and (a.inst or a.uai):
        sys.exit("--inst and --uai write a UAI network's observations for Ace and Merlin, and under --query none "
                 'the program may break its colour symmetry, with observations of its own: write them with queries')
    # A program that asks nothing and observes nothing breaks a Markov
    # network's colour symmetry where it has one.
    network = parse(a.network, a.query == 'none' and not (a.evid and uai_observes(a.evid)) and not a.evidence_file)
    forced, offset = [], None
    if uai:
        network, forced, offset = network
    elif a.evid:
        sys.exit('--evid reads UAI evidence, for a .uai network')
    elif a.cnf:
        sys.exit('--cnf encodes the functions of a UAI file, and the network is not one')
    if a.encoding and not a.cnf:
        sys.exit('--encoding chooses how --cnf writes the tables, and there is no --cnf')
    if a.cnf and a.rescale:
        sys.exit("--cnf encodes the UAI file's own entries, which --rescale does not change")
    if forced and (a.evidence or a.format == 'dice'):
        sys.exit('a Markov network cannot be sampled forward (--evidence) or written for Dice')
    if a.evid and (a.evidence or a.evidence_file):
        sys.exit('--evid, --evidence and --evidence-file each choose the observations: one of them')
    if a.rescale:
        network = rescaled(network)
    if a.evidence_file:
        evidence = read_inst(a.evidence_file, network)
    elif a.evidence:
        evidence = sampled_evidence(network, a.evidence, 1 if a.seed is None else a.seed, a.evidence_from)
    elif a.evid:
        evidence = read_evid(a.evid, network)
    else:
        evidence = []
    if a.cnf:
        with open(a.cnf, 'w') as f:
            f.write(uai_cnf(a.network, evidence, a.encoding or 'enc1')[0])
    evidence = forced + evidence
    if a.inst:
        with open(a.inst, 'w') as f:
            f.write(write_inst(evidence))
    if a.uai:
        model, evid = write_uai(network, evidence)
        with open(a.uai, 'w') as f:
            f.write(model)
        with open(a.uai + '.evid', 'w') as f:
            f.write(evid)
    if a.format == 'dice':
        sys.stdout.write(to_dice(network, a.query))
    else:
        sys.stdout.write(offset_line(offset) + to_problog(network, a.query, evidence))
