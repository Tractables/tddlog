"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl,
// Roulette and Geni), both written by the generator of README.md's results
// and the baselines branch's docs/benchmarks.md; the results by type of
// program from categories.json (written by the results' processing); and, where
// the site has them, the benchmarks' data from the baselines branch
// (benchmarks/, written by the results' processing): each suite's
// families of instances and, for each family, its programs in every
// language, its runs and its answers.
//
// The page is one view at a time, named by the address's hash:
//   #/                       by program type (the front page's #/types): the README's figures, each type's
//                            totals and speed-ups, cactus and scatter plots
//   #/category/C[/in/FAMILY][/vs/SYSTEM]
//                            a category: its systems' answers, totals and speed-ups, its plots (tddlog against
//                            SYSTEM), narrowed to FAMILY (the program or model whose sizes they are)
//                            or to own:SYSTEM (a system's own benchmarks)
//   #/system/SYSTEM          a solver's page, which is the front page's: a baseline's (#/baselines/SYSTEM),
//                            or tddlog's alone or in a pipeline (#/integrations/...)
//   #/SUITE                  a suite: its plot, tddlog against each system, its families
//   #/SUITE/FAMILY[/NAME]    a family: its instances' times, and an instance's programs and answers
//   #/SUITE/@NAME            the family of the instance NAME
// and after an instance's name, what is open in its view:
//   …/tab/T[/F]              the file F (0 where left out) of the instance's language T
//   …/runs/SYSTEM            SYSTEM's runs of the instance, in a drawer beside the view
// A time in a family's table opens that system's runs of the instance
// beside the view. A category's instances, systems, answers, total times and
// speed-ups are the data's (the README's figures draw the same), narrowed to
// a family or a system's own benchmarks too (c.narrowed): the page does not
// choose or compute them. It counts answers and draws the plots, as the
// README's figures do.

// The stamp of the page's own reference (results.js?v=<sha>) goes on to the
// data, as the playground's does to its files. A page elsewhere that reuses
// this app (the site's front page) names where the data is
// (data-base="results/") and where the front page is from it
// (data-front=""): the menu's sections are the front page's views. A view's
// title goes before the page's own.
const stamp = document.currentScript === null ? "" : new URL(document.currentScript.src).search;
const dataRoot = document.currentScript?.dataset.base ?? "";
const front = document.currentScript?.dataset.front ?? "../";
const pageTitle = document.title;

// What a page that reuses this app adds to it (next/next.js): views of its
// own, by their routes (parse: an address's hash, without its #, to a route
// or null; href: a route to its hash or null; draw: a route to the view's
// parts or null; title: a route's title or null), the parts a category's
// view shows after its headline (category), the HTML of a line on how a
// file was made (made), and the data of its own, loaded with this app's
// (load).
const extend = { parse: () => null, href: () => null, draw: () => null, title: () => null, category: () => [],
  made: (html) => html, load: async () => null };

// The site's sections, in the order of the bar at the top of every page and
// of the menu, by their front page's address: their names are these, here
// only (the pages' bars, which tests/next.cjs holds to them, name them too).
const SECTIONS = [["baselines", "By baseline"], ["integrations", "By integration"], ["types", "By program type"], ["circuits", "Circuits"], ["how", "How it works"]];
const sectionName = (key) => SECTIONS.find(([k]) => k === key)[1];
// The first crumbs of a view: the front page, and the section the view is in.
const HOME = ["Home", { view: "section", key: "" }];
const inSection = (key) => [sectionName(key), { view: "section", key }];

// The walkthroughs of How it works (the front page's next.js draws them), by
// their keys, in the menu's order.
const WALK_TITLES = [["reach", "Reachability with cycles"], ["gears", "Negation through a loop"], ["tpch", "A TPC-H query"],
  ["network", "A Bayesian network"], ["markov", "A Markov network"], ["dice", "A Dice program"], ["circuit", "An Alea.jl circuit"],
  ["roulette", "A Roulette program"]];

// The integrations, tddlog's own and its pipelines (categories.json's
// integrations: tddlog alone, Alea.jl+tddlog, Roulette+tddlog), in the
// menu's order, and the integration a member of tddlog's is.
const INTEGRATIONS = ["tddlog", "alea", "roulette"];
const integrationsOf = () => INTEGRATIONS.filter((k) => data.integrations?.[k]);
const integrationOf = (x) => (x === "tddlog" ? "tddlog" : x.replace(/-tddlog$/, ""));
// A system's colour, its line's and its swatch's.
const tint = (key) => `s-${base(key)}`;

// The time axis starts at 0.2 s: runs under 0.2 s are not told apart.
const FLOOR = 0.2;

// How a failure reads in a table cell, and what it means: a key's short
// form before the colon, the whole in a cell's title.
const FAILURES = {
  timeout: [(limit) => `>${limit}`, "the time limit ran out"],
  memout: [() => "memory", "over the memory limit: the memory each run may take (its resident memory, or in the earlier runs its address space; Methods gives them), or the Java heap"],
  error: [() => "error", "the system stopped with an error"],
  unsupported: [() => "n/a", "the system cannot express the program: a translator refuses what its target language lacks (lists, division and recursion in a Dice program's translation; negation through recursion, or more than two million conjunctions, in a ground program's network), as aspmc's harness refuses an observation with a body"],
  skipped: [() => "skipped", "after a smaller size failed: not run, since the system had run out of time or memory on a smaller size of the same program in the same run"],
  wrong: [() => "wrong", "off the reference: an answer off it by more than 1e-9 and either by more than 1e-6 or by more than 1e-3 of it, beyond the last digit either of the two prints"],
  zero: [() => "zero", "every answer 0: printed as 0 where the reference's are positive"],
  underflow: [() => "underflow", "every answer 0, underflowing: printed as 0 where the probability of the evidence is below the smallest double"],
  unread: [() => "not read", "not read by tddlog: only another system's front end reads the program, so tddlog counts as unanswered"],
};
// A failure in a key: its cell (with its shade, where it has one of its own) and its short meaning, the whole as its title.
const failureKey = (v, limit, shade) => element("span", { title: FAILURES[v][1] }, shade, `${FAILURES[v][0](limit)}: ${FAILURES[v][1].split(":")[0]}`);
// All programs: the union of the types of program, every system on every
// program it is given (categories.json's last category).
const ALL = "all-programs";
// The circuits: every circuit Alea.jl exports, which every system reads under
// its plain name (categories.json's `circuits`, a category's entry), a
// section of its own apart from the types.
const APART = "circuits";
// A category's entry by its key: a type's, all programs', or the circuits'.
const categoryOf = (key) => data.categories.find((c) => c.key === key) ?? (data.circuits?.key === key ? data.circuits : undefined);

// A time in a table is shaded by how many times slower it is than its
// instance's fastest, up to this many.
const SLOWEST = 100;

// The most lines of a file the page shows; the whole file can be saved.
const SHOWN_LINES = 2000;

// The highlighter's language (code.js) of a file, by its name's ending.
const HIGHLIGHT = { pl: "problog", py: "python", rs: "rust", c: "c", h: "c", toml: "toml" };

// The fields of a file the page reads for itself; any other text a file
// has (how it was made, a licence) is a line of HTML above its text.
const FILE_FIELDS = new Set(["name", "bytes", "lines", "sha", "src", "part", "gzip", "held", "archive", "form", "readers", "for", "data"]);

// The most of what the search finds that it lists.
const SHOWN_HITS = 40;

const svgNS = "http://www.w3.org/2000/svg";

function element(tag, attributes = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attributes)) {
    if (value === null || value === undefined || value === false) continue;
    if (key === "class") node.className = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children.flat(Infinity)) {
    if (child !== null && child !== undefined && child !== false) node.append(child);
  }
  return node;
}

function svg(tag, attributes = {}, text) {
  const node = document.createElementNS(svgNS, tag);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  if (text !== undefined) node.textContent = text;
  return node;
}

// The prose's markup: `code`, **bold** and [text](https://...), nothing
// else; every piece of text goes in as text.
function prose(tag, text, attributes = {}) {
  const node = element(tag, attributes);
  for (const piece of text.split(/(`[^`]+`|\*\*[^*]+\*\*|\[[^\]]+\]\(https:\/\/[^)\s]+\))/)) {
    if (piece === "") continue;
    let m;
    if ((m = piece.match(/^`([^`]+)`$/))) node.append(element("code", {}, m[1]));
    else if ((m = piece.match(/^\*\*([^*]+)\*\*$/))) node.append(element("b", {}, m[1]));
    else if ((m = piece.match(/^\[([^\]]+)\]\((https:\/\/[^)\s]+)\)$/))) node.append(element("a", { href: m[2] }, m[1]));
    else node.append(piece);
  }
  return node;
}

// The benchmarks' data writes a file's origin and why a file is not shown
// as HTML (bench_pages.py escapes all of it but its <code> and <a>).
function markup(tag, html, attributes = {}) {
  const node = element(tag, attributes);
  node.innerHTML = html;
  return node;
}

// A paragraph's first sentence shown, the rest under "More".
function brief(text, cls) {
  const cut = text.search(/(?<=[a-z0-9)'"]\.) (?=[A-Z])/);
  if (cut < 0 || text.length < 260) return prose("p", text, { class: cls });
  const box = element("div", { class: cls });
  box.append(prose("p", text.slice(0, cut)), element("details", { class: "more" }, element("summary", {}, "More"),
    prose("p", text.slice(cut + 1))));
  return box;
}

const note = (...children) => element("p", { class: "note" }, ...children);
// Numbers in cells of their own, each under its label ([label, value,
// title]), never run together in a sentence.
function cells(...items) {
  return element("div", { class: "cells" }, ...items.map(([label, value, title]) =>
    element("div", { class: "cell", title }, element("span", {}, label), element("b", {}, value))));
}
// A text that would take more than a line, folded under its summary.
const fold = (summary, ...text) => element("details", { class: "fold" }, element("summary", {}, summary), element("p", {}, ...text));

function count(n) {
  return n.toLocaleString("en");
}

// A list in prose: "A", "A and B", "A, B and C".
function and(items) {
  return items.length < 2 ? items.join("") : `${items.slice(0, -1).join(", ")} and ${items[items.length - 1]}`;
}

function seconds(t) {
  if (t < 0.01) return t.toFixed(3);
  if (t < 1) return t.toFixed(2);
  if (t < 100) return t.toFixed(1);
  return t.toFixed(0);
}

// A total time as the README's legends write it: 3.9 s, 108 s, 21 min, 1.2 h.
function duration(t) {
  if (t >= 36000) return `${(t / 3600).toFixed(0)} h`;
  if (t >= 3600) return `${(t / 3600).toFixed(1)} h`;
  if (t >= 60) return `${(t / 60).toFixed(0)} min`;
  return t >= 10 ? `${t.toFixed(0)} s` : `${t.toFixed(1)} s`;
}

// One system's total time against another's: 12×, 1.5×; with down, rounded
// down, what "at least" can say of it.
function times(a, b, down = false) {
  const r = a / b;
  if (down) return `${r >= 10 ? Math.floor(r) : (Math.floor(r * 10) / 10).toFixed(1)}×`;
  return `${r >= 10 ? r.toFixed(0) : r.toFixed(1)}×`;
}

// A total time as duration writes it, rounded down: what "more than" can
// say of it.
function atLeast(t) {
  for (const [unit, size, digits] of [[36000, 3600, 0], [3600, 3600, 1], [60, 60, 0]]) {
    if (t >= unit) return duration(Math.floor(t / size * 10 ** digits) / 10 ** digits * size);
  }
  return duration(Math.floor(t * 10) / 10);
}

// A number in words, to twelve: "five types of program".
const WORDS = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve"];
const inWords = (n) => WORDS[n] ?? count(n);

function plural(n, one, many = `${one}s`) {
  return `${count(n)} ${n === 1 ? one : many}`;
}

const answered = (value) => typeof value === "number";
// A process time (Alea.jl's and Roulette's with their start-up) is drawn
// dashed, in its system's colour.
const base = (key) => key.replace(/-process$/, "");
const dashed = (key) => key.endsWith("-process");
// tddlog and its pipelines (Alea.jl+tddlog: Alea.jl runs the program up to the circuit
// it builds and hands it to tddlog), the subjects of the comparisons. A pipeline is a
// system of its own: in a type tddlog is every row's subject, unanswered ("unread")
// where only another system's front end reads the program; in an integration the
// pipeline is, plain tddlog beside it.
const isMine = (key) => key === "tddlog" || key.endsWith("-tddlog");
// A row's subject (catRows'): the first of them given it in the systems' order,
// and its time there.
const myKey = (r) => Object.keys(r.v).find((x) => isMine(x) && r.v[x] !== null);
const myTime = (r) => r.t[myKey(r)] ?? null;
// The subjects of rows (tddlog first), and how many of the rows they answer.
const subjectsOf = (rows, key = myKey) => [...new Set(rows.map(key))].filter(Boolean)
  .sort((a, b) => (b === "tddlog") - (a === "tddlog") || a.localeCompare(b));
const answeredBy = (rows, key = myKey) => rows.filter((r) => (r.t[key(r)] ?? null) !== null).length;
// A comparison's subject in words: the names of its `me`, tddlog where it has none.
const subject = (keys) => and((keys?.length ? keys : ["tddlog"]).map((x) => data.names[x]));

// An instance's fastest answer of its records `values` (a process's time
// left out by the caller), at least 0.2 s, as every comparison counts it;
// null where none answers.
function fastestOf(values) {
  const ts = values.filter(answered);
  return ts.length ? Math.max(FLOOR, Math.min(...ts)) : null;
}

// How a record reads beside its instance's fastest answer: "best", the
// fastest (every answer under 0.2 s ties for it); "slow", with `p` from 0 to
// 1 as it takes up to SLOWEST times the fastest's time; "fail", no answer;
// "na", a program the system cannot express. `slower` is how many times the
// fastest's time it takes, for a cell's title.
function rated(v, fastest) {
  if (!answered(v)) return { cls: v === "unsupported" || v === "unread" ? "na" : "fail", p: 0 };
  const r = Math.max(v, FLOOR) / fastest;
  if (r <= 1) return { cls: "best", p: 0 };
  return { cls: "slow", p: Math.min(1, Math.log(r) / Math.log(SLOWEST)), slower: times(Math.max(v, FLOOR), fastest) };
}

// The key to a table's shades; `na` where a system cannot express some
// instance.
function shades(na) {
  return [element("span", {}, element("i", { class: "heat best" }), `the fastest (under ${FLOOR} s, a tie)`),
    element("span", {}, element("i", { class: "heat slow" }), `slower, up to ${SLOWEST}× the fastest's time`),
    element("span", {}, element("i", { class: "heat fail" }), "no answer"),
    na ? failureKey("unsupported", 0, element("i", { class: "heat na" })) : null].filter(Boolean);
}

// A system's sample of line: in its colour, dashed for a process's time, or
// grey where its line in a plot is.
function swatch(key, grey = false) {
  return element("i", { class: `sw ${tint(key)}${dashed(key) ? " dashed" : ""}${grey ? " grey" : ""}` });
}

// ------------------------------------------------------------- tables

// A table in a box that scrolls it, sideways too, with its first column held
// (styles.css) and a shade at the right edge while more of it is to the
// right.
function scroller(table, cls = "") {
  const box = element("div", { class: `tscroll ${cls}`.trim() }, table);
  const edge = element("div", { class: "edge" }, box);
  const shade = () => edge.classList.toggle("more", box.scrollLeft + box.clientWidth < box.scrollWidth - 1);
  box.addEventListener("scroll", shade, { passive: true });
  const watch = new ResizeObserver(shade);
  watch.observe(box);
  watch.observe(table);
  return edge;
}

// A board whose rows a phone stacks (styles.css), each value named by its
// column.
function stacked(table) {
  // each column's name, under its group's where a head has two rows (head2)
  const names = [];
  const [top, below] = table.tHead.rows;
  const under = [...(below?.cells ?? [])];
  for (const th of top.cells) {
    if (th.colSpan > 1 || (below && th.rowSpan < 2)) {
      for (let k = 0; k < th.colSpan; k++) names.push(`${th.textContent} ${under.shift().textContent}`);
    } else names.push(th.textContent);
  }
  for (const row of table.tBodies[0].rows) {
    if (row.cells.length !== names.length) continue;
    [...row.cells].forEach((cell, i) => {
      if (cell.tagName === "TD") cell.dataset.label = names[i];
    });
  }
  table.classList.add("stack");
  return table;
}

// A table's head of two rows, so that a long name stands once over the
// columns it names: a column alone, [label, title, class], spans both rows;
// a group, { group, cols: [[label, title, class], ...] }, names its columns
// above them.
function head2(...items) {
  const th = ([label, title, cls], extra = {}) => element("th", { scope: "col", class: cls ?? null, title: title ?? null, ...extra }, label);
  const top = element("tr"), below = element("tr");
  for (const it of items) {
    if (Array.isArray(it)) top.append(th(it, { rowspan: 2 }));
    else {
      top.append(element("th", { scope: "colgroup", colspan: it.cols.length, class: "group" }, it.group));
      below.append(...it.cols.map((c) => th(c)));
    }
  }
  return element("thead", {}, top, below);
}

// A speed-up's name in a header or a cell's label, a link to the methods
// table that defines it on the front page.
const speedName = (label) => element("a", { href: `${front}#/methods`, class: "defined" }, label);

// A <details> whose body is built when it is first opened.
function lazy(attributes, summary, build) {
  const box = element("details", attributes, element("summary", {}, summary));
  box.addEventListener("toggle", () => {
    if (box.open && box.children.length === 1) box.append(...build().filter(Boolean));
  });
  return box;
}

// Where a view's numbers come from: the data's files, as links.
function raw(...files) {
  return note("The data of this view: ", ...files.flatMap((file, i) => [i === 0 ? "" : i === files.length - 1 ? " and " : ", ",
    element("a", { href: dataRoot + file }, file.split("/").pop())]), ".");
}

// ------------------------------------------------------------- the plot

// The lines of a plot: each system's answers within the suite's limit, in
// order, tddlog's (and its pipelines') first and then by how many each answers.
function linesOf(suite) {
  return suite.systems.map((key, column) => ({ key,
    times: suite.instances.map((row) => row[column + 1]).filter(answered).filter((t) => t <= suite.limit * 1.001).sort((a, b) => a - b) }))
    .sort((a, b) => isMine(b.key) - isMine(a.key) || b.times.length - a.times.length);
}

// The instances each system answers within a time limit, the limit on a
// linear scale from 0 to the suite's; a line further up and to the left is
// better. Drawn at the width of its box; the lines not `coloured` grey and
// under the others. Each coloured line is named at its end, by its system
// where there is room, else by its number in the legend; a grey line's name
// shows while it is picked out: under the pointer, or its legend entry's.
// Returns whether the plot is narrow.
function cactus(box, suite, names, series, coloured) {
  const width = Math.max(280, Math.floor(box.clientWidth));
  const narrow = width < 480;
  const ends = series.map(({ key, times }, i) => ({ key, y: times.length, text: narrow ? String(i + 1) : `${names[key]} ${times.length}` }))
    .filter((end) => !narrow || coloured.has(end.key));
  const named = ends.filter((end) => coloured.has(end.key));
  const height = narrow ? 300 : Math.max(380, named.length * 14 + 22 + 46);
  const n = suite.instances.length;
  const hi = suite.limit;
  // The end labels' room: the longest at 7 px a character; the counts' on
  // the left, at about 6.5 px a digit, clear of the axis's name.
  const left = Math.max(46, 28 + 6.5 * String(n).length), right = narrow ? 30 : Math.max(128, 16 + 7 * Math.max(0, ...ends.map((end) => end.text.length))), top = 22, bottom = 46;
  const x0 = left, x1 = width - right, y0 = top, y1 = height - bottom;
  const lx = (t) => x0 + Math.min(Math.max(t, FLOOR), hi) / hi * (x1 - x0);
  const ly = (k) => y1 - (k / n) * (y1 - y0);

  const plot = svg("svg", { class: "cactus", width, height, viewBox: `0 0 ${width} ${height}`, role: "img", tabindex: "0",
    "aria-label": `${suite.title}: instances each system answers within a time limit; the arrow keys move the limit` });
  // a tick every 1, 2 or 5 times a power of ten, about five of them, and one at the limit
  const room = narrow ? 4 : 6;
  const unit = 10 ** Math.floor(Math.log10(hi / room));
  const every = [1, 2, 5, 10].map((m) => m * unit).find((s) => hi / s <= room);
  const ticks = [];
  for (let t = 0; t < hi - every / 2; t += every) ticks.push(t);
  ticks.push(hi);
  for (const t of ticks) {
    const x = x0 + t / hi * (x1 - x0);
    plot.append(svg("line", { class: "grid", x1: x, y1: y0, x2: x, y2: y1 }));
    plot.append(svg("text", { class: "tick", x, y: y1 + 16, "text-anchor": "middle" }, String(t)));
  }
  const steps = [1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500];
  const step = steps.find((s) => n / s <= (narrow ? 5 : 8)) ?? 1000;
  for (let k = 0; k < n; k += step) {
    if (n - k < step / 2) continue;
    plot.append(svg("line", { class: "grid", x1: x0, y1: ly(k), x2: x1, y2: ly(k) }));
    plot.append(svg("text", { class: "tick", x: x0 - 6, y: ly(k) + 4, "text-anchor": "end" }, String(k)));
  }
  plot.append(svg("line", { class: "grid all", x1: x0, y1: ly(n), x2: x1, y2: ly(n) }));
  plot.append(svg("text", { class: "tick", x: x0 - 6, y: ly(n) + 4, "text-anchor": "end" }, String(n)));
  plot.append(svg("text", { class: "tick", x: x0 + 4, y: ly(n) - 6 }, `all ${count(n)} instances`));
  plot.append(svg("line", { class: "axis", x1: x0, y1: y1, x2: x1, y2: y1 }));
  plot.append(svg("line", { class: "axis", x1: x0, y1: y0, x2: x0, y2: y1 }));
  plot.append(svg("text", { class: "label", x: (x0 + x1) / 2, y: height - 8, "text-anchor": "middle" },
    "time limit per instance, seconds"));
  plot.append(svg("text", { class: "label", transform: `translate(13 ${(y0 + y1) / 2}) rotate(-90)`, "text-anchor": "middle" },
    "instances answered"));

  // The grey lines first, then the coloured, tddlog's last, so that it is
  // drawn on top.
  const order = [...series].sort((a, b) => coloured.has(a.key) - coloured.has(b.key) || isMine(a.key) - isMine(b.key));
  for (const { key, times } of order) {
    let k = 0;
    let d = `M${x0},${ly(0)}`;
    for (const t of times) {
      d += `H${lx(t).toFixed(1)}V${ly(++k).toFixed(1)}`;
    }
    d += `H${x1}`;
    plot.append(svg("path", { class: `line ${tint(key)}${isMine(key) ? " mine" : ""}${dashed(key) ? " dashed" : ""}${coloured.has(key) ? "" : " grey"}`,
      d, "data-key": key }));
  }
  // The end labels: each line's name at its last height, moved apart where
  // two would overlap: pushed down, the lowest kept above the time axis's
  // last number and those over it pushed up, so that a crowd at the bottom
  // moves up only as far as it must.
  const labels = named.map((end) => ({ ...end, y: ly(end.y) })).sort((a, b) => a.y - b.y);
  for (let i = 1; i < labels.length; i++) labels[i].y = Math.max(labels[i].y, labels[i - 1].y + 14);
  if (labels.length) labels[labels.length - 1].y = Math.min(labels[labels.length - 1].y, y1 - 3);
  for (let i = labels.length - 2; i >= 0; i--) labels[i].y = Math.min(labels[i].y, labels[i + 1].y - 14);
  for (const l of labels) plot.append(svg("text", { class: "end", x: x1 + 8, y: l.y + 4, "data-key": l.key }, l.text));
  // a grey line's name, at its own end, shown while the line is picked out
  for (const end of ends.filter((e) => !coloured.has(e.key))) {
    plot.append(svg("text", { class: "end grey quiet", x: x1 + 8, y: Math.min(ly(end.y), y1 - 3) + 4, "data-key": end.key }, end.text));
  }

  // The hover layer: a line at a time limit, and how many each system
  // answers within it; the limit under the pointer, where a finger taps (it
  // stays until the next tap), or moved by the arrow keys. The line nearest
  // the pointer, within 12 px, is picked out: drawn in its colour, its name
  // at its end and first in the list, the others faint.
  const cross = svg("line", { class: "cross", x1: 0, y1: y0, x2: 0, y2: y1, visibility: "hidden" });
  plot.append(cross);
  const area = svg("rect", { class: "hit", x: x0, y: y0, width: x1 - x0, height: y1 - y0 });
  plot.append(area);
  const tip = element("div", { class: "tip", hidden: true, "aria-live": "polite" });
  const pick = (key) => {
    for (const mark of plot.querySelectorAll("[data-key]")) {
      mark.classList.toggle("dim", key !== null && mark.dataset.key !== key);
      mark.classList.toggle("hl", key !== null && mark.dataset.key === key);
    }
  };
  let cx = null;
  const at = (x, y = null) => {
    cx = Math.min(Math.max(x, x0), x1);
    // a time under 0.2 s counts as 0.2 s
    const t = Math.max(FLOOR, (cx - x0) / (x1 - x0) * hi);
    cross.setAttribute("x1", cx);
    cross.setAttribute("x2", cx);
    cross.setAttribute("visibility", "visible");
    const rows = series.map(({ key, times }) => [key, times.filter((s) => s <= t).length]);
    let near = null;
    if (y !== null) {
      for (const [key, k] of rows) {
        const d = Math.abs(ly(k) - y);
        if (d <= 12 && (near === null || d < near.d)) near = { key, d };
      }
    }
    pick(near?.key ?? null);
    tip.replaceChildren(element("b", {}, `within ${seconds(t)} s`));
    rows.sort((a, b) => (b[0] === near?.key) - (a[0] === near?.key) || b[1] - a[1]);
    for (const [key, k] of rows) {
      tip.append(element("span", { class: key === near?.key ? "picked" : null }, swatch(key, !coloured.has(key) && key !== near?.key), `${names[key]} ${k}`));
    }
    tip.hidden = false;
    const w = tip.offsetWidth;
    const px = cx + 12 + w > width ? cx - 12 - w : cx + 12;
    tip.style.left = `${Math.max(0, px)}px`;
    tip.style.top = `${y0}px`;
  };
  const hide = () => {
    cross.setAttribute("visibility", "hidden");
    tip.hidden = true;
    pick(null);
  };
  const pointer = (event) => {
    const r = plot.getBoundingClientRect();
    at(event.clientX - r.left, event.clientY - r.top);
  };
  area.addEventListener("pointermove", pointer);
  area.addEventListener("pointerdown", pointer);
  area.addEventListener("pointerleave", (event) => {
    if (event.pointerType === "mouse") hide();
  });
  plot.addEventListener("focus", () => at(cx ?? (x0 + x1) / 2));
  plot.addEventListener("blur", hide);
  plot.addEventListener("keydown", (event) => {
    const to = { ArrowLeft: (cx ?? x0) - (x1 - x0) / 40, ArrowRight: (cx ?? x0) + (x1 - x0) / 40, Home: x0, End: x1 }[event.key];
    if (to === undefined) return;
    event.preventDefault();
    at(to);
  });
  box.replaceChildren(plot, tip);
  return narrow;
}

// The legend: every line in the plot's order, its name and how many
// instances it answers, and in a kind's plot tddlog's speed-up over it
// (faster: {system: text}, the data's); in a narrow plot, the number its line
// ends with. A name is a button that colours its line or greys it, and a
// name under the pointer or the keyboard brings its line forward. A bare
// plot's (recolour null) lists its coloured lines only, without buttons: the
// grey ones are named under the pointer in the plot.
function legend(series, names, n, figure, faster, coloured, narrow, recolour) {
  const list = element("ul", { class: "legend" });
  series.forEach(({ key, times }, i) => {
    if (!recolour && !coloured.has(key)) return;
    const counts = `answers ${count(times.length)} of ${count(n)}${faster?.[key] === undefined ? "" : `, ${faster[key]}`}`;
    const parts = [narrow ? element("span", { class: "no" }, String(i + 1)) : null, swatch(key, !coloured.has(key)),
      element("span", {}, names[key]), element("span", { class: "n" }, counts)];
    const button = !recolour ? element("span", { class: "bare", "data-system": key }, ...parts)
      : element("button", { type: "button", class: "bare", "data-system": key, "aria-pressed": String(coloured.has(key)),
        onclick: () => recolour(key) }, ...parts);
    const focus = (on) => {
      for (const mark of figure.querySelectorAll("svg [data-key]")) {
        mark.classList.toggle("dim", on && mark.dataset.key !== key);
        mark.classList.toggle("hl", on && mark.dataset.key === key);
      }
    };
    for (const [type, on] of [["pointerenter", true], ["pointerleave", false], ["focus", true], ["blur", false]]) {
      button.addEventListener(type, () => focus(on));
    }
    list.append(element("li", {}, button));
  });
  return list;
}

// draw(), again whenever box changes width.
function onWidth(box, draw) {
  let last = 0;
  new ResizeObserver(() => {
    const w = box.clientWidth;
    if (w === 0 || w === last) return;
    last = w;
    draw();
  }).observe(box);
}

// A plot with its legend, drawn again when its box changes width or a line
// is coloured or greyed. The lines `colour` names are in colour, or else,
// where there are more than five lines, tddlog's and the three that answer
// the most; `also`'s too, and the others grey until asked for (a category's:
// tddlog's and its five best native systems', each system in its own
// colour). The figure names the systems it draws and the number of its
// instances; a category's legend gives tddlog's speed-up over each other
// system too (suite.faster). A `bare` plot has no legend where its lines are
// named at their ends, nor a button to colour them all: a line under it
// counts its grey lines.
function plotted(suite, names, { also, bare, colour } = {}) {
  const series = linesOf(suite);
  const top = colour ? series.map((s) => s.key).filter((key) => colour.includes(key))
    : series.slice(0, series.length > 5 ? 4 : series.length).map((s) => s.key);
  if (also && !top.includes(also)) top.push(also);
  let coloured = new Set(top);
  const figure = element("figure", { class: "plot", "data-systems": suite.systems.join(" "),
    "data-instances": String(suite.instances.length) });
  const box = element("div", { class: "box" });
  const every = series.length > top.length && !bare ? element("button", { type: "button", class: "plain every", onclick: () => {
    coloured = new Set(coloured.size === series.length ? top : series.map((s) => s.key));
    draw();
  } }) : null;
  const greys = bare && series.length > top.length ? element("figcaption", { class: "greys" },
    `${plural(series.length - top.length, "more system")} in grey: point at a line to name it.`) : null;
  figure.append(box);
  const draw = () => {
    const narrow = cactus(box, suite, names, series, coloured);
    // a bare plot names its coloured lines at their ends, or in a legend where it is narrow, and its
    // grey ones under the pointer; its colours are not chosen
    if (greys) figure.append(greys);
    if (bare && !narrow) {
      figure.querySelector(".legend")?.remove();
      return;
    }
    const focused = figure.contains(document.activeElement) ? document.activeElement.closest(".legend button")?.dataset.system : undefined;
    const fresh = legend(series, names, suite.instances.length, figure, suite.faster, coloured, narrow, bare ? null : (key) => {
      coloured = new Set(coloured);
      if (coloured.has(key)) coloured.delete(key);
      else coloured.add(key);
      draw();
    });
    const old = figure.querySelector(".legend");
    if (old) old.replaceWith(fresh);
    else figure.prepend(fresh, ...(every ? [every] : []));
    if (every) {
      every.textContent = coloured.size < series.length ? `Colour all ${series.length} lines`
        : `Colour ${and(top.map((key) => `${names[key]}'s`))} lines only`;
    }
    if (focused) fresh.querySelector(`button[data-system="${CSS.escape(focused)}"]`)?.focus();
  };
  onWidth(box, draw);
  return figure;
}

// ------------------------------------------------------------- the README's figures

// A category's instances (the data's rows), narrowed to `only`: each with
// its suite, its time limit, its systems' classes (a letter each: o own, n
// native, t translated, x translated but not idiomatic, u unread, - not
// given), each system's record `v` and its time
// `t`, null where it does not answer or was not given the instance.
function catRows(c, only) {
  const keep = narrowing(c, only);
  return c.rows.filter(keep).map((row) => ({ suite: row[0], name: row[1], limit: row[2], cls: row[3],
    v: Object.fromEntries(c.systems.map((x, j) => [x, row[4 + j]])),
    t: Object.fromEntries(c.systems.map((x, j) => [x, answered(row[4 + j]) ? row[4 + j] : null])) }));
}

// The rows a filter keeps: a family's (the program or model whose sizes
// they are), or the instances that are a system's own (own:SYSTEM).
function narrowing(c, only) {
  if (only === undefined) return () => true;
  if (only.startsWith("own:")) {
    const j = c.systems.indexOf(only.slice(4));
    return (row) => row[3][j] === "o";
  }
  const rows = new Set(c.groups.find((g) => g.key === only).rows);
  return (row, i) => rows.has(i);
}

// The filters a category offers: its families, then the systems that have
// own benchmarks among its instances.
function filtersOf(c) {
  return [...c.groups.map((g) => ({ key: g.key, title: g.title, n: g.rows.length })),
    ...c.systems.filter((x) => c.classes[x].own > 0)
      .map((x) => ({ key: `own:${x}`, title: `${data.names[x]}'s own`, n: c.classes[x].own }))];
}

// The family of a category (c.groups) whose key is `only`, else undefined.
const groupOf = (c, only) => c.groups.find((g) => g.key === only);

// A category's word for one of its instances.
const ONE = { programs: "program", queries: "query", networks: "network", circuits: "circuit" };

// The name of the program of family g of category c: the one file tddlog
// reads for every instance of it (benchmarks/index.json's `program`), else
// how many instances it has (62 networks).
function programName(c, g) {
  const names = new Set(g.rows.map((i) => familyOf(c.rows[i][0], c.rows[i][1])?.program ?? null));
  return names.size === 1 && !names.has(null) ? element("code", {}, [...names][0]) : plural(g.rows.length, ONE[c.noun] ?? "instance", c.noun);
}

// A table's one link of a family's row: the family's view on its type,
// which opens on its program, named after the program.
const programCell = (c, g) => element("td", { class: "program" }, link({ view: "category", category: c.key, only: g.key }, programName(c, g)));

// The families of rows (catRows'), each { c, g, rows }: the type and the
// family there of each (data.place; the circuits' where it is in no type), in
// the types' order and each type's; the rows of neither last, as { rows }.
function familiesOfRows(rows) {
  const out = new Map();
  for (const r of rows) {
    const key = `${r.suite}/${r.name}`;
    const at = data.place.get(key) ?? (data.circuit.has(key) ? { c: data.circuits, g: data.circuit.get(key) } : null);
    const id = at ? `${at.c.key}/${at.g.key}` : "";
    if (!out.has(id)) out.set(id, { ...at, rows: [] });
    out.get(id).rows.push(r);
  }
  const place = (f) => (f.c ? [f.c === data.circuits ? data.categories.length : data.categories.indexOf(f.c), f.c.groups.indexOf(f.g)] : [Infinity, 0]);
  return [...out.values()].sort((a, b) => place(a)[0] - place(b)[0] || place(a)[1] - place(b)[1]);
}

// Families ({ c, g, ... }) in their order under their headings: a type's
// where they are of several types, its families of no domain first; a
// domain's (the data's domains) over its families, which go together.
function headings(families) {
  const several = new Set(families.map((f) => f.c?.key)).size > 1;
  const domain = (f) => (f.g?.domain ? f.c.domains?.find((d) => d.key === f.g.domain) ?? null : null);
  const out = [];
  for (const f of families) {
    const last = out.at(-1)?.c;
    if (several && f.c && f.c !== last) out.push({ heading: f.c.title, level: "type", c: f.c });
    out.push(f);
  }
  // within a run of a type's families: those of no domain, then each domain's under its title
  const runs = [];
  for (const f of out) {
    if (f.heading || !runs.length || runs.at(-1).c !== f.c) runs.push({ c: f.c, items: [] });
    runs.at(-1).items.push(f);
  }
  return runs.flatMap(({ items }) => {
    const plain = items.filter((f) => f.heading || !domain(f));
    const ds = [...new Set(items.map(domain).filter(Boolean))];
    return [...plain, ...ds.flatMap((d) => [{ heading: d.title, level: "domain" }, ...items.filter((f) => domain(f) === d)])];
  });
}

// A heading row of a table of families (headings'), across its n columns.
const headingRow = (h, n) => element("tr", { class: `heading ${h.level}` }, element("th", { colspan: n }, h.heading));

const LETTERS = { o: "own", n: "native", t: "translated", u: "unread", x: "not idiomatic" };

// Each system's instances given, answers and classes over rows: the data's
// where rows are the whole category, else counted from the rows.
function tally(c, rows, only) {
  const out = {};
  c.systems.forEach((x, j) => {
    if (only === undefined) {
      out[x] = { given: c.given[x], answers: c.answers[x], classes: c.classes[x] };
      return;
    }
    const classes = Object.fromEntries(Object.values(LETTERS).map((k) => [k, 0]));
    for (const r of rows) if (r.cls[j] !== "-") classes[LETTERS[r.cls[j]]]++;
    out[x] = { given: rows.filter((r) => r.v[x] !== null && r.v[x] !== "unread").length, answers: rows.filter((r) => r.t[x] !== null).length, classes };
  });
  return out;
}

// The systems drawn: the category's, tddlog (and its pipelines) and the native
// systems given at least 20 of its instances; narrowed, those of tddlog and its
// pipelines and of the native systems given one of the rows.
function drawnOf(c, rows, only) {
  if (only === undefined) return c.drawn;
  return c.systems.filter((x) => (isMine(x) || c.native.includes(x)) && rows.some((r) => r.v[x] !== null));
}

// tddlog against system x by the measure (the processing's
// compare, the data's): over the category's rows (c.vs), or narrowed to
// `only` (c.narrowed), where x was given one of them; null otherwise.
const versus = (c, x, only) => (only === undefined ? c.vs[x] : c.narrowed[only]?.[x]) ?? null;

// The other systems by their answers, then tddlog's speed-up over them, the
// least first and a system without one last, as the data orders them.
function ranked(c, xs, tal, only) {
  const key = (x) => versus(c, x, only)?.speedup ?? Number.MAX_VALUE;
  return xs.filter((x) => !isMine(x)).sort((a, b) => tal[b].answers - tal[a].answers || key(a) - key(b));
}

// tddlog's speed-up of a comparison: 49×, 1.7×, 1.3× slower, or "nothing to
// measure" where no instance both answer over the floor and none only one
// answers; with sentence, "49× faster".
function speedup(v, sentence = false) {
  const r = v?.speedup ?? null;
  if (r === null) return "nothing to measure";
  return r >= 1 ? `${times(r, 1)}${sentence ? " faster" : ""}` : `${times(1, r)} slower`;
}

// A category's rows as one suite for a plot: the systems drawn, tddlog's (and
// its pipelines') first and then by their answers and speed-ups, as the
// README's legends list them; the limit the longest of its instances'; tddlog's speed-up over
// each other system in words (`faster`).
function catSuite(c, rows, drawn, tal, only) {
  const systems = [...drawn.filter(isMine), ...ranked(c, drawn, tal, only)];
  return { key: c.key, title: c.title, noun: c.noun, systems, instances: rows.map((r) => [r.name, ...systems.map((x) => r.v[x])]),
    limit: Math.max(...rows.map((r) => r.limit)),
    faster: Object.fromEntries(systems.filter((x) => !isMine(x)).map((x) => {
      const v = versus(c, x, only);
      return [x, (v?.speedup ?? null) === null ? "nothing to measure" : `${subject(v.me)} ${speedup(v, true)}`];
    })) };
}

// A comparison's two total times in words, after "over": the instances the
// system is given and either answers, a run that does not answer counted as
// twice its limit (PAR-2).
const totals = (v, name) => `the ${plural(v.either, "instance")} ${name} is given and either answers, ${name} takes `
  + `${duration(v.total)} and ${subject(v.me)} ${duration(v.tddlog_total)} (PAR-${data.measure.par})`;

// A category's headline, in cells: the answers of the rows' subjects (tal,
// tally's), and with several of tddlog and its pipelines each one's of the
// instances it was given (headMembers); the best native system's (the most
// answers, then the least speed-up; on all programs and the circuits the best other system's)
// of the instances it was given, and tddlog's speed-up over it on all of them,
// head-to-head and translated together, which the speed-up's label says by
// their number (a type's tables split them).
function headline(c, rows, tal, only) {
  const best = ranked(c, c.native.filter((x) => tal[x]?.given > 0), tal, only)[0];
  const v = best && versus(c, best, only);
  // the circuits are no type: no system is native to them, every system on them is another
  const which = c.key === ALL || c.key === APART ? "other" : "native";
  const given = best ? count(tal[best].given) : "";
  const box = cells([`${subject(subjectsOf(rows))} answers`, `${count(answeredBy(rows))} of ${count(rows.length)}`],
    ...headMembers(c, rows, tal).map(([x, label]) => [label, `${count(tal[x].answers)} of ${count(tal[x].given)}`, `${data.names[x]}'s answers of the instances it was given`]),
    ...(v ? [[`best ${which}: ${data.names[best]}`, `${count(tal[best].answers)} of ${given}`, `the ${which} system with the most answers, of the instances it was given`],
      [speedName(`speed-up over it, on all ${given}`), speedup(v),
        `${subject(v.me)}'s speed-up over ${data.names[best]} on every instance it was given, head-to-head and translated together`]] : []));
  box.classList.add("headline");
  return box;
}

// With several of tddlog and its pipelines in a category, the members its
// headline gives a cell of their own after the subjects' answers, each with
// its label: a pipeline's answers, and a subject's of those it reads where it
// does not read every instance; a subject given every one repeats the first
// cell and has none.
function headMembers(c, rows, tal) {
  const ms = c.systems.filter((x) => isMine(x) && tal[x].given > 0), subjects = subjectsOf(rows);
  if (ms.length < 2) return [];
  return ms.filter((x) => !subjects.includes(x) || tal[x].given < rows.length)
    .map((x) => [x, subjects.includes(x) ? `${data.names[x]} answers, of those it reads` : `${data.names[x]} answers`]);
}

// How the totals and the speed-up are counted, in one line under a table,
// from the data's parameters of the measure (data.measure,
// the processing's); How it works holds the whole definition.
const twice = (par) => (par === 2 ? "twice" : `${par} times`);
// When and on which commit of the source tree the data were processed
// (categories.json's stamp), the one data every headline is from: the
// README's quote the same.
function stampLine() {
  const { date, source } = data.stamp;
  return element("p", { class: "muted small stamp" }, `Results as of ${date}, public main `,
    element("a", { href: `https://github.com/Tractables/tddlog/commit/${source}` }, source), ".");
}

function measureLine() {
  const { floor, par } = data.measure;
  return `An unanswered run counts as ${twice(par)} its time limit (PAR-${par}) and a time under ${floor} s as ${floor} s.`;
}


// The README's overview: for each type of program, tddlog's answers and, for
// each of its five best native systems (c.named), the system's answers and
// the two total times over the instances it is given and either answers
// (PAR-2), on one time axis, and tddlog's speed-up over it; the other
// native systems in one row (c.others), the least of their totals and of
// their speed-ups. A type's title or row opens the type.
function overview() {
  const box = element("div", { class: "box" });
  const figure = element("figure", { class: "overview" }, box,
    element("figcaption", { class: "note" }, `${measureLine()} A type opens its details.`));
  onWidth(box, () => drawOverview(box));
  return figure;
}

function drawOverview(box) {
  const width = Math.max(300, Math.floor(box.clientWidth));
  const narrow = width < 640;
  // a type's title in as many lines as its width needs, about 7.5 px a letter at 13 px
  const titleLines = (text) => {
    const lines = [""];
    for (const word of text.split(" ")) {
      if (lines.at(-1) && (lines.at(-1).length + 1 + word.length) * 7.5 > width) lines.push(word);
      else lines[lines.length - 1] = lines.at(-1) ? `${lines.at(-1)} ${word}` : word;
    }
    return lines;
  };
  const groups = data.types.map((c) => {
    const ms = subjectsOf(catRows(c));
    const title = `${c.title}, ${count(c.instances)} ${c.noun}: ${subject(ms)} ${ms.length > 1 ? "answer" : "answers"} ${count(answeredBy(catRows(c)))}`;
    return { c, rows: c.named.map((x) => ({ x, v: c.vs[x] })), others: c.others, ms, lines: titleLines(title) };
  });
  const row = 23, top = 46, bottom = 30, head = (g) => 32 + 16 * (g.lines.length - 1);
  const nameRight = narrow ? 112 : 190, answersRight = nameRight + 38;
  const ratio = width - (narrow ? 56 : 110);
  const x0 = answersRight + 14, x1 = ratio - (narrow ? 10 : 18);
  const all = groups.flatMap((g) => [...g.rows.flatMap((r) => [r.v.tddlog_total, r.v.total]),
    ...(g.others.systems.length ? [g.others.total] : [])]).filter((t) => t > 0);
  const lo = Math.min(...all) / 3, hi = Math.max(...all) * 3;
  const lx = (t) => x0 + Math.log(Math.max(t, lo) / lo) / Math.log(hi / lo) * (x1 - x0);
  const height = top + groups.reduce((h, g) => h + head(g) + row * (g.rows.length + (g.others.systems.length ? 1 : 0)), 0) + bottom;
  const plot = svg("svg", { class: "overview", width, height, viewBox: `0 0 ${width} ${height}`, role: "group",
    "aria-label": "Each type of program: each native system's answers, its total time and tddlog's, and tddlog's speed-up" });
  const ticks = [];
  for (const tick of [[1, "1 s"], [10, "10 s"], [60, "1 min"], [600, "10 min"], [3600, "1 h"], [36000, "10 h"], [360000, "100 h"]]) {
    // each at least 44 px from the one before, so that their names do not touch
    if (tick[0] < lo || tick[0] > hi || (ticks.length && lx(tick[0]) - lx(ticks.at(-1)[0]) < 44)) continue;
    ticks.push(tick);
  }
  for (const [t, label] of ticks) {
    plot.append(svg("line", { class: "grid", x1: lx(t), y1: top - 6, x2: lx(t), y2: height - bottom + 4 }));
    // above, beside the answers' header, a name only where it clears it (about 6 px a letter at 11 px)
    const clear = lx(t) - label.length * 3 > answersRight + 6;
    for (const y of clear ? [top - 12, height - 10] : [height - 10]) plot.append(svg("text", { class: "tick", x: lx(t), y, "text-anchor": "middle" }, label));
  }
  plot.append(svg("text", { class: "col", x: answersRight, y: top - 12, "text-anchor": "end" }, "answers"));
  // the speed-up's header, a link to the methods table that defines it (speedName)
  const faster = svg("a", { href: `${front}#/methods`, class: "defined" });
  faster.append(svg("text", { class: "col", x: ratio, y: top - 25 }, "tddlog"), svg("text", { class: "col", x: ratio, y: top - 12 }, "faster"));
  plot.append(faster);
  let y = top;
  // A row: its name and answers, its marks, and the whole row a link.
  const line = (to, label, cls, name, answers, ...marks) => {
    const a = svg("a", { href: href(to), class: `row ${cls}`, "aria-label": label });
    a.append(svg("title", {}, label), svg("rect", { class: "hit", x: 0, y, width, height: row }),
      svg("text", { class: "name", x: nameRight, y: y + row / 2 + 4, "text-anchor": "end" }, name),
      ...(answers === null ? [] : [svg("text", { class: "answers", x: answersRight, y: y + row / 2 + 4, "text-anchor": "end" }, count(answers))]),
      ...marks);
    plot.append(a);
    y += row;
  };
  const at = (t, cls, side) => {
    const x = lx(t), cy = y + row / 2;
    const marks = [svg("circle", { class: cls, cx: x, cy, r: 5 })];
    if (!narrow && side) {
      marks.push(svg("text", { class: `t ${cls}`, x: x + side * 9, y: cy + 4, "text-anchor": side < 0 ? "end" : "start" }, duration(t)));
    }
    return marks;
  };
  // the right column: tddlog's speed-up r (with least, "at least"), or nothing to measure
  const mark = (r, least = false) => (r === null
    ? svg("text", { class: "ratio few", x: ratio, y: y + row / 2 + 4 }, narrow ? "none" : "nothing to measure")
    : svg("text", { class: `ratio${r >= 1 ? "" : " slower"}`, x: ratio, y: y + row / 2 + 4 },
      r >= 1 ? `${least ? "≥" : ""}${times(r, 1, least)}` : `${least ? "≤" : ""}${times(1, r)} slower`));
  for (const g of groups) {
    const { c, rows, others, ms, lines } = g;
    const to = { view: "category", category: c.key };
    const title = svg("a", { href: href(to), class: "kind" });
    // each line but the last keeps its space, so that the link's text is the title's
    lines.forEach((text, i) => title.append(svg("text", { x: 0, y: y + 23 + 16 * i }, i < lines.length - 1 ? `${text} ` : text)));
    plot.append(title);
    y += head(g);
    for (const { x, v } of rows) {
      const mine = v.tddlog_total, theirs = v.total;
      const [left, right] = mine <= theirs ? [-1, 1] : [1, -1];
      line({ ...to, vs: x }, `${data.names[x]}: ${count(c.answers[x])} of ${count(c.instances)} answered; over `
        + `${totals(v, data.names[x])}; ${subject(v.me)}'s speed-up: ${speedup(v)}`,
      "other", data.names[x], c.answers[x],
      svg("line", { class: "span", x1: lx(mine), y1: y + row / 2, x2: lx(theirs), y2: y + row / 2 }),
      ...at(theirs, `you ${tint(x)}`, right), ...at(mine, "me", left), mark(v.speedup));
    }
    if (others.systems.length) {
      const who = others.systems.map((x) => data.names[x]);
      const several = who.length > 1;
      // their names where they fit (about 6.5 px a letter), on hover in any case; a bound on
      // their totals only where each has an instance it or tddlog answers (a total above 0)
      const bound = others.total ? atLeast(others.total) : null;
      const tail = bound ? `${several ? "each more than " : ""}${bound} in total` : "";
      const short = bound ? `${several ? "each over " : ""}${bound}` : "";
      const text = [bound ? `${and(who)}: ${tail}` : and(who), tail, short].find((t) => t.length * 6.5 <= x1 - x0) ?? short;
      const r = others.speedup;
      line(to, `${and(who)}` + (bound ? `: ${several ? "each " : ""}more than ${bound} in total over the instances `
        + `${several ? "each" : "it"} is given and either answers (PAR-${data.measure.par})` : "")
        + (r === null ? "" : `; ${subject(ms)}'s speed-up ${r >= 1 ? `at least ${times(r, 1, true)}` : `at most ${times(1, r)} slower`}`),
      "others", several ? `${who.length} others` : who[0], null,
      svg("text", { class: "both", x: x0, y: y + row / 2 + 4 }, text), mark(r, true));
    }
  }
  box.replaceChildren(plot);
}

// tddlog's time against the fastest other system's (or `vs`'s), instance by
// instance, as the README's scatter plots draw it: on log axes from 0.2 s,
// a run that does not answer at "fail", the instances both answer in under
// 0.2 s and those neither answers left out. A dot under the diagonal is an
// instance tddlog answers faster; a dot opens its instance. The fastest
// other is of the systems `drawn`; with `choose`, a menu picks the other
// system.
function scatter(rows, drawn, vs, choose, key = myKey) {
  const box = element("div", { class: "box" });
  const head = element("p", { class: "sc-head" });
  const foot = element("p", { class: "note" });
  const figure = element("figure", { class: "scatter" });
  const others = drawn.filter((x) => !isMine(x));
  const me = subject(subjectsOf(rows, key));
  const mineOf = (r) => r.t[key(r)] ?? null;
  if (choose) {
    const menu = element("select", { class: "vs", "aria-label": `The system ${me}'s times are set against`,
      onchange: () => go({ ...now, vs: menu.value || undefined }) },
      element("option", { value: "" }, "the fastest other system"),
      ...[...others, ...(vs && !others.includes(vs) ? [vs] : [])].map((x) => element("option", { value: x }, data.names[x])));
    menu.value = vs ?? "";
    figure.append(element("label", { class: "pick" }, `${me} against `, menu));
  }
  figure.append(head, box, foot);
  const limit = Math.max(...rows.map((r) => r.limit));
  const against = vs ? [vs] : others;
  const fail = 10 ** (Math.log10(limit) + 0.32);
  const points = [];
  let fast = 0, neither = 0;
  for (const r of rows) {
    let best = null;
    for (const x of against) if (r.t[x] !== null && (best === null || r.t[x] < r.t[best])) best = x;
    const mine = mineOf(r), theirs = best === null ? null : r.t[best];
    if (mine === null && theirs === null) neither++;
    else if (mine !== null && theirs !== null && Math.max(mine, theirs) < FLOOR) fast++;
    else {
      const x = theirs === null ? fail : Math.max(theirs, FLOOR), y = mine === null ? fail : Math.max(mine, FLOOR);
      points.push({ r, best, x, y, cls: y < x ? "win" : y > x ? "loss" : "tie" });
    }
  }
  const wins = points.filter((p) => p.cls === "win").length, losses = points.filter((p) => p.cls === "loss").length;
  figure.dataset.faster = String(wins);
  figure.dataset.slower = String(losses);
  head.replaceChildren(element("b", { class: "win" }, `${me} faster on ${count(wins)}`), element("b", { class: "loss" }, `slower on ${count(losses)}`));
  const who = vs ? data.names[vs] : "the fastest other system";
  const left = [fast ? `${plural(fast, "instance")} both answer in under ${FLOOR} s` : "", neither ? `${count(neither)} neither answers` : ""].filter(Boolean);
  foot.replaceChildren(`A dot is an instance: ${me}'s time against ${who}'s; it opens the instance.${left.length ? ` Not drawn: ${and(left)}.` : ""}`);
  onWidth(box, () => drawScatter(box, points, limit, fail, vs, me, key));
  return figure;
}

function drawScatter(box, points, limit, fail, vs, me, key) {
  const who = vs ? data.names[vs] : "the fastest other system";
  const side = Math.min(400, Math.max(260, Math.floor(box.clientWidth)));
  const left = 48, bottom = 42, top = 6, right = 8;
  const x0 = left, x1 = side - right, y0 = top, w = x1 - x0;
  const lim = [FLOOR / 1.15, fail * 1.3];
  const scale = (v) => Math.log(v / lim[0]) / Math.log(lim[1] / lim[0]) * w;
  const px = (v) => x0 + scale(v), py = (v) => y0 + w - scale(v);
  const height = y0 + w + bottom;
  const plot = svg("svg", { class: "scatter", width: side, height, viewBox: `0 0 ${side} ${height}`, role: "img",
    "aria-label": `${me}'s time against ${who}'s, an instance a dot` });
  const edge = 10 ** (Math.log10(limit) + 0.14);
  plot.append(svg("rect", { class: "band", x: px(edge), y: y0, width: x1 - px(edge), height: w }),
    svg("rect", { class: "band", x: x0, y: y0, width: w, height: py(edge) - y0 }));
  const ticks = [FLOOR, ...[1, 10, 100].filter((t) => t < limit / 2), fail];
  ticks.forEach((t, i) => {
    const label = i === 0 ? `≤${FLOOR}` : i === ticks.length - 1 ? "fail" : String(t);
    plot.append(svg("line", { class: "grid", x1: px(t), y1: y0, x2: px(t), y2: y0 + w }),
      svg("line", { class: "grid", x1: x0, y1: py(t), x2: x1, y2: py(t) }),
      svg("text", { class: "tick", x: px(t), y: y0 + w + 15, "text-anchor": "middle" }, label),
      svg("text", { class: "tick", x: x0 - 5, y: py(t) + 4, "text-anchor": "end" }, label));
  });
  plot.append(svg("line", { class: "diag", x1: px(lim[0]), y1: py(lim[0]), x2: px(lim[1]), y2: py(lim[1]) }));
  for (const m of [10, 100]) {
    plot.append(svg("line", { class: "diag dashed", x1: px(lim[0] * m), y1: py(lim[0]), x2: px(lim[1]), y2: py(lim[1] / m) }));
    const t = limit / 4;
    plot.append(svg("text", { class: "tick", x: px(t), y: py(t / m) - 4, "text-anchor": "middle", transform: `rotate(-45 ${px(t)} ${py(t / m) - 4})` }, `${m}×`));
  }
  plot.append(svg("text", { class: "label", x: x0 + w / 2, y: height - 6, "text-anchor": "middle" }, `${vs ? who : "fastest other system"} (s)`),
    svg("text", { class: "label", transform: `translate(12 ${y0 + w / 2}) rotate(-90)`, "text-anchor": "middle" }, `${me} (s)`));
  for (const p of points) plot.append(svg("circle", { class: `dot ${p.cls}`, cx: px(p.x), cy: py(p.y), r: 3.5 }));
  // The hover layer: the dot nearest the pointer, ringed, its instance and
  // both times beside it; a click opens the instance.
  const ring = svg("circle", { class: "ring", r: 6, visibility: "hidden" });
  plot.append(ring);
  const tip = element("div", { class: "tip", hidden: true, "aria-live": "polite" });
  let near = null;
  const time = (t) => (t === null ? "no answer" : `${seconds(t)} s`);
  const pointer = (event) => {
    const at = plot.getBoundingClientRect();
    const mx = event.clientX - at.left, my = event.clientY - at.top;
    near = null;
    let d = 144;
    for (const p of points) {
      const e = (px(p.x) - mx) ** 2 + (py(p.y) - my) ** 2;
      if (e < d) [d, near] = [e, p];
    }
    plot.style.cursor = near ? "pointer" : "";
    if (!near) {
      ring.setAttribute("visibility", "hidden");
      tip.hidden = true;
      return;
    }
    ring.setAttribute("cx", px(near.x));
    ring.setAttribute("cy", py(near.y));
    ring.setAttribute("visibility", "visible");
    tip.replaceChildren(element("b", {}, near.r.name), element("span", {}, data.suites.get(near.r.suite).title),
      element("span", {}, `${data.names[key(near.r)]}: ${time(near.r.t[key(near.r)] ?? null)}`),
      element("span", {}, near.best !== null ? `${data.names[near.best]}: ${time(near.r.t[near.best])}` : vs ? `${who}: no answer` : "no other system answers"));
    tip.hidden = false;
    const tx = px(near.x) + 12 + tip.offsetWidth > side ? px(near.x) - 12 - tip.offsetWidth : px(near.x) + 12;
    tip.style.left = `${Math.max(0, tx)}px`;
    tip.style.top = `${Math.max(0, py(near.y) - 10)}px`;
  };
  plot.addEventListener("pointermove", pointer);
  plot.addEventListener("pointerleave", () => {
    ring.setAttribute("visibility", "hidden");
    tip.hidden = true;
  });
  plot.addEventListener("click", (event) => {
    pointer(event);
    if (near) location.hash = `#/${near.r.suite}/@${encodeURIComponent(near.r.name)}`;
  });
  box.replaceChildren(plot, tip);
}

// ------------------------------------------------------------- the data

const data = {
  version: "",      // tddlog's version in the release run
  names: {},        // a system's key: its name
  categories: [],   // categories.json's, in the README's order, each with `suites` and `perSuite` (its instances in each)
  measure: {},      // the parameters of the measure: floor, par, named, drawn
  own: [],          // each system on its own benchmarks against tddlog
  suiteVs: {},      // a suite's key: tddlog against each other system of the suite (categories.json's `suites`)
  systems: [],      // each solver, by the instances it answers: what it is, its totals, how it was given the instances
  integrations: null,  // tddlog alone, and X+tddlog against X on X's programs (categories.json's `integrations`)
  circuits: null,   // the circuits' entry, as a category's (categories.json's `circuits`), with `suites` and `perSuite`
  splits: {},       // a suite's key: each system's answered time on its export, {answers, seconds, export}
  stamp: null,      // categories.json's `stamp`: the data's date and public main's commit (stampLine)
  methods: [],      // categories.json's `methods`: the methods table's rows on the runs (next.js's methodsView)
  suites: new Map(),   // a suite's key: the suite, with `home`, the data's family that holds it, and `file`
  bench: null,      // benchmarks/index.json: every suite's families, where the site has them
  families: new Map(), // `${suite}/${family}`: the promise of that family's data
  family: new Map(),   // `${suite}/${instance}`: the instance's family
  place: new Map(),    // `${suite}/${instance}`: the category and the family of its instances (c.groups) that hold it
  circuit: new Map(),  // `${suite}/${instance}` of a circuit's readings: the circuits' family (data.circuits.groups) that holds it
  copyOf: new Map(),   // `${suite}/${instance}` of a copy (categories.json's `copies`): the instance it copies
};

async function load(file) {
  const response = await fetch(dataRoot + file + stamp);
  if (!response.ok) throw new Error(`${file}: ${response.status}`);
  return response.json();
}

// A family's data, loaded once.
function familyData(suite, key) {
  const id = `${suite}/${key}`;
  if (!data.families.has(id)) data.families.set(id, load(`benchmarks/${suite}/${key}.json`));
  return data.families.get(id);
}

const categoriesOf = (suite) => data.types.filter((c) => c.perSuite.has(suite));
const familiesOf = (suite) => data.bench?.suites?.[suite] ?? null;
const familyOf = (suite, name) => data.family.get(`${suite}/${name}`);
// The page of an instance: its family's, or, for a copy without a page of
// its own, the page of the instance it copies; else its suite's.
function instancePage(suite, name) {
  const f = familyOf(suite, name);
  if (f) return { view: "family", suite, family: f.key, instance: name };
  const of = data.copyOf.get(`${suite}/${name}`);
  if (of) {
    const [s, n] = of.split(/\/(.*)/);
    if (familyOf(s, n)) return instancePage(s, n);
  }
  return { view: "suite", suite };
}
// The column of a system in a suite's rows (its first is the instance).
const column = (suite, key) => suite.systems.indexOf(key) + 1;
// The record's system of a column: alea-process's runs are Alea.jl's, and an
// integration's are tddlog's where its records are tddlog's runs.
const recorded = (key) => data.bench?.records?.[key] ?? base(key);
// Whether a tab's reader is (a form of) the system of this name: "Ace auto"
// and "Ace forceC2d" are Ace's.
const readsAs = (name) => (reader) => reader === name || reader.startsWith(`${name} `);

// tddlog against each other system of a suite by the measure (the data's
// cats.suites, the processing's comparison over the instances both are given): by
// the system's answers, then tddlog's speed-up, the least first.
function against(suite) {
  const vs = data.suiteVs[suite.key] ?? {};
  const key = (x) => vs[x].speedup ?? Number.MAX_VALUE;
  return Object.keys(vs).sort((a, b) => vs[b].answers - vs[a].answers || key(a) - key(b)).map((x) => ({ key: x, v: vs[x] }));
}

// ------------------------------------------------------------- the address

// The kinds of problems of the page before this one, and the category each
// one's address opens now; and the categories the pages once had, with where
// their addresses go now, where the data no longer has them: the
// bit-blasted circuits are the circuits'.
const OLD_KINDS = { programs: "logic-programs", networks: "bayesian-networks", ppl: "functional-programs",
  circuits: "bit-blasted", databases: "database-queries" };
const GONE = { "bit-blasted": { view: "category", category: APART } };
const gone = (key) => (categoryOf(key) ? undefined : GONE[key]);

// The view an address names, or what it names that the data does not have.
function parse(hash) {
  const old = hash.replace(/^#/, "");
  const added = extend.parse(old);
  if (added) return added;
  if (old === "" || old === "/") return { view: "home" };
  // the pages before this one named a family of suites, or a kind, by its key
  if (!old.startsWith("/")) {
    if (OLD_KINDS[old]) return gone(OLD_KINDS[old]) ?? { view: "category", category: OLD_KINDS[old] };
    if (data.suites.has(old)) return { view: "suite", suite: old };
    return { view: "missing", what: old };
  }
  const parts = old.slice(1).split("/").filter(Boolean).map((part) => {
    try {
      return decodeURIComponent(part);
    } catch {
      return part;
    }
  });
  const missing = { view: "missing", what: parts.join("/") };
  if (parts[0] === "kind" && gone(OLD_KINDS[parts[1]])) return gone(OLD_KINDS[parts[1]]);
  if (parts[0] === "category" && gone(parts[1])) return gone(parts[1]);
  if (parts[0] === "kind") {
    const c = data.categories.find((x) => x.key === OLD_KINDS[parts[1]]);
    if (!c || parts.length > 3) return missing;
    return { view: "category", category: c.key, vs: c.systems.includes(parts[2]) && !isMine(parts[2]) ? parts[2] : undefined };
  }
  if (parts[0] === APART && parts.length === 1 && data.circuits) return { view: "category", category: APART };
  if (parts[0] === "category") {
    const c = categoryOf(parts[1]);
    if (!c) return missing;
    const route = { view: "category", category: c.key };
    for (let i = 2; i < parts.length; i += 2) {
      const [what, key] = [parts[i], parts[i + 1]];
      if (what === "in" && filtersOf(c).some((f) => f.key === key)) route.only = key;
      else if (what === "in" && (data.suites.has(key) || key === "uai2022")) continue;   // a suite, as the page once narrowed to
      else if (what === "vs" && c.systems.includes(key) && !isMine(key)) route.vs = key;
      else if (what === "program" && groupOf(c, route.only)) {
        // last: the family's instance on show (SUITE/NAME) and the file of its programs, as a family's address names it
        const m = parts.slice(i + 1).join("/").match(/^([^/]+)\/([^/]+)(?:\/tab\/(\d+)(?:\/(\d+))?)?$/);
        if (!m || !groupOf(c, route.only).rows.some((j) => c.rows[j][0] === m[1] && c.rows[j][1] === m[2])) return missing;
        route.program = [m[1], m[2]];
        if (m[3] !== undefined) route.file = [Number(m[3]), Number(m[4] ?? 0)];
        break;
      } else return missing;
    }
    return route;
  }
  if (parts[0] === "system") {
    const s = data.systems.find((x) => x.key === parts[1]);
    return s && parts.length === 2 ? { view: "system", system: s.key } : missing;
  }
  const suite = parts[0];
  if (!data.suites.has(suite)) return missing;
  if (parts.length === 1) return { view: "suite", suite };
  // FAMILY[/NAME] or @NAME, then what is open in the instance's view
  const named = parts[1].startsWith("@");
  // a copy without a page of its own opens on the instance it copies
  if (named && !familyOf(suite, parts[1].slice(1)) && instancePage(suite, parts[1].slice(1)).view === "family") {
    return instancePage(suite, parts[1].slice(1));
  }
  const fams = familiesOf(suite);
  if (!fams) return { view: "suite", suite };   // the site has no benchmarks' data
  const instance = named ? parts[1].slice(1) : parts[2];
  // (a family's former key, with an instance, opens the instance's family)
  const family = named ? familyOf(suite, instance)
    : fams.find((f) => f.key === parts[1]) ?? (instance === undefined ? undefined : familyOf(suite, instance));
  if (!family || (instance !== undefined && !family.instances.includes(instance))) return missing;
  const route = { view: "family", suite, family: family.key, instance: instance ?? family.instances[0] };
  const m = parts.slice(named ? 2 : 3).join("/").match(/^(?:tab\/(\d+)(?:\/(\d+))?)?(?:(?:^|\/)runs\/([^/]+))?$/);
  if (!m || (m[3] !== undefined && !data.suites.get(suite).systems.includes(m[3]))) return missing;
  if (m[1] !== undefined) route.file = [Number(m[1]), Number(m[2] ?? 0)];
  if (m[3] !== undefined) route.runs = m[3];
  return route;
}

function href(route) {
  switch (route.view) {
    // the circuits at the address of their section, a family of theirs as a category's
    case "category": return route.category === APART && !route.only && !route.vs ? `#/${APART}`
      : `#/category/${route.category}${route.only ? `/in/${encodeURIComponent(route.only)}` : ""}`
      + `${route.vs ? `/vs/${encodeURIComponent(route.vs)}` : ""}`
      + `${route.only && route.program ? `/program/${route.program.map(encodeURIComponent).join("/")}${tabPart(route.file)}` : ""}`;
    case "system": return `${front}#/${isMine(route.system) ? `integrations/${integrationOf(route.system)}` : `baselines/${encodeURIComponent(route.system)}`}`
      + `${route.at ? `/${route.at}` : ""}`;
    case "suite": return `#/${route.suite}`;
    case "family": {
      let out = `#/${route.suite}/${encodeURIComponent(route.family)}${route.instance ? `/${encodeURIComponent(route.instance)}` : ""}`;
      out += tabPart(route.file);
      if (route.runs) out += `/runs/${encodeURIComponent(route.runs)}`;
      return out;
    }
    case "missing": return `#/${route.what}`;
    case "section": return `${front}#/${route.key}`;
    default: return extend.href(route) ?? "#/";
  }
}

// An address's part for the file on show of an instance's programs.
const tabPart = (file) => (file ? `/tab/${file[0]}${file[1] ? `/${file[1]}` : ""}` : "");

// A view without what is open in it: the file on show, the drawer, a
// category's filter, the system its times are set against and the instance
// of its family on show, and the section of a page a link scrolls to.
const pageOf = (route) => ({ ...route, file: undefined, runs: undefined, vs: undefined, only: undefined, program: undefined, at: undefined });

function link(route, ...children) {
  return element("a", { href: href(route) }, ...children);
}

// The category a view of a suite belongs to: the one the reader came from,
// where it holds the suite, else the one that holds most of its instances
// (the literature suite's are logic programs, database queries and networks).
let lastCategory = null;
function categoryFor(suite) {
  const cs = categoriesOf(suite);
  return cs.find((c) => c.key === lastCategory) ?? cs.sort((a, b) => b.perSuite.get(suite) - a.perSuite.get(suite))[0];
}

// Where a suite's views sit in the crumbs: under its category, or, for a
// suite in no type, under the circuits where it holds circuits (an
// evaluation of their own), else under the integration whose rows hold it.
function suiteHome(suite, c) {
  if (c) return [inSection("types"), [c.title, { view: "category", category: c.key }]];
  if (data.circuits?.perSuite.has(suite)) return [[data.circuits.title, { view: "category", category: APART }]];
  const k = integrationsOf().find((key) => data.integrations[key].rows?.some((row) => row[0] === suite));
  if (k === undefined) throw new Error(`the suite ${suite} is in no type and no integration`);
  const g = data.integrations[k];
  return [inSection("integrations"), [g.title, { view: "system", system: g.pipeline }]];
}

function crumbs(...items) {
  const nav = element("nav", { class: "crumbs", "aria-label": "Where this is" });
  items.forEach(([text, route], i) => {
    if (i) nav.append(element("span", { "aria-hidden": "true" }, "›"));
    nav.append(route ? link(route, text) : element("span", { "aria-current": "page" }, text));
  });
  return nav;
}

// ------------------------------------------------------------- the views

// By program type, the results page's opening view (#/ here, #/types on
// the front page): the README's figures, each part a link to its details:
// each type's native systems against tddlog (total times and speed-ups), then for each
// category its headline, its cactus plot beside tddlog's times against the
// fastest other system's, and each system on its own benchmarks.
function home() {
  const out = [crumbs(HOME, [sectionName("types")]), element("h2", {}, sectionName("types")),
    element("p", { class: "lede" }, "Each instance counts in one type, that of the model it is; all programs is their union, with every system. ",
      link({ view: "section", key: "coverage" }, COVERAGE), ": what the systems' own sources hold that tddlog does not run, and why."),
    element("div", { class: "fig-head" }, element("h3", {}, "Each type of program against the systems native to it"),
      element("span", { class: "dots" }, element("i", { class: "me" }), subject(subjectsOf(data.types.flatMap((c) => catRows(c)))),
        element("i", { class: "you" }), "a native system")),
  overview(), stampLine()];
  for (const c of data.categories) {
    const to = { view: "category", category: c.key };
    const rows = catRows(c);
    const tal = tally(c, rows);
    const others = c.key === ALL ? plural(c.systems.length - 1, "other system") : plural(c.native.length, "native system");
    out.push(element("section", { class: "kind-figs" },
      element("div", { class: "fig-head" }, element("h3", {}, link(to, c.title)),
        element("span", { class: "n" }, `${count(c.instances)} ${c.noun}, ${others}`),
        link(to, c.key === ALL ? "Every system and type" : "Every native system and family")),
      headline(c, rows, tal),
      element("div", { class: "pair" }, plotted(catSuite(c, rows, c.drawn, tal), data.names, { bare: true, colour: [...c.mine, ...c.named] }),
        scatter(rows, c.drawn))));
  }
  out.push(...ownTable(), note(measureLine()), raw("categories.json"));
  return out;
}

// Each system on its own benchmarks (from its papers, benchmarks or
// distribution) against tddlog on the same instances (o.vs): each
// one's answers and total time over the instances either answers, and
// tddlog's speed-up; a system opens the category that holds most of them,
// narrowed to them. `list` is the rows shown (all of them, or one
// system's), `head` their heading. How the totals are counted is the
// caller's to say, once on its page.
function ownTable(list = data.own, head = "Each system on its own benchmarks") {
  const body = element("tbody");
  for (const o of list) {
    const cs = data.types.filter((c) => c.classes[o.system]?.own > 0)
      .sort((a, b) => b.classes[o.system].own - a.classes[o.system].own);
    const name = cs.length ? link({ view: "category", category: cs[0].key, only: `own:${o.system}` }, data.names[o.system]) : data.names[o.system];
    body.append(element("tr", {}, element("th", { scope: "row" }, swatch(o.system), name), prose("td", o.about, { class: "about" }),
      element("td", { class: "num" }, count(o.instances)),
      element("td", { class: "num" }, count(o.vs.answers)), element("td", { class: "num" }, duration(o.vs.total)),
      element("td", { class: "num" }, o.vs.me.every((x) => x === "tddlog") ? "" : element("span", { class: "muted" }, `${subject(o.vs.me)} `),
        count(o.vs.tddlog_answers)),
      element("td", { class: "num" }, duration(o.vs.tddlog_total)), element("td", { class: "num" }, speedup(o.vs))));
  }
  const par = `over the instances either answers (PAR-${data.measure.par})`;
  return [element("h3", { class: "own-head" }, head), scroller(stacked(element("table", { class: "board own" },
    head2(["system"], ["its benchmarks"], ["instances", null, "num"],
      { group: "the system", cols: [["answers", null, "num"], ["total", par, "num"]] },
      { group: "tddlog", cols: [["answers", null, "num"], ["total", par, "num"], [speedName("faster"), "tddlog's speed-up", "num"]] }), body)), "whole")];
}

// A category's systems by their answers, then tddlog's speed-up over them,
// against tddlog, with how each was given the instances and how
// many it was not given, which together are all of them; `vs`'s row marked;
// a system's name and its numbers open its page (tddlog's, tddlog's own).
function scoreboard(c, rows, tal, n, vs, only) {
  const me = subject(subjectsOf(rows));
  const order = [...c.systems.filter((x) => isMine(x) && tal[x].given > 0), ...ranked(c, c.native.filter((x) => tal[x].given > 0), tal, only),
    ...(vs && !c.native.includes(vs) && tal[vs] ? [vs] : [])];
  const body = element("tbody");
  for (const x of order) {
    const t = tal[x];
    const v = isMine(x) ? null : versus(c, x, only);
    const page = { view: "system", system: x };
    const cls = (k) => (!t.classes[k] ? "" : k === "own" && data.own.some((o) => o.system === x)
      ? link({ ...page, at: "original" }, count(t.classes[k])) : count(t.classes[k]));
    const missing = rows.filter((r) => r.v[x] === null).length;
    body.append(element("tr", { class: isMine(x) ? "mine" : x === vs ? "vs" : null },
      element("th", { scope: "row" }, swatch(x), link(page, data.names[x])),
      element("td", {}, element("span", { class: "bar" }, element("i", { style: `width:${(100 * t.answers / Math.max(1, n)).toFixed(1)}%` })),
        ` ${count(t.answers)}`),
      element("td", { class: "num" }, v ? duration(v.total) : ""),
      element("td", { class: "num" }, v ? duration(v.tddlog_total) : ""),
      element("td", { class: v && (v.speedup ?? null) === null ? "num few" : "num" }, v ? speedup(v) : ""),
      element("td", { class: "num" }, cls("own")), element("td", { class: "num" }, cls("native")),
      element("td", { class: "num" }, cls("translated")), element("td", { class: "num" }, cls("not idiomatic")),
      element("td", { class: "num" }, missing ? count(missing) : "")));
  }
  return scroller(stacked(element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col" }, `answers, of ${count(n)}`),
      element("th", { scope: "col", class: "num", title: `over the instances the system is given and either it or ${me} answers (PAR-${data.measure.par})` }, "its total time"),
      element("th", { scope: "col", class: "num", title: `over the same instances (PAR-${data.measure.par})` }, `${me}'s total time`),
      element("th", { scope: "col", class: "num" }, speedName(`${me} faster`)),
      element("th", { scope: "col", class: "num", title: "instances from the system's own papers, benchmarks or distribution" }, "own"),
      element("th", { scope: "col", class: "num", title: "instances whose program is in the system's own language" }, "native"),
      element("th", { scope: "col", class: "num", title: "instances whose program a translator wrote, as its users would write it" }, "translated"),
      element("th", { scope: "col", class: "num", title: `instances translated as its users would not write them: ${only !== undefined || c.key === ALL ? "in these totals" : "in no total"}` }, "not idiomatic"),
      element("th", { scope: "col", class: "num", title: "instances the system was not given, unanswered in its count" }, "not given"))),
    body)), "whole");
}

function categoryView(route) {
  const c = categoryOf(route.category);
  lastCategory = c.key;
  const only = route.only;
  const g = groupOf(c, only);
  const rows = catRows(c, only);
  const tal = tally(c, rows, only);
  const drawn = drawnOf(c, rows, only);
  const whole = catSuite(c, rows, drawn, tal, only);
  // the circuits' entry is their section's page (#/circuits): one crumb for both
  const out = [crumbs(HOME, ...(c.key === APART ? [] : [inSection("types")]), g ? [c.title, { view: "category", category: c.key }] : [c.title], ...(g ? [[g.title]] : [])),
    element("h2", {}, g ? g.title : c.title)];
  // a family: what it is and where it comes from
  if (g && !g.source) out.push(prose("p", g.about, { class: "lede" }), note("The type on its own, against the systems native to it: ",
    link({ view: "category", category: g.key }, g.title), "."));
  else if (g) out.push(prose("p", g.about, { class: "lede" }), note(`From ${g.source}.`), programSection(c, g, route));
  // the families' table, last, and a way to it from the top: each family's program is a click away
  const famHead = g ? null : element("h3", {}, c.key === ALL ? "Types of program" : "Families");
  out.push(headline(c, rows, tal, only));
  if (famHead && c.key !== ALL) {
    out.push(element("p", { class: "jump" }, element("button", { type: "button", class: "plain",
      onclick: () => famHead.scrollIntoView({ behavior: "smooth" }) }, `Its ${plural(c.groups.length, "family", "families")}, each with its program ↓`)));
  }
  out.push(...extend.category(c, route, rows));
  const filters = filtersOf(c);
  const chips = filters.filter((f) => f.key === only || f.key.startsWith("own:"));
  if (chips.length) {
    const chip = (key, text, n) => {
      const a = link({ view: "category", category: c.key, only: key, vs: route.vs }, text, element("span", { class: "n" }, count(n)));
      if (key === only) a.setAttribute("aria-current", "true");
      return a;
    };
    out.push(element("nav", { class: "filters", "aria-label": "Narrow the category to a family or to a system's own benchmarks" },
      chip(undefined, "All", c.instances), ...chips.map((f) => chip(f.key, f.title, f.n))));
  }
  const hidden = c.native.filter((x) => tal[x].given > 0 && !drawn.includes(x));
  const foreign = c.systems.filter((x) => !isMine(x) && !c.native.includes(x) && tal[x].given > 0);
  const me = subject(subjectsOf(rows));
  out.push(element("h3", {}, c.key === ALL || c.key === APART ? `Each system's answers, and its total time against ${me}'s`
    : `The native systems' answers, and their total times against ${me}'s`), scoreboard(c, rows, tal, rows.length, route.vs, only),
    note(measureLine()),
    ...(hidden.length ? [note(`Not on the plots, given fewer than ${data.measure.drawn} of the ${c.noun}: `
      + `${and(hidden.map((x) => `${data.names[x]} (${count(tal[x].given)})`))}.`)] : []),
    ...(foreign.length ? [note(`Given some of these ${c.noun} through a translation, but not native to them: `,
      `${and(foreign.map((x) => data.names[x]))}; `, link({ view: "category", category: ALL }, "all programs"), " has every system.")] : []),
    element("h3", { id: "figures" }, `Instances answered within each time limit, and ${me}'s times against another system's`),
    element("div", { class: "pair" }, plotted(whole, data.names, { also: route.vs, colour: [...c.mine, ...c.named] }), scatter(rows, drawn, route.vs, true)),
    note(`Answers count over all ${count(rows.length)} ${c.noun}${only ? ` of ${filters.find((f) => f.key === only).title}` : ""}; `
      + "one not given counts as unanswered."
      + (Object.keys(c.copies).length ? ` ${plural(Object.values(c.copies).flat().length, "copy", "copies")} of instances of other suites count once.` : "")));
  // Each family of the instances shown, under its domain where it has one:
  // its one link, its program's, which opens its view on it; tddlog's
  // answers, and the most of another system. All programs' rows are the
  // types, each a link to its view.
  if (!g) {
    const all = only === undefined ? rows : catRows(c);
    const keep = narrowing(c, only);
    const body = element("tbody");
    const cols = c.key === ALL ? 5 : 6;
    const fams = c.groups.map((fam) => ({ c, g: fam, rows: fam.rows.filter((i) => keep(c.rows[i], i)).map((i) => all[i]) }))
      .filter((f) => f.rows.length);
    for (const f of headings(fams)) {
      if (f.heading) {
        body.append(headingRow(f, cols));
        continue;
      }
      const { g: fam, rows: mine } = f;
      let best = null;
      for (const x of c.systems) {
        if (isMine(x)) continue;
        const a = mine.filter((r) => r.t[x] !== null).length;
        if (best === null || a > best[1]) best = [x, a];
      }
      const name = c.key === ALL ? link({ view: "category", category: c.key, only: fam.key }, fam.title) : fam.title;
      body.append(element("tr", {}, element("th", { scope: "row", title: fam.about ?? null }, name), ...(c.key === ALL ? [] : [programCell(c, fam)]),
        element("td", { class: "num" }, count(mine.length)), element("td", { class: "num" }, count(mine.filter((r) => myTime(r) !== null).length)),
        element("td", {}, best?.[1] ? `${data.names[best[0]]}, ${count(best[1])}` : "none"), prose("td", fam.source, { class: "about" })));
    }
    out.push(famHead, scroller(stacked(element("table", { class: "board families" },
      element("thead", {}, element("tr", {}, element("th", { scope: "col" }, c.key === ALL ? "type" : "family"),
        ...(c.key === ALL ? [] : [element("th", { scope: "col", title: "Opens the family's view on its program" }, "program")]),
        element("th", { scope: "col", class: "num" }, "instances"),
        element("th", { scope: "col", class: "num" }, `${me} answers`), element("th", { scope: "col" }, "the most answers of another system"),
        element("th", { scope: "col" }, "from"))), body))));
  }
  if (c.losses?.length) {
    out.push(element("h3", {}, `Where ${me} is slower`), element("ul", { class: "losses" }, ...c.losses.map((text) => prose("li", text))));
  }
  // The data's family of suites the category's first suite is in: the
  // findings that are no suite's are the category's where the category
  // starts with the family's first suite.
  const family = data.suites.get(c.rows[0][0]).home;
  const general = c.key !== ALL && family.suites[0].key === c.rows[0][0] ? (family.findings ?? []).filter((card) => !card.suite) : [];
  if (general.length) out.push(element("h3", {}, "Findings"), findings(general));
  out.push(setup(family), raw("categories.json"));
  return out;
}

// A baseline's parts, which the front page's page of it shows after tddlog
// against it over its idiomatic programs: `types`, in each category it was
// given, its answers, its total time, tddlog's and tddlog's speed-up, and a
// fold of its plot among the others there; `own`, its own benchmarks; and
// `facts`, its version, settings, what it was not given, how it is
// compared, its paper and its link.
function solverParts(x) {
  const s = data.systems.find((y) => y.key === x);
  const name = data.names[x];
  const parts = data.categories.filter((c) => s.categories.includes(c.key)).map((c) => {
    const rows = catRows(c);
    return { c, rows, tal: tally(c, rows) };
  });
  // the members of tddlog and its pipelines it is compared with over these
  // categories: each row's subject
  const all = parts.flatMap((p) => p.rows);
  const ms = subjectsOf(all);
  const par = `over the instances it is given and either answers (PAR-${data.measure.par})`;
  const sides = (me) => [{ group: name, cols: [["answers", null, "num"], ["total", par, "num"]] },
    { group: subject(me), cols: [["answers", null, "num"], ["total", par, "num"], [speedName("faster"), `${subject(me)}'s speed-up`, "num"]] }];
  // each category's numbers, then its plot among the others, x's line and tddlog's coloured
  const body = element("tbody");
  for (const { c, tal } of parts) {
    const t = tal[x], v = c.vs[x];
    const k = (key) => count(t.classes[key]);
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "category", category: c.key, vs: x }, c.title)),
      element("td", { class: "num" }, count(c.instances)), element("td", { class: "num" }, count(t.given)),
      ...["own", "native", "translated", "not idiomatic"].map((key) => element("td", { class: "num" }, k(key))),
      element("td", { class: "num" }, count(v.answers)), element("td", { class: "num" }, duration(v.total)),
      element("td", { class: "num" }, count(v.tddlog_answers)), element("td", { class: "num" }, duration(v.tddlog_total)),
      element("td", { class: (v.speedup ?? null) === null ? "num few" : "num" }, speedup(v))));
  }
  const cms = subjectsOf(all);
  const types = [scroller(stacked(element("table", { class: "board per-type" },
    head2(["type"], ["instances", null, "num"],
      { group: "given", cols: [["all", null, "num"], ["own", "from its own papers, benchmarks or distribution", "num"],
        ["native", "in its own language", "num"], ["translated", "translated for it as its users would write it", "num"],
        ["not idiomatic", "translated for it as its users would not write it: in no total", "num"]] },
      ...sides(cms)),
    body)), "whole"),
  lazy({ class: "cat-figs-fold" }, `Its plot in each type, among the other systems there`, () => [element("div", { class: "cat-figs" },
    ...parts.map(({ c, rows, tal }) => element("section", {}, element("h4", {}, link({ view: "category", category: c.key, vs: x }, c.title)),
      plotted(catSuite(c, rows, [...new Set([...c.drawn, x])], tal), data.names, { colour: [...c.mine, x] }))))])];
  const ownList = data.own.filter((o) => o.system === x);
  const own = ownList.length ? ownTable(ownList, `Against ${subject([...new Set(ownList.flatMap((o) => o.vs.me))])} on them`) : [];
  // version, settings, what it was not given, and how it is compared
  const short = parts.filter((p) => p.tal[x].given < p.c.instances)
    .map((p) => `${p.c.title}, ${count(p.c.instances - p.tal[x].given)} of ${count(p.c.instances)}`);
  const facts = element("dl", {}, element("dt", {}, "Version"), prose("dd", s.version), element("dt", {}, "Settings"), prose("dd", s.settings));
  if (short.length || s.notgiven) {
    facts.append(element("dt", {}, "Not given"), prose("dd", `${short.length ? `${and(short)}. ` : ""}${s.notgiven}`.trim()));
  }
  facts.append(element("dt", {}, "Compared"), element("dd", {}, `Against ${subject(ms)}.`));
  if (s.cite) {
    const c = s.cite;
    facts.append(element("dt", {}, "Paper"), element("dd", {}, `${c.authors}. `,
      c.doi ? element("a", { href: `https://doi.org/${c.doi}` }, c.title) : c.title, `. ${c.venue}, ${c.year}.`));
  }
  facts.append(element("dt", {}, "Site"), element("dd", {}, element("a", { href: s.link }, s.link)));
  return { s, types, own, facts };
}

function findings(list) {
  const box = element("div", { class: "findings" });
  for (const card of list) {
    box.append(element("details", { class: "finding" }, element("summary", {}, card.title),
      ...card.paras.map((para) => prose("p", para))));
  }
  return box;
}

// How the runs were made: the methods table (next.js's), one line under a
// category, not the suites' own paragraphs again.
function setup(home) {
  return note(element("a", { href: `${front}#/methods` }, "How the runs were made"), "; every table and its details: ",
    element("a", { href: home.docs }, "docs/benchmarks.md"), ".");
}

function suiteView(route) {
  const s = data.suites.get(route.suite);
  const c = categoryFor(s.key);
  const out = [crumbs(HOME, ...suiteHome(s.key, c), [s.title]),
    element("h2", {}, s.title), brief(s.about, "lede"),
    element("h3", {}, "Instances answered within each time limit"), plotted(s, data.names)];
  const fams = familiesOf(s.key);
  if (fams) {
    out.push(element("h3", {}, "Families"), note("Each cell counts the family's instances a system answers; a family opens to its programs."),
      familyTable(s, fams));
  }
  const rows = against(s);
  if (rows.length) {
    const paras = new Map();
    for (const card of s.home.cards.filter((c) => c.suite === s.key)) {
      for (const para of card.paras) {
        const m = para.match(/^\*\*Against ([^:*(]+?)(?: \(([^)]*)\))?:\*\*/);
        if (m) paras.set(m[1] + (m[2] ? ` (${m[2]})` : ""), para);
      }
    }
    out.push(element("h3", {}, `${subject(rowsMe(rows))} against each system`), headToHead(s, rows, paras));
  }
  for (const card of (s.home.findings ?? []).filter((c) => c.suite === s.key)) {
    out.push(element("h3", {}, card.title), ...card.paras.map((para) => prose("p", para, { class: "finding-text" })));
  }
  out.push(lazy({ class: "instances" }, `Every instance's times (${count(s.instances.length)})`, () => table(s)), raw(s.file));
  return out;
}

// Each family's instances a system answers, a column a system.
function familyTable(s, fams) {
  const cols = s.systems.filter((x) => !dashed(x));
  const rows = new Map(s.instances.map((row) => [row[0], row]));
  const head = element("tr", {}, element("th", { scope: "col" }, "family"),
    ...cols.map((x) => element("th", { scope: "col", class: `sys${isMine(x) ? " mine" : ""}` }, element("span", {}, data.names[x]))));
  const body = element("tbody");
  for (const f of fams) {
    const tr = element("tr", {}, element("th", { scope: "row" }, link({ view: "family", suite: s.key, family: f.key }, f.title),
      element("span", { class: "n" }, ` ${f.instances.length}`)));
    for (const x of cols) {
      const j = column(s, x);
      const given = f.instances.filter((name) => rows.get(name)?.[j] !== null).length;
      const a = f.instances.filter((name) => answered(rows.get(name)?.[j])).length;
      tr.append(given === 0 ? element("td", { class: "none", title: `${data.names[x]}: not given` })
        : element("td", { class: isMine(x) ? "mine" : null, style: `--p:${(a / f.instances.length).toFixed(3)}`,
          title: `${data.names[x]}: answers ${a} of the ${f.instances.length}` }, String(a)));
    }
    body.append(tr);
  }
  return scroller(element("table", { class: "grid counts" }, element("thead", {}, head), body));
}

// The members of tddlog and its pipelines a suite's rows (against) compare with.
const rowsMe = (rows) => [...new Set(rows.flatMap(({ v }) => v.me))];

function headToHead(s, rows, paras) {
  const me = subject(rowsMe(rows));
  const body = element("tbody");
  for (const { key, v } of rows) {
    const name = data.names[key];
    const para = paras.get(name) ?? [...paras].find(([k]) => k.startsWith(`${data.names[base(key)]} (`) && dashed(key) === /start-up/.test(k) && !/without/.test(k))?.[1]
      ?? [...paras].find(([k]) => k.startsWith(`${name} (`))?.[1];
    const tr = element("tr", {},
      element("th", { scope: "row" }, swatch(key), name),
      element("td", { class: "num" }, count(v.given)),
      element("td", { class: "num" }, `${count(v.tddlog_answers)} / ${count(v.answers)}`),
      element("td", { class: "num" }, `${duration(v.tddlog_total)} / ${duration(v.total)}`),
      element("td", { class: "num ratio" }, speedup(v, true)));
    body.append(tr);
    if (para) {
      const more = element("tr", { class: "words", hidden: true }, element("td", { colspan: "5" }, prose("p", para)));
      const open = element("button", { type: "button", class: "plain", "aria-expanded": "false" }, "In words");
      open.addEventListener("click", () => {
        more.hidden = !more.hidden;
        open.setAttribute("aria-expanded", String(!more.hidden));
      });
      tr.lastChild.append(" ", open);
      body.append(more);
    }
  }
  return element("div", {}, scroller(stacked(element("table", { class: "board h2h" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col", class: "num" }, "both given"),
      element("th", { scope: "col", class: "num" }, `answers, ${me} / it`),
      element("th", { scope: "col", class: "num" }, `total time, ${me} / it`),
      element("th", { scope: "col", class: "num" }, me))), body)), "whole"),
    note(`Over the instances both are given. ${measureLine()}`));
}

// Every instance's times, sortable by a column; an instance's name opens
// its family.
function table(suite) {
  const cols = suite.systems;
  const quick = (row) => {
    const ran = row.slice(1).filter((v) => v !== null && v !== "skipped");
    return ran.length > 0 && ran.every((v) => answered(v) && v < FLOOR);
  };
  const nquick = suite.instances.filter(quick).length;
  const head = element("tr", {}, ...["instance", ...cols.map((key) => data.names[key])].map((name, col) =>
    element("th", { scope: "col", class: col ? "num" : null, "data-col": String(col) }, element("button", { type: "button", class: "bare" }, name))));
  const body = element("tbody");
  const seen = new Set();
  const rowOf = (row) => {
    const fastest = fastestOf(cols.map((key, i) => (dashed(key) ? null : row[i + 1])));
    const page = instancePage(suite.key, row[0]);
    const tr = element("tr", {}, element("th", { scope: "row" }, page.view === "family" ? link(page, row[0]) : row[0]));
    if (quick(row)) tr.className = "quick";
    cols.forEach((key, i) => {
      const v = row[i + 1];
      if (v === null) {
        tr.append(element("td", { class: "num none", title: "not given" }, ""));
        return;
      }
      const { cls, p, slower } = rated(v, fastest);
      let text, title;
      if (answered(v)) [text, title] = [seconds(v), slower ? `${slower} the fastest's time` : null];
      else {
        seen.add(v);
        const [t, meaning] = FAILURES[v] ?? [() => v, v];
        [text, title] = [t(suite.limit), meaning];
      }
      tr.append(element("td", { class: `num ${cls}`, style: p ? `--p:${p.toFixed(3)}` : null, title }, text));
    });
    return tr;
  };
  const rows = suite.instances.map((row) => [row, rowOf(row)]);
  body.append(...rows.map(([, tr]) => tr));
  const grid = element("table", { class: "times" }, element("thead", {}, head), body);

  // A header's button sorts by its column, and again the other way: the
  // instances by name, the times with the failures after the answers.
  let sorted = null;
  head.addEventListener("click", (event) => {
    const th = event.target.closest("th");
    if (th === null) return;
    const col = Number(th.dataset.col);
    const dir = sorted?.[0] === col ? -sorted[1] : 1;
    sorted = [col, dir];
    const key = (row) => (answered(row[col]) ? row[col] : Infinity);
    const order = [...rows].sort(([a], [b]) => dir * (col === 0 ? a[0].localeCompare(b[0], "en", { numeric: true }) : (key(a) - key(b)) || 0));
    for (const h of head.children) h.removeAttribute("aria-sort");
    th.setAttribute("aria-sort", dir > 0 ? "ascending" : "descending");
    body.append(...order.map(([, tr]) => tr));
  });

  const hide = element("input", { type: "checkbox" });
  hide.checked = nquick > 0;
  const show = () => grid.classList.toggle("hide-quick", hide.checked);
  hide.addEventListener("change", show);
  show();
  const parts = [];
  if (nquick) {
    parts.push(element("label", { class: "check" }, hide,
      ` Hide the ${count(nquick)} instances every system that ran finishes in under ${FLOOR} s`));
  }
  parts.push(scroller(grid), element("p", { class: "key" }, ...shades(seen.has("unsupported")),
    ...Object.keys(FAILURES).filter((v) => v !== "unsupported" && seen.has(v)).map((v) => failureKey(v, suite.limit)),
    ...(suite.instances.some((row) => row.slice(1).includes(null)) ? [element("span", {}, "empty: not given")] : [])));
  const notes = ["Seconds; a header sorts by its column."];
  if (familiesOf(suite.key)) notes.push("An instance's name opens its family, with its programs.");
  parts.push(note(notes.join(" ")));
  return parts;
}

// ------------------------------------------------------------- a family

// The part of an instance's name that tells it from the others of its
// family: after the family's common prefix, up to its last underscore.
function shortNames(names) {
  if (names.length < 2) return new Map(names.map((n) => [n, n]));
  let p = names[0];
  for (const n of names) while (!n.startsWith(p)) p = p.slice(0, -1);
  const cut = p.lastIndexOf("_") + 1;
  return new Map(names.map((n) => [n, n.slice(cut) || n]));
}

function familyView(route, view) {
  const s = data.suites.get(route.suite);
  // the category and the family of its instances that hold the instance on show
  const at = data.place.get(`${s.key}/${route.instance}`);
  const c = at?.c ?? categoryFor(s.key);
  if (c) lastCategory = c.key;
  const f = familiesOf(s.key).find((x) => x.key === route.family);
  const short = shortNames(f.instances);
  const rows = new Map(s.instances.map((row) => [row[0], row]));
  const page = href(pageOf(route));
  // the family's last crumb is its view on its type or among the circuits, of every suite's instances
  const ct = at ? undefined : data.circuit.get(`${s.key}/${route.instance}`);
  const out = [crumbs(HOME, ...suiteHome(s.key, c),
    ...(at ? [[at.g.title, { view: "category", category: c.key, only: at.g.key }]]
      : ct ? [[ct.title, { view: "category", category: APART, only: ct.key }]] : [[s.title, { view: "suite", suite: s.key }], [f.title]])),
    element("h2", {}, f.title),
    // the suite that gathered the instances is no group of the site's (its crumbs name it where no type does)
    element("div", { class: "chips" }, element("span", { class: "chip" }, plural(f.instances.length, "instance")),
      element("span", { class: "chip" }, `${s.limit} s a run`))];
  const programs = element("section", { class: "programs", id: "programs" }, note("Loading the programs and the runs…"));
  out.push(programs, element("h3", {}, "Times"), timesGrid(s, f, rows, short, route), raw(`benchmarks/${s.key}/${f.key}.json`, s.file));
  // the instance's whole name where the choice shows a shorter one; a family of one shows its name alone
  const here = { view: "family", suite: s.key, family: f.key };
  const one = f.instances.length === 1;
  const pick = one ? null : picker(f.instances.map((n) => ({ route: { ...here, instance: n }, label: short.get(n), on: n === route.instance })));
  const name = one || short.get(route.instance) !== route.instance ? element("code", { class: "inst" }, route.instance) : null;
  familyData(s.key, f.key).then((fam) => {
    if (view.dataset.route !== page) return;   // another view was chosen meanwhile
    programs.replaceChildren(...instanceView(s, f, fam, now, pick, name));
  }).catch((failure) => {
    programs.replaceChildren(element("p", { class: "failed" }, `The programs and runs did not load: ${failure.message}`));
  });
  return out;
}

// A family's instances by the systems given them: each time shaded by its
// place between 0.2 s and the limit on a log scale, the fastest from 0.2 s
// marked, a failure by its word; a time opens the system's runs of the
// instance, a name the instance's programs. A family of one model along
// several axes (its question, its evidence, its sizes) names each instance
// by its values, a column an axis, the first opening its programs.
let showAll = false;
function timesGrid(s, f, rows, short, route) {
  const given = s.systems.filter((x) => f.instances.some((name) => rows.get(name)[column(s, x)] !== null));
  const cols = showAll ? s.systems : given;
  const axes = f.axes ?? ["instance"];
  const head = element("tr", {}, ...axes.map((a) => element("th", { scope: "col" }, a)),
    ...cols.map((x) => element("th", { scope: "col", class: `sys${isMine(x) ? " mine" : ""}`,
      title: dashed(x) ? `${data.names[x]}: the whole process, with the language's start and compilation` : data.names[x] }, element("span", {}, data.names[x]))));
  const body = element("tbody");
  const seen = new Set();
  for (const name of f.instances) {
    const row = rows.get(name);
    const fastest = fastestOf(cols.filter((x) => !dashed(x)).map((x) => row[column(s, x)]));
    const named = f.values?.[name] ?? [short.get(name)];
    const tr = element("tr", { class: name === route.instance ? "on" : null },
      ...named.map((v, k) => element("th", { scope: "row", title: k ? null : name }, k ? v : link({ ...pageOf(route), instance: name }, v))));
    for (const x of cols) {
      const v = row[column(s, x)];
      const mine = isMine(x) ? "mine" : "";
      if (v === null) {
        tr.append(element("td", { class: `none ${mine}`, title: `${data.names[x]}: not given` }));
        continue;
      }
      const { cls, p, slower } = rated(v, fastest);
      let text, title;
      if (answered(v)) {
        text = seconds(v);
        title = `${data.names[x]}: ${v} s${slower ? `, ${slower} the fastest's time` : ""}`;
      } else {
        seen.add(v);
        const [t, meaning] = FAILURES[v] ?? [() => v, v];
        text = t(s.limit);
        title = `${data.names[x]}: ${meaning}`;
      }
      // The same instance's runs keep its file on show; another's show its first.
      tr.append(element("td", { class: `${cls} ${mine}`.trim() }, element("button", { type: "button", style: `--p:${p.toFixed(3)}`, title,
        "data-runs": `${s.key}/${name}/${x}`,
        onclick: () => go({ ...now, instance: name, file: name === now.instance ? now.file : undefined, runs: x }) }, text)));
    }
    body.append(tr);
  }
  const key = shades(seen.has("unsupported"));
  for (const v of Object.keys(FAILURES)) if (v !== "unsupported" && seen.has(v)) key.push(failureKey(v, s.limit));
  key.push(element("span", {}, "a time opens its runs"));
  const hidden = s.systems.length - given.length;
  const box = element("div", { class: "times-box" }, scroller(element("table", { class: "grid" }, element("thead", {}, head), body)),
    element("p", { class: "key" }, ...key));
  if (hidden) {
    box.append(element("button", { type: "button", class: "plain", onclick: () => { showAll = !showAll; show(true); } },
      showAll ? "Hide the systems not given these instances" : `Show the ${plural(hidden, "system")} not given these instances`));
  }
  return box;
}

// The system of a run by its name: a pipeline's run of tddlog by the
// pipeline's, which its label starts with.
const runName = (r) => r.label?.match(/^\S+\+tddlog\b/)?.[0] ?? data.names[r.system] ?? r.system;
// The name tddlog reads an instance's programs under: its own where it ran
// beside a pipeline (Roulette+tddlog on Geni's programs), the pipeline's
// where its runs are only one's.
const mineReader = (inst) => {
  const names = (inst.runs ?? []).map(runName);
  return names.includes("tddlog") ? "tddlog" : names.find((n) => n.endsWith("+tddlog")) ?? "tddlog";
};

// The systems that ran an instance but read none of its files on show, nor
// translated any: their files were not captured. A run whose grounding
// failed (`ungrounded`), that was never handed a program (`unhanded`: every
// repetition skipped or refused) or that was skipped was handed no file.
function uncaptured(inst) {
  const readers = (inst.tabs ?? []).flatMap((t) => [...t.readers, ...(t.translations ?? []).map(([who]) => who)]);
  return [...new Set((inst.runs ?? []).filter((r) => !r.ungrounded && !r.unhanded && !r.skipped).map(runName))]
    .filter((name) => !readers.some(readsAs(name)));
}

// The program tddlog read, for the playground: the one ProbLog file of the
// language tddlog reads (filled in where it has places: code.js), as a link,
// or why the playground cannot open it.
function playground(s, f, fam, inst, name) {
  const read = (inst.tabs ?? []).find((t) => t.readers.includes(name) && t.files.some(isProgram));
  // files of one name (a loop's two forms) each name their readers
  const pl = read?.files.filter((file) => isProgram(file) && mayRead(file, (r) => r === name)) ?? [];
  if (pl.length !== 1) return null;
  const filled = tddlogCode.filledFile(inst, read, pl[0], fam.texts);
  const { tab, file } = filled ?? { tab: read, file: pl[0] };
  if (file.held) return element("span", { class: "no-try" }, "Not in the playground: the site does not show this program; its file says why.");
  if (file.part) return element("span", { class: "no-try" }, `Not in the playground: the site keeps the first ${size(file.part)} of this ${size(file.bytes)} program.`);
  const places = filled ? ", with this instance's arguments in its places" : "";
  return element("span", { class: "try" }, element("a", { href: `${dataRoot}../playground/#bench=${[s.key, f.key, inst.name].map(encodeURIComponent).join("/")}` },
    "Open in the playground"), element("span", { class: "muted" }, ` the ${tab.lang} program tddlog read${places}, ${size(file.bytes)}`));
}

// The systems that read a tab, each once: a form's label ("ProbLog -k sdd")
// under its system's name, an arm of tddlog's ("tddlog (auto+arm)") as it
// is, and the reference apart.
function readersOf(tab) {
  const names = Object.values(data.names);
  const name = (r) => (r.endsWith(")") ? r : names.find((n) => readsAs(n)(r)) ?? r);
  return [...new Set(tab.readers.filter((r) => r !== "reference").map(name))];
}

// A tab's files in the order shown: the evidence (an .inst or .evid file)
// after the model it observes.
function shownFiles(tab) {
  const observed = (k) => /\.(inst|evid)$/.test(tab.files[k].name);
  return [...tab.files.keys()].sort((a, b) => observed(a) - observed(b));
}

const FORMS = { recursive: "recursive form", unrolled: "unrolled form" };

// Whether a reader that `who` picks may have read a file: any reader, where
// the file's name is its tab's only one; else those it names.
const mayRead = (file, who) => !(file.readers || file.for) || (file.readers ?? []).some(who);

// A ProbLog program of a tab, not the data a program consults at a place ($1: `data`).
const isProgram = (file) => /\.pl$/.test(file.name) && !file.data;

// What tells file k of a tab from the tab's other files of its name (a path
// read with other contents in other runs, a loop's two forms, an arm's copy
// in a directory of its own): its form, else who read it, else its
// directory, else its place among them; null where no other file of the tab
// has its name.
function twinLabel(tab, k) {
  const base = (j) => tab.files[j].name.split("/").pop();
  const same = [...tab.files.keys()].filter((j) => base(j) === base(k));
  if (same.length < 2) return null;
  const who = (f) => (f.readers ?? []).map((r) => (r === "reference" ? "the reference" : r));
  const label = (j) => {
    const f = tab.files[j];
    if (f.form) return FORMS[f.form] ?? `${f.form} form`;
    if (who(f).length) return and(who(f));
    if (f.for?.length) return `for ${and(f.for)}`;
    return f.name.slice(0, -base(j).length).replace(/\/$/, "");
  };
  const labels = same.map(label);
  const mine = labels[same.indexOf(k)];
  return mine && labels.filter((l) => l === mine).length === 1 ? mine : `${mine ? `${mine}, ` : ""}${same.indexOf(k) + 1} of ${same.length}`;
}

// The file an instance's view shows first: the program tddlog read (its
// ProbLog file, where its tab has others), else the original's first.
function firstFile(inst) {
  const tddlog = (r) => readsAs("tddlog")(r) || /^\S+\+tddlog\b/.test(r);
  // the program tddlog itself read before a pipeline's (the circuit Alea.jl+tddlog read beside it)
  const tabs = inst.tabs.map((t, i) => [t, i]);
  const reads = [readsAs("tddlog"), tddlog].map((who) => tabs.filter(([t]) => t.readers.some(who))).find((r) => r.length) ?? [];
  const ti = (reads.find(([t]) => t.files.some(isProgram)) ?? reads[0] ?? [null, -1])[1];
  if (ti < 0) return [0, shownFiles(inst.tabs[0])[0]];
  // of a loop's two forms (files of one name, each with its readers), the one tddlog read
  const pl = inst.tabs[ti].files.findIndex((file) => isProgram(file) && mayRead(file, tddlog));
  return [ti, pl < 0 ? shownFiles(inst.tabs[ti])[0] : pl];
}

// The address's part for a file of an instance: none for the one shown first.
function fileRoute(inst, ti, fi) {
  const [t, f] = firstFile(inst);
  return ti === t && fi === f ? undefined : [ti, fi];
}

// A tab in the menu: its language, what its files are to the original in a
// few words (the original, a translation and the script that wrote it, a
// change of format, a system's own form), and who reads it.
function versionLabel(tab) {
  const who = readersOf(tab);
  const read = who.length ? `read by ${and(who)}` : tab.translations?.length ? "an intermediate step, read by no system" : "read by no system";
  const what = tab.what ?? (tab.original ? "the original" : "");
  return `${tab.lang}: ${what ? `${what}, ` : ""}${read}`;
}

// The note on a translation a system read: automatic, or a port written by hand (by_hand).
const unfair = (tab) => `Note: this is ${tab.by_hand ? "a translation by hand" : "an automatic translation"} from the original `
  + "program, which might have performance implications that make the comparison unfair.";

// An instance picker: a menu where there are more than 12, else a row of
// links. choices: [{ route, label, tag (a small label, or none), on }].
function picker(choices) {
  if (choices.length > 12) {
    return element("select", { "aria-label": "Instance", onchange: (e) => go(choices[Number(e.target.value)].route) },
      ...choices.map((p, i) => element("option", { value: i, selected: p.on }, p.tag ? `${p.label} (${p.tag})` : p.label)));
  }
  return element("div", { class: "seg", role: "group", "aria-label": "Instance" }, ...choices.map((p) => {
    const a = link(p.route, p.label, ...(p.tag ? [element("small", {}, p.tag)] : []));
    if (p.on) a.setAttribute("aria-current", "true");
    return a;
  }));
}

// The command tddlog's run of an instance read its program with, as its tab
// of the program tddlog read lists it; null where it lists none.
function mineCommand(inst) {
  if (!inst.tabs?.length) return null;
  const tab = inst.tabs[firstFile(inst)[0]];
  return tab.cmds.find(([who]) => readsAs("tddlog")(who) || /^\S+\+tddlog\b/.test(who))?.[1] ?? null;
}

// An instance of a family: its programs (the one tddlog read, its command,
// and a menu of every other system's), then its answers. `route` names the
// instance and the file to show, `pick` chooses another instance (none in a
// family of one), `name` is the instance's name beside it (a link, or none),
// and `where` is what an address of the view adds to the file it names.
let compare = false;
function instanceView(s, f, fam, route, pick, name, where = {}) {
  const inst = fam.instances.find((i) => i.name === route.instance) ?? fam.instances[0];
  const out = [];
  const cmd = mineCommand(inst);
  out.push(element("h3", {}, "Programs"), element("div", { class: "pick" }, element("span", { class: "muted" }, "instance"),
    ...(pick ? [pick] : []), ...(name ? [name] : []), playground(s, f, fam, inst, mineReader(inst))),
  ...(cmd ? [element("p", { class: "cmdline", title: "The command of tddlog's run" }, element("code", {}, cmd))] : []));
  if (!inst.tabs?.length) {
    if (inst.gen) out.push(markup("p", inst.gen, { class: "note" }));
    out.push(note("The programs of this instance were not captured."));
  } else {
    const view = element("div", { class: "viewer" });
    view.where = where;
    out.push(view);
    // a file the address names later, without drawing the view again
    view.show = (file) => showFile(view, fam, inst, ...(file ?? firstFile(inst)));
    queueMicrotask(() => view.show(route.file));
  }
  if (inst.answers) out.push(answers(inst.answers));
  return out.filter(Boolean);
}

// A family's program on its view in its type (family g of category c): an
// instance of it (route.program, else its first) and its programs, as a
// benchmark's view shows them, the instance chosen among the family's by
// what each is: its values where its benchmarks' family names some, else its
// name less the start its benchmarks' family shares; where two are alike,
// the suite of each as a small label. A family of several suites lists its
// instances by name, one suite's in its order (by size).
function programSection(c, g, route) {
  const box = element("section", { class: "programs", id: "programs" });
  const rows = g.rows.map((i) => ({ suite: c.rows[i][0], name: c.rows[i][1], f: familyOf(c.rows[i][0], c.rows[i][1]) })).filter((r) => r.f);
  if (!rows.length) return element("section", { id: "programs", hidden: true });   // the site without the benchmarks' data
  const shorts = new Map();
  for (const r of rows) if (!shorts.has(r.f)) shorts.set(r.f, shortNames(r.f.instances));
  const label = (r) => (r.f.values?.[r.name]?.filter(Boolean).join(", ") || shorts.get(r.f).get(r.name));
  const suites = new Set(rows.map((r) => r.suite));
  const order = suites.size > 1 ? [...rows].sort((a, b) => label(a).localeCompare(label(b), undefined, { numeric: true }))
    : [...rows].sort((a, b) => a.f.instances.indexOf(a.name) - b.f.instances.indexOf(b.name));
  const twice = new Set(order.map(label).filter((l, i, all) => all.indexOf(l) !== i));
  const on = order.find((r) => route.program?.[0] === r.suite && route.program?.[1] === r.name) ?? order[0];
  const at = { view: "category", category: c.key, only: g.key };
  const pick = order.length > 1 ? picker(order.map((r) => ({ route: { ...at, program: [r.suite, r.name] }, label: label(r),
    tag: twice.has(label(r)) ? data.suites.get(r.suite).title : null, on: r === on }))) : null;
  // the instance's name opens its benchmark's view, with its times by system
  const name = link({ view: "family", suite: on.suite, family: on.f.key, instance: on.name }, element("code", { class: "inst" }, on.name));
  name.title = "Its times by system";
  const s = data.suites.get(on.suite);
  box.append(note("Loading the programs…"));
  familyData(s.key, on.f.key).then((fam) => {
    if (!box.isConnected) return;   // another instance or view was chosen meanwhile
    box.replaceChildren(...instanceView(s, on.f, fam, { instance: on.name, file: route.file }, pick, name, { program: [on.suite, on.name] }));
  }).catch((failure) => {
    box.replaceChildren(element("p", { class: "failed" }, `The programs did not load: ${failure.message}`));
  });
  return box;
}

function size(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1 << 20) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`;
  return `${(bytes / (1 << 20)).toFixed(1)} MB`;
}

// A file of an instance (tab ti, file fi; the first shown where it has none
// such): the menu of the instance's programs, what its tab says and the
// commands that read it, folded, and the tab's files, beside the original's
// where asked.
let about = false;
function showFile(view, fam, inst, ti, fi) {
  if (!inst.tabs[ti]?.files[fi]) [ti, fi] = firstFile(inst);
  const tab = inst.tabs[ti];
  view.dataset.file = `${ti}:${fi}`;
  const open = (t, k) => {
    showFile(view, fam, inst, t, k);
    remember({ ...view.where, file: fileRoute(inst, t, k) });
  };
  const [first, mine] = firstFile(inst);
  const order = [first, ...inst.tabs.keys()].filter((t, k, all) => all.indexOf(t) === k);
  const menu = element("select", { class: "version", "aria-label": "The program as a system read it", onchange: (e) => {
    const t = Number(e.target.value);
    open(t, t === first ? mine : shownFiles(inst.tabs[t])[0]);
  } },
    ...order.map((t) => element("option", { value: t, selected: t === ti }, versionLabel(inst.tabs[t]))));
  const canCompare = !tab.original && inst.tabs[0].original;
  const two = compare && canCompare;
  const toggle = element("label", { class: "check" }, element("input", { type: "checkbox", checked: two, disabled: !canCompare,
    onchange: (e) => { compare = e.target.checked; showFile(view, fam, inst, ti, fi); } }), " Beside the original");
  const panes = element("div", { class: `panes${two ? " two" : ""}` });
  if (two) {
    // the original's files, chosen in its own pane
    const original = (k) => filePane(fam, inst.tabs[0], k, (j) => {
      const next = original(j);
      panes.firstChild.replaceWith(next);
      return next;
    }, element("p", { class: "cap" }, `The original, ${inst.tabs[0].lang}`));
    panes.append(original(shownFiles(inst.tabs[0])[0]));
  }
  // a network converted to another network format is native: no note
  const cap = tab.translated && !tab.conversion && readersOf(tab).length ? element("p", { class: "unfair" }, unfair(tab))
    : two ? element("p", { class: "cap" }, tab.lang) : null;
  panes.append(filePane(fam, tab, fi, (k) => {
    open(ti, k);
    return view.querySelector(".panes > .pane:last-child");
  }, cap));
  view.replaceChildren(element("div", { class: "pick" }, element("span", { class: "muted" }, "program"), menu, toggle), aboutFiles(inst, tab), panes);
}

// What an instance's files are, how its tab was made and the commands that
// read it, folded under one line.
function aboutFiles(inst, tab) {
  const parts = [element("p", {}, "The files each system was handed, byte for byte, as the benchmark driver's own code writes them at the run's commit. "
    + "The program tddlog read comes first; the menu shows the original and each other system's.")];
  if (inst.gen) parts.push(markup("p", inst.gen));
  parts.push(element("p", {}, tab.says), ...tab.notes.map((n) => element("p", {}, n)));
  if (tab.readers.includes("reference")) parts.push(element("p", {}, `The reference answers${inst.answers?.ref ? `, ${inst.answers.ref},` : ""} are computed from it.`));
  const unseen = uncaptured(inst);
  if (unseen.length) {
    parts.push(element("p", { class: "uncaptured" }, `Not shown: the files that ${and(unseen)} ${unseen.length === 1 ? "was" : "were"} given, `
      + `which were not captured when ${unseen.length === 1 ? "it" : "they"} ran.`));
  }
  // the readers' commands, then the steps of a system's job that translate
  // these files into another tab's, which that system then reads
  const steps = [...new Map((tab.translations ?? []).map(([who, cmd]) => [cmd, who])).entries()];
  const n = tab.cmds.length + Object.values(tab.more).reduce((a, b) => a + b, 0) + steps.length;
  if (n) {
    const dl = element("dl", { class: "cmds" });
    for (const [who, cmd] of tab.cmds) dl.append(element("dt", {}, who), element("dd", {}, element("code", {}, cmd)));
    for (const [who, k] of Object.entries(tab.more)) dl.append(element("dt", {}), element("dd", {}, `${who}: and ${k} more commands of the same form`));
    for (const [cmd, who] of steps) dl.append(element("dt", {}, `for ${who}`), element("dd", {}, element("code", {}, cmd)));
    parts.push(element("h4", {}, `The commands that read it (${count(n)})`), dl);
  }
  return element("details", { class: "about", open: about, ontoggle: (e) => { about = e.target.open; } },
    element("summary", {}, n ? "About these files and the commands that read them" : "About these files"), ...parts);
}

// A tab's file fi in a pane: a line above it where given (the warning on a
// translation), the tab's files as tabs where it has several, and the file.
// pick(k) shows the tab's file k and returns its pane.
function filePane(fam, tab, fi, pick, cap) {
  const box = element("div", { class: "pane" }, cap);
  if (tab.files.length > 1) {
    const choose = (k) => pick(k)?.querySelector('[role="tab"][aria-selected="true"]')?.focus();
    const ks = shownFiles(tab);
    const strip = element("div", { class: "ftabs", role: "tablist", "aria-label": "Files", onkeydown: (e) => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
      if (!step) return;
      e.preventDefault();
      choose(ks[(ks.indexOf(fi) + step + ks.length) % ks.length]);
    } }, ...ks.map((k) => {
      const twin = twinLabel(tab, k);
      const place = tab.files[k].data;   // the data the program consults at $1
      return element("button", { type: "button", role: "tab", "aria-selected": String(k === fi), tabindex: k === fi ? "0" : "-1",
        "data-file": k, onclick: () => k !== fi && choose(k) }, tab.files[k].name.split("/").pop(),
      ...(place ? [element("span", { class: "place", title: `The data the program consults at ${place}` }, `data ${place}`)] : []),
      ...(twin ? [element("span", { class: "twin" }, twin)] : []), element("small", {}, size(tab.files[k].bytes)));
    }));
    box.append(strip);
  }
  return fileIn(box, fam, tab.files[fi]);
}

// A file's text: in its family's data, or a file of its own, gzipped where
// its name ends in .gz.
async function fileText(fam, file) {
  if (fam.texts[file.sha] !== undefined) return fam.texts[file.sha];
  const response = await fetch(`${dataRoot}benchmarks/${file.src}`);
  if (!response.ok) throw new Error(String(response.status));
  return file.src.endsWith(".gz") ? new Response(response.body.pipeThrough(new DecompressionStream("gzip"))).text() : response.text();
}

// Hands the reader a text as a file of this name.
function saveText(name, text) {
  const url = URL.createObjectURL(new Blob([text], { type: "text/plain;charset=utf-8" }));
  const a = element("a", { href: url, download: name, hidden: true });
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 60000);
}

// A file, into its pane: its name, size and SHA-256, how it was made and
// any other line the data gives above it, and its text (or why it is not
// shown); a long file's first SHOWN_LINES lines, and the text the site
// keeps to save.
function fileIn(box, fam, file) {
  box.append(element("div", { class: "fhead" }, element("code", { class: "fname" }, file.name),
    element("span", {}, `${count(file.bytes)} bytes${file.lines !== undefined ? `, ${count(file.lines)} lines` : ""}`),
    element("span", { class: "sha", title: "SHA-256" }, `sha256 ${file.sha}`)));
  const above = Object.keys(file).filter((field) => !FILE_FIELDS.has(field) && typeof file[field] === "string")
    .sort((a, b) => (b === "made") - (a === "made"));
  for (const field of above) box.append(markup("p", field === "made" ? extend.made(file[field]) : file[field], { class: "made" }));
  // a file whose name the tab shows more than once: who read this one (a
  // form's line above says which systems were scored on it)
  const who = (file.readers ?? []).map((r) => (r === "reference" ? "the reference" : r));
  if (who.length && !file.form) box.append(element("p", { class: "made" }, `Read by ${and(who)}.`));
  else if (file.for?.length) box.append(element("p", { class: "made" }, `Translated in the jobs of ${and(file.for)}, which read what it is translated into.`));
  if (file.held) {
    box.append(markup("p", file.held, { class: "held" }));
    return box;
  }
  if (file.archive) {
    box.append(element("p", { class: "held" }, "An archive, not shown; its SHA-256 identifies it."));
    return box;
  }
  const code = element("div", { class: "code" }, note("Loading…"));
  box.append(code);
  if (fam.texts[file.sha] === undefined && !file.src) {
    code.replaceChildren(note("Not kept."));
    return box;
  }
  const fill = (text) => {
    const lines = text.split("\n");
    if (lines[lines.length - 1] === "") lines.pop();
    const shown = lines.slice(0, SHOWN_LINES);
    const body = shown.join("\n");
    const pre = element("pre", { class: "src" });
    const lang = HIGHLIGHT[file.name.match(/\.([a-z]+)$/)?.[1]];
    if (typeof tddlogCode !== "undefined" && lang) pre.innerHTML = tddlogCode.highlight(lang, body);
    else pre.textContent = body;
    const gutter = element("pre", { class: "gutter", "aria-hidden": "true" }, shown.map((_, i) => i + 1).join("\n"));
    const notes = [];
    if (lines.length > SHOWN_LINES) notes.push(`The first ${count(SHOWN_LINES)} of its ${count(lines.length)} lines. `);
    if (file.part) notes.push(`The first ${count(file.part)} bytes${file.gzip ? ", decompressed" : ""}: the file is larger than the site keeps; the command above writes it again, and its SHA-256 checks the copy. `);
    const name = file.name.split("/").pop();
    code.replaceChildren(element("div", { class: "lines" }, gutter, pre), note(...notes,
      element("button", { type: "button", class: "plain save", onclick: () => saveText(name, text) }, file.part ? "Save the part the site keeps" : `Save ${name}`)));
  };
  fileText(fam, file).then(fill, (failure) => code.replaceChildren(element("p", { class: "failed" }, `The file did not load: ${failure.message}`)));
  return box;
}

function answers(a) {
  return lazy({ class: "answers" }, a.log10z ? "The partition function, log10 Z" : `The answers (${plural(a.queries, "query", "queries")})`, () => {
    const head = element("tr", {}, element("th", { scope: "col" }, a.log10z ? "quantity" : "query"),
      ...a.cols.map((c) => element("th", { scope: "col", class: "num" }, c)));
    const body = element("tbody", {}, ...a.rows.map(([q, ...cells]) => element("tr", {}, element("th", { scope: "row" }, q),
      ...cells.map((c) => element("td", { class: "num" }, c)))));
    const notes = [];
    if (a.ref) notes.push(`${a.ref_note} Each system's answers of its first repetition that answered.`);
    if (a.queries > a.rows.length) notes.push(`The first ${a.rows.length} of ${count(a.queries)} queries; the largest difference over all of them is in each run's "from the reference".`);
    return [scroller(element("table", { class: "times" }, element("thead", {}, head), body)), notes.length ? note(notes.join(" ")) : null];
  });
}

// ------------------------------------------------------------- a system's runs

const drawer = () => document.getElementById("drawer");
// What had the focus before the drawer opened, and the drawer's latest
// request, which a later one or closing it supersedes.
let opener = null;
let drawn = 0;

function closeRuns() {
  const box = drawer();
  drawn++;
  box.hidden = true;
  // the time that opened it, or the same one drawn again since
  const back = opener?.isConnected ? opener : document.querySelector(`#view button[data-runs="${CSS.escape(box.dataset.for ?? "")}"]`);
  delete box.dataset.for;
  back?.focus();
  opener = null;
}

// A system's runs of an instance (route.runs and route.instance), beside
// the view: its time in the table and what it is, the commands and files
// it was given, then each run of each of its forms, the latest first, runs
// of the same outcome folded into one; a later run in lanes of its own
// beside that run's tddlog.
async function runs(route) {
  const s = data.suites.get(route.suite);
  const f = familiesOf(s.key).find((x) => x.key === route.family);
  const name = route.instance, key = route.runs;
  const box = drawer();
  const id = `${s.key}/${name}/${key}`;
  if (!box.hidden && box.dataset.for === id) return;
  const ticket = ++drawn;
  if (!box.contains(document.activeElement)) opener = document.activeElement;
  box.dataset.for = id;
  const v = s.instances.find((row) => row[0] === name)[column(s, key)];
  const what = answered(v) ? `${seconds(v)} s` : v === null ? "not given" : FAILURES[v]?.[1] ?? v;
  box.replaceChildren(element("button", { type: "button", class: "plain close", onclick: () => go({ ...now, runs: undefined }) }, "Close"),
    element("p", { class: "muted" }, element("code", {}, name)), element("h3", { id: "drawer-title" }, data.names[key]),
    element("p", {}, element("b", {}, "In the table: "), what), note("Loading…"));
  box.hidden = false;
  box.querySelector(".close").focus();
  let fam;
  try {
    fam = await familyData(s.key, f.key);
  } catch (failure) {
    if (ticket === drawn) box.lastChild.replaceWith(element("p", { class: "failed" }, `The runs did not load: ${failure.message}`));
    return;
  }
  if (ticket !== drawn) return;
  const inst = fam.instances.find((i) => i.name === name);
  // the column's own runs where the instance has them (an integration that ran on the program
  // itself), else its records' system's (Alea.jl's for alea-process)
  const rec = (inst?.runs ?? []).some((r) => r.system === key) ? key : recorded(key);
  const mine = (inst?.runs ?? []).filter((r) => r.system === rec);
  const parts = [];
  const about = [];
  if (dashed(key)) about.push("The whole process's time, with the language's start and compilation; the other time of the same system leaves them out, as its paper times it.");
  if (inst?.reps > 1) about.push("A time is the median of the repetitions, a failed one counted as infinite.");
  if (new Set(mine.map((r) => r.label)).size > 1) about.push("A system run in several forms has the time of its fastest form.");
  if (inst?.notes?.[rec]) about.push(inst.notes[rec]);
  if (about.length) parts.push(note(about.join(" ")));
  const name0 = isMine(key) ? data.names[key] : data.names[rec] ?? data.names[key];
  // its job's commands: the translations it reads first, then its own
  const cmds = [...new Map((inst?.tabs ?? []).flatMap((t) => [...(t.translations ?? []), ...t.cmds])
    .filter(([who]) => readsAs(name0)(who)).map(([who, cmd]) => [cmd, who])).keys()].map((cmd) => [name0, cmd]);
  if (cmds.length) parts.push(element("h4", {}, "Commands"), ...cmds.map(([, cmd]) => element("code", { class: "cmd" }, cmd)));
  const read = (inst?.tabs ?? []).flatMap((t, ti) => (t.readers.some(readsAs(name0)) ? t.files.map((file, fi) => [t, file, ti, fi]) : []));
  if (read.length) {
    parts.push(element("h4", {}, "Reads"), element("div", { class: "files" }, ...read.map(([t, file, ti, fi]) => element("button", { type: "button", class: "file",
      onclick: () => {
        // the focus goes on to the file shown, not back to the time
        opener = document.querySelector("#programs select.version");
        go({ ...now, runs: undefined, file: fileRoute(inst, ti, fi) });
      } }, file.name.split("/").pop(), element("small", {}, t.lang)))));
  } else if (mine.some((r) => !r.ungrounded && !r.unhanded && !r.skipped)) {
    parts.push(note(`The command and the files ${name0} was given were not captured.`));
  }
  const tryIt = rec === "tddlog" && inst ? playground(s, f, fam, inst, name0) : null;
  if (tryIt) parts.push(element("p", {}, tryIt));
  // the runs, the latest first; the same outcome of the same form folded
  const shown = [];
  const folded = new Map();
  for (const r of [...mine].reverse()) {
    const sig = JSON.stringify([r.label, r.skipped ?? false, r.queued ?? false, (r.rows ?? []).map((row) => row.cells.slice(1, 3))]);
    if (folded.has(sig)) { folded.get(sig).more++; continue; }
    const e = { r, more: 0 };
    folded.set(sig, e);
    shown.push(e);
  }
  const med = (r) => {
    const at = (r.cols ?? []).indexOf("seconds");
    const ts = (r.rows ?? []).filter((row) => row.cells[1] === "answered").map((row) => Number(row.cells[at])).filter(Number.isFinite).sort((a, b) => a - b);
    return ts.length ? ts[ts.length >> 1] : null;
  };
  // The run whose median is the time in the table: the closest, within rounding.
  let inTable = null;
  if (answered(v)) {
    let gap = Infinity;
    for (const e of shown) {
      const m = med(e.r);
      if (m !== null && Math.abs(m - v) < gap) [gap, inTable] = [Math.abs(m - v), e];
    }
    if (gap > Math.max(0.011, 0.06 * v)) inTable = null;
  }
  const tddlogIn = (later) => (inst?.runs ?? []).filter((r) => r.system === "tddlog" && r.later === later && !r.skipped);
  parts.push(element("h4", {}, "Runs"));
  if (!shown.length) parts.push(note("No run of this system on this instance."));
  shown.forEach((e, i) => {
    const { r, more } = e;
    const hit = e === inTable;
    const at = (r.cols ?? []).indexOf("seconds");
    const summary = r.skipped ? (r.queued ? "not yet run" : "not run") : r.rows.map((row) => (row.cells[1] === "answered" ? `${row.cells[at]} s` : row.cells[1])).join(", ");
    const block = element("details", { class: "run", open: i === 0 || hit },
      element("summary", {}, element("b", {}, r.label), hit ? element("span", { class: "badge orig" }, "time in the table") : null,
        element("span", { class: "muted" }, ` ${summary}${more ? `; and ${plural(more, "run")} with the same outcome` : ""}`)),
      note(r.later ? "A later run, in lanes of its own." : "The release run."));
    if (r.later && rec !== "tddlog") {
      const t = tddlogIn(r.later).map(med).filter((x) => x !== null);
      if (t.length) block.append(note(`tddlog in the same job: ${seconds(t[0])} s.`));
    }
    if (r.skipped) block.append(note(r.queued ? "Not yet run: a rerun is queued." : `Not run in any repetition: ${FAILURES.skipped[1]}.`));
    else {
      // The sizes, a long line, under the other columns of their row.
      const sizes = r.cols.indexOf("sizes");
      const cols = r.cols.filter((_, j) => j !== sizes);
      block.append(scroller(element("table", { class: "runs" },
        element("thead", {}, element("tr", {}, ...cols.map((c) => element("th", { scope: "col" }, c)))),
        element("tbody", {}, ...r.rows.flatMap((row) => [
          element("tr", {}, ...row.cells.filter((_, j) => j !== sizes).map((c) => element("td", {}, c))),
          sizes >= 0 && row.cells[sizes] ? element("tr", { class: "sizes" }, element("td", { colspan: String(cols.length) }, row.cells[sizes])) : null,
          row.error ? element("tr", { class: "err" }, element("td", { colspan: String(cols.length) }, element("pre", {}, row.error))) : null,
          row.note ? element("tr", { class: "err" }, element("td", { colspan: String(cols.length) }, row.note)) : null,
        ])))));
    }
    parts.push(block);
  });
  box.lastChild.replaceWith(...parts.filter(Boolean));
}

// ------------------------------------------------------------- the rail

// The menu: the front page's sections, as the bar at the top names them,
// each a heading with its entries under it. By baseline lists the other
// systems, the ten that answer most and the rest behind a control whose
// choice the viewer keeps; By integration, tddlog alone and its pipelines;
// By program type, the types and all programs, the one on show (or that
// holds the family or instance on show) with its families, then the
// programs tddlog does not run; How it works, its walkthroughs and the
// methods table. By program type's heading opens the results page's
// figures; the logo above is the front page.
const MENU_TOP = 10;
// The page of the programs tddlog does not run, the last entry under By program type.
const COVERAGE = "Programs tddlog does not run";
// The methods table (next.js's), the last entry under How it works; every
// speed-up's header links to it (speedName).
const METHODS = "Methods";
// The section each view is in, which the bar marks.
const SECTION_OF = { baselines: "baselines", baseline: "baselines", coverage: "types", integrations: "integrations",
  integration: "integrations", types: "types", category: "types", suite: "types", family: "types", how: "how", methods: "how" };
// The circuits and a suite of circuits in no type, with its pages, are the circuits' section.
const circuitSuite = (suite) => !categoriesOf(suite).length && Boolean(data.circuits?.perSuite.has(suite));
const sectionOf = (route) => ((route.view === "category" && route.category === APART)
  || ((route.view === "suite" || route.view === "family") && circuitSuite(route.suite)) ? APART : SECTION_OF[route.view]);
const MENU_KEY = "tddlog-baselines";
function rail(route) {
  const tree = element("ul", { class: "tree" });
  const section = (hash, ...children) => element("a", { href: front + hash, "data-section": hash }, ...children);
  const entry = (hash, text, n, x) => element("li", { class: x ? "solver" : null },
    section(hash, ...(x ? [swatch(x)] : []), text, ...(n === undefined ? [] : [element("span", { class: "n" }, n)])));
  const group = (key, ...entries) => element("li", { class: "group" }, section(`#/${key}`, sectionName(key)), ...entries);
  // by baseline
  const others = data.systems.filter((s) => !isMine(s.key)).sort((a, b) => b.answers - a.answers);
  const shown = route.view === "baseline" ? route.system : null;
  let open = others.findIndex((s) => s.key === shown) >= MENU_TOP;
  try {
    open ||= localStorage.getItem(MENU_KEY) === "all";
  } catch {
    // no storage: the ten alone
  }
  const more = others.length - MENU_TOP;
  // a list's counts say what they count, in a line over them
  const legend = (text) => element("li", { class: "legend", "aria-hidden": "true" }, text);
  const list = element("ul", { class: `baselines${open ? " all" : ""}` }, legend("instances answered"), ...others.map((s, i) => {
    const li = entry(`#/baselines/${encodeURIComponent(s.key)}`, data.names[s.key], count(s.answers), s.key);
    li.firstChild.title = `${data.names[s.key]} answers ${count(s.answers)} of the ${count(s.given)} instances it was given`;
    if (i >= MENU_TOP) li.classList.add("more");
    return li;
  }));
  if (more > 0) {
    const label = () => (list.classList.contains("all") ? "− fewer" : `+ ${more} more`);
    const button = element("button", { type: "button", class: "plain more-toggle", "aria-expanded": String(open), onclick: () => {
      const all = list.classList.toggle("all");
      button.textContent = label();
      button.setAttribute("aria-expanded", String(all));
      try {
        localStorage.setItem(MENU_KEY, all ? "all" : "top");
      } catch {
        // kept for this view alone
      }
    } }, "");
    button.textContent = label();
    list.append(element("li", { class: "toggle" }, button));
  }
  tree.append(group("baselines", list));
  // by integration, where the data has them (and the bar's link to them with it)
  if (data.integrations) tree.append(group("integrations", element("ul", {}, ...integrationsOf().map((k) => entry(`#/integrations/${k}`, data.integrations[k].title)))));
  for (const a of document.querySelectorAll('.site-nav a[href$="#/integrations"]')) a.hidden = !data.integrations;
  // by program type: a benchmark's page is in the category and the family
  // of its instance, and the menu marks that family as it marks a category
  // narrowed to it
  const types = element("ul", {}, legend("instances"));
  const at = route.view === "family" ? data.place.get(`${route.suite}/${route.instance}`) : undefined;
  const current = route.view === "category" ? route.category : at ? at.c.key : route.suite ? categoryFor(route.suite)?.key : undefined;
  const family = route.view === "category" ? route.only : at?.g.key;
  for (const c of data.categories) {
    const li = element("li", { class: "cat" }, link({ view: "category", category: c.key }, c.title, element("span", { class: "n" }, count(c.instances))));
    if (route.view === "category" && route.category === c.key && route.only === undefined) li.firstChild.setAttribute("aria-current", "page");
    if (c.key === current && c.key !== ALL) {
      li.append(element("ul", {}, ...c.groups.map((g) => {
        const gl = element("li", {}, link({ view: "category", category: c.key, only: g.key }, g.title, element("span", { class: "n" }, count(g.rows.length))));
        if (g.key === family) gl.firstChild.setAttribute("aria-current", "page");
        return gl;
      })));
    }
    types.append(li);
  }
  types.append(entry("#/coverage", COVERAGE));
  tree.append(group("types", types));
  // the circuits, where the data has them (and the bar's link to them with it): their families
  // where the view is among the circuits, as a type's families under the type on show
  if (data.circuits && sectionOf(route) !== APART) tree.append(group(APART));
  else if (data.circuits) {
    const c = data.circuits;
    // a circuit's page marks its family, as a category narrowed to it does
    const here = route.view === "category" && route.category === APART ? route.only
      : route.view === "family" ? data.circuit.get(`${route.suite}/${route.instance}`)?.key : undefined;
    tree.append(group(APART, element("ul", {}, legend("instances"), ...c.groups.map((g) => {
      const li = element("li", {}, link({ view: "category", category: APART, only: g.key }, g.title, element("span", { class: "n" }, count(g.rows.length))));
      if (here !== undefined && here === g.key) li.firstChild.setAttribute("aria-current", "page");
      return li;
    }))));
  }
  for (const a of document.querySelectorAll(`.site-nav a[href$="#/${APART}"]`)) a.hidden = !data.circuits;
  // how it works
  tree.append(group("how", element("ul", {}, ...WALK_TITLES.map(([key, text]) => entry(`#/how/${key}`, text)), entry("#/methods", METHODS))));
  // on the front page, the bar's section the view is in, and the menu's entry on show: its view, else the entry it is in
  if (!front) {
    for (const a of document.querySelectorAll(".site-nav a[data-section]")) {
      if (a.dataset.section === `#/${sectionOf(route)}`) a.setAttribute("aria-current", "page");
      else a.removeAttribute("aria-current");
    }
  }
  if (!front && route.view !== "missing") {
    const here = href(route);
    const sections = [...tree.querySelectorAll("a[data-section]")];
    const on = sections.find((a) => a.dataset.section === here)
      ?? sections.filter((a) => here.startsWith(`${a.dataset.section}/`)).sort((a, b) => b.dataset.section.length - a.dataset.section.length)[0];
    on?.setAttribute("aria-current", "page");
  }
  document.getElementById("tree").replaceChildren(tree);
}

// What the search finds, by name: the categories, suites, families and
// solvers, then the instances.
let findable = null;
function hits(text) {
  const q = text.trim().toLowerCase();
  if (q.length < 2) return [];
  if (findable === null) {
    findable = [];
    const add = (name, key, what, route) => findable.push({ name, what, route, text: `${name} ${key}`.toLowerCase() });
    for (const c of data.categories) add(c.title, c.key, "type of program", { view: "category", category: c.key });
    for (const s of data.suites.values()) add(s.title, s.key, `suite, ${and(categoriesOf(s.key).map((c) => c.title))}`, { view: "suite", suite: s.key });
    for (const s of data.suites.values()) {
      for (const f of familiesOf(s.key) ?? []) add(f.title, f.key, `family, ${s.title}`, { view: "family", suite: s.key, family: f.key });
    }
    for (const s of data.systems) add(data.names[s.key], s.key, "solver", { view: "system", system: s.key });
    for (const s of data.suites.values()) {
      for (const [name] of s.instances) {
        // a copy without a page of its own is found as the instance it copies
        if (!familyOf(s.key, name) && data.copyOf.has(`${s.key}/${name}`)) continue;
        add(name, "", s.title, instancePage(s.key, name));
      }
    }
  }
  return findable.filter((h) => h.text.includes(q));
}

// The search box, a combobox: what it finds is listed under it, and chosen
// by the pointer, or by the arrow keys and Enter.
function search(text) {
  const input = document.getElementById("search");
  const list = document.getElementById("found-list");
  const all = hits(text);
  const shown = all.slice(0, SHOWN_HITS);
  list.replaceChildren(...shown.map((h, i) => element("li", { role: "option", id: `hit-${i}`, "aria-selected": "false",
    onclick: () => {
      input.value = "";
      search("");
      go(h.route);
    } }, element("span", {}, h.name), element("span", { class: "n" }, h.what))));
  document.getElementById("found-n").textContent = text.trim().length < 2 ? ""
    : all.length > SHOWN_HITS ? `the first ${SHOWN_HITS} of ${count(all.length)}` : plural(all.length, "match", "matches");
  input.setAttribute("aria-expanded", String(shown.length > 0));
  input.removeAttribute("aria-activedescendant");
}

function searchKeys(event) {
  const input = event.target;
  const options = [...document.querySelectorAll("#found-list [role=option]")];
  const active = options.findIndex((o) => o.getAttribute("aria-selected") === "true");
  if ((event.key === "ArrowDown" || event.key === "ArrowUp") && options.length) {
    event.preventDefault();
    const down = event.key === "ArrowDown";
    const next = active < 0 ? (down ? 0 : options.length - 1) : (active + (down ? 1 : -1) + options.length) % options.length;
    options.forEach((o, i) => o.setAttribute("aria-selected", String(i === next)));
    input.setAttribute("aria-activedescendant", options[next].id);
    options[next].scrollIntoView({ block: "nearest" });
  } else if (event.key === "Enter" && options.length) {
    event.preventDefault();
    options[Math.max(active, 0)].click();
  } else if (event.key === "Escape" && input.value) {
    input.value = "";
    search("");
  }
}

// An address that names nothing in the results: what it names, and what
// has a name like its last part.
function missing(route) {
  const q = route.what.split("/").pop().replace(/^@/, "");
  const found = hits(q).slice(0, SHOWN_HITS);
  return [crumbs(HOME, ["Not found"]), element("h2", {}, "Not found"),
    element("p", {}, "No category, solver, suite or benchmark of the results is named ", element("code", {}, route.what), "."),
    ...(found.length
      ? [element("p", {}, "What has a name like ", element("code", {}, q), ":"), element("ul", { class: "like" },
        ...found.map((h) => element("li", {}, link(h.route, h.name), element("span", { class: "n" }, ` ${h.what}`))))]
      : [element("p", {}, "Nothing has a name like it; the search box finds a category, suite, family, system or instance by part of its name.")])];
}

// ------------------------------------------------------------- the page

// The route on show.
let now = { view: "home" };

function go(route) {
  const to = href(route);
  if (to.startsWith("#")) location.hash = to;
  else location.href = to;
}

// What changes in a view without drawing it again (the file on show) goes in
// the address in place, so that a link or a reload shows it again.
function remember(change) {
  now = { ...now, ...change };
  history.replaceState(null, "", href(now));
}

function title(route) {
  switch (route.view) {
    case "category": return categoryOf(route.category).title;
    case "suite": return data.suites.get(route.suite).title;
    case "family": return `${route.instance} · ${route.suite}/${route.family}`;
    case "missing": return "Not found";
    default: return extend.title(route);
  }
}

// The section of a view a link names (a route's `at`, the id of an element
// of the view), scrolled to and marked for a moment.
function reach(id) {
  const to = document.getElementById(id);
  if (!to) return;
  to.scrollIntoView({ block: "start" });
  to.classList.remove("reached");
  void to.offsetWidth;
  to.classList.add("reached");
}

// The view the address names: drawn again where it is another view (or
// `again`), else only what is open in it changes: the file on show, the
// drawer, and a category's filter and the system its times are set against.
function show(again = false) {
  const route = parse(location.hash);
  // a solver's address, as the pages before these named it: its page is the front page's
  if (route.view === "system") {
    location.replace(href(route));
    return;
  }
  const canonical = href(route);
  if (route.view !== "home" && route.view !== "missing" && location.hash !== canonical) {
    history.replaceState(null, "", canonical);
  }
  const before = now;
  now = route;
  const page = href(pageOf(route));
  const view = document.getElementById("view");
  if (again || view.dataset.route !== page) {
    rail(route);
    view.dataset.route = page;
    const parts = route.view === "category" ? categoryView(route) : route.view === "suite" ? suiteView(route)
      : route.view === "family" ? familyView(route, view) : route.view === "missing" ? missing(route) : extend.draw(route) ?? home();
    view.replaceChildren(...parts);
    // A family's view starts at its top, and another instance of it keeps the
    // place; a category's times set against a system start at its plots.
    const family = route.view === "family" ? `${route.suite}/${route.family}` : null;
    if (route.view === "category" && route.vs) document.getElementById("figures").scrollIntoView();
    else if (route.at) reach(route.at);
    else if (!again && (family === null || family !== show.family)) window.scrollTo({ top: 0 });
    show.family = family;
    const t = title(route);
    document.title = t === null ? pageTitle : `${t} · ${pageTitle}`;
  } else if (route.view === "category" && route.vs === before.vs && route.only === before.only
    && String(route.program ?? "") !== String(before.program ?? "")) {
    // another instance of the family on show: its programs drawn again in place
    const c = categoryOf(route.category);
    document.getElementById("programs")?.replaceWith(programSection(c, groupOf(c, route.only), route));
  } else if (route.view === "category" && (route.vs !== before.vs || route.only !== before.only)) {
    // the same category narrowed otherwise, or against another system: drawn
    // again in place, the menu marking the family it is narrowed to
    const menu = document.activeElement?.matches("select.vs");
    if (route.only !== before.only) rail(route);
    view.replaceChildren(...categoryView(route));
    if (menu) view.querySelector("select.vs").focus();
  } else if (route.at && route.at !== before.at) {
    reach(route.at);
  } else if ((route.view === "family" || route.view === "category") && String(route.file ?? "") !== String(before.file ?? "")) {
    const viewer = document.querySelector("#programs .viewer");
    if (viewer?.show) {
      viewer.show(route.file);
      viewer.scrollIntoView({ block: "start" });
    }
  }
  if (route.runs) runs(route);
  else if (!drawer().hidden) closeRuns();
}

document.addEventListener("DOMContentLoaded", async () => {
  const view = document.getElementById("view");
  try {
    // The page works with the networks and programs alone (suites.json) and
    // the categories (categories.json), and the benchmarks' data
    // (benchmarks/, from the baselines branch) where the site has it.
    const [suites, ppl, cats, bench] = await Promise.all([load("suites.json"), load("ppl.json").catch(() => null),
      load("categories.json"), load("benchmarks/index.json").catch(() => null), extend.load()]);
    data.bench = bench;
    for (const [suite, fams] of Object.entries(bench?.suites ?? {})) {
      for (const f of fams) for (const name of f.instances) data.family.set(`${suite}/${name}`, f);
    }
    for (const [file, part] of [["suites.json", suites], ["ppl.json", ppl]]) {
      if (part === null) continue;
      Object.assign(data.names, part.names);
      for (const f of part.families) for (const s of f.suites) data.suites.set(s.key, { ...s, home: f, file });
    }
    Object.assign(data.names, cats.names);
    data.measure = cats.measure;
    data.own = cats.own;
    data.suiteVs = cats.suites;
    data.systems = cats.systems;
    data.integrations = cats.integrations ?? null;
    data.splits = cats.splits ?? {};
    for (const k of ["stamp", "methods"]) if (!cats[k]) throw new Error(`categories.json has no ${k}`);
    data.stamp = cats.stamp;
    data.methods = cats.methods;
    for (const s of data.systems) if (!data.names[s.key]) throw new Error(`the solver ${s.key} has no name`);
    data.categories = [...cats.categories].sort((a, b) => a.order - b.order);
    data.types = data.categories.filter((c) => c.key !== ALL);
    data.circuits = cats.circuits ?? null;
    // Each category's suites, its rows' and then its copies', with how many
    // of its instances each holds; the circuits' too, whose instances are in
    // their types (data.place) where they are of one
    for (const c of [...data.categories, ...(data.circuits ? [data.circuits] : [])]) {
      c.perSuite = new Map();
      const add = (key) => c.perSuite.set(key, (c.perSuite.get(key) ?? 0) + 1);
      for (const row of c.rows) add(row[0]);
      for (const [key, copies] of Object.entries(c.copies)) {
        for (const copy of copies) {
          add(copy.split("/")[0]);
          data.copyOf.set(copy, key);
        }
      }
      c.suites = [...c.perSuite.keys()];
      for (const key of c.suites) if (!data.suites.has(key)) throw new Error(`the category ${c.key} has no suite ${key}`);
      if (c.key === ALL) continue;
      for (const g of c.groups) {
        for (const i of g.rows) {
          const key = `${c.rows[i][0]}/${c.rows[i][1]}`;
          for (const k of [key, ...(c.copies[key] ?? [])]) (c.key === APART ? data.circuit : data.place).set(k, c.key === APART ? g : { c, g });
        }
      }
    }
    data.version = suites.version;
    window.addEventListener("hashchange", () => show());
    const input = document.getElementById("search");
    input.addEventListener("input", () => search(input.value));
    input.addEventListener("keydown", searchKeys);
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !drawer().hidden) go({ ...now, runs: undefined });
    });
    show();
  } catch (failure) {
    view.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
