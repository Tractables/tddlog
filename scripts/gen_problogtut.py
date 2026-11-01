#!/usr/bin/env python3
"""A program of the ProbLog tutorial, as the page writes it.

usage: gen_problogtut.py PAGE K

PAGE is a page of the tutorial (dtai.cs.kuleuven.be/problog/tutorial/,
e.g. basic/02_bayes), read from data/problog-tutorial/ (its last path
component, 02_bayes.html), which data/fetch.txt names: the pages state no
licence (copyright KU Leuven), so the repository does not ship them. K
counts the page's ProbLog editors from 1, and the program is the text of
the K-th as the page holds it (html.unescape), its queries and evidence
included. Every program of bench.py's problogtut family asks marginal
probabilities (the editor's task `prob`); the pages' learning and MPE
editors are not in it.
"""
import argparse
import html
import os
import re

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
EDITOR = re.compile(r'<div class="problog-editor"([^>]*)>(.*?)</div>', re.S)


def editors(page):
    """(task, program) of the page's ProbLog editors, in order."""
    return [(re.search(r'data-task="(\w+)"', attrs).group(1) if 'data-task' in attrs else 'prob', html.unescape(text))
            for attrs, text in EDITOR.findall(page)]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('page')
    ap.add_argument('k', type=int)
    a = ap.parse_args()
    path = os.path.join(ROOT, 'data', 'problog-tutorial', a.page.rsplit('/', 1)[-1] + '.html')
    found = editors(open(path, encoding='utf-8').read())
    if not 1 <= a.k <= len(found):
        raise SystemExit(f'{a.page}: {len(found)} editors, no editor {a.k}')
    task, text = found[a.k - 1]
    if task != 'prob':
        raise SystemExit(f'{a.page} editor {a.k}: task {task}, not marginal probabilities')
    print(f'% ProbLog tutorial, {a.page}, program {a.k} '
          f'(https://dtai.cs.kuleuven.be/problog/tutorial/{a.page}.html)')
    print(text.strip('\n'))


if __name__ == '__main__':
    main()
