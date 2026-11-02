"use strict";

// The site's front page (index.html): views of the results by tool and by
// type of program, head-to-head apart from translated, what the systems' own
// sources hold that tddlog does not run, and how a program is answered. It
// is the results page's app (results.js, loaded before this script, whose
// helpers and data it uses) with these views added through its `extend`:
//   #/                the home page: what tddlog is, the overview figure, the sections
//   #/tools           by tool: every system, its head-to-head and translated speed-ups and its coverage
//   #/tools/SYSTEM    tddlog against SYSTEM: head-to-head, translated (each direction and translator),
//                     coverage, and how it was run
//   #/types           by type of program: the six types and all programs
//   #/coverage        the programs each system's sources hold that tddlog does not run, and why
//   #/how             how a program is answered: the pipeline and four walkthroughs
//   #/how/WALK[/N]    a walkthrough, at its step N: a run's trace, a stage at a time
// A type of program is results.js's view (#/category/C), with its comparisons
// head-to-head and translated before the rest; a solver's full page, a suite,
// a family and an instance are results.js's views as they are.
//
// The data is views.json (release/results/rel/views.py): each system's kind
// of comparison on each instance of each type (head-to-head, or translated
// for it, for tddlog or for both), and the comparisons over each kind, which
// are panels.py's (rule 16), and the census of the programs tddlog does not
// run. A system is compared on each instance with tddlog or one of its
// pipelines (views.json's mine; Alea.jl+tddlog on Alea.jl's circuits): in a
// type, with the instance's subject (the type's with); on the system's own
// page, with its own pipeline where that ran (Roulette+tddlog for Roulette;
// the system's kinds and with). The plots draw that member's times. The page
// counts and draws; it computes no total or speed-up.

const BRANCH = "https://github.com/Tractables/tddlog/blob/";
const KIND_TEXT = {
  s: (name) => `${name} read a translation of the program tddlog read`,
  m: (name) => `tddlog read a translation of ${name}'s program`,
  b: () => "both read a translation of the original, which neither read",
};
const draft = { views: null };

// ------------------------------------------------------------- the data

// The rows of category c (catRows) that system x was given, each with the
// kind of their comparison (h, s, m or b) and `me`, the member of mine it is
// with: as the type compares them, or with own as x's page does.
function kindRows(c, x, own = false) {
  const v = own ? draft.views.systems[x] : draft.views.categories[c.key];
  const kinds = own ? v?.kinds?.[c.key] : v?.kinds?.[x];
  if (!kinds) return [];
  const ms = own ? v.with[c.key] : v.with;
  return catRows(c).map((r, i) => [r, kinds[i], ms[i]]).filter(([, kind]) => kind !== "-")
    .map(([r, kind, m]) => ({ ...r, kind, me: draft.views.mine[Number(m)] }));
}


// The six types (not all programs) a system was given.
const typesOf = (x) => data.types.filter((c) => c.systems.includes(x));

// A system's rows over the six types, as its page compares them, each with its category.
function systemRows(x) {
  return typesOf(x).flatMap((c) => kindRows(c, x, true).map((r) => ({ ...r, c })));
}

// The types a system natively handles (panels.py NATIVE_TO, as the data's
// categories name them).
const nativeTo = (x) => data.types.filter((c) => c.native.includes(x));

// An instance's address in the results app.
const instanceHref = (suite, name) => `#/${suite}/@${encodeURIComponent(name)}`;

// A script's source on the branch that holds it.
function scriptLink(name, path) {
  return path ? element("a", { href: `${BRANCH}baselines/${path}` }, element("code", {}, name)) : element("code", {}, name);
}

// ------------------------------------------------------------- the parts

// Numbers in cells of their own, each under its label ([label, value,
// title]), never run together in a sentence.
function cells(...items) {
  return element("div", { class: "cells" }, ...items.map(([label, value, title]) =>
    element("div", { class: "cell", title }, element("span", {}, label), element("b", {}, value))));
}

// A text that would take more than a line, folded under its summary.
const fold = (summary, ...text) => element("details", { class: "fold" }, element("summary", {}, summary), element("p", {}, ...text));

// tddlog's speed-up in a cell: "too few" where too few instances both answer.
const speedCell = (v) => (v?.speedup == null ? "too few" : speedup(v));

// A comparison in cells: the instances, the two systems' answers, their
// PAR-2 totals over the instances the system is given and either answers,
// and the speed-up of the members of mine it is with.
function versusCells(v, name, n) {
  if (!v) return note("No instances.");
  const me = subject(v.me), over = `over the ${plural(v.either, "instance")} ${name} is given and either answers, PAR-${data.measure.par}`;
  return cells(["instances", count(n ?? v.given)], [`${name} answers`, count(v.answers)], [`${me} answers`, count(v.tddlog_answers)],
    [`${name}'s total`, duration(v.total), over], [`${me}'s total`, duration(v.tddlog_total), over],
    [`${me}'s speed-up`, speedCell(v), speedup(v, true)]);
}

// The cactus plot of x and the members it is with over rows (kindRows' me,
// drawn as one line), beside the scatter plot of their times.
function pairPlot(rows, x, v, key) {
  const name = data.names[x], me = subject(v?.me);
  const both = { key: `${x}-${key}`, title: `${name} and ${me}`, noun: "instances", systems: ["tddlog", x],
    instances: rows.map((r) => [r.name, r.v[r.me], r.v[x]]), limit: Math.max(...rows.map((r) => r.limit)),
    faster: { [x]: v?.speedup == null ? "too few in common" : `${me} ${speedup(v, true)}` } };
  return element("div", { class: "pair" }, plotted(both, { ...data.names, tddlog: me }, { all: true }), scatter(rows, ["tddlog", x], x, false, (r) => r.me));
}

// The families rows are of, with how many each, each opening its family in
// its type of program.
function familyCounts(rows) {
  const out = new Map();
  for (const r of rows) {
    const at = data.place.get(`${r.suite}/${r.name}`);
    const key = at ? `${at.c.key}/${at.g.key}` : r.suite;
    if (!out.has(key)) out.set(key, { at, n: 0, a: 0 });
    out.get(key).n++;
  }
  const list = [...out.values()].sort((a, b) => b.n - a.n);
  return element("p", { class: "fams" }, "From ", ...list.flatMap(({ at, n }, i) => [i ? ", " : "",
    at ? link({ view: "category", category: at.c.key, only: at.g.key }, at.g.title) : "another suite", ` (${count(n)})`]), ".");
}

// A type's title within a sentence: "probabilistic logic programs", "Bayesian networks".
const lower = (title) => title.toLowerCase().replace(/\bbayesian\b/, "Bayesian").replace(/\bmarkov\b/, "Markov");

// A system's swatch and name, opening its page here.
const toolLink = (x) => element("a", { href: `#/tools/${encodeURIComponent(x)}`, class: "tool" }, swatch(x), data.names[x]);

// ------------------------------------------------------------- home

function draftHome() {
  const all = data.categories.find((c) => c.key === ALL);
  const others = Object.keys(draft.views.systems).length;
  const entries = [
    ["#/tools", "By tool", `tddlog against each of the ${count(others)} other systems: Dice, Alea.jl, ProbLog, PITA, Ace and the rest, head-to-head apart from translated.`],
    ["#/types", "By type of program", "The six types of program and all programs, each against the systems native to it, with every instance's programs and commands."],
    ["#/how", "How it works", "The pipeline from a program to its probabilities, and four runs traced step by step."],
  ];
  return [
    element("section", { class: "intro" },
      element("p", { class: "big" }, "tddlog computes exact probabilities for probabilistic logic programs, Bayesian and Markov networks, "
        + "and probabilistic functional programs, by grounding them to probabilistic Datalog and compiling them bottom-up into tree decision diagrams."),
      element("p", { class: "lede" }, `In its release run it answers ${count(all.answers.tddlog)} of ${count(all.instances)} instances of six types of program, `
        + `against ${count(others)} systems run on the same machines.`)),
    element("nav", { class: "entries", "aria-label": "Sections" }, ...entries.map(([href, title, text]) =>
      element("a", { href, class: "entry" }, element("b", {}, title), element("span", {}, text)))),
    element("div", { class: "fig-head" }, element("h2", {}, "Each type of program against the systems native to it"),
      element("span", { class: "dots" }, element("i", { class: "me" }), "tddlog", element("i", { class: "you" }), "a native system")),
    overview(),
    element("p", { class: "also" }, element("a", { href: "playground/" }, "The playground"), " runs tddlog in the browser; the ",
      element("a", { href: "#/coverage" }, "coverage"), " lists what each system's own sources hold that tddlog does not run, and why."),
  ];
}

// ------------------------------------------------------------- by tool

function toolsView() {
  const out = [crumbs(["Home", { view: "home" }], ["By tool"]), element("h2", {}, "By tool"),
    element("p", { class: "lede" }, "tddlog against each system, head-to-head where both read the same program and translated where one of them read a translation."),
    fold("How the speed-up is measured", measureText())];
  const grid = element("div", { class: "tool-grid" });
  for (const s of data.systems) {
    const v = draft.views.systems[s.key];
    if (isMine(s.key) || !v) continue;
    const row = (label, n, w) => element("tr", {}, element("th", { scope: "row" }, label),
      element("td", { class: "num" }, n ? count(n) : "–"), element("td", { class: "num", title: n ? speedup(w, true) : null }, n ? speedCell(w) : "–"));
    const native = nativeTo(s.key).map((c) => lower(c.title));
    grid.append(element("a", { href: `#/tools/${encodeURIComponent(s.key)}`, class: "tool-card" },
      element("span", { class: "tname" }, swatch(s.key), data.names[s.key]),
      element("span", { class: "muted small one", title: native.length ? `Native to ${and(native)}` : null },
        native.length ? `Native to ${and(native)}` : "Given programs through a translation"),
      cells(["answers", count(s.answers), `of the ${count(s.given)} instances it was given`],
        ["tddlog does not", count(v.beyond.length), `instances ${data.names[s.key]} answers that tddlog does not`]),
      element("table", { class: "mini" }, element("thead", {}, element("tr", {}, element("td"), element("th", { scope: "col" }, "instances"),
        element("th", { scope: "col", title: "tddlog's speed-up" }, "speed-up"))),
      element("tbody", {}, row("head-to-head", v.n.h, v.head), row("translated", v.n.s + v.n.m + v.n.b, v.translated)))));
  }
  out.push(grid);
  return out;
}

// tddlog against one system: head-to-head, translated, coverage, how it was run.
function toolView(route) {
  const x = route.system, s = data.systems.find((y) => y.key === x), v = draft.views.systems[x], name = data.names[x];
  const rows = systemRows(x);
  const site = s.link.replace(/^https?:\/\/(www\.)?/, "").replace(/\/$/, "");
  const out = [crumbs(["Home", { view: "home" }], ["By tool", { view: "tools" }], [name]),
    element("h2", { class: "solver" }, swatch(x), `tddlog against ${name}`),
    element("p", { class: "lede" }, `${s.about} `, element("a", { href: s.link }, site)),
    cells(["given", count(s.given), `instances of ${and(typesOf(x).map((c) => lower(c.title)))}`], ["answers", count(s.answers)],
      ["tddlog does not", count(v.beyond.length), `instances ${name} answers that tddlog does not`]),
    element("nav", { class: "panels-nav", "aria-label": "Panels" }, element("a", { href: "#head" }, "Head-to-head"), element("a", { href: "#translated" }, "Translated"),
      element("a", { href: "#coverage" }, "Coverage"), element("a", { href: "#run" }, "How it was run"))];
  // head-to-head
  const head = rows.filter((r) => r.kind === "h");
  const p1 = element("section", { class: "panel", id: "head" }, element("h3", {}, "Head-to-head"),
    note("Both read the same program: the benchmark's own, or the same network in another network format."));
  if (head.length) p1.append(versusCells(v.head, name, head.length), pairPlot(head, x, v.head, "h"), familyCounts(head));
  out.push(p1);
  // translated, by direction and translator
  const tr = rows.filter((r) => "smb".includes(r.kind));
  const p2 = element("section", { class: "panel", id: "translated" }, element("h3", {}, "Translated"),
    note("One of them, or both, read a translation; tddlog's own readers (of Dice programs, LPADs and networks) are front ends, not translations."));
  if (tr.length) {
    p2.append(versusCells(v.translated, name, tr.length));
    const ways = element("div", { class: "ways" });
    for (const w of v.ways) {
      const how = [...(w.reason ? [element("span", { class: "muted" }, `${name} read: `), w.reason, element("br")] : []),
        ...(w.mine ? [element("span", { class: "muted" }, "tddlog read: "), w.mine, element("br")] : []),
        ...(w.by.length ? ["Written by ", ...w.by.flatMap(([n, path], i) => [i ? ", " : "", scriptLink(n, path)])] : [])];
      const by = w.by.map(([n]) => n).join(", ");
      ways.append(element("div", { class: "way" }, element("p", { class: "way-head" }, element("b", {}, KIND_TEXT[w.kind](name))),
        by ? element("p", { class: "muted small one", title: `Written by ${by}` }, `Written by ${by}`) : null,
        cells(["instances", count(w.n)], ...(w.vs ? [[`${name} answers`, count(w.vs.answers)], ["tddlog answers", count(w.vs.tddlog_answers)],
          ["speed-up", speedCell(w.vs), speedup(w.vs, true)]] : [])),
        ...(how.length ? [fold("The programs and the translator", ...how)] : [])));
    }
    p2.append(ways, pairPlot(tr, x, v.translated, "t"), familyCounts(tr));
  }
  out.push(p2);
  // coverage: what it answers that tddlog does not, what tddlog answers it cannot be given, its sources' census
  const p3 = element("section", { class: "panel", id: "coverage" }, element("h3", {}, "Coverage"));
  const fails = new Map();
  for (const [cat, suite, inst, rec] of v.beyond) {
    const key = typeof rec === "string" ? rec : "other";
    if (!fails.has(key)) fails.set(key, []);
    fails.get(key).push([cat, suite, inst]);
  }
  p3.append(element("h4", {}, `What ${name} answers that tddlog does not: ${count(v.beyond.length)}`));
  if (v.beyond.length) {
    for (const [key, list] of fails) {
      const what = FAILURES[key]?.[1] ?? key;
      p3.append(element("details", { class: "beyond" }, element("summary", {}, `${count(list.length)} where tddlog's run: ${what}`),
        element("ul", { class: "insts" }, ...list.map(([, suite, inst]) => element("li", {}, element("a", { href: instanceHref(suite, inst) }, inst))))));
    }
  }
  const unexp = Object.entries(v.unexpressed).filter(([, [a, b]]) => a + b > 0);
  p3.append(element("h4", {}, `What tddlog answers that ${name} cannot be given`));
  if (unexp.length) {
    p3.append(element("ul", {}, ...unexp.map(([cat, [refused, missing]]) => element("li", {},
      `${data.categories.find((c) => c.key === cat).title}: `,
      [refused ? `${count(refused)} it cannot express (its translator refuses what its language lacks)` : "",
        missing ? `${count(missing)} it was not given` : ""].filter(Boolean).join("; "), "."))));
  } else p3.append(note("None: it was given every instance tddlog answers."));
  const census = draft.views.census.sources.filter((src) => src.system === x);
  p3.append(element("h4", {}, `What ${name}'s own sources hold that tddlog does not run`));
  if (census.length) p3.append(...census.map(censusSource));
  else p3.append(note(`The census has no source of ${name}'s; its programs are the suites'.`));
  out.push(p3);
  // how it was run
  const dl = element("dl", {}, element("dt", {}, "Version"), prose("dd", s.version), element("dt", {}, "Settings"), prose("dd", s.settings));
  if (s.notgiven) dl.append(element("dt", {}, "Not given"), prose("dd", s.notgiven));
  if (s.cite) {
    const c = s.cite;
    dl.append(element("dt", {}, "Paper"), element("dd", {}, `${c.authors}. `,
      c.doi ? element("a", { href: `https://doi.org/${c.doi}` }, c.title) : c.title, `. ${c.venue}, ${c.year}.`));
  }
  dl.append(element("dt", {}, "Site"), element("dd", {}, element("a", { href: s.link }, s.link)),
    element("dt", {}, "Commands"), element("dd", {}, "Each instance's page shows the exact command each system's job ran, and the files it read."));
  out.push(element("section", { class: "panel", id: "run" }, element("h3", {}, "How it was run"), element("div", { class: "facts" }, dl),
    fold("How the speed-up is measured", measureText()), element("p", {}, link({ view: "system", system: x }, `${name}'s full page`),
      ": its plots in each type of program among the other systems, its own benchmarks, and how it was given each suite.")));
  return out;
}

// ------------------------------------------------------------- by type of program

// The comparisons of a type head-to-head and translated, before the rest of
// its view (results.js's categoryView): each system's instances of each
// kind, its answers, the totals and tddlog's speed-up, and the plots of
// tddlog against the system set (vs), else the first.
function typeSplit(c, route) {
  if (route.only !== undefined) {
    return [note("Head-to-head and translated are split over the whole type: ", link({ view: "category", category: c.key }, c.title), ".")];
  }
  const v = draft.views.categories[c.key];
  const xs = (c.key === ALL ? c.systems : c.native).filter((x) => x !== "tddlog" && v.kinds[x]);
  const n = (x, keep) => [...v.kinds[x]].filter((k) => keep.includes(k)).length;
  const out = [];
  for (const [id, title, keep, field, text] of [
    ["head", "Head-to-head", "h", "head", "both read the benchmark's program as its authors or its generator wrote it, or the same network in another network format"],
    ["translated", "Translated", "smb", "translated", "one of them, or both, read a translation (by: for the system, for tddlog, for both)"]]) {
    const have = xs.filter((x) => n(x, keep) > 0);
    const body = element("tbody");
    for (const x of have) {
      const w = v[field][x];
      const num = (text) => element("td", { class: "num" }, text);
      const dirs = keep === "h" ? [] : ["s", "m", "b"].map((k) => num(count(n(x, k))));
      body.append(element("tr", { class: x === route.vs ? "vs" : null }, element("th", { scope: "row" }, toolLink(x)),
        num(link({ view: "category", category: c.key, vs: x }, count(n(x, keep)))), ...dirs,
        num(w ? count(w.answers) : ""), num(w ? count(w.tddlog_answers) : ""), num(w ? duration(w.total) : ""), num(w ? duration(w.tddlog_total) : ""),
        num(w ? speedup(w) : "")));
    }
    const sec = element("section", { class: "panel split", id }, element("h3", {}, `${title}: ${have.length ? and(have.map((x) => data.names[x])) : "no system"}`),
      note(`Where ${text}.`));
    if (have.length) {
      const col = (text, title) => element("th", { scope: "col", class: "num", title }, text);
      const par = `PAR-${data.measure.par}, over the instances the system is given and either answers`;
      sec.append(scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
        element("th", { scope: "col" }, "system"), col("instances"),
        ...(keep === "h" ? [] : [col("for it", "a translation for the system"), col("for tddlog", "a translation for tddlog"),
          col("for both", "a translation of the original for both")]),
        col("it answers"), col("tddlog answers"), col("its total", par), col("tddlog's total", par), col("tddlog faster"))), body)), "whole"));
      const x = have.includes(route.vs) ? route.vs : have[0];
      const rows = kindRows(c, x).filter((r) => keep.includes(r.kind));
      sec.append(element("p", { class: "small" }, `The plots: tddlog against ${data.names[x]} on these ${plural(rows.length, "instance")}; a system's count of instances above sets them, its name opens its page.`),
        pairPlot(rows, x, v[field][x], id));
    }
    out.push(sec);
  }
  out.push(element("h3", { class: "every" }, `Every instance of the type, each system's answers over all of them`));
  return out;
}

function typesView() {
  const body = element("tbody");
  for (const c of data.categories) {
    const v = draft.views.categories[c.key];
    const xs = (c.key === ALL ? c.systems : c.native).filter((x) => x !== "tddlog" && v.kinds[x]);
    const kinds = (k) => xs.filter((x) => v.kinds[x].includes(k)).length;
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "category", category: c.key }, c.title)),
      element("td", { class: "num" }, count(c.instances)), element("td", { class: "num" }, count(answeredBy(catRows(c)))),
      element("td", {}, and((c.key === ALL ? [] : c.native).map((x) => data.names[x])) || "every system"),
      element("td", { class: "num" }, count(kinds("h"))), element("td", { class: "num" }, count(xs.filter((x) => /[smb]/.test(v.kinds[x])).length))));
  }
  return [crumbs(["Home", { view: "home" }], ["By type of program"]), element("h2", {}, "By type of program"),
    element("p", { class: "lede" }, "Each type against the systems native to it; all programs is their union, with every system."),
    fold("How the instances are counted", "Each instance counts in one type, the type of the model it is, whatever language its source was written in. "
      + "A type's comparisons are head-to-head first and translated second. A type opens its families, and a family its instances, "
      + "each with the programs every system read and the commands its job ran."),
    scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "type"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num", title: "by each instance's subject: tddlog, or Alea.jl+tddlog on Alea.jl's circuits" }, "tddlog answers"),
      element("th", { scope: "col" }, "native systems"),
      element("th", { scope: "col", class: "num", title: "the systems compared head-to-head on some instance" }, "systems head-to-head"),
      element("th", { scope: "col", class: "num", title: "the systems compared through a translation on some instance" }, "systems translated"))),
    body)), "whole"),
    element("div", { class: "fig-head" }, element("h3", {}, "Each type against the systems native to it"),
      element("span", { class: "dots" }, element("i", { class: "me" }), "tddlog", element("i", { class: "you" }), "a native system")),
    overview()];
}

// ------------------------------------------------------------- coverage

// A source of the census: what it holds, how many tddlog runs, and why it
// does not run the others.
function censusSource(src, tool = false) {
  const rows = src.items.map((it) => element("tr", {}, element("td", {}, element("span", { class: `cls cls-${it.class.replace(/ /g, "-")}` }, it.class)),
    element("td", { class: "num" }, typeof it.n === "number" ? count(it.n) : it.n), element("td", { class: "about" }, it.programs),
    element("td", { class: "about" }, it.why)));
  return element("div", { class: "census" },
    element("p", {}, ...(tool ? [toolLink(src.system), " · "] : []), element("b", {}, src.source), `: ${src.what}. `,
      element("span", { class: "muted" }, `${typeof src.programs === "number" ? `${count(src.programs)} programs` : src.programs}; tddlog runs `
        + `${typeof src.runs === "number" ? count(src.runs) : src.runs}.`)),
    ...(rows.length ? [scroller(stacked(element("table", { class: "board census-t" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "why"), element("th", { scope: "col", class: "num" }, "programs"), element("th", { scope: "col" }, "which"),
      element("th", { scope: "col" }, "what stops tddlog"))), element("tbody", {}, ...rows))), "whole")] : []));
}

function coverageView() {
  const cen = draft.views.census;
  const out = [crumbs(["Home", { view: "home" }], ["Coverage"]), element("h2", {}, "Coverage"),
    element("p", { class: "lede" }, cen.about),
    element("dl", { class: "classes" }, ...Object.entries(cen.classes).flatMap(([k, text]) => [element("dt", {},
      element("span", { class: `cls cls-${k.replace(/ /g, "-")}` }, k)), element("dd", {}, text)]))];
  // the run's side: each system, what it answers that tddlog does not, and what tddlog answers that it cannot be given
  const body = element("tbody");
  const sums = (v) => Object.values(v.unexpressed).reduce(([a, b], [c, d]) => [a + c, b + d], [0, 0]);
  const refusals = data.systems.some((s) => draft.views.systems[s.key] && sums(draft.views.systems[s.key])[0] > 0);
  for (const s of data.systems) {
    const v = draft.views.systems[s.key];
    if (!v) continue;
    const [refused, missing] = sums(v);
    body.append(element("tr", {}, element("th", { scope: "row" }, toolLink(s.key)),
      element("td", { class: "num" }, count(v.beyond.length)), ...(refusals ? [element("td", { class: "num" }, count(refused))] : []),
      element("td", { class: "num" }, count(missing))));
  }
  out.push(element("h3", {}, "In the release run"), note("Over the six types: the instances a system answers that tddlog does not "
    + "(out of time or memory, or refused), and those tddlog answers that the system cannot express (its translator refuses what its "
    + "language lacks) or was not given."),
  scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {}, element("th", { scope: "col" }, "system"),
    element("th", { scope: "col", class: "num" }, "it answers, tddlog does not"),
    ...(refusals ? [element("th", { scope: "col", class: "num" }, "tddlog answers, it cannot express")] : []),
    element("th", { scope: "col", class: "num" }, "tddlog answers, it was not given"))), body)), "whole"));
  // the census, by type of program
  out.push(element("h3", {}, "In each system's own sources, by type of program"));
  for (const c of data.types) {
    const srcs = cen.sources.filter((src) => src.type === c.key);
    if (!srcs.length) continue;
    out.push(element("h4", {}, c.title), ...srcs.map((src) => censusSource(src, true)));
  }
  return out;
}

// ------------------------------------------------------------- how it works

// The six stages of a run (docs/design.md, "How a program is answered"):
// what each does, and where the code is.
const STAGES = [
  ["Parse", "the text, or a network or a Dice program, read as a Program", "src/syntax"],
  ["Ground", "rules without variables over the atoms the queries and evidence depend on", "src/ground"],
  ["Rewrite", "linear recursion written without recursion; indicators", "src/ground"],
  ["Choose a layout", "a vtree over the facts, by auto's trials under doubling bounds", "src/vtree"],
  ["Compile", "a TDD per derived atom by Tp-compilation, component by component", "src/tp"],
  ["Evidence and queries", "weighted model counts of the queries against the evidence", "src/query"],
];

function pipeline(on) {
  const w = 1000, bw = 150, gap = (w - STAGES.length * bw) / (STAGES.length - 1), h = 150;
  const fig = svg("svg", { class: "pipeline", viewBox: `0 0 ${w} ${h}`, role: "img", "aria-label": "The pipeline: " + STAGES.map((s) => s[0]).join(", then ") });
  STAGES.forEach(([title, text, where], i) => {
    const x = i * (bw + gap);
    const g = svg("g", { class: i === on ? "stage on" : "stage" });
    g.append(svg("rect", { x, y: 8, width: bw, height: 132, rx: 8 }));
    const lines = (str, y, cls, per) => {
      let line = "", k = 0;
      for (const word of str.split(" ")) {
        if (`${line} ${word}`.trim().length > per) {
          g.append(svg("text", { x: x + 10, y: y + 15 * k++, class: cls }, line.trim()));
          line = word;
        } else line += ` ${word}`;
      }
      if (line.trim()) g.append(svg("text", { x: x + 10, y: y + 15 * k++, class: cls }, line.trim()));
      return k;
    };
    const k = lines(`${i + 1}. ${title}`, 30, "st", 16);
    lines(text, 36 + 15 * k, "sx", 20);
    g.append(svg("text", { x: x + 10, y: 128, class: "sw" }, where));
    fig.append(g);
    if (i < STAGES.length - 1) fig.append(svg("path", { d: `M${x + bw + 3} 74 h${gap - 6} m-6 -5 l6 5 l-6 5`, class: "arrow" }));
  });
  return element("figure", { class: "pipe" }, fig);
}

// The stage of the pipeline each step of a trace belongs to.
const STAGE_OF = { read: 0, parse: 0, ground: 1, rewrite: 2, layout: 3, compile: 4, evidence: 5, answers: 5 };

// The walkthroughs: a run of tddlog on each program, its trace
// (walk/KEY.json, written by the command shown, `tddlog --trace`) drawn by
// trace.js a step per stage, and a caption by hand over each step. Each
// fact a caption states is the trace's or docs/design.md's.
const WALKS = [
  {
    key: "reach", title: "Reachability in a graph with cycles", what: "Recursion, written without recursion before the compile; the fixpoint where a cycle stays.",
    file: "examples/cycle_graph.pl", example: "reachability", evidence: false,
    command: "tddlog --trace reach.json examples/cycle_graph.pl",
    lede: "The probability that node 1 reaches each other node of a graph with cycles, each of its 14 edges present with a probability of its own. "
      + "The recursion of path/2 is linear, and tddlog writes it without recursion before the compile, so no fixpoint runs here; the compile step says where one does.",
    captions: {
      read: "`tddlog::read` takes the bytes and their format and returns the program `solve` takes. A ProbLog file is its own program, so there is no translation to show.",
      parse: "The parser turns the text into clauses: the 14 edges as probabilistic facts, the two rules of `path/2`, the second reading `path` itself, and the six queries `path(1,Y)`.",
      ground: "The grounder writes rules without variables over the atoms the queries need. Every query binds the first argument of `path` to 1, so the recursion "
        + "is grounded from that bound end: paths from node 1 alone. The edges into node 1 drop out with `path(1,1)`, which no query needs. What is left is one "
        + "cyclic component of six atoms: `path(1,2)` reads `path(1,7)`, which reads `path(1,4)`, and so on round the graph.",
      rewrite: "A recursive component whose rules each read one of its atoms is reachability, and the rewrite writes it without recursion (src/ground/eliminate.rs): "
        + "its atoms are eliminated one at a time, as in Floyd and Warshall's algorithm, so that each reads only atoms eliminated after it. A label that "
        + "several paths share becomes an atom of its own, here `'$e'(0)` and `'$e'(1)`. What is left is eight acyclic components of one atom each.",
      layout: "`auto` gives a program that grounds to a recursive component the `dynamic` trial, and it finished in its first attempt. Its vtree is right-linear "
        + "over a pre-order of the min-fill elimination tree of the program's interaction graph, and may change during the compile. Its leaves are the 12 edges; "
        + "every diagram that follows is shaped by it.",
      compile: "The components compile in dependency order, a formula over the edges for each atom, built from its rules' bodies by TDD operations, which the "
        + "trace counts by kind. The largest, `path(1,6)`, has 79 pairs. No component is cyclic, so no fixpoint runs. Where a recursive component stays "
        + "(rules that read two of its atoms, negation through a cycle), its formulas are computed again in rounds until none changes, and the table "
        + "gives each such component's rounds.",
      evidence: "Without evidence, a query is counted when its formula is dropped, once nothing reads it, during the compile: all six were counted there, and "
        + "this stage has nothing to conjoin.",
      answers: "The answers in the program's order: what `tddlog examples/cycle_graph.pl` prints, and ProbLog 2.3.0's answers (tests/it/examples.rs).",
    },
  },
  {
    key: "network", title: "A Bayesian network with evidence", what: "A network file read as a program: its tables, its families and the evidence.",
    file: "examples/asia.net", example: "asia", evidence: true,
    command: "tddlog --trace network.json --evidence-probability --evidence tests/networks/asia_leaves.inst examples/asia.net",
    lede: "The asia network in Hugin's format, with its two leaves observed: `xray` yes and `dysp` no (tests/networks/asia_leaves.inst). tddlog reads the network "
      + "as a program, each row of a table an annotated disjunction, and answers the marginals of the six other variables and P(e).",
    captions: {
      read: "The network and its evidence file (Ace's `.inst`) are read as one program, which `tddlog --program` prints: each row of a table an annotated "
        + "disjunction over the variable's values, conditioned on its parents' values, and `either`, a logical or of `tub` and `lung`, as rules. It is "
        + "examples/bn_asia_evidence.pl, which the playground opens.",
      parse: "18 clauses: the 14 rows with a choice as annotated disjunctions and the four rows of `either` as rules; the two observations, and twelve queries, a "
        + "value each of the six variables not observed.",
      ground: "Each annotated disjunction of two values becomes a switch, a probabilistic fact true for its first value (`'$swtub_1_0'` for `tub` given "
        + "`asia(yes)`), and rules over it. Of the observed variables only the observed values' rules remain. A network is acyclic: 14 components of one atom.",
      rewrite: "`families:0` reads the program through indicators: each atom that another family reads is read there through a copy, a fact of its own "
        + "(`'$v'`, 16 copies of 12 atoms), tied to the atom by a link (`'$c'`) that the program observes. The 10 observations are the network's 2 and the 8 links.",
      layout: "Here `auto`'s first trial is `families:0`, and it finished in its first attempt. A family is a formula the query phase conjoins (a link, an observed "
        + "or queried atom), and `families:0` lays them out by a min-fill decomposition of the graph of their groups. The vtree's 30 leaves are the 14 switches and the 16 copies.",
      compile: "Each of the 22 components is one atom. The variables' own atoms take no pairs here; the work is in the links, which hold the tables' rows "
        + "as their clauses (src/indicators), and in the observed atoms, the largest `dysp(no)` at 41 pairs. A formula is stored as the atom's negation (the "
        + "dual) and compiles on the vtree projected to its component's ancestor cone.",
      evidence: "The 10 observations are conjoined along the vtree into one diagram of 120 pairs, whose count is P(e) = 0.0396199356. Each query is counted "
        + "against it: P(q | e) = WMC(q ∧ e) / WMC(e).",
      answers: "Twelve marginals and P(e): ProbLog 2.3.0's answers on examples/bn_asia_evidence.pl, the program the network was read as.",
    },
  },
  {
    key: "dice", title: "A Dice program", what: "A probabilistic functional program: integers held as bits, a derived atom per bit and step.",
    file: "tests/dice/walk.dice", example: "walk", evidence: true,
    command: "tddlog --trace dice.json --evidence-probability tests/dice/walk.dice",
    lede: "A random walk on 0..7 in Dice: ten steps of plus or minus one from 3, absorbed at either end, observed not to end at 0. tddlog reads Dice itself: an "
      + "integer is held as bits, and the answer is the distribution of the walk's end.",
    captions: {
      read: "tddlog's Dice front end evaluates the program symbolically and writes it as Datalog: `x` is three bits, `step_b0(T)` to `step_b2(T)` at step `T`; "
        + "each step's flip is `0.4::x_c_3(T)`; `iterate` becomes a recursion over a counter `T`, a level per step; and each value of the result is a query "
        + "`x(V)` over its bits.",
      parse: "44 clauses: the flip as a probabilistic rule, the start's bits and step 0 as facts, and 40 rules; eight queries, a value each, and the observation "
        + "`obs`, that the walk does not end at 0.",
      ground: "The grounder grounds the recursion over the steps level by level (`levels`) and proves atoms constant as each level completes (`constants`). "
        + "Passes over the gates then merge atoms equal in their rules or in function (`merged`; `swept`, candidates proved equal by a SAT solver), and the "
        + "conditioning fixes atoms the observation decides. 215 atoms become 90: the ten flips and 80 gates, none cyclic.",
      rewrite: "No recursion is left once the steps are ground, and `closed` reads no indicators: the program compiles as ground.",
      layout: "`closed` is `auto`'s first trial for a program of at most 64 facts with 8 or more derived atoms per fact (here 10 and 80), and it finished in its "
        + "first attempt. Each component's new facts go on a right-linear spine above the blocks it reads: over the ten flips, a chain.",
      compile: "Each of the 80 gates is a component of its own, compiled in dependency order, a step after the other. The formulas grow with the steps, as the "
        + "walk's position depends on more flips, up to 59 pairs.",
      evidence: "The observation is conjoined into a diagram of 41 pairs, whose count is P(e) = 0.432898, the probability that the walk does not end at 0; the "
        + "eight values are counted against it.",
      answers: "The end's distribution under the observation. Ten steps from 3 end on an odd number unless absorbed at 0, which is observed away, so the even "
        + "values have probability 0. Dice's own answers are the same (tests/it/examples.rs).",
    },
  },
  {
    key: "circuit", title: "A bit-blasted circuit", what: "The and-inverter graph of Alea.jl's circuit, optimized, then compiled gate by gate in dependency order.",
    file: "examples/alea_dice.pl", example: "circuit", evidence: true,
    command: "tddlog --trace circuit.json --evidence-probability examples/alea_dice.pl",
    lede: "Two dice, the first at least 4, as the circuit Alea.jl builds for them: 8 flips and 78 gates, written as a ProbLog program by scripts/alea2problog.jl. "
      + "The queries are that the sum is 7 (`q1`) and that the first shows more (`q2`). A bit-blasted program is a deep and narrow computation graph.",
    captions: {
      read: "The circuit is a ProbLog program: a probabilistic fact per flip, `f1` to `f8`; an and-gate a rule and an or-gate a rule per input, a negated input "
        + "written `\\+`; then the observation and the two queries.",
      parse: "119 clauses: the 8 flips and 111 rules; two queries and one observation.",
      ground: "Before any diagram is built, the grounder optimizes the gates as an and-inverter graph: 24 views (atoms read once, or of one rule over facts "
        + "alone) replaced by their bodies in their readers, 19 atoms merged into an equal one, and 4 candidates proved equal by SAT sweeping. 88 atoms "
        + "and 111 rules become 40 and 49.",
      rewrite: "The layout that answered, `program`, orders the flips as the program states them, and a view inlined into its readers would move its flips "
        + "to them: it reads the circuit with its views in place, its 81 derived atoms as written. It names the derived atom that crosses the one narrow cut it finds, "
        + "`g76`, read across the cut through a copy `'$v'(g76,q2)` tied to it by an observed link.",
      layout: "Where the grounder inlined views, as in every exported circuit, `auto`'s first trial is `program`: a vtree right-linear over the flips in the "
        + "reverse of the order the program states them, read as an OBDD in the program's order, the variable order Alea.jl's BDD package built the "
        + "circuit in. It finished in its first attempt; its 9 leaves are the 8 flips and the copy.",
      compile: "A component per derived atom, 81 in all, compiled bottom-up in dependency order, each gate once and shared by its readers. A formula is stored as the gate's "
        + "negation (the dual), so an or-gate's is the conjunction of its inputs' negations and the trace counts no disjunctions. The largest formula "
        + "has 60 pairs.",
      evidence: "The observation and the link are conjoined into 40 pairs, whose count is P(e) = 0.5, the probability that the first die shows at least 4; "
        + "the name `g76` is kept to the end, and both queries are counted against the evidence.",
      answers: "1/6 and 2/3, the answers the two dice give by hand (tests/it/examples.rs).",
    },
  },
];

// A walkthrough's trace, fetched once.
function walkTrace(key) {
  draft.walks ??= {};
  draft.walks[key] ??= fetch(`next/walk/${key}.json${stamp}`).then((r) => {
    if (!r.ok) throw new Error(`walk/${key}.json: ${r.status}`);
    return r.json();
  });
  return draft.walks[key];
}

// A caption as a paragraph, `code` between backticks.
const caption = (text, cls) => element("p", cls ? { class: cls } : {},
  ...text.split("`").map((part, i) => (i % 2 ? element("code", {}, part) : part)));

const playgroundHref = (w) => `playground/?example=${w.example}${w.evidence ? "&evidence=1" : ""}&trace=1`;

function howView(route) {
  const w = WALKS.find((x) => x.key === route.walk);
  if (!w) {
    return [crumbs(["Home", { view: "home" }], ["How it works"]), element("h2", {}, "How it works"),
      element("p", { class: "lede" }, "tddlog answers a program in six stages, for the command line and a library caller alike; ",
        element("a", { href: `${BRANCH}main/docs/design.md#how-a-program-is-answered` }, "docs/design.md"),
        " describes each and the decisions behind it. The diagrams are built bottom-up, by TDD operations on the rules."),
      pipeline(-1),
      element("h3", {}, "Four runs, step by step"),
      element("p", { class: "small" }, "Each is the trace of one run of tddlog (", element("code", {}, "tddlog --trace"),
        "), drawn a stage at a time with a caption by hand."),
      element("nav", { class: "entries walks", "aria-label": "Walkthroughs" }, ...WALKS.map((x) =>
        element("a", { href: `#/how/${x.key}`, class: "entry" }, element("b", {}, x.title), element("span", {}, x.what)))),
      element("p", { class: "also" }, "The playground draws the same trace for a program of your own: ",
        element("a", { href: "playground/?trace=1" }, "tick Trace"), " and run it.")];
  }
  const pipe = element("div", {}, pipeline(-1));
  const out = [crumbs(["Home", { view: "home" }], ["How it works", { view: "how" }], [w.title]), element("h2", {}, w.title),
    caption(w.lede, "lede"),
    element("p", { class: "small" }, "The trace of ", element("code", {}, w.command), " (",
      element("a", { href: `${BRANCH}main/${w.file}` }, element("code", {}, w.file)), "). ",
      element("a", { href: playgroundHref(w), class: "try-btn" }, "Open in playground")),
    pipe];
  const walk = element("div", { class: "walk" }, note("Loading the trace…"));
  out.push(walk);
  walkTrace(w.key).then((trace) => {
    const steps = tddlogTrace.steps(trace);
    const i = Math.min(Math.max(route.step, 0), steps.length - 1);
    const s = steps[i];
    pipe.replaceChildren(pipeline(STAGE_OF[s.key] ?? -1));
    const list = element("ol", { class: "steps" }, ...steps.map((t, k) =>
      element("li", {}, element("a", { href: `#/how/${w.key}/${k + 1}`, "aria-current": k === i ? "step" : null }, `${k + 1}. ${t.title}`))));
    const body = element("div", { class: "step" }, element("h4", {}, `${i + 1}. ${s.title}`));
    if (w.captions[s.key]) body.append(caption(w.captions[s.key]));
    const drawn = element("div", { class: "walk-trace" });
    drawn.innerHTML = tddlogTrace.stepHtml(s);
    body.append(drawn);
    const nav = element("p", { class: "step-nav" }, i > 0 ? element("a", { href: `#/how/${w.key}/${i}` }, "← Back") : "",
      i < steps.length - 1 ? element("a", { href: `#/how/${w.key}/${i + 2}` }, "Next →") : "");
    walk.replaceChildren(list, element("div", {}, body, nav));
  }, (e) => walk.replaceChildren(note(`The trace did not load: ${e.message}.`)));
  return out;
}

// ------------------------------------------------------------- the routes

// FNV-1a (32 bits) of a category's rows' SUITE/NAME, a line each, as
// views.py computes it.
function fingerprint(rows) {
  let h = 0x811c9dc5;
  for (const b of new TextEncoder().encode(rows.map((r) => `${r[0]}/${r[1]}`).join("\n"))) h = Math.imul(h ^ b, 0x01000193) >>> 0;
  return h;
}

// The types whose rows views.json was not written from (rule 3: its kinds
// would be another instance's).
function staleViews() {
  if (draft.stale === undefined) {
    draft.stale = data.categories.filter((c) => draft.views.categories[c.key]?.rows !== fingerprint(c.rows)).map((c) => c.title);
  }
  return draft.stale;
}
const staleNote = (stale) => note(`These views (views.json) were written from other results than these (${and(stale)}); `
  + "views.py writes them again from the results page's data.");

const DRAFT_VIEWS = { "draft-home": draftHome, tools: toolsView, tool: toolView, types: typesView, coverage: coverageView, how: howView };

Object.assign(extend, {
  load: async () => {
    const response = await fetch(`next/views.json${stamp}`);
    if (!response.ok) throw new Error(`views.json: ${response.status}`);
    draft.views = await response.json();
  },
  parse: (hash) => {
    const parts = hash.replace(/^\//, "").split("/").filter(Boolean).map((p) => decodeURIComponent(p));
    if (!parts.length) return { view: "draft-home" };
    if (parts[0] === "tools") {
      if (parts.length === 1) return { view: "tools" };
      return draft.views?.systems?.[parts[1]] && parts.length === 2 ? { view: "tool", system: parts[1] } : null;
    }
    if (parts[0] === "types" && parts.length === 1) return { view: "types" };
    if (parts[0] === "coverage" && parts.length === 1) return { view: "coverage" };
    if (parts[0] === "how" && parts.length === 1) return { view: "how" };
    if (parts[0] === "how" && WALKS.some((w) => w.key === parts[1])) {
      if (parts.length === 2) return { view: "how", walk: parts[1], step: 0 };
      if (parts.length === 3 && /^[1-9]\d{0,2}$/.test(parts[2])) return { view: "how", walk: parts[1], step: Number(parts[2]) - 1 };
    }
    return null;
  },
  href: (route) => ({ "draft-home": "#/", tools: "#/tools", tool: `#/tools/${encodeURIComponent(route.system)}`, types: "#/types",
    coverage: "#/coverage", how: route.walk ? `#/how/${route.walk}${route.step ? `/${route.step + 1}` : ""}` : "#/how" })[route.view] ?? null,
  draw: (route) => {
    if (!(route.view in DRAFT_VIEWS)) return null;
    const stale = staleViews();
    if (stale.length && route.view !== "draft-home" && route.view !== "how") return [staleNote(stale)];
    return DRAFT_VIEWS[route.view](route);
  },
  title: (route) => ({ "draft-home": null, tools: "By tool", tool: `tddlog against ${data.names[route.system]}`, types: "By type of program",
    coverage: "Coverage", how: route.walk ? `How it works: ${WALKS.find((w) => w.key === route.walk).title}` : "How it works" })[route.view] ?? null,
  category: (c, route, rows) => (staleViews().length ? [staleNote(staleViews())] : typeSplit(c, route, rows)),
  // a translator's or generator's name in a line on how a file was made, as a link to its source
  made: (html) => html.replace(/<code>([\w-]+\.(?:py|jl|rkt))(\b[^<]*)<\/code>/g, (whole, name, rest) =>
    (draft.views?.scripts.includes(name) ? `<a href="${BRANCH}baselines/scripts/${name}"><code>${name}${rest}</code></a>` : whole)),
});

