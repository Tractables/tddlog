#!/usr/bin/env python3
"""The scaling programs of Roulette's evaluation (Moy, Czenszak, Li,
Marshall and Holtzen, "Roulette: A Language for Expressive, Exact, and
Efficient Discrete Probabilistic Programming", PLDI 2025; the artifact's
benchmark/ directory, data/roulette/README.md) as ProbLog programs, as
Dice programs, and their answers.

Each family's ProbLog program is one first-order program, over the steps
of its loop, its size and parameters its arguments (the places $1, $2,
...) and its data (observations, the totals it asks about) files it
consults (`:- consult($1).`): bench.py's FOLD. `--template` prints the
program (data/roulette/FAMILY.pl; data/dice/FAMILY.pl for the Dice
paper's diamond and ladder; caesar's is written here, its categoricals'
probabilities computed), `FAMILY SIZE --arguments` an instance's
arguments after its data files, a line each, and `FAMILY SIZE --data
KEY` the data file KEY of `fold` (FAMILY SIZE). `FAMILY SIZE --dice PATH`
writes the Dice program, whose value answers the ProbLog program's
queries as bench.py reads Dice's table (a Boolean for one query, a tuple
of Booleans for one query each, an integer whose value k is the k-th
query). `--reference` prints the answers, one `ATOM: PROBABILITY` a
line, from a dynamic program over the model's states (`reference`),
never from either program, and for a program with evidence (caesar and
the hmm queries but the marginal likelihood) the evidence's probability,
`P(e): PROBABILITY`, as tddlog's `--evidence-probability` prints it;
`--reference-of PROGRAM` those of an instance's program, named by its
instance (bench.py's rl_FAMILY_SIZE.pl), or of a program whose first
line is `% gen_roulette.py FAMILY SIZE` (the unrolled programs this
script wrote before, which tddlog's tests keep). A probability below
the smallest normal double is printed with its decimal exponent
(`1.234e-400`) and stands for 0 in a double.

Families (the artifact's file; its sizes; the size here):
  motivating   modular/motivating.rkt, the Markov chain of the paper's
               Figure 2: x0 ~ flip 0.1, x_k = if x_{k-1} then flip 0.2
               else flip 0.3; P(x_N). N steps (artifact to 5000); the
               program's argument N.
  diamond      modular/diamond.rkt: N diamonds in a row, each passing its
               input along one of two routes chosen by flip 0.5, one of
               which drops it with probability 0.0001; P(through).
  ladder       modular/ladder.rkt: a ladder of N rungs over two wires;
               the joint of the two last wires, (true, false), (false,
               true) or neither, queried as the two marginals.
               Both are the Dice paper's networks (DICE_PAPER): their
               ProbLog program is data/dice/FAMILY.pl, its argument N, and
               their Dice program the Dice paper's own.
  caesar       modular/caesar.rkt: a key uniform over 26 letters, N
               letters drawn from English letter frequencies, each seen
               as letter + key (no wrap around: the benchmark adds 32-bit
               vectors) or, with probability 0.0001, as itself; the
               posterior of the key given the N observations of
               data/roulette/caesar_obs.txt (the benchmark's, from
               Racket's generator seeded with 0). N letters (to 300).
               Roulette draws each categorical by bisection (`bin-cat`):
               a flip with the left half's mass, then the half's values
               renormalized, so that a value's probability is the product
               of the flips' on the way to it, which differs from the table
               as written (the key's 26 x 0.038461538 sum to 1 - 1.2e-8) in
               the eighth digit. The ProbLog program's categoricals are
               annotated disjunctions with those probabilities, the letter
               and the key integers, added with `is`; its data the
               observations, evidence on seen(I, O), and its argument N.
  hmm-QUERY    hmm.rkt: a hidden chain h0 ~ flip 0.5, h_k ~ flip 0.3 after
               true and flip 0.65 after false, and observations v_k ~ flip
               0.65 when h_k and flip 0.3 otherwise; N steps. QUERY:
               marginal-likelihood P(v_1..v_N all true);
               filtering-marginal P(h_N | all v true); smoothing-marginal
               P(h_0 | all v true); filtering-joint P(h_{N/2}..h_N all
               true | all v true); smoothing-joint P(h_0..h_{N/2-1} all
               true | all v true) (N/2 rounded down). The artifact runs 1,
               100, 500, 1000 and 2000 steps. One ProbLog program for the
               five, its data the observations (none for the marginal
               likelihood), its arguments N and the question.
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
               for a lost message; the ProbLog program's argument is K.
  hardware     scaling/hardware.rkt: an interpreter of a loop over a
               memory of two cells of BW bits (total, counter) whose
               every read and write first resets the cell to 0 with
               probability 0.0001 (`mem-perturb`), running `tri n`: counter
               := n, then while counter != 0, total := total + counter and
               counter := counter + (int -1). The benchmark's (int -1) is
               the bit vector 1 (natural->bv of -1 sets bit 0 only), so
               the counter counts up to 2^BW and wraps to 0. The loop runs
               on FUEL: entered with none left, it ends the run with no
               result. The distribution of the total as Roulette prints
               it: res(T) for each total T a run can end with (the
               reference's states), and res(L), L one past the largest
               of them, for a run out of fuel (at BW = 2 every total of
               0 .. 3, and res(4)). The size is FUEL; BW = 2 and n = FUEL
               - 2 as in the artifact's fuel ladder (n saturates at 2^BW -
               1). A write's reset is overwritten at once and is left out.
               The cells are bit vectors, added as bvadd adds them.
  hardware-bitwidth
               the same with n = 2 and FUEL 3, as in the artifact's
               bit-width ladder (bit widths 5 to 20). The size is BW. The
               artifact's third ladder, n from 5 to 20 at BW = 2 and FUEL
               3, is left out: `(int n)` saturates at 2^BW - 1 = 3, so
               every n of it is the same program.
               The two are one ProbLog program, written once over the
               iterations and the bits, as Roulette's program takes BW as a
               parameter: a cell's bits are atoms with the bit as an
               argument, so that a reader grounds the bits a run can set
               and finds the constant high bits itself, and the program
               does not grow with BW. Its arguments are BW, the counter's
               start and FUEL; its data the totals it asks about.

The ProbLog programs hold Roulette's bit vectors as bits, an atom a bit,
and its integers as integers; the queries are what Roulette's query
computes, the distribution of the program's value. The Dice programs are
ports of Roulette's, which is the model, written at Dice's level: a loop
of the model is `iterate` over a function of one step, where the
artifact's Dice generators (gen-*.rkt, hmm.rkt's dice mode), written by
Roulette's authors as a baseline, unroll it. Caesar's observations are
data, a call of drawChar each, as caesar.dice.scrbl writes them: Dice has
no array to loop over that every target's translator reads. The Dice
paper's diamond and ladder keep the paper's own program for Dice and
noDice, a call a diamond or a rung; `--dice-port PATH` writes the loop
over the same function that the other systems' translators read.

Usage: gen_roulette.py FAMILY --template | FAMILY SIZE --arguments | FAMILY SIZE --data KEY
       gen_roulette.py FAMILY SIZE --dice PATH | diamond|ladder SIZE --dice-port PATH
       gen_roulette.py FAMILY SIZE --reference
       gen_roulette.py --reference-of PROGRAM.pl
"""
import argparse
import math
import os
import re
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
FAMILIES = ['motivating', 'diamond', 'ladder', 'caesar', 'network', 'hardware', 'hardware-bitwidth'] + \
    [f'hmm-{q}' for q in HMM_QUERIES]
HW_BW = 2
# The artifact's ladders of hardware.rkt: a size's (bit width, n, fuel).
HARDWARE = {'hardware': lambda fuel: (HW_BW, max(0, fuel - 2), fuel), 'hardware-bitwidth': lambda bw: (bw, 2, 3)}
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


def observations(n):
    with open(OBS) as f:
        obs = [int(line) for line in f]
    if n > len(obs):
        raise SystemExit(f'caesar: {OBS} holds {len(obs)} observations, not {n}')
    return obs[:n]


# --- ProbLog programs: a family's program, an instance's arguments and data --

TEMPLATES = os.path.join(HERE, '..', 'data', 'roulette')
PORTS = os.path.join(HERE, '..', 'data', 'dice')


def program_name(family):
    """The ProbLog program of a family: hmm's five queries are one program, and
    the two hardware ladders one."""
    return 'hmm' if family.startswith('hmm-') else 'hardware' if family in HARDWARE else family


def disjunction(heads, ps, body=None):
    """The annotated disjunction of `heads` with the probabilities `ps`, a head
    a line, with `body`; the last probability less what their sum passes 1 by
    in doubles, as the sum of Roulette's bisection may (bin_cat), so that every
    reader takes it."""
    ps = [float(q) for q in ps]
    for _ in range(4):
        over = sum(ps) - 1
        if over <= 0:
            break
        ps[-1] -= over
    lines = [f'{p(q)}::{h};' for h, q in zip(heads, ps)]
    lines[-1] = lines[-1][:-1] + (f' :- {body}.' if body else '.')
    return lines


CAESAR = """\
% The cipher of Roulette's caesar.rkt, ported to ProbLog: a key uniform
% over 26 letters (key) and $2 letters drawn from English letter
% frequencies (letter(I, L), the I-th letter L), each seen as letter + key
% (no wrap around: the benchmark adds 32-bit vectors) or, with probability
% 0.0001 (fail), as itself. The observations, consulted from $1, are
% evidence on seen(I, O) for I = 1 .. $2 (data/roulette/caesar_obs.txt, the
% benchmark's); the queries are the posterior of the key. caesar.rkt draws
% each categorical by bisection (`bin-cat`), and the probabilities here are
% its values' (gen_roulette.py's bin_cat).
:- consult($1).
letters($2).
index(1) :- letters(N), N > 0.
index(I1) :- index(I), letters(N), I1 is I + 1, I1 =< N.
"""


def template(family):
    """The text of a family's ProbLog program."""
    if family in DICE_PAPER:
        path = os.path.join(PORTS, family + '.pl')
    elif family != 'caesar':
        path = os.path.join(TEMPLATES, program_name(family) + '.pl')
    else:
        lines = disjunction([f'key({v})' for v in range(26)], bin_cat(KEY))
        lines += disjunction([f'letter(I, {v})' for v in range(26)], bin_cat(LETTERS), 'index(I)')
        lines += [f'{FAIL}::fail(I) :- index(I).',
                  'seen(I, L) :- fail(I), letter(I, L).',
                  'seen(I, O) :- \\+fail(I), key(K), letter(I, L), O is L + K.']
        lines += [f'query(key({v})).' for v in range(26)]
        return CAESAR + '\n'.join(lines) + '\n'
    with open(path) as f:
        return f.read()


def fold(family, size):
    """bench.py's FOLD entry of an instance: (its program's name, the
    arguments that print the program with --template, the keys of its data
    files, which come first among its places)."""
    if family == 'caesar':
        keys = [f'caesar_{size}']
    elif family == 'hmm-marginal-likelihood':
        keys = ['hmm_none']
    elif family.startswith('hmm-'):
        keys = [f'hmm_{size}']
    elif family in HARDWARE:
        keys = [f'{family.replace("-", "_")}_{size}']
    else:
        keys = []
    return program_name(family), [family], keys


def arguments(family, size):
    """An instance's arguments after its data files: the size, and hmm's
    question, the hardware ladders' bit width, counter and fuel."""
    if family.startswith('hmm-'):
        return [size, family[4:].replace('-', '_')]
    if family in HARDWARE:
        bw, n, fuel = HARDWARE[family](size)
        return [bw, hardware_start(bw, n), fuel]
    return [size]


def data(family, size, key):
    """The text of an instance's data file `key` (fold's)."""
    if key not in fold(family, size)[2]:
        raise SystemExit(f'{family} {size}: its data files are {fold(family, size)[2]}, not {key}')
    if family == 'caesar':
        lines = ['% The observations of caesar.rkt (data/roulette/caesar_obs.txt): letter I seen as O.']
        lines += [f'evidence(seen({i},{o}),true).' for i, o in enumerate(observations(size), 1)]
    elif key == 'hmm_none':
        lines = ['% No observations: the marginal likelihood asks for them.']
    elif family.startswith('hmm-'):
        lines = [f'% The observations of hmm.rkt: v(T) true for T = 1 .. {size}.']
        lines += [f'evidence(v({t}),true).' for t in range(1, size + 1)]
    else:
        totals, label = hardware_queries(*HARDWARE[family](size))
        lines = ['% The totals a run of hardware.rkt can end with (the reference\'s states), the label',
                 '% of a run out of fuel, one past the largest, and the queries.']
        lines += [f'total({t}).' for t in totals] + [f'out_of_fuel({label}).']
        lines += [f'query(res({t})).' for t in totals + [label]]
    return '\n'.join(lines) + '\n'


# --- Dice programs ---------------------------------------------------------

# The Dice programs are written at Dice's own level: a loop
# of the model is `iterate` over a function of one step, as the Dice paper's
# caesar-iter does, where the artifact's generators unroll it. A loop that
# runs no step is its start.

def loop(f, start, k):
    """`iterate(f, start, k)`, or `start` for no step."""
    return f'iterate({f}, {start}, {k})' if k else start


def motivating(n):
    # motivating.rkt: x0 ~ flip 0.1, x_k = if x_{k-1} then flip 0.2 else flip 0.3
    return ['fun step(x: bool) {', '  if x then flip 0.2 else flip 0.3', '}',
            'let x0 = flip 0.1 in', loop('step', 'x0', n)]


# The Dice paper's diamond and ladder networks (Holtzen, Van den Broeck and
# Millstein, OOPSLA 2020; SHoltzen/dice bench/dicebench.ml, gen_diamond and
# gen_ladder), the model of Roulette's diamond.rkt and ladder.rkt: N
# diamonds or rungs. Their Dice program is the text gen_diamond N and
# gen_ladder N build, byte for byte (noDice's diamond-L-p and ladder-L-p are
# it at N = L + 1); their ProbLog program is the port data/dice/FAMILY.pl.
DICE_PAPER = {
    'diamond': ('\nfun diamond(s1: bool) {\n//      if flip(0.5) then s1 else (flip(0.0001) && s1)\n'
                '      let route = flip 0.5 in\n      let s2 = if route then s1 else false in\n'
                '      let s3 = if route then false else s1 in\n      let drop = flip 0.0001 in\n'
                '      s2 || (s3 && !drop)\n}\nlet s = true in', '\nlet s = diamond(s) in', '\ns'),
    'ladder': ('\nfun ladder(s1: bool, s2: bool) {\n      if s1 then\n        if flip 0.001 then\n'
               '          (false, false)\n        else\n           let f = flip 0.5 in (f, !f)\n'
               '      else if s2 then\n           let f = flip 0.5 in (f, !f)\n      else (false, false)\n}\n'
               '      let x = (true, false) in\n', '\nlet x = ladder(fst x, snd x) in', '\nx'),
}


def dice_paper(family, n):
    """The Dice program of the Dice paper's generator for `family` at n."""
    head, call, tail = DICE_PAPER[family]
    return head + call * n + tail


# The port of the Dice paper's networks that the other systems' translators
# read (`--dice-port`): Dice and noDice read the paper's own program, a call
# a diamond or a rung, and the others its loop, `iterate` over the same
# function (the ladder's of one pair), so that each target writes its loop.
DICE_PORT = {
    'diamond': ['fun diamond(s1: bool) {', '  let route = flip 0.5 in', '  let s2 = if route then s1 else false in',
                '  let s3 = if route then false else s1 in', '  let drop = flip 0.0001 in', '  s2 || (s3 && !drop)',
                '}'],
    'ladder': ['fun ladder(x: (bool, bool)) {', '  let s1 = fst x in', '  let s2 = snd x in',
               '  if s1 then', '    if flip 0.001 then', '      (false, false)', '    else',
               '      let f = flip 0.5 in (f, !f)', '  else if s2 then', '    let f = flip 0.5 in (f, !f)',
               '  else (false, false)', '}'],
}


def dice_port(family, n):
    """The loop of the Dice paper's network `family` at n (DICE_PORT)."""
    start = {'diamond': 'true', 'ladder': '(true, false)'}[family]
    return '\n'.join(DICE_PORT[family] + [loop(family, start, n)]) + '\n'


def caesar(n):
    key, letter = bin_cat(KEY), bin_cat(LETTERS)
    obs = observations(n)
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
    return dice


def hmm(query, n):
    # hmm.rkt's model, a step of the chain a function: `step` observes the
    # step's visible variable, `stepAnd` also conjoins the hidden one into
    # the pair's second, and `stepSeen` (the marginal likelihood's)
    # conjoins the visible one without observing it
    def body(name, arg, h, value, observe=True):
        return [f'fun {name}({arg}) {{', f'  let nextHidden = if {h} then flip 0.3 else flip 0.65 in',
                '  let nextVisible = if nextHidden then flip 0.65 else flip 0.3 in'] + \
               (['  let _ = observe nextVisible in'] if observe else []) + [f'  {value}', '}']
    pair = 's: (bool, bool)'
    if query == 'marginal-likelihood':            # P(v_1 .. v_n all true)
        return body('stepSeen', pair, 'fst s', '(nextHidden, (snd s) && nextVisible)', False) + \
            ['let hidden0 = flip 0.5 in', f'let s = {loop("stepSeen", "(hidden0, true)", n)} in', 'snd s']
    dice = body('step', 'h: bool', 'h', 'nextHidden') + \
        (body('stepAnd', pair, 'fst s', '(nextHidden, (snd s) && nextHidden)') if query.endswith('-joint') else []) + \
        ['let hidden0 = flip 0.5 in']
    m = n // 2
    if query == 'filtering-marginal':             # h_n
        return dice + [loop('step', 'hidden0', n)]
    if query == 'smoothing-marginal':             # h_0
        return dice + [f'let h = {loop("step", "hidden0", n)} in', 'hidden0']
    if query == 'filtering-joint':                # h_m && .. && h_n
        return dice + [f'let hm = {loop("step", "hidden0", m)} in',
                       f'let s = {loop("stepAnd", "(hm, hm)", n - m)} in', 'snd s']
    if m == 0:                                    # smoothing-joint: h_0 && .. && h_{m-1}, none
        return dice + [f'let h = {loop("step", "hidden0", n)} in', 'true']
    return dice + [f'let s = {loop("stepAnd", "(hidden0, hidden0)", m - 1)} in',
                   f'let h = {loop("step", "fst s", n - m + 1)} in', 'snd s']


def outcomes(query, n):
    """The variables a query's outcome conjoins: ('v' or 'h', indices)."""
    return {'marginal-likelihood': ('v', list(range(1, n + 1))),
            'filtering-marginal': ('h', [n]),
            'smoothing-marginal': ('h', [0]),
            'filtering-joint': ('h', list(range(n // 2, n + 1))),
            'smoothing-joint': ('h', list(range(0, n // 2)))}[query]


def network(k):
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
            f'let m = {loop("diamond", "(false, (true, (true, (true, true))))", k)} in']
    dice += ['let lost = fst m in', 'let b0 = fst (snd m) in', 'let b1 = fst (snd (snd m)) in',
             'let b2 = fst (snd (snd (snd m))) in', 'let b3 = snd (snd (snd (snd m))) in',
             'if lost then int(5, 16) else '
             '((if b0 then int(5, 1) else int(5, 0)) + (if b1 then int(5, 2) else int(5, 0)) + '
             '(if b2 then int(5, 4) else int(5, 0)) + (if b3 then int(5, 8) else int(5, 0)))']
    return dice


def hardware_start(bw, n):
    """`(int n)`: n, or 2^bw - 1 past it."""
    return n if n.bit_length() <= bw else 2 ** bw - 1


def hardware_dice(bw, n, fuel):
    """The Dice program of hardware.rkt at bit width bw, a step of the
    loop (one unit of fuel) a function of the machine's state: the total
    `t`, the counter `c`, `running` while the loop runs, `finished` once
    it has ended with the total `r`. The value is the index of the total's
    query, or that of res(out_of_fuel), the last, without fuel."""
    c0 = hardware_start(bw, n)
    totals, _ = hardware_queries(bw, n, fuel)
    w, i = f'int({bw}, 0)', f'int({bw})'
    dice = [f'fun step(s: ({i}, ({i}, (bool, (bool, {i}))))) {{',
            '  let t = fst s in', '  let c = fst (snd s) in', '  let running = fst (snd (snd s)) in',
            '  let finished = fst (snd (snd (snd s))) in', '  let r = snd (snd (snd (snd s))) in',
            f'  let d = if flip {PERTURB} then {w} else c in',      # the counter read
            f'  let x = running && d == {w} in',                     # the loop's test fails: the run ends
            '  let r2 = if x then t else r in',
            f'  let u = if flip {PERTURB} then {w} else t in',      # total := total + counter
            f'  let g = if flip {PERTURB} then {w} else d in',
            f'  let f = if flip {PERTURB} then {w} else g in',      # counter := counter + (int -1)
            f'  (u + g, (f + int({bw}, 1), (running && !x, (finished || x, r2))))', '}',
            f'let s = {loop("step", f"({w}, (int({bw}, {c0}), (true, (false, {w}))))", fuel)} in',
            'let finished = fst (snd (snd (snd s))) in', 'let r = snd (snd (snd (snd s))) in']
    width = len(totals).bit_length()
    out = f'int({width}, {len(totals)})'
    for k in range(len(totals) - 1, -1, -1):
        out = f'(if r == int({bw}, {totals[k]}) then int({width}, {k}) else {out})'
    dice.append(f'if finished then {out} else int({width}, {len(totals)})')
    return dice


def dice(family, size):
    """The text of an instance's Dice program."""
    if family in DICE_PAPER:
        return dice_paper(family, size)
    if family.startswith('hmm-'):
        lines = hmm(family[4:], size)
    elif family in HARDWARE:
        lines = hardware_dice(*HARDWARE[family](size))
    else:
        lines = {'motivating': motivating, 'caesar': caesar, 'network': network}[family](size)
    return '\n'.join(lines) + '\n'


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
    if family in HARDWARE:
        return hardware_reference(*HARDWARE[family](n))
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


def hardware_step(t, c, mod):
    """One iteration entered with total t and counter c, for each outcome
    of its reads' resets: (P, the total the run ends with or None, the next
    state (total, counter) or None)."""
    z = PERTURB
    for c1, q1 in ((0, z), (c, 1 - z)):
        if c1 == 0:
            yield q1, t, None
            continue
        for t2, q2 in ((0, z), (t, 1 - z)):
            for c2, q3 in ((0, z), (c1, 1 - z)):
                for c3, q4 in ((0, z), (c2, 1 - z)):
                    yield q1 * q2 * q3 * q4, None, ((t2 + c2) % mod, (c3 + 1) % mod)


def hardware_totals(bw, n, fuel):
    """The totals a run can end with, smallest first, from the sets of
    states entering each iteration, which repeat after a few iterations."""
    states, seen, totals = {(0, hardware_start(bw, n))}, set(), set()
    for _ in range(fuel):
        if frozenset(states) in seen:
            break
        seen.add(frozenset(states))
        new = set()
        for t, c in states:
            for _, end, nxt in hardware_step(t, c, 2 ** bw):
                if nxt is None:
                    totals.add(end)
                else:
                    new.add(nxt)
        states = new
    return sorted(totals)


def hardware_queries(bw, n, fuel):
    """The totals a run can end with and the label of a run out of fuel,
    one past the largest of them: the values Roulette prints."""
    totals = hardware_totals(bw, n, fuel)
    return totals, (totals[-1] + 1 if totals else 0)


def hardware_reference(bw, n, fuel):
    """The distribution of the total, over the states (total, counter)
    entering each iteration."""
    states, res = {(0, hardware_start(bw, n)): 1.0}, {}
    for _ in range(fuel):
        new = {}
        for (t, c), q in states.items():
            for p, end, nxt in hardware_step(t, c, 2 ** bw):
                if nxt is None:
                    res[end] = res.get(end, 0.0) + q * p
                else:
                    new[nxt] = new.get(nxt, 0.0) + q * p
        states = new
    totals, label = hardware_queries(bw, n, fuel)
    out = {f'res({t})': Log(res.get(t, 0.0)) for t in totals}
    out[f'res({label})'] = Log(sum(states.values()))
    return out


HEADER = '% gen_roulette.py'


def program_of(path):
    """(family, size) of an instance's program: from its name, bench.py's
    rl_FAMILY_SIZE.pl (the family's dashes underscores), or from its first
    line, `% gen_roulette.py FAMILY SIZE`."""
    named = re.fullmatch(r'rl_(.+)_(\d+)\.pl', os.path.basename(path))
    by_name = {f.replace('-', '_'): f for f in FAMILIES}
    if named and named.group(1) in by_name:
        return by_name[named.group(1)], int(named.group(2))
    with open(path) as f:
        head = f.readline().split()
    if head[:2] != HEADER.split() or len(head) != 4 or head[2] not in FAMILIES:
        raise SystemExit(f'{path}: not an instance of gen_roulette.py (not named rl_FAMILY_SIZE.pl, and its '
                         f'first line is not `{HEADER} FAMILY SIZE`)')
    return head[2], int(head[3])


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('family', nargs='?', choices=FAMILIES)
    ap.add_argument('size', nargs='?', type=int)
    ap.add_argument('--template', action='store_true', help="print the family's ProbLog program")
    ap.add_argument('--arguments', action='store_true', help="print an instance's arguments after its data files")
    ap.add_argument('--data', metavar='KEY', help="print an instance's data file KEY")
    ap.add_argument('--dice', metavar='PATH', help="write an instance's Dice program here")
    ap.add_argument('--dice-port', metavar='PATH', help="write the loop of the Dice paper's diamond or ladder, "
                    "which the systems other than Dice and noDice translate, here")
    ap.add_argument('--reference', action='store_true', help="print an instance's answers")
    ap.add_argument('--reference-of', metavar='PROGRAM', help="print the answers of an instance's program")
    a = ap.parse_args()
    modes = [m for m in ('template', 'arguments', 'data', 'dice', 'dice_port', 'reference', 'reference_of')
             if getattr(a, m)]
    if len(modes) != 1:
        raise SystemExit('give one of --template, --arguments, --data KEY, --dice PATH, --dice-port PATH, '
                         '--reference and --reference-of PROGRAM')
    if a.dice_port and a.family not in DICE_PORT:
        raise SystemExit(f'--dice-port: {a.family} has no port of its own (its Dice program is the loop); '
                         f'the families with one are {", ".join(DICE_PORT)}')
    if a.reference_of:
        if a.family or a.size is not None:
            raise SystemExit('--reference-of reads the family and size from the program and takes nothing else')
        a.family, a.size = program_of(a.reference_of)
        a.reference = True
    if a.template:
        if a.family is None or a.size is not None:
            raise SystemExit('--template takes a family and no size')
        sys.stdout.write(template(a.family))
        return
    if a.family is None or a.size is None:
        raise SystemExit('give FAMILY and SIZE, or --reference-of PROGRAM')
    if a.size < 1:
        raise SystemExit('size: at least 1')
    if a.reference:
        for atom, x in reference(a.family, a.size).items():
            print(f'{atom}: {fmt(x)}')
    elif a.arguments:
        sys.stdout.write(''.join(f'{w}\n' for w in arguments(a.family, a.size)))
    elif a.data:
        sys.stdout.write(data(a.family, a.size, a.data))
    elif a.dice_port:
        with open(a.dice_port, 'w') as f:
            f.write(dice_port(a.family, a.size))
    else:
        with open(a.dice, 'w') as f:
            f.write(dice(a.family, a.size))


if __name__ == '__main__':
    main()
