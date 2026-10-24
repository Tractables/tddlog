#!/usr/bin/env python3
"""The programs of Geni's evaluation (Li and Zhang, "Compiling with
Generating Functions", ICFP 2025; its artifact, MIT licence,
data/geni/README.md) that Dice runs, as Dice programs, as ProbLog
programs, and their answers.

A family and its sizes give one program:

  link N      an N x N grid of routers; a packet leaves the top-left one
              and crosses each link with probability 0.5: a router is
              reached when its left, upper or upper-left neighbour is
              reached and the link from it works (the artifact's
              bench/ICFP/grid-networks/link-failure.dice.py, mode
              `3-flow`, p 0.5); P(the bottom-right router is reached).
              Fierens et al.'s grid (TPLP 2015), which ProbLog's own
              evaluation used; the paper's Figures 10 and 11.
  router N    an N x N grid of routers, each working with a probability
              that depends on whether its left and upper neighbours work
              (router-failure.dice.py with the artifact's parameters: 0.5
              for the first router, 0.5000001 and 0.5000002 on the edges
              after a working and a failed neighbour, 0.5000003 to
              0.5000006 inside for the four cases of left and upper);
              P(the bottom-right router works); the paper's Figure 11.
  weather N H Rubicon's translation (Holtzen et al., CAV 2021) of the
              weather factory, a Markov chain of N factories, each on
              strike or not, whose strikes depend on the weather of the
              day, a chain of its own; P(all N factories are on strike
              within H days), the first of the program's value (the
              second is `false`); the paper's Figure 9. The artifact's
              bench/ICFP/weather-factory/rubicon-dice/weather-factory-N-H.dice
              for N in 2, 7 to 13 and 15 to 19 at H = 10, and N = 10 at
              H = 1 to 10 and 100 to 1000 in steps of 100. The programs at
              H = 10 are in data/geni/weather_factory; the program at
              another H is the one at H = 10 with H steps in place of 10,
              which is the artifact's file at each H it has.

The Dice programs of link and router are the artifact's generators',
written here anew (their text the same). The ProbLog program goes to
standard output, its first line `% gen_geni.py FAMILY SIZES`, the rest
dice2problog.py's translation of the Dice program. `--dice PATH` also
writes the Dice program. `--reference` prints the answers instead, `ATOM:
PROBABILITY` a line, from a dynamic program over the model's states, never
from either program: for link and router over the grid's frontier, row
by row (2^(N+1) states for `link`, 2^N for `router`); for weather over
the chain's 2^(N+1) states, day by day, with the transition
probabilities read from the Dice program (`weather_chain`).
`--reference-of PROGRAM` prints those of a program this script wrote,
read from its first line.
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import source_tree  # noqa: E402,F401  (the source tree's converters)
from dice2problog import translate  # noqa: E402

HEADER = '% gen_geni.py'
FAMILIES = {'link': 1, 'router': 1, 'weather': 2}      # the number of sizes
LINK_P = 0.5
ROUTER_INIT = 0.5
ROUTER_EDGE = (0.5000001, 0.5000002)                      # after a working, a failed neighbour
ROUTER_INNER = (0.5000003, 0.5000004, 0.5000005, 0.5000006)  # left and up: TT, TF, FT, FF
WEATHER_DIR = os.path.join(os.path.dirname(HERE), 'data', 'geni', 'weather_factory')
WEATHER_H = 10                                            # the horizon of the stored programs


def link_dice(n):
    lines = []
    for x in range(1, n + 1):
        for y in range(1, n + 1):
            if x == 1 and y == 1:
                lines.append('let conn_1_1 = true in')
            elif x == 1:
                lines.append(f'let conn_{x}_{y} = (conn_{x}_{y - 1} && flip {LINK_P}) in')
            elif y == 1:
                lines.append(f'let conn_{x}_{y} = (conn_{x - 1}_{y} && flip {LINK_P}) in')
            else:
                lines.append(f'let conn_{x}_{y} = ((conn_{x}_{y - 1} && flip {LINK_P}) || '
                             f'(conn_{x - 1}_{y} && flip {LINK_P}) || (conn_{x - 1}_{y - 1} && flip {LINK_P})) in')
    lines.append(f'conn_{n}_{n}')
    return '\n'.join(lines) + '\n'


def router_dice(n):
    e, i = ROUTER_EDGE, ROUTER_INNER
    lines = []
    for r in range(n):
        for c in range(n):
            if r == 0 and c == 0:
                lines.append(f'let x_0_0 = flip ({ROUTER_INIT}) in')
            elif r == 0:
                lines.append(f'let x_{r}_{c} = if x_{r}_{c - 1} then flip ({e[0]}) else flip ({e[1]}) in')
            elif c == 0:
                lines.append(f'let x_{r}_{c} = if x_{r - 1}_{c} then flip ({e[0]}) else flip ({e[1]}) in')
            else:
                lines.append(f'let x_{r}_{c} = if x_{r}_{c - 1} then (if x_{r - 1}_{c} then flip ({i[0]}) else '
                             f'flip ({i[1]})) else (if x_{r - 1}_{c} then flip ({i[2]}) else flip ({i[3]})) in')
    lines.append(f'x_{n - 1}_{n - 1}')
    return '\n'.join(lines) + '\n'


WEATHER_STEP = ['let _goal = if !_goal then hit(res) else _goal in', 'let res = if !_goal then step(res) else res in']
WEATHER_LAST = '(hit(res), false)'
WEATHER_START = 'let _goal = false in '                # (with its space)


def weather_dice(n, h):
    """The artifact's weather-factory-N-H.dice: the stored program at
    H = 10 with H steps (each a test of the goal and a step) in place of
    its 10."""
    path = os.path.join(WEATHER_DIR, f'weather-factory-{n}-{WEATHER_H}.dice')
    if not os.path.exists(path):
        have = sorted(int(f.split('-')[2]) for f in os.listdir(WEATHER_DIR) if f.endswith(f'-{WEATHER_H}.dice'))
        raise SystemExit(f'weather: no program of {n} factories in {WEATHER_DIR} (it has {have})')
    if h < 1:
        raise SystemExit('weather: the horizon is at least 1')
    with open(path) as f:
        lines = f.read().split('\n')
    k = lines.index(WEATHER_START) + 2           # after `let res = (initial state) in`
    if lines[k:] != WEATHER_STEP * WEATHER_H + [WEATHER_LAST]:
        raise SystemExit(f'{path}: not {WEATHER_H} steps followed by `{WEATHER_LAST}`')
    return '\n'.join(lines[:k] + WEATHER_STEP * h + [WEATHER_LAST])


# --- Geni's programs ---------------------------------------------------------
# The programs of the artifact's `gennifer` (the .gir language), which
# `bench.py --geni` runs: the grids as the artifact's
# bench/ICFP/grid-networks/{link,router}-failure.gennifer.py write them (a
# flip is `sample(Bernoulli(p))`), the weather factory from the artifact's
# bench/ICFP/weather-factory/rubicon-geni/weather-factory-N-10.gir with H
# days in place of 10.

def _bern(text):
    """`flip P` or `flip (P)` as `sample(Bernoulli(P))`."""
    return re.sub(r'flip (?:\(([0-9.]+)\)|([0-9.]+))', lambda m: f'sample(Bernoulli({m.group(1) or m.group(2)}))', text)


def link_gir(n):
    return _bern(link_dice(n))


def router_gir(n):
    return _bern(router_dice(n))


GIR_DAY = re.compile(r'(sun|state_\d+)_(\d+)')


def weather_gir(n, h, bench):
    """The artifact's weather-factory-N-H.gir: its program at H = 10 with
    H days (the line that tests the goal, then the update of the sun and
    of each factory) in place of 10, the days' names numbered on."""
    path = os.path.join(bench, 'weather-factory', 'rubicon-geni', f'weather-factory-{n}-{WEATHER_H}.gir')
    if not os.path.exists(path):
        raise SystemExit(f'weather: no {path} (the artifact\'s bench/ICFP is --geni-benchmarks)')
    if h < 1:
        raise SystemExit('weather: the horizon is at least 1')
    with open(path) as f:
        lines = f.read().rstrip('\n').split('\n')
    first = next(i for i, l in enumerate(lines) if l.startswith('if !('))
    day = n + 2                                       # the goal test, the sun and n factories
    head, body, last = lines[:first], lines[first:-1], lines[-1]
    if len(body) != day * WEATHER_H:
        raise SystemExit(f'{path}: not {WEATHER_H} days of {day} lines')
    first_day = body[:day]                            # the test of day 0, the updates of day 1
    out = list(head)
    for d in range(h):
        for l in first_day:
            out.append(GIR_DAY.sub(lambda m: f'{m.group(1)}_{int(m.group(2)) + d}', l))
    out.append(GIR_DAY.sub(lambda m: f'{m.group(1)}_{int(m.group(2)) - WEATHER_H + h}', last))
    return '\n'.join(out) + '\n'


def gir_program(family, sizes, bench):
    if family == 'weather':
        return weather_gir(*sizes, bench)
    return {'link': link_gir, 'router': router_gir}[family](*sizes)


def dice_program(family, sizes):
    return {'link': link_dice, 'router': router_dice, 'weather': weather_dice}[family](*sizes)


# --- reference answers -----------------------------------------------------

def link_reference(n):
    """P(conn_n_n): the joint of the frontier, bits b[0..n-1] (columns
    left of the current one in the current row, the others in the row
    above) and d (the row above at the column left of the current one),
    as an array of 2^(n+1) probabilities, updated cell by cell."""
    import numpy as np

    def bit(axis):
        return np.arange(2).reshape([2 if j == axis else 1 for j in range(n + 1)])
    a = np.zeros((2,) * (n + 1))
    a[(0,) * (n + 1)] = 1.0
    for x in range(n):
        for y in range(n):
            # the number of reached neighbours whose link is tried: left
            # b[y-1], up b[y], upper-left d
            k = np.zeros([1] * (n + 1), dtype=int)
            if y > 0:
                k = k + bit(y - 1)
            if x > 0:
                k = k + bit(y)
            if x > 0 and y > 0:
                k = k + bit(n)
            miss = (1.0 - LINK_P) ** k if (x, y) != (0, 0) else np.zeros([1] * (n + 1))
            g = np.stack([miss, 1.0 - miss], axis=-1)       # P(new | frontier)
            t = a[..., None] * g
            s = t.sum(axis=n)                               # b[0..n-1] (b[y] the old up), new
            a = np.swapaxes(s, y, n)                        # b[y] = new, d = the old b[y]
    m = a.sum(axis=tuple(j for j in range(n + 1) if j != n - 1))
    return [float(m[1])]


def router_reference(n):
    """P(x_{n-1}_{n-1}): the joint of the frontier row, bits b[0..n-1],
    as an array of 2^n probabilities, updated cell by cell."""
    import numpy as np
    a = np.zeros((2,) * n)
    a[(0,) * n] = 1.0
    e, i = ROUTER_EDGE, ROUTER_INNER
    for r in range(n):
        for c in range(n):
            # P(works | left, up) over [left, up]
            if r == 0 and c == 0:
                p = np.full([2, 2], ROUTER_INIT)
            elif r == 0:
                p = np.array([[e[1], e[1]], [e[0], e[0]]])           # by left
            elif c == 0:
                p = np.array([[e[1], e[0]], [e[1], e[0]]])           # by up
            else:
                p = np.array([[i[3], i[2]], [i[1], i[0]]])           # [left][up]
            f = np.stack([1.0 - p, p], axis=-1)                       # [left, up, new]
            if c == 0:
                f = f[:1]                                             # no left neighbour
                g = f[0].reshape([2 if j == c else 1 for j in range(n)] + [2])
            else:
                g = f.reshape([2 if j in (c - 1, c) else 1 for j in range(n)] + [2])
            t = a[..., None] * g
            s = t.sum(axis=c)                                         # the old b[c] summed out
            a = np.moveaxis(s, -1, c)
    m = a.sum(axis=tuple(j for j in range(n) if j != n - 1))
    return [float(m[1])]


# One update of the weather factory's step function: under the condition,
# draw `discrete(P, Q)` and take the first value with probability P.
WEATHER_UPDATE = re.compile(
    r'if (.+?) then\s*\n\s*let _sel_update_(\d+) = discrete\(([0-9.]+), ([0-9.]+)\) in\s*\n'
    r'\s*if _sel_update_\2 == int\(1, 0\) then\s*\n\s*\(\s*(.+?)\s*\)\s*\n\s*else\s*\n\s*\(\s*(.+?)\s*\)\s*\n')


def tautology(guard):
    """Whether a Dice Boolean expression of variables, `!`, `&&` and `||`
    holds in every state, decided conjunct by conjunct, each over the
    states of its own variables (at most 4): Rubicon's guard is a
    conjunction of one tautology a variable."""
    import ast
    import itertools
    expr = ast.parse(guard.replace('&&', ' and ').replace('||', ' or ').replace('!', ' not '), mode='eval').body

    def holds(node):
        if isinstance(node, ast.BoolOp) and isinstance(node.op, ast.And):
            return all(holds(v) for v in node.values)
        names = sorted({x.id for x in ast.walk(node) if isinstance(x, ast.Name)})
        if len(names) > 4:
            return False
        code = compile(ast.Expression(node), '<guard>', 'eval')
        return all(eval(code, {'__builtins__': {}}, dict(zip(names, vals)))
                   for vals in itertools.product([False, True], repeat=len(names)))
    return holds(expr)


def weather_chain(text):
    """The Markov chain of a weather factory program, read from its Dice
    text: (n, initial state, P(sun' | sun) as [sun], P(strike_i' |
    strike_i, sun) as [i][strike][sun], horizon). It checks that the
    program has the shape Rubicon writes for the model and fails
    otherwise: one command whose guard holds in every state, the sun
    updated from the sun alone, factory i from its own strike and the
    sun, the goal the conjunction of the strikes, tested before each
    step and after the last, and the value `(hit(res), false)`."""
    m = re.search(r'fun hit\(packedstate : (.*?)\) \{', text)
    n = m.group(1).count('bool') - 1
    names = ['sun'] + [f'state{i}' for i in range(1, n + 1)]
    for k, v in enumerate(names):
        # the k-th of the nested pairs (sun, (state1, (..., staten)))
        e = 'packedstate'
        for j in range(k):
            e = f'snd {e}' if j == 0 else f'snd ({e})'
        if k < n:
            e = f'fst {e}' if k == 0 else f'fst ({e})'
        if text.count(f'let {v} = {e} in \n') != 2:
            raise SystemExit(f'weather: {v} is not the {k}-th of the state in hit and step')
    goal = re.findall(r'let allStrike = (.+) in\n', text)
    if len(goal) != 2 or {re.sub(r'[()\s]', '', g) for g in goal} != {'&&'.join(names[1:])} or \
            'allStrike\n}' not in text:
        raise SystemExit('weather: the goal is not the conjunction of the strikes')
    step = text[text.index('fun step('):]
    if step.count('_cmd_act_') != 3 or '(if _cmd_act_1 then int(1, 1) else int(1, 0))' not in step:
        raise SystemExit('weather: not one command')
    guard = re.search(r'let _cmd_act_1 = (.+) in\n', step).group(1)
    if not tautology(guard):
        raise SystemExit('weather: the guard of the command does not always hold')
    updates = WEATHER_UPDATE.findall(step)
    sun_p = [None, None]
    strike_p = [[[None, None], [None, None]] for _ in range(n)]
    for cond, k, p, q, first, second in updates:
        k = int(k)
        if abs(float(p) + float(q) - 1) > 1e-12:
            raise SystemExit(f'weather: discrete({p}, {q}) does not sum to 1')
        p = float(p)
        if k == 0:
            s = {'sun': True, '!(sun)': False}[cond]
            if (first, second) != ('sun', '!(sun)'):
                raise SystemExit('weather: the sun keeps or changes its value')
            sun_p[s] = p if s else 1 - p                    # P(sun' = true | sun = s)
        else:
            v = names[k]
            cases = {f'({v} && sun)': (1, 1), f'(!({v}) && sun)': (0, 1), f'({v} && !(sun))': (1, 0),
                     f'(!({v}) && !(sun))': (0, 0)}
            if cond not in cases or (first, second) != ('true', 'false'):
                raise SystemExit(f'weather: update of {v} under {cond}')
            st, s = cases[cond]
            strike_p[k - 1][st][s] = p
    if None in sun_p or any(None in row for f in strike_p for row in f):
        raise SystemExit('weather: an update is missing')
    init = re.search(r'let _goal = false in \nlet res = (.+) in\n', text).group(1)
    init = [v == 'true' for v in re.findall(r'true|false', init)]
    if len(init) != n + 1:
        raise SystemExit('weather: the initial state is not n + 1 Booleans')
    tail = text[text.index(WEATHER_START + '\n'):].split('\n')[2:]
    h = len(tail) // 2
    if tail != WEATHER_STEP * h + [WEATHER_LAST]:
        raise SystemExit('weather: the steps are not tests of the goal and steps')
    return n, init, sun_p, strike_p, h


def weather_reference(n, h):
    """P(all strike within h days) and P(false) = 0: the distribution
    over the 2^(n+1) states (axis 0 the sun, axis i factory i) of the
    days the goal has not been reached, day by day; the goal's mass is
    taken out when reached and summed."""
    import numpy as np
    n2, init, sun_p, strike_p, h2 = weather_chain(dice_program('weather', [n, h]))
    assert (n2, h2) == (n, h)
    a = np.zeros((2,) * (n + 1))
    a[tuple(int(v) for v in init)] = 1.0
    reached = 0.0
    goal = (slice(None),) + (1,) * n
    for day in range(h + 1):
        reached += float(a[goal].sum())
        a[goal] = 0.0
        if day == h:
            break
        for i in range(n):
            # M[sun, strike, strike']
            p = np.array(strike_p[i]).T                     # [sun][strike] = P(strike' | strike, sun)
            M = np.stack([1.0 - p, p], axis=-1)
            b = np.moveaxis(a, i + 1, 1).reshape(2, 2, -1)
            b = np.einsum('asr,ast->atr', b, M)
            a = np.moveaxis(b.reshape((2,) * (n + 1)), 1, i + 1)
        S = np.array([[1.0 - sun_p[0], sun_p[0]], [1.0 - sun_p[1], sun_p[1]]])   # [sun][sun']
        a = np.tensordot(S, a, axes=([0], [0]))
    return [reached, 0.0]


def reference(family, sizes):
    return {'link': link_reference, 'router': router_reference, 'weather': weather_reference}[family](*sizes)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('family', nargs='?', choices=list(FAMILIES))
    ap.add_argument('sizes', nargs='*', type=int)
    ap.add_argument('--dice', help='also write the Dice program here')
    ap.add_argument('--gir', help="also write Geni's program here (weather: from the artifact's bench/ICFP, "
                    '--geni-benchmarks)')
    ap.add_argument('--geni-benchmarks', default=os.path.expanduser('~/local/geni/src/bench/ICFP'),
                    help="the Geni artifact's bench/ICFP directory (default ~/local/geni/src/bench/ICFP)")
    ap.add_argument('--reference', action='store_true', help='print the answers instead of the program')
    ap.add_argument('--reference-of', metavar='PROGRAM', help='print the answers of a program this script wrote')
    a = ap.parse_args()
    query = None
    if a.reference_of:
        if a.family or a.sizes or a.dice or a.reference:
            raise SystemExit('--reference-of reads the family and sizes from the program and takes nothing else')
        with open(a.reference_of) as f:
            text = f.read()
        head = text.split('\n', 1)[0].split()
        if head[:2] != HEADER.split() or len(head) < 3 or head[2] not in FAMILIES:
            raise SystemExit(f'{a.reference_of}: not a program of gen_geni.py (its first line is not '
                             f'`{HEADER} FAMILY SIZES`)')
        a.family, a.sizes, a.reference = head[2], [int(s) for s in head[3:]], True
        query = re.findall(r'^query\((.+)\)\.$', text, re.M)
    if a.family is None:
        raise SystemExit('give FAMILY and SIZES, or --reference-of PROGRAM')
    if len(a.sizes) != FAMILIES[a.family]:
        raise SystemExit(f'{a.family} takes {FAMILIES[a.family]} size(s): ' +
                         {'link': 'N', 'router': 'N', 'weather': 'N H'}[a.family])
    if a.gir and a.reference:
        raise SystemExit('--gir writes a program, --reference prints answers instead')
    if a.family != 'weather' and a.sizes[0] < 2:
        raise SystemExit('size: at least 2')
    dice = dice_program(a.family, a.sizes)
    if a.reference:
        if a.dice:
            raise SystemExit('--reference prints the answers and writes no program; --dice has nothing to write')
        if query is None:
            query = re.findall(r'^query\((.+)\)\.$', translate(dice), re.M)
        values = reference(a.family, a.sizes)
        if len(query) != len(values):
            raise SystemExit(f'expected {len(values)} queries, found {len(query)}')
        for q, x in zip(query, values):
            print(f'{q}: {x!r}')
        return
    sys.stdout.write(f'{HEADER} {a.family} {" ".join(map(str, a.sizes))}\n' + translate(dice))
    if a.dice:
        with open(a.dice, 'w') as f:
            f.write(dice)
    if a.gir:
        with open(a.gir, 'w') as f:
            f.write(gir_program(a.family, a.sizes, a.geni_benchmarks))


if __name__ == '__main__':
    main()
