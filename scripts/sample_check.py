# Errors, timeouts and answers that differ (over 1e-9, Merlin aside) from the reference in the result files of
# a sample run (bn_baselines_sample.sh, db_mc_sample.sh): python3 sample_check.py DIR
import json, sys, glob, collections
for f in sorted(glob.glob(sys.argv[1] + '/*.jsonl')):
    recs = [json.loads(l) for l in open(f)]
    print('##', f.split('/')[-1], len(recs))
    by = collections.defaultdict(dict)
    for r in recs:
        by[r['instance']][r['system'] + ':' + r['variant']] = r
    for inst, d in sorted(by.items()):
        ref = None
        for k in ('exact:cliquetree', 'exact:tpch', 'exact:dp', 'merlin:bte'):
            if k in d and d[k]['status'] == 'ok' and isinstance(d[k].get('answers'), dict):
                ref = (k, d[k]['answers']); break
        for k, r in sorted(d.items()):
            if r['status'] != 'ok':
                print(f"{inst} {k} {r['status']} {str(r.get('error',''))[:160]!r}")
            elif ref and isinstance(r.get('answers'), dict) and k != ref[0]:
                diff = max((abs(r['answers'][q] - ref[1][q]) for q in ref[1] if q in r['answers']), default=-1)
                if diff > 1e-9 and not k.startswith('merlin'):
                    print(f"{inst} {k} DIFF {diff:.3g} vs {ref[0]}")
