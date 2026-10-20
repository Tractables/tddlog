"""readme_figures.py: the README's figures, drawn from the results page's
data (bindings/wasm/results/suites.json and ppl.json), and the numbers
its text quotes, printed as JSON.

    python3 scripts/readme_figures.py [--png] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at.

Writes, as SVG:
  speedups         tddlog's average speedup over each baseline;
  results-GROUP    a cactus plot (the instances each system answers within
                   a time limit) beside a scatter plot (tddlog's time
                   against the fastest other system's on each instance).

Each figure is drawn at the width it is shown at (900 pixels) on a white
background, which a dark page shows as a light card.

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
WIDTH = 9.4  # inches; 900 pixels in a browser
OVERVIEW = 20  # the instances a baseline shares with tddlog to have a bar in the overview

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

# tddlog's colour, then the baselines' in the order a figure lists them.
TDDLOG = '#0d6670'
PALETTE = ['#d55e00', '#0072b2', '#e69f00', '#cc79a7', '#6a3d9a', '#8c564b', '#999933', '#56b4e9', '#882255',
           '#7f7f7f', '#332288', '#117733']
THEME = dict(bg='#ffffff', text='#1f2328', muted='#59636e', grid='#d8dee4')
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


def style():
    c = THEME
    plt.rcdefaults()
    plt.rcParams.update({
        'svg.fonttype': 'none', 'svg.hashsalt': 'tddlog', 'font.family': 'DejaVu Sans', 'font.size': 12,
        'text.color': c['text'], 'axes.labelcolor': c['text'], 'axes.edgecolor': c['muted'],
        'xtick.color': c['muted'], 'ytick.color': c['muted'], 'xtick.labelcolor': c['text'],
        'ytick.labelcolor': c['text'], 'xtick.labelsize': 11.5, 'ytick.labelsize': 11.5,
        'axes.labelsize': 12, 'axes.spines.top': False, 'axes.spines.right': False,
        'figure.facecolor': c['bg'], 'axes.facecolor': c['bg'], 'savefig.facecolor': c['bg'],
        'legend.frameon': False, 'axes.titleweight': 'bold', 'axes.titlesize': 13,
    })
    return c


def save(fig, out, name):
    path = os.path.join(out, name + '.svg')
    if PNG:
        fig.savefig(path[:-4] + '.png', format='png', dpi=96, bbox_inches='tight', pad_inches=0.15)
    fig.savefig(path, format='svg', bbox_inches='tight', pad_inches=0.15, metadata={'Date': None})
    plt.close(fig)
    text = open(path).read()
    # The SVG names the font it was measured with; a page draws it with the system's sans serif.
    text = re.sub(r"'?DejaVu Sans'?", FONTS, text)
    open(path, 'w').write(text)


def log_axis(ax, axis, ticks, labels):
    getattr(ax, f'set_{axis}scale')('log')
    a = getattr(ax, f'{axis}axis')
    a.set_major_locator(FixedLocator(ticks))
    a.set_minor_locator(NullLocator())
    getattr(ax, f'set_{axis}ticklabels')(labels)


def colours(systems):
    out = {'tddlog': TDDLOG}
    for k, s in enumerate(s for s in systems if s != 'tddlog'):
        out[s] = PALETTE[k % len(PALETTE)]
    return out


def cactus(ax, group, rs, names, hi, c, col):
    """The instances each system answers within each time limit; returns
    {system: answered} and the lines' legend entries, most answered first."""
    n = len(rs)
    solved, entries = {}, []
    for s in group['systems']:
        ts = sorted(clamp(v[s]) for _, _, v in rs if v.get(s) is not None)
        solved[s] = len(ts)
        xs = [FLOOR] + ts + [hi]
        ys = [sum(t <= FLOOR for t in ts)] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        line, = ax.step(xs, ys, where='post', color=col[s], lw=3.2 if lead else 1.9, zorder=10 if lead else 5)
        entries.append((line, f'{names[s]}  {len(ts)}', len(ts)))
    ax.axhline(n, color=c['muted'], lw=0.9, ls=(0, (2, 3)), zorder=1)
    ax.text(FLOOR * 1.1, n * 1.01, f'all {n}', va='bottom', ha='left', color=c['muted'], fontsize=11)
    ticks = [t for t in (0.1, 1, 10, 100) if t < hi * 0.6] + [hi]
    log_axis(ax, 'x', ticks, ['≤0.1' if t == FLOOR else f'{t:g}' for t in ticks])
    ax.set_xlim(FLOOR, hi)
    ax.set_ylim(0, n * 1.09)
    ax.grid(True, color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    ax.set_xlabel('time limit (s)')
    ax.set_ylabel('instances answered')
    ax.set_title(group['title'], loc='left', pad=10)
    entries.sort(key=lambda e: -e[2])
    return solved, entries


def scatter(ax, group, rs, names, hi, c, col):
    """tddlog's time against the fastest other system's, instance by instance."""
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
    faster = [p for p in pts if p[1] < p[0]]
    slower = [p for p in pts if p[1] > p[0]]
    tie = [p for p in pts if p[1] == p[0]]
    other = col[group['systems'][1]]
    for ps, colour in ((faster, col['tddlog']), (slower, other), (tie, c['muted'])):
        if ps:
            ax.scatter([p[0] for p in ps], [p[1] for p in ps], s=24, color=colour, alpha=0.75, lw=0, zorder=5)
    lim = (FLOOR / 1.15, fail * 1.3)
    edge = 10 ** (math.log10(hi) + 0.14)
    ax.axvspan(edge, lim[1], color=c['grid'], alpha=0.6, lw=0, zorder=1)
    ax.axhspan(edge, lim[1], color=c['grid'], alpha=0.6, lw=0, zorder=1)
    ax.plot(lim, lim, color=c['muted'], lw=1.1, zorder=2)
    for k in (10, 100):
        ax.plot(lim, [x / k for x in lim], color=c['muted'], lw=0.9, ls=(0, (3, 3)), zorder=2)
        ax.text(hi / 4, hi / 4 / k * 1.3, f'{k}×', color=c['muted'], fontsize=11, ha='center', va='bottom',
                rotation=45, rotation_mode='anchor')
    ticks = [t for t in (0.1, 1, 10, 100) if t < hi / 2] + [fail]
    labels = ['≤0.1' if t == FLOOR else f'{t:g}' for t in ticks[:-1]] + ['fail']
    log_axis(ax, 'x', ticks, labels)
    log_axis(ax, 'y', ticks, labels)
    ax.set_xlim(*lim)
    ax.set_ylim(*lim)
    ax.set_aspect('equal')
    ax.grid(True, color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    others = [names[s] for s in group['systems'][1:]]
    ax.set_xlabel(f'{others[0] if len(others) == 1 else "fastest other system"} (s)')
    ax.set_ylabel('tddlog (s)')
    ax.set_title(f'tddlog faster on {len(faster)}', loc='left', color=col['tddlog'], fontsize=11.5, pad=10)
    ax.set_title(f'slower on {len(slower)}', loc='right', color=other, fontsize=11.5, pad=10)
    return dict(points=len(pts), faster=len(faster), slower=len(slower), tie=len(tie), both_fast=both_fast,
                neither=neither)


def legend_order(entries, ncol):
    """entries in the order that fills a matplotlib legend of ncol columns
    (column by column) so that it reads row by row."""
    rows_, large = divmod(len(entries), ncol)
    per_col = [rows_ + 1] * large + [rows_] * (ncol - large)
    return [entries[r * ncol + k] for k in range(ncol) for r in range(per_col[k])]


def results(group, rs, names, hi, out):
    """The cactus plot beside the scatter plot, the systems' legend under both."""
    c = style()
    col = colours(group['systems'])
    n = len(group['systems'])
    ncol = math.ceil(n / math.ceil(n / 6))
    lrows = math.ceil(n / ncol)
    h = 4.3 + 0.32 * lrows
    fig = plt.figure(figsize=(WIDTH, h))
    grid = fig.add_gridspec(1, 2, width_ratios=[1.3, 1], wspace=0.3, left=0.08, right=0.98, top=0.93,
                            bottom=(0.75 + 0.32 * lrows) / h)
    solved, entries = cactus(fig.add_subplot(grid[0]), group, rs, names, hi, c, col)
    sc = scatter(fig.add_subplot(grid[1]), group, rs, names, hi, c, col)
    entries = legend_order(entries, ncol)
    fig.legend([e[0] for e in entries], [e[1] for e in entries], loc='lower center', bbox_to_anchor=(0.5, 0.0),
               ncol=ncol, fontsize=12, handlelength=1.5, columnspacing=1.6, labelcolor=c['text'])
    save(fig, out, f'results-{group["key"]}')
    return solved, sc


def speedups(entries, out):
    """entries: [(group title, [(name, stats)])], each name's bar its geometric
    mean, the first two groups in a left column and the others in a right one."""
    c = style()

    def lines(groups):
        out = []
        for title, items in groups:
            out.append(('head', title, None))
            out += [('bar', name, st) for name, st in sorted(items, key=lambda x: -x[1]['geomean'])]
        return out

    cols = [lines(entries[:2]), lines(entries[2:])]
    gs = [st['geomean'] for col in cols for kind, _, st in col if kind == 'bar']
    xlim = (min(min(gs), 1) / 2.5, max(gs) * 14)
    nrow = max(len(col) for col in cols)
    row, top, bottom = 0.36, 0.1, 0.6
    names_w, bars_w, count_w = 1.75, 2.35, 0.5
    h = nrow * row + top + bottom
    fig = plt.figure(figsize=(WIDTH, h))
    for k, col in enumerate(cols):
        ax = fig.add_axes([(k * WIDTH / 2 + names_w) / WIDTH, bottom / h, bars_w / WIDTH, nrow * row / h])
        ys, labels = [], []
        for y, (kind, name, st) in enumerate(col):
            if kind == 'head':
                ax.text(-names_w / bars_w, -y, name, transform=ax.get_yaxis_transform(), ha='left', va='center',
                        fontweight='bold', fontsize=12.5, zorder=4,
                        bbox=dict(boxstyle='square,pad=0.15', fc=c['bg'], ec='none'))
                ax.text(1 + count_w / bars_w, -y, 'instances', transform=ax.get_yaxis_transform(), va='center',
                        ha='right', fontsize=10.5, color=c['muted'])
                continue
            g = st['geomean']
            colour = TDDLOG if g >= 1 else PALETTE[0]
            ax.barh(-y, max(g, 1) - min(g, 1), left=min(g, 1), height=0.64, color=colour, lw=0, zorder=3)
            if g >= 1:
                label = f'{g:.0f}×' if g >= 10 else f'{g:.1f}×'
            else:
                label = f'{1 / g:.0f}× slower' if g <= 0.1 else f'{1 / g:.1f}× slower'
            ax.text(max(g, 1) * 1.1, -y, label, va='center', ha='left', fontsize=12, fontweight='bold',
                    color=colour, zorder=4)
            ax.text(1 + count_w / bars_w, -y, f'{st["both"]}', transform=ax.get_yaxis_transform(), va='center',
                    ha='right', fontsize=11, color=c['muted'])
            ys.append(-y)
            labels.append(name)
        # The bars carry their values: no axis but the line at 1×, where tddlog is as fast.
        ax.axvline(1, color=c['muted'], lw=1.1, zorder=2)
        ax.text(1, -nrow + 0.4, '1×', ha='center', va='top', fontsize=11, color=c['muted'])
        log_axis(ax, 'x', [], [])
        ax.set_xlim(*xlim)
        ax.set_yticks(ys)
        ax.set_yticklabels(labels, fontsize=12)
        ax.tick_params(axis='y', length=0)
        ax.set_ylim(-nrow + 0.4, 0.6)
        for side in ('left', 'bottom'):
            ax.spines[side].set_visible(False)
    fig.text(0.5, 0.1 / h, "tddlog's speedup: the geometric mean over the instances both systems answer",
             ha='center', va='bottom', fontsize=12, color=c['text'])
    save(fig, out, 'speedups')


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
        res['solved'], res['scatter'] = results(g, rs, names, hi, out)
        res['against'] = {s: compare(rs, 'tddlog', s) for s in g['systems'][1:]}
        best = [(k, i, dict(v, best=best_other(v, g['systems']))) for k, i, v in rs
                if any(s in v for s in g['systems'][1:])]
        res['against']['best'] = compare(best, 'tddlog', 'best')
        numbers[g['key']] = res
    # The overview: each comparison's baselines that share OVERVIEW instances or more with
    # tddlog; an average over fewer says little, and the README's text has those systems.
    overview = [(g['title'], [(names[s], numbers[g['key']]['against'][s]) for s in g['systems'][1:]
                                 if numbers[g['key']]['against'][s]['both'] >= OVERVIEW]) for g in GROUPS]
    numbers['overview'] = {t: {n: st for n, st in items} for t, items in overview}
    speedups(overview, out)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
