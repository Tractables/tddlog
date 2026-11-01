#!/usr/bin/env python3
"""cplint's hidden Markov model for part-of-speech tagging, emitting the
example's sentence K times.

usage: gen_hmmpos.py --template
       gen_hmmpos.py K --arguments

cplint's example hmmpos.pl (data/cplint/hmmpos.pl; Torbjorn Lager's
model, adapted by Fabrizio Riguzzi): from the start state the chain moves
from tag to tag (det, aux, v, n, pron), each tag emitting a word it
allows (a for det, can for aux, v and n, he for pron), and each choice of
the next tag is indexed by the list of tags before it, so that every
sequence of tags is a choice of its own, as in PRISM. The program is the
example's, converted to ProbLog by lpad2problog.py. The example writes the
history of each choice of a tag as an anonymous variable, `trans(det, n,
_):0.77`, once per head; in ProbLog the heads' anonymous variables are
distinct and the choice has no index of its own, so each is named S, as
cplint's hmm.pl names its choices', and library(lists), which gives
ProbLog reverse/2, is loaded, as SWI-Prolog loads it by itself. The
example samples the tags of the sentence "he can can a can"; the question
here is the probability of that sentence said K times (`hmm(O)`), whose
3K words "can" each have three tags, 27^K sequences of tags.

--template prints the program, whose sentence is its place $1 (tddlog
reads it with the sentence as its argument: `tddlog hmmpos.pl
[he,can,can,a,can]`); --arguments prints the argument of K, the sentence
said K times.
"""
import argparse
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
import lpad2problog  # noqa: E402

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
SENTENCE = ['he', 'can', 'can', 'a', 'can']


def template():
    program = lpad2problog.convert(open(os.path.join(ROOT, 'data', 'cplint', 'hmmpos.pl')).read())
    kept = [line for line in program.rstrip().splitlines() if not line.startswith('query(')]
    named = 0
    for i, line in enumerate(kept):
        if '::trans(' in line:
            kept[i], n = re.subn(r'::trans\((\w+),(\w+),_\)', r'::trans(\1,\2,S)', line)
            named += n
    if named != 30:
        sys.exit(f'gen_hmmpos.py: {named} choices of a tag named, not the example\'s 30')
    return ('% cplint example hmmpos, converted by lpad2problog.py, written by gen_hmmpos.py --template:\n'
            "% the probability of the sentence $1, the program's argument (the example's sentence said\n"
            '% some times over); the history of each choice of a tag, an anonymous variable in each head\n'
            '% of the example, is named S; reverse/2, which SWI-Prolog loads by itself, comes from\n'
            "% ProbLog's library(lists).\n"
            ':- use_module(library(lists)).\n'
            + '\n'.join(kept).rstrip() + '\n\nquery(hmm($1)).\n')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('k', type=int, nargs='?')
    mode = ap.add_mutually_exclusive_group(required=True)
    mode.add_argument('--template', action='store_true', help='the program, its sentence the place $1')
    mode.add_argument('--arguments', action='store_true', help="the example's sentence said K times")
    a = ap.parse_args()
    if a.template:
        if a.k is not None:
            ap.error('--template prints the program of every sentence: K gives its argument')
        print(template(), end='')
        return
    if a.k is None or a.k < 1:
        ap.error('--arguments needs K, the number of times the sentence is said: 1 or more')
    print(f'[{",".join(SENTENCE * a.k)}]')


if __name__ == '__main__':
    main()
