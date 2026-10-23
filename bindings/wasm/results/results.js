"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl and
// Roulette), both written by the generator of README.md's results and
// the baselines branch's docs/benchmarks.md. A family of suites is its
// findings in prose, plots of the instances each system answers within a
// time limit (see cactus), the table of every instance's times, and how the
// runs were made. A family's first plots are its kinds of problems (the
// data's `kinds`, the README's figures): each kind's systems over all of its
// instances, with each system's answers and total time, which the results
// pipeline writes; the page does not choose a kind's instances or systems,
// nor compute its totals.

// The stamp of the page's own reference (results.js?v=<sha>) goes on to the
// data, as the playground's does to its files.
const stamp = document.currentScript === null ? "" : new URL(document.currentScript.src).search;

// The time axis starts at 0.2 s: runs under 0.2 s are not told apart.
const FLOOR = 0.2;

// How a failure reads in a table cell, and what it means under the table.
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

// A total time as the README's legends write it: 3.9 s, 108 s, 21 min, 1.2 h.
function duration(t) {
  if (t >= 36000) return `${(t / 3600).toFixed(0)} h`;
  if (t >= 3600) return `${(t / 3600).toFixed(1)} h`;
  if (t >= 60) return `${(t / 60).toFixed(0)} min`;
  return t >= 10 ? `${t.toFixed(0)} s` : `${t.toFixed(1)} s`;
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
  const redraw = () => {
    const series = cactus(box, suite, names);
    const old = figure.querySelector(".legend");
    const fresh = legend(series, names, suite.instances.length, figure, suite.total);
    if (old) old.replaceWith(fresh);
    else box.before(fresh);
  };
  return { figure, redraw };
}

// ------------------------------------------------------------- a kind

// A kind of the data (its suites) as one suite: the kind's systems, tddlog
// first and then by the instances they answer, as the README's legends list
// them; a system without a record of an instance null; the limit the
// longest of its suites'; each system's total time, the data's.
function kinded(kind, byKey) {
  const suites = kind.suites.map((key) => {
    const suite = byKey.get(key);
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

// One sentence for each system of a kind that was not given some of its
// instances (docs/baselines.md on the baselines branch says why).
function cannot(suite, names) {
  const list = element("ul", { class: "cannot" });
  suite.systems.forEach((key, i) => {
    const n = suite.instances.filter((row) => row[i + 1] === null).length;
    if (n) list.append(element("li", {}, `${names[key]} was not run on ${count(n)} of the ${count(suite.instances.length)} ${suite.noun} in this release.`));
  });
  return list;
}

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
  if (pages?.[suite.key]) notes.push("An instance's name opens its page: its programs in every language and every run.");
  for (const status of Object.keys(FAILURES)) {
    if (!seen.has(status)) continue;
    notes.push(`${FAILURES[status][0](suite.limit)}: ${FAILURES[status][1]}.`);
  }
  if (suite.instances.some((row) => row.slice(1).includes(null))) notes.push("An empty cell: the system was not given the instance.");
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

// A family's section: a tab for each of its kinds of problems (the data's
// kinds whose suites are the family's), then a tab for each suite, its
// findings and its setup; the first kind's tab is shown first.
function family(f, names, pages, kinds) {
  const section = element("section", { class: "card family", id: f.key });
  section.append(element("h2", {}, f.title), prose("p", f.intro, { class: "intro" }));
  const panel = element("div", { class: "suite" });
  const tabs = element("div", { class: "tabs", role: "group", "aria-label": `${f.title}: kinds and suites` });
  const suites = new Map(f.suites.map((s) => [s.key, s]));
  // A kind's tab is keyed apart from the suites' (the kind ppl, the suite ppl).
  const byKey = new Map([...kinds.map((k) => [`kind:${k.key}`, kinded(k, suites)]), ...f.suites.map((s) => [s.key, s])]);
  const buttons = new Map();
  let current = null;
  let redraw = () => {};
  const choose = (key, scroll = false) => {
    const suite = byKey.get(key);
    if (suite === undefined) return;
    current = key;
    for (const [k, b] of buttons) b.setAttribute("aria-pressed", String(k === key));
    const whole = plotted(suite, names);
    // A suite's findings beside its plot and its table under it; under a
    // kind's, which systems were not given some of its instances.
    const kind = key.startsWith("kind:");
    const aside = element("aside", { class: "finding" });
    for (const card of f.cards.filter((c) => c.suite === key)) {
      for (const para of card.paras) aside.append(prose("p", para));
    }
    panel.replaceChildren(element("h3", {}, suite.title), prose("p", suite.about, { class: "about" }),
      element("div", { class: "view" }, ...(kind ? [whole.figure] : [whole.figure, aside])),
      kind ? cannot(suite, names) : table(suite, names, pages));
    redraw = whole.redraw;
    redraw();
    if (scroll) tabs.scrollIntoView({ behavior: "smooth", block: "start" });
  };
  choose.has = (key) => byKey.has(key);
  for (const [key, suite] of byKey) {
    const b = element("button", { type: "button", [key.startsWith("kind:") ? "data-kind" : "data-suite"]: key.replace(/^kind:/, ""),
      "aria-pressed": "false" }, `${suite.title} (${count(suite.instances.length)})`);
    b.addEventListener("click", () => choose(key));
    buttons.set(key, b);
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
  choose(byKey.keys().next().value);
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
    // the site has them, and a link to their index under the contents.
    const pages = await load("benchmarks/index.json").catch(() => null);
    if (pages !== null) {
      toc.after(element("p", { class: "note", id: "pages" }, "Each instance in a suite's table links to its own page: ",
        "its programs in every language a system read, and every run. ",
        element("a", { href: "benchmarks/" }, "All the benchmark pages"), "."));
    }
    const families = parts.filter(Boolean).flatMap((p) => p.families);
    const names = Object.assign({}, ...parts.filter(Boolean).map((p) => p.names));
    const kinds = parts.filter(Boolean).flatMap((p) => p.kinds).sort((a, b) => a.order - b.order);
    const instances = families.reduce((n, f) => n + f.suites.reduce((m, s) => m + s.instances.length, 0), 0);
    const suites = families.reduce((n, f) => n + f.suites.length, 0);
    const baselines = new Set(families.flatMap((f) => f.suites.flatMap((s) => s.systems)).map(base));
    baselines.delete("tddlog");
    document.getElementById("version").textContent = parts[0].version;
    document.getElementById("stats").replaceChildren(
      ...[[count(instances), "instances"], [String(suites), "suites"], [String(baselines.size), "other systems"]]
        .map(([b, what]) => element("span", { class: "stat" }, element("b", {}, b), what)));
    // Each kind goes in the family that holds its suites.
    const home = (k) => families.find((f) => k.suites.every((key) => f.suites.some((s) => s.key === key)));
    for (const k of kinds) {
      if (home(k) === undefined) throw new Error(`the kind ${k.key} has no family of its suites`);
    }
    for (const f of families) {
      root.append(family(f, names, pages, kinds.filter((k) => home(k) === f)));
      toc.append(element("a", { href: `#${f.key}` }, f.title));
    }
  } catch (failure) {
    root.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
