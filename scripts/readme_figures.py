"""readme_figures.py: the README's figures, drawn from the results page's
data (bindings/wasm/results/suites.json and ppl.json), and the numbers
its text quotes, printed as JSON.

    python3 scripts/readme_figures.py [--png] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at.

Writes, as SVG:
  overview         each system's total time against tddlog's;
  results-GROUP    a cactus plot (the instances each system answers within
                   a time limit) beside a scatter plot (tddlog's time
                   against the fastest other system's on each instance).

Each figure is drawn at the width it is shown at (900 pixels) on a white
background, which a dark page shows as a light card.

A comparison of tddlog with another system is over the instances the other
system is given and one of the two answers: each system's total time, a
run that does not answer counted at its suite's time limit (a lower bound
on the time it would take). The plots' time axes start at 0.2 s, shorter
runs drawn at 0.2 s. Alea.jl's and Roulette's times are their in-process
times, without their language's start-up. Needs matplotlib."""
import json
import math
import os
import re
import sys

import matplotlib

matplotlib.use('svg')
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.ticker import FixedLocator, NullLocator  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FLOOR = 0.2  # seconds: a plot's time axis starts here, shorter runs drawn at it
MIN_INSTANCES = 20  # the overview leaves out a system compared with tddlog on fewer instances
PNG = '--png' in sys.argv
WIDTH = 9.4  # inches; 900 pixels in a browser

# The comparisons: the suites of each and the systems drawn in it, tddlog first.
GROUPS = [
    dict(key='programs', title='Probabilistic logic programs', noun='programs',
         suites=['std', 'lit', 'biomine', 'tpch'], systems=['tddlog', 'problog', 'pita', 'aspmc']),
    dict(key='networks', title='Bayesian and Markov networks', noun='networks',
         suites=['std_bn', 'ace', 'large', 'uai', 'uai_pr'],
         systems=['tddlog', 'ace', 'merlin', 'agrum', 'pgmpy', 'libdai', 'tensorinference', 'problog', 'pita',
                  'aspmc']),
    dict(key='ppl', title='Probabilistic programs', noun='programs',
         suites=['dice', 'ppl', 'bitblast', 'roulette', 'geni'],
         systems=['tddlog', 'alea', 'dice', 'roulette', 'geni', 'nodice', 'psi', 'genfer', 'sppl', 'pluck',
                  'storm', 'prism', 'problog']),
    dict(key='databases', title='Probabilistic databases', noun='queries',
         suites=['tpch'], systems=['tddlog', 'provsql']),
]

# tddlog's colour, then the baselines' in the order a figure lists them.
TDDLOG = '#0d6670'
PALETTE = ['#d55e00', '#0072b2', '#e69f00', '#cc79a7', '#6a3d9a', '#8c564b', '#999933', '#56b4e9', '#882255',
           '#7f7f7f', '#222222', '#117733']
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


def compare(rs, a, b, limits):
    """a against b over the rows that give both systems the program and that
    one of them answers at least: each system's total time, a run that does
    not answer counted at its suite's time limit, and the two totals over the
    rows both answer."""
    st = dict(instances=0, answered_tddlog=0, answered_other=0, total_tddlog=0.0, total_other=0.0, both=0,
              both_tddlog=0.0, both_other=0.0)
    for k, _, v in rs:
        if a not in v or b not in v or (v[a] is None and v[b] is None):
            continue
        ta, tb = v[a], v[b]
        st['instances'] += 1
        st['answered_tddlog'] += ta is not None
        st['answered_other'] += tb is not None
        st['total_tddlog'] += limits[k] if ta is None else ta
        st['total_other'] += limits[k] if tb is None else tb
        if ta is not None and tb is not None:
            st['both'] += 1
            st['both_tddlog'] += ta
            st['both_other'] += tb
    if st['instances']:
        st['speedup'] = st['total_other'] / st['total_tddlog']
    return st


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


def cactus(ax, systems, title, rs, names, hi, c, col, small=False):
    """The instances of rs each system answers within each time limit, every
    system drawn tried on all of them; returns {system: answered}."""
    n = len(rs)
    solved = {}
    for s in systems:
        ts = sorted(clamp(v[s]) for _, _, v in rs if v[s] is not None)
        solved[s] = len(ts)
        xs = [FLOOR] + ts + [hi]
        ys = [sum(t <= FLOOR for t in ts)] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        ax.step(xs, ys, where='post', color=col[s], lw=(2.6 if small else 3.2) if lead else (1.6 if small else 1.9),
                zorder=10 if lead else 5)
    ax.axhline(n, color=c['muted'], lw=0.9, ls=(0, (2, 3)), zorder=1)
    if not small:
        ax.text(FLOOR * 1.1, n * 1.01, f'all {n}', va='bottom', ha='left', color=c['muted'], fontsize=11)
    ticks = [FLOOR] + [t for t in (1, 10, 100) if t < hi * 0.6] + [hi]
    log_axis(ax, 'x', ticks, [f'≤{t:g}' if t == FLOOR else f'{t:g}' for t in ticks])
    ax.set_xlim(FLOOR, hi)
    ax.set_ylim(0, n * 1.09)
    ax.grid(True, color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    if small:
        ax.tick_params(labelsize=10)
        ax.set_title(title, loc='left', fontsize=11.5, pad=6)
    else:
        ax.set_xlabel('time limit (s)')
        ax.set_ylabel('instances answered')
        ax.set_title(title, loc='left', pad=10)
    return solved


def tried(rs, s):
    return all(s in v for _, _, v in rs)


def subsets(group, rs):
    """A small cactus plot's worth for each system tried on part of rs (on
    MIN_INSTANCES or more): (the systems tried on exactly that part, its rows,
    the systems drawn, those tried on all of it), largest part first."""
    parts = {}
    for s in group['systems'][1:]:
        idx = tuple(i for i, (_, _, v) in enumerate(rs) if s in v)
        if MIN_INSTANCES <= len(idx) < len(rs):
            parts.setdefault(idx, []).append(s)
    out = []
    for idx, owners in sorted(parts.items(), key=lambda p: -len(p[0])):
        sub = [rs[i] for i in idx]
        out.append((owners, sub, [s for s in group['systems'] if tried(sub, s)]))
    return out


def joined(xs):
    return xs[0] if len(xs) == 1 else ', '.join(xs[:-1]) + ' and ' + xs[-1]


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
    ticks = [FLOOR] + [t for t in (1, 10, 100) if t < hi / 2] + [fail]
    labels = [f'≤{t:g}' if t == FLOOR else f'{t:g}' for t in ticks[:-1]] + ['fail']
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


def results(group, rs, names, limits, out):
    """The cactus plot of the systems tried on every instance beside the
    scatter plot; under them a small cactus plot for each system tried on
    part of the instances, over that part, with every system tried on all of
    it; the systems' legend under all."""
    c = style()
    col = colours(group['systems'])
    hi = max(limits[k] for k, _, _ in rs)
    full = [s for s in group['systems'] if tried(rs, s)]
    parts = subsets(group, rs)
    drawn = [s for s in group['systems'] if s in full or any(s in p[2] for p in parts)]
    n = len(drawn)
    ncol = math.ceil(n / math.ceil(n / 6))
    lrows = math.ceil(n / ncol)
    per_row, main_h, pitch = 3, 4.55, 2.6
    prows = math.ceil(len(parts) / per_row)
    h = main_h + prows * pitch + 0.32 * lrows + 0.3
    fig = plt.figure(figsize=(WIDTH, h))

    def box(x, top, w, height):  # inches, from the figure's top left
        return [x / WIDTH, (h - top - height) / h, w / WIDTH, height / h]

    solved = cactus(fig.add_axes(box(0.8, 0.4, 4.35, 3.45)), full, group['title'], rs, names, hi, c, col)
    sc = scatter(fig.add_axes(box(5.85, 0.4, 3.45, 3.45)), group, rs, names, hi, c, col)
    for k, (owners, sub, systems) in enumerate(parts):
        r, q = divmod(k, per_row)
        title = (f"{names[owners[0]]}'s" if len(owners) == 1 else joined([names[s] for s in owners]) + ':') + \
            f' {len(sub)} {group["noun"]}'
        part = cactus(fig.add_axes(box(0.55 + q * 3.05, main_h + r * pitch + 0.3, 2.55, 1.8)), systems, title, sub,
                      names, max(limits[k_] for k_, _, _ in sub), c, col, small=True)
        for s in owners:
            solved[s] = part[s]
    marks = [plt.Line2D([], [], color=col[s], lw=3.2 if s == 'tddlog' else 1.9) for s in drawn]
    entries = legend_order(list(zip(marks, [names[s] for s in drawn])), ncol)
    fig.legend([e[0] for e in entries], [e[1] for e in entries], loc='lower center', bbox_to_anchor=(0.5, 0.0),
               ncol=ncol, fontsize=12, handlelength=1.5, columnspacing=1.6, labelcolor=c['text'])
    save(fig, out, f'results-{group["key"]}')
    return solved, sc


def seconds(t):
    if t >= 36000:
        return f'{t / 3600:.0f} h'
    if t >= 3600:
        return f'{t / 3600:.1f} h'
    if t >= 60:
        return f'{t / 60:.0f} min'
    return f'{t:.0f} s' if t >= 10 else f'{t:.1f} s'


def times(ts):
    """A log axis's ticks, durations with their names, within the range of ts."""
    lo, hi = min(ts), max(ts)
    named = [(1, '1 s'), (10, '10 s'), (60, '1 min'), (600, '10 min'), (3600, '1 h'), (36000, '10 h'),
            ]
    return [(t, s) for t, s in named if lo <= t <= hi]


def overview(entries, out):
    """entries: [(group title, [(name, stats)])]. Each system's total time
    and tddlog's over the same instances, two points on one time axis, and
    how many times less time tddlog takes."""
    c = style()
    other = '#6e7781'
    lines = []
    for title, items in entries:
        lines.append(('head', title, None))
        lines += [('row', name, st) for name, st in sorted(items, key=lambda x: -x[1]['speedup'])]
    ts = [t for kind, _, st in lines if kind == 'row' for t in (st['total_tddlog'], st['total_other'])]
    names_w, ratio_w, row, top, bottom = 2.3, 1.0, 0.34, 0.8, 0.55
    n = len(lines)
    h = n * row + top + bottom
    axes_w = WIDTH - names_w - ratio_w
    fig = plt.figure(figsize=(WIDTH, h))
    ax = fig.add_axes([names_w / WIDTH, bottom / h, axes_w / WIDTH, n * row / h])
    at = ax.get_yaxis_transform()
    ys, labels = [], []
    for y, (kind, name, st) in enumerate(lines):
        y = -y
        if kind == 'head':
            ax.text(-names_w / axes_w + 0.005, y, name, transform=at, ha='left', va='center', fontweight='bold',
                    fontsize=12.5)
            continue
        t, o = st['total_tddlog'], st['total_other']
        ax.plot([t, o], [y, y], color=c['grid'], lw=3, solid_capstyle='round', zorder=2)
        ax.scatter([o], [y], s=64, color=other, lw=0, zorder=3)
        ax.scatter([t], [y], s=64, color=TDDLOG, lw=0, zorder=4)
        (lo, lo_c), (hi, hi_c) = sorted([(t, TDDLOG), (o, other)])
        ax.text(lo / 1.3, y, seconds(lo), ha='right', va='center', color=lo_c, fontsize=11.5)
        ax.text(hi * 1.3, y, seconds(hi), ha='left', va='center', color=hi_c, fontsize=11.5)
        r = st['speedup']
        label = (f'{r:.0f}×' if r >= 10 else f'{r:.1f}×') if r >= 1 else f'{1 / r:.1f}× slower'
        ax.text(1 + 0.25 / axes_w, y, label, transform=at, ha='left', va='center', fontsize=12.5,
                fontweight='bold', color=TDDLOG if r >= 1 else other)
        ax.text(-0.12 / axes_w, y, f'{st["instances"]}', transform=at, ha='right', va='center', fontsize=11,
                color=c['muted'])
        ys.append(y)
        labels.append(name)
    ticks = times([min(ts) / 3, max(ts) * 3])
    log_axis(ax, 'x', [t for t, _ in ticks], [s for _, s in ticks])
    ax.set_xlim(min(ts) / 3, max(ts) * 3)
    ax.set_ylim(-n + 0.5, 0.5)
    ax.set_yticks(ys)
    ax.set_yticklabels(labels, fontsize=12)
    ax.tick_params(axis='y', length=0, pad=44)
    ax.tick_params(axis='x', length=0, pad=6)
    ax.grid(True, axis='x', color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    for side in ('left', 'bottom'):
        ax.spines[side].set_visible(False)
    head = (bottom + n * row + 0.5) / h
    fig.text(0.006, head, 'Total time over each system’s benchmarks', fontweight='bold', fontsize=13.5,
             va='center', ha='left')
    ax.text(-0.12 / axes_w, 1 + 0.12 / (n * row), 'instances', transform=ax.transAxes, ha='right', va='bottom',
            fontsize=11, color=c['muted'])
    ax.text(1 + 0.25 / axes_w, 1 + 0.12 / (n * row), 'tddlog\nfaster', transform=ax.transAxes, ha='left',
            va='bottom', fontsize=11, color=c['muted'], linespacing=1.1)
    marks = [plt.Line2D([], [], marker='o', ls='', ms=8, color=col) for col in (TDDLOG, other)]
    fig.legend(marks, ['tddlog', 'the other system'], loc='center right', bbox_to_anchor=(1 - ratio_w / WIDTH, head),
               ncol=2, fontsize=12, handletextpad=0.2, columnspacing=1.2, labelcolor=c['text'])
    fig.text(0.006, 0.08 / h, 'Over the instances each system is given and either answers; a run that does not '
             'finish counts as its time limit (300 or 600 s).', ha='left', va='bottom', fontsize=11, color=c['muted'])
    save(fig, out, 'overview')


def main():
    args = [a for a in sys.argv[1:] if a != '--png']
    out = args[0] if args else os.path.join(ROOT, 'docs', 'images')
    os.makedirs(out, exist_ok=True)
    names, suites = load()
    limits = {k: s['limit'] for k, s in suites.items()}
    numbers = {}
    for g in GROUPS:
        rs = rows(suites, g['suites'])
        hi = max(limits[k] for k in g['suites'])
        res = dict(instances=len(rs), limit=hi)
        res['solved'], res['scatter'] = results(g, rs, names, limits, out)
        res['against'] = {s: compare(rs, 'tddlog', s, limits) for s in g['systems'][1:]}
        best = [(k, i, dict(v, best=best_other(v, g['systems']))) for k, i, v in rs
                if any(s in v for s in g['systems'][1:])]
        res['against']['best'] = compare(best, 'tddlog', 'best', limits)
        numbers[g['key']] = res
    # The overview: the systems compared with tddlog on enough instances for a total to mean something.
    entries = [(g['title'], [(names[s], numbers[g['key']]['against'][s]) for s in g['systems'][1:]
                             if numbers[g['key']]['against'][s]['instances'] >= MIN_INSTANCES]) for g in GROUPS]
    numbers['overview'] = {t: {n: st for n, st in items} for t, items in entries}
    overview(entries, out)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
