"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl,
// Roulette and Geni), both written by the generator of README.md's results
// and the baselines branch's docs/benchmarks.md; the results by type of
// program from categories.json (release/results/rel/panels.py); and, where
// the site has them, the benchmarks' data from the baselines branch
// (benchmarks/, which release/pages/bench_pages.py writes): each suite's
// families of instances and, for each family, its programs in every
// language, its runs and its answers.
//
// The page is one view at a time, named by the address's hash:
//   #/                       the README's figures: each category's total times, cactus and scatter plots
//   #/category/C[/in/FAMILY][/vs/SYSTEM]
//                            a category: its systems' answers and totals, its plots (tddlog against
//                            SYSTEM), narrowed to FAMILY (the program or model whose sizes they are)
//                            or to own:SYSTEM (a system's own benchmarks)
//   #/system/SYSTEM          a solver: its plots against tddlog on the programs in its own language
//                            and on those translated for it, and against the others in each category
//                            it was given, its totals, how it was given the instances, its version,
//                            settings, paper and link
//   #/SUITE                  a suite: its plot, tddlog against each system, its families
//   #/SUITE/FAMILY[/NAME]    a family: its instances' times, and an instance's programs and answers
//   #/SUITE/@NAME            the family of the instance NAME
// and after an instance's name, what is open in its view:
//   …/tab/T[/F]              the file F (0 where left out) of the instance's language T
//   …/runs/SYSTEM            SYSTEM's runs of the instance, in a drawer beside the view
// A time in a family's table opens that system's runs of the instance
// beside the view. A category's instances, systems, answers and total times
// are the data's (the README's figures draw the same): the page does not
// choose them. Narrowed to a source, it sums the same rows' times; it
// computes the scatter plots, as the README's figures do.

// The stamp of the page's own reference (results.js?v=<sha>) goes on to the
// data, as the playground's does to its files.
const stamp = document.currentScript === null ? "" : new URL(document.currentScript.src).search;

// The time axis starts at 0.2 s: runs under 0.2 s are not told apart.
const FLOOR = 0.2;

// How a failure reads in a table cell, and what it means.
const FAILURES = {
  timeout: [(limit) => `>${limit}`, "the time limit ran out"],
  memout: [() => "memory", "out of memory (the address space a run gets, or the Java heap)"],
  error: [() => "error", "the system stopped with an error"],
  unsupported: [() => "n/a", "the system cannot express the program: a translator refuses what its target language lacks (lists, division and recursion in a Dice program's translation; negation through recursion, or more than two million conjunctions, in a ground program's network), as aspmc's harness refuses an observation with a body"],
  skipped: [() => "skipped", "not run: the system had run out of time or memory on a smaller size of the same program"],
  wrong: [() => "wrong", "an answer off the reference by more than 1e-9 and either by more than 1e-6 or by more than 1e-3 of it, beyond the last digit either of the two prints"],
  zero: [() => "zero", "every answer printed as 0 where the reference's are positive"],
  underflow: [() => "underflow", "every answer printed as 0 where the probability of the evidence is below the smallest double"],
};
// A time in a table is shaded by how many times slower it is than its
// instance's fastest, up to this many.
const SLOWEST = 100;

// The most lines of a file the page shows; the whole file can be saved.
const SHOWN_LINES = 2000;

// The highlighter's language (code.js) of a file, by its name's ending.
const HIGHLIGHT = { pl: "problog", py: "python", rs: "rust", c: "c", h: "c", toml: "toml" };

// The fields of a file the page reads for itself; any other text a file
// has (how it was made, a licence) is a line of HTML above its text.
const FILE_FIELDS = new Set(["name", "bytes", "lines", "sha", "src", "part", "gzip", "held", "archive"]);

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

// One system's total time against another's: 12×, 1.5×.
function times(a, b) {
  const r = a / b;
  return `${r >= 10 ? r.toFixed(0) : r.toFixed(1)}×`;
}

function plural(n, one, many = `${one}s`) {
  return `${count(n)} ${n === 1 ? one : many}`;
}

const answered = (value) => typeof value === "number";
// A process time (Alea.jl's and Roulette's with their start-up) is drawn
// dashed, in its system's colour.
const base = (key) => key.replace(/-process$/, "");
const dashed = (key) => key.endsWith("-process");

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
  if (!answered(v)) return { cls: v === "unsupported" ? "na" : "fail", p: 0 };
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
    na ? element("span", {}, element("i", { class: "heat na" }), "cannot express it") : null].filter(Boolean);
}

// A system's sample of line: in its colour, dashed for a process's time, or
// grey where its line in a plot is.
function swatch(key, grey = false) {
  return element("i", { class: `sw s-${base(key)}${dashed(key) ? " dashed" : ""}${grey ? " grey" : ""}` });
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
  const names = [...table.tHead.rows[0].cells].map((th) => th.textContent);
  for (const row of table.tBodies[0].rows) {
    if (row.cells.length !== names.length) continue;
    [...row.cells].forEach((cell, i) => {
      if (cell.tagName === "TD") cell.dataset.label = names[i];
    });
  }
  table.classList.add("stack");
  return table;
}

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
    element("a", { href: file }, file.split("/").pop())]), ".");
}

// ------------------------------------------------------------- the plot

// The lines of a plot: each system's answers within the suite's limit, in
// order, tddlog's first and then by how many each answers.
function linesOf(suite) {
  return suite.systems.map((key, column) => ({ key,
    times: suite.instances.map((row) => row[column + 1]).filter(answered).filter((t) => t <= suite.limit * 1.001).sort((a, b) => a - b) }))
    .sort((a, b) => (b.key === "tddlog") - (a.key === "tddlog") || b.times.length - a.times.length);
}

// The instances each system answers within a time limit, the limit on a
// linear scale from 0 to the suite's; a line further up and to the left is
// better. Drawn at the width of its box; the lines not `coloured` grey and
// under the others. Each line is named at its end, by its system where there
// is room, else by its number in the legend (the coloured lines only).
// Returns whether the plot is narrow.
function cactus(box, suite, names, series, coloured) {
  const width = Math.max(280, Math.floor(box.clientWidth));
  const narrow = width < 480;
  const height = narrow ? 300 : Math.max(380, series.length * 14 + 22 + 46);
  const n = suite.instances.length;
  const hi = suite.limit;
  const ends = series.map(({ key, times }, i) => ({ key, y: times.length, text: narrow ? String(i + 1) : `${names[key]} ${times.length}` }))
    .filter((end) => !narrow || coloured.has(end.key));
  // The end labels' room: the longest at 7 px a character.
  const left = 46, right = narrow ? 30 : Math.max(128, 16 + 7 * Math.max(0, ...ends.map((end) => end.text.length))), top = 22, bottom = 46;
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
  const order = [...series].sort((a, b) => coloured.has(a.key) - coloured.has(b.key) || (a.key === "tddlog") - (b.key === "tddlog"));
  for (const { key, times } of order) {
    let k = 0;
    let d = `M${x0},${ly(0)}`;
    for (const t of times) {
      d += `H${lx(t).toFixed(1)}V${ly(++k).toFixed(1)}`;
    }
    d += `H${x1}`;
    plot.append(svg("path", { class: `line s-${base(key)}${key === "tddlog" ? " mine" : ""}${dashed(key) ? " dashed" : ""}${coloured.has(key) ? "" : " grey"}`,
      d, "data-key": key }));
  }
  // The end labels: each line's name at its last height, moved apart where
  // two would overlap: pushed down, the lowest kept above the axis and those
  // over it pushed up, so that a crowd at the bottom moves up only as far as
  // it must.
  const labels = ends.map((end) => ({ ...end, y: ly(end.y) })).sort((a, b) => a.y - b.y);
  for (let i = 1; i < labels.length; i++) labels[i].y = Math.max(labels[i].y, labels[i - 1].y + 13);
  if (labels.length) labels[labels.length - 1].y = Math.min(labels[labels.length - 1].y, y1 + 4);
  for (let i = labels.length - 2; i >= 0; i--) labels[i].y = Math.min(labels[i].y, labels[i + 1].y - 13);
  for (const l of labels) {
    plot.append(svg("text", { class: `end${coloured.has(l.key) ? "" : " grey"}`, x: x1 + 8, y: l.y + 4, "data-key": l.key }, l.text));
  }

  // The hover layer: a line at a time limit, and how many each system
  // answers within it; the limit under the pointer, where a finger taps (it
  // stays until the next tap), or moved by the arrow keys.
  const cross = svg("line", { class: "cross", x1: 0, y1: y0, x2: 0, y2: y1, visibility: "hidden" });
  plot.append(cross);
  const area = svg("rect", { class: "hit", x: x0, y: y0, width: x1 - x0, height: y1 - y0 });
  plot.append(area);
  const tip = element("div", { class: "tip", hidden: true, "aria-live": "polite" });
  let cx = null;
  const at = (x) => {
    cx = Math.min(Math.max(x, x0), x1);
    // a time under 0.2 s counts as 0.2 s
    const t = Math.max(FLOOR, (cx - x0) / (x1 - x0) * hi);
    cross.setAttribute("x1", cx);
    cross.setAttribute("x2", cx);
    cross.setAttribute("visibility", "visible");
    tip.replaceChildren(element("b", {}, `within ${seconds(t)} s`));
    const rows = series.map(({ key, times }) => [key, times.filter((s) => s <= t).length]).sort((a, b) => b[1] - a[1]);
    for (const [key, k] of rows) tip.append(element("span", {}, swatch(key, !coloured.has(key)), `${names[key]} ${k}`));
    tip.hidden = false;
    const w = tip.offsetWidth;
    const px = cx + 12 + w > width ? cx - 12 - w : cx + 12;
    tip.style.left = `${Math.max(0, px)}px`;
    tip.style.top = `${y0}px`;
  };
  const hide = () => {
    cross.setAttribute("visibility", "hidden");
    tip.hidden = true;
  };
  const pointer = (event) => at(event.clientX - plot.getBoundingClientRect().left);
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
// instances it answers, and in a kind's plot its total time (total:
// {system: seconds}, the data's); in a narrow plot, the number its line
// ends with. A name is a button that colours its line or greys it, and a
// name under the pointer or the keyboard brings its line forward.
function legend(series, names, n, figure, total, coloured, narrow, recolour) {
  const list = element("ul", { class: "legend" });
  series.forEach(({ key, times }, i) => {
    const counts = `answers ${count(times.length)} of ${count(n)}${total === undefined ? "" : `, total ${duration(total[key])}`}`;
    const button = element("button", { type: "button", class: "bare", "data-system": key, "aria-pressed": String(coloured.has(key)),
      onclick: () => recolour(key) },
      narrow ? element("span", { class: "no" }, String(i + 1)) : null, swatch(key, !coloured.has(key)),
      element("span", {}, names[key]), element("span", { class: "n" }, counts));
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
// is coloured or greyed. Where there are more than five lines, tddlog's and
// the three that answer the most are in colour, and `also`'s, and the others
// grey until asked for; with `all`, every line is in colour (a category's,
// whose colours go by family). The figure names the systems it draws and the
// number of its instances; a category's legend gives each system's total
// time too (suite.total). A `bare` plot has no legend where its lines are
// named at their ends, nor a button to colour them all.
function plotted(suite, names, { also, bare, all, colour } = {}) {
  const series = linesOf(suite);
  const top = colour ? series.map((s) => s.key).filter((key) => colour.includes(key))
    : all ? series.map((s) => s.key) : series.slice(0, series.length > 5 ? 4 : series.length).map((s) => s.key);
  if (also && !top.includes(also)) top.push(also);
  let coloured = new Set(top);
  const figure = element("figure", { class: "plot", "data-systems": suite.systems.join(" "),
    "data-instances": String(suite.instances.length) });
  const box = element("div", { class: "box" });
  const every = series.length > top.length && !bare ? element("button", { type: "button", class: "plain every", onclick: () => {
    coloured = new Set(coloured.size === series.length ? top : series.map((s) => s.key));
    draw();
  } }) : null;
  figure.append(box);
  const draw = () => {
    const narrow = cactus(box, suite, names, series, coloured);
    if (bare && !narrow) {
      figure.querySelector(".legend")?.remove();
      return;
    }
    const focused = figure.contains(document.activeElement) ? document.activeElement.closest(".legend button")?.dataset.system : undefined;
    const fresh = legend(series, names, suite.instances.length, figure, suite.total, coloured, narrow, (key) => {
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
        : colour ? `Colour ${and(top.map((key) => `${names[key]}'s`))} lines only` : "Colour tddlog's and the top three lines only";
    }
    if (focused) fresh.querySelector(`button[data-system="${CSS.escape(focused)}"]`)?.focus();
  };
  onWidth(box, draw);
  return figure;
}

// ------------------------------------------------------------- the README's figures

// A category's instances (the data's rows), narrowed to `only`: each with
// its suite, its time limit, its systems' classes (a letter each: o own, n
// native, t translated, - not given), each system's record `v` and its time
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

// The benchmarks' pages of rows (catRows'): each family of the benchmarks'
// data that holds some of them (without that data, each suite), with how
// many (n) and how many of them tddlog answers (a), named by its title and,
// where two share a title, its suite's too.
function pagesOf(rows) {
  const out = new Map();
  for (const r of rows) {
    const f = familyOf(r.suite, r.name), s = data.suites.get(r.suite);
    const key = f ? `${r.suite}/${f.key}` : r.suite;
    if (!out.has(key)) {
      out.set(key, { route: f ? { view: "family", suite: r.suite, family: f.key } : { view: "suite", suite: r.suite },
        title: f ? f.title : s.title, suite: s.title, n: 0 });
    }
    out.get(key).n++;
    if (r.t.tddlog !== null) out.get(key).a = (out.get(key).a ?? 0) + 1;
  }
  const list = [...out.values()];
  const twice = new Set(list.map((p) => p.title).filter((t, i, all) => all.indexOf(t) !== i));
  return list.map((p) => ({ ...p, label: twice.has(p.title) && p.title !== p.suite ? `${p.title} (${p.suite})` : p.title }));
}

// A run's cost in a total: its time, at least 0.2 s, or its instance's limit
// where it does not answer.
const cost = (r, x) => (r.t[x] === null ? r.limit : Math.max(r.t[x], FLOOR));

const LETTERS = { o: "own", n: "native", t: "translated" };

// Each system's instances given, answers, total time and classes over rows:
// the data's where rows are the whole category, else summed from the rows.
function tally(c, rows, only) {
  const out = {};
  c.systems.forEach((x, j) => {
    if (only === undefined) {
      out[x] = { given: c.given[x], answers: c.answers[x], total: c.total[x], classes: c.classes[x] };
      return;
    }
    const classes = { own: 0, native: 0, translated: 0 };
    for (const r of rows) if (r.cls[j] !== "-") classes[LETTERS[r.cls[j]]]++;
    out[x] = { given: rows.filter((r) => r.v[x] !== null).length, answers: rows.filter((r) => r.t[x] !== null).length,
      total: rows.reduce((t, r) => t + cost(r, x), 0), classes };
  });
  return out;
}

// The fastest of xs on each row: how many rows it answers and its total.
function virtual(rows, xs) {
  let answers = 0, total = 0;
  for (const r of rows) {
    const ts = xs.map((x) => r.t[x]).filter((t) => t !== null);
    if (ts.length) answers++;
    total += ts.length ? Math.max(FLOOR, Math.min(...ts)) : r.limit;
  }
  return { answers, total };
}

// The systems drawn: the category's, given at least 20 of its instances;
// narrowed, every system given one of the rows.
function drawnOf(c, rows, only) {
  if (only === undefined) return c.drawn;
  return c.systems.filter((x) => x === "tddlog" || rows.some((r) => r.v[x] !== null));
}

// The other systems by their answers, then their total time.
const ranked = (xs, tal) => xs.filter((x) => x !== "tddlog")
  .sort((a, b) => tal[b].answers - tal[a].answers || tal[a].total - tal[b].total);

// A category's rows as one suite for a plot: the systems drawn, tddlog first
// and then by the instances they answer, as the README's legends list them;
// the limit the longest of its instances'; each system's total time.
function catSuite(c, rows, drawn, tal) {
  const systems = ["tddlog", ...ranked(drawn, tal)];
  return { key: c.key, title: c.title, noun: c.noun, systems, instances: rows.map((r) => [r.name, ...systems.map((x) => r.v[x])]),
    limit: Math.max(...rows.map((r) => r.limit)), total: Object.fromEntries(systems.map((x) => [x, tal[x].total])) };
}

// How many times less time tddlog takes than another system: 49×, 1.7×,
// or 1.3× slower.
function multiple(mine, theirs) {
  return theirs >= mine ? times(theirs, mine) : `${times(mine, theirs)} slower`;
}

// A category's headline: tddlog's answers and total time against the best
// other system's (the most answers) and the virtual best of all others.
function headline(c, rows, tal, only) {
  const others = c.systems.filter((x) => x !== "tddlog");
  const best = only === undefined ? c.best : ranked(others, tal)[0];
  const vb = only === undefined ? c.vbest : virtual(rows, others);
  const me = tal.tddlog;
  return element("p", { class: "headline" }, element("b", {}, `tddlog ${count(me.answers)} of ${count(rows.length)}`),
    ` in ${duration(me.total)}`,
    best ? `; the best other system, ${data.names[best]}, ${count(tal[best].answers)} in ${duration(tal[best].total)}` : "",
    others.length > 1 ? `; the virtual best of all others ${count(vb.answers)} in ${duration(vb.total)}` : "", ".");
}

// The time limits of the data, in words: "300, 600 or 1200 s".
function limitsText() {
  const limits = [...new Set(data.categories.flatMap((c) => c.rows.map((r) => r[2])))].sort((a, b) => a - b);
  return `${limits.length > 1 ? `${limits.slice(0, -1).join(", ")} or ` : ""}${limits.at(-1)} s`;
}

// The README's overview: for each category, tddlog's answers and total time
// and each family's virtual best's (the fastest of its systems on each
// instance, the data's), on one time axis, and how many times less time
// tddlog takes. A category's title or row opens the category.
function overview() {
  const box = element("div", { class: "box" });
  const figure = element("figure", { class: "overview" }, box,
    element("figcaption", { class: "note" }, `The release run of tddlog ${data.version}. Each instance counts in one category, `
      + "and a family's line is the fastest of its systems on each instance. A run that does not answer (out of time "
      + "or memory, a wrong answer, a program the system cannot express, or not given) counts as its time limit, "
      + `${limitsText()}, and a time under ${FLOOR} s as ${FLOOR} s. A category opens its details.`));
  onWidth(box, () => drawOverview(box));
  return figure;
}

function drawOverview(box) {
  const groups = data.categories.map((c) => ({ c,
    rows: Object.entries(c.vfamily).map(([key, v]) => ({ key, ...v })).sort((a, b) => a.total - b.total) }));
  const titles = Object.fromEntries(data.groups.map((f) => [f.key, f]));
  const width = Math.max(300, Math.floor(box.clientWidth));
  const narrow = width < 640;
  const row = 23, head = 32, top = 46, bottom = 30;
  const nameRight = narrow ? 112 : 190, answersRight = nameRight + 38;
  const ratio = width - (narrow ? 56 : 110);
  const x0 = answersRight + 14, x1 = ratio - (narrow ? 10 : 18);
  const all = groups.flatMap((g) => [g.c.total.tddlog, ...g.rows.map((r) => r.total)]);
  const lo = Math.min(...all) / 3, hi = Math.max(...all) * 3;
  const lx = (t) => x0 + Math.log(t / lo) / Math.log(hi / lo) * (x1 - x0);
  const height = top + groups.reduce((h, g) => h + head + row * (g.rows.length + 1), 0) + bottom;
  const plot = svg("svg", { class: "overview", width, height, viewBox: `0 0 ${width} ${height}`, role: "group",
    "aria-label": "Answers and total time of tddlog and of each family's virtual best, by type of program" });
  const ticks = [];
  for (const tick of [[1, "1 s"], [10, "10 s"], [60, "1 min"], [600, "10 min"], [3600, "1 h"], [36000, "10 h"], [360000, "100 h"]]) {
    // each at least 44 px from the one before, so that their names do not touch
    if (tick[0] < lo || tick[0] > hi || (ticks.length && lx(tick[0]) - lx(ticks.at(-1)[0]) < 44)) continue;
    ticks.push(tick);
  }
  for (const [t, label] of ticks) {
    plot.append(svg("line", { class: "grid", x1: lx(t), y1: top - 6, x2: lx(t), y2: height - bottom + 4 }));
    for (const y of [top - 12, height - 10]) plot.append(svg("text", { class: "tick", x: lx(t), y, "text-anchor": "middle" }, label));
  }
  plot.append(svg("text", { class: "col", x: answersRight, y: top - 12, "text-anchor": "end" }, "answers"));
  plot.append(svg("text", { class: "col", x: ratio, y: top - 25 }, "tddlog"), svg("text", { class: "col", x: ratio, y: top - 12 }, "faster"));
  let y = top;
  // A row: its name and answers, its dots and their times, and the whole row
  // a link.
  const line = (to, label, cls, name, answers, ...marks) => {
    const a = svg("a", { href: href(to), class: `row ${cls}`, "aria-label": label });
    a.append(svg("title", {}, label), svg("rect", { class: "hit", x: 0, y, width, height: row }),
      svg("text", { class: "name", x: nameRight, y: y + row / 2 + 4, "text-anchor": "end" }, name),
      svg("text", { class: "answers", x: answersRight, y: y + row / 2 + 4, "text-anchor": "end" }, count(answers)), ...marks);
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
  for (const { c, rows } of groups) {
    const to = { view: "category", category: c.key };
    const title = svg("a", { href: href(to), class: "kind" });
    title.append(svg("text", { x: 0, y: y + head - 9 }, `${c.title}, ${count(c.instances)} ${c.noun}`));
    plot.append(title);
    y += head;
    const mine = c.total.tddlog;
    line(to, `tddlog: ${count(c.answers.tddlog)} of ${count(c.instances)} ${c.noun} answered, ${duration(mine)} in total`,
      "mine", "tddlog", c.answers.tddlog, ...at(mine, "me", -1));
    for (const r of rows) {
      const f = titles[r.key];
      const faster = multiple(mine, r.total);
      const [left, right] = mine <= r.total ? [-1, 1] : [1, -1];
      line(to, `${f.title}, the fastest of its systems on each instance: ${count(r.answers)} of ${count(c.instances)} answered, `
        + `${duration(r.total)} in total against tddlog's ${duration(mine)}, tddlog ${faster}`,
      "other", narrow ? f.short : f.title, r.answers,
      svg("line", { class: "span", x1: lx(mine), y1: y + row / 2, x2: lx(r.total), y2: y + row / 2 }),
      ...at(r.total, `you f-${r.key}`, right), ...at(mine, "me", left),
      svg("text", { class: `ratio${r.total >= mine ? "" : " slower"}`, x: ratio, y: y + row / 2 + 4 }, faster));
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
function scatter(rows, drawn, vs, choose) {
  const box = element("div", { class: "box" });
  const head = element("p", { class: "sc-head" });
  const foot = element("p", { class: "note" });
  const figure = element("figure", { class: "scatter" });
  const others = drawn.filter((x) => x !== "tddlog");
  if (choose) {
    const menu = element("select", { class: "vs", "aria-label": "The system tddlog's times are set against",
      onchange: () => go({ ...now, vs: menu.value || undefined }) },
      element("option", { value: "" }, "the fastest other system"),
      ...[...others, ...(vs && !others.includes(vs) ? [vs] : [])].map((x) => element("option", { value: x }, data.names[x])));
    menu.value = vs ?? "";
    figure.append(element("label", { class: "pick" }, "tddlog against ", menu));
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
    const mine = r.t.tddlog, theirs = best === null ? null : r.t[best];
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
  head.replaceChildren(element("b", { class: "win" }, `tddlog faster on ${count(wins)}`), element("b", { class: "loss" }, `slower on ${count(losses)}`));
  const who = vs ? data.names[vs] : "the fastest other system";
  const left = [fast ? `${plural(fast, "instance")} both answer in under ${FLOOR} s` : "", neither ? `${count(neither)} neither answers` : ""].filter(Boolean);
  foot.replaceChildren(`A dot is an instance: tddlog's time against ${who}'s; it opens the instance.${left.length ? ` Not drawn: ${and(left)}.` : ""}`);
  onWidth(box, () => drawScatter(box, points, limit, fail, vs));
  return figure;
}

function drawScatter(box, points, limit, fail, vs) {
  const who = vs ? data.names[vs] : "the fastest other system";
  const side = Math.min(400, Math.max(260, Math.floor(box.clientWidth)));
  const left = 48, bottom = 42, top = 6, right = 8;
  const x0 = left, x1 = side - right, y0 = top, w = x1 - x0;
  const lim = [FLOOR / 1.15, fail * 1.3];
  const scale = (v) => Math.log(v / lim[0]) / Math.log(lim[1] / lim[0]) * w;
  const px = (v) => x0 + scale(v), py = (v) => y0 + w - scale(v);
  const height = y0 + w + bottom;
  const plot = svg("svg", { class: "scatter", width: side, height, viewBox: `0 0 ${side} ${height}`, role: "img",
    "aria-label": `tddlog's time against ${who}'s, an instance a dot` });
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
    svg("text", { class: "label", transform: `translate(12 ${y0 + w / 2}) rotate(-90)`, "text-anchor": "middle" }, "tddlog (s)"));
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
      element("span", {}, `tddlog: ${time(near.r.t.tddlog)}`),
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
  groups: [],       // the families of systems: key, title, short, systems
  own: [],          // each system on its own benchmarks against tddlog
  systems: [],      // each solver, by the instances it answers: what it is, its totals, how it was given the instances
  suites: new Map(),   // a suite's key: the suite, with `home`, the data's family that holds it, and `file`
  bench: null,      // benchmarks/index.json: every suite's families, where the site has them
  families: new Map(), // `${suite}/${family}`: the promise of that family's data
  family: new Map(),   // `${suite}/${instance}`: the instance's family
  place: new Map(),    // `${suite}/${instance}`: the category and the family of its instances (c.groups) that hold it
};

async function load(file) {
  const response = await fetch(file + stamp);
  if (!response.ok) throw new Error(`${file}: ${response.status}`);
  return response.json();
}

// A family's data, loaded once.
function familyData(suite, key) {
  const id = `${suite}/${key}`;
  if (!data.families.has(id)) data.families.set(id, load(`benchmarks/${suite}/${key}.json`));
  return data.families.get(id);
}

const categoriesOf = (suite) => data.categories.filter((c) => c.perSuite.has(suite));
const familiesOf = (suite) => data.bench?.suites?.[suite] ?? null;
const familyOf = (suite, name) => data.family.get(`${suite}/${name}`);
// The column of a system in a suite's rows (its first is the instance).
const column = (suite, key) => suite.systems.indexOf(key) + 1;
// The record's system of a column: alea-process's runs are Alea.jl's.
const recorded = (key) => data.bench?.records?.[key] ?? base(key);
// Whether a tab's reader is (a form of) the system of this name: "Ace auto"
// and "Ace forceC2d" are Ace's.
const readsAs = (name) => (reader) => reader === name || reader.startsWith(`${name} `);

// tddlog against each other system of a suite, as the findings write it:
// over the instances both are given, how many each answers; over those that
// either answers, each one's total time, a run that does not answer counted
// at the suite's limit and a time under 0.2 s at 0.2 s.
function against(suite) {
  const me = column(suite, "tddlog");
  if (me === 0) return [];
  const cost = (v) => (answered(v) ? Math.max(v, FLOOR) : suite.limit);
  return suite.systems.filter((key) => key !== "tddlog").map((key) => {
    const j = column(suite, key);
    const both = suite.instances.filter((row) => row[me] !== null && row[j] !== null);
    const either = both.filter((row) => answered(row[me]) || answered(row[j]));
    return {
      key, given: both.length, mine: both.filter((row) => answered(row[me])).length,
      theirs: both.filter((row) => answered(row[j])).length, either: either.length,
      myTotal: either.reduce((t, row) => t + cost(row[me]), 0), theirTotal: either.reduce((t, row) => t + cost(row[j]), 0),
    };
  }).filter((r) => r.given > 0);
}

// ------------------------------------------------------------- the address

// The kinds of problems of the page before this one, and the category each
// one's address opens now.
const OLD_KINDS = { programs: "logic-programs", networks: "bayesian-networks", ppl: "functional-programs",
  circuits: "bit-blasted", databases: "database-queries" };

// The view an address names, or what it names that the data does not have.
function parse(hash) {
  const old = hash.replace(/^#/, "");
  if (old === "" || old === "/") return { view: "home" };
  // the pages before this one named a family of suites, or a kind, by its key
  if (!old.startsWith("/")) {
    if (OLD_KINDS[old]) return { view: "category", category: OLD_KINDS[old] };
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
  if (parts[0] === "kind") {
    const c = data.categories.find((x) => x.key === OLD_KINDS[parts[1]]);
    if (!c || parts.length > 3) return missing;
    return { view: "category", category: c.key, vs: c.systems.includes(parts[2]) && parts[2] !== "tddlog" ? parts[2] : undefined };
  }
  if (parts[0] === "category") {
    const c = data.categories.find((x) => x.key === parts[1]);
    if (!c) return missing;
    const route = { view: "category", category: c.key };
    for (let i = 2; i < parts.length; i += 2) {
      const [what, key] = [parts[i], parts[i + 1]];
      if (what === "in" && filtersOf(c).some((f) => f.key === key)) route.only = key;
      else if (what === "in" && (data.suites.has(key) || key === "uai2022")) continue;   // a suite, as the page once narrowed to
      else if (what === "vs" && c.systems.includes(key) && key !== "tddlog") route.vs = key;
      else return missing;
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
  const fams = familiesOf(suite);
  if (!fams) return { view: "suite", suite };   // the site has no benchmarks' data
  // FAMILY[/NAME] or @NAME, then what is open in the instance's view
  const named = parts[1].startsWith("@");
  const instance = named ? parts[1].slice(1) : parts[2];
  const family = named ? familyOf(suite, instance) : fams.find((f) => f.key === parts[1]);
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
    case "category": return `#/category/${route.category}${route.only ? `/in/${encodeURIComponent(route.only)}` : ""}`
      + `${route.vs ? `/vs/${encodeURIComponent(route.vs)}` : ""}`;
    case "system": return `#/system/${encodeURIComponent(route.system)}`;
    case "suite": return `#/${route.suite}`;
    case "family": {
      let out = `#/${route.suite}/${encodeURIComponent(route.family)}${route.instance ? `/${encodeURIComponent(route.instance)}` : ""}`;
      if (route.file) out += `/tab/${route.file[0]}${route.file[1] ? `/${route.file[1]}` : ""}`;
      if (route.runs) out += `/runs/${encodeURIComponent(route.runs)}`;
      return out;
    }
    case "missing": return `#/${route.what}`;
    default: return "#/";
  }
}

// A view without what is open in it: the file on show, the drawer, and a
// category's filter and the system its times are set against.
const pageOf = (route) => ({ ...route, file: undefined, runs: undefined, vs: undefined, only: undefined });

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

function crumbs(...items) {
  const nav = element("nav", { class: "crumbs", "aria-label": "Where this is" });
  items.forEach(([text, route], i) => {
    if (i) nav.append(element("span", { "aria-hidden": "true" }, "›"));
    nav.append(route ? link(route, text) : element("span", { "aria-current": "page" }, text));
  });
  return nav;
}

// ------------------------------------------------------------- the views

// The overview: the README's figures, each part a link to its details:
// tddlog's and each family's total times in every category, then for each
// category its headline, its cactus plot beside tddlog's times against the
// fastest other system's, and each system on its own benchmarks.
function home() {
  const out = [element("div", { class: "fig-head" }, element("h2", {}, "Total time over every instance of a category"),
    element("span", { class: "dots" }, element("i", { class: "me" }), "tddlog", element("i", { class: "you" }), "a family's virtual best")),
  overview()];
  for (const c of data.categories) {
    const to = { view: "category", category: c.key };
    const rows = catRows(c);
    const tal = tally(c, rows);
    out.push(element("section", { class: "kind-figs" },
      element("div", { class: "fig-head" }, element("h3", {}, link(to, c.title)),
        element("span", { class: "n" }, `${count(c.instances)} ${c.noun}, ${plural(c.systems.length - 1, "other system")}`),
        link(to, "Every system and family")),
      headline(c, rows, tal),
      element("div", { class: "pair" }, plotted(catSuite(c, rows, c.drawn, tal), data.names, { bare: true, all: true }),
        scatter(rows, c.drawn))));
  }
  out.push(...ownTable(), how(), raw("categories.json"));
  return out;
}

// Each system on its own benchmarks (from its papers, benchmarks or
// distribution) against tddlog on the same instances; a system opens the
// category that holds most of them, narrowed to them. `list` is the rows
// shown (all of them, or one system's), `head` their heading.
function ownTable(list = data.own, head = "Each system on its own benchmarks") {
  const body = element("tbody");
  for (const o of list) {
    const cs = data.categories.filter((c) => c.classes[o.system]?.own > 0)
      .sort((a, b) => b.classes[o.system].own - a.classes[o.system].own);
    const name = cs.length ? link({ view: "category", category: cs[0].key, only: `own:${o.system}` }, data.names[o.system]) : data.names[o.system];
    body.append(element("tr", {}, element("th", { scope: "row" }, swatch(o.system), name), prose("td", o.about, { class: "about" }),
      element("td", { class: "num" }, count(o.instances)),
      element("td", { class: "num" }, `${count(o.answers.tddlog)} in ${duration(o.total.tddlog)}`),
      element("td", { class: "num" }, `${count(o.answers[o.system])} in ${duration(o.total[o.system])}`)));
  }
  return [element("h3", { class: "own-head" }, head), scroller(stacked(element("table", { class: "board own" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"), element("th", { scope: "col" }, "its benchmarks"),
      element("th", { scope: "col", class: "num" }, "instances"), element("th", { scope: "col", class: "num" }, "tddlog"),
      element("th", { scope: "col", class: "num" }, "the system"))), body)), "whole")];
}

// How the plots and totals are read, under every overview.
function how() {
  return element("details", { class: "how" }, element("summary", {}, "How to read these results"),
    element("p", {}, "The results are partitioned by type of program: each instance counts in one category, whatever "
      + "language its source was written in, and a copy of an instance (the same files) counts once. A UAI 2022 network "
      + "is a Bayesian network when its factors are conditional probability tables over an acyclic graph, else a Markov "
      + "network. Within a category, the instances are grouped by family: the program or model whose sizes they are, "
      + "wherever it was written."),
    element("p", {}, "Every system given some instance of a category is measured on all of its instances, and an instance "
      + "a system does not answer counts as unanswered at its time limit, whatever the reason: out of time or memory, a "
      + `wrong answer, a program the system cannot express, or one it was not given. A time under ${FLOOR} s counts as `
      + `${FLOOR} s. A system run in several forms counts as its fastest form on each instance, and a virtual best is the `
      + "fastest of several systems on each instance. On each instance a system is marked own (from its own papers, "
      + "benchmarks or distribution), native (the program it read is in its own language, as the benchmark's authors or "
      + "generator wrote it) or translated (a translator wrote the program it read from a program in another language, "
      + "or a converter wrote it from the network in another format)."),
    element("p", {}, "A plot counts the instances each system answers within a time limit, on a linear scale from 0 to "
      + "the longest limit: a line further up and to the left is better. A category's plots draw the systems given at "
      + "least 20 of its instances, coloured by family; a name in the legend greys its line or colours it again."),
    element("p", {}, "Every system ran single-threaded, and each comparison pairs a baseline with tddlog run on the same "
      + "machine in the same lanes; every answer of tddlog's is checked against a reference. The numbers come from the "
      + "same records as the README's and ",
      element("a", { href: "https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md" }, "docs/benchmarks.md"), "'s."));
}

// A category's systems by their answers, then their total time, against
// tddlog, with how each was given the instances; a system's name sets
// tddlog's times against its, `vs`'s row marked, and a count of own
// instances narrows the category to them.
function scoreboard(c, tal, n, vs, only) {
  const order = ["tddlog", ...ranked(c.systems, tal)];
  const body = element("tbody");
  for (const x of order) {
    const t = tal[x];
    const cls = (k) => (!t.classes[k] ? "" : k === "own" && only !== `own:${x}`
      ? link({ view: "category", category: c.key, only: `own:${x}` }, count(t.classes[k])) : count(t.classes[k]));
    body.append(element("tr", { class: x === "tddlog" ? "mine" : x === vs ? "vs" : null },
      element("th", { scope: "row" }, swatch(x), x === "tddlog" ? data.names[x] : link({ view: "category", category: c.key, vs: x, only }, data.names[x])),
      element("td", {}, element("span", { class: "bar" }, element("i", { style: `width:${(100 * t.answers / Math.max(1, n)).toFixed(1)}%` })),
        ` ${count(t.answers)}`),
      element("td", { class: "num" }, duration(t.total)),
      element("td", { class: "num" }, x === "tddlog" ? "" : times(t.total, tal.tddlog.total)),
      element("td", { class: "num" }, cls("own")), element("td", { class: "num" }, cls("native")),
      element("td", { class: "num" }, cls("translated"))));
  }
  return scroller(stacked(element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col" }, `answers, of ${count(n)}`),
      element("th", { scope: "col", class: "num" }, "total time"),
      element("th", { scope: "col", class: "num" }, "its time over tddlog's"),
      element("th", { scope: "col", class: "num", title: "instances from the system's own papers, benchmarks or distribution" }, "own"),
      element("th", { scope: "col", class: "num", title: "instances whose program is in the system's own language" }, "native"),
      element("th", { scope: "col", class: "num", title: "instances whose program a translator wrote from another language" }, "translated"))),
    body)), "whole");
}

// The systems that were not given some of the instances of a plot.
function cannot(suite) {
  const list = element("ul", { class: "cannot" });
  suite.systems.forEach((key, i) => {
    const n = suite.instances.filter((row) => row[i + 1] === null).length;
    if (n) list.append(element("li", {}, `${data.names[key]} was not given ${count(n)} of the ${count(suite.instances.length)} ${suite.noun ?? "instances"}.`));
  });
  return list;
}

function categoryView(route) {
  const c = data.categories.find((x) => x.key === route.category);
  lastCategory = c.key;
  const only = route.only;
  const g = groupOf(c, only);
  const rows = catRows(c, only);
  const tal = tally(c, rows, only);
  const drawn = drawnOf(c, rows, only);
  const whole = catSuite(c, rows, drawn, tal);
  const out = [crumbs(["Results", { view: "home" }], g ? [c.title, { view: "category", category: c.key }] : [c.title], ...(g ? [[g.title]] : [])),
    element("h2", {}, g ? g.title : c.title)];
  // a family: what it is, where it comes from and its benchmarks' pages
  if (g) {
    const pages = pagesOf(rows);
    out.push(prose("p", g.about, { class: "lede" }), note(`From ${g.source}. `, pages.length > 1 ? "Its benchmarks' pages: " : "Its benchmark's page: ",
      ...pages.flatMap((p, i) => [i ? ", " : "", link(p.route, p.label), pages.length > 1 ? ` (${count(p.n)})` : ""]), "."));
  }
  out.push(headline(c, rows, tal, only));
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
  const hidden = c.systems.filter((x) => !drawn.includes(x));
  out.push(element("h3", {}, "Each system's answers and total time"), scoreboard(c, tal, rows.length, route.vs, only),
    ...(hidden.length ? [note(`Not on the plots, given fewer than 20 of the ${c.noun}: `
      + `${and(hidden.map((x) => `${data.names[x]} (${count(tal[x].given)})`))}.`)] : []),
    element("h3", { id: "figures" }, "Instances answered within each time limit, and tddlog's times against another system's"),
    element("div", { class: "pair" }, plotted(whole, data.names, { also: route.vs, all: true }), scatter(rows, drawn, route.vs, true)),
    note(`The ${count(rows.length)} ${c.noun}${only ? ` of ${filters.find((f) => f.key === only).title}` : ""}. Every system given `
      + "some instance of the category is measured on all of them, an instance it does not answer or was not given counted "
      + `at its time limit and a time under ${FLOOR} s at ${FLOOR} s.`
      + (Object.keys(c.copies).length ? ` ${plural(Object.values(c.copies).flat().length, "instance")} of other suites `
        + "are the same problems as instances here (the same files): each counts once, its systems' fastest answers joined." : "")),
    cannot(whole));
  // Each family of the instances shown: tddlog's answers, and the most of
  // another system; a family narrows the category to it.
  if (!g) {
    const all = only === undefined ? rows : catRows(c);
    const keep = narrowing(c, only);
    const body = element("tbody");
    for (const fam of c.groups) {
      const mine = fam.rows.filter((i) => keep(c.rows[i], i)).map((i) => all[i]);
      if (!mine.length) continue;
      let best = null;
      for (const x of c.systems) {
        if (x === "tddlog") continue;
        const a = mine.filter((r) => r.t[x] !== null).length;
        if (best === null || a > best[1]) best = [x, a];
      }
      const name = link({ view: "category", category: c.key, only: fam.key }, fam.title);
      name.title = fam.about;
      body.append(element("tr", {}, element("th", { scope: "row" }, name),
        element("td", { class: "num" }, count(mine.length)), element("td", { class: "num" }, count(mine.filter((r) => r.t.tddlog !== null).length)),
        element("td", {}, best ? `${data.names[best[0]]}, ${count(best[1])}` : ""), prose("td", fam.source, { class: "about" })));
    }
    out.push(element("h3", {}, "Families"), scroller(stacked(element("table", { class: "board families" },
      element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "family"), element("th", { scope: "col", class: "num" }, "instances"),
        element("th", { scope: "col", class: "num" }, "tddlog answers"), element("th", { scope: "col" }, "the most answers of another system"),
        element("th", { scope: "col" }, "from"))), body))));
  }
  if (c.losses?.length) {
    out.push(element("h3", {}, "Where tddlog is slower"), element("ul", { class: "losses" }, ...c.losses.map((text) => prose("li", text))));
  }
  // The data's family of suites the category's first suite is in: the
  // findings that are no suite's are the category's where the category
  // starts with the family's first suite.
  const family = data.suites.get(c.rows[0][0]).home;
  const general = family.suites[0].key === c.rows[0][0] ? (family.findings ?? []).filter((card) => !card.suite) : [];
  if (general.length) out.push(element("h3", {}, "Findings"), findings(general));
  out.push(setup(family), raw("categories.json"));
  return out;
}

// A solver on the instances whose program is in its own language (its own
// benchmarks among them) and on those a translator wrote for it: for each,
// its answers and total time against tddlog's on the same instances (on
// tddlog's page, against the virtual best of all others), their plot and
// tddlog's times against its, and the families they are of.
function ways(s, parts) {
  const x = s.key, me = x === "tddlog", name = data.names[x];
  const out = [];
  for (const [letters, head, none] of [["on", "In its own language", "programs in its own language"],
    ["t", "Translated for it", "programs a translator wrote for it"]]) {
    const sets = parts.map(({ c, rows }) => {
      const j = c.systems.indexOf(x);
      return { c, rows: rows.filter((r) => letters.includes(r.cls[j])), own: rows.filter((r) => r.cls[j] === "o") };
    }).filter((p) => p.rows.length);
    const rows = sets.flatMap((p) => p.rows);
    out.push(element("h3", { id: letters === "t" ? "translated" : "native" }, head));
    if (!rows.length) {
      out.push(note(`${name} was given no ${none}.`));
      continue;
    }
    const t = virtual(rows, [x]), ts = virtual(rows, ["tddlog"]);
    // the virtual best of the other systems of each category, over its rows
    const vb = sets.map((p) => virtual(p.rows, p.c.systems.filter((y) => y !== "tddlog")))
      .reduce((a, v) => ({ answers: a.answers + v.answers, total: a.total + v.total }), { answers: 0, total: 0 });
    const own = sets.reduce((n, p) => n + p.own.length, 0);
    out.push(element("p", { class: "headline" }, element("b", {}, `${name} ${count(t.answers)} of ${count(rows.length)}`), ` in ${duration(t.total)}`,
      me ? `; the virtual best of all others ${count(vb.answers)} in ${duration(vb.total)}` : `; tddlog ${count(ts.answers)} in ${duration(ts.total)}`,
      own ? `; ${count(own)} of them its own benchmarks` : "", "."));
    if (me) {
      const best = { key: `${x}-${letters}`, title: head, noun: "instances", systems: ["tddlog"],
        instances: rows.map((r) => [r.name, r.v.tddlog]), limit: Math.max(...rows.map((r) => r.limit)), total: { tddlog: ts.total } };
      out.push(plotted(best, data.names, { all: true }));
    } else {
      const both = { key: `${x}-${letters}`, title: `${name} and tddlog`, noun: "instances", systems: ["tddlog", x],
        instances: rows.map((r) => [r.name, r.v.tddlog, r.v[x]]), limit: Math.max(...rows.map((r) => r.limit)),
        total: { tddlog: ts.total, [x]: t.total } };
      out.push(element("div", { class: "pair" }, plotted(both, data.names, { all: true }), scatter(rows, ["tddlog", x], x)));
    }
    // the families, by category: how many of their instances, its own among them
    const body = element("tbody");
    for (const { c, rows: mine } of sets) {
      const index = new Map(c.rows.map((row, i) => [`${row[0]}/${row[1]}`, i]));
      const at = new Set(mine.map((r) => index.get(`${r.suite}/${r.name}`)));
      const j = c.systems.indexOf(x);
      for (const g of c.groups) {
        const n = g.rows.filter((i) => at.has(i));
        if (!n.length) continue;
        const o = n.filter((i) => c.rows[i][3][j] === "o").length;
        const answers = (y) => n.filter((i) => answered(c.rows[i][4 + c.systems.indexOf(y)])).length;
        const fam = link({ view: "category", category: c.key, only: g.key }, g.title);
        fam.title = g.about;
        body.append(element("tr", {}, element("th", { scope: "row" }, fam), element("td", {}, c.title),
          element("td", { class: "num" }, count(n.length)), element("td", { class: "num" }, o ? count(o) : ""),
          element("td", { class: "num" }, count(answers(x))), ...(me ? [] : [element("td", { class: "num" }, count(answers("tddlog")))])));
      }
    }
    out.push(lazy({ class: "families" }, `The ${plural(body.rows.length, "family", "families")} of these instances`, () => [scroller(stacked(
      element("table", { class: "board" }, element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "family"),
        element("th", { scope: "col" }, "category"), element("th", { scope: "col", class: "num" }, "instances"),
        element("th", { scope: "col", class: "num", title: "from its own papers, benchmarks or distribution" }, "its own"),
        element("th", { scope: "col", class: "num" }, `${name} answers`),
        ...(me ? [] : [element("th", { scope: "col", class: "num" }, "tddlog answers")]))), body)), "whole")]));
  }
  return out;
}

// A solver's page: what it is, its plots (against tddlog on the instances in
// its own language and on those translated for it, then in each category it
// was given against the others), its answers and total time in each, its own
// benchmarks, how it was given the instances, its version and settings, and
// its paper. tddlog's page has its plot in each category beside the best
// other system's.
function systemView(route) {
  const s = data.systems.find((x) => x.key === route.system);
  const x = s.key, me = x === "tddlog", name = data.names[x];
  const parts = data.categories.filter((c) => s.categories.includes(c.key)).map((c) => {
    const rows = catRows(c);
    return { c, rows, tal: tally(c, rows) };
  });
  const all = parts.flatMap((p) => p.rows);
  const mine = virtual(all, ["tddlog"]);
  const site = s.link.replace(/^https?:\/\/(www\.)?/, "").replace(/\/$/, "");
  const out = [crumbs(["Results", { view: "home" }], [name]), element("h2", { class: "solver" }, swatch(x), name),
    element("p", { class: "lede" }, `${s.about} `, element("a", { href: s.link }, site)),
    element("p", { class: "headline" }, element("b", {}, `${name} ${count(s.answers)} of ${count(all.length)}`),
      ` in ${duration(s.total)}`, me ? "" : `; tddlog ${count(mine.answers)} in ${duration(mine.total)}`,
      `, over every instance of the ${plural(parts.length, "category", "categories")} ${me ? "" : "it was given"}`.trimEnd(), ".")];
  out.push(...ways(s, parts));
  // each category: its systems drawn (and this one), tddlog's and this one's
  // lines coloured (on tddlog's page, the best other system's)
  const grid = element("div", { class: "cat-figs" });
  for (const { c, rows, tal } of parts) {
    const other = me ? c.best ?? x : x;
    const drawn = [...new Set([...c.drawn, x, other])];
    const t = tal[other];
    grid.append(element("section", {}, element("h3", {}, link({ view: "category", category: c.key, vs: me ? undefined : x }, c.title)),
      element("p", { class: "headline" }, `${data.names[other]} ${count(t.answers)} of ${count(c.instances)} in ${duration(t.total)}; `
        + `tddlog ${count(tal.tddlog.answers)} in ${duration(tal.tddlog.total)}`),
      plotted(catSuite(c, rows, drawn, tal), data.names, { colour: ["tddlog", other] })));
  }
  out.push(element("h3", { id: "figures" }, me ? "In each category, against the best other system" : "In each category it was given, against the others"), grid);
  // each category's numbers
  const body = element("tbody");
  for (const { c, tal } of parts) {
    const t = tal[x], ts = tal.tddlog;
    const k = (key) => count(t.classes[key]);
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "category", category: c.key, vs: me ? undefined : x }, c.title)),
      element("td", { class: "num" }, count(c.instances)), element("td", { class: "num" }, count(t.given)),
      element("td", { class: "num" }, `${k("own")} / ${k("native")} / ${k("translated")}`),
      element("td", { class: "num" }, `${count(t.answers)} in ${duration(t.total)}`),
      ...(me ? [element("td", {}, c.best ? `${data.names[c.best]}, ${count(tal[c.best].answers)} in ${duration(tal[c.best].total)}` : "")]
        : [element("td", { class: "num" }, `${count(ts.answers)} in ${duration(ts.total)}`), element("td", { class: "num" }, times(t.total, ts.total))])));
  }
  out.push(element("h3", {}, "Answers and total time in each category"), scroller(stacked(element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "category"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num" }, "given"),
      element("th", { scope: "col", class: "num", title: "own / native / translated" }, "own / native / translated"),
      element("th", { scope: "col", class: "num" }, name),
      ...(me ? [element("th", { scope: "col" }, "the best other system")]
        : [element("th", { scope: "col", class: "num" }, "tddlog"), element("th", { scope: "col", class: "num", title: "its total time over tddlog's" }, "over tddlog's")]))),
    body)), "whole"));
  const own = data.own.filter((o) => o.system === x);
  if (own.length) out.push(...ownTable(own, "Its own benchmarks, against tddlog"));
  // how it was given the instances: the rows of the classes that apply
  const how = element("tbody");
  for (const h of s.how) {
    // the instances a row names: those whose name starts with its pattern (a
    // regular expression; one that ends in $, that name alone)
    const pattern = h.instances.endsWith("$") ? h.instances.slice(0, -1) : `${h.instances}…`;
    how.append(element("tr", {}, element("td", {}, h.cls), element("td", { class: "num" }, count(h.n)),
      element("td", { class: "about" }, and(h.suites.map((key) => data.suites.get(key)?.title ?? key)),
        h.instances ? [": ", element("code", {}, pattern)] : ""),
      prose("td", h.reason, { class: "about" })));
  }
  out.push(element("h3", {}, "How it was given the instances"), scroller(stacked(element("table", { class: "board how-given" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "as"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col" }, "suites"), element("th", { scope: "col" }, "the program it read"))), how)), "whole"));
  // version, settings, what it was not given, and how it is compared
  const short = parts.filter((p) => p.tal[x].given < p.c.instances)
    .map((p) => `${p.c.title}, ${count(p.c.instances - p.tal[x].given)} of ${count(p.c.instances)}`);
  const dl = element("dl", {}, element("dt", {}, "Version"), prose("dd", s.version), element("dt", {}, "Settings"), prose("dd", s.settings));
  if (short.length || s.notgiven) {
    dl.append(element("dt", {}, "Not given"), prose("dd", `${short.length ? `${and(short)}. ` : ""}${s.notgiven}`.trim()));
  }
  dl.append(element("dt", {}, "Compared"), prose("dd", (me ? `On every instance of the ${plural(parts.length, "category", "categories")}. `
    : "On every instance of the categories it was given, against tddlog run on the same machine in the same lanes. ")
    + "An instance it does not answer counts at its time limit, whatever the reason (out of time or memory, a wrong answer, "
    + `a program it cannot express, or not given), and a time under ${FLOOR} s as ${FLOOR} s. An instance is its own (from `
    + "its own papers, benchmarks or distribution), native (the program it read is in its own language) or translated (a "
    + "translator wrote the program it read from a program in another language, or a converter wrote it from the "
    + "network in another format)."));
  if (s.cite) {
    const c = s.cite;
    dl.append(element("dt", {}, "Paper"), element("dd", {}, `${c.authors}. `,
      c.doi ? element("a", { href: `https://doi.org/${c.doi}` }, c.title) : c.title, `. ${c.venue}, ${c.year}.`));
  }
  dl.append(element("dt", {}, me ? "Repository" : "Site"), element("dd", {}, element("a", { href: s.link }, s.link)));
  out.push(element("h3", {}, "Version, settings and paper"), element("div", { class: "facts" }, dl), raw("categories.json"));
  return out;
}

function findings(list) {
  const box = element("div", { class: "findings" });
  for (const card of list) {
    box.append(element("details", { class: "finding" }, element("summary", {}, card.title),
      ...card.paras.map((para) => prose("p", para))));
  }
  return box;
}

function setup(home) {
  const dl = element("dl");
  for (const [term, text] of home.setup) dl.append(element("dt", {}, term), prose("dd", text));
  return element("details", { class: "setup" }, element("summary", {}, "How the runs were made"), dl,
    note("Every table and its details: ", element("a", { href: home.docs }, "docs/benchmarks.md"), "."));
}

function suiteView(route) {
  const s = data.suites.get(route.suite);
  const c = categoryFor(s.key);
  const out = [crumbs(["Results", { view: "home" }], [c.title, { view: "category", category: c.key }], [s.title]),
    element("h2", {}, s.title), brief(s.about, "lede"),
    element("h3", {}, "Instances answered within each time limit"), plotted(s, data.names)];
  const fams = familiesOf(s.key);
  if (fams) {
    out.push(element("h3", {}, "Families"), note(
      "A family is one program at several sizes, or programs written the same way. Each cell counts the family's instances "
      + "a system answers; a family opens to its times, every run and the programs the systems were given."), familyTable(s, fams));
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
    out.push(element("h3", {}, "tddlog against each system"), headToHead(s, rows, paras));
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
    ...cols.map((x) => element("th", { scope: "col", class: `sys${x === "tddlog" ? " mine" : ""}` }, element("span", {}, data.names[x]))));
  const body = element("tbody");
  for (const f of fams) {
    const tr = element("tr", {}, element("th", { scope: "row" }, link({ view: "family", suite: s.key, family: f.key }, f.title),
      element("span", { class: "n" }, ` ${f.instances.length}`)));
    for (const x of cols) {
      const j = column(s, x);
      const given = f.instances.filter((name) => rows.get(name)?.[j] !== null).length;
      const a = f.instances.filter((name) => answered(rows.get(name)?.[j])).length;
      tr.append(given === 0 ? element("td", { class: "none", title: `${data.names[x]}: not given` })
        : element("td", { class: x === "tddlog" ? "mine" : null, style: `--p:${(a / f.instances.length).toFixed(3)}`,
          title: `${data.names[x]}: answers ${a} of the ${f.instances.length}` }, String(a)));
    }
    body.append(tr);
  }
  return scroller(element("table", { class: "grid counts" }, element("thead", {}, head), body));
}

function headToHead(s, rows, paras) {
  const body = element("tbody");
  for (const r of rows.sort((a, b) => b.theirs - a.theirs || a.theirTotal - b.theirTotal)) {
    const name = data.names[r.key];
    const para = paras.get(name.replace(/'s process$/, " (with Julia's start-up and compilation)"))
      ?? paras.get(name) ?? [...paras].find(([k]) => k.startsWith(`${data.names[base(r.key)]} (`) && dashed(r.key) === /start-up/.test(k) && !/without/.test(k))?.[1]
      ?? [...paras].find(([k]) => k.startsWith(`${name} (`))?.[1];
    const tr = element("tr", {},
      element("th", { scope: "row" }, swatch(r.key), name),
      element("td", { class: "num" }, count(r.given)),
      element("td", { class: "num" }, `${count(r.mine)} / ${count(r.theirs)}`),
      element("td", { class: "num" }, `${duration(r.myTotal)} / ${duration(r.theirTotal)}`),
      element("td", { class: "num ratio" }, r.myTotal <= r.theirTotal ? `${times(r.theirTotal, r.myTotal)} faster` : `${times(r.myTotal, r.theirTotal)} slower`));
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
      element("th", { scope: "col", class: "num" }, "answers, tddlog / it"),
      element("th", { scope: "col", class: "num" }, "total time, tddlog / it"),
      element("th", { scope: "col", class: "num" }, "tddlog"))), body)), "whole"),
    note(`Over the instances both are given: how many each answers, and the total times over those either `
      + `answers, a run that does not answer counted at the ${s.limit} s limit and a time under ${FLOOR} s at ${FLOOR} s.`));
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
    const family = familyOf(suite.key, row[0]);
    const tr = element("tr", {}, element("th", { scope: "row" },
      family ? link({ view: "family", suite: suite.key, family: family.key, instance: row[0] }, row[0]) : row[0]));
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
  parts.push(scroller(grid), element("p", { class: "key" }, ...shades(seen.has("unsupported"))));
  const notes = ["Seconds. A header sorts by its column, and again the other way."];
  if (familiesOf(suite.key)) notes.push("An instance's name opens its family: its times, every run and the programs the systems were given.");
  for (const status of Object.keys(FAILURES)) {
    if (!seen.has(status)) continue;
    notes.push(`${FAILURES[status][0](suite.limit)}: ${FAILURES[status][1]}.`);
  }
  if (suite.instances.some((row) => row.slice(1).includes(null))) notes.push("An empty cell: the system was not given the instance.");
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
  lastCategory = c.key;
  const f = familiesOf(s.key).find((x) => x.key === route.family);
  const short = shortNames(f.instances);
  const rows = new Map(s.instances.map((row) => [row[0], row]));
  const page = href(pageOf(route));
  const out = [crumbs(["Results", { view: "home" }], [c.title, { view: "category", category: c.key }],
    at ? [at.g.title, { view: "category", category: c.key, only: at.g.key }] : [s.title, { view: "suite", suite: s.key }], [f.title]),
    element("h2", {}, f.title),
    element("div", { class: "chips" }, element("span", { class: "chip" }, plural(f.instances.length, "instance")),
      element("span", { class: "chip" }, `${s.limit} s a run`), Object.assign(link({ view: "suite", suite: s.key }, s.title), { className: "chip" }))];
  const programs = element("section", { class: "programs", id: "programs" }, note("Loading the programs and the runs…"));
  out.push(element("h3", {}, "Times"), timesGrid(s, f, rows, short, route), programs, raw(`benchmarks/${s.key}/${f.key}.json`, s.file));
  familyData(s.key, f.key).then((fam) => {
    if (view.dataset.route !== page) return;   // another view was chosen meanwhile
    programs.replaceChildren(...instanceView(s, f, fam, now, short));
  }).catch((failure) => {
    programs.replaceChildren(element("p", { class: "failed" }, `The programs and runs did not load: ${failure.message}`));
  });
  return out;
}

// A family's instances by the systems given them: each time shaded by its
// place between 0.2 s and the limit on a log scale, the fastest from 0.2 s
// marked, a failure by its word; a time opens the system's runs of the
// instance, a name the instance's programs.
let showAll = false;
function timesGrid(s, f, rows, short, route) {
  const given = s.systems.filter((x) => f.instances.some((name) => rows.get(name)[column(s, x)] !== null));
  const cols = showAll ? s.systems : given;
  const head = element("tr", {}, element("th", { scope: "col" }, "instance"),
    ...cols.map((x) => element("th", { scope: "col", class: `sys${x === "tddlog" ? " mine" : ""}`, title: data.names[x] }, element("span", {}, data.names[x]))));
  const body = element("tbody");
  const seen = new Set();
  for (const name of f.instances) {
    const row = rows.get(name);
    const fastest = fastestOf(cols.filter((x) => !dashed(x)).map((x) => row[column(s, x)]));
    const tr = element("tr", { class: name === route.instance ? "on" : null },
      element("th", { scope: "row" }, link({ ...pageOf(route), instance: name }, short.get(name))));
    for (const x of cols) {
      const v = row[column(s, x)];
      const mine = x === "tddlog" ? "mine" : "";
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
  for (const v of Object.keys(FAILURES)) if (seen.has(v)) key.push(element("span", {}, `${FAILURES[v][0](s.limit)}: ${FAILURES[v][1].split(":")[0]}`));
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

// The systems that ran an instance but read none of its files on show, nor
// translated any: their files were not captured. A run whose grounding
// failed (`ungrounded`) was handed no file.
function uncaptured(inst) {
  const readers = (inst.tabs ?? []).flatMap((t) => [...t.readers, ...(t.translations ?? []).map(([who]) => who)]);
  return [...new Set((inst.runs ?? []).filter((r) => !r.ungrounded).map((r) => data.names[r.system] ?? r.system))]
    .filter((name) => !readers.some(readsAs(name)));
}

// The program tddlog read, for the playground: the one ProbLog file of the
// language tddlog reads, as a link, or why the playground cannot open it.
function playground(s, f, inst) {
  const tab = (inst.tabs ?? []).find((t) => t.readers.includes("tddlog"));
  const pl = tab?.files.filter((file) => /\.pl$/.test(file.name)) ?? [];
  if (pl.length !== 1) return null;
  const [file] = pl;
  if (file.held) return element("span", { class: "no-try" }, "Not in the playground: the site does not show this program; its file says why.");
  if (file.part) return element("span", { class: "no-try" }, `Not in the playground: the site keeps the first ${size(file.part)} of this ${size(file.bytes)} program.`);
  return element("span", { class: "try" }, element("a", { href: `../#bench=${[s.key, f.key, inst.name].map(encodeURIComponent).join("/")}` },
    "Open in the playground"), element("span", { class: "muted" }, ` the ${tab.lang} program tddlog read, ${size(file.bytes)}`));
}

// The systems that read a tab, each once: a form's label ("ProbLog -k sdd")
// under its system's name, an arm of tddlog's ("tddlog (auto+onehot)") as it
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

// The file an instance's view shows first: the program tddlog read (its
// ProbLog file, where its tab has others), else the original's first.
function firstFile(inst) {
  const ti = inst.tabs.findIndex((t) => t.readers.some(readsAs("tddlog")));
  if (ti < 0) return [0, shownFiles(inst.tabs[0])[0]];
  const pl = inst.tabs[ti].files.findIndex((file) => /\.pl$/.test(file.name));
  return [ti, pl < 0 ? shownFiles(inst.tabs[ti])[0] : pl];
}

// The address's part for a file of an instance: none for the one shown first.
function fileRoute(inst, ti, fi) {
  const [t, f] = firstFile(inst);
  return ti === t && fi === f ? undefined : [ti, fi];
}

// A tab in the menu: its language, and who reads it.
function versionLabel(tab) {
  const who = readersOf(tab);
  const what = who.length ? `read by ${and(who)}` : tab.translations?.length ? "an intermediate step, read by no system" : "read by no system";
  return `${tab.lang}${tab.original ? ", the original" : ""}: ${what}`;
}

const UNFAIR = "Note: this is an automatic translation from the original program, which might have performance "
  + "implications that make the comparison unfair.";
// The same for the original network converted to another network format.
const CONVERTED = "Note: this is the original network converted automatically to another format, which might have "
  + "performance implications that make the comparison unfair.";

// An instance of a family: its programs (the one tddlog read, and a menu of
// every other system's), then its answers. `route` names the file to show.
let compare = false;
function instanceView(s, f, fam, route, short) {
  const inst = fam.instances.find((i) => i.name === route.instance) ?? fam.instances[0];
  const at = { view: "family", suite: s.key, family: f.key };
  const out = [];
  const pick = f.instances.length > 12
    ? element("select", { "aria-label": "Instance", onchange: (e) => go({ ...at, instance: e.target.value }) },
      ...f.instances.map((n) => element("option", { value: n, selected: n === inst.name }, short.get(n))))
    : element("div", { class: "seg", role: "group", "aria-label": "Instance" },
      ...f.instances.map((n) => link({ ...at, instance: n }, short.get(n))).map((a, i) => {
        if (f.instances[i] === inst.name) a.setAttribute("aria-current", "true");
        return a;
      }));
  out.push(element("h3", {}, "Programs"), element("div", { class: "pick" }, element("span", { class: "muted" }, "instance"), pick,
    element("code", { class: "inst" }, inst.name), playground(s, f, inst)));
  if (!inst.tabs) {
    if (inst.gen) out.push(markup("p", inst.gen, { class: "note" }));
    out.push(note("The programs of this instance were not captured."));
  } else {
    const view = element("div", { class: "viewer" });
    out.push(view);
    // a file the address names later, without drawing the view again
    view.show = (file) => showFile(view, fam, inst, ...(file ?? firstFile(inst)));
    queueMicrotask(() => view.show(route.file));
  }
  if (inst.answers) out.push(answers(inst.answers));
  return out.filter(Boolean);
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
    remember({ file: fileRoute(inst, t, k) });
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
  const note = tab.conversion ? CONVERTED : UNFAIR;
  const cap = tab.translated && readersOf(tab).length ? element("p", { class: "unfair" }, note)
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
    } }, ...ks.map((k) => element("button", { type: "button", role: "tab", "aria-selected": String(k === fi), tabindex: k === fi ? "0" : "-1",
      "data-file": k, onclick: () => k !== fi && choose(k) }, tab.files[k].name.split("/").pop(), element("small", {}, size(tab.files[k].bytes)))));
    box.append(strip);
  }
  return fileIn(box, fam, tab.files[fi]);
}

// A file's text: in its family's data, or a file of its own, gzipped where
// its name ends in .gz.
async function fileText(fam, file) {
  if (fam.texts[file.sha] !== undefined) return fam.texts[file.sha];
  const response = await fetch(`benchmarks/${file.src}`);
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
  for (const field of above) box.append(markup("p", file[field], { class: "made" }));
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
    if (a.ref) notes.push(`The reference is ${a.ref}. Each system's answers of its first repetition that answered.`);
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
  const rec = recorded(key);
  const mine = (inst?.runs ?? []).filter((r) => r.system === rec);
  const parts = [];
  const about = [];
  if (dashed(key)) about.push("The whole process's time, with the language's start and compilation; the other time of the same system leaves them out, as its paper times it.");
  if (inst?.reps > 1) about.push("A time is the median of the repetitions, a failed one counted as infinite.");
  if (new Set(mine.map((r) => r.label)).size > 1) about.push("A system run in several forms has the time of its fastest form.");
  if (inst?.notes?.[rec]) about.push(inst.notes[rec]);
  if (about.length) parts.push(note(about.join(" ")));
  const name0 = data.names[rec] ?? data.names[key];
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
  } else if (mine.some((r) => !r.ungrounded)) {
    parts.push(note(`The command and the files ${name0} was given were not captured.`));
  }
  const tryIt = rec === "tddlog" && inst ? playground(s, f, inst) : null;
  if (tryIt) parts.push(element("p", {}, tryIt));
  // the runs, the latest first; the same outcome of the same form folded
  const shown = [];
  const folded = new Map();
  for (const r of [...mine].reverse()) {
    const sig = JSON.stringify([r.label, r.skipped ?? false, (r.rows ?? []).map((row) => row.cells.slice(1, 3))]);
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
    const summary = r.skipped ? "not run" : r.rows.map((row) => (row.cells[1] === "answered" ? `${row.cells[at]} s` : row.cells[1])).join(", ");
    const block = element("details", { class: "run", open: i === 0 || hit },
      element("summary", {}, element("b", {}, r.label), hit ? element("span", { class: "badge orig" }, "time in the table") : null,
        element("span", { class: "muted" }, ` ${summary}${more ? `; and ${plural(more, "run")} with the same outcome` : ""}`)),
      note(r.later ? "A later run, in lanes of its own." : "The release run."));
    if (r.later && rec !== "tddlog") {
      const t = tddlogIn(r.later).map(med).filter((x) => x !== null);
      if (t.length) block.append(note(`tddlog in the same lanes: ${seconds(t[0])} s.`));
    }
    if (r.skipped) block.append(note(`Not run in any repetition: ${FAILURES.skipped[1]}.`));
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

// The categories, the families of the category on show and the benchmarks'
// pages of the family on show; and a search.
function rail(route) {
  const tree = element("ul", { class: "tree" });
  // a benchmark's page is in the category and the family of its instance
  const at = route.view === "family" ? data.place.get(`${route.suite}/${route.instance}`) : undefined;
  const current = route.view === "category" ? route.category : at ? at.c.key : route.suite ? categoryFor(route.suite).key : null;
  const family = route.view === "category" ? route.only : at?.g.key;
  tree.append(element("li", {}, link({ view: "home" }, "Overview")), element("li", { class: "group" }, "Types of programs"));
  for (const c of data.categories) {
    const li = element("li", {}, link({ view: "category", category: c.key }, c.title, element("span", { class: "n" }, count(c.instances))));
    if (route.view === "category" && route.category === c.key && route.only === undefined) li.firstChild.setAttribute("aria-current", "page");
    if (c.key === current) {
      const ul = element("ul");
      for (const g of c.groups) {
        const gl = element("li", {}, link({ view: "category", category: c.key, only: g.key }, g.title,
          element("span", { class: "n" }, count(g.rows.length))));
        if (route.view === "category" && route.only === g.key) gl.firstChild.setAttribute("aria-current", "page");
        if (g.key === family) {
          const fl = element("ul");
          const me = 4 + c.systems.indexOf("tddlog");
          const rows = g.rows.map((i) => ({ suite: c.rows[i][0], name: c.rows[i][1], t: { tddlog: answered(c.rows[i][me]) ? c.rows[i][me] : null } }));
          for (const p of pagesOf(rows)) {
            const a = p.a ?? 0;
            const fi = element("li", {}, link(p.route,
              element("i", { class: `dot ${a === p.n ? "all" : a ? "some" : ""}`, title: `tddlog answers ${a} of ${p.n}` }),
              p.label, element("span", { class: "n" }, String(p.n))));
            if (route.view === p.route.view && route.suite === p.route.suite && route.family === p.route.family) fi.firstChild.setAttribute("aria-current", "page");
            fl.append(fi);
          }
          gl.append(fl);
        }
        ul.append(gl);
      }
      li.append(ul);
    }
    tree.append(li);
  }
  // the solvers, by the instances they answer
  tree.append(element("li", { class: "group" }, "Solvers"));
  for (const s of data.systems) {
    const li = element("li", { class: "solver" }, link({ view: "system", system: s.key }, swatch(s.key), data.names[s.key],
      element("span", { class: "n", title: `answers ${count(s.answers)} instances` }, count(s.answers))));
    if (route.view === "system" && route.system === s.key) li.firstChild.setAttribute("aria-current", "page");
    tree.append(li);
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
        const family = familyOf(s.key, name);
        add(name, "", s.title, family ? { view: "family", suite: s.key, family: family.key, instance: name } : { view: "suite", suite: s.key });
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
  return [crumbs(["Results", { view: "home" }], ["Not found"]), element("h2", {}, "Not found"),
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
  location.hash = href(route);
}

// What changes in a view without drawing it again (the file on show) goes in
// the address in place, so that a link or a reload shows it again.
function remember(change) {
  now = { ...now, ...change };
  history.replaceState(null, "", href(now));
}

function title(route) {
  switch (route.view) {
    case "category": return data.categories.find((c) => c.key === route.category).title;
    case "system": return data.names[route.system];
    case "suite": return data.suites.get(route.suite).title;
    case "family": return `${route.instance} · ${route.suite}/${route.family}`;
    case "missing": return "Not found";
    default: return null;
  }
}

// The view the address names: drawn again where it is another view (or
// `again`), else only what is open in it changes: the file on show, the
// drawer, and a category's filter and the system its times are set against.
function show(again = false) {
  const route = parse(location.hash);
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
    const parts = route.view === "category" ? categoryView(route) : route.view === "system" ? systemView(route)
      : route.view === "suite" ? suiteView(route)
      : route.view === "family" ? familyView(route, view) : route.view === "missing" ? missing(route) : home();
    view.replaceChildren(...parts);
    // A family's view starts at its top, and another instance of it keeps the
    // place; a category's times set against a system start at its plots.
    const family = route.view === "family" ? `${route.suite}/${route.family}` : null;
    if (route.view === "category" && route.vs) document.getElementById("figures").scrollIntoView();
    else if (!again && (family === null || family !== show.family)) window.scrollTo({ top: 0 });
    show.family = family;
    const t = title(route);
    document.title = t === null ? "tddlog results" : `${t} · tddlog results`;
  } else if (route.view === "category" && (route.vs !== before.vs || route.only !== before.only)) {
    // the same category narrowed otherwise, or against another system: drawn
    // again in place, the menu marking the family it is narrowed to
    const menu = document.activeElement?.matches("select.vs");
    if (route.only !== before.only) rail(route);
    view.replaceChildren(...categoryView(route));
    if (menu) view.querySelector("select.vs").focus();
  } else if (route.view === "family" && String(route.file ?? "") !== String(before.file ?? "")) {
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
      load("categories.json"), load("benchmarks/index.json").catch(() => null)]);
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
    data.groups = cats.families;
    data.own = cats.own;
    data.systems = cats.systems;
    for (const s of data.systems) if (!data.names[s.key]) throw new Error(`the solver ${s.key} has no name`);
    data.categories = [...cats.categories].sort((a, b) => a.order - b.order);
    // Each category's suites, its rows' and then its copies', with how many
    // of its instances each holds.
    for (const c of data.categories) {
      c.perSuite = new Map();
      const add = (key) => c.perSuite.set(key, (c.perSuite.get(key) ?? 0) + 1);
      for (const row of c.rows) add(row[0]);
      for (const copies of Object.values(c.copies)) for (const copy of copies) add(copy.split("/")[0]);
      c.suites = [...c.perSuite.keys()];
      for (const key of c.suites) if (!data.suites.has(key)) throw new Error(`the category ${c.key} has no suite ${key}`);
      for (const g of c.groups) {
        for (const i of g.rows) {
          const key = `${c.rows[i][0]}/${c.rows[i][1]}`;
          for (const k of [key, ...(c.copies[key] ?? [])]) data.place.set(k, { c, g });
        }
      }
    }
    data.version = suites.version;
    window.addEventListener("hashchange", () => show());
    const input = document.getElementById("search");
    input.addEventListener("input", () => search(input.value));
    input.addEventListener("keydown", searchKeys);
    const toggle = document.getElementById("browse");
    toggle.addEventListener("click", () => {
      const rail = document.getElementById("rail");
      rail.classList.toggle("open");
      toggle.setAttribute("aria-expanded", String(rail.classList.contains("open")));
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !drawer().hidden) go({ ...now, runs: undefined });
    });
    show();
  } catch (failure) {
    view.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
