"""The weighted CNF of a UAI network and its evidence, whose weighted model
count is the network's partition function Z given the evidence: the
encoding of Darwiche (A logical approach to factoring belief networks, KR
2002) and Chavira and Darwiche (On probabilistic inference by weighted
model counting, AIJ 2008), for a weighted model counter (Ganak).
`gen_bn.py --cnf OUT` writes it.

Indicators. A variable of two values is one Boolean variable, true at its
value 1 and false at its value 0. A variable of k > 2 values is k Boolean
variables, one per value, with one clause saying that one of them holds
and k(k-1)/2 saying that no two do. A variable of one value has none: its
value always holds. Indicators weigh 1 whether true or false.

Parameters. Each entry of a function, at one assignment of its scope, is
the conjunction of that assignment's indicators:
- an entry of 0 is a clause that rules the assignment out, the disjunction
  of the negated indicators;
- an entry of 1 needs nothing;
- any other entry is a parameter variable P equivalent to the conjunction,
  `-P | l` for each indicator l and `P | -l1 | ... | -lm`, which weighs the
  entry when true and 1 when false.

Evidence. Each observed value is a unit clause on its indicator.

Every model of the CNF is determined by its indicators, which are an
assignment of the network's variables that agrees with the evidence and
meets no zero entry, and its weight is the product of the entries of that
assignment; so the weighted model count is Z given the evidence, for a
MARKOV network as for a BAYES one (where it is P(e)). The weights are the
UAI file's entries as gen_bn.read_uai reads them, written as the shortest
decimals that read back as the same doubles; a counter that reads them as
exact decimals counts with the decimals the doubles stand for.

The file is a weighted CNF in DIMACS: `c t wmc`, `p cnf VARIABLES CLAUSES`, the weights as
`c p weight LITERAL W 0`, both literals of each parameter given, since a
weight 0 < W < 1 given for one literal alone leaves 1 - W to the other.

Two other encodings of the tables are written on request (`encode`'s
`encoding`, `gen_bn.py --cnf OUT --encoding`): Sang, Beame and Kautz's
chains (ENC2) for the conditional tables, and ENC5, Chavira and
Darwiche's parameter equality and context-specific independence with one
implication: a parameter per distinct entry of a table, implied by the
entries' assignments, weighing the entry when true and one less the entry
when false.
"""
import math


def indicators(sizes):
    """For each variable, the literal of each of its values: none for a
    variable of one value (`None`, the value always holds), one Boolean
    variable for two values, one per value for more; and the number of
    Boolean variables used."""
    literals, n = [], 0
    for k in sizes:
        if k < 1:
            raise ValueError(f'a variable of {k} values')
        if k == 1:
            literals.append([None])
        elif k == 2:
            n += 1
            literals.append([-n, n])
        else:
            literals.append(list(range(n + 1, n + k + 1)))
            n += k
    return literals, n


def conditional(sizes, scope, table):
    """Whether a table is a conditional one of its scope's last variable:
    its entries over that variable's values sum to one at each assignment
    of the others, to within the doubles' rounding (1e-12). A chain writes
    a row divided by its sum, so a row short of one as written (by 10^-9
    in some of the UAI networks) is not one for it."""
    if not scope:
        return False
    k = sizes[scope[-1]]
    return all(abs(math.fsum(table[r:r + k]) - 1) <= 1e-12 for r in range(0, len(table), k))


ENCODINGS = ('enc1', 'chains', 'enc5')


def merged(cubes, domain):
    """Chavira and Darwiche's merge of the cubes of one class of equal
    entries of a table (context-specific independence): cubes, tuples of a
    value or None (any value) per position of the scope, that differ in
    one position only and there take each of its `domain[i]` values are
    one cube with None there, to a fixpoint. The cubes of a class are
    disjoint and stay so, and their union does not change."""
    cubes = set(cubes)
    changed = True
    while changed:
        changed = False
        for i, k in enumerate(domain):
            parts = {}
            for c in cubes:
                if c[i] is not None:
                    parts.setdefault(c[:i] + (None,) + c[i + 1:], []).append(c)
            for key, cs in parts.items():
                if len(cs) == k:
                    cubes.difference_update(cs)
                    cubes.add(key)
                    changed = True
    return sorted(cubes, key=lambda c: tuple(-1 if u is None else u for u in c))


def encode(sizes, scopes, tables, evidence, encoding='enc1'):
    """The weighted CNF (module docstring) of a network of variables of
    `sizes` with functions of `scopes` and `tables`, the scope's last
    variable changing fastest, and `evidence` as (variable, value) pairs of
    indices, its tables as `encoding` writes them: `enc1` (the module
    docstring), `chains` or `enc5` (below).
    Returns the file's text and its counts: `variables`, `indicators`,
    `parameters`, `clauses`, and of the clauses `exactly_one`, `zeros`,
    `parameter_clauses` and `evidence`; with `chains`, also `chains`
    (rows written as a chain), `switches` and `chain_clauses`; with
    `enc5`, also `constants` (a variable and a clause each).

    `chains` writes each conditional table of its scope's last variable as
    Sang, Beame and Kautz's chains, the other tables as `enc1`. A chain
    writes the row at an assignment of the
    parents (their indicators, the context c) over the values v_1 ... v_m
    of nonzero entry, in the order of the values: switches s_1 ...
    s_(m-1), and the clauses `c & -s_1 & ... & -s_(j-1) & s_j -> v_j` and
    `c & -s_1 & ... & -s_(m-1) -> v_m`. Switch s_j weighs q_j = p_j / (p_j
    + ... + p_m) when true and 1 - q_j when false: where c holds, the
    first true switch chooses the child's value, at the product of the
    weights before it, p_j, and the switches after it sum to 1; where c
    does not, every clause holds and the switches sum to 1. A row of one
    nonzero entry is the clause `c -> v`; the other values' zeros follow
    from it and the exactly-one clauses.

    `enc5` writes every table with a parameter P per distinct entry 0 < p
    < 1, weighing p when true and 1 - p when false, and for each entry of
    p the clause `cube -> P`, its assignment's indicators; an entry of 0 is
    the clause that rules its assignment out, an entry of 1 nothing. The
    cubes of each distinct entry, zeros included, are merged first
    (`merged`). In each assignment of the scope one entry's cube holds:
    its parameter is true, weighing the entry, and every other parameter
    of the table sums out to p + (1 - p) = 1. A table with an entry above
    1 is divided by its largest entry first, so that every weight is at
    least 0, and the largest is a constant: a variable of that weight
    that a unit clause makes true."""
    if encoding not in ENCODINGS:
        raise ValueError(f'no encoding {encoding!r}; one of {", ".join(ENCODINGS)}')
    chains = encoding == 'chains'
    literals, n = indicators(sizes)
    counts = {'indicators': n, 'parameters': 0, 'exactly_one': 0, 'zeros': 0, 'parameter_clauses': 0,
              'evidence': 0}
    if chains:
        counts.update(chains=0, switches=0, chain_clauses=0)
    if encoding == 'enc5':
        counts.update(constants=0)
    clauses, weights = [], []

    def clause(lits):
        clauses.append(' '.join(map(str, lits)) + (' 0' if lits else '0'))

    for lits in literals:
        if len(lits) > 2:
            clause(lits)
            for a in range(len(lits)):
                for b in range(a + 1, len(lits)):
                    clause([-lits[a], -lits[b]])
            counts['exactly_one'] += 1 + len(lits) * (len(lits) - 1) // 2
    for scope, table in zip(scopes, tables):
        if len(set(scope)) != len(scope):
            raise ValueError(f'a scope that names a variable twice: {scope}')
        if len(table) != math.prod(sizes[v] for v in scope):
            raise ValueError(f'a table of {len(table)} entries for a scope of sizes {[sizes[v] for v in scope]}')
        bad = [w for w in table if not (w >= 0 and math.isfinite(w))]
        if bad:
            raise ValueError(f'an entry of {bad[0]}, which is no weight')
        strides = [math.prod(sizes[u] for u in scope[i + 1:]) for i in range(len(scope))]
        if chains and conditional(sizes, scope, table):
            child, k = scope[-1], sizes[scope[-1]]
            for r in range(0, len(table), k):
                ctx = [literals[v][r // stride % sizes[v]] for v, stride in zip(scope[:-1], strides[:-1])]
                ctx = [-l for l in ctx if l is not None]
                row = [(u, w) for u, w in enumerate(table[r:r + k]) if w > 0]
                if literals[child][0] is None:
                    continue   # a variable of one value
                if len(row) == 1:
                    clause(ctx + [literals[child][row[0][0]]])
                    counts['chain_clauses'] += 1
                    continue
                rest = [math.fsum(w for _, w in row[j:]) for j in range(len(row))] + [0.0]
                switches = []
                for j, (u, w) in enumerate(row):
                    if j < len(row) - 1:
                        n += 1
                        weights.append(f'c p weight {n} {w / rest[j]!r} 0\nc p weight {-n} {rest[j + 1] / rest[j]!r} 0\n')
                        clause(ctx + switches + [-n, literals[child][u]])
                        switches.append(n)
                    else:
                        clause(ctx + switches + [literals[child][u]])
                counts['chains'] += 1
                counts['switches'] += len(row) - 1
                counts['chain_clauses'] += len(row)
            continue
        if encoding == 'enc5':
            top = max(table, default=1)
            if top > 1:
                n += 1
                clause([n])
                weights.append(f'c p weight {n} {top!r} 0\nc p weight {-n} 1 0\n')
                counts['constants'] += 1
                table = [w / top for w in table]
            classes = {}
            for j, w in enumerate(table):
                if w != 1:
                    classes.setdefault(w, []).append(tuple(j // stride % sizes[v] for v, stride in zip(scope, strides)))
            for w, cubes in classes.items():
                if w > 0:
                    n += 1
                    counts['parameters'] += 1
                    weights.append(f'c p weight {n} {w!r} 0\nc p weight {-n} {1 - w!r} 0\n')
                for cube in merged(cubes, [sizes[v] for v in scope]):
                    lits = [-literals[v][u] for v, u in zip(scope, cube) if u is not None and literals[v][u] is not None]
                    if w > 0:
                        clause(lits + [n])
                        counts['parameter_clauses'] += 1
                    else:
                        clause(lits)
                        counts['zeros'] += 1
            continue
        for j, w in enumerate(table):
            if w == 1:
                continue
            conj = [literals[v][j // stride % sizes[v]] for v, stride in zip(scope, strides)]
            conj = [l for l in conj if l is not None]
            if w == 0:
                clause([-l for l in conj])
                counts['zeros'] += 1
                continue
            n += 1
            counts['parameters'] += 1
            for l in conj:
                clause([-n, l])
            clause([n] + [-l for l in conj])
            counts['parameter_clauses'] += len(conj) + 1
            weights.append(f'c p weight {n} {w!r} 0\nc p weight {-n} 1 0\n')
    seen = set()
    for v, u in evidence:
        if not (0 <= v < len(sizes) and 0 <= u < sizes[v]):
            raise ValueError(f'evidence {v} = {u} beyond the network')
        if v in seen:
            raise ValueError(f'variable {v} observed twice')
        seen.add(v)
        if literals[v][u] is not None:
            clause([literals[v][u]])
            counts['evidence'] += 1
    counts['variables'] = n
    counts['clauses'] = len(clauses)
    text = f'c t wmc\np cnf {n} {len(clauses)}\n' + ''.join(weights) + '\n'.join(clauses) + ('\n' if clauses else '')
    return text, counts
