"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl,
// Roulette and Geni), both written by the generator of README.md's results
// and the baselines branch's docs/benchmarks.md, and, where the site has
// them, the benchmarks' data from the baselines branch (benchmarks/, which
// release/pages/bench_pages.py writes): each suite's families of instances
// and, for each family, its programs in every language, its runs and its
// answers.
//
// The page is one view at a time, named by the address's hash:
//   #/                       the kinds of problems, each system's answers and total time
//   #/kind/KIND              a kind: its plot, its systems' totals, its suites
//   #/SUITE                  a suite: its plot, tddlog against each system, its families
//   #/SUITE/FAMILY[/NAME]    a family: its instances' times, and an instance's programs and answers
//   #/SUITE/@NAME            the family of the instance NAME
// and after an instance's name, what is open in its view:
//   …/tab/T[/F]              the file F (0 where left out) of the instance's language T
//   …/runs/SYSTEM            SYSTEM's runs of the instance, in a drawer beside the view
// A time in a family's table opens that system's runs of the instance
// beside the view. A kind's plot is the data's `kinds` (the README's
// figures): each kind's systems over all of its instances, with each
// system's answers and total time, which the results pipeline writes; the
// page does not choose a kind's instances or systems, nor compute its
// totals.

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
  skipped: [() => "skipped", "not run: the system had failed on a smaller size of the same program"],
  wrong: [() => "wrong", "an answer off the reference by more than 1e-6"],
  zero: [() => "zero", "every answer printed as 0 where the reference's are positive"],
  underflow: [() => "underflow", "every answer printed as 0 where the probability of the evidence is below the smallest double"],
};
// A failure that is a verdict on the answers, not a run that stopped.
const VERDICTS = new Set(["wrong", "zero", "underflow"]);

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

// The instances each system answers within a time limit, the limit on a log
// scale from 0.2 s to the suite's; a line further up and to the left is
// better. Drawn at the width of its box; the lines not `coloured` grey and
// under the others. Each line is named at its end, by its system where there
// is room, else by its number in the legend (the coloured lines only).
// Returns whether the plot is narrow.
function cactus(box, suite, names, series, coloured) {
  const width = Math.max(280, Math.floor(box.clientWidth));
  const narrow = width < 560;
  const height = narrow ? 300 : Math.max(380, series.length * 14 + 22 + 46);
  const n = suite.instances.length;
  const hi = suite.limit;
  const ends = series.map(({ key, times }, i) => ({ key, y: times.length, text: narrow ? String(i + 1) : `${names[key]} ${times.length}` }))
    .filter((end) => !narrow || coloured.has(end.key));
  // The end labels' room: the longest at 7 px a character.
  const left = 46, right = narrow ? 30 : Math.max(128, 16 + 7 * Math.max(0, ...ends.map((end) => end.text.length))), top = 22, bottom = 46;
  const x0 = left, x1 = width - right, y0 = top, y1 = height - bottom;
  const lx = (t) => x0 + (Math.log10(Math.min(Math.max(t, FLOOR), hi)) - Math.log10(FLOOR)) / (Math.log10(hi) - Math.log10(FLOOR)) * (x1 - x0);
  const ly = (k) => y1 - (k / n) * (y1 - y0);

  const plot = svg("svg", { class: "cactus", width, height, viewBox: `0 0 ${width} ${height}`, role: "img", tabindex: "0",
    "aria-label": `${suite.title}: instances each system answers within a time limit; the arrow keys move the limit` });
  const ticks = [FLOOR, 1, 10, 100, 1000].filter((t) => t < hi * 0.75).concat([hi]);
  for (const t of ticks) {
    const x = lx(t);
    plot.append(svg("line", { class: "grid", x1: x, y1: y0, x2: x, y2: y1 }));
    plot.append(svg("text", { class: "tick", x, y: y1 + 16, "text-anchor": "middle" }, t === FLOOR ? `≤${FLOOR}` : String(t)));
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
    "time limit per instance, seconds (log scale)"));
  plot.append(svg("text", { class: "label", transform: `translate(13 ${(y0 + y1) / 2}) rotate(-90)`, "text-anchor": "middle" },
    "instances answered"));

  // The grey lines first, then the coloured, tddlog's last, so that it is
  // drawn on top.
  const order = [...series].sort((a, b) => coloured.has(a.key) - coloured.has(b.key) || (a.key === "tddlog") - (b.key === "tddlog"));
  for (const { key, times } of order) {
    let k = times.filter((t) => t <= FLOOR).length;
    let d = `M${x0},${ly(k)}`;
    for (const t of times.slice(k)) {
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
    const t = Math.pow(10, Math.log10(FLOOR) + (cx - x0) / (x1 - x0) * (Math.log10(hi) - Math.log10(FLOOR)));
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

// A plot with its legend, drawn again when its box changes width or a line
// is coloured or greyed. Where there are more than five lines, tddlog's and
// the three that answer the most are in colour and the others grey until
// asked for. The figure names the systems it draws and the number of its
// instances; a kind's legend gives each system's total time too
// (suite.total).
function plotted(suite, names) {
  const series = linesOf(suite);
  const top = series.slice(0, series.length > 5 ? 4 : series.length).map((s) => s.key);
  let coloured = new Set(top);
  const figure = element("figure", { class: "plot", "data-systems": suite.systems.join(" "),
    "data-instances": String(suite.instances.length) });
  const box = element("div", { class: "box" });
  const every = series.length > top.length ? element("button", { type: "button", class: "plain every", onclick: () => {
    coloured = new Set(coloured.size === series.length ? top : series.map((s) => s.key));
    draw();
  } }) : null;
  figure.append(box);
  const draw = () => {
    const narrow = cactus(box, suite, names, series, coloured);
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
    if (every) every.textContent = coloured.size === series.length ? "Colour tddlog's and the top three lines only" : `Colour all ${series.length} lines`;
    if (focused) fresh.querySelector(`button[data-system="${CSS.escape(focused)}"]`)?.focus();
  };
  let last = 0;
  new ResizeObserver(() => {
    const w = box.clientWidth;
    if (w === 0 || w === last) return;
    last = w;
    draw();
  }).observe(box);
  return figure;
}

// ------------------------------------------------------------- the data

const data = {
  names: {},        // a system's key: its name
  kinds: [],        // the data's kinds, in the README's order, each with `file`, the data's file that holds it
  suites: new Map(),   // a suite's key: the suite, with `home`, the data's family that holds it, and `file`
  bench: null,      // benchmarks/index.json: every suite's families, where the site has them
  families: new Map(), // `${suite}/${family}`: the promise of that family's data
  family: new Map(),   // `${suite}/${instance}`: the instance's family
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

const kindsOf = (suite) => data.kinds.filter((k) => k.suites.includes(suite));
const familiesOf = (suite) => data.bench?.suites?.[suite] ?? null;
const familyOf = (suite, name) => data.family.get(`${suite}/${name}`);
// The column of a system in a suite's rows (its first is the instance).
const column = (suite, key) => suite.systems.indexOf(key) + 1;
// The record's system of a column: alea-process's runs are Alea.jl's.
const recorded = (key) => data.bench?.records?.[key] ?? base(key);
// Whether a tab's reader is (a form of) the system of this name: "Ace auto"
// and "Ace forceC2d" are Ace's.
const readsAs = (name) => (reader) => reader === name || reader.startsWith(`${name} `);

// A kind of the data (its suites) as one suite: the kind's systems, tddlog
// first and then by the instances they answer, as the README's legends list
// them; a system without a record of an instance null; the limit the
// longest of its suites'; each system's total time, the data's.
function kinded(kind) {
  const suites = kind.suites.map((key) => {
    const suite = data.suites.get(key);
    if (suite === undefined) throw new Error(`the kind ${kind.key} has no suite ${key}`);
    return suite;
  });
  const systems = ["tddlog", ...kind.systems.filter((key) => key !== "tddlog").sort((a, b) => kind.answers[b] - kind.answers[a])];
  const instances = suites.flatMap((suite) => suite.instances.map((row) => {
    const out = [row[0], ...systems.map(() => null)];
    suite.systems.forEach((key, i) => {
      if (systems.includes(key)) out[systems.indexOf(key) + 1] = row[i + 1];
    });
    return out;
  }));
  const n = count(instances.length);
  const of = suites.length === 1 ? `the suite ${suites[0].title}` : `${suites.length} suites (${suites.map((s) => s.title).join("; ")})`;
  const about = `The ${n} ${kind.noun} of ${of}. Every system of the kind is tried on all of them, `
    + "and an instance a system does not answer counts as unanswered, whatever the reason: out of time or memory, a wrong answer, "
    + "a program the system cannot express, or one it was not given. The legend gives each system's answers and its total time "
    + `over the ${count(kind.scored)} ${kind.noun} that some system answers, a run that does not answer counted at its suite's `
    + `time limit and a time under ${FLOOR} s at ${FLOOR} s.`;
  return { key: kind.key, title: kind.title, noun: kind.noun, about, systems, instances,
    limit: Math.max(...suites.map((suite) => suite.limit)), total: kind.total };
}

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

// The view an address names, or what it names that the data does not have.
function parse(hash) {
  const old = hash.replace(/^#/, "");
  if (old === "" || old === "/") return { view: "home" };
  // the page before this one named a family of suites by its key
  if (!old.startsWith("/")) {
    if (data.kinds.some((k) => k.key === old)) return { view: "kind", kind: old };
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
  if (parts[0] === "kind") return parts.length === 2 && data.kinds.some((k) => k.key === parts[1]) ? { view: "kind", kind: parts[1] } : missing;
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
  if (Number(m[1]) || Number(m[2])) route.file = [Number(m[1]), Number(m[2] ?? 0)];
  if (m[3] !== undefined) route.runs = m[3];
  return route;
}

function href(route) {
  switch (route.view) {
    case "kind": return `#/kind/${route.kind}`;
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

// A view without what is open in it: the file on show and the drawer.
const pageOf = (route) => ({ ...route, file: undefined, runs: undefined });

function link(route, ...children) {
  return element("a", { href: href(route) }, ...children);
}

// The kind a view of a suite belongs to: the one the reader came from, else
// the first that holds the suite (TPC-H is a logic program's and a
// database's).
let lastKind = null;
function kindFor(suite) {
  const ks = kindsOf(suite);
  return ks.find((k) => k.key === lastKind) ?? ks[0];
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

function home() {
  const instances = [...data.suites.values()].reduce((n, s) => n + s.instances.length, 0);
  const others = new Set([...data.suites.values()].flatMap((s) => s.systems).map(base));
  others.delete("tddlog");
  const out = [
    element("div", { class: "stats" },
      ...[[count(instances), "instances"], [count(data.suites.size), "suites"], [count(others.size), "other systems"]]
        .map(([b, what]) => element("span", { class: "stat" }, element("b", {}, b), what))),
    element("h2", {}, "Kinds of problems"),
    element("p", { class: "lede" }, "Each kind is one set of instances, with every system of the kind tried on all of them. ",
      "Total time counts a run that does not answer at its time limit and a time under 0.2 s as 0.2 s."),
  ];
  const grid = element("div", { class: "kinds" });
  for (const k of data.kinds) {
    const fastest = Object.keys(k.total).filter((x) => x !== "tddlog").sort((a, b) => k.total[a] - k.total[b])[0];
    const most = Math.max(...Object.keys(k.answers).filter((x) => x !== "tddlog").map((x) => k.answers[x]));
    const best = Object.keys(k.answers).find((x) => x !== "tddlog" && k.answers[x] === most);
    const less = k.total.tddlog <= k.total[fastest];
    grid.append(link({ view: "kind", kind: k.key },
      element("article", { class: "kind-card" },
        element("h3", {}, k.title),
        element("p", { class: "n" }, `${plural(k.instances, k.noun.replace(/s$/, ""), k.noun)}, ${plural(k.systems.length - 1, "other system")}; `
          + "the answers of tddlog and of the other system that answers the most:"),
        meter("tddlog", k.answers.tddlog, k.instances, true),
        meter(data.names[best], most, k.instances, false),
        element("p", { class: "total" },
          element("b", {}, less ? `${times(k.total[fastest], k.total.tddlog)} less time` : `${times(k.total.tddlog, k.total[fastest])} more time`),
          ` than the fastest other system, ${data.names[fastest]}: ${duration(k.total.tddlog)} against ${duration(k.total[fastest])}`))));
  }
  out.push(grid, how(), raw(...new Set(data.kinds.map((k) => k.file))));
  return out;
}

// A system's answers out of n, as a bar.
function meter(name, k, n, mine) {
  return element("div", { class: `meter${mine ? " mine" : ""}` },
    element("span", { class: "who" }, name),
    element("span", { class: "bar", role: "img", "aria-label": `${name}: ${count(k)} of ${count(n)}` }, element("i", { style: `width:${(100 * k / n).toFixed(1)}%` })),
    element("span", { class: "k" }, `${count(k)}/${count(n)}`));
}

// How the plots and totals are read, under every overview.
function how() {
  return element("details", { class: "how" }, element("summary", {}, "How to read these results"),
    element("p", {}, "A plot counts the instances each system answers within a time limit, on a log scale from 0.2 s to "
      + "the suite's limit: a line further up and to the left is better, and a run that takes less than 0.2 s counts at "
      + "0.2 s, since differences of a few milliseconds of start-up are not compared. Where a plot has more than five "
      + "lines, tddlog's and the three that answer the most are in colour; a name in the legend colours its line."),
    element("p", {}, "A kind is one set of instances, the programs naturally written in the languages of that kind, and "
      + "its systems are those whose language is of that kind, with ProbLog, PITA and aspmc on every kind: tddlog reads "
      + "ProbLog, and they are given the program tddlog reads. The bit-blasted circuits are a kind of their own: Alea.jl "
      + "compiles each of these programs, written in Alea.jl, to a circuit, and every other system is given that circuit, "
      + "an intermediate representation rather than the program as written."),
    element("p", {}, "Every system of a kind is tried on all of its instances, and an instance a system does not answer "
      + "counts as unanswered, whatever the reason: out of time or memory, a wrong answer, a program the system cannot "
      + "express, or one it was not given. A total time is over the instances some system of the kind answers, a run "
      + "that does not answer counted at its suite's time limit."),
    element("p", {}, "Every system ran single-threaded, in the same job and the same lanes of one machine as the systems "
      + "it is compared with, and every answer of tddlog's is checked against a reference. The numbers come from the "
      + "same records as the README's and ",
      element("a", { href: "https://github.com/Tractables/tddlog/blob/baselines/docs/benchmarks.md" }, "docs/benchmarks.md"), "'s."));
}

// A kind's systems by their answers, then their total time, against tddlog.
function scoreboard(k) {
  const order = Object.keys(k.answers).sort((a, b) => k.answers[b] - k.answers[a] || k.total[a] - k.total[b]);
  const body = element("tbody");
  for (const x of order) {
    body.append(element("tr", { class: x === "tddlog" ? "mine" : null },
      element("th", { scope: "row" }, swatch(x), data.names[x]),
      element("td", {}, element("span", { class: "bar" }, element("i", { style: `width:${(100 * k.answers[x] / k.instances).toFixed(1)}%` })),
        ` ${count(k.answers[x])}`),
      element("td", { class: "num" }, duration(k.total[x])),
      element("td", { class: "num" }, x === "tddlog" ? "" : times(k.total[x], k.total.tddlog))));
  }
  return scroller(stacked(element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col" }, `answers, of ${count(k.instances)}`),
      element("th", { scope: "col", class: "num" }, "total time"),
      element("th", { scope: "col", class: "num" }, "its time over tddlog's"))), body)), "whole");
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

function kindView(route) {
  const k = data.kinds.find((x) => x.key === route.kind);
  lastKind = k.key;
  const whole = kinded(k);
  // The data's family of suites the kind's first suite is in: its intro and
  // the findings that are no suite's are the kind's where the kind starts
  // with the family's first suite (the circuits and the databases do not).
  const family = data.suites.get(k.suites[0]).home;
  const own = family.suites[0].key === k.suites[0];
  const out = [crumbs(["Results", { view: "home" }], [k.title]), element("h2", {}, k.title)];
  if (own && family.intro) out.push(brief(family.intro, "lede"));
  out.push(element("h3", {}, "Each system's answers and total time"), scoreboard(k),
    element("h3", {}, "Instances answered within each time limit"), plotted(whole, data.names),
    note(whole.about), cannot(whole));
  // Each suite: tddlog's answers, and the most of another system of the kind.
  const body = element("tbody");
  for (const key of k.suites) {
    const s = data.suites.get(key);
    const mine = s.instances.filter((row) => answered(row[column(s, "tddlog")])).length;
    let best = null;
    for (const x of s.systems) {
      if (x === "tddlog" || !k.systems.includes(x)) continue;
      const a = s.instances.filter((row) => answered(row[column(s, x)])).length;
      if (best === null || a > best[1]) best = [x, a];
    }
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "suite", suite: key }, s.title)),
      element("td", { class: "num" }, count(s.instances.length)), element("td", { class: "num" }, count(mine)),
      element("td", {}, best ? `${data.names[best[0]]}, ${count(best[1])}` : "")));
  }
  out.push(element("h3", {}, "Suites"), scroller(stacked(element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "suite"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num" }, "tddlog answers"), element("th", { scope: "col" }, "the most answers of another system"))), body))));
  if (k.losses?.length) {
    out.push(element("h3", {}, "Where tddlog is slower"), element("ul", { class: "losses" }, ...k.losses.map((text) => prose("li", text))));
  }
  const general = (family.findings ?? []).filter((c) => !c.suite);
  if (own && general.length) out.push(element("h3", {}, "Findings"), findings(general));
  out.push(setup(family), raw(k.file));
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
  const k = kindFor(s.key);
  const out = [crumbs(["Results", { view: "home" }], [k.title, { view: "kind", kind: k.key }], [s.title]),
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
  // An instance's fastest answer, where it takes 0.2 s or more.
  const fastest = (row) => {
    const ts = cols.map((key, i) => (dashed(key) ? null : row[i + 1])).filter(answered);
    const t = ts.length ? Math.min(...ts) : null;
    return t !== null && t >= FLOOR ? t : null;
  };
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
    const best = fastest(row);
    const family = familyOf(suite.key, row[0]);
    const tr = element("tr", {}, element("th", { scope: "row" },
      family ? link({ view: "family", suite: suite.key, family: family.key, instance: row[0] }, row[0]) : row[0]));
    if (quick(row)) tr.className = "quick";
    cols.forEach((key, i) => {
      const v = row[i + 1];
      let td;
      if (answered(v)) td = element("td", { class: `num${v === best && !dashed(key) ? " best" : ""}` }, seconds(v));
      else if (v === null) td = element("td", { class: "num none", title: "not given" }, "");
      else {
        seen.add(v);
        const [text, meaning] = FAILURES[v] ?? [() => v, v];
        td = element("td", { class: "num fail", title: meaning }, text(suite.limit));
      }
      tr.append(td);
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
  parts.push(scroller(grid));
  const notes = [`Seconds; an instance's fastest answer in bold, where it takes ${FLOOR} s or more. A header sorts by its column, and again the other way.`];
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
  const k = kindFor(s.key);
  const f = familiesOf(s.key).find((x) => x.key === route.family);
  const short = shortNames(f.instances);
  const rows = new Map(s.instances.map((row) => [row[0], row]));
  const page = href(pageOf(route));
  const out = [crumbs(["Results", { view: "home" }], [k.title, { view: "kind", kind: k.key }], [s.title, { view: "suite", suite: s.key }], [f.title]),
    element("h2", {}, f.title),
    element("div", { class: "chips" }, element("span", { class: "chip" }, plural(f.instances.length, "instance")),
      element("span", { class: "chip" }, `${s.limit} s a run`))];
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
    const ts = cols.filter((x) => !dashed(x)).map((x) => row[column(s, x)]).filter(answered);
    const best = ts.length && Math.min(...ts) >= FLOOR ? Math.min(...ts) : null;
    const tr = element("tr", { class: name === route.instance ? "on" : null },
      element("th", { scope: "row" }, link({ ...pageOf(route), instance: name }, short.get(name))));
    for (const x of cols) {
      const v = row[column(s, x)];
      const mine = x === "tddlog" ? "mine" : "";
      if (v === null) {
        tr.append(element("td", { class: `none ${mine}`, title: `${data.names[x]}: not given` }));
        continue;
      }
      let text, title, cls = mine, p = 0;
      if (answered(v)) {
        text = seconds(v);
        title = `${data.names[x]}: ${v} s`;
        p = Math.max(0, Math.log(Math.max(v, FLOOR) / FLOOR) / Math.log(s.limit / FLOOR));
        if (v === best && !dashed(x)) cls += " best";
      } else {
        seen.add(v);
        const [t, meaning] = FAILURES[v] ?? [() => v, v];
        text = t(s.limit);
        title = `${data.names[x]}: ${meaning}`;
        cls += VERDICTS.has(v) || v === "error" ? " bad" : " fail";
      }
      // The same instance's runs keep its file on show; another's show its first.
      tr.append(element("td", { class: cls.trim() }, element("button", { type: "button", style: `--p:${p.toFixed(3)}`, title,
        "data-runs": `${s.key}/${name}/${x}`,
        onclick: () => go({ ...now, instance: name, file: name === now.instance ? now.file : undefined, runs: x }) }, text)));
    }
    body.append(tr);
  }
  const key = [element("span", {}, element("i", { class: "heat lo" }), `${FLOOR} s`), element("span", {}, element("i", { class: "heat hi" }), `${s.limit} s`),
    element("span", {}, `underlined: the fastest, from ${FLOOR} s`)];
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

// The systems that ran an instance but read none of its files on show:
// their files were not captured.
function uncaptured(inst) {
  const readers = (inst.tabs ?? []).flatMap((t) => t.readers);
  return [...new Set((inst.runs ?? []).map((r) => data.names[r.system] ?? r.system))]
    .filter((name) => !readers.some(readsAs(name)));
}

// The program tddlog read, for the playground: the one ProbLog file of the
// language tddlog reads, as a link, or why the playground cannot open it.
function playground(s, f, inst) {
  const tab = (inst.tabs ?? []).find((t) => t.readers.includes("tddlog"));
  const pl = tab?.files.filter((file) => /\.pl$/.test(file.name)) ?? [];
  if (pl.length !== 1) return null;
  const [file] = pl;
  if (file.held) return note("Not in the playground: the site does not show this program (below, why).");
  if (file.part) return note(`Not in the playground: the site keeps the first ${size(file.part)} of this ${size(file.bytes)} program.`);
  return element("p", { class: "try" }, element("a", { href: `../#bench=${[s.key, f.key, inst.name].map(encodeURIComponent).join("/")}` },
    "Open in the playground"), element("span", { class: "muted" }, ` the ${tab.lang} program tddlog read, ${size(file.bytes)}`));
}

// An instance of a family: its programs (a tab per language, the files each
// system read, and one file shown, beside the original where asked), then
// its answers. `route` names the file to show.
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
    element("code", { class: "inst" }, inst.name)), playground(s, f, inst));
  if (inst.gen) out.push(markup("p", inst.gen, { class: "note" }));
  if (!inst.tabs) {
    out.push(note("The programs of this instance were not captured."));
  } else {
    out.push(note("The files each system below was handed, byte for byte: written by the release's scripts at "
      + "the run's commit and captured by running the benchmark driver's own code with no system started. The first is the "
      + "original program; each other language's is a translation of it, and says which script wrote it."));
    const unseen = uncaptured(inst);
    if (unseen.length) {
      out.push(element("p", { class: "note uncaptured" }, `Not shown: the files that ${and(unseen)} ${unseen.length === 1 ? "was" : "were"} given, `
        + `which were not captured when ${unseen.length === 1 ? "it" : "they"} ran.`));
    }
    const view = element("div", { class: "viewer" });
    const tabs = scroller(element("table", { class: "given" },
      element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "language"), element("th", { scope: "col" }, "files"),
        element("th", { scope: "col" }, "read by"))),
      element("tbody", {}, ...inst.tabs.map((tab, ti) => element("tr", {},
        element("th", { scope: "row" }, tab.lang, element("span", { class: `badge${tab.original ? " orig" : ""}` }, tab.original ? "original" : "translation")),
        element("td", {}, element("div", { class: "files" }, ...tab.files.map((file, fi) => element("button", { type: "button", class: "file", "data-file": `${ti}:${fi}`,
          onclick: () => {
            showFile(view, fam, inst, ti, fi);
            remember({ file: ti || fi ? [ti, fi] : undefined });
          } }, file.name.split("/").pop(), element("small", {}, size(file.bytes)))))),
        element("td", {}, element("div", { class: "chips" }, ...tab.readers.map((r) => (r === "reference"
          ? element("span", { class: "chip ref" }, inst.answers?.ref ? `the reference: ${inst.answers.ref}` : "the reference")
          : element("span", { class: "chip" }, r))))))))), "whole");
    out.push(tabs, view);
    const [ti, fi] = route.file ?? [0, 0];
    queueMicrotask(() => {
      showFile(view, fam, inst, ti, fi);
      if (route.file) document.querySelector("#programs button.file.on")?.scrollIntoView({ block: "center" });
    });
  }
  if (inst.answers) out.push(answers(inst.answers));
  return out.filter(Boolean);
}

function size(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1 << 20) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`;
  return `${(bytes / (1 << 20)).toFixed(1)} MB`;
}

// A file of an instance (tab ti, file fi; the first where it has none
// such), with what its tab says, the commands that read it, and the
// original beside it where asked.
function showFile(view, fam, inst, ti, fi) {
  if (!inst.tabs[ti]?.files[fi]) [ti, fi] = [0, 0];
  const tab = inst.tabs[ti];
  const file = tab.files[fi];
  for (const b of view.parentElement.querySelectorAll("button.file")) b.classList.toggle("on", b.dataset.file === `${ti}:${fi}`);
  const head = element("div", { class: "tab-says" }, element("p", {}, element("b", {}, tab.says)),
    ...tab.notes.map((n) => element("p", {}, n)));
  if (tab.cmds.length) {
    const dl = element("dl", { class: "cmds" });
    for (const [who, cmd] of tab.cmds) dl.append(element("dt", {}, who), element("dd", {}, element("code", {}, cmd)));
    for (const [who, n] of Object.entries(tab.more)) dl.append(element("dt", {}), element("dd", {}, `${who}: and ${n} more commands of the same form`));
    head.append(dl);
  }
  const canCompare = ti > 0 && inst.tabs[0].original;
  const toggle = element("label", { class: "check" }, element("input", { type: "checkbox", checked: compare && canCompare, disabled: !canCompare,
    onchange: (e) => { compare = e.target.checked; showFile(view, fam, inst, ti, fi); } }), " Beside the original");
  const panes = element("div", { class: `panes${compare && canCompare ? " two" : ""}` });
  if (compare && canCompare) panes.append(pane(fam, inst.tabs[0].files[0]));
  panes.append(pane(fam, file));
  view.replaceChildren(head, toggle, panes);
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

// A file: its name, size and SHA-256, how it was made and any other line
// the data gives above it, and its text (or why it is not shown); a long
// file's first SHOWN_LINES lines, and the text the site keeps to save.
function pane(fam, file) {
  const box = element("div", { class: "pane" });
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
  const cmds = (inst?.tabs ?? []).flatMap((t) => t.cmds).filter(([who]) => readsAs(name0)(who));
  if (cmds.length) parts.push(element("h4", {}, "Commands"), ...cmds.map(([, cmd]) => element("code", { class: "cmd" }, cmd)));
  const read = (inst?.tabs ?? []).flatMap((t, ti) => (t.readers.some(readsAs(name0)) ? t.files.map((file, fi) => [t, file, ti, fi]) : []));
  if (read.length) {
    parts.push(element("h4", {}, "Reads"), element("div", { class: "files" }, ...read.map(([t, file, ti, fi]) => element("button", { type: "button", class: "file",
      onclick: () => {
        // the focus goes on to the file shown, not back to the time
        opener = document.querySelector(`#programs button.file[data-file="${ti}:${fi}"]`);
        go({ ...now, runs: undefined, file: ti || fi ? [ti, fi] : undefined });
      } }, file.name.split("/").pop(), element("small", {}, t.lang)))));
  } else if (mine.length) {
    parts.push(note(`The command and the files ${name0} was given were not captured.`));
  }
  if (rec === "tddlog" && inst) parts.push(playground(s, f, inst));
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

// The kinds, the suites of the kind on show and the families of the suite on
// show; and a search.
function rail(route) {
  const tree = element("ul", { class: "tree" });
  const current = route.view === "kind" ? route.kind : route.suite ? kindFor(route.suite).key : null;
  tree.append(element("li", {}, link({ view: "home" }, "Overview")));
  for (const k of data.kinds) {
    const li = element("li", {}, link({ view: "kind", kind: k.key }, k.title, element("span", { class: "n" }, count(k.instances))));
    if (route.view === "kind" && route.kind === k.key) li.firstChild.setAttribute("aria-current", "page");
    if (k.key === current) {
      const ul = element("ul");
      for (const key of k.suites) {
        const s = data.suites.get(key);
        const sl = element("li", {}, link({ view: "suite", suite: key }, s.title, element("span", { class: "n" }, count(s.instances.length))));
        if (route.view === "suite" && route.suite === key) sl.firstChild.setAttribute("aria-current", "page");
        const fams = familiesOf(key);
        if (key === route.suite && fams) {
          const fl = element("ul");
          const me = column(s, "tddlog");
          const rows = new Map(s.instances.map((row) => [row[0], row]));
          for (const f of fams) {
            const a = f.instances.filter((n) => answered(rows.get(n)?.[me])).length;
            const fi = element("li", {}, link({ view: "family", suite: key, family: f.key },
              element("i", { class: `dot ${a === f.instances.length ? "all" : a ? "some" : ""}`, title: `tddlog answers ${a} of ${f.instances.length}` }),
              f.title, element("span", { class: "n" }, String(f.instances.length))));
            if (route.view === "family" && route.family === f.key) fi.firstChild.setAttribute("aria-current", "page");
            fl.append(fi);
          }
          sl.append(fl);
        }
        ul.append(sl);
      }
      li.append(ul);
    }
    tree.append(li);
  }
  document.getElementById("tree").replaceChildren(tree);
}

// What the search finds, by name: the kinds, suites, families and systems
// (a system once for each kind it is in), then the instances.
let findable = null;
function hits(text) {
  const q = text.trim().toLowerCase();
  if (q.length < 2) return [];
  if (findable === null) {
    findable = [];
    const add = (name, key, what, route) => findable.push({ name, what, route, text: `${name} ${key}`.toLowerCase() });
    for (const k of data.kinds) add(k.title, k.key, "kind of problems", { view: "kind", kind: k.key });
    for (const s of data.suites.values()) add(s.title, s.key, `suite, ${and(kindsOf(s.key).map((k) => k.title))}`, { view: "suite", suite: s.key });
    for (const s of data.suites.values()) {
      for (const f of familiesOf(s.key) ?? []) add(f.title, f.key, `family, ${s.title}`, { view: "family", suite: s.key, family: f.key });
    }
    for (const k of data.kinds) {
      for (const x of k.systems) add(data.names[x], x, `system, ${k.title}`, { view: "kind", kind: k.key });
    }
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
    element("p", {}, "No kind, suite or benchmark of the results is named ", element("code", {}, route.what), "."),
    ...(found.length
      ? [element("p", {}, "What has a name like ", element("code", {}, q), ":"), element("ul", { class: "like" },
        ...found.map((h) => element("li", {}, link(h.route, h.name), element("span", { class: "n" }, ` ${h.what}`))))]
      : [element("p", {}, "Nothing has a name like it; the search box finds a kind, suite, family, system or instance by part of its name.")])];
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
    case "kind": return data.kinds.find((k) => k.key === route.kind).title;
    case "suite": return data.suites.get(route.suite).title;
    case "family": return `${route.instance} · ${route.suite}/${route.family}`;
    case "missing": return "Not found";
    default: return null;
  }
}

// The view the address names: drawn again where it is another view (or
// `again`), else only what is open in it changes: the file on show and the
// drawer.
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
    const parts = route.view === "kind" ? kindView(route) : route.view === "suite" ? suiteView(route)
      : route.view === "family" ? familyView(route, view) : route.view === "missing" ? missing(route) : home();
    view.replaceChildren(...parts);
    // A family's view starts at its top, and another instance of it keeps the place.
    const family = route.view === "family" ? `${route.suite}/${route.family}` : null;
    if (!again && (family === null || family !== show.family)) window.scrollTo({ top: 0 });
    show.family = family;
    const t = title(route);
    document.title = t === null ? "tddlog results" : `${t} · tddlog results`;
  } else if (route.view === "family" && String(route.file ?? "") !== String(before.file ?? "")) {
    const [ti, fi] = route.file ?? [0, 0];
    const button = document.querySelector(`#programs button.file[data-file="${ti}:${fi}"]`);
    if (button) {
      button.click();
      button.scrollIntoView({ block: "center" });
    }
  }
  if (route.runs) runs(route);
  else if (!drawer().hidden) closeRuns();
}

document.addEventListener("DOMContentLoaded", async () => {
  const view = document.getElementById("view");
  try {
    // The page works with the networks and programs alone (suites.json), and
    // the benchmarks' data (benchmarks/, from the baselines branch) where the
    // site has it.
    const [suites, ppl, bench] = await Promise.all([load("suites.json"), load("ppl.json").catch(() => null),
      load("benchmarks/index.json").catch(() => null)]);
    data.bench = bench;
    for (const [suite, fams] of Object.entries(bench?.suites ?? {})) {
      for (const f of fams) for (const name of f.instances) data.family.set(`${suite}/${name}`, f);
    }
    for (const [file, part] of [["suites.json", suites], ["ppl.json", ppl]]) {
      if (part === null) continue;
      Object.assign(data.names, part.names);
      data.kinds.push(...part.kinds.map((k) => ({ ...k, file })));
      for (const f of part.families) for (const s of f.suites) data.suites.set(s.key, { ...s, home: f, file });
    }
    data.kinds.sort((a, b) => a.order - b.order);
    for (const k of data.kinds) {
      for (const key of k.suites) if (!data.suites.has(key)) throw new Error(`the kind ${k.key} has no suite ${key}`);
    }
    document.getElementById("version").textContent = suites.version;
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
