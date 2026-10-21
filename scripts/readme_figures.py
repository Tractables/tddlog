"""readme_figures.py: the README's figures, drawn from the results page's
data (bindings/wasm/results/suites.json and ppl.json), and the numbers
its text quotes, printed as JSON.

    python3 scripts/readme_figures.py [--png] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at.

Writes, as SVG:
  overview         each system's total time against tddlog's;
  cactus-all       a cactus plot (the instances each system answers within a
                   time limit) of every system over every problem;
  results-KIND     the cactus plot of every system over the kind's problems
                   beside a scatter plot (tddlog's time against the fastest
                   other system's on each instance), and under them the
                   kind's panels: a small cactus plot over the instances a
                   cluster of systems with similar expressive capabilities
                   all express, with every system that expresses all of
                   them.

The kinds and their panels are the data's (`kinds`, written by the results
pipeline, release/results/rel/panels.py): an instance a system cannot
express or was not given counts as unanswered, and each legend gives how
many instances each system answers and expresses.

Each figure is drawn at the width it is shown at (900 pixels) on a white
background, which a dark page shows as a light card.

A comparison of tddlog with another system is over the instances the other
system is given and one of the two answers: each system's total time, a
run that does not answer counted at its suite's time limit (a lower bound
on the time it would take). The plots' time axes start at 0.2 s, shorter
runs drawn at 0.2 s. Alea.jl's and Roulette's times are their in-process
times, without their language's start-up (the page's dashed process lines
are left out). Needs matplotlib."""
import json
import math
import os
import re
import sys
import textwrap

import matplotlib

matplotlib.use('svg')
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.ticker import FixedLocator, NullLocator  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FLOOR = 0.2  # seconds: a plot's time axis starts here, shorter runs drawn at it
MIN_INSTANCES = 20  # the overview leaves out a system compared with tddlog on fewer instances
PNG = '--png' in sys.argv
WIDTH = 9.4  # inches; 900 pixels in a browser

# The kinds drawn, in the README's order, each with the systems its scatter
# plot and the overview compare with tddlog.
COMPARED = {
    'programs': ['problog', 'pita', 'aspmc'],
    'networks': ['ace', 'merlin', 'agrum', 'pgmpy', 'libdai', 'tensorinference', 'problog', 'pita', 'aspmc'],
    'ppl': ['alea', 'dice', 'roulette', 'geni', 'nodice', 'psi', 'genfer', 'sppl', 'pluck', 'storm', 'prism',
            'problog'],
    'databases': ['provsql'],
}

# tddlog's colour, then the baselines' in the order a figure lists them; past
# the palette's end the colours come round again with dashed lines.
TDDLOG = '#0d6670'
PALETTE = ['#d55e00', '#0072b2', '#e69f00', '#cc79a7', '#6a3d9a', '#8c564b', '#999933', '#56b4e9', '#882255',
           '#7f7f7f', '#222222', '#117733']
THEME = dict(bg='#ffffff', text='#1f2328', muted='#59636e', grid='#d8dee4')
FONTS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def load():
    names, suites, kinds = {}, {}, {}
    for f in ('suites.json', 'ppl.json'):
        data = json.load(open(os.path.join(ROOT, 'bindings', 'wasm', 'results', f)))
        names.update(data['names'])
        for fam in data['families']:
            for s in fam['suites']:
                suites[s['key']] = s
        for k in data['kinds']:
            kinds[k['key']] = k
    return names, suites, kinds


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


def drawn(systems):
    """The systems a figure draws: the page's process times left out."""
    return [s for s in systems if not s.endswith('-process')]
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
    """{system: (colour, line style)}: tddlog's colour, then the palette's in
    the order of systems; past the palette's end its colours again, dashed."""
    out = {'tddlog': (TDDLOG, '-')}
    for k, s in enumerate(s for s in systems if s != 'tddlog'):
        out[s] = (PALETTE[k % len(PALETTE)], '-' if k < len(PALETTE) else (0, (4, 2)))
    return out


def answered(rs, s):
    return sum(v.get(s) is not None for _, _, v in rs)


def ordered(systems, rs):
    """tddlog first, then the others by the instances of rs they answer."""
    return ['tddlog'] + sorted((s for s in systems if s != 'tddlog'), key=lambda s: -answered(rs, s))


def cactus(ax, systems, title, rs, hi, c, col, small=False):
    """The instances of rs each system answers within each time limit, an
    instance a system does not answer (or cannot express, or was not given)
    counted as unanswered; returns {system: answered}."""
    n = len(rs)
    solved = {}
    for s in systems:
        ts = sorted(clamp(v[s]) for _, _, v in rs if v.get(s) is not None)
        solved[s] = len(ts)
        xs = [FLOOR] + ts + [hi]
        ys = [sum(t <= FLOOR for t in ts)] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        ax.step(xs, ys, where='post', color=col[s][0], ls=col[s][1],
                lw=(2.6 if small else 3.2) if lead else (1.6 if small else 1.9), zorder=10 if lead else 5)
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
        ax.set_title(textwrap.fill(title, 28), loc='left', fontsize=11.5, pad=6, linespacing=1.15)
    else:
        ax.set_xlabel('time limit (s)')
        ax.set_ylabel('instances answered')
        ax.set_title(title, loc='left', pad=10)
    return solved


LEGEND_BLOCK, LEGEND_HEAD, LEGEND_ROW = 3.85, 0.3, 0.235  # inches: a block's width, the header's height, a row's


def legend_table(fig, h, x, top, entries, n, noun, ncol, c, col):
    """A legend as a table, in ncol blocks side by side from (x, top), inches
    from the figure's top left: each system's line, its name, how many of the
    n instances it answers and how many it expresses."""
    per = math.ceil(len(entries) / ncol)
    for b in range(ncol):
        x0 = x + b * LEGEND_BLOCK
        part = entries[b * per:(b + 1) * per]
        y = 1 - (top + 0.15) / h
        if b == 0:
            fig.text((x0 + 0.02) / WIDTH, y, f'of {n} {noun}', ha='left', va='center', fontsize=10.5, color=c['muted'])
        fig.text((x0 + 2.65) / WIDTH, y, 'answers', ha='right', va='center', fontsize=10.5, color=c['muted'])
        fig.text((x0 + 3.5) / WIDTH, y, 'expresses', ha='right', va='center', fontsize=10.5, color=c['muted'])
        for k, (s, name, a, m) in enumerate(part):
            y = 1 - (top + LEGEND_HEAD + (k + 0.5) * LEGEND_ROW) / h
            lead = s == 'tddlog'
            fig.add_artist(plt.Line2D([(x0 + 0.02) / WIDTH, (x0 + 0.3) / WIDTH], [y, y], color=col[s][0],
                                      ls=col[s][1], lw=3.2 if lead else 1.9, transform=fig.transFigure))
            fig.text((x0 + 0.4) / WIDTH, y, name, ha='left', va='center', fontsize=11.5,
                     fontweight='bold' if lead else 'normal')
            fig.text((x0 + 2.65) / WIDTH, y, str(a), ha='right', va='center', fontsize=11.5)
            fig.text((x0 + 3.5) / WIDTH, y, str(m), ha='right', va='center', fontsize=11.5, color=c['muted'])


def panel_rows(rs, panel):
    keys = {(suite, name) for suite, names in panel['instances'].items() for name in names}
    sub = [r for r in rs if (r[0], r[1]) in keys]
    assert len(sub) == len(keys), f"{panel['title']}: {len(keys) - len(sub)} instances not in the kind's suites"
    return sub


def scatter(ax, systems, rs, names, hi, c, col):
    """tddlog's time against the fastest other system's, instance by instance."""
    fail = 10 ** (math.log10(hi) + 0.32)
    pts, both_fast, neither = [], 0, 0
    for _, _, v in rs:
        if 'tddlog' not in v or not any(s in v for s in systems[1:]):
            continue
        t, b = v['tddlog'], best_other(v, systems)
        if t is None and b is None:
            neither += 1
        elif t is not None and b is not None and max(t, b) < FLOOR:
            both_fast += 1
        else:
            pts.append((clamp(b) if b is not None else fail, clamp(t) if t is not None else fail))
    faster = [p for p in pts if p[1] < p[0]]
    slower = [p for p in pts if p[1] > p[0]]
    tie = [p for p in pts if p[1] == p[0]]
    other = col[systems[1]][0]
    for ps, colour in ((faster, col['tddlog'][0]), (slower, other), (tie, c['muted'])):
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
    others = [names[s] for s in systems[1:]]
    ax.set_xlabel(f'{others[0] if len(others) == 1 else "fastest other system"} (s)')
    ax.set_ylabel('tddlog (s)')
    ax.set_title(f'tddlog faster on {len(faster)}', loc='left', color=col['tddlog'][0], fontsize=11.5, pad=10)
    ax.set_title(f'slower on {len(slower)}', loc='right', color=other, fontsize=11.5, pad=10)
    return dict(points=len(pts), faster=len(faster), slower=len(slower), tie=len(tie), both_fast=both_fast,
                neither=neither)


def results(kind, rs, names, limits, out):
    """The cactus plot of every system over all of the kind's instances
    beside the scatter plot; under them the kind's panels, each a small
    cactus plot over its instances with its systems; the legend under all.
    Returns each system's answers, the panels' and the scatter plot's
    numbers."""
    c = style()
    systems = ordered(drawn(kind['systems']), rs)
    col = colours(systems)
    compared = ['tddlog'] + [s for s in COMPARED[kind['key']] if s in systems]
    hi = max(limits[k] for k, _, _ in rs)
    n = len(rs)
    ncol = 2
    per_row, main_h, pitch = 3, 4.55, 3.0
    prows = math.ceil(len(kind['panels']) / per_row)
    legend_top = main_h + prows * pitch + 0.15
    h = legend_top + LEGEND_HEAD + LEGEND_ROW * math.ceil(len(systems) / ncol) + 0.2
    fig = plt.figure(figsize=(WIDTH, h))

    def box(x, top, w, height):  # inches, from the figure's top left
        return [x / WIDTH, (h - top - height) / h, w / WIDTH, height / h]

    solved = cactus(fig.add_axes(box(0.8, 0.4, 4.35, 3.45)), systems, kind['title'], rs, hi, c, col)
    sc = scatter(fig.add_axes(box(5.85, 0.4, 3.45, 3.45)), compared, rs, names, hi, c, col)
    panels = []
    for k, p in enumerate(kind['panels']):
        r, q = divmod(k, per_row)
        sub = panel_rows(rs, p)
        part = cactus(fig.add_axes(box(0.55 + q * 3.05, main_h + r * pitch + 0.75, 2.55, 1.8)), drawn(p['systems']),
                      p['title'], sub, max(limits[k_] for k_, _, _ in sub), c, col, small=True)
        panels.append(dict(title=p['title'], instances=len(sub), answers=part))
    entries = [(s, names[s], solved[s], kind['expresses'][s]) for s in systems]
    legend_table(fig, h, (WIDTH - ncol * LEGEND_BLOCK) / 2 + 0.2, legend_top, entries, n, kind['noun'], ncol, c, col)
    save(fig, out, f'results-{kind["key"]}')
    return solved, panels, sc


def headline(kind, rs, names, limits, out):
    """The cactus plot of every system over every problem, the legend beside
    it; returns each system's answers."""
    c = style()
    systems = ordered(drawn(kind['systems']), rs)
    col = colours(systems)
    hi = max(limits[k] for k, _, _ in rs)
    h = max(5.2, LEGEND_HEAD + LEGEND_ROW * len(systems) + 0.55)
    fig = plt.figure(figsize=(WIDTH, h))
    ax = fig.add_axes([0.8 / WIDTH, 0.75 / h, 4.6 / WIDTH, (h - 1.25) / h])
    solved = cactus(ax, systems, f'{kind["title"]}: {len(rs)} instances of every kind', rs, hi, c, col)
    entries = [(s, names[s], solved[s], kind['expresses'][s]) for s in systems]
    legend_table(fig, h, WIDTH - 3.6, 0.4, entries, len(rs), kind['noun'], 1, c, col)
    save(fig, out, 'cactus-all')
    return solved


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
    names, suites, kinds = load()
    limits = {k: s['limit'] for k, s in suites.items()}
    numbers = {}
    for key in ['all'] + list(COMPARED):
        kind = kinds[key]
        rs = rows(suites, kind['suites'])
        assert len(rs) == kind['instances'], f'{key}: {len(rs)} instances, the data says {kind["instances"]}'
        res = dict(instances=len(rs), limit=max(limits[k] for k in kind['suites']), expresses=kind['expresses'])
        if key == 'all':
            res['answers'] = headline(kind, rs, names, limits, out)
            numbers[key] = res
            continue
        res['answers'], res['panels'], res['scatter'] = results(kind, rs, names, limits, out)
        compared = ['tddlog'] + [s for s in COMPARED[key] if s in kind['systems']]
        res['against'] = {s: compare(rs, 'tddlog', s, limits) for s in compared[1:]}
        best = [(k, i, dict(v, best=best_other(v, compared))) for k, i, v in rs
                if any(s in v for s in compared[1:])]
        res['against']['best'] = compare(best, 'tddlog', 'best', limits)
        numbers[key] = res
    # The overview: the systems compared with tddlog on enough instances for a total to mean something.
    entries = [(kinds[k]['title'], [(names[s], numbers[k]['against'][s]) for s in numbers[k]['against']
                                    if s != 'best' and numbers[k]['against'][s]['instances'] >= MIN_INSTANCES])
               for k in COMPARED]
    numbers['overview'] = {t: {n: st for n, st in items} for t, items in entries}
    overview(entries, out)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
