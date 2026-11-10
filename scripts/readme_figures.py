"""readme_figures.py: the README's figures, drawn from the results page's
categories (bindings/wasm/results/categories.json), and the numbers its text quotes,
printed as JSON.

    python3 scripts/readme_figures.py [--png] [--data FILE] [--readme README | --check README] [OUT_DIR]
                                                                           (default docs/images)

--png also writes each figure as a PNG at the SVG's own size, to look at;
--data reads another categories.json than the tree's; --readme rewrites
README's generated blocks from the same data, and --check only compares
them, each failing (exit 1) where a block differs from the data or a
block or its marker is missing. A generated block is the text between
`<!-- results:KEY -->` and `<!-- /results:KEY -->` (blocks): the headline
(tddlog's speed-up over the leading system of two types, HEADLINE), the
overview's caption, with the data's date, and the integrations' table. The
README links to the site's pages for each category's own figure and
table; the figures are written and the numbers printed all the same.
The overview's block names the SHA-256 of the categories.json it was
written from, which release/push_check.sh checks against the file pushed
beside it.

The results are partitioned by type of program: each instance counts in
one type, whatever language its source was written in, and all programs
are a sixth category, their union. A type is compared with the systems
native to it, all programs with every system. The circuits (the data's
`circuits`: every circuit Alea.jl exports, read by every system) are an
evaluation apart from the types and written as one. A system's answers count
over all of a category's instances, an instance it does not answer
(out of time or memory, a wrong answer, a program it cannot express, a
skip, or one it was not given) counting as unanswered. tddlog is compared
with a system by rule 16 (the data's vs, which panels.py computes): the
two total times over the instances the system is given and either
answers, a run that does not finish counted as twice its time limit
(PAR-2), and tddlog's speed-up, the ratio of their total times on the
instances both answer, together with the instances only one of them
answers, the other at its time limit, each only where that lower bound
raises the multiplier of the system that answered it: one only tddlog
answers where the limit over its time is at least the speed-up, one only
the other answers where the limit over its time is at least the
speed-up's inverse, so that swapping the two gives the inverse. A time under
0.2 s counts as 0.2 s, and a comparison with nothing to measure (no
instance both answer over 0.2 s and none only one answers) has no speed-up. The
figures draw a category's drawn systems (tddlog and the native systems
given at least 20 of its instances), and their legends' numbers are the
data's, which the results page shows.

tddlog above is a category's `mine` (rule 26): plain tddlog, where it read
the program with its own front ends, unanswered (unread) on a type's
instances that only another system's front end reads, and each pipeline
where another system's code ran first and handed it the inference
(Alea.jl+tddlog, tddlog on the circuit Alea.jl builds), a system of its
own. Each is drawn as a curve of its own in tddlog's colour (a pipeline
dashed or dotted), and the texts, the scatter plot and the comparisons
take each row's subject, plain tddlog in a type (the data's vs[x]['me']
names them).

Writes, as SVG:
  overview          for each type, tddlog's answers and, for each of its
                    five best native systems, the system's answers, its
                    total time and tddlog's on one time axis, and
                    tddlog's speed-up over it; the other native systems
                    in one row, the least of their totals and of their
                    speed-ups;
  results-CATEGORY  the cactus plot of the category's drawn systems over
                    all its instances (how many each answers within each
                    time limit), tddlog and the five best native systems
                    each in its colour and line, the others grey, beside
                    the scatter plot of tddlog's time against the fastest
                    other drawn system's, instance by instance; the legend
                    under them, ordered by answers, each system's answers
                    and tddlog's speed-up over it; results-circuits the
                    same of the circuits.

The JSON printed is {categories, integrations}: for each category and the
circuits (`circuits`), the numbers its text quotes and `table`, the lines
of the README's table of its native systems (all programs and the
circuits: every system); for each integration
(rule 26: a pipeline against the system whose code runs first, over the
instances the pipeline is given, the data's `integrations`), its
instances, answers and comparison, and `table`, the lines of the README's
table of them all, a row each. Each system on its own benchmarks
is the results page's table alone; the README leaves it out (the owner,
2026-10-30).

The cactus plots' time axes are linear, from 0 to the category's longest
time limit; the scatter plots' are logarithmic from 0.2 s, shorter runs
drawn at 0.2 s. Alea.jl's and Roulette's times are their in-process
times, without their language's start-up. Needs matplotlib."""
import collections
import decimal
import hashlib
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
# its light steps): tddlog teal, the logic programming systems orange, the
# network tools blue, the probabilistic programming languages violet and
# magenta, the model checkers green and the database ochre, systems of one
# hue told apart by their lines too.
TDDLOG = '#0d6670'
DASH, DOT, DASHDOT = (0, (6, 3)), (0, (1.5, 2.5)), (0, (7, 3, 2, 3))
COLOURS = {
    'tddlog': (TDDLOG, '-'), 'alea-tddlog': (TDDLOG, DASH), 'roulette-tddlog': (TDDLOG, DOT),
    'problog': ('#b35c00', '-'), 'pita': ('#7f3f00', DASH), 'aspmc': ('#c4720f', DOT), 'psouffle': ('#a0522d', DASHDOT),
    'scallop': ('#d9480f', DASH),
    'ace': ('#1f5fa8', '-'), 'merlin': ('#3b7dd8', DASH), 'agrum': ('#173f73', DOT), 'pgmpy': ('#6186d2', '-'),
    'libdai': ('#2c4f9e', DASHDOT), 'tensorinference': ('#4f6fd0', DASH),
    'dice': ('#6a3fc0', DASH), 'alea': ('#a8327a', '-'), 'roulette': ('#8f5fd6', '-'), 'nodice': ('#4b2a8f', DOT),
    'psi': ('#c0508f', DASHDOT), 'genfer': ('#7b2c5c', DASH), 'sppl': ('#9b6fd0', DOT), 'geni': ('#d0569f', DOT),
    'pluck': ('#5c3a9e', DASHDOT), 'webppl': ('#9a3fb5', DASHDOT),
    'storm': ('#1a7f37', '-'), 'prism': ('#3f9a50', DASH),
    'provsql': ('#8a6a12', '-'),
}
ALL = 'all-programs'  # all programs, the union of the types: a category of its own, not on the overview
APART = 'circuits'    # the circuits, apart from the types: written as a category, not on the overview
THEME = dict(bg='#ffffff', text='#1f2328', muted='#59636e', grid='#d8dee4')
# a line past tddlog's and the type's five best native systems': the results page's
# --grey-line, its muted colour 45% over the background
GREY = '#b4b9be'
FONTS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


def load(path):
    """The data panels.py writes: names, categories (in the README's order),
    circuits and own."""
    data = json.load(open(path))
    data['categories'].sort(key=lambda c: c['order'])
    return data


def written(data):
    """The categories the README writes, in its order, then the circuits
    where the data have them."""
    return data['categories'] + ([data['circuits']] if data.get('circuits') else [])


def colours(categories):
    """{system: (colour, line style)}; a system of the data without a
    colour stops the figures."""
    missing = sorted({s for c in categories for s in c['systems']} - set(COLOURS))
    assert not missing, f'systems without a colour (COLOURS): {missing}'
    return dict(COLOURS)


def answered(v):
    return isinstance(v, (int, float))


def clamp(t):
    return max(t, FLOOR)


def column(cat, s):
    """[(limit, s's record)] over the category's rows."""
    j = 4 + cat['systems'].index(s)
    return [(r[2], r[j]) for r in cat['rows']]


def is_mine(s):
    """tddlog or one of its pipelines (rule 26: alea-tddlog)."""
    return s == 'tddlog' or s.endswith('-tddlog')


def my_keys(cat):
    """The row's subject, row by row: the first of the category's `mine`
    given the instance (panels.subject: tddlog in a type, unread where
    only another system's front end read the program; a pipeline is a
    curve of its own, not a subject)."""
    js = [(m, 4 + cat['systems'].index(m)) for m in cat['mine']]
    return [next((m for m, j in js if r[j] is not None), None) for r in cat['rows']]


def my_column(cat):
    """[(limit, the record of the row's subject)] over the category's rows."""
    return [(r[2], None if m is None else r[4 + cat['systems'].index(m)]) for r, m in zip(cat['rows'], my_keys(cat))]


def subject(cat, names, join='and'):
    """The names of the members of MINE that are the subject of some row of
    a category, listed in words: tddlog, Alea.jl+tddlog, tddlog and
    Alea.jl+tddlog, or tddlog, Alea.jl+tddlog and Roulette+tddlog."""
    keys = set(my_keys(cat))
    xs = [names[m] for m in cat['mine'] if m in keys]
    return xs[0] if len(xs) == 1 else f"{', '.join(xs[:-1])} {join} {xs[-1]}"


def my_answers(cat):
    """The rows whose subject answers."""
    return sum(answered(v) for _, v in my_column(cat))


def fastest(vs):
    ts = [v for v in vs if answered(v)]
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


def linear_ticks(hi, room=5):
    """Ticks from 0 every 1, 2 or 5 times a power of ten, at most `room`
    of them, and one at hi."""
    unit = 10 ** math.floor(math.log10(hi / room))
    every = next(m * unit for m in (1, 2, 5, 10) if hi / (m * unit) <= room)
    return [k * every for k in range(int((hi - every / 2) // every) + 1)] + [hi]


def ordered(cat):
    """The drawn systems by answers, then in the data's order (the native
    systems by answers, then speed-up)."""
    rank = cat['mine'] + cat['native']
    return sorted(cat['drawn'], key=lambda s: (-cat['answers'][s], rank.index(s)))


def lit(cat):
    """The systems whose lines are in colour: tddlog's (and its pipelines')
    and the five best native systems', the overview's rows; the others grey."""
    return set(cat['mine']) | set(cat['named'])


def line(s, cat, col):
    """A system's line: its colour, its dash and its width."""
    if s not in lit(cat):
        return GREY, '-', 1.2
    return col[s][0], col[s][1], 3.2 if is_mine(s) else 1.7


def cactus(ax, cat, systems, hi, c, col):
    """The instances each system answers within each time limit, an
    instance a system does not answer (or was not given) counted as
    unanswered, the lines past lit()'s grey and under the others; returns
    {system: answered}."""
    n = cat['instances']
    solved = {}
    for s in reversed(systems):
        ts = sorted(clamp(v) for _, v in column(cat, s) if answered(v))
        solved[s] = len(ts)
        xs = [0] + ts + [hi]
        ys = [0] + list(range(1, len(ts) + 1)) + [len(ts)]
        colour, ls, lw = line(s, cat, col)
        ax.step(xs, ys, where='post', color=colour, ls=ls, lw=lw,
                zorder=10 if is_mine(s) else 5 if s in lit(cat) else 3)
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
LEGEND_COLUMNS = [('answers', 2.8), ('tddlog faster', 4.35)]
NONE = 'nothing to measure'  # a comparison with no instance both answer over 0.2 s and none only one answers
ANSWERS_NONE = 'answers none'  # a system that answers no instance: its speed-up is the measure's ceiling, the limit over 0.2 s


def legend_table(fig, h, x, top, entries, n, noun, ncol, c, col, cat):
    """A legend as a table, in ncol blocks side by side from (x, top), inches
    from the figure's top left: each system's line (grey past lit()'s), its
    name, how many of the n instances it answers and tddlog's speed-up over
    it."""
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
            colour, ls, lw = line(s, cat, col)
            fig.add_artist(plt.Line2D([(x0 + 0.02) / WIDTH, (x0 + 0.3) / WIDTH], [y, y], color=colour,
                                      ls=ls, lw=lw, transform=fig.transFigure))
            fig.text((x0 + 0.4) / WIDTH, y, name, ha='left', va='center', fontsize=11.5,
                     fontweight='bold' if is_mine(s) else 'normal', color=c['text'] if s in lit(cat) else c['muted'])
            for j, ((_, right), v) in enumerate(zip(LEGEND_COLUMNS, numbers)):
                fig.text((x0 + right) / WIDTH, y, str(v), ha='right', va='center', fontsize=9.5 if v in (NONE, ANSWERS_NONE) else 11.5,
                         color=c['text'] if j == 0 else c['muted'])


def scatter(ax, cat, others, hi, c, col, other, label, me):
    """tddlog's time against the fastest of others', instance by instance;
    other: the colour of tddlog's losses, label: the x axis's, me: the
    names of tddlog and its pipelines in the category (subject)."""
    fail = 10 ** (math.log10(hi) + 0.32)
    mine = [v for _, v in my_column(cat)]
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
    for ps, colour in ((faster, TDDLOG), (slower, other), (tie, c['muted'])):
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
    ax.set_xlabel(f'{label} (s)')
    ax.set_ylabel(f'{me} (s)')
    # tddlog and a pipeline (all programs) are named by the y axis alone
    ax.set_title((f'{me} ' if len(set(my_keys(cat))) == 1 else '') + f'faster on {len(faster)}', loc='left', color=TDDLOG,
                 fontsize=11.5, pad=10)
    ax.set_title(f'slower on {len(slower)}', loc='right', color=other, fontsize=11.5, pad=10)
    return dict(points=len(pts), faster=len(faster), slower=len(slower), tie=len(tie), both_fast=both_fast,
                neither=neither)


def faster(v):
    """tddlog's speed-up of a comparison (the data's vs, rule 16): 14×, 1.3×
    slower, NONE where there is nothing to measure, or ANSWERS_NONE where
    the other system answers no instance."""
    if v['speedup'] is None:
        return NONE
    return ANSWERS_NONE if v['answers'] == 0 else ratio_text(v['speedup'])


def results(cat, names, col, out):
    """The cactus plot of the category's drawn systems beside the scatter
    plot, the legend under them: each system's answers and tddlog's
    speed-up over it. Returns each system's answers and the scatter plot's
    numbers."""
    c = style()
    systems = ordered(cat)
    others = [s for s in systems if not is_mine(s)]
    hi = max(r[2] for r in cat['rows'])
    ncol = 2
    legend_top = 4.9
    h = legend_top + LEGEND_HEAD + LEGEND_ROW * math.ceil(len(systems) / ncol) + 0.2
    fig = plt.figure(figsize=(WIDTH, h))

    def box(x, top, w, height):  # inches, from the figure's top left
        return [x / WIDTH, (h - top - height) / h, w / WIDTH, height / h]

    solved = cactus(fig.add_axes(box(0.8, 0.4, 4.35, 3.45)), cat, systems, hi, c, col)
    loss = col[cat['named'][0]][0] if cat['named'] else c['muted']
    sc = scatter(fig.add_axes(box(5.85, 0.4, 3.45, 3.45)), cat, others, hi, c, col, loss,
                 'fastest other system' if cat['key'] in (ALL, APART) else 'fastest native system', subject(cat, names))
    legend = [(s, names[s], [solved[s], '' if is_mine(s) else faster(cat['vs'][s])]) for s in systems]
    legend_table(fig, h, (WIDTH - ncol * LEGEND_BLOCK) / 2, legend_top, legend, cat['instances'], cat['noun'],
                 ncol, c, col, cat)
    save(fig, out, f'results-{cat["key"]}')
    return solved, sc


def fixed(x, digits):
    """x with digits decimals, as the results page writes it (JavaScript's
    toFixed): a tie rounded up, not to even, so that 156.5 is 157 on both."""
    return str(decimal.Decimal(x).quantize(decimal.Decimal(1).scaleb(-digits), rounding=decimal.ROUND_HALF_UP))


def seconds(t):
    if t >= 36000:
        return f'{fixed(t / 3600, 0)} h'
    if t >= 3600:
        return f'{fixed(t / 3600, 1)} h'
    if t >= 60:
        return f'{fixed(t / 60, 0)} min'
    return f'{fixed(t, 0)} s' if t >= 10 else f'{fixed(t, 1)} s'


def times(lo, hi):
    """A log axis's ticks, durations with their names, from lo to hi."""
    named = [(1, '1 s'), (10, '10 s'), (60, '1 min'), (600, '10 min'), (3600, '1 h'), (36000, '10 h'),
             (360000, '100 h')]
    return [(t, s) for t, s in named if lo <= t <= hi]


def ratio_text(r, least=False):
    """How many times less time tddlog takes, r its ratio to the other's:
    14×, 2.5×, 1.3× slower; with least, "at least" in a sign, rounded down
    (up where tddlog is slower)."""
    x = max(r, 1 / r)
    if least:
        x = math.floor(x) if x >= 10 else math.floor(x * 10) / 10
        if r < 1:
            x = math.ceil(1 / r) if 1 / r >= 10 else math.ceil(10 / r) / 10
    text = f'{fixed(x, 0 if x >= 10 else 1)}×'
    if r < 1:
        return ('≤' if least else '') + text + ' slower'
    return ('≥' if least else '') + text


def at_least(t):
    """A duration as seconds() writes it, rounded down: what "more than"
    can say of t."""
    for unit, size, digits in ((36000, 3600, 0), (3600, 3600, 1), (60, 60, 0)):
        if t >= unit:
            v = math.floor(t / size * 10 ** digits) / 10 ** digits
            return seconds(v * size)
    return seconds(math.floor(t * 10) / 10)


def speed_mark(ax, x, y, r, transform, colour, muted, least=False, none=False, quiet=False):
    """tddlog's speed-up r at x of the overview's right column, bold, teal
    where tddlog is faster, with least "at least"; NONE where r is None,
    ANSWERS_NONE with none (the other system answers no instance); quiet,
    muted and not bold (one of the others tddlog leads most)."""
    if r is None or none:
        ax.text(x, y, 'answers\nnone' if none else 'nothing to\nmeasure', transform=transform, ha='left', va='center',
                fontsize=9.5, color=muted, linespacing=1.05)
        return
    ax.text(x, y, ratio_text(r, least=least), transform=transform, ha='left', va='center', fontsize=12.5,
            fontweight='normal' if quiet else 'bold', color=muted if quiet else TDDLOG if r >= 1 else colour)


def others_text(o, names):
    """The others' row of the overview: their names and, where each has an
    instance that it or tddlog answers (a total time above 0), the least of
    their total times."""
    text = ', '.join(names[x] for x in o['systems'])
    return text + (f": each more than {at_least(o['total'])} in total" if o['total'] else '')


def overview(data, names, col, out):
    """For each type, under its title, its number of instances and tddlog's
    answers, its NAMED best native systems (cat['named']), each with its
    answers to the left, its total time and tddlog's over the instances it
    is given and either answers (PAR-2) as points on one time axis, and
    tddlog's speed-up over it to the right; the other native systems in one
    row that names them, the least of their totals and of their speed-ups,
    and below it the others tddlog leads most (cat['others']['leads']), each
    in a row of its own, muted."""
    c = style()
    lines = []
    for cat in data['categories']:
        if cat['key'] == ALL:
            continue
        who = subject(cat, names)
        lines.append(('head', f"{cat['title']}, {cat['instances']} {cat['noun']}: {who} "
                      f"{'answers' if len(set(my_keys(cat))) == 1 else 'answer'} {my_answers(cat)}", None, None))
        for x in cat['named']:
            v = cat['vs'][x]
            lines.append(('row', names[x], v['answers'], (x, v['tddlog_total'], v['total'], v['speedup'])))
            # (its answers are those of the comparison beside them: in a type, the instances
            # as their users would write them)
        o = cat['others']
        if o['systems']:
            text = textwrap.fill(others_text(o, names), 70)
            lines.append(('others', f"{len(o['systems'])} others", None, (text, o['speedup'])))
        for x in o.get('leads', []):
            v = cat['vs'][x]
            lines.append(('lead', names[x], v['answers'], (x, v['tddlog_total'], v['total'], v['speedup'])))
    ts = [t for kind, _, _, st in lines if kind in ('row', 'lead') for t in st[1:3]]
    ts += [o['others']['total'] for o in data['categories'] if o['key'] != ALL and o['others']['total']]
    names_w, ratio_w, row, top, bottom = 2.6, 1.2, 0.34, 1.0, 0.55
    # each line's height in rows, and its middle, from the top
    tall = [0.6 + 0.55 * st[0].count('\n') + 0.4 if kind == 'others' else 1 for kind, _, _, st in lines]
    mids = [sum(tall[:k]) + tall[k] / 2 for k in range(len(lines))]
    n = sum(tall)
    h = n * row + top + bottom
    axes_w = WIDTH - names_w - ratio_w
    fig = plt.figure(figsize=(WIDTH, h))
    ax = fig.add_axes([names_w / WIDTH, bottom / h, axes_w / WIDTH, n * row / h])
    at = ax.get_yaxis_transform()
    ratio_x = 1 + 0.25 / axes_w
    # room left of the smallest time for its label, which sits to the left of its point
    lo, hi = min(ts) / 10, max(ts) * 3
    ys, labels, quiet = [], [], []
    for y, (kind, name, answers, st) in zip(mids, lines):
        y = -y
        if kind == 'head':
            ax.text(-names_w / axes_w + 0.005, y, name, transform=at, ha='left', va='center', fontweight='bold',
                    fontsize=12.5)
            continue
        ys.append(y)
        labels.append(name)
        quiet.append(kind == 'lead')
        if kind == 'others':
            text, r = st
            ax.text(0.01, y, text, transform=at, ha='left', va='center', fontsize=10.5, color=c['muted'],
                    linespacing=1.25)
            speed_mark(ax, ratio_x, y, r, at, c['muted'], c['muted'], least=True)
            continue
        x, mine, t, r = st
        colour = c['muted'] if kind == 'lead' else col[x][0]
        ax.plot([mine, t], [y, y], color=c['grid'], lw=3, solid_capstyle='round', zorder=2)
        speed_mark(ax, ratio_x, y, r, at, colour, c['muted'], none=answers == 0 and r is not None,
                   quiet=kind == 'lead')
        ax.scatter([t], [y], s=64, color=colour, lw=0, zorder=3)
        ax.scatter([mine], [y], s=64, color=TDDLOG, lw=0, zorder=4)
        out_right = t >= mine
        ax.text(t * 1.3 if out_right else t / 1.3, y, seconds(t), ha='left' if out_right else 'right', va='center',
                color=c['text'], fontsize=11.5)
        ax.text(mine / 1.3 if out_right else mine * 1.3, y, seconds(mine), ha='right' if out_right else 'left',
                va='center', color=TDDLOG, fontsize=11.5)
        ax.text(-0.12 / axes_w, y, f'{answers}', transform=at, ha='right', va='center', fontsize=11, color=c['text'])
    ticks = times(lo, hi)
    log_axis(ax, 'x', [t for t, _ in ticks], [s for _, s in ticks])
    ax.set_xlim(lo, hi)
    ax.set_ylim(-n, 0)
    ax.set_yticks(ys)
    ax.set_yticklabels(labels, fontsize=12)
    for label, q in zip(ax.get_yticklabels(), quiet):
        if q:
            label.set_color(c['muted'])
    ax.tick_params(axis='y', length=0, pad=56)
    ax.tick_params(axis='x', length=0, pad=6)
    ax.grid(True, axis='x', color=c['grid'], lw=0.7)
    ax.set_axisbelow(True)
    for side in ('left', 'bottom'):
        ax.spines[side].set_visible(False)
    head = (bottom + n * row + 0.72) / h
    fig.text(0.006, head, 'Each type of program against the systems native to it', fontweight='bold', fontsize=13.5,
             va='center', ha='left')
    over = 1 + 0.12 / (n * row)
    ax.text(-0.12 / axes_w, over, 'answers', transform=ax.transAxes, ha='right', va='bottom', fontsize=11,
            color=c['muted'])
    ax.text(ratio_x, over, 'tddlog\nfaster', transform=ax.transAxes, ha='left', va='bottom', fontsize=11,
            color=c['muted'], linespacing=1.1)
    save(fig, out, 'overview')


def misses(cat):
    """tddlog's records on the instances it does not answer, by kind."""
    return dict(collections.Counter(v for _, v in my_column(cat) if not answered(v)))


def wins(cat):
    """{native system: {family group: the instances it answers and tddlog
    does not}}, the families' groups as the data orders them, for the
    README's "Where others win" bullets, whose counts and names come from
    it."""
    j, me = {x: 4 + cat['systems'].index(x) for x in cat['native']}, [v for _, v in my_column(cat)]
    out = {}
    for x in cat['native']:
        by = {g['key']: [cat['rows'][i][1] for i in g['rows']
                         if answered(cat['rows'][i][j[x]]) and not answered(me[i])] for g in cat['groups']}
        out[x] = {g: names for g, names in by.items() if names}
    return out


def within(cat, systems, limits=(1, 2, 10)):
    """{limit: {system: the instances it answers within limit seconds}}."""
    return {t: {s: sum(answered(v) and clamp(v) <= t for _, v in column(cat, s)) for s in systems} for t in limits}


def least_text(r):
    """A table's "at least" speed-up: at least 400×, at most 1.3× slower."""
    if r is None:
        return NONE
    text = ratio_text(r, least=True)
    return ('at most ' if r < 1 else 'at least ') + text.lstrip('≤≥')


def table(cat, names):
    """The README's table of the category's native systems (all programs:
    every system), rule 16: each named system's answers, its total time,
    tddlog's and tddlog's speed-up; the others in one row, the range of
    their answers, the least of their totals, the range of tddlog's and the
    least of their speed-ups; then the others tddlog leads most, a row each."""
    who = subject(cat, names, 'or')
    lines = [f"| System | Answers | Its total time | {who}'s total time | {who} faster |", '|---|--:|--:|--:|--:|']

    def row(x):
        v = cat['vs'][x]
        lines.append(f"| {names[x]} | {v['answers']} | {seconds(v['total'])} | {seconds(v['tddlog_total'])} | "
                     f"{faster(v)} |")
    for x in cat['named']:
        row(x)
    o = cat['others']
    if o['systems']:
        span = lambda lo, hi: lo if lo == hi else f'{lo} to {hi}'
        mine = span(*map(seconds, o['tddlog']))
        total = f"over {at_least(o['total'])} each" if o['total'] else '–'
        lines.append(f"| {len(o['systems'])} others: {', '.join(names[x] for x in o['systems'])} | "
                     f"{span(*o['answers'])} | {total} | {mine} | {least_text(o['speedup'])} |")
    for x in o.get('leads', []):
        row(x)
    return lines


def integrations(data, names):
    """Each integration's numbers (its instances, the pipeline's and the
    system's given and answers, and the system against the pipeline, rule
    16), and the lines of the README's table of them: a row each, the
    pipeline's answers and total time with the system's in brackets, and
    the pipeline's speed-up."""
    out = {}
    lines = ['| Integration | Instances | Answers | Total time | Faster |', '|---|--:|--:|--:|--:|']
    for key, g in data['integrations'].items():
        if 'pipeline' not in g:     # tddlog's own: its answers by type, no comparison
            continue
        p, x = g['pipeline'], g['system']
        v = g['vs'][x]
        out[key] = dict(instances=g['instances'], pipeline=p, system=x, given=g['given'], answers=g['answers'], vs=v)
        lines.append(f"| {names[p]} against {names[x]} | {g['instances']} | {g['answers'][p]} ({names[x]} "
                     f"{g['answers'][x]}) | {seconds(v['tddlog_total'])} ({names[x]} {seconds(v['total'])}) | "
                     f"{faster(v)} |")
    return dict(out, table=lines)


SITE = 'https://tractables.github.io/tddlog'
# a generated block of the README: its key, then its text up to its end marker
BLOCK = re.compile(r'<!-- results:(?P<key>[\w-]+) -->\n(?P<body>.*?)<!-- /results:(?P=key) -->', re.S)


def blocks(data, numbers, names, digest):
    """{key: text} of the README's generated blocks, from the data and the
    numbers main computes of it: the headline (headline), the overview's
    caption (the data's date; digest, the SHA-256 of the categories.json
    read, in a comment) and the integrations' table. The setup, the
    measure and each category's own figure and table are the site's pages,
    which the overview links to (the owner, 2026-11-09: "these are details
    that don't belong in a README"; "People can click on the figure")."""
    out = {
        'overview': (f"<!-- written from bindings/wasm/results/categories.json, sha256 {digest} -->\n"
                     f"<sub>As of {data['stamp']['date']}. Click the figure for every result.</sub>"),
        'integrations': '\n'.join(numbers['integrations']['table']),
    }
    out['headline'] = headline(data, names)
    return out


# The README's headline types, as it names them: the two whose systems most
# readers know (the owner, 2026-11-09: "tddlog outperforms existing [probabilistic
# logic programming] systems 214x and ... the best existing Bayesian network
# solvers 55x. I think those are the two highlights").
HEADLINE = (('logic-programs', 'probabilistic logic programs'), ('bayesian-networks', 'Bayesian networks'))


def headline(data, names):
    """The README's first numbers: tddlog's speed-up over the first of the
    named native systems (the one that answers the most) of each HEADLINE
    type, linked to the type's page; a type without a measured speed-up is
    left out."""
    cats = {c['key']: c for c in data['categories']}
    parts = []
    for key, what in HEADLINE:
        c = cats.get(key)
        if not c or not c['named'] or faster(c['vs'][c['named'][0]]) in (NONE, ANSWERS_NONE):
            continue
        x = c['named'][0]
        r = c['vs'][x]['speedup']
        how = f"**{ratio_text(max(r, 1 / r))}** {'faster' if r >= 1 else 'slower'} than {names[x]}"
        parts.append(f"On [{what}]({SITE}/#/category/{key}), {names['tddlog']} is {how}" if not parts
                     else f"on [{what}]({SITE}/#/category/{key}) {how}")
    if not parts:
        return ''
    return (f"{', and '.join(parts)}, the existing system that answers the most of {'each' if len(parts) > 1 else 'them'}. "
            f"[Detailed results below](#how-well-does-it-work).")


def readme(text, want):
    """text, a README, with each generated block's body the one want gives
    (blocks), and the keys whose body differed; a block of want without its
    markers in text, or markers want has no block for, stops (rule 3)."""
    seen, changed = set(), []

    def put(m):
        k = m['key']
        assert k in want, f'README: a block results:{k} that the data have none of'
        assert k not in seen, f'README: the block results:{k} twice'
        seen.add(k)
        body = want[k] + '\n'
        if m['body'] != body:
            changed.append(k)
        return f'<!-- results:{k} -->\n{body}<!-- /results:{k} -->'

    new = BLOCK.sub(put, text)
    missing = sorted(set(want) - seen)
    assert not missing, f'README: no block for {missing} (<!-- results:KEY --> ... <!-- /results:KEY -->)'
    return new, changed


def main():
    args = [a for a in sys.argv[1:] if a != '--png']
    path = os.path.join(SOURCE, 'bindings', 'wasm', 'results', 'categories.json')
    if args[:1] == ['--data']:
        path, args = args[1], args[2:]
    write = check = None
    if args[:1] == ['--readme']:
        write, args = args[1], args[2:]
    elif args[:1] == ['--check']:
        check, args = args[1], args[2:]
    out = args[0] if args else os.path.join(SOURCE, 'docs', 'images')
    os.makedirs(out, exist_ok=True)
    data = load(path)
    global FLOOR
    FLOOR = data['measure']['floor']  # the data's, as its numbers were computed with it
    names, col = data["names"], colours(written(data))
    numbers = {}
    for cat in written(data):
        assert cat['drawn'][:len(cat['mine'])] == cat['mine'], f"{cat['key']}: tddlog not drawn"
        res = dict(instances=cat['instances'], limit=max(r[2] for r in cat['rows']))
        # who the text names for tddlog (tddlog, Alea.jl+tddlog or both) and their answers
        res['subject'], res['mine'], res['my_answers'] = subject(cat, names), cat['mine'], my_answers(cat)
        res['answers'], res['scatter'] = results(cat, names, col, out)
        # The legend's numbers are the results page's (the data's).
        assert all(res['answers'][s] == cat['answers'][s] for s in cat['drawn']), f"{cat['key']}: not the data"
        # each native system against tddlog by rule 16, by answers, then speed-up
        res['native'] = {x: dict(answers=cat['answers'][x], given=cat['given'][x], vs=cat['vs'][x])
                         for x in cat['native']}
        res['named'], res['others'] = cat['named'], cat['others']
        res['not_drawn'] = {s: cat['given'][s] for s in cat['systems'] if s not in cat['drawn']}
        res['tddlog_failures'] = misses(cat)
        res['wins'], res['groups'] = wins(cat), {g['key']: len(g['rows']) for g in cat['groups']}
        res['within'] = within(cat, cat['drawn'])
        res['table'] = table(cat, names)
        numbers[cat['key']] = res
    overview(data, names, col, out)
    printed = dict(categories=numbers, integrations=integrations(data, names))
    json.dump(printed, sys.stdout, indent=1, ensure_ascii=False)
    print()
    if write or check:
        digest = hashlib.sha256(open(path, 'rb').read()).hexdigest()
        text = open(write or check).read()
        new, changed = readme(text, blocks(data, printed, names, digest))
        if write:
            open(write, 'w').write(new)
            changed = []
        if changed:
            sys.exit(f"README.md: the blocks {', '.join(changed)} differ from the data")


if __name__ == '__main__':
    main()
