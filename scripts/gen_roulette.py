#!/usr/bin/env python3
"""The scaling programs of Roulette's evaluation (Moy, Czenszak, Li,
Marshall and Holtzen, "Roulette: A Language for Expressive, Exact, and
Efficient Discrete Probabilistic Programming", PLDI 2025; the artifact's
benchmark/ directory, data/roulette/README.md) as ProbLog programs, as
Dice programs, and their answers.

A family and a size give one program. The ProbLog program goes to
standard output, its first line `% gen_roulette.py FAMILY SIZE`; `--dice
PATH` also writes the Dice program, whose value answers the ProbLog
program's queries as bench.py reads Dice's table (a Boolean for one
query, a tuple of Booleans for one query each, an integer whose value k
is the k-th query). `--reference` prints the answers instead, one `ATOM:
PROBABILITY` a line, from a dynamic program over the model's states
(`reference`), never from either program, and for a program with
evidence (caesar and the hmm queries but the marginal likelihood) the
evidence's probability, `P(e): PROBABILITY`, as tddlog's
`--evidence-probability` prints it; `--reference-of PROGRAM` those of a
program it wrote, read from its first line. A probability below the
smallest normal double is printed with its decimal exponent
(`1.234e-400`) and stands for 0 in a double.

Families (the artifact's file; its sizes; the size here):
  motivating   modular/motivating.rkt, the Markov chain of the paper's
               Figure 2: x0 ~ flip 0.1, x_k = if x_{k-1} then flip 0.2
               else flip 0.3; P(x_N). N steps (artifact to 5000).
  diamond      modular/diamond.rkt: N diamonds in a row, each passing its
               input along one of two routes chosen by flip 0.5, one of
               which drops it with probability 0.0001; P(through).
  ladder       modular/ladder.rkt: a ladder of N rungs over two wires;
               the joint of the two last wires, (true, false), (false,
               true) or neither, queried as the two marginals.
  caesar       modular/caesar.rkt: a key uniform over 26 letters, N
               letters drawn from English letter frequencies, each seen
               as letter + key (no wrap around: the benchmark adds 32-bit
               vectors) or, with probability 0.0001, as itself; the
               posterior of the key given the N observations of
               data/roulette/caesar_obs.txt (the benchmark's, from
               Racket's generator seeded with 0). N letters (to 300).
               Roulette draws each categorical by bisection (`bin-cat`):
               a flip with the left half's mass, then the half's values
               renormalized. The program here makes the same flips and
               holds each value as its bits, an if-then-else of a node's
               flip over its halves' bits, and adds the letter and the key
               as bvadd does, a carry rippling from bit 0. A value's
               probability is the product of the flips' on the way to it,
               which differs from the table as written (the key's 26 x
               0.038461538 sum to 1 - 1.2e-8) in the eighth digit.
  hmm-QUERY    hmm.rkt: a hidden chain h0 ~ flip 0.5, h_k ~ flip 0.3 after
               true and flip 0.65 after false, and observations v_k ~ flip
               0.65 when h_k and flip 0.3 otherwise; N steps. QUERY:
               marginal-likelihood P(v_1..v_N all true);
               filtering-marginal P(h_N | all v true); smoothing-marginal
               P(h_0 | all v true); filtering-joint P(h_{N/2}..h_N all
               true | all v true); smoothing-joint P(h_0..h_{N/2-1} all
               true | all v true) (N/2 rounded down). The artifact runs 1,
               100, 500, 1000 and 2000 steps.
  network      scaling/network.rkt: a 4-bit message 1111 sent through K
               diamonds, each sending it with probability 0.5 to a noisy
               node, which with probability 0.01 flips one of its bits
               chosen uniformly, or else to a node that forwards it with
               probability 0.99 and loses it otherwise; the distribution of
               the message, m(K, V) for V = 0..15 (bit i of the message is
               bit i of V), and m(K, 16) for a lost message. K diamonds
               (to 100; the paper's
               to 400). Roulette only in the paper. Both programs here
               hold the message as Roulette does, four bits, and a flag
               for a lost message.
  hardware     scaling/hardware.rkt: an interpreter of a loop over a
               memory of two cells of BW bits (total, counter) whose
               every read and write first resets the cell to 0 with
               probability 0.0001 (`mem-perturb`), running `tri n`: counter
               := n, then while counter != 0, total := total + counter and
               counter := counter + (int -1). The benchmark's (int -1) is
               the bit vector 1 (natural->bv of -1 sets bit 0 only), so
               the counter counts up to 2^BW and wraps to 0. The loop runs
               on FUEL: entered with none left, it ends the run with no
               result. The distribution of the total, res(T) for T = 0 ..
               2^BW - 1, and res(2^BW) for a run out of fuel. The size is
               FUEL; BW = 2 and n = FUEL
               - 2 as in the artifact's fuel ladder (n saturates at 2^BW -
               1). A write's reset is overwritten at once and is left out.
               The cells are bit vectors, added as bvadd adds them.

The ProbLog programs state the facts in the order Roulette creates its
flips and hold its bit vectors as bits, not one atom a value; the queries
are what Roulette's query computes, the distribution of the program's
value. The Dice programs are as the artifact's Dice generators (gen-*.rkt,
hmm.rkt's dice mode) write them, where it has one.

Usage: gen_roulette.py FAMILY SIZE [--dice PATH] [--reference]
       gen_roulette.py --reference-of PROGRAM.pl
"""
import argparse
import math
import os
import sys
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
OBS = os.path.join(HERE, '..', 'data', 'roulette', 'caesar_obs.txt')

LETTERS = [0.08167, 0.01492, 0.02782, 0.04253, 0.04378, 0.02228, 0.02015, 0.06094, 0.06966, 0.0153,
           0.0772, 0.04025, 0.02406, 0.06749, 0.07507, 0.01929, 0.00095, 0.05987, 0.06327, 0.09056,
           0.02758, 0.00978, 0.02360, 0.00150, 0.01974, 0.00074]
KEY = [0.038461538] * 26
FAIL = 0.0001
HMM_QUERIES = ['marginal-likelihood', 'filtering-marginal', 'smoothing-marginal', 'filtering-joint',
               'smoothing-joint']
FAMILIES = ['motivating', 'diamond', 'ladder', 'caesar', 'network', 'hardware'] + [f'hmm-{q}' for q in HMM_QUERIES]
HW_BW = 2
PERTURB = 0.0001


def p(x):
    """A probability as written into a program: the shortest decimal that
    reads back as the same double."""
    return repr(float(x))


def bisect(ps):
    """Roulette's `bin-cat` over the values 0..n-1 of a categorical with
    masses `ps`, as its tree: a value at a leaf, `(s, left, right)` at a
    node whose flip, true with probability s, takes the left half (the
    first n // 2 values); s is the left half's mass, renormalized, in IEEE
    doubles as Racket computes it (`(foldl + 0 ...)` from the left, `(- 1
    left-sum)`, `/`)."""
    def go(items):
        if len(items) == 1:
            return items[0][0]
        h = len(items) // 2
        left, right = items[:h], items[h:]
        s = 0
        for _, x in left:
            s = x + s
        rest = 1 - s
        return (s, go([(v, x / s) for v, x in left]), go([(v, x / rest) for v, x in right]))
    return go(list(enumerate(ps)))


def bin_cat(ps):
    """The probability Roulette's `bin-cat` gives each value 0..n-1: the
    product of the probabilities of the flips on the way to it."""
    out = {}

    def go(t, f):
        if isinstance(t, int):
            out[t] = f
        else:
            go(t[1], f * Fraction(t[0]))
            go(t[2], f * Fraction(1 - t[0]))
    go(bisect(ps), Fraction(1))
    return [float(out[v]) for v in range(len(ps))]


# Bits as ProbLog: a literal is an atom, `\+atom`, True or False.

def neg(x):
    return (not x) if isinstance(x, bool) else x[2:] if x.startswith('\\+') else '\\+' + x


def gate(lines, atom, terms, keep=False):
    """`atom` as the disjunction of the conjunctions `terms` (lists of
    literals), constants folded: a constant, or the literal of a single
    term of one literal, is returned as it is; otherwise `atom` gets a rule
    a term, appended to `lines`, and is returned. `keep` always defines
    `atom` (as a fact, or `atom :- fail.`)."""
    out = []
    for t in terms:
        if any(x is False for x in t):
            continue
        t = [x for x in t if x is not True]
        if not t:
            if keep:
                lines.append(f'{atom}.')
                return atom
            return True
        out.append(t)
    if not keep and not out:
        return False
    if not keep and len(out) == 1 and len(out[0]) == 1:
        return out[0][0]
    lines += [f'{atom} :- {", ".join(t)}.' for t in out] or [f'{atom} :- fail.']
    return atom


def equals(bits, v):
    """The literals that make `bits` (least significant first) read v."""
    return [b if v >> k & 1 else neg(b) for k, b in enumerate(bits)]


def adder(lines, atom, a, b, carry_out=False):
    """The bits of a + b as `bvadd` makes them, the carry rippling from bit
    0: bit k is `atom('s', k)`, over `atom('x', k)`, a_k xor b_k, and the
    carry into it, `atom('c', k)`; with `carry_out`, the carry out of the
    last bit is one bit more."""
    c, s = False, []
    for k, (x, y) in enumerate(zip(a, b)):
        h = gate(lines, atom('x', k), [[x, neg(y)], [neg(x), y]])
        s.append(gate(lines, atom('s', k), [[h, neg(c)], [neg(h), c]]))
        if k + 1 < len(a) or carry_out:
            c = gate(lines, atom('c', k + 1), [[x, y], [c, h]])
    return s + [c] if carry_out else s


def categorical(lines, flip, bit, ps, width):
    """A categorical as Roulette's `bin-cat` draws it: the fact `S::FLIP(T)`
    at each node T of its `bisect` tree, numbered in the order Roulette
    makes its flips (a node, then its left half's, then its right half's),
    and bit k of the value under node T, `BIT(T,K)`, the node's flip's
    if-then-else over its halves' bits. Returns the value's `width` bits,
    least significant first."""
    facts, rules, count = [], [], [0]

    def go(t):
        if isinstance(t, int):
            return [bool(t >> k & 1) for k in range(width)]
        node, count[0] = count[0], count[0] + 1
        f = flip(node)
        facts.append(f'{p(t[0])}::{f}.')
        left, right = go(t[1]), go(t[2])
        return [x if x == y else gate(rules, bit(node, k), [[f, x], [neg(f), y]])
                for k, (x, y) in enumerate(zip(left, right))]
    bits = go(bisect(ps))
    lines += facts + rules
    return bits


def observations(n):
    with open(OBS) as f:
        obs = [int(line) for line in f]
    if n > len(obs):
        raise SystemExit(f'caesar: {OBS} holds {len(obs)} observations, not {n}')
    return obs[:n]


def ad(heads):
    return '; '.join(f'{p(q)}::{h}' for h, q in heads) + '.'


# --- ProbLog and Dice programs ---------------------------------------------

def motivating(n):
    pl = ['% Roulette motivating.rkt: x0 ~ flip 0.1, x_k = if x_{k-1} then flip 0.2 else flip 0.3.']
    for k in range(n, 0, -1):          # go(k) creates its flips before go(k-1)'s
        pl += [f'0.2::a({k}).', f'0.3::b({k}).']
    pl.append('0.1::x(0).')
    for k in range(1, n + 1):
        pl += [f'x({k}) :- x({k - 1}), a({k}).', f'x({k}) :- \\+x({k - 1}), b({k}).']
    pl.append(f'query(x({n})).')
    # gen-motivating.rkt
    dice = []
    for k in range(n, 0, -1):
        dice += [f'let a{k} = flip 0.2 in', f'let b{k} = flip 0.3 in']
    dice.append('let x0 = flip 0.1 in')
    for k in range(1, n + 1):
        dice.append(f'let x{k} = (if x{k - 1} then a{k} else b{k}) in')
    dice.append(f'x{n}')
    return pl, dice


def diamond(n):
    pl = ['% Roulette diamond.rkt: s_k = (route_k and s_{k-1}) or (not route_k and s_{k-1} and not drop_k).']
    for k in range(n, 0, -1):
        pl += [f'0.0001::drop({k}).', f'0.5::route({k}).']
    pl.append('s(0).')
    for k in range(1, n + 1):
        pl += [f's({k}) :- s({k - 1}), route({k}).', f's({k}) :- s({k - 1}), \\+route({k}), \\+drop({k}).']
    pl.append(f'query(s({n})).')
    # gen-diamond.rkt
    dice = []
    for k in range(n, 0, -1):
        dice += [f'let drop{k} = flip 0.0001 in', f'let route{k} = flip 0.5 in']
    dice.append('let s0 = true in')
    for k in range(1, n + 1):
        dice += [f'let t{k} = if route{k} then s{k - 1} else false in',
                 f'let u{k} = if route{k} then false else s{k - 1} in',
                 f'let s{k} = t{k} || (u{k} && !drop{k}) in']
    dice.append(f's{n}')
    return pl, dice


def ladder(n):
    pl = ['% Roulette ladder.rkt: from (true, false), a rung gives (false, false) if s1 and a (flip 0.001),',
          '% else (b, not b) with b ~ flip 0.5 if s1 or s2, else (false, false).']
    for k in range(n, 0, -1):
        pl += [f'0.001::a({k}).', f'0.5::b({k}).']
    pl.append('o1(0).')
    for k in range(1, n + 1):
        j = k - 1
        pl += [f'o1({k}) :- o1({j}), \\+a({k}), b({k}).', f'o1({k}) :- \\+o1({j}), o2({j}), b({k}).',
               f'o2({k}) :- o1({j}), \\+a({k}), \\+b({k}).', f'o2({k}) :- \\+o1({j}), o2({j}), \\+b({k}).']
    pl += [f'query(o1({n})).', f'query(o2({n})).']
    # gen-ladder.rkt
    dice = []
    for k in range(n, 0, -1):
        dice += [f'let a{k} = flip 0.001 in', f'let b{k} = flip 0.5 in']
    dice += ['let p0 = true in', 'let q0 = false in']
    for k in range(1, n + 1):
        j = k - 1
        dice += [f'let x{k} = (p{j} && a{k}) in', f'let y{k} = (p{j} || q{j}) in',
                 f'let p{k} = (if x{k} then false else if y{k} then b{k} else false) in',
                 f'let q{k} = (if x{k} then false else if y{k} then !b{k} else false) in']
    dice.append(f'(p{n}, q{n})')
    return pl, dice


def caesar(n):
    key, letter = bin_cat(KEY), bin_cat(LETTERS)
    obs = observations(n)
    pl = ['% Roulette caesar.rkt: key ~ 26 letters; letter i ~ English frequencies, seen as letter + key,',
          '% or with probability 0.0001 as itself. The categoricals as Roulette bisects them, their',
          '% values in bits (kb, db at a node of the bisection, kc and dc its flips), letter + key as',
          '% its bit vectors add (x the bits\' exclusive or, s the sum, c the carry).']
    kbits = categorical(pl, lambda t: f'kc({t})', lambda t, k: f'kb({t},{k})', KEY, 5)
    for i, o in enumerate(obs, 1):
        drawn = categorical(pl, lambda t: f'dc({i},{t})', lambda t, k: f'db({i},{t},{k})', LETTERS, 5)
        pl.append(f'{FAIL}::fail({i}).')
        enc = adder(pl, lambda kind, k: f'{kind}({i},{k})', kbits, drawn, carry_out=True)
        gate(pl, f'ok({i})', [[f'fail({i})'] + equals(drawn, o), [f'\\+fail({i})'] + equals(enc, o)], keep=True)
        pl.append(f'evidence(ok({i}), true).')
    for v in range(26):
        gate(pl, f'key({v})', [equals(kbits, v)], keep=True)
    pl += [f'query(key({v})).' for v in range(26)]
    # As caesar.dice.scrbl, but with 6-bit integers, which hold a letter
    # plus a key (at most 50) as the benchmark's 32-bit vectors do, where
    # its 5 bits wrap around at 32. A categorical is padded with zeros to
    # 64 values, Dice's discrete taking its width from their number.
    pad = lambda qs: ', '.join([p(q) for q in qs] + ['0.0'] * (64 - len(qs)))
    dice = ['fun drawChar(key: int(6), observation: int(6)) {',
            f'    let drawnChar = discrete({pad(letter)}) in',
            '    let encrypted = key + drawnChar in',
            f'    let fail = flip {FAIL} in',
            '    if fail then (observe observation == drawnChar)',
            '    else (observe observation == encrypted)',
            '}',
            f'let key1 = discrete({pad(key)}) in']
    dice += [f'let a = drawChar(key1, int(6, {o})) in' for o in obs]
    dice.append('key1')
    return pl, dice


def hmm(query, n):
    pl = [f'% Roulette hmm.rkt, {query}, {n} steps.', '0.5::h(0).']
    for k in range(1, n + 1):
        j = k - 1
        pl += [f'0.3::t1({k}).', f'0.65::t0({k}).', f'0.65::e1({k}).', f'0.3::e0({k}).',
               f'h({k}) :- h({j}), t1({k}).', f'h({k}) :- \\+h({j}), t0({k}).',
               f'v({k}) :- h({k}), e1({k}).', f'v({k}) :- \\+h({k}), e0({k}).']
    out = outcomes(query, n)
    if query != 'marginal-likelihood':
        pl += [f'evidence(v({k}), true).' for k in range(1, n + 1)]
    # q: the conjunction of the outcomes, as a chain.
    kind, idx = out
    if not idx:
        pl.append('q.')
    else:
        pl.append(f'c({idx[0]}) :- {kind}({idx[0]}).')
        pl += [f'c({b}) :- c({a}), {kind}({b}).' for a, b in zip(idx, idx[1:])]
        pl.append(f'q :- c({idx[-1]}).')
    pl.append('query(q).')
    # hmm.rkt's gen-hmm-dice
    dice = ['fun nextState(prevHidden: bool) {',
            '  let nextHidden = if prevHidden then flip 0.3 else flip 0.65 in',
            '  let nextVisible = if nextHidden then flip 0.65 else flip 0.3 in',
            '  (nextHidden, nextVisible)', '}', 'let hidden0 = flip 0.5 in']
    for k in range(1, n + 1):
        dice += [f'let state{k} = nextState(hidden{k - 1}) in', f'let hidden{k} = fst state{k} in',
                 f'let visible{k} = snd state{k} in']
    if query != 'marginal-likelihood':
        dice += [f'let _ = observe(visible{k}) in' for k in range(1, n + 1)]
    name = {'v': 'visible', 'h': 'hidden'}[kind]
    dice.append(' && '.join(['true'] + [f'{name}{k}' for k in idx]))
    return pl, dice


def outcomes(query, n):
    """The variables a query's outcome conjoins: ('v' or 'h', indices)."""
    return {'marginal-likelihood': ('v', list(range(1, n + 1))),
            'filtering-marginal': ('h', [n]),
            'smoothing-marginal': ('h', [0]),
            'filtering-joint': ('h', list(range(n // 2, n + 1))),
            'smoothing-joint': ('h', list(range(0, n // 2)))}[query]


def network(k):
    pl = ['% Roulette network.rkt: message 1111 through diamonds of a noisy node (flip 0.01: one bit flipped,',
          '% chosen uniformly) and a forwarding one (flip 0.99: passed, else lost), split by flip 0.5.']
    pl.append('% The message as its four bits b(I,B) and l(I) for lost; t(I,B) flips bit B in diamond I.')
    for i in range(1, k + 1):           # noisy's select and flip, forward's flip, split's flip
        pl += [ad([(f'bit({i},{b})', 0.25) for b in range(4)]), f'0.01::nz({i}).', f'0.99::fw({i}).',
               f'0.5::sp({i}).']
    bits, lost = [True] * 4, False
    for i in range(1, k + 1):
        # A lost message's bits are never read, so a bit flips where the
        # noisy node flips it whether or not the message is lost.
        flips = [gate(pl, f't({i},{b})', [[f'sp({i})', f'nz({i})', f'bit({i},{b})']]) for b in range(4)]
        bits = [gate(pl, f'b({i},{b})', [[x, neg(t)], [neg(x), t]]) for b, (x, t) in enumerate(zip(bits, flips))]
        lost = gate(pl, f'l({i})', [[lost], [f'\\+sp({i})', f'\\+fw({i})']])
    for v in range(16):
        gate(pl, f'm({k},{v})', [[neg(lost)] + equals(bits, v)], keep=True)
    gate(pl, f'm({k},16)', [[lost]], keep=True)
    pl += [f'query(m({k},{v})).' for v in range(17)]
    # The message as a flag (lost) and its four bits; the value an
    # integer, 16 for a lost message.
    dice = ['fun diamond(m: (bool, (bool, (bool, (bool, bool))))) {',
            '  let lost = fst m in', '  let b0 = fst (snd m) in', '  let b1 = fst (snd (snd m)) in',
            '  let b2 = fst (snd (snd (snd m))) in', '  let b3 = snd (snd (snd (snd m))) in',
            '  let bit = discrete(0.25, 0.25, 0.25, 0.25) in',
            '  let nz = flip 0.01 in', '  let fw = flip 0.99 in', '  let sp = flip 0.5 in',
            '  let c = nz && !lost in',
            '  let n0 = b0 ^ (c && bit == int(2, 0)) in', '  let n1 = b1 ^ (c && bit == int(2, 1)) in',
            '  let n2 = b2 ^ (c && bit == int(2, 2)) in', '  let n3 = b3 ^ (c && bit == int(2, 3)) in',
            '  if sp then (lost, (n0, (n1, (n2, n3))))',
            '  else (if fw then m else (true, (false, (false, (false, false)))))', '}',
            'let m0 = (false, (true, (true, (true, true)))) in']
    dice += [f'let m{i} = diamond(m{i - 1}) in' for i in range(1, k + 1)]
    dice += [f'let lost = fst m{k} in', f'let b0 = fst (snd m{k}) in', f'let b1 = fst (snd (snd m{k})) in',
             f'let b2 = fst (snd (snd (snd m{k}))) in', f'let b3 = snd (snd (snd (snd m{k}))) in',
             'if lost then int(5, 16) else '
             '((if b0 then int(5, 1) else int(5, 0)) + (if b1 then int(5, 2) else int(5, 0)) + '
             '(if b2 then int(5, 4) else int(5, 0)) + (if b3 then int(5, 8) else int(5, 0)))']
    return pl, dice


def hardware_start(fuel, bw=HW_BW):
    return min(max(0, fuel - 2), 2 ** bw - 1)


def hardware(fuel, bw=HW_BW):
    mod, c0 = 2 ** bw, hardware_start(fuel, bw)
    pl = [f'% Roulette hardware.rkt, bit width {bw}, n = {max(0, fuel - 2)}, fuel {fuel}: t(J, K) and c(J, K),',
          '% bit K of the total and the counter entering the loop with J iterations done, on(J) while',
          '% it runs; z(J, R) resets a cell before the R-th read of iteration J: counter (test, d),',
          '% total (u), counter (sum, g), counter (step, f); x(J) leaves the loop at iteration J.']
    for j in range(fuel):
        pl += [f'{PERTURB}::z({j},{r}).' for r in range(1, 5)]
    total, counter, on, leave = [False] * bw, [bool(c0 >> k & 1) for k in range(bw)], True, []
    for j in range(fuel):
        def read(name, bits, r):
            return [gate(pl, f'{name}({j},{k})', [[b, f'\\+z({j},{r})']]) for k, b in enumerate(bits)]
        d = read('d', counter, 1)
        nonzero = gate(pl, f'nz({j})', [[b] for b in d])
        leave.append((gate(pl, f'x({j})', [[on, neg(nonzero)]]), total))
        on = gate(pl, f'on({j + 1})', [[on, nonzero]])
        u = read('u', total, 2)
        g = read('g', d, 3)
        f = read('f', g, 4)
        total = adder(pl, lambda kind, k: {'s': 't'}.get(kind, f't{kind}') + f'({j + 1},{k})', u, g)
        counter = adder(pl, lambda kind, k: {'s': 'c'}.get(kind, f'c{kind}') + f'({j + 1},{k})', f,
                        [k == 0 for k in range(bw)])
    # The total the loop left with.
    done = gate(pl, 'done', [[x] for x, _ in leave])
    result = [gate(pl, f'r({k})', [[x, bits[k]] for x, bits in leave]) for k in range(bw)]
    for t in range(mod):
        gate(pl, f'res({t})', [[done] + equals(result, t)], keep=True)
    gate(pl, f'res({mod})', [[on]], keep=True)
    pl += [f'query(res({t})).' for t in range(mod + 1)]
    # Unrolled: `on` while the loop runs, `done` once it has ended with
    # the total `r`; the value the total, or 2^bw without fuel.
    w = f'int({bw}, 0)'
    dice = [f'let t0 = {w} in', f'let c0 = int({bw}, {c0}) in', 'let on0 = true in', 'let done0 = false in',
            f'let r0 = {w} in']
    for j in range(fuel):
        i = j + 1
        dice += [f'let z{i}a = flip {PERTURB} in', f'let d{i} = if z{i}a then {w} else c{j} in',
                 f'let x{i} = on{j} && d{i} == {w} in',
                 f'let r{i} = if x{i} then t{j} else r{j} in', f'let done{i} = done{j} || x{i} in',
                 f'let on{i} = on{j} && !x{i} in',
                 f'let z{i}b = flip {PERTURB} in', f'let u{i} = if z{i}b then {w} else t{j} in',
                 f'let z{i}c = flip {PERTURB} in', f'let g{i} = if z{i}c then {w} else d{i} in',
                 f'let t{i} = u{i} + g{i} in',
                 f'let z{i}d = flip {PERTURB} in', f'let f{i} = if z{i}d then {w} else g{i} in',
                 f'let c{i} = f{i} + int({bw}, 1) in']
    out = f'int({bw + 1}, {mod})'
    for t in range(mod - 1, -1, -1):
        out = f'(if r{fuel} == int({bw}, {t}) then int({bw + 1}, {t}) else {out})'
    dice.append(f'if done{fuel} then {out} else int({bw + 1}, {mod})')
    return pl, dice


def program(family, size):
    if family.startswith('hmm-'):
        return hmm(family[4:], size)
    return {'motivating': motivating, 'diamond': diamond, 'ladder': ladder, 'caesar': caesar,
            'network': network, 'hardware': hardware}[family](size)


# --- reference answers -----------------------------------------------------

class Log:
    """A nonnegative number as its natural logarithm (-inf for 0), for
    probabilities below the doubles' range."""
    __slots__ = ('l',)

    def __init__(self, x=0.0, log=None):
        self.l = log if log is not None else (math.log(x) if x > 0 else -math.inf)

    def __add__(self, o):
        a, b = max(self.l, o.l), min(self.l, o.l)
        return Log(log=a if b == -math.inf else a + math.log1p(math.exp(b - a)))

    def __mul__(self, o):
        return Log(log=self.l + (o.l if isinstance(o, Log) else math.log(o) if o > 0 else -math.inf))

    def __truediv__(self, o):
        return Log(log=self.l - o.l)


def fmt(x):
    """A Log as text: a double where it is a normal one, else mantissa and
    decimal exponent."""
    if x.l == -math.inf:
        return '0.0'
    if x.l > math.log(2.2250738585072014e-308):
        return repr(math.exp(x.l))
    d = x.l / math.log(10)
    e = math.floor(d)
    return f'{10 ** (d - e):.15g}e{e}'


def chain(n, start, step):
    """P(x_n) of a Boolean Markov chain: start P(x_0), step(p) the next."""
    q = start
    for _ in range(n):
        q = step(q)
    return q


def reference(family, size):
    """{atom: Log}: the answers, from the model's states, step by step, and
    under 'P(e)' the evidence's probability where there is evidence."""
    n = size
    if family == 'motivating':
        return {f'x({n})': Log(chain(n, Fraction(1, 10), lambda q: q * Fraction(2, 10) + (1 - q) * Fraction(3, 10))
                               if n <= 200 else chain(n, 0.1, lambda q: 0.2 * q + 0.3 * (1 - q)))}
    if family == 'diamond':
        return {f's({n})': Log(log=n * math.log1p(-0.5 * 0.0001))}
    if family == 'ladder':
        # (s1, s2): (true, false), (false, true), (false, false)
        tf, ft, ff = 1.0, 0.0, 0.0
        for _ in range(n):
            alive = tf * (1 - 0.001) + ft
            tf, ft, ff = alive * 0.5, alive * 0.5, ff + tf * 0.001
        return {f'o1({n})': Log(tf), f'o2({n})': Log(ft)}
    if family == 'caesar':
        key, letter = bin_cat(KEY), bin_cat(LETTERS)
        post = []
        for k in range(26):
            lp = math.log(key[k])
            for o in observations(n):
                lp += math.log(FAIL * letter[o] + (1 - FAIL) * (letter[o - k] if o >= k else 0.0))
            post.append(lp)
        top = max(post)
        z = sum(math.exp(x - top) for x in post)
        out = {f'key({k})': Log(math.exp(x - top) / z) for k, x in enumerate(post)}
        out['P(e)'] = Log(log=top + math.log(z))
        return out
    if family.startswith('hmm-'):
        out = {'q': hmm_reference(family[4:], n)}
        if family != 'hmm-marginal-likelihood':
            out['P(e)'] = hmm_reference('marginal-likelihood', n)
        return out
    if family == 'network':
        dist = {15: 1.0}
        lost = 0.0
        for _ in range(n):
            new = {}
            for v, q in dist.items():
                new[v] = new.get(v, 0.0) + q * (0.5 * 0.99 + 0.5 * 0.99)
                for b in range(4):
                    w = v ^ (1 << b)
                    new[w] = new.get(w, 0.0) + q * 0.5 * 0.01 * 0.25
            lost = lost + (1 - lost) * 0.5 * 0.01
            dist = new
        out = {f'm({n},{v})': Log(dist.get(v, 0.0)) for v in range(16)}
        out[f'm({n},16)'] = Log(lost)
        return out
    if family == 'hardware':
        return hardware_reference(n)
    raise SystemExit(f'unknown family {family}')


def hmm_reference(query, n):
    """The query's probability by the forward algorithm over the hidden
    state, in logarithms: P(outcome and conditions) / P(conditions)."""
    trans = {True: 0.3, False: 0.65}     # P(h_k | h_{k-1})
    emit = {True: 0.65, False: 0.3}      # P(v_k | h_k)
    kind, idx = outcomes(query, n)
    need = set(idx) if kind == 'h' else set()

    def forward(require):
        a = {h: Log(0.5) for h in (True, False)}
        if 0 in require:
            a[False] = Log(0.0)
        for k in range(1, n + 1):
            b = {}
            for h in (True, False):
                s = Log(0.0)
                for g in (True, False):
                    s = s + a[g] * (trans[g] if h else 1 - trans[g])
                b[h] = s * emit[h] if not (k in require and not h) else Log(0.0)
            a = b
        return a[True] + a[False]
    if query == 'marginal-likelihood':
        return forward(set())
    return forward(need) / forward(set())


def hardware_reference(fuel, bw=HW_BW):
    mod = 2 ** bw
    states = {(0, hardware_start(fuel, bw)): 1.0}
    res = [0.0] * mod
    z = PERTURB
    for _ in range(fuel):
        new = {}
        for (t, c), q in states.items():
            for c1, q1 in ((0, z), (c, 1 - z)):
                if c1 == 0:
                    res[t] += q * q1
                    continue
                for t2, q2 in ((0, z), (t, 1 - z)):
                    for c2, q3 in ((0, z), (c1, 1 - z)):
                        s = (t2 + c2) % mod
                        for c3, q4 in ((0, z), (c2, 1 - z)):
                            key = (s, (c3 + 1) % mod)
                            new[key] = new.get(key, 0.0) + q * q1 * q2 * q3 * q4
        states = new
    out = {f'res({t})': Log(r) for t, r in enumerate(res)}
    out[f'res({mod})'] = Log(sum(states.values()))
    return out


HEADER = '% gen_roulette.py'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('family', nargs='?', choices=FAMILIES)
    ap.add_argument('size', nargs='?', type=int)
    ap.add_argument('--dice', help='also write the Dice program here')
    ap.add_argument('--reference', action='store_true', help='print the answers instead of the program')
    ap.add_argument('--reference-of', metavar='PROGRAM', help='print the answers of a program this script wrote')
    a = ap.parse_args()
    if a.reference_of:
        if a.family or a.size is not None or a.dice or a.reference:
            raise SystemExit('--reference-of reads the family and size from the program and takes nothing else')
        with open(a.reference_of) as f:
            head = f.readline().split()
        if head[:2] != HEADER.split() or len(head) != 4 or head[2] not in FAMILIES:
            raise SystemExit(f'{a.reference_of}: not a program of gen_roulette.py (its first line is not '
                             f'`{HEADER} FAMILY SIZE`)')
        a.family, a.size, a.reference = head[2], int(head[3]), True
    if a.family is None or a.size is None:
        raise SystemExit('give FAMILY and SIZE, or --reference-of PROGRAM')
    if a.size < 1:
        raise SystemExit('size: at least 1')
    if a.reference:
        if a.dice:
            raise SystemExit('--reference prints answers and writes no program; --dice has nothing to write')
        for atom, x in reference(a.family, a.size).items():
            print(f'{atom}: {fmt(x)}')
        return
    pl, dice = program(a.family, a.size)
    sys.stdout.write('\n'.join([f'{HEADER} {a.family} {a.size}'] + pl) + '\n')
    if a.dice:
        with open(a.dice, 'w') as f:
            f.write('\n'.join(dice) + '\n')


if __name__ == '__main__':
    main()
