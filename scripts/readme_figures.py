"""readme_figures.py: the README's figures, drawn from the results page's
data (bindings/wasm/results/suites.json and ppl.json), and the numbers
its text quotes, printed as JSON.

    python3 scripts/readme_figures.py [--png] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG, to look at.

Writes, each as NAME.svg for a light page and NAME-dark.svg for a dark one:
  speedups         tddlog's average speedup over each baseline;
  cactus-GROUP     the instances each system answers within a time limit;
  scatter-GROUP    tddlog's time against the fastest other system's.

Times under 0.1 s count as 0.1 s: an instance both systems answer in under
0.1 s is left out of a comparison, and no ratio is larger for a run that
takes milliseconds. A speedup's average is the geometric mean of the ratios
over the instances both systems answer. Alea.jl's and Roulette's times
are their in-process times, without their language's start-up. Needs
matplotlib."""
import json
import math
import os
import re
import statistics
import sys

import matplotlib

matplotlib.use('svg')
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.ticker import FixedLocator, NullLocator  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FLOOR = 0.1
PNG = '--png' in sys.argv

# The comparisons: the suites of each and the systems drawn in it, tddlog first.
GROUPS = [
    dict(key='programs', title='Probabilistic logic programs',
         suites=['std', 'lit', 'biomine', 'tpch'], systems=['tddlog', 'problog', 'pita', 'aspmc']),
    dict(key='networks', title='Bayesian and Markov networks',
         suites=['std_bn', 'ace', 'large', 'uai', 'uai_pr'],
         systems=['tddlog', 'ace', 'merlin', 'agrum', 'pgmpy', 'libdai', 'tensorinference']),
    dict(key='ppl', title='Probabilistic programs',
         suites=['dice', 'ppl', 'bitblast', 'roulette', 'geni'],
         systems=['tddlog', 'alea', 'dice', 'roulette', 'geni', 'nodice', 'psi', 'genfer', 'sppl', 'pluck',
                  'storm', 'prism']),
    dict(key='databases', title='Probabilistic databases',
         suites=['tpch'], systems=['tddlog', 'provsql']),
]

# (light, dark): tddlog's colour, then the baselines' in the order a figure lists them.
TDDLOG = ('#0d6670', '#39c5cf')
PALETTE = [('#d55e00', '#ff8a4c'), ('#0072b2', '#58a6ff'), ('#e69f00', '#f2b84b'), ('#cc79a7', '#e7a4cc'),
           ('#6a3d9a', '#b392f0'), ('#8c564b', '#c9a38f'), ('#999933', '#c8c86a'), ('#56b4e9', '#9ad4f5'),
           ('#882255', '#e0709f'), ('#7f7f7f', '#a8a8a8'), ('#332288', '#8b8bf0'), ('#117733', '#56d364')]
THEMES = {
    '': dict(text='#1f2328', muted='#59636e', grid='#d1d9e0', i=0),
    '-dark': dict(text='#e6edf3', muted='#9198a1', grid='#3d444d', i=1),
}
FONTS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def load():
    names, suites = {}, {}
    for f in ('suites.json', 'ppl.json'):
        data = json.load(open(os.path.join(ROOT, 'bindings', 'wasm', 'results', f)))
        names.update(data['names'])
        for fam in data['families']:
            for s in fam['suites']:
                suites[s['key']] = s
    return names, suites


def rows(suites, keys):
    """[(suite, instance, {system: seconds, or None where it did not answer})],
    a system missing from a row where the suite did not give it the program."""
    out = []
    for k in keys:
        s = suites[k]
        for r in s['instances']:
            out.append((k, r[0], {sys_: (v if isinstance(v, (int, float)) else None)
                                  for sys_, v in zip(s['systems'], r[1:]) if v is not None}))
    return out


def clamp(t):
    return max(t, FLOOR)


def compare(rs, a, b):
    """a against b over the rows: b's time over a's on the rows both answer,
    leaving out those both answer under FLOOR."""
    ratios, both_fast, only_a, only_b, given = [], 0, 0, 0, 0
    for _, _, v in rs:
        if b not in v or a not in v:
            continue
        given += 1
        ta, tb = v[a], v[b]
        if ta is not None and tb is not None:
            if max(ta, tb) < FLOOR:
                both_fast += 1
            else:
                ratios.append(clamp(tb) / clamp(ta))
        elif ta is not None:
            only_a += 1
        elif tb is not None:
            only_b += 1
    out = dict(given=given, both=len(ratios), both_fast=both_fast, only_tddlog=only_a, only_other=only_b)
    if ratios:
        out.update(geomean=math.exp(statistics.fmean(math.log(r) for r in ratios)),
                   median=statistics.median(ratios), faster=sum(r > 1 for r in ratios),
                   slower=sum(r < 1 for r in ratios), max=max(ratios), min=min(ratios))
    return out


def best_other(v, systems):
    ts = [v[s] for s in systems[1:] if v.get(s) is not None]
    return min(ts) if ts else None


def style(theme):
    c = THEMES[theme]
    plt.rcdefaults()
    plt.rcParams.update({
        'svg.fonttype': 'none', 'svg.hashsalt': 'tddlog', 'font.family': 'DejaVu Sans', 'font.size': 10.5,
        'text.color': c['text'], 'axes.labelcolor': c['text'], 'axes.edgecolor': c['muted'],
        'xtick.color': c['muted'], 'ytick.color': c['muted'], 'xtick.labelcolor': c['text'],
        'ytick.labelcolor': c['text'], 'axes.spines.top': False, 'axes.spines.right': False,
        'figure.facecolor': 'none', 'axes.facecolor': 'none', 'savefig.facecolor': 'none',
        'legend.frameon': False, 'axes.titleweight': 'bold', 'axes.titlesize': 12,
    })
    return c


def save(fig, out, name):
    path = os.path.join(out, name + '.svg')
    if PNG:
        fig.savefig(path[:-4] + '.png', format='png', dpi=110, bbox_inches='tight', pad_inches=0.08,
                    facecolor='#0d1117' if name.endswith('-dark') else 'white')
    fig.savefig(path, format='svg', bbox_inches='tight', pad_inches=0.08, metadata={'Date': None})
    plt.close(fig)
    text = open(path).read()
    # The SVG names the font it was measured with; a page draws it with the system's sans serif.
    text = re.sub(r"'?DejaVu Sans'?", FONTS, text)
    open(path, 'w').write(text)


def seconds_axis(ax, hi, axis='x'):
    ticks = [t for t in (0.1, 1, 10, 100) if t < hi * 0.6] + [hi]
    labels = ['≤0.1' if t == FLOOR else f'{t:g}' for t in ticks]
    if axis == 'x':
        ax.set_xscale('log')
        ax.xaxis.set_major_locator(FixedLocator(ticks))
        ax.xaxis.set_minor_locator(NullLocator())
        ax.set_xticklabels(labels)
    else:
        ax.set_yscale('log')
        ax.yaxis.set_major_locator(FixedLocator(ticks))
        ax.yaxis.set_minor_locator(NullLocator())
        ax.set_yticklabels(labels)


def colours(systems, i):
    out = {'tddlog': TDDLOG[i]}
    for k, s in enumerate(s for s in systems if s != 'tddlog'):
        out[s] = PALETTE[k % len(PALETTE)][i]
    return out


def cactus(group, rs, names, hi, out, theme):
    c = style(theme)
    col = colours(group['systems'], c['i'])
    n = len(rs)
    fig, ax = plt.subplots(figsize=(6.4, 4.2))
    solved = {}
    for s in group['systems']:
        ts = sorted(clamp(v[s]) for _, _, v in rs if v.get(s) is not None)
        solved[s] = len(ts)
        xs = [FLOOR] + ts + [hi]
        ys = [sum(t <= FLOOR for t in ts)] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        ax.step(xs, ys, where='post', color=col[s], lw=2.8 if lead else 1.5, zorder=10 if lead else 5,
                label=f'{names[s]}  {len(ts)}')
    ax.axhline(n, color=c['muted'], lw=0.8, ls=(0, (2, 3)), zorder=1)
    ax.text(FLOOR * 1.08, n, f'all {n}', va='bottom', ha='left', color=c['muted'], fontsize=9)
    seconds_axis(ax, hi)
    ax.set_xlim(FLOOR, hi)
    ax.set_ylim(0, n * 1.06)
    ax.grid(True, color=c['grid'], lw=0.6)
    ax.set_axisbelow(True)
    ax.set_xlabel('time limit (s)')
    ax.set_ylabel('instances answered')
    ax.set_title(group['title'], loc='left')
    handles, labels = ax.get_legend_handles_labels()
    order = sorted(range(len(labels)), key=lambda k: -solved[group['systems'][k]])
    ax.legend([handles[k] for k in order], [labels[k] for k in order], loc='upper left',
              bbox_to_anchor=(1.01, 1.0), fontsize=9.5, handlelength=1.6, labelcolor=c['text'])
    save(fig, out, f'cactus-{group["key"]}{theme}')
    return solved


def scatter(group, rs, names, hi, out, theme):
    c = style(theme)
    col = colours(group['systems'], c['i'])
    fail = 10 ** (math.log10(hi) + 0.32)
    pts, both_fast, neither = [], 0, 0
    for _, _, v in rs:
        if 'tddlog' not in v or not any(s in v for s in group['systems'][1:]):
            continue
        t, b = v['tddlog'], best_other(v, group['systems'])
        if t is None and b is None:
            neither += 1
        elif t is not None and b is not None and max(t, b) < FLOOR:
            both_fast += 1
        else:
            pts.append((clamp(b) if b is not None else fail, clamp(t) if t is not None else fail))
    fig, ax = plt.subplots(figsize=(4.6, 4.4))
    faster = [p for p in pts if p[1] < p[0]]
    slower = [p for p in pts if p[1] > p[0]]
    tie = [p for p in pts if p[1] == p[0]]
    other = col[group['systems'][1]]
    for ps, colour in ((faster, col['tddlog']), (slower, other), (tie, c['muted'])):
        if ps:
            ax.scatter([p[0] for p in ps], [p[1] for p in ps], s=16, color=colour, alpha=0.75, lw=0, zorder=5)
    lim = (FLOOR / 1.15, fail * 1.3)
    edge = 10 ** (math.log10(hi) + 0.14)
    ax.axvspan(edge, lim[1], color=c['grid'], alpha=0.45, lw=0, zorder=1)
    ax.axhspan(edge, lim[1], color=c['grid'], alpha=0.45, lw=0, zorder=1)
    ax.plot(lim, lim, color=c['muted'], lw=1, zorder=2)
    for k in (10, 100):
        ax.plot(lim, [x / k for x in lim], color=c['muted'], lw=0.8, ls=(0, (3, 3)), zorder=2)
        ax.text(hi / 4, hi / 4 / k * 1.25, f'{k}×', color=c['muted'], fontsize=8.5, ha='center', va='bottom',
                rotation=45, rotation_mode='anchor')
    ticks = [t for t in (0.1, 1, 10, 100) if t < hi / 2] + [fail]
    labels = ['≤0.1' if t == FLOOR else f'{t:g}' for t in ticks[:-1]] + ['fail']
    for axis in ('x', 'y'):
        getattr(ax, f'set_{axis}scale')('log')
        a = getattr(ax, f'{axis}axis')
        a.set_major_locator(FixedLocator(ticks))
        a.set_minor_locator(NullLocator())
        getattr(ax, f'set_{axis}ticklabels')(labels)
    ax.set_xlim(*lim)
    ax.set_ylim(*lim)
    ax.set_aspect('equal')
    ax.grid(True, color=c['grid'], lw=0.6)
    ax.set_axisbelow(True)
    others = [names[s] for s in group['systems'][1:]]
    who = others[0] if len(others) == 1 else 'the fastest other system'
    ax.set_xlabel(f'{who} (s)')
    ax.set_ylabel('tddlog (s)')
    ax.set_title(f'tddlog faster on {len(faster)}', loc='left', color=col['tddlog'], fontsize=10.5)
    ax.set_title(f'slower on {len(slower)}', loc='right', color=other, fontsize=10.5)
    save(fig, out, f'scatter-{group["key"]}{theme}')
    return dict(points=len(pts), faster=len(faster), slower=len(slower), tie=len(tie), both_fast=both_fast,
                neither=neither)


def speedups(entries, out, theme):
    """entries: [(group title, [(name, stats)])], each name's bar its geometric mean."""
    c = style(theme)
    lines = []
    for title, items in entries:
        lines.append(('head', title))
        for name, st in sorted(items, key=lambda x: -x[1]['geomean']):
            lines.append(('bar', name, st))
    h = 0.30 * len(lines) + 0.5
    fig, ax = plt.subplots(figsize=(7.2, h))
    y = 0
    ys, labels = [], []
    gs = [rest[1]['geomean'] for kind, *rest in lines if kind == 'bar']
    hi, lo = max(gs), min(gs)
    for kind, *rest in lines:
        if kind == 'head':
            ax.text(-0.02, -y, rest[0], transform=ax.get_yaxis_transform(), ha='right', va='center',
                    fontweight='bold', fontsize=10.5)
        else:
            name, st = rest
            g = st['geomean']
            colour = TDDLOG[c['i']] if g >= 1 else PALETTE[0][c['i']]
            ax.barh(-y, max(g, 1) - min(g, 1), left=min(g, 1), height=0.62, color=colour, lw=0)
            label = f'{g:.0f}×' if g >= 10 else f'{g:.1f}×'
            if g < 1:
                label = f'{1 / g:.0f}× slower' if g <= 0.1 else f'{1 / g:.1f}× slower'
            ax.text(max(g, 1) * 1.08, -y, label, va='center', ha='left', fontsize=10, fontweight='bold',
                    color=colour)
            ax.text(1.0, -y, f'{st["both"]}', transform=ax.get_yaxis_transform(), va='center', ha='right',
                    fontsize=9, color=c['muted'])
            ys.append(-y)
            labels.append(name)
        y += 1
    ax.set_xscale('log')
    ax.set_xlim(min(0.5, lo / 2), hi * 12)
    ax.axvline(1, color=c['muted'], lw=1)
    ticks = [t for t in (0.01, 0.1, 1, 10, 100, 1000, 10000) if lo / 2 <= t <= hi * 12 or t == 1]
    ax.xaxis.set_major_locator(FixedLocator(ticks))
    ax.xaxis.set_minor_locator(NullLocator())
    ax.set_xticklabels([f'{t:g}×' for t in ticks])
    ax.set_yticks(ys)
    ax.set_yticklabels(labels)
    ax.tick_params(axis='y', length=0)
    ax.set_ylim(-y + 0.4, 0.6)
    ax.spines['left'].set_visible(False)
    ax.grid(True, axis='x', color=c['grid'], lw=0.6)
    ax.set_axisbelow(True)
    ax.text(1.0, 0, 'instances', transform=ax.get_yaxis_transform(), va='center', ha='right', fontsize=8.5,
            color=c['muted'])
    ax.set_xlabel("tddlog's speedup, geometric mean over the instances both answer")
    save(fig, out, f'speedups{theme}')


def main():
    args = [a for a in sys.argv[1:] if a != '--png']
    out = args[0] if args else os.path.join(ROOT, 'docs', 'images')
    os.makedirs(out, exist_ok=True)
    names, suites = load()
    numbers = {}
    for g in GROUPS:
        rs = rows(suites, g['suites'])
        hi = max(suites[k]['limit'] for k in g['suites'])
        res = dict(instances=len(rs), limit=hi)
        for theme in THEMES:
            res['solved'] = cactus(g, rs, names, hi, out, theme)
            if g['key'] != 'databases':
                res['scatter'] = scatter(g, rs, names, hi, out, theme)
        res['against'] = {s: compare(rs, 'tddlog', s) for s in g['systems'][1:]}
        best = [(k, i, dict(v, best=best_other(v, g['systems']))) for k, i, v in rs
                if any(s in v for s in g['systems'][1:])]
        res['against']['best'] = compare(best, 'tddlog', 'best')
        numbers[g['key']] = res
    # The overview: each comparison's baselines that share 5 instances or more with tddlog.
    overview = [(g['title'], [(names[s], numbers[g['key']]['against'][s]) for s in g['systems'][1:]
                                 if numbers[g['key']]['against'][s]['both'] >= 5]) for g in GROUPS]
    numbers['overview'] = {t: {n: st for n, st in items} for t, items in overview}
    for theme in THEMES:
        speedups(overview, out, theme)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
