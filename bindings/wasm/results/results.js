"use strict";

// The results page: the release run's suites from suites.json (the networks
// and the logic programs) and ppl.json (the programs of Dice, Alea.jl and
// Roulette), both written by the generator of README.md's results and
// the baselines branch's docs/benchmarks.md. A family of suites is its
// findings in prose, plots of the instances each system answers within a
// time limit (see plots), the table of every instance's times, and how the
// runs were made. The plots over several suites, and their panels, are the
// data's kinds (`kinds`), which the results pipeline writes; the page does
// not split instances itself.

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

const answered = (value) => typeof value === "number";
// A process time (Alea.jl's and Roulette's with their start-up) is drawn
// dashed, in its system's colour.
const base = (key) => key.replace(/-process$/, "");
const dashed = (key) => key.endsWith("-process");

// ------------------------------------------------------------- the plot

// The instances each system answers within a time limit, the limit on a log
// scale from 0.2 s to the suite's; a line further up and to the left is
// better. Drawn at the width of its box, again when the box changes width;
// a small plot is shorter and names its lines in its legend only.
function cactus(box, suite, names, small = false) {
  const width = Math.max(small ? 240 : 280, Math.floor(box.clientWidth));
  const narrow = small || width < 560;
  const height = small ? 240 : narrow ? 300 : Math.max(380, suite.systems.length * 14 + 22 + 46);
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
    small ? "time limit, seconds" : "time limit per instance, seconds (log scale)"));
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
// in a large plot how many of them are its own, natural to it and attempted
// (classes: {system: [own, natural, attempted]}); a name under the pointer
// brings its line forward.
function legend(series, names, n, figure, classes) {
  const list = element("ul", { class: "legend" });
  for (const { key, times } of series) {
    const c = classes?.[key];
    const counts = c === undefined ? `answers ${count(times.length)} of ${count(n)}`
      : `answers ${count(times.length)} of ${count(n)}; own ${count(c[0])}, natural ${count(c[1])}, attempted ${count(c[2])}`;
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

// A plot with its legend, drawn again when its box changes width; a small
// one (a panel) under its title. The figure names the systems it draws and
// the number of its instances, and a panel the cluster of systems it is for.
// A large plot's legend says how many instances are each system's own,
// natural to it and attempted.
function plotted(suite, names, small = false) {
  const figure = element("figure", { class: small ? "plot part" : "plot", "data-systems": suite.systems.join(" "),
    "data-instances": String(suite.instances.length) });
  if (small) {
    figure.dataset.members = suite.members.join(" ");
    figure.append(element("figcaption", {}, suite.title));
  }
  const box = element("div", { class: "box" });
  figure.append(box);
  const redraw = () => {
    const series = cactus(box, suite, names, small);
    const old = figure.querySelector(".legend");
    const fresh = legend(series, names, suite.instances.length, figure, small ? undefined : suite.counts);
    if (old) old.replaceWith(fresh);
    else box.before(fresh);
  };
  return { figure, redraw };
}

// A panel of a plot over a kind's suites (kinded): the plot cut to the
// panel's instances ({suite: [name, ...]}) and systems, its limit the
// longest of its instances'.
function panelOf(scope, p) {
  const key = (suite, name) => `${suite}/${name}`;
  const wanted = new Set(Object.entries(p.instances).flatMap(([suite, list]) => list.map((name) => key(suite, name))));
  const rows = scope.instances.flatMap((row, j) => (wanted.has(key(scope.sources[j], row[0])) ? [j] : []));
  if (rows.length !== wanted.size) throw new Error(`${p.title}: ${wanted.size - rows.length} instances not in its kind`);
  const columns = p.systems.map((system) => scope.systems.indexOf(system) + 1);
  return {
    key: scope.key, title: p.title, systems: p.systems, members: p.members,
    instances: rows.map((j) => [scope.instances[j][0], ...columns.map((c) => scope.instances[j][c])]),
    limit: Math.max(...rows.map((j) => scope.limits[j])),
  };
}

// A plot's figures: the plot of every system over all of its instances, the
// panels of a kind under a heading, and a function that draws them all.
function plots(suite, names) {
  const whole = plotted(suite, names);
  const parts = (suite.panels ?? []).map((p) => plotted(panelOf(suite, p), names, true));
  const grid = parts.length === 0 ? [] : [element("h4", { class: "more" }, "The instances natural to each cluster of systems"),
    element("div", { class: "parts" }, ...parts.map((p) => p.figure))];
  const redraw = () => {
    for (const p of [whole, ...parts]) p.redraw();
  };
  return { whole: whole.figure, parts: grid, redraw };
}

// ------------------------------------------------------------- every problem

// Why a system is not given some of the problems, as the baselines'
// coverage states it, by system (a system's variants, as alea-process or
// storm-sparse, share their stem's): a reason for every suite (`all`) or
// for one suite, by its key. A problem without a reason was not run in this
// release: every system is given every program it can read, through a
// translation where it reads no program of the suite's own (a network or
// a Dice program of a program's ground program, a Markov chain of a Dice
// program), and what the release's runs did not reach is left out.
const DICE_OWN = "the standard suite's dice family is the dice suite's programs, which it is given there";
const CANNOT = {
  dice: { std: DICE_OWN },
  nodice: { std: DICE_OWN },
  psi: { std: DICE_OWN },
  geni: { std: DICE_OWN },
  genfer: { std: DICE_OWN },
  sppl: { std: DICE_OWN },
  alea: { std: DICE_OWN },
  roulette: { std: DICE_OWN },
  provsql: { all: "its queries are non-recursive SQL over tuple-independent tables, and the translation has no counterpart for a recursive rule, a probabilistic rule, an annotated disjunction or a Dice program" },
};
const why = (system, suite) => CANNOT[system]?.[suite] ?? CANNOT[system]?.all ?? null;
const stem = (key) => key.split("-")[0];
// A system's name without its variant's: Alea.jl for alea-process.
const named = (key, names) => names[stem(key)] ?? (names[key] ?? key).split(",")[0];

// A kind of the data (its suites) as one suite: the kind's systems (tddlog
// first), a system without a record of an instance null, a time past its
// own suite's limit a timeout, each instance's limit its suite's; the
// kind's panels and how many instances are each system's own, natural to it
// and attempted.
function kinded(kind, byKey, title, about) {
  const suites = kind.suites.map((key) => byKey.get(key)).filter(Boolean);
  const systems = kind.systems;
  const instances = suites.flatMap((suite) => suite.instances.map((row) => {
    const out = [row[0], ...systems.map(() => null)];
    suite.systems.forEach((key, i) => {
      const v = row[i + 1];
      out[systems.indexOf(key) + 1] = answered(v) && v > suite.limit * 1.001 ? "timeout" : v;
    });
    return out;
  }));
  const sources = suites.flatMap((suite) => suite.instances.map(() => suite.key));
  const limits = suites.flatMap((suite) => suite.instances.map(() => suite.limit));
  return { key: "all", title, about, systems, instances, sources, limits,
    limit: Math.max(...limits), counts: kind.counts, panels: kind.panels };
}

// One sentence for each system and reason for which it was not given some of
// the instances.
function cannot(suite, names) {
  const list = element("ul", { class: "cannot" });
  const roots = [...new Set(suite.systems.map(stem))];
  for (const r of roots) {
    const columns = suite.systems.flatMap((key, i) => (stem(key) === r ? [i + 1] : []));
    const reasons = new Map();
    suite.instances.forEach((row, j) => {
      if (!columns.every((c) => row[c] === null)) return;
      const w = why(r, suite.sources[j]);
      reasons.set(w, (reasons.get(w) ?? 0) + 1);
    });
    const name = named(r, names);
    const all = count(suite.instances.length);
    for (const [w, n] of reasons) {
      list.append(element("li", {}, w === null
        ? `${name} was not run on ${count(n)} of the ${all} problems in this release.`
        : `${name} was not given ${count(n)} of the ${all} problems: ${w}.`));
    }
  }
  return list;
}

// What each system is compared on: the data's classification (the rows of
// the results pipeline's table that classify an instance), by class, each
// row its systems, its suites, its instances (a pattern their names start
// with, or all of them) and why.
const CLASSES = [["own", "Own"], ["natural", "Natural"], ["attempted", "Attempted only"]];
function compared(rows, titles) {
  const list = element("ul", { class: "cannot" });
  for (const [cls, heading] of CLASSES) {
    for (const r of rows.filter((x) => x.class === cls)) {
      const item = element("li", {}, element("b", {}, heading), `: ${r.systems.join(", ")} on ${r.suites.map((s) => titles.get(s) ?? s).join(", ")}`);
      if (r.instances) item.append(", the instances whose names match ", element("code", {}, `^${r.instances}`));
      item.append(`: ${r.reason}.`);
      list.append(item);
    }
  }
  return element("details", { class: "setup" }, element("summary", {}, "What each system is compared on"),
    element("p", { class: "note" }, "An instance is a system's own if it comes from the system's own papers, benchmarks or distribution; natural if the program is written as the system's users write programs, in a translation that stays within a constant factor of the original's size and does none of the system's work for it (no grounding, no unrolling of recursion, no bit-blasting), its own included; attempted only if the system is given it through a translation that does. The speed comparisons are over each system's natural instances; the first row that matches an instance classifies it."),
    list);
}

// What the plots of a kind draw.
function drawn(kind) {
  const whole = "The large plot draws every system over all of these instances, so that its lines count the same instances: an instance a system does not attempt (it cannot express the program, or a translator refuses it) counts as unanswered, as does one where it ran out of time or memory, failed, or was skipped after a smaller instance of its family failed. It shows coverage, not speed. Its legend gives how many instances each system answers, and how many are its own (from its own papers, benchmarks or distribution), natural to it (written as its users write programs, its own included) and attempted (given to it at all, through a translation that grounds, unrolls or bit-blasts the program where it is not natural).";
  if (kind.panels.length === 0) return whole;
  return `${whole} Under it, a small plot for each cluster of systems whose natural instances are much the same (systems linked by pairs whose sets of natural instances have a Jaccard index of ${kind.jaccard} or more), over the instances natural to every member, with tddlog and every system those instances are all natural to, where those are ${kind.least} or more.`;
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
  // An instance a system only attempts (its class letter `a`: a translation
  // that grounds, unrolls or bit-blasts the program) is set apart.
  let translated = false;
  const rowOf = (row, j) => {
    const best = fastest(row);
    const page = pages?.[suite.key]?.[row[0]];
    const tr = element("tr", {}, element("th", { scope: "row" }, page ? element("a", { href: `benchmarks/${page}` }, row[0]) : row[0]));
    if (quick(row)) tr.className = "quick";
    cols.forEach((key, i) => {
      const v = row[i + 1];
      const only = suite.classes?.[key]?.[j] === "a";
      translated ||= only && v !== null;
      const mark = only ? " attempted" : "";
      const why = only ? "; attempted only, through a translation" : "";
      let td;
      if (answered(v)) td = element("td", { class: `num${v === best && !dashed(key) ? " best" : ""}${mark}`, ...(only ? { title: why.slice(2) } : {}) }, seconds(v));
      else if (v === null) td = element("td", { class: "num none", title: "not given" }, "");
      else {
        seen.add(v);
        const [text, meaning] = FAILURES[v] ?? [() => v, v];
        td = element("td", { class: `num fail${mark}`, title: meaning + why }, text(suite.limit));
      }
      tr.append(td);
    });
    return tr;
  };
  let rows = suite.instances.map((row, j) => [row, rowOf(row, j)]);
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
  if (translated) notes.push("In italics: an instance the system only attempts, through a translation that grounds, unrolls or bit-blasts the program, not one written as its users write programs; such times count for coverage, not in the speed comparisons.");
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

// A family's section: its tab of all problems (the data's kind of the same
// key, with its panels), a tab for each suite, its findings and its setup.
function family(f, names, pages, kinds) {
  const section = element("section", { class: "card family", id: f.key });
  section.append(element("h2", {}, f.title), prose("p", f.intro, { class: "intro" }));
  const panel = element("div", { class: "suite" });
  const tabs = element("div", { class: "tabs", role: "group", "aria-label": `${f.title}: suites` });
  const kind = kinds.get(f.key);
  if (kind === undefined) throw new Error(`the family ${f.key} has no kind in the data`);
  const everything = kinded(kind, new Map(f.suites.map((s) => [s.key, s])), "All problems",
    `The instances of all ${f.suites.length} suites of this family. ${drawn(kind)}`);
  const byKey = new Map([everything, ...f.suites].map((s) => [s.key, s]));
  let current = null;
  let redraw = () => {};
  const choose = (key, scroll = false) => {
    const suite = byKey.get(key);
    if (suite === undefined) return;
    current = key;
    for (const b of tabs.children) b.setAttribute("aria-pressed", String(b.dataset.suite === key));
    const { whole, parts, redraw: draw } = plots(suite, names);
    // The suite's findings beside its plot.
    const aside = element("aside", { class: "finding" });
    for (const card of f.cards.filter((c) => c.suite === key)) {
      for (const para of card.paras) aside.append(prose("p", para));
    }
    // The tab of every instance has no table (each suite's has its own) and
    // says which systems were not given some of the problems.
    const rest = key === "all" ? [cannot(suite, names)] : [table(suite, names, pages)];
    panel.replaceChildren(element("h3", {}, suite.title), prose("p", suite.about, { class: "about" }),
      element("div", { class: "view" }, ...(key === "all" ? [whole] : [whole, aside])), ...parts, ...rest);
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

// The plot of every system over every instance of the run (the data's kind
// `all`), above the families, and what each system is compared on.
function overall(families, names, kinds, classification) {
  const kind = kinds.get("all");
  if (kind === undefined) throw new Error("the data has no kind all");
  const suites = new Map(families.flatMap((f) => f.suites).map((s) => [s.key, s]));
  const total = kind.suites.reduce((n, key) => n + (suites.get(key)?.instances.length ?? 0), 0);
  const suite = kinded(kind, suites, "Every problem of the release run",
    `The ${count(total)} instances of all ${families.length} families. ${drawn(kind)} The families below have a plot for each cluster of systems over the instances natural to them. The list under the plot says why a system was not given some of the problems, and the table under it what each system is compared on.`);
  const { whole, parts, redraw } = plots(suite, names);
  const titles = new Map(families.flatMap((f) => f.suites).map((s) => [s.key, s.title]));
  const section = element("section", { class: "card family", id: "overall" },
    element("h2", {}, suite.title), prose("p", suite.about, { class: "about" }),
    element("div", { class: "view" }, whole), ...parts, cannot(suite, names), compared(classification, titles));
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
    const kinds = new Map(parts.filter(Boolean).flatMap((p) => p.kinds).map((k) => [k.key, k]));
    const instances = families.reduce((n, f) => n + f.suites.reduce((m, s) => m + s.instances.length, 0), 0);
    const suites = families.reduce((n, f) => n + f.suites.length, 0);
    const baselines = new Set(families.flatMap((f) => f.suites.flatMap((s) => s.systems)).map(base));
    baselines.delete("tddlog");
    document.getElementById("version").textContent = parts[0].version;
    document.getElementById("stats").replaceChildren(
      ...[[count(instances), "instances"], [String(suites), "suites"], [String(baselines.size), "other systems"]]
        .map(([b, what]) => element("span", { class: "stat" }, element("b", {}, b), what)));
    // A row of the classification that classifies instances of both files
    // is in both.
    const rows = new Map(parts.filter(Boolean).flatMap((p) => p.classification ?? []).map((r) => [JSON.stringify(r), r]));
    root.append(overall(families, names, kinds, [...rows.values()]));
    toc.append(element("a", { href: "#overall" }, "Every problem"));
    for (const f of families) {
      root.append(family(f, names, pages, kinds));
      toc.append(element("a", { href: `#${f.key}` }, f.title));
    }
  } catch (failure) {
    root.replaceChildren(element("p", { class: "failed" }, `The results did not load: ${failure.message}`));
  }
});
