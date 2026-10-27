"""scripts/alea_bench.jl: the circuit it writes for an Alea.jl program
answers as Alea does.

Julia runs only where TDDLOG_ALEA names it, as a command template with
{file} for the script (bench.py's --alea, e.g. "julia --project=ALEA
{file}"); otherwise the tests are skipped. Each test program of
tests/alea/ is answered by Alea (`run`), its circuit is written
(`export`), and the ProbLog program and the Dice programs are evaluated
here by enumerating the worlds of their flips; both must give Alea's
answers, and Alea's must be the hand-computed ones in the program's
comment. The circuit must be the one committed beside the program, which
tests/it/examples.rs answers with tddlog.
"""
import itertools
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / 'scripts'))
from source_tree import SOURCE  # noqa: E402
script = root / 'scripts' / 'alea_bench.jl'
programs = Path(__file__).resolve().parent / 'alea'
ALEA = os.environ.get('TDDLOG_ALEA', '')


def julia(*args):
    cmd = ALEA.format(file=script).split() + [str(a) for a in args]
    return subprocess.run(cmd, capture_output=True, text=True, check=True).stdout


def alea_answers(out):
    answers, values, expectations = {}, {}, {}
    for line in out.splitlines():
        f = line.split()
        if f[0] == 'alea_q':
            answers[f[1]] = float(f[2])
        elif f[0] == 'alea_value':
            values[f[1]] = float(f[2])
        elif f[0] == 'alea_expectation':
            expectations[int(f[1])] = float(f[2])
    return answers, values, expectations


def enumerate_worlds(facts, value_of):
    """The probability of each world of the flips, and the function
    `value_of` of it, summed by value."""
    names = list(facts)
    for bits in itertools.product([False, True], repeat=len(names)):
        world = dict(zip(names, bits))
        p = 1.0
        for n, b in world.items():
            p *= facts[n] if b else 1 - facts[n]
        yield p, value_of(world)


def problog_answers(text):
    """The answers of the ProbLog program alea_bench.jl writes: facts
    `P::fN.`, rules over literals, `evidence(A, V).`, `query(A).`."""
    facts, rules, evidence, queries = {}, {}, [], []
    for line in text.splitlines():
        if line.startswith('%') or not line.strip():
            continue
        if m := re.fullmatch(r'(\S+)::(\w+)\.', line):
            facts[m.group(2)] = float(m.group(1))
        elif m := re.fullmatch(r'evidence\((\w+), (true|false)\)\.', line):
            evidence.append((m.group(1), m.group(2) == 'true'))
        elif m := re.fullmatch(r'query\((.+)\)\.', line):
            queries.append(m.group(1))
        elif m := re.fullmatch(r'(.+?) :- (.+)\.', line):
            rules.setdefault(m.group(1), []).append(m.group(2).split(', '))
        elif m := re.fullmatch(r'(.+)\.', line):
            rules.setdefault(m.group(1), []).append([])
        else:
            raise AssertionError(f'unexpected line {line!r}')

    def holds(atom, world, memo):
        if atom in world:
            return world[atom]
        if atom not in memo:
            memo[atom] = any(all(not holds(l[2:], world, memo) if l.startswith('\\+') else
                                 False if l == 'fail' else holds(l, world, memo) for l in body)
                             for body in rules.get(atom, []))
        return memo[atom]

    def value_of(world):
        memo = {}
        if not all(holds(a, world, memo) == v for a, v in evidence):
            return None
        return tuple(holds(q, world, memo) for q in queries)

    pe, joint = 0.0, [0.0] * len(queries)
    for p, v in enumerate_worlds(facts, value_of):
        if v is not None:
            pe += p
            joint = [j + (p if t else 0.0) for j, t in zip(joint, v)]
    return {q: j / pe for q, j in zip(queries, joint)}


def dice_answers(directory, prefix):
    """The answers of the Dice programs alea_bench.jl writes, listed in
    PREFIX.dicemap, each a chain of `let`s over flips, `&&`, `||` and `!`,
    observations, and a returned literal or nested pair of literals."""
    answers = {}
    for line in Path(directory, prefix + '.dicemap').read_text().splitlines():
        file, kind, atom, *rest = line.split()
        facts, gates, observed, ret = {}, [], [], None
        for l in Path(directory, file).read_text().splitlines():
            if l.startswith('//'):
                continue
            if m := re.fullmatch(r'let (\w+) = flip (\S+) in', l):
                facts[m.group(1)] = float(m.group(2))
            elif m := re.fullmatch(r'let (\w+) = (!?\w+) (&&|\|\|) (!?\w+) in', l):
                gates.append(m.groups())
            elif m := re.fullmatch(r'let _ = observe (!?\w+) in', l):
                observed.append(m.group(1))
            else:
                ret = re.findall(r'!?\w+', l)

        def value_of(world):
            env = dict(world)
            lit = lambda s: (not env[s[1:]]) if s.startswith('!') else env[s]  # noqa: E731
            for name, x, op, y in gates:
                env[name] = (lit(x) and lit(y)) if op == '&&' else (lit(x) or lit(y))
            if not all(lit(o) for o in observed):
                return None
            return tuple(lit(r) for r in ret)

        dist = {}
        for p, v in enumerate_worlds(facts, value_of):
            if v is not None:
                dist[v] = dist.get(v, 0.0) + p
        pe = sum(dist.values())
        if kind == 'bool':
            answers[atom] = dist.get((True,), 0.0) / pe
        else:
            n = int(rest[0])
            for u in range(2 ** n):
                bits = tuple((u >> (n - 1 - i)) & 1 == 1 for i in range(n))
                answers[f'{atom}({u})'] = dist.get(bits, 0.0) / pe
    return answers


@unittest.skipUnless(ALEA, 'TDDLOG_ALEA names no Julia with Alea.jl')
class AleaBenchTests(unittest.TestCase):
    def check_circuit(self, name):
        with tempfile.TemporaryDirectory() as d:
            prefix = Path(d, name)
            answers, values, expectations = alea_answers(julia('run', programs / f'{name}.jl', prefix))
            text = prefix.with_suffix('.pl').read_text()
            for q, p in problog_answers(text).items():
                self.assertAlmostEqual(p, answers[q], places=12, msg=q)
            dice = dice_answers(d, name)
            self.assertEqual(set(dice), set(answers))
            for q, p in dice.items():
                self.assertAlmostEqual(p, answers[q], places=12, msg=q)
            # `export` writes the same circuit as `run` does, and the one
            # tests/it/examples.rs reads.
            julia('export', programs / f'{name}.jl', Path(d, 'again'))
            self.assertEqual(Path(d, 'again.pl').read_text(), text)
            # Timed, it writes that circuit too, and times it twice.
            out = julia('export', '--warm-below', 'Inf', programs / f'{name}.jl', Path(d, 'timed'))
            self.assertEqual(Path(d, 'timed.pl').read_text(), text)
            times = dict(line.split() for line in out.splitlines())
            self.assertEqual(set(times), {'alea_seconds', 'alea_compile_seconds', 'alea_warm_seconds'})
            self.assertLess(float(times['alea_warm_seconds']), float(times['alea_seconds']))
            # The public copy's first line also says where the script is.
            public = (SOURCE / 'tests' / 'alea' / f'{name}.pl').read_text().split('\n', 1)
            first, rest = text.split('\n', 1)
            self.assertEqual(public[0], first.rstrip('.') + ' (baselines branch).')
            self.assertEqual(public[1], rest)
        return answers, values, expectations

    def test_booleans_and_worlds_of_a_number(self):
        answers, values, _ = self.check_circuit('tiny_pr')
        self.assertAlmostEqual(answers['q1'], 0.12 / 0.72, places=12)
        by_value = {}
        for q, v in values.items():
            by_value[v] = by_value.get(v, 0.0) + answers[q]
        self.assertEqual(set(by_value), {1.0, 2.0, 3.0, 4.0})
        for v, p in by_value.items():
            self.assertAlmostEqual(p, (0.405 if v == 2 else 0.105) / 0.72, places=12, msg=v)
        # Worlds no value names have probability zero.
        for q, p in answers.items():
            if q.startswith('q2(') and q not in values:
                self.assertEqual(p, 0.0, q)

    def test_the_bits_of_an_expectation(self):
        answers, _, expectations = self.check_circuit('tiny_expectation')
        self.assertAlmostEqual(expectations[1], -0.6, places=12)
        self.assertTrue(all(re.fullmatch(r'e1\(\d\)', q) for q in answers))
        # The sign bit is true for -3, -2 and -1: three of the five values.
        self.assertAlmostEqual(answers['e1(1)'], 0.6, places=12)

    def test_set_runs_the_program_with_a_constant_replaced(self):
        with tempfile.TemporaryDirectory() as d:
            prefix = Path(d, 'set')
            answers, _, expectations = alea_answers(
                julia('run', '--set', 'lo=-1', programs / 'tiny_expectation.jl', prefix))
            self.assertAlmostEqual(expectations[1], 2 / 3, places=12)
            text = prefix.with_suffix('.pl').read_text()
            self.assertIn('% tiny_expectation.jl lo=-1: ', text)
            for q, p in problog_answers(text).items():
                self.assertAlmostEqual(p, answers[q], places=12, msg=q)
            # A constant the program does not define is an error.
            with self.assertRaises(subprocess.CalledProcessError) as e:
                julia('export', '--set', 'hi=1', programs / 'tiny_expectation.jl', prefix)
            self.assertIn('has 0 lines `const hi = ...`', e.exception.stderr)


if __name__ == '__main__':
    unittest.main()
