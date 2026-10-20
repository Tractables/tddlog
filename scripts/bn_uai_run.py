#!/usr/bin/env python3
"""pyAgrum's and pgmpy's exact inference on a UAI network, for `bench.py
run --agrum` and `--pgmpy`. Usage:

    bn_uai_run.py SYSTEM MODEL.uai EVID OUT MAR|PR [I,J,...]

SYSTEM is `agrum` (LazyPropagation on a BAYES file, ShaferShenoy on a
MARKOV file: the junction tree engines) or `pgmpy` (its variable
elimination, one query a variable). The observations of EVID are applied by slicing the
tables, so that both libraries run on the same reduced network. OUT is
written in Merlin's format (`PR`, ln Z and Z; `STATUS`; `MAR`, the number of
variables, then each one's domain size and marginals), 12 digits, which
`bench.py` reads as it reads Merlin's. MAR gives the marginals of the
variables listed last (all by default; an observed one is its indicator). The
seconds of the inference proper, after the imports and the construction of
the model, go to the last line of the output: `inference_seconds X`.
Everything runs in one thread."""
import argparse
import math
import os
import sys
import time

for v in ('OMP_NUM_THREADS', 'OPENBLAS_NUM_THREADS', 'MKL_NUM_THREADS'):
    os.environ.setdefault(v, '1')

import numpy as np  # noqa: E402  (after the thread limits)


def read_uai(path):
    tok = open(path).read().split()
    kind, pos = tok[0].upper(), 1
    n = int(tok[pos]); pos += 1
    sizes = [int(x) for x in tok[pos:pos + n]]; pos += n
    m = int(tok[pos]); pos += 1
    scopes = []
    for _ in range(m):
        k = int(tok[pos]); scopes.append([int(x) for x in tok[pos + 1:pos + 1 + k]]); pos += 1 + k
    tables = []
    for sc in scopes:
        e = int(tok[pos]); pos += 1
        a = np.array(tok[pos:pos + e], dtype=float); pos += e
        tables.append(a.reshape([sizes[v] for v in sc]))
    return kind, sizes, scopes, tables


def read_evid(path):
    tok = open(path).read().split()
    k = int(tok[0]) if tok else 0
    return {int(tok[1 + 2 * i]): int(tok[2 + 2 * i]) for i in range(k)}


def reduce(sizes, scopes, tables, evid):
    """The factors with the observed variables sliced out, and the logarithm
    of the product of the factors that are left without a variable."""
    out, log_const = [], 0.0
    for sc, t in zip(scopes, tables):
        idx = tuple(evid[v] if v in evid else slice(None) for v in sc)
        t = t[idx]
        rest = [v for v in sc if v not in evid]
        if not rest:
            log_const += math.log(float(t)) if float(t) > 0 else -math.inf
        else:
            out.append((rest, np.asarray(t)))
    return out, log_const


def run_pgmpy(sizes, factors, qvars, want_z):
    from pgmpy.factors.discrete import DiscreteFactor
    from pgmpy.inference import VariableElimination
    from pgmpy.models import DiscreteMarkovNetwork as MarkovNetwork
    fs = [DiscreteFactor([str(v) for v in sc], [sizes[v] for v in sc], t) for sc, t in factors]
    present = sorted({v for sc, _ in factors for v in sc})
    mn = MarkovNetwork()
    mn.add_nodes_from(str(v) for v in present)
    for sc, _ in factors:
        mn.add_edges_from((str(a), str(b)) for i, a in enumerate(sc) for b in sc[i + 1:])
    mn.add_factors(*fs)
    t0 = time.time()
    log_z = 0.0
    marg = {}
    ve = VariableElimination(mn)
    # On a Markov network `query` leaves its factors unnormalized: the entries
    # of a one-variable marginal sum to Z (the other components' too).
    if want_z and present:
        s = float(np.sum(ve.query([str(present[0])], joint=False, show_progress=False)[str(present[0])].values))
        log_z = math.log(s) if s > 0 else -math.inf
    for v in qvars if not want_z else []:
        if v in present:
            marg[v] = np.asarray(ve.query([str(v)], joint=False, show_progress=False)[str(v)].values, dtype=float)
    return log_z, marg, time.time() - t0


def run_agrum(kind, sizes, scopes, tables, evid, factors, qvars, want_z):
    import pyagrum as gum
    gum.setNumberOfThreads(1)
    present = sorted({v for sc, _ in factors for v in sc})
    name = {v: f'x{v}' for v in range(len(sizes))}
    marg = {}
    if kind == 'BAYES':
        # a Bayesian network, the engine of aGrUM's documentation; the evidence is its own
        bn = gum.BayesNet()
        for v in range(len(sizes)):
            bn.add(gum.LabelizedVariable(name[v], name[v], sizes[v]))
        for sc in scopes:
            for p in sc[:-1]:
                bn.addArc(name[p], name[sc[-1]])
        for sc, t in zip(scopes, tables):
            cpt = bn.cpt(name[sc[-1]])
            order = [int(n[1:]) for n in reversed(cpt.names)]   # aGrUM's flat layout: the last name slowest
            cpt.fillWith(np.ascontiguousarray(np.transpose(t, [sc.index(v) for v in order])).reshape(-1))
        ie = gum.LazyPropagation(bn)
        t0 = time.time()
        ie.setEvidence({name[v]: int(u) for v, u in evid.items()})
        for v in qvars:
            if v not in evid:
                ie.addTarget(name[v])
        ie.makeInference()
        log_z = math.log(ie.evidenceProbability()) if want_z and ie.evidenceProbability() > 0 else (-math.inf if want_z else 0.0)
        for v in qvars:
            if v not in evid:
                marg[v] = np.asarray(ie.posterior(name[v]).toarray(), dtype=float).reshape(-1)
        return log_z, marg, time.time() - t0
    mrf = gum.MarkovRandomField()
    for v in present:
        mrf.add(gum.LabelizedVariable(name[v], name[v], sizes[v]))
    # the engine holds one factor a set of variables: the observations can leave two alike
    merged = {}
    for sc, t in factors:
        key = tuple(sorted(sc))
        t = np.transpose(t, [sc.index(v) for v in key])
        merged[key] = merged[key] * t if key in merged else t
    for sc, t in ((list(k), t) for k, t in merged.items()):
        names = [name[v] for v in sc]
        mrf.addFactor(names)
        f = mrf.factor(names)
        order = [int(n[1:]) for n in reversed(f.names)]
        f.fillWith(np.ascontiguousarray(np.transpose(t, [sc.index(v) for v in order])).reshape(-1))
    ie = gum.ShaferShenoyMRFInference(mrf)
    t0 = time.time()
    for v in qvars:
        if v in present:
            ie.addTarget(name[v])
    ie.makeInference()
    log_z = math.log(ie.evidenceProbability()) if want_z else 0.0
    for v in qvars:
        if v in present:
            marg[v] = np.asarray(ie.posterior(name[v]).toarray(), dtype=float).reshape(-1)
    return log_z, marg, time.time() - t0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('system', choices=['agrum', 'pgmpy'])
    ap.add_argument('model')
    ap.add_argument('evid')
    ap.add_argument('out')
    ap.add_argument('task', choices=['MAR', 'PR'])
    ap.add_argument('vars', nargs='?', default='')
    a = ap.parse_args()
    kind, sizes, scopes, tables = read_uai(a.model)
    evid = read_evid(a.evid)
    qvars = [int(x) for x in a.vars.split(',')] if a.vars else list(range(len(sizes)))
    factors, log_const = reduce(sizes, scopes, tables, evid)
    want_z = a.task == 'PR'
    if a.system == 'pgmpy':
        log_z, marg, secs = run_pgmpy(sizes, factors, qvars, want_z)
    else:
        log_z, marg, secs = run_agrum(kind, sizes, scopes, tables, evid, factors, qvars, want_z)
        if kind == 'BAYES':
            log_const = 0.0   # the engine's evidence probability holds it
    consistent = log_const > -math.inf and log_z > -math.inf
    lines = ['PR', f'{log_z + log_const:.12g} ({math.exp(log_z + log_const) if consistent else 0.0:.12e})',
             'STATUS', 'true' if consistent else 'false']
    if a.task == 'MAR' and consistent:
        lines += ['MAR', str(len(qvars))]
        for v in qvars:
            if v in evid:
                row = [1.0 if u == evid[v] else 0.0 for u in range(sizes[v])]
            elif v in marg:
                row = marg[v] / marg[v].sum()
            else:
                row = [1.0 / sizes[v]] * sizes[v]      # a variable no table mentions
            lines.append(' '.join([str(sizes[v])] + [f'{float(p):.12g}' for p in row]))
    else:
        lines += ['MAR', '0']
    lines.append(f'inference_seconds {secs:.6f}')
    with open(a.out, 'w') as f:
        f.write('\n'.join(lines) + '\n')


if __name__ == '__main__':
    main()
