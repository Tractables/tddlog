#!/usr/bin/env python3
"""DeepProbLog's handwritten formulas: the probability of a formula of N
symbols' label.

usage: gen_hwf.py N [--seed S] [--peak P]

The handwritten-formula task (HWF) of DeepProbLog (Manhaeve et al.,
Artificial Intelligence 2021; data/deepproblog/hwf_model.pl): a formula
of N images, digits and operators in turn, is classified symbol by
symbol, and parse/2 evaluates the symbols, products and quotients first,
to compare the value with the formula's label. Each image is an
annotated disjunction that stands in for its classifier's softmax, as in
gen_mnist_sum.py: mass P on a true symbol drawn from the seed, the rest
spread at random in units of 1e-4. The label is the true formula's value
(a formula that divides by zero is drawn again), and the question is the
task's training query, `expression(Images, Label)`: the probability that
the classified symbols make a formula of that value, within 1e-4.

The model calls parse/2 in two modes, which almost_equal/2 tells apart
with ground/1 and var/1: with the label bound, it compares; with the
result of a sum's right operand unbound, it computes. The program here
is the model with its parse written once for each mode, parse/2 with the
result bound and parse_free/2 with it unbound, almost_equal/2 keeping
for each mode the clause that holds there; the rest is the model's text.
A formula of N symbols has 10^((N+1)/2) 4^((N-1)/2) readings.
"""
import argparse
import os
import random
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
OPERATORS = '+-*/'


def replace(text, old, new, count):
    """`text` with `old` replaced by `new`, which occurs `count` times."""
    if text.count(old) != count:
        sys.exit(f'gen_hwf.py: {old!r} occurs {text.count(old)} times in the model, not {count}')
    return text.replace(old, new)


def value(symbols):
    """The formula's value as parse/2 computes it, None where it divides by
    zero: products and quotients from the left, then the sum from the
    right, a difference adding the next term times -1."""
    s = list(symbols)
    if len(s) == 1:
        return float(s[0])
    n1, o = s[0], s[1]
    if o == '*':
        return value([n1 * s[2]] + s[3:])
    if o == '/':
        return None if s[2] == 0 else value([n1 / s[2]] + s[3:])
    rest = value(s[2:] if o == '+' else [-1, '*'] + s[2:])
    return None if rest is None else float(n1 + rest)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('n', type=int)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--peak', type=float, default=0.7)
    a = ap.parse_args()
    if a.n < 1 or a.n % 2 == 0 or not 0 <= a.peak <= 1:
        ap.error('N must be odd and positive, the peak in [0, 1]')
    rng = random.Random(a.seed)

    def softmax(k, true):
        w = [rng.random() for _ in range(k)]
        units = [int((1 - a.peak) * 10000 * x / sum(w)) for x in w]
        units[true] += 10000 - sum(units)
        return units

    while True:
        truth = [rng.randrange(10) if i % 2 == 0 else rng.choice(OPERATORS) for i in range(a.n)]
        label = value(truth)
        if label is not None:
            break
    model = open(os.path.join(ROOT, 'data', 'deepproblog', 'hwf_model.pl')).read()
    model = replace(model, 'nn(net1,[X],Y,[0,1,2,3,4,5,6,7,8,9]) :: detect_number(X,Y).\n', '', 1)
    model = replace(model, 'nn(net2,[X],Y,[+,-,*,/]) :: detect_operator(X,Y).\n', '', 1)
    model = replace(model, 'almost_equal(X,Y) :- ground(Y),abs(X-Y) < 0.0001.',
                    'almost_equal(X,Y) :- abs(X-Y) < 0.0001.', 1)
    model = replace(model, 'almost_equal(X,Y) :- var(Y), Y is float(X).',
                    'almost_equal_free(X,Y) :- Y is float(X).', 1)
    head, sep, parse = model.partition('parse([N],R)')
    parse = sep + parse.rstrip() + '\n'
    bound = replace(replace(parse, 'parse(T,R2)', 'parse_free(T,R2)', 1),
                    'parse([-1,*|T],R2)', 'parse_free([-1,*|T],R2)', 1)
    free = replace(replace(parse, 'parse(', 'parse_free(', 9), 'almost_equal(', 'almost_equal_free(', 3)
    images = [f'i{i}' for i in range(1, a.n + 1)]
    shown = int(label) if label == int(label) else f'{label:.6f}'.rstrip('0')
    print(f'% DeepProbLog\'s handwritten formulas (data/deepproblog/hwf_model.pl), written by gen_hwf.py '
          f'{a.n} --seed {a.seed} --peak {a.peak}:')
    print(f'% the formula {" ".join(map(str, truth))} = {label!r}; each image an annotated disjunction in place')
    print('% of its classifier, and parse/2 written for each of the two modes the model tells apart')
    print('% with ground/1 and var/1 (parse_free/2 with the result unbound).')
    for img, t in zip(images, truth):
        if isinstance(t, int):
            print('; '.join(f'{u / 10000:.4f}::detect_number({img},{d})' for d, u in enumerate(softmax(10, t))) + '.')
        else:
            units = softmax(4, OPERATORS.index(t))
            print('; '.join(f'{u / 10000:.4f}::detect_operator({img},{o})' for o, u in zip(OPERATORS, units)) + '.')
    print(head.strip() + '\n')
    print(bound)
    print(free)
    print(f'query(expression([{",".join(images)}],{shown})).')


if __name__ == '__main__':
    main()
