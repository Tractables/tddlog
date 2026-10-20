"""Dice programs and their ProbLog translations (scripts/dice2problog.py).

`exact` evaluates a Dice program by enumeration: every expression denotes
a distribution over values, an observation drops the mass where it fails,
a loop's state is a distribution after every step, and probabilities are
exact fractions. It is checked against Dice's own answers, or answers
computed by hand on a program written for these tests, and the committed
translations in tests/dice/ against what the translator writes now;
tests/it/examples.rs checks tddlog's answers on those translations
against the same answers.
"""
from fractions import Fraction
from pathlib import Path
import math
import re
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
data = root / 'data' / 'dice'
translations = root / 'tests' / 'dice'
sys.path.insert(0, str(scripts))
import dice2problog  # noqa: E402

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
# of 1e-6 and below the smallest double.
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
                ('n2(6)', 0.0), ('result_2_6', 0.247460985881)],
    'recursion': [('result_else_1', 0.125)],
    'caesar-recur': [('key(0)', 0.0036496350365), ('key(1)', 0.0583941605839), ('key(2)', 0.934306569343),
                     ('key(3)', 0.0036496350365)],
    'diamond': [('result_199', 0.995012354812)],
    'ladder': [('result_1_98', 0.475370841777), ('result_2_98', 0.475370841777)],
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
                got = answers(exact((data / f'{name}.dice').read_text()), queries(text))
                for (q, x), g in zip(expected, got):
                    self.assertAlmostEqual(g, x, delta=1e-9, msg=q)

    def test_exact_gives_the_answers_by_hand(self):
        for name, expected in BY_HAND.items():
            with self.subTest(name=name):
                text = (translations / f'{name}.pl').read_text()
                self.assertEqual(queries(text), [q for q, _ in expected])
                got = answers(exact((data / f'{name}.dice').read_text()), queries(text))
                for (q, x), g in zip(expected, got):
                    self.assertAlmostEqual(g, x, delta=1e-15, msg=q)

    def test_translations_are_current(self):
        names = [*DICE, *BY_HAND]
        self.assertEqual(sorted(p.stem for p in translations.glob('*.pl')), sorted(names))
        for name in names:
            with self.subTest(name=name):
                out = subprocess.run([sys.executable, str(scripts / 'dice2problog.py'), str(data / f'{name}.dice')],
                                     capture_output=True, text=True, check=True).stdout
                self.assertEqual(out, (translations / f'{name}.pl').read_text())

    def test_the_baseline_programs_translate(self):
        for path in sorted(data.glob('*.dice')):
            if path.stem in ('pmc1', 'digitRecognition'):
                continue
            with self.subTest(name=path.stem):
                text = dice2problog.translate(path.read_text())
                self.assertIn('query(', text)

    def test_unsupported_constructs_fail(self):
        for program, error in [('let x = [flip 0.5] in x', NotImplementedError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError),
                               ('sample flip 0.5', NotImplementedError),
                               ('int(2, 1) + int(3, 1)', TypeError),
                               ('int(2, 4)', ValueError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    dice2problog.translate(program)

    def test_a_discrete_off_one_by_rounding_is_normalized(self):
        # Dice accepts a list within 1e-4 of one and normalizes it (each bit
        # a ratio of sums); noDice's networks print thirds as 0.333333.
        for ps in [(0.333333, 0.333333, 0.333333), (0.5, 0.50005)]:
            with self.subTest(ps=ps):
                got = dice2problog.discrete_probs(ps)
                self.assertAlmostEqual(math.fsum(got), 1.0, delta=1e-15)
                self.assertAlmostEqual(got[0] / got[-1], ps[0] / ps[-1], delta=1e-15)
        self.assertEqual(dice2problog.discrete_probs((0.25, 0.75)), [0.25, 0.75])
        self.assertEqual(dice2problog.discrete_probs((0.25, 0.7)), [0.25, 0.7])

    def test_a_disjunctive_guard_restricts_its_branch(self):
        # Under `x == 1 || x == 3`, x takes the values 1 and 3 alone, whose
        # bit 0 is set: in that branch bit 0 is the constant true, so the
        # result's bit 0, true in the other branch too, is constant, and
        # the values with bit 0 clear have no rule.
        program = ('let x = discrete(0.1, 0.2, 0.3, 0.4) in\n'
                   'if x == int(2, 1) || x == int(2, 3) then x else int(2, 1)')
        text = dice2problog.translate(program)
        qs = queries(text)
        heads = {line.split(' :- ')[0].rstrip('.') for line in text.splitlines() if not line.startswith('query(')}
        self.assertEqual([q in heads for q in qs], [False, True, False, True], text)
        self.assertEqual(answers(exact(program), qs), [0.0, 0.6, 0.0, 0.4])

    def test_an_integer_is_its_bits(self):
        # A discrete is drawn bit by bit from the most significant, as Dice
        # draws it: bit 1 with the mass of 2 and 3; bit 0 with that of 1
        # among 0 and 1, and of 3 among 2 and 3. A uniform over a power of
        # two takes one flip a bit. No atom stands for one value but the
        # queries.
        facts = lambda text: sorted(float(line.split('::')[0]) for line in text.splitlines() if '::' in line)
        text = dice2problog.translate('let x = discrete(0.1, 0.2, 0.3, 0.4) in x')
        p = [Fraction(x) for x in (0.1, 0.2, 0.3, 0.4)]   # the masses as written, exactly
        self.assertEqual(facts(text), sorted(float(r) for r in [p[2] + p[3], p[1] / (p[0] + p[1]),
                                                                p[3] / (p[2] + p[3])]))
        text = dice2problog.translate('let x = uniform(3, 0, 8) in let y = uniform(3, 0, 8) in x + y == int(3, 5)')
        self.assertEqual(facts(text), [0.5] * 6)
        self.assertFalse(re.search(r'^[a-z_0-9]+\(\d+\)', text, re.M), text)

    def test_precedence_is_dices(self):
        # `!` binds looser than == and <, tighter than && (lib/Parser.mly);
        # == shares the level of + and -.
        tree = dice2problog.Parser(dice2problog.tokenize('!a == b && c < d + e')).expr()
        self.assertEqual(tree, ('bin', '&&', ('not', ('bin', '==', ('var', 'a'), ('var', 'b'))),
                                ('bin', '<', ('var', 'c'), ('bin', '+', ('var', 'd'), ('var', 'e')))))


if __name__ == '__main__':
    unittest.main()
