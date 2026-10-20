#!/usr/bin/env python3
"""Psi's own linear regression program (R2's, Nori, Hur, Rajamani and
Samuel, AAAI 2014, as PSI has it, Gehr, Misailovic and Vechev, CAV 2016,
test/r2/linearRegression.psi of github.com/eth-sri/psi) on the model of
data/alea/psi/linear_regression.jl, which has the program's data: a slope
a ~ N(0, 1), an intercept b ~ N(5, 1.8257) and N observations y_k ~ N(a x_k
+ b, 1) at x_k = -2 + 0.004 k, for the points k = round(1000 j / N),
j = 1 .. N. Psi's `gauss` takes a variance, and `cobserve` conditions on a
density.

    gen_psi_linreg.py a|b [N]

writes the program that returns a (or b); psi --expectation prints its
posterior mean. N is the number of observations, by default the program's
(10).
"""
import os
import re
import sys

JL = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'alea', 'psi', 'linear_regression.jl')


def data():
    with open(JL) as f:
        text = f.read()
    body = re.search(r'const Y1 = \[(.*?)\]', text, re.S).group(1)
    return [float(v) for v in body.replace('\n', ' ').split(',') if v.strip()]


def default_n():
    with open(JL) as f:
        return int(re.search(r'^const n = (\d+)', f.read(), re.M).group(1))


def program(n, ret):
    y = data()
    lines = ['def main() {', '  a := gauss(0, 1);', '  b := gauss(5, 1.8257);']
    for j in range(1, n + 1):
        k = round(1000 * j / n)
        lines.append(f'  cobserve(gauss(a * ({round(-2 + 0.004 * k, 3)}) + b, 1), {y[k - 1]});')
    return '\n'.join(lines + [f'  return {ret};', '}', ''])


if __name__ == '__main__':
    if len(sys.argv) not in (2, 3) or sys.argv[1] not in ('a', 'b') or (len(sys.argv) == 3 and not sys.argv[2].isdigit()):
        sys.exit(__doc__)
    sys.stdout.write(program(int(sys.argv[2]) if len(sys.argv) == 3 else default_n(), sys.argv[1]))
