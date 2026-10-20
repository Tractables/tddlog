"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl and
// Roulette), both written by the generator of README.md's results and
// docs/benchmarks.md. A family of suites is its findings in prose, a plot
// of the instances each system answers within a time limit, the table of
// every instance's times, and how the runs were made.

// The stamp of the page's own reference (results.js?v=<sha>) goes on to the
// data, as the playground's does to its files.
const stamp = document.currentScript === null ? "" : new URL(document.currentScript.src).search;

// The time axis starts at 0.1 s: runs under 0.1 s are not told apart.
const FLOOR = 0.1;

// How a failure reads in a table cell, and what it means under the table.
const FAILURES = {
  timeout: [(limit) => `>${limit}`, "the time limit ran out"],
  memout: [() => "memory", "out of memory (the address space a run gets, or the Java heap)"],
  error: [() => "error", "the system stopped with an error"],
  unsupported: [() => "n/a", "the system does not take the program (aspmc's front end does not condition on evidence; tddlog's decision vtree needs a guard)"],
  skipped: [() => "skipped", "not run: the system had failed on a smaller size of the same program"],
  wrong: [() => "wrong", "an answer off the reference by more than 1e-6"],
  zero: [() => "zero", "every answer printed as 0 where the reference's are positive"],
  underflow: [() => "underflow", "every answer printed as 0 where the probability of the evidence is below the smallest double"],
};

const svgNS = "http://www.w3.org/2000/svg";

function element(tag, attributes = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attributes)) {
    if (key === "class") node.className = value;
    else node.setAttribute(key, value);
  }
  for (const child of children) node.append(child);
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

function count(n) {
  return n >= 10000 ? n.toLocaleString("en").replace(/,/g, " ") : String(n);
}

function seconds(t) {
  if (t < 0.01) return t.toFixed(3);
  if (t < 1) return t.toFixed(2);
  if (t < 100) return t.toFixed(1);
  return t.toFixed(0);
}

const answered = (value) => typeof value === "number";
// A process time (Alea.jl's and Roulette's with their start-up) is drawn
// dashed, in its system's colour.
const base = (key) => key.replace(/-process$/, "");
const dashed = (key) => key.endsWith("-process");

// ------------------------------------------------------------- the plot

// The instances each system answers within a time limit, the limit on a log
// scale from 0.1 s to the suite's; a line further up and to the left is
// better. Drawn at the width of its box, again when the box changes width.
function cactus(box, suite, names) {
  const width = Math.max(280, Math.floor(box.clientWidth));
  const narrow = width < 560;
  const height = narrow ? 300 : Math.max(380, suite.systems.length * 14 + 22 + 46);
  const left = 46, right = narrow ? 12 : 128, top = 22, bottom = 46;
  const x0 = left, x1 = width - right, y0 = top, y1 = height - bottom;
  const n = suite.instances.length;
  const hi = suite.limit;
  const lx = (t) => x0 + (Math.log10(Math.min(Math.max(t, FLOOR), hi)) - Math.log10(FLOOR)) / (Math.log10(hi) - Math.log10(FLOOR)) * (x1 - x0);
  const ly = (k) => y1 - (k / n) * (y1 - y0);

  const plot = svg("svg", { class: "cactus", width, height, viewBox: `0 0 ${width} ${height}`, role: "img",
    "aria-label": `${suite.title}: instances each system answers within a time limit` });
  const ticks = [0.1, 1, 10, 100, 1000].filter((t) => t < hi * 0.75).concat([hi]);
  for (const t of ticks) {
    const x = lx(t);
    plot.append(svg("line", { class: "grid", x1: x, y1: y0, x2: x, y2: y1 }));
    plot.append(svg("text", { class: "tick", x, y: y1 + 16, "text-anchor": "middle" }, t === FLOOR ? "≤0.1" : String(t)));
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

  const series = suite.systems.map((key, column) => {
    const times = suite.instances.map((row) => row[column + 1]).filter(answered).filter((t) => t <= hi * 1.001).sort((a, b) => a - b);
    return { key, times };
  });
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
  // height, moved apart where two would overlap.
  if (!narrow) {
    const labels = series.map(({ key, times }) => ({ key, y: ly(times.length), text: `${names[key]} ${times.length}` }))
      .sort((a, b) => a.y - b.y);
    for (let i = 1; i < labels.length; i++) labels[i].y = Math.max(labels[i].y, labels[i - 1].y + 13);
    const over = labels.length ? labels[labels.length - 1].y - (y1 + 4) : 0;
    if (over > 0) for (const l of labels) l.y -= over;
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
  const tip = element("div", { class: "tip", hidden: "" });
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

// The legend: every line, its name and how many instances it answers; a
// name under the pointer brings its line forward.
function legend(series, names, n, figure) {
  const list = element("ul", { class: "legend" });
  for (const { key, times } of series) {
    const item = element("li", {}, element("i", { class: `sw s-${base(key)}${dashed(key) ? " dashed" : ""}` }),
      element("span", {}, names[key]), element("span", { class: "n" }, `${times.length} of ${count(n)}`));
    const focus = (on) => {
      for (const mark of figure.querySelectorAll("[data-key]")) mark.classList.toggle("dim", on && mark.dataset.key !== key);
    };
    item.addEventListener("pointerenter", () => focus(true));
    item.addEventListener("pointerleave", () => focus(false));
    list.append(item);
  }
  return list;
}

// ------------------------------------------------------------- every problem

// Why a system is not given some of the problems: the reasons the coverage
// of the run states, by system (a system's variants, as storm-sparse and
// storm-exact, share their stem's).
const CANNOT = {
  ace: "it reads Bayesian and Markov networks, not logic programs",
  merlin: "it reads Bayesian and Markov networks, not logic programs",
  agrum: "it reads Bayesian networks, not logic programs",
  pgmpy: "it reads Bayesian networks, not logic programs",
  libdai: "it reads graphical models, not logic programs",
  tensorinference: "it reads graphical models, not logic programs",
  aspmc: "its ProbLog front end ignores evidence, so it is not given programs with evidence",
  dice: "its language has no relations and no recursion, and it is given the programs written in it",
  nodice: "its language has no relations and no recursion, and it is given the programs written in it",
  psi: "its language has no relations and no recursion, and it is given the programs written in it",
  alea: "it takes programs written in its own language, not relational logic programs",
  roulette: "it takes programs written in its own language, not relational logic programs",
  geni: "it runs only its own programs",
  genfer: "it runs only its own programs",
  sppl: "it runs only its own programs",
  pluck: "it runs only its own programs",
  provsql: "it reads SQL, and only the TPC-H queries are translated for it",
  storm: "it reads Markov chains, and only the weather factory is written for it",
  prism: "it reads Markov chains, and only the weather factory is written for it",
  ganak: "it reads weighted CNFs, which only the partition-function sets of the networks are written as",
  sharpsat: "it reads weighted CNFs, which only the partition-function sets of the networks are written as",
};
// The network readers' reason holds for the program suites only: a network
// suite they missed is one they were not run on in this release.
const READS_NETWORKS = new Set(["ace", "merlin", "agrum", "pgmpy", "libdai", "tensorinference"]);
const NETWORK_SUITES = new Set(["std_bn", "uai", "uai_pr", "large", "ace", "uaipr", "uaitpr"]);
const stem = (key) => key.split("-")[0];

// The instances of several suites as one suite: its systems the union of
// theirs (tddlog first), a system that did not run an instance an empty
// cell, a time past its own suite's limit a timeout.
function merged(title, about, suites) {
  const systems = [];
  for (const suite of suites) for (const key of suite.systems) if (!systems.includes(key)) systems.push(key);
  systems.sort((a, b) => (b === "tddlog") - (a === "tddlog"));
  const instances = suites.flatMap((suite) => suite.instances.map((row) => {
    const out = [row[0], ...systems.map(() => null)];
    suite.systems.forEach((key, i) => {
      const v = row[i + 1];
      out[systems.indexOf(key) + 1] = answered(v) && v > suite.limit * 1.001 ? "timeout" : v;
    });
    return out;
  }));
  const sources = suites.flatMap((suite) => suite.instances.map(() => suite.key));
  return { key: "all", title, about, systems, instances, sources, limit: Math.max(...suites.map((suite) => suite.limit)) };
}

// One sentence for each system that was not given some of the instances.
function cannot(suite, names) {
  const list = element("ul", { class: "cannot" });
  const roots = [...new Set(suite.systems.map(stem))];
  for (const r of roots) {
    const columns = suite.systems.flatMap((key, i) => (stem(key) === r ? [i + 1] : []));
    const missed = suite.instances.flatMap((row, j) => (columns.every((c) => row[c] === null) ? [suite.sources[j]] : []));
    if (missed.length === 0) continue;
    const first = suite.systems.find((key) => stem(key) === r);
    const name = names[r] ?? names[first].split(",")[0];
    const all = count(suite.instances.length);
    const own = READS_NETWORKS.has(r) ? missed.filter((s) => NETWORK_SUITES.has(s)).length : 0;
    if (missed.length > own) list.append(element("li", {}, `${name} was not given ${count(missed.length - own)} of the ${all} problems: ${CANNOT[r] ?? "it was not run on them"}.`));
    if (own) list.append(element("li", {}, `${name} was not run on ${count(own)} of the ${all} problems, networks it reads, in this release.`));
  }
  return list;
}

// A plot of a suite with its legend, drawn again when its box changes width.
function plotted(suite, names) {
  const figure = element("figure", { class: "plot" });
  const box = element("div", { class: "box" });
  figure.append(box);
  const redraw = () => {
    const series = cactus(box, suite, names);
    const old = figure.querySelector(".legend");
    const fresh = legend(series, names, suite.instances.length, figure);
    if (old) old.replaceWith(fresh);
    else figure.prepend(fresh);
  };
  return { figure, redraw };
}

const EVERYTHING = "Every system on every instance: an instance a system did not answer counts as unanswered, whether it timed out, failed, or the system cannot express it or was not given it. Each line is one system, and the legend says how many of the instances it answers. The time axis starts at 0.1 s.";

// ------------------------------------------------------------- the table

function table(suite, names, pages) {
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
  cols.forEach((key, i) => head.append(element("th", { scope: "col", class: "num", "data-col": String(i + 1) }, names[key])));
  const body = element("tbody");
  const seen = new Set();
  const rowOf = (row) => {
    const best = fastest(row);
    const page = pages?.[suite.key]?.[row[0]];
    const tr = element("tr", {}, element("th", { scope: "row" }, page ? element("a", { href: `benchmarks/${page}` }, row[0]) : row[0]));
    if (quick(row)) tr.className = "quick";
    cols.forEach((key, i) => {
      const v = row[i + 1];
      let td;
      if (answered(v)) td = element("td", { class: `num${v === best && !dashed(key) ? " best" : ""}` }, seconds(v));
      else if (v === null) td = element("td", { class: "num none", title: "not run" }, "");
      else {
        seen.add(v);
        const [text, meaning] = FAILURES[v] ?? [() => v, v];
        td = element("td", { class: "num fail", title: meaning }, text(suite.limit));
      }
      tr.append(td);
    });
    return tr;
  };
  let rows = suite.instances.map((row) => [row, rowOf(row)]);
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
      ` Hide the ${count(nquick)} instances every system that ran finishes in under 0.1 s`));
  }
  parts.push(element("div", { class: "tscroll" }, grid));
  const notes = [`Seconds; the fastest answer of an instance in bold. A header sorts by its column.`];
  if (pages?.[suite.key]) notes.push("An instance's name opens its page: its programs in every language and every run.");
  for (const status of Object.keys(FAILURES)) {
    if (!seen.has(status)) continue;
    notes.push(`${FAILURES[status][0](suite.limit)}: ${FAILURES[status][1]}.`);
  }
  if (suite.instances.some((row) => row.slice(1).includes(null))) notes.push("An empty cell: the system did not run on the instance.");
  parts.push(element("p", { class: "note" }, notes.join(" ")));
  return element("details", { class: "instances" },
    element("summary", {}, `Every instance's times (${count(suite.instances.length)})`), ...parts);
}

// ------------------------------------------------------------- a family

function cards(list, choose) {
  const grid = element("div", { class: "findings" });
  for (const card of list) {
    const article = element("article", { class: "finding" }, element("h3", {}, card.title));
    for (const para of card.paras) article.append(prose("p", para));
    if (card.suite) {
      const go = element("button", { type: "button", class: "plain" }, "Show its plot");
      go.addEventListener("click", () => choose(card.suite, true));
      if (choose.has(card.suite)) article.append(go);
    }
    grid.append(article);
  }
  return grid;
}

function family(f, names, pages) {
  const section = element("section", { class: "card family", id: f.key });
  section.append(element("h2", {}, f.title), prose("p", f.intro, { class: "intro" }));
  const panel = element("div", { class: "suite" });
  const tabs = element("div", { class: "tabs", role: "group", "aria-label": `${f.title}: suites` });
  const everything = merged("All problems",
    `The instances of all ${f.suites.length} suites of this family. ${EVERYTHING}`, f.suites);
  const byKey = new Map([everything, ...f.suites].map((s) => [s.key, s]));
  let current = null;
  let redraw = () => {};
  const choose = (key, scroll = false) => {
    const suite = byKey.get(key);
    if (suite === undefined) return;
    current = key;
    for (const b of tabs.children) b.setAttribute("aria-pressed", String(b.dataset.suite === key));
    const { figure, redraw: draw } = plotted(suite, names);
    // The suite's findings beside its plot.
    const aside = element("aside", { class: "finding" });
    for (const card of f.cards.filter((c) => c.suite === key)) {
      for (const para of card.paras) aside.append(prose("p", para));
    }
    // The tab of every instance has no table (each suite's has its own) and
    // says which systems were not given some of the problems.
    const rest = key === "all" ? [cannot(suite, names)] : [table(suite, names, pages)];
    panel.replaceChildren(element("h3", {}, suite.title), prose("p", suite.about, { class: "about" }),
      element("div", { class: "view" }, ...(key === "all" ? [figure] : [figure, aside])), ...rest);
    redraw = draw;
    redraw();
    if (scroll) tabs.scrollIntoView({ behavior: "smooth", block: "start" });
  };
  choose.has = (key) => byKey.has(key);
  for (const suite of [everything, ...f.suites]) {
    const b = element("button", { type: "button", "data-suite": suite.key, "aria-pressed": "false" },
      `${suite.title} (${count(suite.instances.length)})`);
    b.addEventListener("click", () => choose(suite.key));
    tabs.append(b);
  }
  section.append(tabs, panel);
  if (f.findings && f.findings.length) {
    section.append(element("h3", { class: "more" }, "Findings across the suites"), cards(f.findings, choose));
  }
  const dl = element("dl");
  for (const [term, text] of f.setup) dl.append(element("dt", {}, term), prose("dd", text));
  section.append(element("details", { class: "setup" }, element("summary", {}, "How the runs were made"), dl,
    element("p", { class: "note" }, "Every table and its details: ",
      element("a", { href: f.docs }, "docs/benchmarks.md"), ".")));
  choose("all");
  let last = 0;
  new ResizeObserver(() => {
    const w = panel.clientWidth;
    if (w !== last && current !== null) {
      last = w;
      redraw();
    }
  }).observe(panel);
  return section;
}

// One plot over every instance of the run, every system, above the families.
function overall(families, names) {
  const suites = families.flatMap((f) => f.suites);
  const total = suites.reduce((n, s) => n + s.instances.length, 0);
  const suite = merged("Every problem of the release run",
    `The ${count(total)} instances of all ${families.length} families. ${EVERYTHING}`, suites);
  const { figure, redraw } = plotted(suite, names);
  const section = element("section", { class: "card family", id: "overall" },
    element("h2", {}, suite.title), prose("p", suite.about, { class: "about" }),
    element("div", { class: "view" }, figure), cannot(suite, names));
  redraw();
  let last = 0;
  new ResizeObserver(() => {
    const w = section.clientWidth;
    if (w !== last) {
      last = w;
      redraw();
    }
  }).observe(section);
  return section;
}

async function load(file) {
  const response = await fetch(file + stamp);
  if (!response.ok) throw new Error(`${file}: ${response.status}`);
  return response.json();
}

document.addEventListener("DOMContentLoaded", async () => {
  const root = document.getElementById("families");
  const toc = document.getElementById("toc");
  try {
    const parts = await Promise.all(["suites.json", "ppl.json"].map((file) => load(file).catch((failure) => {
      if (file === "ppl.json") return null;     // the page works with the networks and programs alone
      throw failure;
    })));
    // The benchmark pages (benchmarks/, from the baselines branch), where
    // the site has them.
    const pages = await load("benchmarks/index.json").catch(() => null);
    const families = parts.filter(Boolean).flatMap((p) => p.families);
    const names = Object.assign({}, ...parts.filter(Boolean).map((p) => p.names));
    const instances = families.reduce((n, f) => n + f.suites.reduce((m, s) => m + s.instances.length, 0), 0);
    const suites = families.reduce((n, f) => n + f.suites.length, 0);
    const baselines = new Set(families.flatMap((f) => f.suites.flatMap((s) => s.systems)).map(base));
    baselines.delete("tddlog");
    document.getElementById("version").textContent = parts[0].version;
    document.getElementById("stats").replaceChildren(
      ...[[count(instances), "instances"], [String(suites), "suites"], [String(baselines.size), "other systems"]]
        .map(([b, what]) => element("span", { class: "stat" }, element("b", {}, b), what)));
    root.append(overall(families, names));
    toc.append(element("a", { href: "#overall" }, "Every problem"));
    for (const f of families) {
      root.append(family(f, names, pages));
      toc.append(element("a", { href: `#${f.key}` }, f.title));
    }
  } catch (failure) {
    root.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
