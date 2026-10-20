"""Tests for the tddlog extension module, run against an installed wheel.

The answers are checked against ProbLog's, recorded in the `%Expected
outcome:` comments of the ProbLog system tests the repository copies into
tests/problog, and against probabilities computed by hand.
"""

import ast
import inspect
import math
import runpy
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

import tddlog

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[2]
PROBLOG = REPOSITORY / "tests" / "problog"

# A varied dozen of ProbLog's system tests, all small: conjunction,
# disjunction, negation, probabilistic rules, annotated disjunctions,
# evidence, recursion, and a negative query.
PROGRAMS = [
    "00_trivial_and.pl",
    "00_trivial_not.pl",
    "00_trivial_or.pl",
    "10_cards.pl",
    "11_ads_numerical.pl",
    "3_tossing_coin.pl",
    "4_bayesian_net.pl",
    "7_probabilistic_graph.pl",
    "8_smokers_network.pl",
    "add.pl",
    "negation.pl",
    "negative_query.pl",
    "smokers_or.pl",
    "tc_3.pl",
]

STRATEGIES = ["auto", "td", "closed", "dynamic", "program"]

ALARM = """
0.1::burglary.
0.2::earthquake.
0.7::hears_alarm(john).
0.4::hears_alarm(mary).
alarm :- burglary.
alarm :- earthquake.
calls(X) :- alarm, hears_alarm(X).
evidence(calls(john), true).
query(burglary).
query(earthquake).
query(calls(mary)).
"""


def expected_outcome(source):
    """The `%Expected outcome:` block of a ProbLog system test, as a dict
    from each query, without whitespace, to ProbLog's probability."""
    expected = {}
    reading = False
    for line in source.splitlines():
        text = line.strip()
        if text.startswith(("%Expected outcome:", "% Expected outcome:")):
            reading = True
            continue
        if not reading:
            continue
        if not text.startswith("%"):
            break
        body = text.lstrip("%").strip()
        if not body:
            continue
        parts = body.rsplit(None, 1)
        try:
            expected["".join(parts[0].split())] = float(parts[1])
        except (IndexError, ValueError):
            break
    return expected


@pytest.mark.parametrize("name", PROGRAMS)
@pytest.mark.parametrize("vtree", STRATEGIES)
def test_answers_agree_with_problog(name, vtree):
    source = (PROBLOG / name).read_text()
    expected = expected_outcome(source)
    assert expected, f"{name} states no expected outcome"
    answers = tddlog.solve(source, vtree=vtree).answers
    assert answers is not None
    for query, probability in expected.items():
        assert answers[query] == pytest.approx(probability, abs=1e-6), query


def test_alarm_by_hand():
    solution = tddlog.solve(ALARM, evidence_probability=True)
    # P(alarm) = 1 - 0.9 * 0.8 = 0.28, P(e) = P(calls(john)) = 0.28 * 0.7.
    evidence = 0.28 * 0.7
    assert list(solution.answers) == ["burglary", "earthquake", "calls(mary)"]
    assert solution.answers["burglary"] == pytest.approx(0.1 * 0.7 / evidence, abs=1e-12)
    assert solution.answers["earthquake"] == pytest.approx(0.2 * 0.7 / evidence, abs=1e-12)
    assert solution.answers["calls(mary)"] == pytest.approx(0.4, abs=1e-12)
    assert solution.evidence_probability == pytest.approx(evidence, abs=1e-12)
    assert solution.log10_evidence_probability == pytest.approx(math.log10(evidence), abs=1e-12)


def test_evidence_probability_only_where_asked():
    solution = tddlog.solve(ALARM)
    assert solution.evidence_probability is None
    assert solution.log10_evidence_probability is None


def test_queries_left_out():
    solution = tddlog.solve(ALARM, evidence_probability=True, queries=False)
    assert solution.answers == {}
    assert solution.evidence_probability == pytest.approx(0.196, abs=1e-12)


def test_no_evidence_has_probability_one():
    solution = tddlog.solve("0.3::a. query(a).", evidence_probability=True)
    assert solution.answers == {"a": pytest.approx(0.3, abs=1e-12)}
    assert solution.evidence_probability == 1.0
    assert solution.log10_evidence_probability == 0.0


def test_inconsistent_evidence():
    program = "0.3::a. 0.4::b. c :- a, b. evidence(a, false). evidence(c, true). query(b)."
    solution = tddlog.solve(program, evidence_probability=True)
    assert solution.answers is None
    assert solution.evidence_probability == 0.0
    assert solution.log10_evidence_probability == -math.inf


def test_probability_below_the_least_float():
    # 400 independent observations of probability 1/10 each: P(e) = 1e-400,
    # under the least float, whose logarithm is still exact.
    facts = "".join(f"0.1::f({i}). evidence(f({i}), true). " for i in range(400))
    solution = tddlog.solve(facts + "0.5::q. query(q).", evidence_probability=True)
    assert solution.answers == {"q": pytest.approx(0.5, abs=1e-12)}
    assert solution.evidence_probability == 0.0
    assert solution.log10_evidence_probability == pytest.approx(-400.0, abs=1e-9)


def test_errors():
    with pytest.raises(tddlog.ParseError, match="line 1"):
        tddlog.solve("0.3::a query(a).")
    with pytest.raises(tddlog.GroundingError):
        tddlog.solve("p(X) :- q(Y). q(1). query(p(1)).")
    unsound = (REPOSITORY / "examples" / "unsound.pl").read_text()
    with pytest.raises(tddlog.CompilationError, match="well-founded"):
        tddlog.solve(unsound)
    for error in (tddlog.ParseError, tddlog.GroundingError, tddlog.CompilationError):
        assert issubclass(error, tddlog.Error)


def test_option_errors():
    with pytest.raises(ValueError, match="unknown vtree strategy"):
        tddlog.solve(ALARM, vtree="balanced")
    with pytest.raises(ValueError, match="nothing to answer"):
        tddlog.solve(ALARM, queries=False)


def test_threads_solve_concurrently():
    source = (PROBLOG / "8_smokers_network.pl").read_text()
    expected = expected_outcome(source)
    with ThreadPoolExecutor(max_workers=4) as pool:
        solutions = list(pool.map(lambda vtree: tddlog.solve(source, vtree=vtree), STRATEGIES * 2))
    for solution in solutions:
        for query, probability in expected.items():
            assert solution.answers[query] == pytest.approx(probability, abs=1e-6)


@pytest.mark.parametrize("example", sorted((HERE.parent / "examples").glob("*.py")), ids=lambda p: p.name)
def test_examples_print_what_they_say(example, capsys):
    runpy.run_path(str(example), run_name="__main__")
    printed = capsys.readouterr().out
    # Each example's docstring ends with the lines it prints, indented.
    docstring = ast.get_docstring(ast.parse(example.read_text()))
    stated = docstring.split("to print\n", 1)[1]
    assert printed.splitlines() == [line.strip() for line in stated.strip().splitlines()]


def test_readme_prints_what_it_says(capsys):
    # The README's example, which the site's page shows too, and the lines
    # it says the example prints.
    readme = (HERE.parent / "README.md").read_text()
    code = readme.split("```python\n", 1)[1].split("```", 1)[0]
    stated = readme.split("prints\n\n```text\n", 1)[1].split("```", 1)[0]
    exec(compile(code, "README.md", "exec"), {})
    assert capsys.readouterr().out == stated


def test_module_surface():
    assert isinstance(tddlog.__version__, str)
    assert set(tddlog.__all__) == {
        "CompilationError",
        "Error",
        "GroundingError",
        "ParseError",
        "Solution",
        "__version__",
        "solve",
    }


def test_repr_shows_the_values():
    solution = tddlog.solve("0.5::a. query(a). query(\\+a).", evidence_probability=True)
    assert repr(solution) == "Solution(answers={'a': 0.5, '\\\\+a': 0.5}, evidence_probability=1.0)"
    solution = tddlog.solve("0.5::a. evidence(a). evidence(\\+a). query(a).")
    assert repr(solution) == "Solution(answers=None, evidence_probability=None)"


def test_solve_stub_matches_runtime_defaults():
    stub = ast.parse((HERE.parent / "tddlog.pyi").read_text())
    declaration = next(
        node for node in stub.body if isinstance(node, ast.FunctionDef) and node.name == "solve"
    )
    signature = inspect.signature(tddlog.solve)
    assert [a.arg for a in declaration.args.args + declaration.args.kwonlyargs] == list(signature.parameters)
    for argument, default in zip(declaration.args.kwonlyargs, declaration.args.kw_defaults):
        assert signature.parameters[argument.arg].default == ast.literal_eval(default)
