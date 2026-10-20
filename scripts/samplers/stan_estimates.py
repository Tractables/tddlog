#!/usr/bin/env python3
"""stan_estimates.py CSV NAME: print `estimate LABEL VALUE` lines (posterior
means of the queried quantity of benchmark NAME) from a CmdStan output CSV,
and `sampler_seconds warmup W sampling S` from the CSV's timing comment.
Which column answers which query is a table below (see the benchmark notes
in stan.sh)."""
import csv, math, re, sys

def softmax2(a, b):
    m = max(a, b)
    ea, eb = math.exp(a - m), math.exp(b - m)
    return ea / (ea + eb)

# NAME -> list of (label, function(row) -> float)
Q = {
 'pi':                 [('pi.P_answer_is_1', lambda r: r['answer'])],
 'weekend':            [('weekend.P_weekend', lambda r: softmax2(r['fp.1'], r['fp.2']))],
 'spacex':             [('spacex.E_cr', lambda r: r['cr'])],
 'tug_of_war':         [('tug_of_war.E_ans', lambda r: r['ans'])],
 'altermu2':           [('altermu2.E_mu1', lambda r: r['mu.1'])],
 'conjugate_gaussians':   [('conjugate_gaussians.E_mu', lambda r: r['mu'])],
 'conjugate_gaussians_2': [('conjugate_gaussians_2.E_mu', lambda r: r['mu'])],
 'normal_mixture':     [('normal_mixture.E_theta', lambda r: r['theta']),
                        ('normal_mixture.E_mu1', lambda r: r['mu.1']),
                        ('normal_mixture.E_mu2', lambda r: r['mu.2'])],
 'zeroone':            [('zeroone.E_w1', lambda r: r['w1']),
                        ('zeroone.E_w2', lambda r: r['w2'])],
 'coinBias':           [('coinBias.E_b', lambda r: r['b'])],
 'addFun_sum':         [('addFun_sum.E_sum', lambda r: r['ans'])],
 'addFun_max':         [('addFun_max.E_max', lambda r: r['ans'])],
 'clickGraph':         [('clickGraph.E_similarityAll', lambda r: r['similarityAll'])],
 'trueskill':          [('trueskill.P_final', lambda r: r['final'])],
 # categorical_logit_rng returns 1 or 2; 2 = isEffective true
 'clinicalTrial1':     [('clinicalTrial1.P_isEffective', lambda r: r['isEffective'] - 1)],
 'clinicalTrial2':     [('clinicalTrial2.E_probIfControl', lambda r: r['probIfControl'])],
 'multimodal':         [('multimodal.E_mu1', lambda r: r['mu1'])],
}

def queries(name):
    if name in Q:
        return Q[name]
    if re.fullmatch(r'or_\d+', name):
        return [(name + '.E_prior1', lambda r: r['prior1'])]
    sys.exit('stan_estimates: no query defined for model ' + name)

def main():
    # One row at a time: the gene expression models write up to 2^(T-1)
    # transformed parameters a draw, too many to hold every row.
    path, name = sys.argv[1], sys.argv[2]
    qs = queries(name)
    sums, n, comments, header = [0.0] * len(qs), 0, [], None
    with open(path) as f:
        for line in f:
            if line.startswith('#'):
                comments.append(line)
            elif header is None:
                header = next(csv.reader([line]))
            else:
                r = dict(zip(header, map(float, next(csv.reader([line])))))
                for i, (_, fn) in enumerate(qs):
                    sums[i] += fn(r)
                n += 1
    if not n:
        sys.exit('stan_estimates: no samples in ' + path)
    for (label, _), v in zip(qs, sums):
        print('estimate %s %.6f' % (label, v / n))
    txt = ''.join(comments)
    w = re.search(r'([\d.eE+-]+) seconds \(Warm-up\)', txt)
    s = re.search(r'([\d.eE+-]+) seconds \(Sampling\)', txt)
    if w and s:
        print('sampler_seconds warmup %s sampling %s' % (w.group(1), s.group(1)))
    print('n_samples %d' % n)

main()
