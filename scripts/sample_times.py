# Each system's time against Merlin's in the result files of a sample run: python3 sample_times.py DIR [REF]
# (REF, default merlin:bte, or merlin:bte-pr for the `-pr` variants: the run the others are compared with). Per system: the
# instances where it and REF both finished, the geometric mean of the ratio of its seconds to REF's, the slowest
# ratio and the instance it comes from, and the number of errors and timeouts. Runs under 0.1 s are left out (rule 16).
import json, sys, glob, math, collections
ref = sys.argv[2] if len(sys.argv) > 2 else None
runs = collections.defaultdict(dict)  # instance -> system:variant -> record
for f in sorted(glob.glob(sys.argv[1] + '/*.jsonl')):
    for line in open(f):
        r = json.loads(line)
        runs[(f.split('/')[-1], r['instance'])][r['system'] + ':' + r['variant']] = r
stats = collections.defaultdict(lambda: {'ratios': [], 'bad': 0, 'n': 0, 'worst': (0, '')})
for (fn, inst), d in runs.items():
    for k, r in d.items():
        # the PR variants (`-pr`) against Merlin's PR run, the others against its marginals run
        rk = ref or ('merlin:bte-pr' if k.endswith('-pr') else 'merlin:bte')
        s = stats[k]
        s['n'] += 1
        if r['status'] != 'ok':
            s['bad'] += 1
        elif rk in d and d[rk]['status'] == 'ok' and k != rk and max(r['seconds'], d[rk]['seconds']) >= 0.1:
            ratio = r['seconds'] / max(d[rk]['seconds'], 0.01)
            s['ratios'].append(ratio)
            if ratio > s['worst'][0]:
                s['worst'] = (ratio, f'{inst} {r["seconds"]:.1f}s vs {d[rk]["seconds"]:.1f}s')
print(f"{'system':28} {'n':>4} {'bad':>4} {'both':>5} {'geomean':>9}  slowest")
for k, s in sorted(stats.items()):
    g = math.exp(sum(map(math.log, s['ratios'])) / len(s['ratios'])) if s['ratios'] else float('nan')
    print(f"{k:28} {s['n']:4d} {s['bad']:4d} {len(s['ratios']):5d} {g:9.2f}  {s['worst'][0]:.1f}x {s['worst'][1]}")
