"""The alarm example of Fierens et al. (2015): the probabilities of three
queries given that John calls, and the probability that he calls.

P(alarm) = 1 - 0.9 * 0.8 = 0.28 and P(calls(john)) = 0.28 * 0.7 = 0.196, so
P(burglary | calls(john)) = 0.1 * 0.7 / 0.196 = 0.357143. Run it with

    python examples/alarm.py

to print

    burglary: 0.357143
    earthquake: 0.714286
    calls(mary): 0.400000
    P(e) = 0.196000 (log10 -0.707744)
"""

import tddlog

PROGRAM = """
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

solution = tddlog.solve(PROGRAM, evidence_probability=True)
for query, probability in solution.answers.items():
    print(f"{query}: {probability:.6f}")
print(
    f"P(e) = {solution.evidence_probability:.6f}"
    f" (log10 {solution.log10_evidence_probability:.6f})"
)
