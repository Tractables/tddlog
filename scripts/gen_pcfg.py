#!/usr/bin/env python3
"""A probabilistic context-free grammar of the ProbLog tutorial, parsing a word of length N.

usage: gen_pcfg.py ENCODING --template
       gen_pcfg.py ENCODING N [--seed S] --arguments

The ProbLog tutorial's page "Probabilistic Context Free Grammars"
(dtai.cs.kuleuven.be/problog/tutorial/various/06_pcfg.html) writes the
grammar

    S -> 0.4 :: O M | 0.6 :: S S      O -> a
    M -> 0.7 :: b | 0.3 :: S C        C -> b

which derives the nonempty balanced words of a and b (a opens, b closes),
in a way that gives each choice of a rule an identifier of its own, as
a PCFG's repeated choices are independent: the page's second program, in
which the state of a leftmost derivation, the terminals derived (reversed)
and the symbols still to expand, identifies the choice made in it (the
only encoding, `stack`).

The program is the page's, as written, but for its queries: the page asks
the probability of every word of a few lengths (`query(word([_,_]))`).
This asks, for one word W of length N, drawn uniformly with seed S from
the balanced words of three or more top-level blocks (a ... b, balanced
inside), how its derivation starts, given W: `evidence(word(W))` and the
probability that the first S and the first S it expands to both expand
to S S (`state([],[s,s,s])`, the stack's second step). The
word's blocks fix the first rule (S -> S S, as there are two or more),
not how S S brackets them: the k blocks' bracketings are equally likely,
so the answer is 1 - C(k-2)/C(k-1) in Catalan numbers, 1/2 for three
blocks and 3/5 for four. A word's probability is too small to compare to
a tolerance at the lengths that take time; the posterior is not. The
page has no licence (copyright KU Leuven), so the repository does not
ship it: it is read from data/problog-tutorial, which data/fetch.txt
names.

--template prints the encoding's program, whose word is its place $1
(tddlog reads it with the word as its argument: `tddlog pcfg.pl
[a,b,a,b,a,b]`); --arguments prints the argument of length N and seed S,
the word. The draw needs no page.
"""
import argparse
import html
import os
import random
import re

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
PAGE = os.path.join(ROOT, 'data', 'problog-tutorial', '06_pcfg.html')
# The page's program of the encoding (its second), and the atom that holds
# when the first S and the first S it expands to both expand to S S.
BLOCK = {'stack': 2}
SECOND = {'stack': 'state([],[s,s,s])'}
# The fewest top-level blocks of the word: with fewer, the posterior is 0.
BLOCKS = 3


def programs(page):
    """The text of the page's ProbLog editors, in order."""
    return [html.unescape(m) for m in re.findall(r'<div class="problog-editor"[^>]*>(.*?)</div>', page, re.S)]


def balanced(pairs, rng):
    """A word of `pairs` a's and as many b's whose every prefix has no more
    b's than a's, uniformly: by the cycle lemma, the one rotation of a
    random arrangement of `pairs` a's and `pairs + 1` b's whose proper
    prefixes all have no more b's than a's, without its last b."""
    word = ['a'] * pairs + ['b'] * (pairs + 1)
    rng.shuffle(word)
    height, low, at = 0, 0, 0
    for i, c in enumerate(word):
        height += 1 if c == 'a' else -1
        if height < low:
            low, at = height, i + 1
    return (word[at:] + word[:at])[:-1]


def blocks(word):
    """The number of top-level blocks of a balanced word: its returns to
    height 0."""
    height, k = 0, 0
    for c in word:
        height += 1 if c == 'a' else -1
        k += height == 0
    return k


def template(encoding):
    """The encoding's program of the page, its word the place $1."""
    if not os.path.exists(PAGE):
        raise SystemExit(f'gen_pcfg.py reads {PAGE}: run `bench.py fetch`')
    program = programs(open(PAGE, encoding='utf-8').read())[BLOCK[encoding] - 1]
    lines = program.rstrip().splitlines()
    queries = [i for i, line in enumerate(lines) if line.startswith('query(')]
    assert queries and all(line.startswith('query(word(') for line in (lines[i] for i in queries)), queries
    # The page's queries, and the comment above them, give way to this one.
    first = queries[0] - 1 if lines[queries[0] - 1].startswith('%') else queries[0]
    body = lines[:first] + lines[queries[-1] + 1:]
    return (f'% The ProbLog tutorial, "Probabilistic Context Free Grammars", program {BLOCK[encoding]}\n'
            '% (https://dtai.cs.kuleuven.be/problog/tutorial/various/06_pcfg.html), written by gen_pcfg.py\n'
            f"% {encoding} --template: its queries replaced by the first two rules of the word $1, the program's "
            'argument.\n'
            + '\n'.join(body).rstrip() + f'\n\nevidence(word($1)).\nquery({SECOND[encoding]}).\n')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('encoding', choices=sorted(BLOCK))
    ap.add_argument('n', type=int, nargs='?')
    ap.add_argument('--seed', type=int)
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help="the encoding's program, its word the place $1")
    mode.add_argument('--arguments', action='store_true', help='the word of length N and seed S')
    a = ap.parse_args()
    if a.template:
        if a.n is not None or a.seed is not None:
            ap.error('--template prints the program of every word: N and S give its argument')
        print(template(a.encoding), end='')
        return
    if a.n is None or a.n < 2 * BLOCKS or a.n % 2:
        ap.error(f'--arguments needs N, the length of a balanced word of {BLOCKS} blocks: even, {2 * BLOCKS} or more')
    rng = random.Random(1 if a.seed is None else a.seed)
    word = balanced(a.n // 2, rng)
    while blocks(word) < BLOCKS:
        word = balanced(a.n // 2, rng)
    print(f'[{",".join(word)}]')


if __name__ == '__main__':
    main()
