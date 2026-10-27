#!/usr/bin/env python3
"""Exact P(taught_by(c1,p1)): the reference for gen_uwcse.py's programs,
where ProbLog, PITA, Ace and Merlin do not finish (uwcse2560).

usage: uwcse_exact.py program.pl [--precision N]

The program is cplint's uwcse3s with one course c1, one professor p1 and
N students (gen_uwcse.py N). The method shares nothing with tddlog's: it
is lifted. Given the course's level and the professor's position, the
students are independent and alike; they meet only in three disjunctions
over all of them, aux1_prof_nb_publications(p1) (some advisee with three
or more publications), aux1_taught_by(c1,p1) (some teaching assistant
advised by p1) and aux2_taught_by(c1,p1) (one who also shares a
publication with p1), and in prof_nb_publications(p1), which the first
decides. So the answer sums, over the course's level, the position, the
professor's publications and the values of the three disjunctions, whose
probabilities inclusion and exclusion give from powers N of one student's.
Every probability is the program's decimal as an exact fraction, so the
answer is exact up to its printing. The script stops on a program that is
not gen_uwcse.py's, or whose probabilities are not the ones below.
"""
import argparse
import os
import re
import sys
from fractions import Fraction
from itertools import product

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import source_tree  # noqa: E402,F401  (the source tree's converters)
import lpad2problog  # noqa: E402

# Each probability as the program writes it; main() checks that each is there.
ADVISED = {True: '0.10708782742681', False: '0.0278422273781903'}   # by p1, faculty or not
LEVEL = {'300': '0.0666666666666667', '400': '0.318518518518519', '500': '0.614814814814815'}
FACULTY = '0.732142857142857'
PROF = {1: {'none': '0.0416666666666667', 'one_to_nine': '0.416666666666667', 'ten': '0.541666666666667'},
        0: {'none': '0.5', 'one_to_nine': '0.294117647058824', 'ten': '0.205882352941176'}}  # by aux1_prof
NB = {'none': '0.594405594405594', 'one_or_two': '0.195804195804196', 'three': '0.20979020979021'}
YEAR6 = '0.32258064516129'              # year_6_or_more of a student with publications
SHARED = {'nopub': '0.00101010101010101', 'year6': '0.956521739130435',
          'prof_none': '0.125', 'prof_some': '0.631578947368421'}  # advised by p1
TA = {('300', True): '0.0333333333333333', ('300', False): '0.158227848101266',
      '400': '0.0337519623233909', '500': '0.0120999219359875'}   # (level, three or more)
TAUGHT = {'both': '0.777777777777778', 'aux1': '0.333333333333333', ('fac', True): '0.266666666666667',
          ('fac', False): '0.102941176470588', 'notfac': '0.029673590504451'}  # (fac, level 300)


def F(s):
    return Fraction(s)


def one_student(level, fac, q):
    """{(e, t1, t2): probability} for one student: e = advised by p1 with three
    or more publications, t1 = a teaching assistant of c1 advised by p1, t2 =
    t1 and a publication shared with p1; q is p1's publications."""
    padv = F(ADVISED[fac])
    out = {c: Fraction(0) for c in product((0, 1), repeat=3)}
    for nb, pn in NB.items():
        pn = F(pn)
        ta = F(TA[(level, nb == 'three')] if level == '300' else TA[level])
        if nb == 'none':
            sh = F(SHARED['nopub'])
        else:
            y6 = F(YEAR6)
            sh = y6 * F(SHARED['year6']) + (1 - y6) * F(SHARED['prof_none' if q == 'none' else 'prof_some'])
        e = int(nb == 'three')
        out[(0, 0, 0)] += pn * (1 - padv)
        out[(e, 1, 1)] += pn * padv * ta * sh
        out[(e, 1, 0)] += pn * padv * ta * (1 - sh)
        out[(e, 0, 0)] += pn * padv * (1 - ta)
    return out


def disjunctions(p, n):
    """{(a, x, y): probability} that the disjunctions over n students of e,
    t1 and t2 take those values, by inclusion and exclusion over which of
    them are forced false."""
    def none_of(zero):
        return sum(v for c, v in p.items() if all(c[i] == 0 for i in zero)) ** n
    out = {}
    for vals in product((0, 1), repeat=3):
        zeros = [i for i in range(3) if vals[i] == 0]
        ones = [i for i in range(3) if vals[i] == 1]
        out[vals] = sum((-1) ** k * none_of(zeros + [ones[j] for j in range(len(ones)) if mask >> j & 1])
                        for mask in range(1 << len(ones))
                        for k in [bin(mask).count('1')])
    return out


def answer(n):
    total = Fraction(0)
    for (level, pl), fac in product(LEVEL.items(), (True, False)):
        pf = F(FACULTY) if fac else 1 - F(FACULTY)
        for q in ('none', 'one_to_nine', 'ten'):
            for (a, x, y), pd in disjunctions(one_student(level, fac, q), n).items():
                if x:
                    t = TAUGHT['both' if y else 'aux1']
                else:
                    t = TAUGHT[('fac', level == '300')] if fac else TAUGHT['notfac']
                total += F(pl) * pf * F(PROF[a][q]) * pd * F(t)
    return total


def constants():
    for d in (ADVISED, LEVEL, PROF[0], PROF[1], NB, SHARED, TA, TAUGHT):
        yield from d.values()
    yield from (FACULTY, YEAR6)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    a = ap.parse_args()
    text = open(a.program).read()
    n = len(re.findall(r'^student\(s\d+\)\.$', text, re.M))
    src = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'cplint', 'uwcse3s.pl')
    model = [x for x in lpad2problog.convert(open(src).read()).splitlines() if not x.startswith('student(s')]
    read = [x for x in text.splitlines() if not x.startswith('%') and not x.startswith('student(s')]
    if n == 0 or read != model:
        sys.exit(f"uwcse_exact.py: {a.program} is not gen_uwcse.py's program")
    missing = [c for c in constants() if not re.search(rf'(?<![0-9.]){re.escape(c)}::', text)]
    if missing:
        sys.exit(f'uwcse_exact.py: the program has none of the probabilities {missing}')
    print(f'taught_by(c1,p1):\t{float(answer(n)):.{a.precision}f}')


if __name__ == '__main__':
    main()
