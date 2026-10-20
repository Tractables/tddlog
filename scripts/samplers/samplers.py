#!/usr/bin/env python3
"""samplers.py: the bit-blasting paper's other systems on its programs,
from the artifact's own Stan, WebPPL and Psi programs (data/hybit, from
Alea.jl branch hybit 96b69a5: baselines/ and benchmarks/, and the Psi
programs with the cobserve rewrites of altermu2, conjugate_gaussians and
normal_mixture, see its README).

    samplers.py prepare                print a `compile MODEL` line per Stan model
    samplers.py compile MODEL          build one Stan model (in STAN_WORK)
    samplers.py jobs SEEDS OUT         print a `series OUT ...` line per series
    samplers.py psijobs OUT            print a `psi OUT ...` line per Psi program
    samplers.py series OUT SYSTEM PROGRAM SEED
    samplers.py psi OUT PROGRAM

The `prepare`, `jobs` and `psijobs` lines are commands for this script,
to be run one a processor in parallel (`compile` lines first).

A series is one sampler on one program with one seed: sample counts N =
10^2 (WebPPL) or 10^3 (Stan) up to 10^7, ten times more each run, each run
timed (the wall time of the process, and the sampler's own time where it
prints one) and its estimates recorded. The series stops after a run that
failed or took more than STOP seconds (60): the next, with ten times the
samples, would pass the TIMEOUT (600 s). Stan runs one chain with 100
warm-up iterations, as the artifact's gene expression runs; above 10^6
samples it keeps every (N / 10^6)th draw so that its CSV stays at 10^6
rows. WebPPL runs the artifact's program with `--m METHOD --s N` (samples
and particles both N), its default kernels. Psi runs each program once
with `--mathematica`, then twice more when the first took less than STOP
seconds, TIMEOUT each. One JSON record per run appended to OUT under a
lock; estimates are compared with the ground truth afterwards, not here.

The environment names the tools: RUN_STAN, RUN_WEBPPL (the artifact
runners stan.sh and webppl.sh beside this script), STAN_HOME,
WEBPPL_HOME, NODE, PSI, STAN_WORK (where models are built, one directory a
program), PROGS (data/hybit).
"""
import fcntl
import json
import os
import signal
import subprocess
import sys
import time

TIMEOUT = 600
STOP = 60
GENES = [5, 10, 15, 20, 25, 30, 40, 50]   # bench.py's genes ladder up to the artifact's 50

# program: (directory under PROGS/baselines or benchmarks, model name);
# Table 2 marks Stan unsupported on weekend and GPA; the artifact's GPA.stan
# needs data it does not ship, weekend.stan runs and is kept.
STAN = {
    'pi': ('baselines/stan/pi', 'pi'), 'weekend': ('baselines/stan/weekend', 'weekend'),
    'spacex': ('baselines/stan/spacex', 'spacex'), 'tug_of_war': ('baselines/stan/tug_of_war', 'tug_of_war'),
    'altermu2': ('baselines/stan/altermu2', 'altermu2'),
    'conjugate_gaussians': ('baselines/stan/conjugate_gaussians', 'conjugate_gaussians'),
    'normal_mixture': ('baselines/stan/normal_mixture', 'normal_mixture'),
    'zeroone': ('baselines/stan/zeroone', 'zeroone'), 'coinBias': ('baselines/stan/coinBias', 'coinBias'),
    'addFun_sum': ('baselines/stan/addFun_sum', 'addFun_sum'),
    'clickGraph': ('baselines/stan/clickGraph', 'clickGraph'), 'trueskill': ('baselines/stan/trueskill', 'trueskill'),
    'clinicalTrial1': ('baselines/stan/clinicalTrial1', 'clinicalTrial1'),
    'clinicalTrial2': ('baselines/stan/clinicalTrial2', 'clinicalTrial2'),
    'addFun_max': ('baselines/stan/addFun_max', 'addFun_max'),
    'multimodal': ('benchmarks/multimodal/multimodal', 'multimodal'),
    'conjugate_ex3': ('benchmarks/conjugate_gaussians/conjugate_gaussians', 'conjugate_gaussians'),
}
STAN.update({f'genes_{t}': ('benchmarks/or/or', f'or_{t}') for t in GENES})
W = 'baselines/webppl/'
WEBPPL = {
    'pi': W + 'pi/pi.wppl', 'weekend': W + 'weekend/weekend.wppl', 'spacex': W + 'spacex/spacex.wppl',
    'GPA': W + 'GPA/GPA.wppl', 'tug_of_war': W + 'tug_of_war/tug_of_war.wppl',
    'altermu2': W + 'altermu2/altermu2.wppl', 'conjugate_gaussians': W + 'conjugate_gaussians/conjugate_gaussians.wppl',
    'normal_mixture_theta': W + 'normal_mixture/normal_mixture.wppl',
    'normal_mixture_mu1': W + 'normal_mixture/normal_mixture2.wppl',
    'normal_mixture_mu2': W + 'normal_mixture/normal_mixture3.wppl',
    'zeroone_w1': W + 'zeroone/zeroone.wppl', 'zeroone_w2': W + 'zeroone/zeroone2.wppl',
    'coinBias': W + 'coinBias/coinBias.wppl', 'addFun_sum': W + 'addFun_sum/addFun_sum.wppl',
    'clickGraph': W + 'clickGraph/clickGraph.wppl', 'trueskill': W + 'trueskill/trueskill.wppl',
    'clinicalTrial1': W + 'clinicalTrial1/clinicalTrial1.wppl',
    'clinicalTrial2': W + 'clinicalTrial2/clinicalTrial2.wppl', 'addFun_max': W + 'addFun_max/addFun_max.wppl',
    'multimodal': W + 'multimodal/multimodal.wppl',
    'conjugate_ex3': W + 'conjugate_gaussians3/conjugate_gaussians.wppl',
}
WEBPPL.update({f'genes_{t}': W + f'or_{t}/or_{t}.wppl' for t in GENES})
METHODS = ['rejection', 'MCMC', 'SMC']
# Psi: program -> file under PROGS/psi (cobs/: the cobserve rewrites).
PSI = {
    'weekend': 'weekend.psi', 'GPA': 'GPA.psi', 'altermu2': 'cobs/altermu2.psi',
    'conjugate_gaussians': 'cobs/conjugate_gaussians.psi', 'normal_mixture': 'cobs/normal_mixture.psi',
    'zeroone': 'zeroone.psi', 'coinBias': 'coinBiasEquivalent.psi',
    'addFun_max': 'max.psi', 'clinicalTrial1': 'clinicalTrial1.psi', 'clinicalTrial2': 'clinicalTrial2.psi',
    'pi': 'pi.psi', 'spacex': 'spacex.psi', 'trueskill': 'trueskill.psi', 'tug_of_war': 'tug_of_war.psi',
}
PSI.update({f'genes_{t}': f'or/or_{t}.psi' for t in GENES})


def env(name):
    v = os.environ.get(name)
    if not v:
        sys.exit(f'samplers.py: set {name}')
    return v


def append(out, rec):
    with open(out, 'a') as f:
        fcntl.flock(f, fcntl.LOCK_EX)
        f.write(json.dumps(rec) + '\n')
        f.flush()


def run(cmd, extra_env=None, cwd=None):
    """Run cmd in a session of its own, so that a timeout kills its children
    (node, a Stan model) too."""
    e = dict(os.environ, **(extra_env or {}))
    t0 = time.monotonic()
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=e, cwd=cwd,
                         start_new_session=True)
    try:
        out, err = p.communicate(timeout=TIMEOUT)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        out, err = p.communicate()
        rc = 'timeout'
    return rc, round(time.monotonic() - t0, 3), out, err


def parse(out):
    est, sampler = {}, None
    for line in out.splitlines():
        f = line.split()
        if len(f) == 3 and f[0] == 'estimate':
            try:
                est[f[1]] = float(f[2])
            except ValueError:
                est[f[1]] = None
        elif f and f[0] == 'sampler_seconds':
            nums = [float(x) for x in f[1:] if x.replace('.', '', 1).replace('e-', '', 1).isdigit()]
            sampler = round(sum(nums), 3) if nums else None
    return est, sampler


def stan_env(prog):
    return {'STAN_WORK': os.path.join(env('STAN_WORK'), prog), 'WARMUP': '100'}


def series(out, system, prog, seed):
    progs = env('PROGS')
    if system == 'stan':
        d, model = STAN[prog]
        ns = [10 ** k for k in range(3, 8)]
    else:
        method = system.split(':', 1)[1]
        ns = [10 ** k for k in range(2, 8)]
    # MAX_SAMPLES caps the ladder, for a quick check of the script.
    ns = [n for n in ns if n <= int(os.environ.get('MAX_SAMPLES', 10 ** 7))]
    for n in ns:
        if system == 'stan':
            extra = stan_env(prog)
            extra['STAN_NAME'] = model
            if n > 10 ** 6:
                extra['STAN_EXTRA'] = f'thin={n // 10 ** 6}'
            cmd = [env('RUN_STAN'), os.path.join(progs, d), str(n), str(seed)]
        else:
            extra = {'WEBPPL_TIMEOUT': '0'}
            cmd = [env('RUN_WEBPPL'), os.path.join(progs, WEBPPL[prog]), method, str(n), str(seed)]
        rc, secs, o, e = run(cmd, extra)
        est, sampler = parse(o)
        status = 'ok' if rc == 0 and est else ('timeout' if rc == 'timeout' else 'error')
        rec = {'system': system, 'program': prog, 'seed': seed, 'samples': n, 'status': status,
               'seconds': secs, 'sampler_seconds': sampler, 'estimates': est}
        if status != 'ok':
            rec['error'] = (e or o)[-600:]
        append(out, rec)
        if status != 'ok' or secs > STOP:
            break


def psi(out, prog):
    d = os.path.join(env('PROGS'), 'psi')
    for rep in range(3):
        rc, secs, o, e = run([env('PSI'), '--mathematica', PSI[prog]], cwd=d)
        status = 'ok' if rc == 0 else ('timeout' if rc == 'timeout' else 'error')
        rec = {'system': 'psi', 'program': prog, 'rep': rep, 'status': status, 'seconds': secs,
               'output': o[-20000:]}
        if status != 'ok':
            rec['error'] = (e or o)[-600:]
        append(out, rec)
        if status != 'ok' or secs > STOP:
            break


def compile_model(prog):
    d, model = STAN[prog]
    # 10 draws: stan.sh builds the model once, then samples.
    rc, secs, o, e = run([env('RUN_STAN'), os.path.join(env('PROGS'), d), '10', '1'],
                         dict(stan_env(prog), STAN_NAME=model))
    print(f'### compile {prog} rc {rc} {secs} s' + ('' if rc == 0 else ': ' + (e or o)[-400:].replace('\n', ' | ')))


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else ''
    if cmd == 'prepare':
        for p in STAN:
            print(f'compile {p}')
    elif cmd == 'compile':
        compile_model(sys.argv[2])
    elif cmd == 'jobs':
        # jobs SEEDS OUT: the series of each seed, writing to OUT; a sampler
        # whose series on a program are all in OUT already with no run that
        # answered (a rejection sampler that timed out at 100 samples) is
        # not run again with another seed.
        seeds, out = sys.argv[2].split(','), sys.argv[3]
        answered, tried = set(), set()
        if os.path.exists(out):
            for line in open(out):
                r = json.loads(line)
                tried.add((r['system'], r['program']))
                if r['status'] == 'ok':
                    answered.add((r['system'], r['program']))
        pairs = [('stan', p) for p in STAN] + [(f'webppl:{m}', p) for p in WEBPPL for m in METHODS]
        for seed in seeds:
            for system, p in pairs:
                if (system, p) in tried and (system, p) not in answered:
                    continue
                print(f'series {out} {system} {p} {seed}')
    elif cmd == 'psijobs':
        for p in PSI:
            print(f'psi {sys.argv[2]} {p}')
    elif cmd == 'series':
        out, system, prog, seed = sys.argv[2:6]
        series(out, system, prog, int(seed))
    elif cmd == 'psi':
        psi(sys.argv[2], sys.argv[3])
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main()
