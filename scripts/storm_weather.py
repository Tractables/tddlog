#!/usr/bin/env python3
"""Storm (through stormpy) on a PRISM model and the bounded reachability
property of Rubicon's weather factory, for `bench.py run --storm`. Usage:

    storm_weather.py MODEL.prism HORIZON ENGINE

computes `P=? [F<=HORIZON "allStrike"]` in the initial state and prints
`result X` (17 digits) and `inference_seconds Y` (building the model and
checking it, after the import). ENGINE is `sparse` (explicit model, double
precision), `exact` (explicit model, rational numbers) or `dd` (the model
as decision diagrams, double precision)."""
import sys
import time

import stormpy

model_path, horizon, engine = sys.argv[1], sys.argv[2], sys.argv[3]
if engine not in ('sparse', 'exact', 'dd'):
    sys.exit(f'engine {engine}: sparse, exact or dd')
try:
    stormpy.set_settings(['--sylvan:threads', '1'])
except Exception:
    pass
prog = stormpy.parse_prism_program(model_path)
props = stormpy.parse_properties(f'P=? [F<={int(horizon)} "allStrike"]', prog)
t0 = time.time()
if engine == 'sparse':
    m = stormpy.build_model(prog, props)
    r = stormpy.model_checking(m, props[0])
    value = r.at(m.initial_states[0])
elif engine == 'exact':
    m = stormpy.build_sparse_exact_model(prog, props)
    r = stormpy.model_checking(m, props[0])
    value = r.at(m.initial_states[0])
else:
    m = stormpy.build_symbolic_model(prog, props)
    r = stormpy.check_model_dd(m, props[0], only_initial_states=True)
    r.filter(stormpy.create_filter_initial_states_symbolic(m))
    value = r.min
secs = time.time() - t0
print(f'result {float(value):.17g}')
print(f'inference_seconds {secs:.6f}')
