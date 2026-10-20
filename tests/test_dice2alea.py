"""Dice programs as Alea.jl scripts (scripts/dice2alea.py) and how bench.py
reads what the scripts print.

Julia does not run here: the scripts are checked by reading them. Every
variable a generated function reads is defined before it on every path
(a variable an `if` block defines is read after the block only by the
line that merges the branches, under the condition that ran it), every
call passes as many arguments as the function takes, and the flips come
in the program's order. The answers' side runs on the exact distribution
of test_dice2problog.py, written as a script would print it, against
Dice's answers.
"""
from pathlib import Path
import re
import subprocess
import sys
import unittest

root = Path(__file__).resolve().parents[1]
scripts = root / 'scripts'
data = root / 'data' / 'dice'
sys.path.insert(0, str(scripts))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench  # noqa: E402
import dice2alea  # noqa: E402
import dice2problog  # noqa: E402
import gen_bn  # noqa: E402
import test_dice2problog as reference  # noqa: E402

VAR = dice2alea.VAR
SIGNATURE = re.compile(r'function (\w+)\((.*)\)')
CALL = re.compile(r'\b(chunk_\d+|d_\w+)\(')
MERGE = re.compile(r'(x\d+_\w*) = (\S+) === true \? (\S+) : \2 === false \? (\S+) : ifelse\(\2, \3, \4\)')


def body(script):
    """The generated functions, between the prelude and the runner."""
    head = dice2alea.PRELUDE.format(source='').split('\n', 1)[1]
    start, end = script.index(head) + len(head), script.rindex(dice2alea.RUNNER)
    return script[start:end]


def arguments(text):
    """The top-level arguments of a call's parenthesised list."""
    depth, out, cur = 0, [], ''
    for ch in text:
        if ch in '([{':
            depth += 1
        elif ch in ')]}':
            depth -= 1
            if depth < 0:
                break
        if ch == ',' and depth == 0:
            out.append(cur.strip())
            cur = ''
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def functions(code):
    """{name: (params, lines)} of the top-level functions of `code`."""
    out, current = {}, None
    for line in code.splitlines():
        if current is None:
            m = SIGNATURE.fullmatch(line)
            if m:
                current = m.group(1)
                out[current] = ([a.strip() for a in m.group(2).split(',') if a.strip()], [])
            continue
        if line == 'end':
            current = None
        else:
            out[current][1].append(line)
    return out


def check(script):
    """Problems of a generated script, as messages: a read before a
    definition, a call with the wrong number of arguments or of an unknown
    function."""
    funs = functions(body(script))
    problems = []
    for name, (params, lines) in funs.items():
        # `conditional`: what the `if` blocks closed so far defined, which
        # only a merge line reads, under the condition that ran the block.
        defined, conditional, scopes = set(params), set(), []
        for line in lines:
            text = line.strip()
            if text.startswith(('if ', 'for ')) or text == 'try':
                problems += [f'{name}: {v} read before defined: {text}'
                             for v in VAR.findall(text) if v not in defined]
                scopes.append((text.split()[0], set(defined)))
                continue
            if text == 'finally':
                continue
            if text == 'end':
                kind, before = scopes.pop()
                if kind == 'if':
                    conditional |= defined - before
                    defined = before
                continue
            for m in CALL.finditer(text):
                callee = m.group(1)
                if callee not in funs:
                    problems.append(f'{name}: calls unknown {callee}')
                elif len(arguments(text[m.end():])) != len(funs[callee][0]):
                    problems.append(f'{name}: {callee} takes {len(funs[callee][0])} arguments: {text}')
            merge = MERGE.fullmatch(text)
            if merge:
                lhs, strict, branches = merge.group(1), [merge.group(2)], list(merge.group(3, 4))
            elif ' = ' in text and not text.startswith(('return ', 'observe!(')):
                lhs, _, rhs = text.partition(' = ')
                strict, branches = VAR.findall(rhs), []
            else:
                lhs, strict, branches = '', VAR.findall(text), []
            problems += [f'{name}: {v} read before defined: {text}' for v in strict
                         if VAR.fullmatch(v) and v not in defined]
            problems += [f'{name}: branch value {v} defined nowhere before: {text}' for v in branches
                         if VAR.fullmatch(v) and v not in defined | conditional]
            defined |= set(VAR.findall(lhs))
        if scopes:
            problems.append(f'{name}: {len(scopes)} blocks left open')
    return problems


def translate(text):
    return dice2alea.translate(text, 'test')


def flips(script):
    return [float(p) for p in re.findall(r'\bflip\(([0-9.e-]+)\)', body(script))]


def components(dist):
    """The lines a script prints for the exact distribution of a program's
    value, read back by bench.parse_alea."""
    leaves = [reference.leaves(v) for v in dist]
    out = []
    for k in range(len(leaves[0])):
        first = leaves[0][k]
        if first is True or first is False:
            p = sum(q for v, q in zip(leaves, dist.values()) if v[k] is True)
            out += [f'alea {k + 1} true {float(p)!r}', f'alea {k + 1} false {float(1 - p)!r}']
        else:
            for value in range(first[1]):
                p = sum(q for v, q in zip(leaves, dist.values()) if v[k][0] == value)
                out.append(f'alea {k + 1} {value} {float(p)!r}')
    return '\n'.join(out + ['alea_seconds 0.25', 'alea_compile_seconds 0.01']) + '\n'


class TranslationTests(unittest.TestCase):
    def test_the_baseline_programs_translate(self):
        for path in sorted(data.glob('*.dice')):
            with self.subTest(name=path.stem):
                script = dice2alea.translate(path.read_text(), path.name)
                self.assertTrue(script.startswith(f'# {path.name}, translated'))
                self.assertIn('\nusing Alea\n', script)
                self.assertTrue(script.endswith(dice2alea.RUNNER))
                self.assertEqual(check(script), [])
                self.assertIn('dice_main', functions(body(script)))

    def test_the_program_is_interpreted_and_alea_compiled(self):
        # A module of its own, whose functions Julia interprets: the
        # options apply to that module's code, not to Alea's.
        script = translate('flip 0.5')
        lines = script.splitlines()
        self.assertEqual(lines[1:3], ['module AleaProgram', 'using Alea'])
        self.assertIn('Base.Experimental.@compiler_options compile=min optimize=0 infer=false', lines[:8])
        self.assertEqual(lines[-1], 'end')
        self.assertEqual(sum(l.startswith('module ') for l in lines), 1)

    def test_the_script_runs_from_the_command_line(self):
        path = data / 'twocoins.dice'
        out = subprocess.run([sys.executable, str(scripts / 'dice2alea.py'), str(path)],
                             capture_output=True, text=True, check=True).stdout
        self.assertEqual(out, dice2alea.translate(path.read_text(), path.name))

    def test_long_bodies_are_split(self):
        script = dice2alea.translate((data / 'digitRecognition.dice').read_text())
        funs = functions(body(script))
        chunks = [n for n in funs if n.startswith('chunk_')]
        self.assertGreater(len(chunks), 10)
        # A merge line and its blocks stay together, so a function may run
        # over CHUNK lines by one statement's blocks, not by a multiple.
        self.assertLess(max(len(lines) for _, lines in funs.values()), 2 * dice2alea.CHUNK)
        self.assertEqual(check(script), [])
        observations = body(script).count('observe!(')
        self.assertEqual(observations, (data / 'digitRecognition.dice').read_text().count('observe'))

    def test_flips_follow_the_program(self):
        # Alea orders its BDD's variables by the creation of the flips: the
        # order of the text, as Dice, whatever reads them first.
        script = translate('let a = flip 0.1 in let b = flip 0.2 in\n'
                           'let c = if a then flip 0.3 else flip 0.4 in\n'
                           'if b then c else (flip 0.5 && a)')
        self.assertEqual(flips(script), [0.1, 0.2, 0.3, 0.4, 0.5])

    def test_constant_conditions_choose_their_branch(self):
        script = translate('if true then flip 0.1 else flip 0.2')
        self.assertEqual(flips(script), [0.1])
        script = translate('let x = flip 0.3 in if !true then flip 0.1 else x')
        self.assertEqual(flips(script), [0.3])

    def test_an_observation_in_a_branch_holds_on_its_path(self):
        script = translate('let x = flip 0.5 in\n'
                           'let y = if x then let _ = observe flip 0.3 in true else false in x')
        main = functions(body(script))['dice_main'][1]
        cond = re.search(r'(x\d+_x) = flip\(0\.5\)', '\n'.join(main)).group(1)
        self.assertTrue(any(re.fullmatch(rf'observe!\({cond}, x\d+_\w*\)', l.strip()) for l in main), main)
        # The other branch observes nothing and keeps no path.
        self.assertEqual(sum('observe!' in l for l in main), 1)

    def test_a_nested_observation_conjoins_the_paths(self):
        script = translate('let x = flip 0.5 in let y = flip 0.4 in\n'
                           'let z = if x then (if !y then let _ = observe flip 0.3 in true else true) else true in x')
        self.assertRegex(body(script), r'(x\d+_z) = !\(x\d+_y\)\n.*\n +x\d+_path = \(x\d+_x & \1\)')
        self.assertEqual(check(script), [])

    def test_functions_take_the_path_when_they_observe(self):
        script = translate('fun f(a: bool) { let _ = observe a in a }\n'
                           'fun g(a: bool) { !a }\n'
                           'fun h(a: bool) { f(a) }\n'
                           'let x = flip 0.5 in if x then h(flip 0.2) else g(x)')
        funs = functions(body(script))
        self.assertEqual(len(funs['d_f'][0]), 2)
        self.assertEqual(check(script), [])
        main = '\n'.join(funs['dice_main'][1])
        self.assertRegex(main, r'd_h\(x\d+_x, x\d+_\w*\)')   # under the branch's path, x
        self.assertRegex(main, r'd_g\(true, x\d+_x\)')       # g observes nothing

    def test_recursion_is_cut_at_dices_limit(self):
        script = dice2alea.translate((data / 'recursion.dice').read_text())
        self.assertIn(f'> {dice2problog.RECURSION_LIMIT}', script)
        self.assertRegex(script, r'const DEPTH_d_\w+ = Ref\(0\)')
        self.assertEqual(check(script), [])

    def test_iterate_is_a_loop(self):
        script = translate('fun step(x: int(3)) { x + int(3, 1) }\niterate(step, int(3, 2), 5)')
        main = '\n'.join(functions(body(script))['dice_main'][1])
        self.assertRegex(main, r'for _ in 1:5\n +(x\d+_\w*) = d_step\(true, \1\)')

    def test_integers_wrap_and_shift_as_dices(self):
        script = translate('let x = discrete(0.25, 0.25, 0.5) in ((x + int(2, 3)) >> 1, x << 1)')
        self.assertIn('discrete(DistUInt{2}, [0.25, 0.25, 0.5])', script)
        self.assertRegex(script, r'shr\(\(x\d+_x \+ DistUInt\{2\}\(3\)\), 1\)')
        self.assertNotIn('@alea', script)

    def test_unsupported_constructs_fail(self):
        for program, error in [('let x = [flip 0.5] in x', NotImplementedError),
                               ('int(3, 4) / int(3, 2)', NotImplementedError),
                               ('int(3, 4) % int(3, 2)', NotImplementedError),
                               ('sample flip 0.5', NotImplementedError),
                               ('discrete(0.5, 0.3)', ValueError),
                               ('int(2, 4)', ValueError)]:
            with self.subTest(program=program):
                with self.assertRaises(error):
                    translate(program)

    def test_rounded_distributions_are_normalised_as_for_problog(self):
        # digitRecognition's rows sum to 1.000004.
        script = translate('discrete(0.500002, 0.500002)')
        self.assertIn('discrete(DistUInt{1}, [0.5, 0.5])', script)


# A network written for this test, with variables of three and four values
# and two leaves, as the networks of the benchmarks have.
TRAFFIC = '''
node weather { states = ( "sun" "rain" "snow" ); }
node traffic { states = ( "light" "heavy" ); }
node delay { states = ( "none" "short" "long" "cancelled" ); }
node mood { states = ( "good" "bad" ); }
node call { states = ( "none" "text" "phone" ); }
potential ( weather ) { data = ( 0.6 0.3 0.1 ); }
potential ( traffic | weather ) { data = (( 0.7 0.3 )( 0.4 0.6 )( 0.2 0.8 )); }
potential ( delay | traffic weather ) {
  data = ((( 0.8 0.15 0.04 0.01 )( 0.6 0.3 0.08 0.02 )( 0.4 0.3 0.2 0.1 ))
          (( 0.5 0.3 0.15 0.05 )( 0.3 0.4 0.2 0.1 )( 0.1 0.3 0.3 0.3 )));
}
potential ( mood | delay ) { data = (( 0.9 0.1 )( 0.7 0.3 )( 0.4 0.6 )( 0.1 0.9 )); }
potential ( call | delay ) { data = (( 0.95 0.04 0.01 )( 0.6 0.3 0.1 )( 0.3 0.4 0.3 )( 0.1 0.3 0.6 )); }
'''


class NetworkTests(unittest.TestCase):
    def test_a_network_returns_its_queried_variables(self):
        for net in ['cancer', 'asia', 'traffic']:
            with self.subTest(net=net):
                network = gen_bn.parse_net(TRAFFIC) if net == 'traffic' else \
                    gen_bn.parse(str(root / 'examples' / f'{net}.net'))
                text = gen_bn.to_dice(network, 'leaves')
                program = gen_bn.to_problog(network, 'leaves')
                predicates = list(dict.fromkeys(q.split('(')[0] for q in reference.queries(program)))
                result = text.strip().splitlines()[-1]
                self.assertEqual(re.findall(r'\w+', result), [p.upper() for p in predicates])
                script = dice2alea.translate(text)
                self.assertEqual(check(script), [])

    def test_one_queried_variable_is_returned_alone(self):
        network = gen_bn.parse(str(root / 'examples' / 'cancer.net'))
        self.assertEqual(gen_bn.to_dice(network, 'Cancer').strip().splitlines()[-1], 'CANCER')


class AnswerTests(unittest.TestCase):
    def test_answers_line_up_with_the_queries(self):
        # Every component of the value in turn: a Boolean answers one query,
        # an integer the queries of one predicate, value k the k-th.
        for name, expected in reference.DICE.items():
            with self.subTest(name=name):
                dist = reference.exact((data / f'{name}.dice').read_text())
                comps, times = bench.parse_alea(components(dist))
                self.assertEqual(times, {'alea_seconds': 0.25, 'alea_compile_seconds': 0.01})
                got = bench.alea_answers([q for q, _ in expected], comps)
                self.assertEqual(list(got), [q for q, _ in expected])
                for q, x in expected:
                    self.assertAlmostEqual(got[q], x, delta=1e-9, msg=q)

    def test_a_network_answers_each_value_of_each_variable(self):
        out = ('alea 1 0 0.25\nalea 1 1 0.75\nalea 2 true 0.5\nalea 2 false 0.5\n'
               'alea 3 0 0.2\nalea 3 1 0.3\nalea 3 2 0.5\nalea 3 3 0.0\nalea_seconds 1.5\n')
        comps, times = bench.parse_alea(out)
        self.assertEqual(times, {'alea_seconds': 1.5})
        got = bench.alea_answers(['a(low)', 'a(high)', 'b', 'c(x)', 'c(y)', 'c(z)'], comps)
        self.assertEqual(got, {'a(low)': 0.25, 'a(high)': 0.75, 'b': 0.5, 'c(x)': 0.2, 'c(y)': 0.3, 'c(z)': 0.5})

    def test_two_integers_of_one_predicate_stay_apart(self):
        # Two components queried under one name: the second starts again at
        # value 0 where a query repeats.
        comps, _ = bench.parse_alea('alea 1 0 0.5\nalea 1 1 0.5\nalea 2 0 0.1\nalea 2 1 0.9\n')
        with self.assertRaises(ValueError):
            bench.alea_answers(['v(0)', 'v(1)', 'v(0)', 'v(1)'], comps)

    def test_a_mismatch_is_an_error(self):
        comps, _ = bench.parse_alea('alea 1 true 0.5\nalea 1 false 0.5\n')
        for qs in [[], ['a', 'b'], ['a(0)']]:
            with self.subTest(queries=qs):
                if qs == ['a(0)']:
                    # A Boolean component answers one query whatever its name.
                    self.assertEqual(bench.alea_answers(qs, comps), {'a(0)': 0.5})
                    continue
                with self.assertRaises(ValueError):
                    bench.alea_answers(qs, comps)
        comps, _ = bench.parse_alea('alea 1 0 0.5\nalea 1 1 0.5\n')
        with self.assertRaises(ValueError):
            bench.alea_answers(['v(0)', 'v(1)', 'v(2)'], comps)
        with self.assertRaises(ValueError):
            bench.alea_answers(['b'], comps)


if __name__ == '__main__':
    unittest.main()
