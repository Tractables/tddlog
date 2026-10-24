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

// The most lines of a file the page shows; the whole file is a link.
const SHOWN_LINES = 2000;

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

function count(n) {
  return n >= 10000 ? n.toLocaleString("en").replace(/,/g, " ") : String(n);
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

// ------------------------------------------------------------- the plot

// The instances each system answers within a time limit, the limit on a log
// scale from 0.2 s to the suite's; a line further up and to the left is
// better. Drawn at the width of its box, again when the box changes width;
// a narrow plot names its lines in its legend only.
function cactus(box, suite, names) {
  const width = Math.max(280, Math.floor(box.clientWidth));
  const narrow = width < 560;
  const height = narrow ? 300 : Math.max(380, suite.systems.length * 14 + 22 + 46);
  const n = suite.instances.length;
  const hi = suite.limit;
  const series = suite.systems.map((key, column) => {
    const times = suite.instances.map((row) => row[column + 1]).filter(answered).filter((t) => t <= hi * 1.001).sort((a, b) => a - b);
    return { key, times };
  });
  // The end labels' room: the longest at 7 px a character.
  const label = ({ key, times }) => `${names[key]} ${times.length}`;
  const left = 46, right = narrow ? 12 : Math.max(128, 16 + 7 * Math.max(0, ...series.map((s) => label(s).length))), top = 22, bottom = 46;
  const x0 = left, x1 = width - right, y0 = top, y1 = height - bottom;
  const lx = (t) => x0 + (Math.log10(Math.min(Math.max(t, FLOOR), hi)) - Math.log10(FLOOR)) / (Math.log10(hi) - Math.log10(FLOOR)) * (x1 - x0);
  const ly = (k) => y1 - (k / n) * (y1 - y0);

  const plot = svg("svg", { class: "cactus", width, height, viewBox: `0 0 ${width} ${height}`, role: "img",
    "aria-label": `${suite.title}: instances each system answers within a time limit` });
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

  // tddlog last, so that it is drawn on top.
  const order = [...series].sort((a, b) => (a.key === "tddlog") - (b.key === "tddlog"));
  for (const { key, times } of order) {
    let k = times.filter((t) => t <= FLOOR).length;
    let d = `M${x0},${ly(k)}`;
    for (const t of times.slice(k)) {
      d += `H${lx(t).toFixed(1)}V${ly(++k).toFixed(1)}`;
    }
    d += `H${x1}`;
    plot.append(svg("path", { class: `line s-${base(key)}${key === "tddlog" ? " mine" : ""}${dashed(key) ? " dashed" : ""}`, d, "data-key": key }));
  }
  // The end labels, where there is room: each line's name at its last
  // height, moved apart where two would overlap: pushed down, the lowest
  // kept above the axis and those over it pushed up, so that a crowd at the
  // bottom moves up only as far as it must.
  if (!narrow) {
    const labels = series.map((s) => ({ key: s.key, y: ly(s.times.length), text: label(s) }))
      .sort((a, b) => a.y - b.y);
    for (let i = 1; i < labels.length; i++) labels[i].y = Math.max(labels[i].y, labels[i - 1].y + 13);
    if (labels.length) labels[labels.length - 1].y = Math.min(labels[labels.length - 1].y, y1 + 4);
    for (let i = labels.length - 2; i >= 0; i--) labels[i].y = Math.min(labels[i].y, labels[i + 1].y - 13);
    for (const l of labels) {
      plot.append(svg("text", { class: "end", x: x1 + 8, y: l.y + 4, "data-key": l.key }, l.text));
    }
  }

  // The hover layer: a line at the time under the pointer, and how many each
  // system answers within it.
  const cross = svg("line", { class: "cross", x1: 0, y1: y0, x2: 0, y2: y1, visibility: "hidden" });
  plot.append(cross);
  const area = svg("rect", { class: "hit", x: x0, y: y0, width: x1 - x0, height: y1 - y0 });
  plot.append(area);
  const tip = element("div", { class: "tip", hidden: true });
  const at = (event) => {
    const r = plot.getBoundingClientRect();
    const x = Math.min(Math.max(event.clientX - r.left, x0), x1);
    const t = Math.pow(10, Math.log10(FLOOR) + (x - x0) / (x1 - x0) * (Math.log10(hi) - Math.log10(FLOOR)));
    cross.setAttribute("x1", x);
    cross.setAttribute("x2", x);
    cross.setAttribute("visibility", "visible");
    tip.replaceChildren(element("b", {}, `within ${seconds(t)} s`));
    const rows = series.map(({ key, times }) => [key, times.filter((s) => s <= t).length]).sort((a, b) => b[1] - a[1]);
    for (const [key, k] of rows) {
      tip.append(element("span", {}, element("i", { class: `sw s-${base(key)}${dashed(key) ? " dashed" : ""}` }), `${names[key]} ${k}`));
    }
    tip.hidden = false;
    const w = tip.offsetWidth;
    const px = x + 12 + w > width ? x - 12 - w : x + 12;
    tip.style.left = `${Math.max(0, px)}px`;
    tip.style.top = `${y0}px`;
  };
  area.addEventListener("pointermove", at);
  area.addEventListener("pointerdown", at);
  area.addEventListener("pointerleave", () => {
    cross.setAttribute("visibility", "hidden");
    tip.hidden = true;
  });
  box.replaceChildren(plot, tip);
  return series;
}

// The legend: every line, its name and how many instances it answers, and
// in a kind's plot its total time (total: {system: seconds}, the data's); a
// name under the pointer brings its line forward.
function legend(series, names, n, figure, total) {
  const list = element("ul", { class: "legend" });
  for (const { key, times } of series) {
    const counts = `answers ${count(times.length)} of ${count(n)}${total === undefined ? "" : `, total ${duration(total[key])}`}`;
    const item = element("li", {}, element("i", { class: `sw s-${base(key)}${dashed(key) ? " dashed" : ""}` }),
      element("span", {}, names[key]), element("span", { class: "n" }, counts));
    const focus = (on) => {
      for (const mark of figure.querySelectorAll("[data-key]")) mark.classList.toggle("dim", on && mark.dataset.key !== key);
    };
    item.addEventListener("pointerenter", () => focus(true));
    item.addEventListener("pointerleave", () => focus(false));
    list.append(item);
  }
  return list;
}

// A plot with its legend, drawn again when its box changes width. The
// figure names the systems it draws and the number of its instances; a
// kind's legend gives each system's total time too (suite.total).
function plotted(suite, names) {
  const figure = element("figure", { class: "plot", "data-systems": suite.systems.join(" "),
    "data-instances": String(suite.instances.length) });
  const box = element("div", { class: "box" });
  figure.append(box);
  let last = 0;
  const redraw = () => {
    const w = box.clientWidth;
    if (w === 0 || w === last) return;
    last = w;
    const series = cactus(box, suite, names);
    const old = figure.querySelector(".legend");
    const fresh = legend(series, names, suite.instances.length, figure, suite.total);
    if (old) old.replaceWith(fresh);
    else box.before(fresh);
  };
  new ResizeObserver(redraw).observe(box);
  return figure;
}

// ------------------------------------------------------------- the data

const data = {
  names: {},        // a system's key: its name
  kinds: [],        // the data's kinds, in the README's order
  suites: new Map(),   // a suite's key: the suite, with `home`, the data's family that holds it
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
// The column of a system in a suite's rows (its first is the instance).
const column = (suite, key) => suite.systems.indexOf(key) + 1;
// The record's system of a column: alea-process's runs are Alea.jl's.
const recorded = (key) => data.bench?.records?.[key] ?? base(key);

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

function parse(hash) {
  const old = hash.replace(/^#/, "");
  // the page before this one named a family of suites by its key
  if (!old.startsWith("/")) {
    if (data.kinds.some((k) => k.key === old)) return { view: "kind", kind: old };
    if (data.suites.has(old)) return { view: "suite", suite: old };
    return { view: "home" };
  }
  const parts = old.slice(1).split("/").filter(Boolean).map(decodeURIComponent);
  if (parts.length === 0) return { view: "home" };
  if (parts[0] === "kind") return data.kinds.some((k) => k.key === parts[1]) ? { view: "kind", kind: parts[1] } : { view: "home" };
  const suite = parts[0];
  if (!data.suites.has(suite)) return { view: "home" };
  if (parts.length === 1) return { view: "suite", suite };
  if (parts[1].startsWith("@")) {
    const name = parts[1].slice(1);
    const family = data.family.get(`${suite}/${name}`);
    return family ? { view: "family", suite, family, instance: name } : { view: "suite", suite };
  }
  const fams = familiesOf(suite);
  const family = fams?.find((f) => f.key === parts[1]);
  if (!family) return { view: "suite", suite };
  return { view: "family", suite, family: family.key, instance: family.instances.includes(parts[2]) ? parts[2] : family.instances[0] };
}

function href(route) {
  switch (route.view) {
    case "kind": return `#/kind/${route.kind}`;
    case "suite": return `#/${route.suite}`;
    case "family": return `#/${route.suite}/${encodeURIComponent(route.family)}${route.instance ? `/${encodeURIComponent(route.instance)}` : ""}`;
    default: return "#/";
  }
}

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
      ...[[count(instances), "instances"], [String(data.suites.size), "suites"], [String(others.size), "other systems"]]
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
    grid.append(link({ view: "kind", kind: k.key },
      element("article", { class: "kind-card" },
        element("h3", {}, k.title),
        element("p", { class: "n" }, `${plural(k.instances, k.noun.replace(/s$/, ""), k.noun)}, ${plural(k.systems.length - 1, "other system")}`),
        meter("tddlog", k.answers.tddlog, k.instances, true),
        meter(data.names[best], most, k.instances, false),
        element("p", { class: "total" }, element("b", {}, `${times(k.total[fastest], k.total.tddlog)} less time`),
          ` than the fastest other system, ${data.names[fastest]}: ${duration(k.total.tddlog)} against ${duration(k.total[fastest])}`))));
  }
  out.push(grid, how());
  return out;
}

// A system's answers out of n, as a bar.
function meter(name, k, n, mine) {
  return element("div", { class: `meter${mine ? " mine" : ""}` },
    element("span", { class: "who" }, name),
    element("span", { class: "bar", role: "img", "aria-label": `${k} of ${n}` }, element("i", { style: `width:${(100 * k / n).toFixed(1)}%` })),
    element("span", { class: "k" }, `${count(k)}/${count(n)}`));
}

// How the plots and totals are read, under every overview.
function how() {
  return element("details", { class: "how" }, element("summary", {}, "How to read these results"),
    element("p", {}, "A plot counts the instances each system answers within a time limit, on a log scale from 0.2 s to "
      + "the suite's limit: a line further up and to the left is better, and a run that takes less than 0.2 s counts at "
      + "0.2 s, since differences of a few milliseconds of start-up are not compared."),
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
      element("th", { scope: "row" }, element("i", { class: `sw s-${base(x)}${dashed(x) ? " dashed" : ""}` }), data.names[x]),
      element("td", {}, element("span", { class: "bar" }, element("i", { style: `width:${(100 * k.answers[x] / k.instances).toFixed(1)}%` })),
        ` ${count(k.answers[x])}`),
      element("td", { class: "num" }, duration(k.total[x])),
      element("td", { class: "num" }, x === "tddlog" ? "" : times(k.total[x], k.total.tddlog))));
  }
  return element("div", { class: "tscroll whole" }, element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col" }, `answers, of ${count(k.instances)}`),
      element("th", { scope: "col", class: "num" }, "total time"),
      element("th", { scope: "col", class: "num" }, "its time over tddlog's"))), body));
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
    element("p", { class: "note" }, whole.about), cannot(whole));
  const body = element("tbody");
  for (const key of k.suites) {
    const s = data.suites.get(key);
    const mine = s.instances.filter((row) => answered(row[column(s, "tddlog")])).length;
    let best = null;
    for (const x of s.systems) {
      if (x === "tddlog" || dashed(x)) continue;
      const a = s.instances.filter((row) => answered(row[column(s, x)])).length;
      if (best === null || a > best[1]) best = [x, a];
    }
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "suite", suite: key }, s.title)),
      element("td", { class: "num" }, count(s.instances.length)), element("td", { class: "num" }, count(mine)),
      element("td", {}, best ? `${data.names[best[0]]}, ${count(best[1])}` : "")));
  }
  out.push(element("h3", {}, "Suites"), element("div", { class: "tscroll" }, element("table", { class: "board" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "suite"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num" }, "tddlog answers"), element("th", { scope: "col" }, "the most answers of another system"))), body)));
  const general = (family.findings ?? []).filter((c) => !c.suite);
  if (own && general.length) out.push(element("h3", {}, "Findings"), findings(general));
  out.push(setup(family));
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
    element("p", { class: "note" }, "Every table and its details: ", element("a", { href: home.docs }, "docs/benchmarks.md"), "."));
}

function suiteView(route) {
  const s = data.suites.get(route.suite);
  const k = kindFor(s.key);
  const out = [crumbs(["Results", { view: "home" }], [k.title, { view: "kind", kind: k.key }], [s.title]),
    element("h2", {}, s.title), brief(s.about, "lede"),
    element("h3", {}, "Instances answered within each time limit"), plotted(s, data.names)];
  const fams = familiesOf(s.key);
  if (fams) {
    out.push(element("h3", {}, "Families"), element("p", { class: "note" },
      "A family is one program at several sizes, or programs written the same way. Each cell counts the family's instances "
      + "a system answers; a family opens to its times, its programs in every language and every run."), familyTable(s, fams));
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
  out.push(element("details", { class: "instances" }, element("summary", {}, `Every instance's times (${count(s.instances.length)})`),
    ...table(s)));
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
  return element("div", { class: "tscroll" }, element("table", { class: "grid counts" }, element("thead", {}, head), body));
}

function headToHead(s, rows, paras) {
  const body = element("tbody");
  for (const r of rows.sort((a, b) => b.theirs - a.theirs || a.theirTotal - b.theirTotal)) {
    const name = data.names[r.key];
    const para = paras.get(name.replace(/'s process$/, " (with Julia's start-up and compilation)"))
      ?? paras.get(name) ?? [...paras].find(([k]) => k.startsWith(`${data.names[base(r.key)]} (`) && dashed(r.key) === /start-up/.test(k) && !/without/.test(k))?.[1]
      ?? [...paras].find(([k]) => k.startsWith(`${name} (`))?.[1];
    const tr = element("tr", {},
      element("th", { scope: "row" }, element("i", { class: `sw s-${base(r.key)}${dashed(r.key) ? " dashed" : ""}` }), name),
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
  return element("div", {}, element("div", { class: "tscroll whole" }, element("table", { class: "board h2h" },
    element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
      element("th", { scope: "col", class: "num" }, "both given"),
      element("th", { scope: "col", class: "num" }, "answered, tddlog / it"),
      element("th", { scope: "col", class: "num" }, "total time, tddlog / it"),
      element("th", { scope: "col", class: "num" }, "tddlog"))), body)),
    element("p", { class: "note" }, `Over the instances both are given: how many each answers, and the total times over those either `
      + `answers, a run that does not answer counted at the ${s.limit} s limit and a time under ${FLOOR} s at ${FLOOR} s.`));
}

// Every instance's times, sortable by a column; an instance's name opens
// its family.
function table(suite) {
  const cols = suite.systems;
  const fastest = (row) => {
    const ts = cols.map((key, i) => (dashed(key) ? null : row[i + 1])).filter(answered);
    return ts.length ? Math.min(...ts) : null;
  };
  const quick = (row) => {
    const ran = row.slice(1).filter((v) => v !== null && v !== "skipped");
    return ran.length > 0 && ran.every((v) => answered(v) && v < FLOOR);
  };
  const nquick = suite.instances.filter(quick).length;
  const head = element("tr", {}, element("th", { scope: "col", "data-col": "0" }, "instance"));
  cols.forEach((key, i) => head.append(element("th", { scope: "col", class: "num", "data-col": String(i + 1) }, data.names[key])));
  const body = element("tbody");
  const seen = new Set();
  const rowOf = (row) => {
    const best = fastest(row);
    const family = data.family.get(`${suite.key}/${row[0]}`);
    const tr = element("tr", {}, element("th", { scope: "row" },
      family ? link({ view: "family", suite: suite.key, family, instance: row[0] }, row[0]) : row[0]));
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

  // A header sorts by its column: answers by time, failures after them;
  // the instance column restores the suite's order.
  let sorted = 0;
  head.addEventListener("click", (event) => {
    const th = event.target.closest("th");
    if (th === null) return;
    const col = Number(th.dataset.col);
    const key = (row) => (answered(row[col]) ? row[col] : Infinity);
    const order = col === 0 || col === sorted ? rows : [...rows].sort((a, b) => key(a[0]) - key(b[0]));
    sorted = col === sorted ? 0 : col;
    for (const h of head.children) h.removeAttribute("aria-sort");
    if (sorted) th.setAttribute("aria-sort", "ascending");
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
  parts.push(element("div", { class: "tscroll" }, grid));
  const notes = [`Seconds; the fastest answer of an instance in bold. A header sorts by its column.`];
  if (familiesOf(suite.key)) notes.push("An instance's name opens its family: its programs in every language and every run.");
  for (const status of Object.keys(FAILURES)) {
    if (!seen.has(status)) continue;
    notes.push(`${FAILURES[status][0](suite.limit)}: ${FAILURES[status][1]}.`);
  }
  if (suite.instances.some((row) => row.slice(1).includes(null))) notes.push("An empty cell: the system was not given the instance.");
  parts.push(element("p", { class: "note" }, notes.join(" ")));
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
  const out = [crumbs(["Results", { view: "home" }], [k.title, { view: "kind", kind: k.key }], [s.title, { view: "suite", suite: s.key }], [f.title]),
    element("h2", {}, f.title),
    element("div", { class: "chips" }, element("span", { class: "chip" }, plural(f.instances.length, "instance")),
      element("span", { class: "chip" }, `${s.limit} s a run`))];
  const programs = element("section", { class: "programs", id: "programs" }, element("p", { class: "note" }, "Loading the programs and the runs…"));
  out.push(element("h3", {}, "Times"), timesGrid(s, f, rows, short, route), programs);
  familyData(s.key, f.key).then((fam) => {
    if (view.dataset.route !== href(route)) return;   // another view was chosen meanwhile
    programs.replaceChildren(...instanceView(s, f, fam, route, short));
  }).catch((failure) => {
    programs.replaceChildren(element("p", { class: "failed" }, `The programs and runs did not load: ${failure.message}`));
  });
  return out;
}

// A family's instances by the systems given them: each time shaded by its
// place between 0.2 s and the limit on a log scale, the fastest marked, a
// failure by its word; a time opens the system's runs of the instance, a
// name the instance's programs.
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
    const best = ts.length ? Math.min(...ts) : null;
    const tr = element("tr", { class: name === route.instance ? "on" : null },
      element("th", { scope: "row" }, link({ ...route, instance: name }, short.get(name))));
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
      tr.append(element("td", { class: cls.trim() }, element("button", { type: "button", style: `--p:${p.toFixed(3)}`, title,
        onclick: () => runs(s, f, name, x) }, text)));
    }
    body.append(tr);
  }
  const key = [element("span", {}, element("i", { class: "heat lo" }), `${FLOOR} s`), element("span", {}, element("i", { class: "heat hi" }), `${s.limit} s`),
    element("span", {}, "underlined: the fastest")];
  for (const v of Object.keys(FAILURES)) if (seen.has(v)) key.push(element("span", {}, `${FAILURES[v][0](s.limit)}: ${FAILURES[v][1].split(":")[0]}`));
  key.push(element("span", {}, "a time opens its runs"));
  const hidden = s.systems.length - given.length;
  const box = element("div", { class: "times-box" }, element("div", { class: "tscroll" }, element("table", { class: "grid" }, element("thead", {}, head), body)),
    element("p", { class: "key" }, ...key));
  if (hidden) {
    box.append(element("button", { type: "button", class: "plain", onclick: () => { showAll = !showAll; show(); } },
      showAll ? "Hide the systems not given these instances" : `Show the ${plural(hidden, "system")} not given these instances`));
  }
  return box;
}

// An instance of a family: its programs (a tab per language, the files each
// system read, and one file shown, beside the original where asked), then
// its answers.
let compare = false;
function instanceView(s, f, fam, route, short) {
  const inst = fam.instances.find((i) => i.name === route.instance) ?? fam.instances[0];
  const out = [];
  const pick = f.instances.length > 12
    ? element("select", { "aria-label": "Instance", onchange: (e) => go({ ...route, instance: e.target.value }) },
      ...f.instances.map((n) => element("option", { value: n, selected: n === inst.name }, short.get(n))))
    : element("div", { class: "seg", role: "group", "aria-label": "Instance" },
      ...f.instances.map((n) => link({ ...route, instance: n }, short.get(n))).map((a, i) => {
        if (f.instances[i] === inst.name) a.setAttribute("aria-current", "true");
        return a;
      }));
  out.push(element("h3", {}, "Programs"), element("div", { class: "pick" }, element("span", { class: "muted" }, "instance"), pick,
    element("code", { class: "inst" }, inst.name)));
  if (inst.gen) out.push(markup("p", inst.gen, { class: "note" }));
  if (!inst.tabs) {
    out.push(element("p", { class: "note" }, "The programs of this instance were not captured."));
  } else {
    out.push(element("p", { class: "note" }, "The files each system was handed, byte for byte: written by the release's scripts at "
      + "the run's commit and captured by running the benchmark driver's own code with no system started. The first is the "
      + "original program; each other language's is a translation of it, and says which script wrote it."));
    const view = element("div", { class: "viewer" });
    const tabs = element("div", { class: "tscroll" }, element("table", { class: "given" },
      element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "language"), element("th", { scope: "col" }, "files"),
        element("th", { scope: "col" }, "read by"))),
      element("tbody", {}, ...inst.tabs.map((tab, ti) => element("tr", {},
        element("th", { scope: "row" }, tab.lang, element("span", { class: `badge${tab.original ? " orig" : ""}` }, tab.original ? "original" : "translation")),
        element("td", {}, element("div", { class: "files" }, ...tab.files.map((file, fi) => element("button", { type: "button", class: "file", "data-file": `${ti}:${fi}`,
          onclick: () => showFile(view, fam, inst, ti, fi) }, file.name.split("/").pop(), element("small", {}, size(file.bytes)))))),
        element("td", {}, element("div", { class: "chips" }, ...tab.readers.map((r) => element("span", { class: "chip" }, r)))))))));
    out.push(tabs, view);
    queueMicrotask(() => showFile(view, fam, inst, 0, 0));
  }
  if (inst.answers) out.push(answers(inst.answers));
  return out;
}

function size(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1 << 20) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`;
  return `${(bytes / (1 << 20)).toFixed(1)} MB`;
}

// A file of an instance (tab ti, file fi), with what its tab says, the
// commands that read it, and the original beside it where asked.
function showFile(view, fam, inst, ti, fi) {
  const tab = inst.tabs[ti];
  const file = tab?.files[fi];
  if (!file) return;
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

// A file: its name, size and SHA-256, how it was made, and its text (or
// why it is not shown); a long file's first SHOWN_LINES lines.
function pane(fam, file) {
  const box = element("div", { class: "pane" });
  box.append(element("div", { class: "fhead" }, element("code", { class: "fname" }, file.name),
    element("span", {}, `${count(file.bytes)} bytes${file.lines !== undefined ? `, ${count(file.lines)} lines` : ""}`),
    element("span", { class: "sha", title: "SHA-256" }, `sha256 ${file.sha}`)));
  if (file.made) box.append(markup("p", file.made, { class: "made" }));
  if (file.held) {
    box.append(markup("p", file.held, { class: "held" }));
    return box;
  }
  if (file.archive) {
    box.append(element("p", { class: "held" }, "An archive, not shown; its SHA-256 identifies it."));
    return box;
  }
  const code = element("div", { class: "code" }, element("p", { class: "note" }, "Loading…"));
  box.append(code);
  const fill = (text) => {
    const lines = text.split("\n");
    if (lines[lines.length - 1] === "") lines.pop();
    const shown = lines.slice(0, SHOWN_LINES);
    const body = shown.join("\n");
    const pre = element("pre", { class: "src" });
    if (typeof tddlogCode !== "undefined" && /\.pl$/.test(file.name)) pre.innerHTML = tddlogCode.highlight("problog", body);
    else pre.textContent = body;
    const gutter = element("pre", { class: "gutter", "aria-hidden": "true" }, shown.map((_, i) => i + 1).join("\n"));
    const parts = [element("div", { class: "lines" }, gutter, pre)];
    const notes = [];
    if (lines.length > SHOWN_LINES) notes.push(`The first ${count(SHOWN_LINES)} of its ${count(lines.length)} lines.`);
    if (file.part) notes.push(`The first ${count(file.part)} bytes${file.gzip ? ", decompressed" : ""}: the file is larger than the site keeps; the command above writes it again, and its SHA-256 checks the copy.`);
    if (file.src) parts.push(element("p", { class: "note" }, ...notes, " ", element("a", { href: `benchmarks/${file.src}` }, notes.length ? "The text the site keeps" : "The file alone")));
    else if (notes.length) parts.push(element("p", { class: "note" }, ...notes));
    code.replaceChildren(...parts);
  };
  if (fam.texts[file.sha] !== undefined) fill(fam.texts[file.sha]);
  else if (file.src) {
    fetch(`benchmarks/${file.src}`).then((r) => (r.ok ? r.text() : Promise.reject(new Error(String(r.status)))))
      .then(fill, (failure) => code.replaceChildren(element("p", { class: "failed" }, `The file did not load: ${failure.message}`)));
  } else code.replaceChildren(element("p", { class: "note" }, "Not kept."));
  return box;
}

function answers(a) {
  const head = element("tr", {}, element("th", { scope: "col" }, a.log10z ? "quantity" : "query"),
    ...a.cols.map((c) => element("th", { scope: "col", class: "num" }, c)));
  const body = element("tbody", {}, ...a.rows.map(([q, ...cells]) => element("tr", {}, element("th", { scope: "row" }, q),
    ...cells.map((c) => element("td", { class: "num" }, c)))));
  const notes = [];
  if (a.ref) notes.push(`The reference is ${a.ref}. Each system's answers of its first repetition that answered.`);
  if (a.queries > a.rows.length) notes.push(`The first ${a.rows.length} of ${count(a.queries)} queries; the largest difference over all of them is in each run's "from the reference".`);
  return element("details", { class: "answers" }, element("summary", {}, a.log10z ? "The partition function, log10 Z" : `The answers (${plural(a.queries, "query", "queries")})`),
    element("div", { class: "tscroll" }, element("table", { class: "times" }, element("thead", {}, head), body)),
    notes.length ? element("p", { class: "note" }, notes.join(" ")) : null);
}

// ------------------------------------------------------------- a system's runs

const drawer = () => document.getElementById("drawer");
let opener = null;

function closeRuns() {
  drawer().hidden = true;
  opener?.focus();
  opener = null;
}

// A system's runs of an instance, beside the view: its time in the table
// and what it is, the commands and files it was given, then each run of
// each of its forms, the latest first, runs of the same outcome folded
// into one; a later run in lanes of its own beside that run's tddlog.
async function runs(s, f, name, key) {
  opener = document.activeElement;
  const box = drawer();
  const v = s.instances.find((row) => row[0] === name)[column(s, key)];
  const what = answered(v) ? `${seconds(v)} s` : v === null ? "not given" : FAILURES[v]?.[1] ?? v;
  box.replaceChildren(element("button", { type: "button", class: "plain close", onclick: closeRuns }, "Close"),
    element("p", { class: "muted" }, element("code", {}, name)), element("h3", {}, data.names[key]),
    element("p", {}, element("b", {}, "In the table: "), what), element("p", { class: "note" }, "Loading…"));
  box.hidden = false;
  box.querySelector(".close").focus();
  let fam;
  try {
    fam = await familyData(s.key, f.key);
  } catch (failure) {
    box.lastChild.replaceWith(element("p", { class: "failed" }, `The runs did not load: ${failure.message}`));
    return;
  }
  const inst = fam.instances.find((i) => i.name === name);
  const rec = recorded(key);
  const mine = (inst?.runs ?? []).filter((r) => r.system === rec);
  const parts = [];
  const about = [];
  if (dashed(key)) about.push("The whole process's time, with the language's start and compilation; the other time of the same system leaves them out, as its paper times it.");
  if (inst?.reps > 1) about.push("A time is the median of the repetitions, a failed one counted as infinite.");
  if (new Set(mine.map((r) => r.label)).size > 1) about.push("A system run in several forms has the time of its fastest form.");
  if (inst?.notes?.[rec]) about.push(inst.notes[rec]);
  if (about.length) parts.push(element("p", { class: "note" }, about.join(" ")));
  const name0 = data.names[rec] ?? data.names[key];
  const ofMine = (label) => label === name0 || label.startsWith(`${name0} `);
  const cmds = (inst?.tabs ?? []).flatMap((t) => t.cmds).filter(([who]) => ofMine(who));
  if (cmds.length) parts.push(element("h4", {}, "Commands"), ...cmds.map(([, cmd]) => element("code", { class: "cmd" }, cmd)));
  const read = (inst?.tabs ?? []).flatMap((t, ti) => (t.readers.some(ofMine) ? t.files.map((file, fi) => [t, file, ti, fi]) : []));
  if (read.length) {
    parts.push(element("h4", {}, "Reads"), element("div", { class: "files" }, ...read.map(([t, file, ti, fi]) => element("button", { type: "button", class: "file",
      onclick: () => {
        closeRuns();
        const target = { view: "family", suite: s.key, family: f.key, instance: name };
        pendingFile = [ti, fi];
        if (location.hash === href(target)) openPending();
        else go(target);
      } }, file.name.split("/").pop(), element("small", {}, t.lang)))));
  }
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
  if (!shown.length) parts.push(element("p", { class: "note" }, "No run of this system on this instance."));
  shown.forEach((e, i) => {
    const { r, more } = e;
    const hit = e === inTable;
    const at = (r.cols ?? []).indexOf("seconds");
    const summary = r.skipped ? "not run" : r.rows.map((row) => (row.cells[1] === "answered" ? `${row.cells[at]} s` : row.cells[1])).join(", ");
    const block = element("details", { class: "run", open: i === 0 || hit },
      element("summary", {}, element("b", {}, r.label), hit ? element("span", { class: "badge orig" }, "time in the table") : null,
        element("span", { class: "muted" }, ` ${summary}${more ? `; and ${plural(more, "run")} with the same outcome` : ""}`)),
      element("p", { class: "note" }, r.later ? "A later run, in lanes of its own." : "The release run."));
    if (r.later && rec !== "tddlog") {
      const t = tddlogIn(r.later).map(med).filter((x) => x !== null);
      if (t.length) block.append(element("p", { class: "note" }, `tddlog in the same lanes: ${seconds(t[0])} s.`));
    }
    if (r.skipped) block.append(element("p", { class: "note" }, `Not run in any repetition: ${FAILURES.skipped[1]}.`));
    else {
      block.append(element("div", { class: "tscroll" }, element("table", { class: "runs" },
        element("thead", {}, element("tr", {}, ...r.cols.map((c) => element("th", { scope: "col" }, c)))),
        element("tbody", {}, ...r.rows.flatMap((row) => [
          element("tr", {}, ...row.cells.map((c, j) => element("td", { class: r.cols[j] === "sizes" ? "sizes" : null }, c))),
          row.error ? element("tr", { class: "err" }, element("td", { colspan: String(r.cols.length) }, element("pre", {}, row.error))) : null,
          row.note ? element("tr", { class: "err" }, element("td", { colspan: String(r.cols.length) }, row.note)) : null,
        ])))));
    }
    parts.push(block);
  });
  box.lastChild.replaceWith(...parts);
}

// A file asked for from the runs, shown once its family's view is drawn.
let pendingFile = null;
function openPending() {
  if (!pendingFile) return;
  const [ti, fi] = pendingFile;
  const button = document.querySelector(`#programs button.file[data-file="${ti}:${fi}"]`);
  if (!button) return;
  pendingFile = null;
  button.click();
  button.scrollIntoView({ block: "center" });
}

// ------------------------------------------------------------- the rail

// The kinds, the suites of the kind on show and the families of the suite on
// show; and a search over every instance's name.
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

function search(text) {
  const out = document.getElementById("found");
  const q = text.trim().toLowerCase();
  if (q.length < 2) {
    out.replaceChildren();
    return;
  }
  const hits = [];
  for (const s of data.suites.values()) {
    for (const row of s.instances) {
      if (row[0].toLowerCase().includes(q)) hits.push([row[0], s]);
    }
  }
  const list = element("ul", {}, ...hits.slice(0, 40).map(([name, s]) => {
    const family = data.family.get(`${s.key}/${name}`);
    const route = family ? { view: "family", suite: s.key, family, instance: name } : { view: "suite", suite: s.key };
    return element("li", {}, link(route, element("code", {}, name), element("span", { class: "n" }, s.title)));
  }));
  out.replaceChildren(element("p", { class: "n" }, hits.length > 40 ? `the first 40 of ${count(hits.length)}` : plural(hits.length, "instance")), list);
}

// ------------------------------------------------------------- the page

function go(route) {
  location.hash = href(route);
}

function show() {
  const route = parse(location.hash);
  const canonical = href(route);
  if (route.view !== "home" && location.hash !== canonical) {
    history.replaceState(null, "", canonical);
  }
  rail(route);
  const view = document.getElementById("view");
  const same = view.dataset.route === canonical;
  view.dataset.route = canonical;
  const parts = route.view === "kind" ? kindView(route) : route.view === "suite" ? suiteView(route)
    : route.view === "family" ? familyView(route, view) : home();
  view.replaceChildren(...parts);
  if (!same && route.view !== "family") window.scrollTo({ top: 0 });
  if (route.view === "family") {
    const prev = show.family;
    show.family = `${route.suite}/${route.family}`;
    if (prev !== show.family) window.scrollTo({ top: 0 });
    familyData(route.suite, route.family).then(() => queueMicrotask(openPending), () => {});
  } else show.family = null;
  document.title = route.view === "home" ? "tddlog results" : `${view.querySelector("h2")?.textContent ?? ""} · tddlog results`;
}

document.addEventListener("DOMContentLoaded", async () => {
  const view = document.getElementById("view");
  try {
    const parts = await Promise.all(["suites.json", "ppl.json"].map((file) => load(file).catch((failure) => {
      if (file === "ppl.json") return null;     // the page works with the networks and programs alone
      throw failure;
    })));
    // The benchmarks' data (benchmarks/, from the baselines branch), where the site has it.
    data.bench = await load("benchmarks/index.json").catch(() => null);
    for (const p of parts.filter(Boolean)) {
      Object.assign(data.names, p.names);
      data.kinds.push(...p.kinds);
      for (const f of p.families) for (const s of f.suites) data.suites.set(s.key, { ...s, home: f });
    }
    data.kinds.sort((a, b) => a.order - b.order);
    for (const k of data.kinds) {
      for (const key of k.suites) if (!data.suites.has(key)) throw new Error(`the kind ${k.key} has no suite ${key}`);
    }
    for (const [suite, fams] of Object.entries(data.bench?.suites ?? {})) {
      for (const f of fams) for (const name of f.instances) data.family.set(`${suite}/${name}`, f.key);
    }
    document.getElementById("version").textContent = parts[0].version;
    window.addEventListener("hashchange", show);
    document.getElementById("search").addEventListener("input", (e) => search(e.target.value));
    const toggle = document.getElementById("browse");
    toggle.addEventListener("click", () => {
      const rail = document.getElementById("rail");
      rail.classList.toggle("open");
      toggle.setAttribute("aria-expanded", String(rail.classList.contains("open")));
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !drawer().hidden) closeRuns();
    });
    show();
  } catch (failure) {
    view.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
