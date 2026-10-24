"""readme_figures.py: the README's figures, drawn from the results page's
data (bindings/wasm/results/suites.json and ppl.json), and the numbers
its text quotes, printed as JSON.

    python3 scripts/readme_figures.py [--png] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at.

Each kind of problem is one set of instances, the programs written in the
languages of that kind, and the kind's systems are each given all of
them, through a translation where a system cannot read the original. The
data's `kinds` give each kind's suites and systems, and each system's
answers and total time, which the results page shows and the legends
here must match. An instance a system does not answer counts as
unanswered, whatever the reason: out of time or memory, a wrong answer,
a program the system cannot express, or one it was not given. So every
system of a kind is measured on the same instances.

Writes, as SVG:
  overview         for each kind, each system's total time against
                   tddlog's, and how many times less time tddlog takes;
  results-KIND     the cactus plot of the kind's systems over all its
                   instances (how many each answers within each time
                   limit) beside a scatter plot of tddlog's time against
                   the fastest other system's, instance by instance.

A total time is over the kind's instances that some system answers, the
same instances for every system of the kind: a run that does not answer
counts as its suite's time limit, a lower bound on the time it would
take, and a time under 0.2 s counts as 0.2 s. Beside it, the totals over
the instances both answer. The cactus plots' time axes are linear, from 0
to the time limit; the scatter plots' are logarithmic from 0.2 s, shorter
runs drawn at 0.2 s. Alea.jl's and Roulette's times are their in-process
times, without their language's start-up (the page's process times are
left out). Needs matplotlib."""
import collections
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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from source_tree import SOURCE  # noqa: E402  (the README's tree: its results and images)
FLOOR = 0.2  # seconds: a time under it counts as it, and a scatter plot's axes start here
PNG = '--png' in sys.argv
WIDTH = 9.4  # inches; 900 pixels in a browser

# tddlog's colour, then the baselines' in the order a figure lists them; past
# the palette's end the colours come round again with dashed lines.
TDDLOG = '#0d6670'
PALETTE = ['#d55e00', '#0072b2', '#e69f00', '#cc79a7', '#6a3d9a', '#8c564b', '#999933', '#56b4e9', '#882255',
           '#7f7f7f', '#222222', '#117733']
THEME = dict(bg='#ffffff', text='#1f2328', muted='#59636e', grid='#d8dee4')
FONTS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def load():
    """The systems' names, the suites by key and the kinds in the README's
    order."""
    names, suites, kinds = {}, {}, []
    for f in ('suites.json', 'ppl.json'):
        data = json.load(open(os.path.join(SOURCE, 'bindings', 'wasm', 'results', f)))
        names.update(data['names'])
        for fam in data['families']:
            for s in fam['suites']:
                suites[s['key']] = s
        kinds += data['kinds']
    return names, suites, sorted(kinds, key=lambda k: k['order'])


def rows(suites, keys):
    """[(suite, instance, {system: seconds, or None where it did not
    answer})], a system missing where the suite did not give it the
    program."""
    out = []
    for k in keys:
        s = suites[k]
        for r in s['instances']:
            out.append((k, r[0], {sys_: (v if isinstance(v, (int, float)) else None)
                                  for sys_, v in zip(s['systems'], r[1:]) if v is not None}))
    return out


def clamp(t):
    return max(t, FLOOR)


def scored(rs, systems):
    """The rows of rs that some system of systems answers, the instances a
    kind's total times are over."""
    return [r for r in rs if any(r[2].get(s) is not None for s in systems)]


def total(rs, s, limits):
    """s's total time over rs, a run that does not answer counted at its
    suite's time limit."""
    return sum(limits[k] if v.get(s) is None else clamp(v[s]) for k, _, v in rs)


def compare(rs, b, limits, a='tddlog'):
    """a (tddlog) against b over rs (scored rows): each one's answers and
    total time; `both`: the rows both answer, and their totals."""
    both = [r for r in rs if r[2].get(a) is not None and r[2].get(b) is not None]
    st = dict(instances=len(rs), answered_tddlog=sum(r[2].get(a) is not None for r in rs),
              answered_other=sum(r[2].get(b) is not None for r in rs),
              total_tddlog=total(rs, a, limits), total_other=total(rs, b, limits), both=len(both),
              both_tddlog=total(both, a, limits), both_other=total(both, b, limits),
              other_faster=sum(r[2].get(b) is not None and (r[2].get(a) is None or clamp(r[2][b]) < clamp(r[2][a]))
                               for r in rs))
    st['speedup'] = st['total_other'] / st['total_tddlog']
    if both:
        st['both_speedup'] = st['both_other'] / st['both_tddlog']
    return st


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


def linear_ticks(hi, room=5):
    """Ticks from 0 every 1, 2 or 5 times a power of ten, at most `room`
    of them, and one at hi."""
    unit = 10 ** math.floor(math.log10(hi / room))
    every = next(m * unit for m in (1, 2, 5, 10) if hi / (m * unit) <= room)
    return [k * every for k in range(int((hi - every / 2) // every) + 1)] + [hi]


def colours(systems):
    """{system: (colour, line style)}: tddlog's colour, then the palette's in
    the order of systems; past the palette's end its colours again, dashed."""
    out = {'tddlog': (TDDLOG, '-')}
    for k, s in enumerate(s for s in systems if s != 'tddlog'):
        out[s] = (PALETTE[k % len(PALETTE)], '-' if k < len(PALETTE) else (0, (4, 2)))
    return out


def answered(rs, s):
    return sum(r[2].get(s) is not None for r in rs)


def ordered(systems, rs):
    """tddlog first, then the others by the instances of rs they answer."""
    return ['tddlog'] + sorted((s for s in systems if s != 'tddlog'), key=lambda s: -answered(rs, s))


def cactus(ax, systems, title, rs, hi, c, col):
    """The instances of rs each system answers within each time limit, an
    instance a system does not answer (or was not given) counted as
    unanswered; returns {system: answered}."""
    n = len(rs)
    solved = {}
    for s in systems:
        ts = sorted(clamp(r[2][s]) for r in rs if r[2].get(s) is not None)
        solved[s] = len(ts)
        xs = [0] + ts + [hi]
        ys = [0] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        ax.step(xs, ys, where='post', color=col[s][0], ls=col[s][1], lw=3.2 if lead else 1.9,
                zorder=10 if lead else 5)
    ax.axhline(n, color=c['muted'], lw=0.9, ls=(0, (2, 3)), zorder=1)
    ax.text(hi * 0.01, n * 1.01, f'all {n}', va='bottom', ha='left', color=c['muted'], fontsize=11)
    ticks = linear_ticks(hi)
    ax.xaxis.set_major_locator(FixedLocator(ticks))
    ax.xaxis.set_minor_locator(NullLocator())
    ax.set_xticklabels([f'{t:g}' for t in ticks])
    ax.set_xlim(0, hi)
    ax.set_ylim(0, n * 1.09)
    ax.grid(True, color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    ax.set_xlabel('time limit (s)')
    ax.set_ylabel('instances answered')
    ax.set_title(title, loc='left', pad=10)
    return solved


# inches: a legend block's width, its header's height, a row's height, and
# the right edges of its columns of numbers
LEGEND_BLOCK, LEGEND_HEAD, LEGEND_ROW = 4.5, 0.3, 0.235
LEGEND_COLUMNS = [('answers', 2.85), ('total time', 4.1)]


def legend_table(fig, h, x, top, entries, n, noun, ncol, c, col):
    """A legend as a table, in ncol blocks side by side from (x, top), inches
    from the figure's top left: each system's line, its name, how many of the
    n instances it answers and its total time."""
    per = math.ceil(len(entries) / ncol)
    for b in range(ncol):
        x0 = x + b * LEGEND_BLOCK
        part = entries[b * per:(b + 1) * per]
        y = 1 - (top + 0.15) / h
        if b == 0:
            fig.text((x0 + 0.02) / WIDTH, y, f'of {n} {noun}', ha='left', va='center', fontsize=10.5, color=c['muted'])
        for head, right in LEGEND_COLUMNS:
            fig.text((x0 + right) / WIDTH, y, head, ha='right', va='center', fontsize=10, color=c['muted'])
        for k, (s, name, numbers) in enumerate(part):
            y = 1 - (top + LEGEND_HEAD + (k + 0.5) * LEGEND_ROW) / h
            lead = s == 'tddlog'
            fig.add_artist(plt.Line2D([(x0 + 0.02) / WIDTH, (x0 + 0.3) / WIDTH], [y, y], color=col[s][0],
                                      ls=col[s][1], lw=3.2 if lead else 1.9, transform=fig.transFigure))
            fig.text((x0 + 0.4) / WIDTH, y, name, ha='left', va='center', fontsize=11.5,
                     fontweight='bold' if lead else 'normal')
            for j, ((_, right), v) in enumerate(zip(LEGEND_COLUMNS, numbers)):
                fig.text((x0 + right) / WIDTH, y, str(v), ha='right', va='center', fontsize=11.5,
                         color=c['text'] if j == 0 else c['muted'])


def scatter(ax, rs, systems, hi, c, col, other):
    """tddlog's time against the fastest of systems', instance by instance;
    other: the colour of tddlog's losses."""
    fail = 10 ** (math.log10(hi) + 0.32)
    pts, both_fast, neither = [], 0, 0
    for _, _, v in rs:
        bs = [v[s] for s in systems if v.get(s) is not None]
        t, b = v.get('tddlog'), min(bs) if bs else None
        if t is None and b is None:
            neither += 1
        elif t is not None and b is not None and max(t, b) < FLOOR:
            both_fast += 1
        else:
            pts.append((clamp(b) if b is not None else fail, clamp(t) if t is not None else fail))
    faster = [p for p in pts if p[1] < p[0]]
    slower = [p for p in pts if p[1] > p[0]]
    tie = [p for p in pts if p[1] == p[0]]
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
    ax.set_xlabel('fastest other system (s)')
    ax.set_ylabel('tddlog (s)')
    ax.set_title(f'tddlog faster on {len(faster)}', loc='left', color=col['tddlog'][0], fontsize=11.5, pad=10)
    ax.set_title(f'slower on {len(slower)}', loc='right', color=other, fontsize=11.5, pad=10)
    return dict(points=len(pts), faster=len(faster), slower=len(slower), tie=len(tie), both_fast=both_fast,
                neither=neither)


def results(kind, rs, systems, names, limits, out):
    """The cactus plot of tddlog and systems over all of the kind's
    instances beside the scatter plot, the legend under them. Returns each
    system's answers and the scatter plot's numbers."""
    c = style()
    systems = ordered(['tddlog'] + systems, rs)
    col = colours(systems)
    hi = max(limits[k] for k, _, _ in rs)
    n = len(rs)
    ncol = 2
    legend_top = 4.9
    h = legend_top + LEGEND_HEAD + LEGEND_ROW * math.ceil(len(systems) / ncol) + 0.2
    fig = plt.figure(figsize=(WIDTH, h))

    def box(x, top, w, height):  # inches, from the figure's top left
        return [x / WIDTH, (h - top - height) / h, w / WIDTH, height / h]

    solved = cactus(fig.add_axes(box(0.8, 0.4, 4.35, 3.45)), systems, kind['title'], rs, hi, c, col)
    sc = scatter(fig.add_axes(box(5.85, 0.4, 3.45, 3.45)), rs, systems[1:], hi, c, col, col[systems[1]][0])
    over = scored(rs, systems)
    legend = [(s, names[s], [solved[s], seconds(total(over, s, limits))]) for s in systems]
    legend_table(fig, h, (WIDTH - ncol * LEGEND_BLOCK) / 2, legend_top, legend, n, kind['noun'], ncol, c, col)
    save(fig, out, f'results-{kind["key"]}')
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
             (360000, '100 h')]
    return [(t, s) for t, s in named if lo <= t <= hi]


def multiple(r):
    x = max(r, 1 / r)
    return (f'{x:.0f}×' if x >= 10 else f'{x:.1f}×') + ('' if r >= 1 else ' slower')


def overview(groups, out):
    """groups: [(kind's title, its instances, tddlog's answers, [(name, stats)])],
    stats compare()'s. Under each kind's title tddlog's answers and total
    time, then each system's total time and tddlog's, two points on one
    time axis; how many times less time tddlog takes, and beside it,
    smaller, the same over the instances both answer."""
    c = style()
    other = '#6e7781'
    lines = []
    for title, n, mine, items in groups:
        lines.append(('head', f'{title}, {n}', None))
        lines.append(('tddlog', 'tddlog', (mine, items[0][1]['total_tddlog'])))
        lines += [('row', name, st) for name, st in sorted(items, key=lambda x: -x[1]['speedup'])]
    ts = [t for kind, _, st in lines if kind == 'row' for t in (st['total_tddlog'], st['total_other'])]
    names_w, ratio_w, row, top, bottom = 2.3, 1.75, 0.34, 1.0, 0.95
    n = len(lines)
    h = n * row + top + bottom
    axes_w = WIDTH - names_w - ratio_w
    fig = plt.figure(figsize=(WIDTH, h))
    ax = fig.add_axes([names_w / WIDTH, bottom / h, axes_w / WIDTH, n * row / h])
    at = ax.get_yaxis_transform()
    bold_x, both_x = 1 + 0.25 / axes_w, 1 + 1.12 / axes_w
    ys, labels = [], []
    for y, (kind, name, st) in enumerate(lines):
        y = -y
        if kind == 'head':
            ax.text(-names_w / axes_w + 0.005, y, name, transform=at, ha='left', va='center', fontweight='bold',
                    fontsize=12.5)
            continue
        if kind == 'tddlog':
            mine, t = st
            ax.scatter([t], [y], s=64, color=TDDLOG, lw=0, zorder=4)
            ax.text(t / 1.3, y, seconds(t), ha='right', va='center', color=TDDLOG, fontsize=11.5)
            ax.text(-0.12 / axes_w, y, f'{mine}', transform=at, ha='right', va='center', fontsize=11,
                    color=TDDLOG, fontweight='bold')
            ys.append(y)
            labels.append(name)
            continue
        t, o = st['total_tddlog'], st['total_other']
        ax.plot([t, o], [y, y], color=c['grid'], lw=3, solid_capstyle='round', zorder=2)
        ax.scatter([o], [y], s=64, color=other, lw=0, zorder=3)
        ax.scatter([t], [y], s=64, color=TDDLOG, lw=0, zorder=4)
        (lo, lo_c), (hi, hi_c) = sorted([(t, TDDLOG), (o, other)])
        ax.text(lo / 1.3, y, seconds(lo), ha='right', va='center', color=lo_c, fontsize=11.5)
        ax.text(hi * 1.3, y, seconds(hi), ha='left', va='center', color=hi_c, fontsize=11.5)
        r = st['speedup']
        ax.text(bold_x, y, multiple(r), transform=at, ha='left', va='center', fontsize=12.5, fontweight='bold',
                color=TDDLOG if r >= 1 else other)
        if st['both']:
            ax.text(both_x, y, multiple(st['both_speedup']), transform=at, ha='left', va='center', fontsize=11,
                    color=c['muted'])
        ax.text(-0.12 / axes_w, y, f'{st["answered_other"]}', transform=at, ha='right', va='center', fontsize=11,
                color=c['muted'])
        ys.append(y)
        labels.append(name)
    ticks = times([min(ts) / 3, max(ts) * 3])
    log_axis(ax, 'x', [t for t, _ in ticks], [s for _, s in ticks])
    ax.set_xlim(min(ts) / 3, max(ts) * 3)
    ax.set_ylim(-n + 0.5, 0.5)
    ax.set_yticks(ys)
    ax.set_yticklabels(labels, fontsize=12)
    for label in ax.get_yticklabels():
        if label.get_text() == 'tddlog':
            label.set_color(TDDLOG)
            label.set_fontweight('bold')
    ax.tick_params(axis='y', length=0, pad=56)
    ax.tick_params(axis='x', length=0, pad=6)
    ax.grid(True, axis='x', color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    for side in ('left', 'bottom'):
        ax.spines[side].set_visible(False)
    head = (bottom + n * row + 0.72) / h
    fig.text(0.006, head, 'Total time, every system on the same instances', fontweight='bold', fontsize=13.5,
             va='center', ha='left')
    over = 1 + 0.12 / (n * row)
    ax.text(-0.12 / axes_w, over, 'answers', transform=ax.transAxes, ha='right', va='bottom', fontsize=11,
            color=c['muted'])
    ax.text(bold_x, over, 'tddlog\nfaster', transform=ax.transAxes, ha='left', va='bottom', fontsize=11,
            color=c['muted'], linespacing=1.1)
    ax.text(both_x, over, 'both\nanswer', transform=ax.transAxes, ha='left', va='bottom', fontsize=11,
            color=c['muted'], linespacing=1.1)
    marks = [plt.Line2D([], [], marker='o', ls='', ms=8, color=col) for col in (TDDLOG, other)]
    fig.legend(marks, ['tddlog', 'the other system'], loc='center right', bbox_to_anchor=(1, head),
               ncol=2, fontsize=12, handletextpad=0.2, columnspacing=1.2, labelcolor=c['text'])
    fig.text(0.006, 0.08 / h, textwrap.fill('Each kind’s instances are the programs written in its languages, '
                                           'and every system was given all of them. Total time over the instances '
                                           'some system answers: a run that does not answer (out of time or memory, '
                                           'failed, or a program the system cannot express) counts as its time limit '
                                           '(300 or 600 s), a time under 0.2 s as 0.2 s. Beside it, the same over '
                                           'the instances both answer.', 112),
             ha='left', va='bottom', fontsize=10.5, color=c['muted'], linespacing=1.35)
    save(fig, out, 'overview')


def misses(suites, keys, systems):
    """The instances of the suites keys that tddlog does not answer: (suite,
    name, tddlog's record, {system: seconds} for the systems of systems
    that answer it)."""
    out = []
    for k in keys:
        s = suites[k]
        for r in s['instances']:
            v = dict(zip(s['systems'], r[1:]))
            if not isinstance(v['tddlog'], (int, float)):
                out.append((k, r[0], v['tddlog'], {x: v[x] for x in systems if isinstance(v.get(x), (int, float))}))
    return out


def within(rs, systems, limits=(1, 2, 10)):
    """{limit: {system: the instances of rs it answers within limit seconds}}."""
    return {t: {s: sum(r[2].get(s) is not None and clamp(r[2][s]) <= t for r in rs) for s in systems}
            for t in limits}


def main():
    args = [a for a in sys.argv[1:] if a != '--png']
    out = args[0] if args else os.path.join(SOURCE, 'docs', 'images')
    os.makedirs(out, exist_ok=True)
    names, suites, kinds = load()
    limits = {k: s['limit'] for k, s in suites.items()}
    numbers, groups = {}, []
    for kind in kinds:
        key, systems = kind['key'], [s for s in kind['systems'] if s != 'tddlog']
        rs = rows(suites, kind['suites'])
        assert all('tddlog' in r[2] for r in rs), f'{key}: an instance tddlog was not given'
        over = scored(rs, ['tddlog'] + systems)
        res = dict(instances=len(rs), scored=len(over), limit=max(limits[k] for k in kind['suites']))
        res['answers'], res['scatter'] = results(kind, rs, systems, names, limits, out)
        # The legend's numbers are the results page's (the data's).
        assert res['answers'] == kind['answers'] and all(
            abs(total(over, s, limits) - kind['total'][s]) < 0.01 for s in kind['systems']), f'{key}: not the data'
        res['against'] = {s: compare(over, s, limits) for s in systems}
        res['tddlog_total'] = total(over, 'tddlog', limits)
        res['tddlog_misses'] = misses(suites, kind['suites'], systems)
        res['tddlog_failures'] = dict(collections.Counter(m[2] for m in res['tddlog_misses']))
        res['within'] = within(rs, ['tddlog'] + systems)
        numbers[key] = res
        groups.append((kind['title'], len(rs), res['answers']['tddlog'],
                       [(names[s], res['against'][s]) for s in systems]))
    overview(groups, out)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
