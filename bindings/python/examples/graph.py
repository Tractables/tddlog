"""A program built from Python data: the probability that a path joins two
nodes of a graph whose edges are present independently, ProbLog's system
test 7. ProbLog answers path(1,5) 0.25824 and path(1,6) 0.2167296.

Run it with

    python examples/graph.py

to print

    path(1,5): 0.258240
    path(1,6): 0.216730
"""

import tddlog

EDGES = {
    (1, 2): 0.6,
    (1, 3): 0.1,
    (2, 5): 0.4,
    (2, 6): 0.3,
    (3, 4): 0.3,
    (4, 5): 0.8,
    (5, 6): 0.2,
}
QUERIES = [(1, 5), (1, 6)]

facts = [f"{p}::edge({a},{b})." for (a, b), p in EDGES.items()]
rules = [
    "path(X,Y) :- edge(X,Y).",
    "path(X,Y) :- edge(X,Z), path(Z,Y).",
]
queries = [f"query(path({a},{b}))." for a, b in QUERIES]
program = "\n".join(facts + rules + queries)

solution = tddlog.solve(program)
for query, probability in solution.answers.items():
    print(f"{query}: {probability:.6f}")
