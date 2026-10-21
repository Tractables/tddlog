#!/usr/bin/env python3
"""Storm (through stormpy) on a PRISM model and its probability
properties, for `bench.py run --storm`: Rubicon's weather factory with its
bounded reachability property, and the models dice2prism.py writes with
theirs. Usage:

    storm_check.py MODEL.prism ENGINE PROPERTIES

PROPERTIES is a file of properties, one a line (an optional `"name":`, a
`;` at the end and `//` comments, as dice2prism.py writes them), or one
property. ENGINE is `sparse` (the explicit model in double precision),
`exact` (the explicit model in rational numbers) or `dd` (the model as
decision diagrams, Sylvan on one thread, double precision). The explicit
engines solve unbounded reachability with Storm's topological solver, which
on an acyclic chain computes each state's value once from its successors'
(Storm's default for doubles, GMRES to a precision of 1e-6, would not be
exact); the decision diagrams iterate to a relative precision of 1e-15.
The model is built once for all the properties. Prints `result X` for each
property in order (17 digits; with `exact` followed by the rational),
`states N`, `transitions M` and `inference_seconds Y` (building the model
and checking it, after the import)."""
import os
import sys
import time

import stormpy

model_path, engine, spec = sys.argv[1:4]
if engine not in ('sparse', 'exact', 'dd'):
    sys.exit(f'engine {engine}: sparse, exact or dd')
lines = open(spec).read().splitlines() if os.path.exists(spec) else [spec]
formulas = []
for line in lines:
    line = line.split('//')[0].strip().rstrip(';').strip()
    if line.startswith('"'):
        line = line.split('":', 1)[1].strip()   # the property's name
    if line:
        formulas.append(line)
stormpy.set_settings(['--sylvan:threads', '1'] +
                     (['--precision', '1e-15'] if engine == 'dd' else ['--eqsolver', 'topological']))
env = stormpy.Environment()
if engine != 'dd':
    env.solver_environment.set_linear_equation_solver_type(stormpy.EquationSolverType.topological)
prog = stormpy.parse_prism_program(model_path)
props = [p for f in formulas for p in stormpy.parse_properties(f, prog)]
t0 = time.time()
values = []
if engine == 'dd':
    m = stormpy.build_symbolic_model(prog, props)
    for p in props:
        r = stormpy.check_model_dd(m, p, only_initial_states=True, environment=env)
        r.filter(stormpy.create_filter_initial_states_symbolic(m))
        values.append(r.min)
else:
    m = (stormpy.build_sparse_exact_model if engine == 'exact' else stormpy.build_model)(prog, props)
    for p in props:
        values.append(stormpy.model_checking(m, p, environment=env).at(m.initial_states[0]))
secs = time.time() - t0
for v in values:
    print(f'result {float(v):.17g}' + (f' {v}' if engine == 'exact' else ''))
print(f'states {m.nr_states}')
print(f'transitions {m.nr_transitions}')
print(f'inference_seconds {secs:.6f}')
