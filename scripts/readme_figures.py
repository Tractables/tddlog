"""readme_figures.py: the README's figures, drawn from the results page's
categories (bindings/wasm/results/categories.json), and the numbers its text quotes,
printed as JSON.

    python3 scripts/readme_figures.py [--png] [--data FILE] [OUT_DIR]     (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at;
--data reads another categories.json than the tree's.

The results are partitioned by type of program: each instance counts in
one category, whatever language its source was written in. Every system
given some instance of a category is measured on all of the category's
instances: an instance it does not answer counts as unanswered at the
instance's time limit, whatever the reason (out of time or memory, a
wrong answer, a program the system cannot express, a skip, or one it
was not given), and a time under 0.2 s counts as 0.2 s. The figures draw
the category's drawn systems (those given at least 20 of its instances),
and their legends' numbers are the data's, which the results page shows.

Writes, as SVG:
  overview          for each category, tddlog's answers and total time
                    and each family's virtual best (the fastest of the
                    family's drawn systems on each instance);
  results-CATEGORY  the cactus plot of the category's drawn systems over
                    all its instances (how many each answers within each
                    time limit), coloured by family with shades within it,
                    beside the scatter plot of tddlog's time against the
                    fastest other drawn system's, instance by instance; the
                    legend under them, ordered by answers.

The cactus plots' time axes are linear, from 0 to the category's longest
time limit; the scatter plots' are logarithmic from 0.2 s, shorter runs
drawn at 0.2 s. Alea.jl's and Roulette's times are their in-process
times, without their language's start-up. Needs matplotlib."""
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

# Each system's colour and line, the results page's (bindings/wasm/styles.css,
# its light steps): tddlog teal, and each system a shade of its family's hue
# (logic programming orange, graphical models blue, probabilistic
# programming violet and magenta, model checking green, databases ochre),
# the shades of a family told apart by their lines too. A family's colour
# is its first system's.
TDDLOG = '#0d6670'
DASH, DOT, DASHDOT = (0, (6, 3)), (0, (1.5, 2.5)), (0, (7, 3, 2, 3))
COLOURS = {
    'tddlog': (TDDLOG, '-'),
    'problog': ('#b35c00', '-'), 'pita': ('#7f3f00', DASH), 'aspmc': ('#c4720f', DOT), 'psouffle': ('#a0522d', DASHDOT),
    'ace': ('#1f5fa8', '-'), 'merlin': ('#3b7dd8', DASH), 'agrum': ('#173f73', DOT), 'pgmpy': ('#6186d2', '-'),
    'libdai': ('#2c4f9e', DASHDOT), 'tensorinference': ('#4f6fd0', DASH),
    'dice': ('#6a3fc0', DASH), 'alea': ('#a8327a', '-'), 'roulette': ('#8f5fd6', '-'), 'nodice': ('#4b2a8f', DOT),
    'psi': ('#c0508f', DASHDOT), 'genfer': ('#7b2c5c', DASH), 'sppl': ('#9b6fd0', DOT), 'geni': ('#d0569f', DOT),
    'pluck': ('#5c3a9e', DASHDOT),
    'storm': ('#1a7f37', '-'), 'prism': ('#3f9a50', DASH),
    'provsql': ('#8a6a12', '-'),
}
THEME = dict(bg='#ffffff', text='#1f2328', muted='#59636e', grid='#d8dee4')
FONTS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def load(path):
    """The data panels.py writes: names, families, categories (in the
    README's order) and own."""
    data = json.load(open(path))
    data['categories'].sort(key=lambda c: c['order'])
    return data


def colours(families):
    """{system: (colour, line style)}, and {family: colour}; a system of
    the data without a colour stops the figures."""
    systems = [s for f in families for s in f['systems'].split()]
    missing = [s for s in systems if s not in COLOURS]
    assert not missing, f'systems without a colour (COLOURS): {missing}'
    return COLOURS, {f['key']: COLOURS[f['systems'].split()[0]][0] for f in families}


def answered(v):
    return isinstance(v, (int, float))


def clamp(t):
    return max(t, FLOOR)


def cost(limit, v):
    return clamp(v) if answered(v) else limit


def column(cat, s):
    """[(limit, s's record)] over the category's rows."""
    j = 4 + cat['systems'].index(s)
    return [(r[2], r[j]) for r in cat['rows']]


def fastest(vs):
    ts = [v for v in vs if answered(v)]
    return min(ts) if ts else None


def total(pairs):
    return sum(cost(limit, v) for limit, v in pairs)


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


def ordered(cat):
    """The drawn systems by answers, then by total time."""
    return sorted(cat['drawn'], key=lambda s: (-cat['answers'][s], cat['total'][s]))


def cactus(ax, cat, systems, hi, c, col):
    """The instances each system answers within each time limit, an
    instance a system does not answer (or was not given) counted as
    unanswered; returns {system: answered}."""
    n = cat['instances']
    solved = {}
    for s in reversed(systems):
        ts = sorted(clamp(v) for _, v in column(cat, s) if answered(v))
        solved[s] = len(ts)
        xs = [0] + ts + [hi]
        ys = [0] + list(range(1, len(ts) + 1)) + [len(ts)]
        lead = s == 'tddlog'
        ax.step(xs, ys, where='post', color=col[s][0], ls=col[s][1], lw=3.2 if lead else 1.7,
                zorder=10 if lead else 5)
    ax.axhline(n, color=c['muted'], lw=0.9, zorder=1)
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
    ax.set_title(cat['title'], loc='left', pad=10)
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
        if not part:
            continue
        y = 1 - (top + 0.15) / h
        if b == 0:
            fig.text((x0 + 0.02) / WIDTH, y, f'of {n} {noun}', ha='left', va='center', fontsize=10.5, color=c['muted'])
        for head, right in LEGEND_COLUMNS:
            fig.text((x0 + right) / WIDTH, y, head, ha='right', va='center', fontsize=10, color=c['muted'])
        for k, (s, name, numbers) in enumerate(part):
            y = 1 - (top + LEGEND_HEAD + (k + 0.5) * LEGEND_ROW) / h
            lead = s == 'tddlog'
            fig.add_artist(plt.Line2D([(x0 + 0.02) / WIDTH, (x0 + 0.3) / WIDTH], [y, y], color=col[s][0],
                                      ls=col[s][1], lw=3.2 if lead else 1.7, transform=fig.transFigure))
            fig.text((x0 + 0.4) / WIDTH, y, name, ha='left', va='center', fontsize=11.5,
                     fontweight='bold' if lead else 'normal')
            for j, ((_, right), v) in enumerate(zip(LEGEND_COLUMNS, numbers)):
                fig.text((x0 + right) / WIDTH, y, str(v), ha='right', va='center', fontsize=11.5,
                         color=c['text'] if j == 0 else c['muted'])


def scatter(ax, cat, others, hi, c, col, other):
    """tddlog's time against the fastest of others', instance by instance;
    other: the colour of tddlog's losses."""
    fail = 10 ** (math.log10(hi) + 0.32)
    mine = [v for _, v in column(cat, 'tddlog')]
    best = [fastest(vs) for vs in zip(*[[v for _, v in column(cat, s)] for s in others])] if others else []
    pts, both_fast, neither = [], 0, 0
    for t, b in zip(mine, best):
        t = t if answered(t) else None
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
    ticks = [FLOOR] + [t for t in (1, 10, 100, 1000) if t < hi / 2] + [fail]
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


def results(cat, names, col, out):
    """The cactus plot of the category's drawn systems beside the scatter
    plot, the legend under them. Returns each system's answers and the
    scatter plot's numbers."""
    c = style()
    systems = ordered(cat)
    others = [s for s in systems if s != 'tddlog']
    hi = max(r[2] for r in cat['rows'])
    ncol = 2
    legend_top = 4.9
    h = legend_top + LEGEND_HEAD + LEGEND_ROW * math.ceil(len(systems) / ncol) + 0.2
    fig = plt.figure(figsize=(WIDTH, h))

    def box(x, top, w, height):  # inches, from the figure's top left
        return [x / WIDTH, (h - top - height) / h, w / WIDTH, height / h]

    solved = cactus(fig.add_axes(box(0.8, 0.4, 4.35, 3.45)), cat, systems, hi, c, col)
    loss = col[cat['best']][0] if cat['best'] in col else c['muted']
    sc = scatter(fig.add_axes(box(5.85, 0.4, 3.45, 3.45)), cat, others, hi, c, col, loss)
    legend = [(s, names[s], [solved[s], seconds(total(column(cat, s)))]) for s in systems]
    legend_table(fig, h, (WIDTH - ncol * LEGEND_BLOCK) / 2, legend_top, legend, cat['instances'], cat['noun'],
                 ncol, c, col)
    save(fig, out, f'results-{cat["key"]}')
    return solved, sc


def seconds(t):
    if t >= 36000:
        return f'{t / 3600:.0f} h'
    if t >= 3600:
        return f'{t / 3600:.1f} h'
    if t >= 60:
        return f'{t / 60:.0f} min'
    return f'{t:.0f} s' if t >= 10 else f'{t:.1f} s'


def times(lo, hi):
    """A log axis's ticks, durations with their names, from lo to hi."""
    named = [(1, '1 s'), (10, '10 s'), (60, '1 min'), (600, '10 min'), (3600, '1 h'), (36000, '10 h'),
             (360000, '100 h')]
    return [(t, s) for t, s in named if lo <= t <= hi]


def multiple(r):
    x = max(r, 1 / r)
    return (f'{x:.0f}×' if x >= 10 else f'{x:.1f}×') + ('' if r >= 1 else ' slower')


def overview(data, fam_colour, out):
    """For each category, under its title and number of instances, tddlog's
    total time and each family's virtual best's, points on one time axis,
    with their answers to the left and how many times less time tddlog
    takes to the right."""
    c = style()
    titles = {f['key']: f['title'] for f in data['families']}
    lines = []
    for cat in data['categories']:
        lines.append(('head', f"{cat['title']}, {cat['instances']} {cat['noun']}", None, None))
        mine = cat['total']['tddlog']
        lines.append(('tddlog', 'tddlog', (cat['answers']['tddlog'], mine), None))
        fams = sorted(cat['vfamily'].items(), key=lambda kv: kv[1]['total'])
        lines += [('row', titles[k], (v['answers'], v['total']), (k, mine)) for k, v in fams]
    ts = [st[1] for kind, _, st, _ in lines if kind != 'head']
    names_w, ratio_w, row, top, bottom = 2.6, 1.2, 0.34, 1.0, 0.95
    n = len(lines)
    h = n * row + top + bottom
    axes_w = WIDTH - names_w - ratio_w
    fig = plt.figure(figsize=(WIDTH, h))
    ax = fig.add_axes([names_w / WIDTH, bottom / h, axes_w / WIDTH, n * row / h])
    at = ax.get_yaxis_transform()
    ratio_x = 1 + 0.25 / axes_w
    ys, labels = [], []
    for y, (kind, name, st, fam) in enumerate(lines):
        y = -y
        if kind == 'head':
            ax.text(-names_w / axes_w + 0.005, y, name, transform=at, ha='left', va='center', fontweight='bold',
                    fontsize=12.5)
            continue
        a, t = st
        colour = TDDLOG if kind == 'tddlog' else fam_colour[fam[0]]
        if kind == 'row':
            mine = fam[1]
            ax.plot([mine, t], [y, y], color=c['grid'], lw=3, solid_capstyle='round', zorder=2)
            r = t / mine
            ax.text(ratio_x, y, multiple(r), transform=at, ha='left', va='center', fontsize=12.5,
                    fontweight='bold', color=TDDLOG if r >= 1 else colour)
        ax.scatter([t], [y], s=64, color=colour, lw=0, zorder=4 if kind == 'tddlog' else 3)
        ax.text(t * 1.3 if kind == 'row' and t >= fam[1] else t / 1.3, y, seconds(t),
                ha='left' if kind == 'row' and t >= fam[1] else 'right', va='center', color=c['text'], fontsize=11.5)
        ax.text(-0.12 / axes_w, y, f'{a}', transform=at, ha='right', va='center', fontsize=11, color=c['text'],
                fontweight='bold' if kind == 'tddlog' else 'normal')
        ys.append(y)
        labels.append(name)
    lo, hi = min(ts) / 3, max(ts) * 3
    ticks = times(lo, hi)
    log_axis(ax, 'x', [t for t, _ in ticks], [s for _, s in ticks])
    ax.set_xlim(lo, hi)
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
    fig.text(0.006, head, 'Total time over every instance of a category', fontweight='bold', fontsize=13.5,
             va='center', ha='left')
    over = 1 + 0.12 / (n * row)
    ax.text(-0.12 / axes_w, over, 'answers', transform=ax.transAxes, ha='right', va='bottom', fontsize=11,
            color=c['muted'])
    ax.text(ratio_x, over, 'tddlog\nfaster', transform=ax.transAxes, ha='left', va='bottom', fontsize=11,
            color=c['muted'], linespacing=1.1)
    fig.text(0.006, 0.08 / h, textwrap.fill(
        'Each instance counts in one category, whatever language its source was written in. A family’s line is '
        'its virtual best: the fastest of its systems on each instance. Total time over all of a category’s '
        'instances: a run that does not answer (out of time or memory, failed, a program the system cannot '
        'express, or not given) counts as its time limit, a time under 0.2 s as 0.2 s.', 112),
        ha='left', va='bottom', fontsize=10.5, color=c['muted'], linespacing=1.35)
    save(fig, out, 'overview')


def misses(cat):
    """tddlog's records on the instances it does not answer, by kind."""
    return dict(collections.Counter(v for _, v in column(cat, 'tddlog') if not answered(v)))


def within(cat, systems, limits=(1, 2, 10)):
    """{limit: {system: the instances it answers within limit seconds}}."""
    return {t: {s: sum(answered(v) and clamp(v) <= t for _, v in column(cat, s)) for s in systems} for t in limits}


def main():
    args = [a for a in sys.argv[1:] if a != '--png']
    path = os.path.join(SOURCE, 'bindings', 'wasm', 'results', 'categories.json')
    if args[:1] == ['--data']:
        path, args = args[1], args[2:]
    out = args[0] if args else os.path.join(SOURCE, 'docs', 'images')
    os.makedirs(out, exist_ok=True)
    data = load(path)
    names, (col, fam_colour) = data["names"], colours(data["families"])
    numbers = {}
    for cat in data['categories']:
        assert cat['drawn'][0] == 'tddlog', f"{cat['key']}: tddlog not drawn"
        res = dict(instances=cat['instances'], limit=max(r[2] for r in cat['rows']))
        res['answers'], res['scatter'] = results(cat, names, col, out)
        # The legend's numbers are the results page's (the data's).
        assert all(res['answers'][s] == cat['answers'][s] and abs(total(column(cat, s)) - cat['total'][s]) < 0.01
                   for s in cat['drawn']), f"{cat['key']}: not the data"
        res['total'] = {s: cat['total'][s] for s in cat['drawn']}
        res['best'] = dict(system=cat['best'], answers=cat['answers'].get(cat['best']),
                           total=cat['total'].get(cat['best']))
        res['vbest'] = cat['vbest']
        res['vfamily'] = cat['vfamily']
        res['not_drawn'] = {s: cat['given'][s] for s in cat['systems'] if s not in cat['drawn']}
        res['tddlog_failures'] = misses(cat)
        res['within'] = within(cat, cat['drawn'])
        numbers[cat['key']] = res
    overview(data, fam_colour, out)
    json.dump(numbers, sys.stdout, indent=1)
    print()


if __name__ == '__main__':
    main()
