"use strict";

// The draft of the site's redesign: views of the results by tool and by type
// of program, head-to-head apart from translated, what the systems' own
// sources hold that tddlog does not run, and how a program is answered. It
// is the results page's app (results.js, loaded before this script, whose
// helpers and data it uses) with these views added through its `extend`:
//   #/                the home page: what tddlog is, the overview figure, the sections
//   #/tools           by tool: every system, its head-to-head and translated speed-ups and its coverage
//   #/tools/SYSTEM    tddlog against SYSTEM: head-to-head, translated (each direction and translator),
//                     coverage, and how it was run
//   #/types           by type of program: the six types and all programs
//   #/coverage        the programs each system's sources hold that tddlog does not run, and why
//   #/how             how a program is answered: the pipeline and a walkthrough
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

// A comparison's line: the two systems' answers, their PAR-2 totals over the
// instances the system is given and either answers, and the speed-up of the
// members of mine it is with.
function versusLine(v, name, n) {
  if (!v) return note(`No instances.`);
  return element("p", { class: "headline" }, element("b", {}, `${name} ${count(v.answers)} of ${count(n ?? v.given)}, ${subject(v.me)} ${count(v.tddlog_answers)}`),
    `. Over ${totals(v, name)}; ${subject(v.me)}'s speed-up: ${speedup(v)}.`);
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
    ["#/how", "How it works", "The pipeline from a program to its probabilities, and a walk through one program's run."],
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
    element("p", { class: "also" }, element("a", { href: "../" }, "The playground"), " runs tddlog in the browser; the ",
      element("a", { href: "#/coverage" }, "coverage"), " lists what each system's own sources hold that tddlog does not run, and why."),
  ];
}

// ------------------------------------------------------------- by tool

function toolsView() {
  const out = [crumbs(["Home", { view: "home" }], ["By tool"]), element("h2", {}, "By tool"),
    element("p", { class: "lede" }, "tddlog against each system. Head-to-head: both read the benchmark's program as its authors or "
      + "its generator wrote it, or the same network in another network format. Translated: one of them, or both, read a "
      + "translation. Each line gives the instances and tddlog's speed-up (the results page's measure, below), and what "
      + "the system answers that tddlog does not.")];
  const grid = element("div", { class: "tool-grid" });
  for (const s of data.systems) {
    if (s.key === "tddlog") continue;
    const v = draft.views.systems[s.key];
    if (!v) continue;
    const t = v.n.s + v.n.m + v.n.b;
    const line = (label, n, w) => element("div", { class: "kv" }, element("span", {}, label),
      element("b", {}, n ? `${count(n)}, ${speedup(w)}` : "none"));
    const native = nativeTo(s.key).map((c) => c.title);
    grid.append(element("a", { href: `#/tools/${encodeURIComponent(s.key)}`, class: "tool-card" },
      element("span", { class: "tname" }, swatch(s.key), data.names[s.key]),
      element("span", { class: "muted small" }, native.length ? `Native to ${and(native.map(lower))}` : "Given programs through a translation"),
      line("head-to-head", v.n.h, v.head), line("translated", t, v.translated),
      element("div", { class: "kv" }, element("span", {}, "answers tddlog does not"), element("b", {}, count(v.beyond.length)))));
  }
  out.push(grid, note(measureText()));
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
    element("p", { class: "headline" }, element("b", {}, `${name} answers ${count(s.answers)} of the ${count(s.given)} instances it was given`),
      `, of ${and(typesOf(x).map((c) => lower(c.title)))}.`),
    element("nav", { class: "panels-nav", "aria-label": "Panels" }, element("a", { href: "#head" }, "Head-to-head"), element("a", { href: "#translated" }, "Translated"),
      element("a", { href: "#coverage" }, "Coverage"), element("a", { href: "#run" }, "How it was run"))];
  // head-to-head
  const head = rows.filter((r) => r.kind === "h");
  const p1 = element("section", { class: "panel", id: "head" }, element("h3", {}, "Head-to-head"),
    note(`The ${plural(head.length, "instance")} where ${name} and tddlog read the same program: the benchmark's as its authors or its `
      + "generator wrote it, or the same network in another network format (the same variables and tables)."));
  if (head.length) p1.append(versusLine(v.head, name, head.length), pairPlot(head, x, v.head, "h"), familyCounts(head));
  out.push(p1);
  // translated, by direction and translator
  const tr = rows.filter((r) => "smb".includes(r.kind));
  const p2 = element("section", { class: "panel", id: "translated" }, element("h3", {}, "Translated"),
    note(`The ${plural(tr.length, "instance")} where one of them, or both, read a translation: below, each direction and the `
      + "translator, with its source on the baselines branch. tddlog's own readers (of Dice programs, LPADs and networks) are front ends, not translations."));
  if (tr.length) {
    p2.append(versusLine(v.translated, name, tr.length));
    const ways = element("div", { class: "ways" });
    for (const w of v.ways) {
      const kind = KIND_TEXT[w.kind](name);
      ways.append(element("div", { class: "way" },
        element("p", { class: "way-head" }, element("b", {}, kind), element("span", { class: "n" }, ` ${plural(w.n, "instance")}`)),
        ...(w.reason ? [element("p", {}, element("span", { class: "muted" }, `${name} read: `), w.reason)] : []),
        ...(w.mine ? [element("p", {}, element("span", { class: "muted" }, "tddlog read: "), w.mine)] : []),
        ...(w.by.length ? [element("p", { class: "by" }, "Written by ", ...w.by.flatMap(([n, path], i) => [i ? ", " : "", scriptLink(n, path)]))] : []),
        element("p", { class: "small" }, w.vs ? `${name} ${count(w.vs.answers)}, tddlog ${count(w.vs.tddlog_answers)}; tddlog's speed-up: ${speedup(w.vs)}` : "")));
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
    note(measureText()), element("p", {}, link({ view: "system", system: x }, `${name}'s full page`),
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
      const dirs = keep === "h" ? [] : [element("td", { class: "num" }, ["s", "m", "b"].map((k) => count(n(x, k))).join(" / "))];
      body.append(element("tr", { class: x === route.vs ? "vs" : null }, element("th", { scope: "row" }, toolLink(x)),
      element("td", { class: "num" }, link({ view: "category", category: c.key, vs: x }, count(n(x, keep)))), ...dirs,
      element("td", { class: "num" }, w ? `${count(w.answers)} / ${count(w.tddlog_answers)}` : ""),
      element("td", { class: "num" }, w ? `${duration(w.total)} / ${duration(w.tddlog_total)}` : ""),
      element("td", { class: "num" }, w ? speedup(w) : "")));
    }
    const sec = element("section", { class: "panel split", id }, element("h3", {}, `${title}: ${have.length ? and(have.map((x) => data.names[x])) : "no system"}`),
      note(`Where ${text}.`));
    if (have.length) {
      sec.append(scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
        element("th", { scope: "col" }, "system"), element("th", { scope: "col", class: "num" }, "instances"),
        ...(keep === "h" ? [] : [element("th", { scope: "col", class: "num", title: "a translation for the system / for tddlog / for both" }, "for it / tddlog / both")]),
        element("th", { scope: "col", class: "num" }, "answers, it / tddlog"),
        element("th", { scope: "col", class: "num", title: "PAR-2, over the instances the system is given and either answers" }, "total times, it / tddlog"),
        element("th", { scope: "col", class: "num" }, "tddlog faster"))), body)), "whole"));
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
      element("td", { class: "num" }, `${count(kinds("h"))} / ${count(xs.filter((x) => /[smb]/.test(v.kinds[x])).length)}`)));
  }
  return [crumbs(["Home", { view: "home" }], ["By type of program"]), element("h2", {}, "By type of program"),
    element("p", { class: "lede" }, "Each instance counts in one type, the type of the model it is, whatever language its source was written in. "
      + "Each type is compared with the systems native to it, head-to-head first and translated second; all programs is their union, with every system. "
      + "A type opens its families, and a family its instances, each with the programs every system read and the commands its job ran."),
    scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "type"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num", title: "by each instance's subject: tddlog, or Alea.jl+tddlog on Alea.jl's circuits" }, "tddlog answers"),
      element("th", { scope: "col" }, "native systems"),
      element("th", { scope: "col", class: "num", title: "the systems compared head-to-head on some instance / through a translation on some" }, "systems head-to-head / translated"))),
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

// The walkthrough: examples/cycle_graph.pl, from a run of tddlog 0.1.0
// (walk/: its answers, --ground, --ground --vtree closed and --stats), a
// step per stage.
const WALK = [
  { title: "Parse", api: "src/syntax; tddlog::solve(source, &options, progress) runs the six stages",
    text: "A graph of seven nodes whose fourteen edges are each present with a probability of their own, independently, and the probability of a path from node 1 to each other node. The edges are probabilistic facts: a choice of them is a world. The paths are derived: in each world, the least fixpoint of the two rules. The graph has cycles (1, 2, 3 and back to 1, among others).",
    file: "program", why: "The progress callback hears of each stage (Parsed, Grounded, Compiling, Answered); the playground shows them." },
  { title: "Ground", api: "src/ground; tddlog --ground",
    text: "The grounder writes only what the queries depend on. path/2 is grounded from the bound end, node 1, as path(1,3) :- edge(2,3), path(1,2): the paths from node 1 alone, 6 atoms and 12 rules, not the paths between every two nodes. edge(3,1) and edge(6,1), which lead back to node 1, are left out: a path that returns to node 1 reaches nothing that a path from it does not.",
    file: "ground", why: "Grounding for the queries keeps the program as small as what they ask." },
  { title: "Rewrite", api: "src/ground/eliminate.rs, ground::without_linear_recursion; tddlog --ground --vtree closed",
    text: "The six paths are one cyclic component (path(1,2) reads path(1,7), which reads path(1,4), which reads path(1,3), which reads path(1,2)), and each of its rules reads one of its atoms, positively: linear recursion. tddlog writes it without recursion by eliminating its atoms one at a time; a label that joins several paths and goes on becomes an atom of its own, '$e'(0) and '$e'(1). path(1,4) now reads edges alone, and every other path follows from it in order: 20 atoms (12 facts, 8 derived) and 16 rules.",
    file: "ground_closed", stats: ["ground:"], why: "Without a cycle, each atom's diagram is built once, in dependency order, with no fixpoint to iterate." },
  { title: "Choose a layout", api: "src/vtree, src/auto; vtree::Kind, Strategy; the layout and trial lines of --stats",
    text: "A vtree, a binary tree whose leaves are the 12 edges, shapes every diagram. auto, the default strategy, tries a few layouts in turns under bounds in pairs and work that double each turn, and the first trial to finish answers. Here its first trial, dynamic:minfill (a vtree over a min-fill elimination order of the edges, which the compile may change between components), finishes in its first attempt.",
    stats: ["dynamic:", "layout:", "trial:"], why: "One function can take a few pairs on one vtree and millions on another." },
  { title: "Compile", api: "src/tp; the compile lines of --stats",
    text: "Each derived atom gets a formula, a TDD over the edges that holds in exactly the worlds where the atom does, by Tp-compilation: the rules applied to diagrams rather than to truth values. The 8 atoms go as 8 components in dependency order, none cyclic, so none is iterated: 20 conjunctions, 16 negations and 23 minimizations, the largest formula 79 pairs, 14,580 of the library's work units.",
    stats: ["compile (", "compile operations:", "compile negations:"], why: "Sizes are counted in pairs and work in work units, never in seconds, so a run takes the same decisions on every machine." },
  { title: "Evidence and queries", api: "src/query, src/eval; the evidence and queries line of --stats",
    text: "There is no evidence here, so each query's probability is the weighted model count of its formula, an edge weighing its probability true and one minus it false. The six queries are counted in the compile itself.",
    stats: ["evidence and queries:"], file: "answers", why: "ProbLog gives the same six probabilities (tests/it/examples.rs)." },
];

async function walkFiles() {
  if (!draft.walk) {
    draft.walk = Promise.all(["program", "ground", "ground_closed", "stats", "answers"].map((f) =>
      fetch(f === "program" ? `../examples/cycle_graph.pl${stamp}` : `walk/${f}.txt${stamp}`).then((r) => (r.ok ? r.text() : `(${f}: ${r.status})`)))).then(([program, ground, ground_closed, stats, answers]) =>
      ({ program, ground, ground_closed, stats, answers }));
  }
  return draft.walk;
}

function howView(route) {
  const step = Math.min(Math.max(route.step ?? 0, 0), WALK.length - 1);
  const s = WALK[step];
  const out = [crumbs(["Home", { view: "home" }], ["How it works"]), element("h2", {}, "How it works"),
    element("p", { class: "lede" }, "tddlog answers a program in six stages, for the command line and a library caller alike; ",
      element("a", { href: `${BRANCH}main/docs/design.md#how-a-program-is-answered` }, "docs/design.md"),
      " describes each and the decisions behind it. The diagrams are built bottom-up, by TDD operations on the rules."),
    pipeline(step),
    element("h3", {}, "A walk through one run: reachability in a graph with cycles"),
    element("p", { class: "small" }, "From a run of tddlog 0.1.0 on ", element("a", { href: `${BRANCH}main/examples/cycle_graph.pl` }, element("code", {}, "examples/cycle_graph.pl")),
      ": what each stage printed. ", element("a", { href: "../?example=reachability", class: "try-btn" }, "Open in playground"))];
  const steps = element("ol", { class: "steps" }, ...WALK.map((w, i) => element("li", {},
    element("a", { href: `#/how/${i + 1}`, "aria-current": i === step ? "step" : null }, `${i + 1}. ${w.title}`))));
  const body = element("div", { class: "step" }, element("h4", {}, `${step + 1}. ${s.title}`), element("p", {}, s.text),
    element("p", { class: "api" }, element("span", { class: "muted" }, "Where: "), element("code", {}, s.api)),
    element("p", { class: "why" }, element("span", { class: "muted" }, "Why: "), s.why));
  const code = element("div", { class: "walk-code" }, note("Loading…"));
  body.append(code);
  walkFiles().then((files) => {
    const parts = [];
    if (s.file) parts.push(element("pre", { class: "src" }, files[s.file]));
    if (s.stats) {
      const want = [].concat(s.stats);
      const lines = files.stats.split("\n").filter((l) => want.some((p) => l.startsWith(p)));
      parts.push(element("p", { class: "small muted" }, "tddlog --stats examples/cycle_graph.pl, the lines of this stage:"),
        element("pre", { class: "src stats" }, lines.join("\n")));
    }
    code.replaceChildren(...parts);
  });
  const nav = element("p", { class: "step-nav" }, step > 0 ? element("a", { href: `#/how/${step}` }, "← Back") : "",
    step < WALK.length - 1 ? element("a", { href: `#/how/${step + 2}` }, "Next →") : "");
  out.push(element("div", { class: "walk" }, steps, element("div", {}, body, nav)),
    element("h3", {}, "Coming"),
    element("ul", {}, element("li", {}, "Three more walkthroughs: a Bayesian network (tables, families, evidence), a Dice program (bits and values) and a bit-blasted circuit (the and-inverter graph, names and links)."),
      element("li", {}, "A trace of the run (tddlog --trace, JSON), which the playground draws for a program of the reader's own, step by step as here.")));
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
const staleNote = (stale) => note(`This draft's views (views.json) were written from other results than these (${and(stale)}); `
  + "views.py writes them again from the results page's data.");

const DRAFT_VIEWS = { "draft-home": draftHome, tools: toolsView, tool: toolView, types: typesView, coverage: coverageView, how: howView };

Object.assign(extend, {
  load: async () => {
    const response = await fetch(`views.json${stamp}`);
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
    if (parts[0] === "how" && parts.length === 1) return { view: "how", step: 0 };
    if (parts[0] === "how" && parts.length === 2 && /^[1-9]\d*$/.test(parts[1]) && Number(parts[1]) <= WALK.length) {
      return { view: "how", step: Number(parts[1]) - 1 };
    }
    return null;
  },
  href: (route) => ({ "draft-home": "#/", tools: "#/tools", tool: `#/tools/${encodeURIComponent(route.system)}`, types: "#/types",
    coverage: "#/coverage", how: route.step ? `#/how/${route.step + 1}` : "#/how" })[route.view] ?? null,
  draw: (route) => {
    if (!(route.view in DRAFT_VIEWS)) return null;
    const stale = staleViews();
    if (stale.length && route.view !== "draft-home" && route.view !== "how") return [staleNote(stale)];
    return DRAFT_VIEWS[route.view](route);
  },
  title: (route) => ({ "draft-home": "Home", tools: "By tool", tool: `tddlog against ${data.names[route.system]}`, types: "By type of program",
    coverage: "Coverage", how: "How it works" })[route.view] ?? null,
  category: (c, route, rows) => (staleViews().length ? [staleNote(staleViews())] : typeSplit(c, route, rows)),
  // a translator's or generator's name in a line on how a file was made, as a link to its source
  made: (html) => html.replace(/<code>([\w-]+\.(?:py|jl|rkt))(\b[^<]*)<\/code>/g, (whole, name, rest) =>
    (draft.views?.scripts.includes(name) ? `<a href="${BRANCH}baselines/scripts/${name}"><code>${name}${rest}</code></a>` : whole)),
});

// The site's sections in the header, the one on show marked; the rail of
// the results beside the views of the results only.
function mark() {
  const route = parse(location.hash);
  const section = { "draft-home": "home", home: "home", tools: "tools", tool: "tools", types: "types", category: "types",
    coverage: "coverage", how: "how", system: "tools" }[route.view] ?? "types";
  for (const a of document.querySelectorAll("[data-nav]")) {
    if (a.dataset.nav === section) a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  }
  document.body.classList.toggle("wide", route.view in DRAFT_VIEWS);
}
window.addEventListener("hashchange", mark);
document.addEventListener("DOMContentLoaded", () => {
  const ready = setInterval(() => {
    if (draft.views && data.categories.length) {
      clearInterval(ready);
      mark();
    }
  }, 50);
});
