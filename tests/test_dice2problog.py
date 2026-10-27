"""An exact evaluation of Dice programs, by enumeration.

`exact` evaluates a Dice program by enumeration: every expression denotes
a distribution over values, an observation drops the mass where it fails,
a loop's state is a distribution after every step, and probabilities are
exact fractions. It is checked against Dice's own answers, or answers
computed by hand on a program written for these tests, on the programs of
tests/dice/; tests/it/examples.rs checks tddlog's answers on those
programs, as its Dice reader reads them, against the same answers, and
test_gen_bn.py checks the Dice programs gen_bn.py writes with it. It reads
a program with scripts/dice2problog.py's parser.
"""
from fractions import Fraction
from pathlib import Path
import re
import sys
import unittest

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / 'scripts'))
from source_tree import SOURCE  # noqa: E402  (the source tree's tests/dice)
import dice2problog  # noqa: E402

translations = SOURCE / 'tests' / 'dice'

RECURSION_LIMIT = dice2problog.RECURSION_LIMIT


class IntValue(tuple):
    """An integer value and its modulus."""


def num(v, mod):
    return IntValue((v % mod, mod))


def product(*dists):
    out = [((), Fraction(1))]
    for d in dists:
        out = [(vs + (v,), p * q) for vs, p in out for v, q in d.items()]
    return out


def add(out, v, p):
    if p:
        out[v] = out.get(v, 0) + p


def exact(text):
    """The distribution of the program's value given its observations, as
    {value: probability}: a Boolean, an IntValue (value, modulus), or a
    pair of values."""
    funs, body = dice2problog.Parser(dice2problog.tokenize(text)).program()
    funs = {name: (params, ret, fbody) for name, params, ret, fbody in funs}
    active, calls = {}, {}

    def call(fname, args):
        """A call's distribution depends on its arguments and on how deep
        it recurses alone: a loop meets the same states again."""
        params, ret, fbody = funs[fname]
        depth = active.get(fname, 0)
        if depth > RECURSION_LIMIT:
            return {}
        key = (fname, depth, tuple(args))
        if key not in calls:
            active[fname] = depth + 1
            try:
                calls[key] = ev(fbody, dict(zip(params, args)))
            finally:
                active[fname] = depth
        return calls[key]

    def ev(e, env):
        kind = e[0]
        if kind == 'let':
            out = {}
            for v, p in ev(e[2], env).items():
                for w, q in ev(e[3], {**env, e[1]: v}).items():
                    add(out, w, p * q)
            return out
        if kind == 'var':
            return {env[e[1]]: Fraction(1)}
        if kind == 'bool':
            return {e[1]: Fraction(1)}
        if kind == 'flip':
            p = Fraction(e[1])
            out = {}
            add(out, True, p)
            add(out, False, 1 - p)
            return out
        if kind in ('discrete', 'uniform'):
            if kind == 'discrete':
                ps = [Fraction(p) for p in e[1]]
                mod = 2 ** dice2problog.bits_for(len(ps))
            else:
                width, lo, hi = e[1:]
                ps = [Fraction(0)] * lo + [Fraction(1, hi - lo)] * (hi - lo)
                mod = 2 ** width
            total = sum(ps)
            return {num(k, mod): p / total for k, p in enumerate(ps) if p}
        if kind == 'int':
            return {num(e[2], 2 ** e[1]): Fraction(1)}
        if kind == 'tuple':
            return {(a, b): p for (a, b), p in product(ev(e[1], env), ev(e[2], env))}
        if kind in ('fst', 'snd'):
            out = {}
            for v, p in ev(e[1], env).items():
                add(out, v[0] if kind == 'fst' else v[1], p)
            return out
        if kind == 'not':
            return {not v: p for v, p in ev(e[1], env).items()}
        if kind == 'shift':
            out = {}
            for v, p in ev(e[2], env).items():
                add(out, num(v[0] << e[3] if e[1] == '<<' else v[0] >> e[3], v[1]), p)
            return out
        if kind == 'bin':
            out = {}
            for (a, b), p in product(ev(e[2], env), ev(e[3], env)):
                add(out, apply(e[1], a, b), p)
            return out
        if kind == 'observe':
            return {True: sum(p for v, p in ev(e[1], env).items() if v)}
        if kind == 'if':
            out = {}
            for c, p in ev(e[1], env).items():
                for v, q in ev(e[2] if c else e[3], env).items():
                    add(out, v, p * q)
            return out
        if kind == 'call':
            out = {}
            for args, p in product(*(ev(x, env) for x in e[2])):
                for v, q in call(e[1], list(args)).items():
                    add(out, v, p * q)
            return out
        if kind == 'iterate':
            state = ev(e[2], env)
            for _ in range(e[3]):
                nxt = {}
                for s, p in state.items():
                    for v, q in call(e[1], [s]).items():
                        add(nxt, v, p * q)
                state = nxt
            return state
        raise ValueError(f'unknown expression {kind}')

    dist = ev(body, {})
    total = sum(dist.values())
    return {v: p / total for v, p in dist.items()}


def apply(op, a, b):
    if op in ('&&', '||', '^', '<=>') or isinstance(a, bool):
        return {'&&': a and b, '||': a or b, '^': a != b, '<=>': a == b, '==': a == b, '!=': a != b}[op]
    if op in dice2problog.COMPARE:
        return dice2problog.COMPARE[op](a[0], b[0])
    return num({'+': a[0] + b[0], '-': a[0] - b[0], '*': a[0] * b[0]}[op], a[1])


def leaves(v):
    return leaves(v[0]) + leaves(v[1]) if isinstance(v, tuple) and not isinstance(v, IntValue) else [v]


def answers(dist, queries):
    """The answers to a translation's queries from the distribution of the
    program's value: the components depth first, P(true) of a Boolean one,
    P(k) of each value k of an integer one (queries `v(0)`, `v(1)`, ...)."""
    out, family, leaf = [], None, -1
    for q in queries:
        m = re.fullmatch(r'([a-z][a-z0-9_]*)\((\d+)\)', q)
        if not (m and m.group(1) == family):
            leaf += 1
            family = m.group(1) if m else None
        out.append(float(sum(p for v, p in dist.items()
                             if (leaves(v)[leaf] is True if not m else leaves(v)[leaf][0] == int(m.group(2))))))
    assert leaf == len(leaves(next(iter(dist)))) - 1, 'one query group per component'
    return out


def queries(text):
    return re.findall(r'^query\((.*)\)\.$', text, re.M)


# program -> its translation's queries with Dice's answers (Dice 0ea228e,
# `dice FILE`, the marginals of its joint table, which prints 12 digits).
# The same answers are in tests/it/examples.rs. For pmc1 and pmc2 Dice's test
# suite (test/Test.ml, commented out there) gives 1023/2048 and 31/64,
# which tddlog gives too (and `exact` on pmc2); Dice prints all zeros on
# caesar-iter and simpleCaesar, whose evidence is below Dice 1.0's cutoff
# of 1e-6 and below the smallest double. A loop's state takes the values of
# a fixpoint over its step, which may hold more than the last step's: the
# counter's n2(7) has no row in Dice's table, probability 0.
DICE = {
    'arith': [('w(0)', 0.321428571429), ('w(1)', 0.353571428571), ('w(2)', 0.325), ('w(3)', 0.0)],
    'compare': [('result_1', 0.112676056338), ('result_2_1(0)', 0.131455399061), ('result_2_1(1)', 0.112676056338),
                ('result_2_1(2)', 0.107981220657), ('result_2_1(3)', 0.056338028169),
                ('result_2_1(4)', 0.112676056338), ('result_2_1(5)', 0.140845070422),
                ('result_2_1(6)', 0.18779342723), ('result_2_1(7)', 0.150234741784), ('result_2_2(0)', 0.0),
                ('result_2_2(1)', 0.281690140845), ('result_2_2(2)', 0.154929577465),
                ('result_2_2(3)', 0.281690140845), ('result_2_2(4)', 0.093896713615),
                ('result_2_2(5)', 0.18779342723)],
    'walk': [('x(0)', 0.0), ('x(1)', 0.245556733026), ('x(2)', 0.0), ('x(3)', 0.362357126637),
             ('x(4)', 0.0), ('x(5)', 0.190068881992), ('x(6)', 0.0), ('x(7)', 0.202017258345)],
    'geometric': [('result(0)', 0.500244259893), ('result(1)', 0.250122129946), ('result(2)', 0.125061064973),
                  ('result(3)', 0.0625305324866), ('result(4)', 0.0312652662433),
                  ('result(5)', 0.0156326331216), ('result(6)', 0.00781631656082),
                  ('result(7)', 0.00390815828041), ('result(8)', 0.00195407914021),
                  ('result(9)', 0.000977039570103), ('result(10)', 0.000488519785051)],
    'counter': [('n2(0)', 0.118949715135), ('n2(1)', 0.305870696061), ('n2(2)', 0.327718602923),
                ('n2(3)', 0.187267773099), ('n2(4)', 0.0601932127818), ('n2(5)', 0.0),
                ('n2(6)', 0.0), ('n2(7)', 0.0), ('result_2_2', 0.247460985881)],
    'recursion': [('result_else_1', 0.125)],
    'caesar-recur': [('key(0)', 0.0036496350365), ('key(1)', 0.0583941605839), ('key(2)', 0.934306569343),
                     ('key(3)', 0.0036496350365)],
    'diamond': [('result_2', 0.995012354812)],
    'ladder': [('result_1_1', 0.475370841777), ('result_2_1', 0.475370841777)],
}

# program -> its translation's queries with answers computed by hand,
# which ProbLog gives too. nested: A is uniform on three values, B = 0 with
# probability 0.9, 0.4 or 0.2 by A, so P(B = 1) = 1/2; C has (0.1, 0.6,
# 0.3) where B = 0 and A < 2 (mass 13/30), (0.5, 0.3, 0.2) where B = 0
# and A = 2 (1/15), a third each where B = 1: P(C = 0) = 13/300 + 1/30 +
# 1/6 = 73/300, P(C = 1) = 134/300, P(C = 2) = 93/300.
BY_HAND = {
    'nested': [('c(0)', 73 / 300), ('c(1)', 134 / 300), ('c(2)', 93 / 300)],
}


class DiceTests(unittest.TestCase):
    def test_exact_gives_dices_answers(self):
        for name, expected in DICE.items():
            with self.subTest(name=name):
                text = (translations / f'{name}.pl').read_text()
                self.assertEqual(queries(text), [q for q, _ in expected])
                got = answers(exact((translations / f'{name}.dice').read_text()), queries(text))
                for (q, x), g in zip(expected, got):
                    self.assertAlmostEqual(g, x, delta=1e-9, msg=q)

    def test_exact_gives_the_answers_by_hand(self):
        for name, expected in BY_HAND.items():
            with self.subTest(name=name):
                text = (translations / f'{name}.pl').read_text()
                self.assertEqual(queries(text), [q for q, _ in expected])
                got = answers(exact((translations / f'{name}.dice').read_text()), queries(text))
                for (q, x), g in zip(expected, got):
                    self.assertAlmostEqual(g, x, delta=1e-15, msg=q)


if __name__ == '__main__':
    unittest.main()
