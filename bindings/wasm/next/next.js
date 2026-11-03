"use strict";

// The site's front page (index.html): its sections, as the bar at the top
// of every page and the menu name them (results.js's SECTIONS), and how a
// program is answered. It is the results page's app (results.js, loaded
// before this script, whose helpers and data it uses) with these views
// added through its `extend`:
//   #/                     the home page: what tddlog is, each type of program against its best baselines, the integrations
//   #/baselines            by baseline: every other system, what it is native to and what it answers
//   #/baselines/SYSTEM[/AT]
//                          tddlog against SYSTEM: head-to-head, translated (each direction and translator), in
//                          each type of program, its own benchmarks, coverage, and how it was run; AT names
//                          one of these, which the page scrolls to
//   #/integrations         by integration: tddlog alone, and in a pipeline after another system's own code
//   #/integrations/KEY     one of them: tddlog alone's answers by type; X+tddlog against X on X's programs
//   #/types                by program type: the five types and all programs, against the systems native to each
//   #/coverage             the programs each system's sources hold that tddlog does not run, and why
//   #/how[/WALK[/N]]       how a program is answered: a run's stages, a walkthrough at its step N
// #/tools/... and #/system/... are the addresses these pages once had. A
// type of program is results.js's view (#/category/C), with its comparisons
// head-to-head and translated before the rest; a suite, a family and an
// instance are results.js's views as they are.
//
// The data is views.json (release/results/rel/views.py): each system's kind
// of comparison on each instance of each type (head-to-head, or translated
// for it, for tddlog or for both), and the comparisons over each kind, which
// are panels.py's (rule 16), and the census of the programs tddlog does not
// run; and categories.json's integrations (panels.py's). A system is
// compared on each instance with tddlog or one of its pipelines (views.json's
// mine): in a type, with the instance's subject (the type's with); on the
// system's own page, with its own pipeline where that ran (Roulette+tddlog
// for Roulette; the system's kinds and with). The plots draw that member's
// times. The page counts and draws; it computes no total or speed-up.

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


// The types (not all programs) a system was given.
const typesOf = (x) => data.types.filter((c) => c.systems.includes(x));

// A system's rows over the types, as its page compares them, each with its category.
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

// tddlog's speed-up in a cell: "too few" where too few instances both answer.
const speedCell = (v) => (v?.speedup == null ? "too few" : speedup(v));

// A comparison in cells, each side's under its name: the instances; the
// system's answers and its PAR-2 total over the instances it is given and
// either answers; the same of the members of mine it is with, and their
// speed-up.
function versusCells(v, name, n) {
  if (!v) return note("No instances.");
  const me = subject(v.me), over = `over the ${plural(v.either, "instance")} ${name} is given and either answers, PAR-${data.measure.par}`;
  const side = (who, ...items) => element("div", { class: "side" }, element("span", { class: "who" }, who), cells(...items));
  return element("div", { class: "versus" }, side("", ["instances", count(n ?? v.given)]),
    side(name, ["answers", count(v.answers)], [`total (PAR-${data.measure.par})`, duration(v.total), over]),
    side(me, ["answers", count(v.tddlog_answers)], [`total (PAR-${data.measure.par})`, duration(v.tddlog_total), over], ["speed-up", speedCell(v), speedup(v, true)]));
}

// The cactus plot of x and the members it is with over rows (kindRows' me,
// drawn as one line), beside the scatter plot of their times; the speed-up
// is in the cells above it (versusCells), once.
function pairPlot(rows, x, v, key) {
  const name = data.names[x], me = subject(v?.me);
  const both = { key: `${x}-${key}`, title: `${name} and ${me}`, noun: "instances", systems: ["tddlog", x],
    instances: rows.map((r) => [r.name, r.v[r.me], r.v[x]]), limit: Math.max(...rows.map((r) => r.limit)) };
  return element("div", { class: "pair" }, plotted(both, { ...data.names, tddlog: me }, { all: true }), scatter(rows, ["tddlog", x], x, false, (r) => r.me));
}

// The families rows are of, by how many each: a family opens in its type of
// program, its code on its benchmark pages; past the first eight, the rest
// folded.
const FAMILIES_SHOWN = 8;
function familyCounts(rows) {
  const out = new Map();
  for (const r of rows) {
    const at = data.place.get(`${r.suite}/${r.name}`);
    const key = at ? `${at.c.key}/${at.g.key}` : r.suite;
    if (!out.has(key)) out.set(key, { at, rows: [] });
    out.get(key).rows.push(r);
  }
  const list = [...out.values()].sort((a, b) => b.rows.length - a.rows.length);
  const table = (items) => scroller(stacked(element("table", { class: "board fams-t" }, element("thead", {}, element("tr", {},
    element("th", { scope: "col" }, "family"), element("th", { scope: "col", title: "the family's programs, each system's, and its runs" }, "program"),
    element("th", { scope: "col", class: "num" }, "instances"))),
  element("tbody", {}, ...items.map(({ at, rows: rs }) => element("tr", {},
    element("th", { scope: "row" }, at ? link({ view: "category", category: at.c.key, only: at.g.key }, at.g.title) : "another suite"),
    codeCell(rs), element("td", { class: "num" }, count(rs.length))))))), "whole");
  const first = list.slice(0, FAMILIES_SHOWN), rest = list.slice(FAMILIES_SHOWN);
  return element("div", { class: "fams" }, table(first),
    ...(rest.length ? [lazy({ class: "fams-more" }, `${plural(rest.length, "more family", "more families")}`, () => [table(rest)])] : []));
}

// A type's title within a sentence: "probabilistic logic programs", "Bayesian networks".
const lower = (title) => title.toLowerCase().replace(/\bbayesian\b/, "Bayesian").replace(/\bmarkov\b/, "Markov");

// A system's swatch and name, opening its page: a baseline's, or an integration's.
const systemLink = (x) => element("a", { href: href({ view: "system", system: x }), class: "tool" }, swatch(x), data.names[x]);

// ------------------------------------------------------------- home

// What tddlog is; each type of program, with tddlog's answers and its
// speed-up over the best baselines native to it (the README's, c.named);
// and a line per integration.
function draftHome() {
  const all = data.categories.find((c) => c.key === ALL);
  const others = data.systems.filter((s) => !isMine(s.key)).length;
  const body = element("tbody");
  for (const c of data.types) {
    // tddlog's answers; on a type it is not given, those of the pipelines it is in
    const answers = c.answers.tddlog !== undefined ? [count(c.answers.tddlog)]
      : [element("span", { class: "muted small" }, `${subject(subjectsOf(catRows(c)))} `), count(answeredBy(catRows(c)))];
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "category", category: c.key }, c.title)),
      element("td", { class: "num" }, count(c.instances)), element("td", { class: "num" }, ...answers),
      element("td", { class: "best" }, element("div", { class: "chips" }, ...c.named.map((x) => element("span", { class: "chip" }, systemLink(x),
        element("span", { class: /×/.test(speedup(c.vs[x])) ? "x" : "x none", title: `${subject(c.vs[x].me)} ${speedup(c.vs[x], true)}` }, speedup(c.vs[x]))))))));
  }
  const lines = integrationsOf().filter((k) => data.integrations[k].system).map((k) => {
    const i = data.integrations[k];
    return element("li", {}, link({ view: "integration", key: k }, i.title), ` against ${data.names[i.system]}: `,
      `${speedup(i.vs[i.system], true)}, on ${plural(i.instances, "instance")} of ${data.names[i.system]}'s.`);
  });
  return [
    element("section", { class: "intro" },
      element("p", { class: "big" }, "tddlog computes exact probabilities for probabilistic logic programs, Bayesian and Markov networks, "
        + "and probabilistic functional programs, by compiling them into tree decision diagrams."),
      element("p", { class: "lede" }, `In its release run it answers ${count(all.answers.tddlog ?? 0)} of ${count(all.instances)} instances of `
        + `${inWords(data.types.length)} types of program, against ${count(others)} baselines run on the same machines. `,
        element("a", { href: "playground/" }, "The playground"), " runs it in the browser.")),
    element("h2", { class: "home-h" }, "Each type of program against its best baselines"),
    scroller(stacked(element("table", { class: "board home-t" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "type of program"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num" }, "tddlog answers"),
      element("th", { scope: "col", title: "the baselines that answer most, each with tddlog's speed-up over it" }, "its best baselines, and tddlog's speed-up"))),
    body)), "whole"),
    ...(lines.length ? [element("h2", { class: "home-h" }, "tddlog after another system's own code"), element("ul", { class: "home-int" }, ...lines)] : []),
    note(measureLine()),
  ];
}

// ------------------------------------------------------------- by baseline

function baselinesView() {
  const body = element("tbody");
  for (const s of [...data.systems].sort((a, b) => b.answers - a.answers)) {
    const v = draft.views.systems[s.key];
    if (isMine(s.key) || !v) continue;
    const native = nativeTo(s.key).map((c) => lower(c.title));
    body.append(element("tr", {}, element("th", { scope: "row" }, systemLink(s.key)),
      element("td", { class: "about" }, native.length ? and(native) : "none: given programs through a translation"),
      element("td", { class: "num" }, count(s.given)), element("td", { class: "num" }, count(s.answers)),
      element("td", { class: "num" }, count(v.beyond.length))));
  }
  return [crumbs(HOME, [sectionName("baselines")]), element("h2", {}, sectionName("baselines")),
    element("p", { class: "lede" }, "Every other exact system, by the instances it answers. Each one's page compares it with tddlog."),
    scroller(stacked(element("table", { class: "board baselines-t" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "baseline"), element("th", { scope: "col" }, "native to"),
      element("th", { scope: "col", class: "num" }, "given"), element("th", { scope: "col", class: "num" }, "answers"),
      element("th", { scope: "col", class: "num", title: "the instances it answers that tddlog does not" }, "answers tddlog misses"))),
    body)), "whole")];
}

// The sections of a baseline's page, by the ids a link names (#/baselines/X/AT).
const BASELINE_AT = [["head", "Head-to-head"], ["translated", "Translated"], ["language", "Its own language"], ["types", "In each type"],
  ["own", "Its own benchmarks"], ["coverage", "Coverage"], ["run", "How it was run"]];

// tddlog against one baseline: head-to-head, translated, in each type of
// program, its own benchmarks, coverage, how it was run.
function baselineView(route) {
  const x = route.system, s = data.systems.find((y) => y.key === x), v = draft.views.systems[x], name = data.names[x];
  const rows = systemRows(x);
  const parts = solverParts(x);
  const site = s.link.replace(/^https?:\/\/(www\.)?/, "").replace(/\/$/, "");
  const at = BASELINE_AT.filter(([id]) => (id !== "own" || parts.own.length) && (id !== "language" || parts.languages.length));
  const out = [crumbs(HOME, inSection("baselines"), [name]),
    element("h2", { class: "solver" }, swatch(x), `tddlog against ${name}`),
    element("p", { class: "lede" }, `${s.about} `, element("a", { href: s.link }, site)),
    cells(["given", count(s.given), `instances of ${and(typesOf(x).map((c) => lower(c.title)))}`], ["answers", count(s.answers)],
      ["answers tddlog misses", count(v.beyond.length), `instances ${name} answers that tddlog does not`]),
    element("nav", { class: "panels-nav", "aria-label": "Sections" }, ...at.map(([id, text]) => link({ view: "baseline", system: x, at: id }, text)))];
  // head-to-head, its own benchmarks among them
  const head = rows.filter((r) => r.kind === "h");
  const own = head.filter((r) => r.cls[r.c.systems.indexOf(x)] === "o").length;
  const p1 = element("section", { class: "panel", id: "head" }, element("h3", {}, "Head-to-head"),
    note("Both read the same program: the benchmark's own, or the same network in another network format",
      own ? `; ${count(own)} of them are ${name}'s own benchmarks.` : "."));
  if (head.length) p1.append(versusCells(v.head, name, head.length), pairPlot(head, x, v.head, "h"), familyCounts(head));
  out.push(p1);
  // translated, by direction and translator
  const tr = rows.filter((r) => "smb".includes(r.kind));
  const p2 = element("section", { class: "panel", id: "translated" }, element("h3", {}, "Translated"),
    note("One of them, or both, read a translation; tddlog's own readers (of Dice programs, LPADs and networks) are front ends, not translations."));
  if (tr.length) {
    p2.append(versusCells(v.translated, name, tr.length));
    // the ways, under their direction, each named by its translators
    for (const kind of new Set(v.ways.map((w) => w.kind))) {
      const ways = element("div", { class: "ways" });
      for (const w of v.ways.filter((y) => y.kind === kind)) {
        const how = [...(w.reason ? [element("span", { class: "muted" }, `${name} read: `), w.reason, element("br")] : []),
          ...(w.mine ? [element("span", { class: "muted" }, "tddlog read: "), w.mine, element("br")] : []),
          ...(w.by.length ? ["Written by ", ...w.by.flatMap(([n, path], i) => [i ? ", " : "", scriptLink(n, path)])] : [])];
        ways.append(element("div", { class: "way" },
          w.by.length ? element("p", { class: "way-head" }, element("span", { class: "muted" }, "Written by "),
            ...w.by.flatMap(([n], i) => [i ? ", " : "", element("code", {}, n)])) : null,
          w.vs ? versusCells(w.vs, name, w.n) : cells(["instances", count(w.n)]),
          ...(how.length ? [fold("The programs and the translator", ...how)] : [])));
      }
      p2.append(element("h4", {}, KIND_TEXT[kind](name)), ways);
    }
    p2.append(pairPlot(tr, x, v.translated, "t"), familyCounts(tr));
  }
  out.push(p2);
  // in its own language and translated for it, then in each type of program it was given
  if (parts.languages.length) {
    out.push(element("section", { class: "panel", id: "language" }, element("h3", {}, "In its own language and translated for it"),
      note("Its own language: the program it read is the benchmark's, or the network in another network format; translated: a translator wrote it."),
      ...parts.languages));
  }
  out.push(element("section", { class: "panel", id: "types" }, element("h3", {}, "In each type of program it was given"), ...parts.types));
  if (parts.own.length) out.push(element("section", { class: "panel", id: "own" }, element("h3", {}, "Its own benchmarks"), ...parts.own.slice(1)));
  // coverage: what it answers that tddlog does not, what tddlog answers that it cannot be given, each in a cell;
  // then the programs of its own sources that tddlog does not run, and why
  const p3 = element("section", { class: "panel", id: "coverage" }, element("h3", {}, "Coverage"));
  const unexp = Object.entries(v.unexpressed).filter(([, [a, b]]) => a + b > 0);
  const [refused, missing] = unexp.reduce(([a, b], [, [c, d]]) => [a + c, b + d], [0, 0]);
  p3.append(cells(["it answers, tddlog does not", count(v.beyond.length), `instances ${name} answers that tddlog does not`],
    ...(refused ? [["tddlog answers, it cannot express", count(refused), "its translator refuses what its language lacks"]] : []),
    ["tddlog answers, it was not given", count(missing)]));
  const fails = new Map();
  for (const [cat, suite, inst, rec] of v.beyond) {
    const key = typeof rec === "string" ? rec : "other";
    if (!fails.has(key)) fails.set(key, []);
    fails.get(key).push([cat, suite, inst]);
  }
  for (const [key, list] of fails) {
    const what = FAILURES[key]?.[1].split(":")[0] ?? key;
    p3.append(element("details", { class: "beyond" }, element("summary", {}, `${count(list.length)} it answers where tddlog's run: ${what}`),
      element("ul", { class: "insts" }, ...list.map(([, suite, inst]) => element("li", {}, element("a", { href: instanceHref(suite, inst) }, inst))))));
  }
  if (unexp.length) {
    p3.append(fold(`What tddlog answers that ${name} cannot be given, by type`, ...unexp.flatMap(([cat, [r, m]], i) => [i ? "; " : "",
      `${data.categories.find((c) => c.key === cat)?.title ?? cat}: `, [r ? `${count(r)} it cannot express` : "", m ? `${count(m)} it was not given` : ""]
        .filter(Boolean).join(", ")]), "."));
  }
  const census = draft.views.census.sources.filter((src) => src.system === x);
  p3.append(element("h4", {}, `Programs of ${name}'s own that tddlog does not run`));
  if (census.length) p3.append(...census.map((src) => censusSource(src, false, true)));
  else p3.append(note(`The census has no source of ${name}'s; its programs are the suites'.`));
  p3.append(element("p", { class: "small" }, "Every system's: ", element("a", { href: "#/coverage" }, COVERAGE), "."));
  out.push(p3);
  // how it was run
  parts.facts.append(element("dt", {}, "Counted"), element("dd", {}, measureLine()),
    element("dt", {}, "Commands"), element("dd", {}, "Each instance's page shows the commands each system ran and the files it read."));
  out.push(element("section", { class: "panel", id: "run" }, element("h3", {}, "How it was run"), element("div", { class: "facts" }, parts.facts),
    lazy({ class: "given" }, "How it was given each suite's instances", () => [parts.given])));
  return out;
}

// ------------------------------------------------------------- by integration

// The integrations: tddlog alone, and each pipeline, by the instances it was given and answers.
function integrationsView() {
  // results from before the integrations had their own comparisons: the pipelines are a type of program there
  if (!data.integrations) {
    const bit = data.categories.find((c) => c.mine.some((m) => m !== "tddlog") && c.key !== ALL);
    return [crumbs(HOME, [sectionName("integrations")]), element("h2", {}, sectionName("integrations")),
      element("p", { class: "lede" }, "tddlog alone is compared in each type of program",
        ...(bit ? ["; Alea.jl+tddlog and Roulette+tddlog in ", link({ view: "category", category: bit.key }, bit.title)] : []), ".")];
  }
  const body = element("tbody");
  for (const k of integrationsOf()) {
    const i = data.integrations[k], alone = !i.system;
    body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "integration", key: k }, i.title)),
      element("td", { class: "about" }, alone ? "its own front ends: ProbLog, LPADs, Dice, and the networks' formats"
        : `${data.names[i.system]}'s own code, up to what it hands tddlog`),
      element("td", { class: "num" }, count(alone ? i.given : i.given[i.pipeline])),
      element("td", { class: "num" }, count(alone ? i.answers : i.answers[i.pipeline])),
      element("td", { class: "num" }, alone ? "" : count(i.answers[i.system]))));
  }
  return [crumbs(HOME, [sectionName("integrations")]), element("h2", {}, sectionName("integrations")),
    element("p", { class: "lede" }, "tddlog with its own front ends, and tddlog doing the inference after another system's own code."),
    scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "integration"), element("th", { scope: "col" }, "what reads the program"),
      element("th", { scope: "col", class: "num" }, "given"), element("th", { scope: "col", class: "num" }, "answers"),
      element("th", { scope: "col", class: "num", title: "the system alone, on the same instances" }, "the system alone answers"))),
    body)), "whole")];
}

// tddlog alone (its answers in each type, which open the type), or X+tddlog
// against X on X's programs (an integration of categories.json, drawn as a
// category is): the comparison, its plots and the families.
function integrationView(route) {
  const k = route.key, i = data.integrations[k];
  const out = [crumbs(HOME, inSection("integrations"), [i.title])];
  if (!i.system) {
    const body = element("tbody");
    for (const t of i.types) {
      body.append(element("tr", {}, element("th", { scope: "row" }, link({ view: "category", category: t.key }, t.title)),
        element("td", { class: "num" }, count(t.instances)), element("td", { class: "num" }, count(t.given)), element("td", { class: "num" }, count(t.answers))));
    }
    out.push(element("h2", { class: "solver" }, swatch("tddlog"), i.title),
      element("p", { class: "lede" }, "tddlog reading each program with its own front ends: ProbLog, LPADs, Dice and the network formats."),
      scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
        element("th", { scope: "col" }, "type of program"), element("th", { scope: "col", class: "num" }, "instances"),
        element("th", { scope: "col", class: "num" }, "given"), element("th", { scope: "col", class: "num" }, "answers"))), body)), "whole"));
    return out;
  }
  const x = i.system, me = i.pipeline, name = data.names[x], v = i.vs[x];
  const rows = catRows(i).map((r) => ({ ...r, me }));
  out.push(element("h2", { class: "solver", title: i.about ?? null }, swatch(me), `${i.title} against ${name}`),
    note(`${name}'s own code, then tddlog's inference; the time includes ${name}'s after its warm-up.`),
    versusCells(v, name, i.instances), note(measureLine()), pairPlot(rows, x, v, `int-${k}`));
  // the families
  const fams = element("tbody");
  for (const g of i.groups) {
    const rs = g.rows.map((j) => rows[j]);
    fams.append(element("tr", {}, element("th", { scope: "row", title: g.about ?? null }, g.title), codeCell(rs),
      element("td", { class: "num" }, count(rs.length)), element("td", { class: "num" }, count(rs.filter((r) => r.t[me] !== null).length)),
      element("td", { class: "num" }, count(rs.filter((r) => r.t[x] !== null).length))));
  }
  out.push(element("h3", {}, "Families"), scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
    element("th", { scope: "col" }, "family"), element("th", { scope: "col", title: "the family's programs, each system's, and its runs" }, "program"),
    element("th", { scope: "col", class: "num" }, "instances"), element("th", { scope: "col", class: "num" }, `${i.title} answers`), element("th", { scope: "col", class: "num" }, `${name} answers`))), fams)), "whole"));
  return out;
}

// ------------------------------------------------------------- by program type

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
    ["translated", "Translated", "smb", "translated", "one of them, or both, read a translation (for the system, for tddlog, or for both)"]]) {
    const have = xs.filter((x) => n(x, keep) > 0);
    const body = element("tbody");
    for (const x of have) {
      const w = v[field][x];
      const num = (text) => element("td", { class: "num" }, text);
      const dirs = keep === "h" ? [] : ["s", "m", "b"].map((k) => num(count(n(x, k))));
      body.append(element("tr", { class: x === route.vs ? "vs" : null }, element("th", { scope: "row" }, systemLink(x)),
        num(count(n(x, keep))), ...dirs,
        num(w ? count(w.answers) : ""), num(w ? count(w.tddlog_answers) : ""), num(w ? duration(w.total) : ""), num(w ? duration(w.tddlog_total) : ""),
        num(w ? speedup(w) : "")));
    }
    const sec = element("section", { class: "panel split", id }, element("h3", {}, title), note(`Where ${text}.`));
    if (have.length) {
      const col = (text, title) => element("th", { scope: "col", class: "num", title }, text);
      const par = `PAR-${data.measure.par}, over the instances the system is given and either answers`;
      sec.append(scroller(stacked(element("table", { class: "board" }, element("thead", {}, element("tr", {},
        element("th", { scope: "col" }, "system"), col("instances"),
        ...(keep === "h" ? [] : [col("for it", "a translation for the system"), col("for tddlog", "a translation for tddlog"),
          col("for both", "a translation of the original for both")]),
        col("it answers"), col("tddlog answers"), col("its total", par), col("tddlog's total", par), col("faster", "tddlog's speed-up"))), body)), "whole"));
      const x = have.includes(route.vs) ? route.vs : have[0];
      const rows = kindRows(c, x).filter((r) => keep.includes(r.kind));
      sec.append(element("p", { class: "small" }, `tddlog against ${data.names[x]} on ${plural(rows.length, "instance")}:`),
        pairPlot(rows, x, v[field][x], id));
    } else sec.append(note("No system here."));
    out.push(sec);
  }
  out.push(element("h3", { class: "every" }, "Every instance of the type, each system's answers over all of them"));
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
      element("td", { class: "about" }, c.key === ALL ? "every system" : element("div", { class: "chips" }, ...c.native.map((x) => element("span", { class: "chip" }, systemLink(x))))),
      element("td", { class: "num" }, count(kinds("h"))), element("td", { class: "num" }, count(xs.filter((x) => /[smb]/.test(v.kinds[x])).length))));
  }
  return [crumbs(HOME, [sectionName("types")]), element("h2", {}, sectionName("types")),
    element("p", { class: "lede" }, "Each instance counts in one type, that of the model it is; all programs is their union, with every system."),
    element("p", { class: "small" }, element("a", { href: "#/coverage" }, COVERAGE), ": from the systems' own tutorials, tests, papers and repositories, and why."),
    scroller(stacked(element("table", { class: "board types-t" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "type"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num", title: "by each instance's subject: tddlog, or the pipeline that ran it" }, "tddlog answers"),
      element("th", { scope: "col" }, "native systems"),
      element("th", { scope: "col", class: "num", title: "the systems compared head-to-head on some instance" }, "head-to-head"),
      element("th", { scope: "col", class: "num", title: "the systems compared through a translation on some instance" }, "translated"))),
    body)), "whole"),
    element("div", { class: "fig-head" }, element("h3", {}, "Each type against the systems native to it"),
      element("span", { class: "dots" }, element("i", { class: "me" }), "tddlog", element("i", { class: "you" }), "a native system")),
    overview()];
}

// ------------------------------------------------------------- coverage

// A source of the census: what it holds, how many programs and how many
// tddlog runs, each in a cell of its own, and why it does not run the others
// (folded on the census's page, open on a baseline's, where it is the point).
function censusSource(src, tool = false, open = false) {
  const n = (x) => (typeof x === "number" ? count(x) : x);
  const rows = src.items.map((it) => element("tr", {}, element("td", {}, element("span", { class: `cls cls-${it.class.replace(/ /g, "-")}` }, it.class)),
    element("td", { class: "num" }, n(it.n)), element("td", { class: "about" }, it.programs), element("td", { class: "about" }, it.why)));
  const table = () => [scroller(stacked(element("table", { class: "board census-t" }, element("thead", {}, element("tr", {},
    element("th", { scope: "col" }, "why"), element("th", { scope: "col", class: "num" }, "programs"), element("th", { scope: "col" }, "which"),
    element("th", { scope: "col" }, "what stops tddlog"))), element("tbody", {}, ...rows))), "whole")];
  // counts in cells; a source counted in words says so in its line
  const counted = typeof src.programs === "number" && typeof src.runs === "number";
  return element("div", { class: "census" },
    element("p", {}, element("b", {}, src.source), ...(tool ? [" (", systemLink(src.system), ")"] : []), `: ${src.what}.`,
      ...(counted ? [] : [" ", element("span", { class: "muted" }, `${n(src.programs)}; tddlog runs ${n(src.runs)}.`)])),
    ...(counted ? [cells(["programs", count(src.programs)], ["tddlog runs", count(src.runs)], ["tddlog does not run", count(src.programs - src.runs)])] : []),
    ...(!rows.length ? [] : open ? table() : [lazy({ class: "census-fold" }, `Why tddlog does not run the others: ${plural(rows.length, "kind")}`, table)]));
}

function coverageView() {
  const cen = draft.views.census;
  const out = [crumbs(HOME, inSection("types"), [COVERAGE]), element("h2", {}, COVERAGE),
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
    body.append(element("tr", {}, element("th", { scope: "row" }, systemLink(s.key)),
      element("td", { class: "num" }, count(v.beyond.length)), ...(refusals ? [element("td", { class: "num" }, count(refused))] : []),
      element("td", { class: "num" }, count(missing))));
  }
  out.push(element("h3", {}, "In the release run"), note(`Over the ${inWords(data.types.length)} types: what a system answers that tddlog does not, `
    + "and what tddlog answers that the system cannot express or was not given."),
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
  // the sources of programs of no type (the circuits), after them
  const rest = cen.sources.filter((src) => !data.types.some((c) => c.key === src.type));
  if (rest.length) out.push(element("h4", {}, "Programs of no type"), ...rest.map((src) => censusSource(src, true)));
  return out;
}

// ------------------------------------------------------------- how it works

// The stages of a run, the steps of its trace (trace.js names them, and
// these are its titles), and the six of `solve` between reading the program
// and its answers (docs/design.md, "How a program is answered"): what each
// does, and where the code is.
const STAGES = [
  ["read", "a ProbLog file, a network or a Dice program, as a program", "src/input.rs"],
  ["parse", "the program's clauses, queries and evidence", "src/syntax"],
  ["ground", "rules without variables over the atoms the queries and evidence need", "src/ground"],
  ["rewrite", "linear recursion written without recursion; indicators", "src/ground"],
  ["layout", "a vtree over the facts, by auto's trials", "src/vtree"],
  ["compile", "a TDD per derived atom, component by component", "src/tp"],
  ["evidence", "weighted model counts of the queries against the evidence", "src/query"],
  ["answers", "the probabilities, in the program's order", "src/solve"],
];

// A walkthrough's steps as a strip of the stages' names, one line, each
// opening its step of the walkthrough, the step on show marked.
function stepper(w, steps, on) {
  return element("nav", { class: "pipe", "aria-label": "The stages of a run" }, element("ol", { class: "pipeline" },
    ...steps.map((t, k) => element("li", {}, element("a", { href: `#/how/${w.key}/${k + 1}`, "aria-current": k === on ? "step" : null,
      title: STAGES.find(([key]) => key === t.key)?.[1] ?? null }, `${k + 1}. ${t.title}`)))));
}

// The walkthroughs (each titled by results.js's WALK_TITLES, which the menu
// lists): a run of tddlog on each program, its trace
// (walk/KEY.json, written by the command shown, `tddlog --trace`) drawn by
// trace.js a step per stage, and a caption by hand over each step. Each
// fact a caption states is the trace's or docs/design.md's.
const WALKS = [
  {
    key: "reach", what: "Recursion, written without recursion before the compile; the fixpoint where a cycle stays.",
    file: "examples/cycle_graph.pl", example: "reachability", evidence: false,
    command: "tddlog --trace reach.json examples/cycle_graph.pl",
    lede: "The probability that node 1 reaches each node of a graph with cycles, its 14 edges each with a probability of its own. The recursion is linear, so tddlog writes it without recursion.",
    captions: {
      read: "`tddlog::read` takes the bytes and their format. A ProbLog file is its own program, so there is no translation to show.",
      parse: "The 14 edges as probabilistic facts, the two rules of `path/2`, the second reading `path` itself, and six queries `path(1,Y)`.",
      ground: "Rules without variables over the atoms the queries need, paths from node 1 alone: one cyclic component of six atoms, round the graph.",
      rewrite: "Reachability is written without recursion, its atoms eliminated one at a time as in Floyd and Warshall's algorithm (src/ground/eliminate.rs): eight acyclic components.",
      layout: "`auto` gives a recursive program the `dynamic` trial, which finished in its first attempt: a vtree right-linear over a min-fill order of the 12 edges.",
      compile: "Each atom's formula over the edges, in dependency order, by TDD operations; the largest, `path(1,6)`, has 79 pairs. No component is cyclic, so no fixpoint runs.",
      evidence: "No evidence: each query was counted during the compile, once nothing read its formula.",
      answers: "What `tddlog examples/cycle_graph.pl` prints, and ProbLog 2.3.0's answers (tests/it/examples.rs).",
    },
  },
  {
    key: "gears", what: "Atoms that support only each other are false; the loop left is written without recursion.",
    file: "examples/gearwheels.pl", example: "gears", evidence: false,
    command: "tddlog --trace gears.json examples/gearwheels.pl",
    lede: "Two gear wheels that turn each other and a button for each (Bogaerts and Van den Broeck 2015, Example 4.13): not stratified, so its meaning is its well-founded model.",
    captions: {
      read: "A ProbLog file is its own program, so there is no translation to show.",
      parse: "10 clauses: the two buttons as probabilistic facts and 8 rules, two reading `turns1(0)` and `turns2(0)` under negation; three queries.",
      ground: "At time 0, `turns1(0)` and `turns2(0)` support only each other, so both are false and the rules reading them simplify; `turns1(1)` and `turns2(1)` form one cycle.",
      rewrite: "Each rule reads one atom of the cycle, so it is written without recursion: `turns1(1)` reads the two buttons, `turns2(1)` its button and `turns1(1)`.",
      layout: "`auto` gives this program the `dynamic` trial, and it finished in its first attempt. The vtree's two leaves are the buttons.",
      compile: "`turns1(1)` first, then `turns2(1)`, which reads it, each from its rules' bodies by TDD operations, 3 pairs each. No component is cyclic, so no fixpoint runs.",
      evidence: "No evidence: the queries were counted during the compile, and `turns1(0)` is false.",
      answers: "0 at time 0, and 1 − 0.7 × 0.4 = 0.72 for each wheel at time 1: ProbLog 2.3.0's answers (tests/it/examples.rs).",
    },
  },
  {
    key: "network", what: "A network file read as a program: its tables, its families and the evidence.",
    file: "examples/car.net", example: "car", evidence: true,
    command: "tddlog --trace network.json --evidence-probability --evidence examples/car.inst --query fuel,plugs,starter examples/car.net",
    lede: "A car that does not start on a cold morning, a Hugin network of 13 variables: the headlights are bright, the fuel gauge low (examples/car.inst). Is it the fuel, the plugs or the starter?",
    captions: {
      read: "The network and its evidence are read as one program, `tddlog --program`'s: each row of a table an annotated disjunction over the values it gives a chance. It is examples/bn_car_evidence.pl.",
      parse: "42 clauses: 31 annotated disjunctions and 11 rules, the rows certain of one value (`cranks(no) :- battery(flat), starter(faulty)`); four observations and seven queries.",
      ground: "A choice among n values becomes n − 1 switches, each given the ones before (`'$swbattery_3_1'` at 0.4/0.8 = 0.5). `weather(cold)` is fixed, its 0.3 a factor of P(e): 24 components.",
      rewrite: "`families:0` reads each atom another family reads through a copy (`'$v'`, 29 copies of 21 atoms), tied to it by an observed link (`'$c'`): 16 observations, 3 of the network's and 13 links.",
      layout: "`auto`'s `families:0` trial finished in its first attempt: 23 families in 12 groups, laid out by a tree decomposition of width 2; 65 leaves, 36 switches and 29 copies.",
      compile: "37 components of one atom, none cyclic, each stored as its negation (the dual). The links hold the tables' rows; the largest, `cranks(yes)` as `starts(no)` reads it, has 106 pairs.",
      evidence: "The 16 observations are conjoined along the vtree into 438 pairs, whose count is P(e) = 0.0226. Each query is counted against it: P(q | e) = WMC(q ∧ e) / WMC(e).",
      answers: "An empty tank goes from 0.10 to 0.29, fouled plugs from 0.20 on a cold day to 0.47, a faulty starter from 0.03 to 0.10: a sum over the worlds' answers and ProbLog's (tests/it/networks.rs).",
    },
  },
  {
    key: "markov", what: "A UAI Markov network: its factors as observed atoms, its colour symmetry broken, Z from P(e).",
    file: "tests/regress/potts.uai", example: null, evidence: false,
    command: "tddlog --trace markov.json --no-queries --evidence-probability tests/regress/potts.uai",
    lede: "A UAI Markov network: five variables of three values and six Potts factors over pairs, a weighted graph colouring. Asked nothing (`--no-queries`), tddlog answers Z.",
    captions: {
      read: "Each variable an annotated disjunction, each factor an observed atom `fJ(ok)` with a rule per nonzero entry; fixing `x0` breaks the colours' symmetry, a third of Z left (docs/design.md).",
      parse: "55 clauses: the fact `x0(v_0)`, four annotated disjunctions, 50 rules (19 of them probabilistic) and seven observations, `f0` to `f6`.",
      ground: "The observations fix `x0` and `x2` before any diagram is built, and rows of one weight share a class atom: 41 rules over 42 atoms become 33 over 29, none cyclic.",
      rewrite: "`families:0` reads each atom another family reads through a copy (`'$v'`, 22 copies of 13 atoms), tied to it by an observed link (`'$c'`).",
      layout: "`auto`'s `families:0` trial finished in its first attempt: 14 families in 10 groups, laid out by a tree decomposition of width 2; 32 leaves, 10 switches and 22 copies.",
      compile: "27 components of one atom, in dependency order, each stored as its negation (the dual). The largest, `f3(ok)` at 34 pairs, is the one factor nothing fixed.",
      evidence: "The 14 observations are conjoined along the vtree, each variable summed out once nothing later reads it: 11 summing joins, one pair. P(e) = 5/81, about 0.0617.",
      answers: "Z = P(e) × 10^c, c the offset of the program's first line: 112.5, the 225/2 that tests/it/networks.rs takes from an enumeration of the worlds.",
    },
  },
  {
    key: "dice", what: "A probabilistic functional program: integers held as bits, a derived atom per bit and step.",
    file: "tests/dice/walk.dice", example: "walk", evidence: true,
    command: "tddlog --trace dice.json --evidence-probability tests/dice/walk.dice",
    lede: "A random walk on 0..7 in Dice: ten steps of plus or minus one from 3, absorbed at either end, observed not to end at 0. tddlog reads Dice itself, an integer as bits.",
    captions: {
      read: "The Dice front end evaluates the program symbolically into Datalog: `x` three bits per step, each step's flip `0.4::x_c_3(T)`, and a query `x(V)` per value.",
      parse: "44 clauses: the flip as a probabilistic rule, the start as facts, and 40 rules; eight queries and the observation `obs`.",
      ground: "The steps are ground level by level, constants proved as each completes, and equal gates merged (`swept`: proved equal by a SAT solver): 215 atoms become 90.",
      rewrite: "No recursion is left once the steps are ground, and `closed` reads no indicators: the program compiles as ground.",
      layout: "`closed` is `auto`'s first trial for few facts with many derived atoms per fact (here 10 and 80): new facts on a right-linear spine, over the ten flips a chain.",
      compile: "Each of the 80 gates is a component, compiled in dependency order a step after the other; the formulas grow with the steps, up to 59 pairs.",
      evidence: "The observation is conjoined into 41 pairs, whose count is P(e) = 0.432898, the probability that the walk does not end at 0.",
      answers: "Ten steps from 3 end on an odd number unless absorbed at 0, which is observed away, so the even values have probability 0. Dice's answers are the same (tests/it/examples.rs).",
    },
  },
  {
    key: "circuit", what: "The and-inverter graph of Alea.jl's circuit, optimized, then compiled gate by gate in dependency order.",
    file: "examples/alea_dice.pl", example: "circuit", evidence: true,
    command: "tddlog --trace circuit.json --evidence-probability examples/alea_dice.pl",
    lede: "Two dice, the first at least 4, as the circuit Alea.jl builds (8 flips, 78 gates), in ProbLog. The queries: the sum is 7 (`q1`), the first shows more (`q2`).",
    captions: {
      read: "A probabilistic fact per flip, `f1` to `f8`; an and-gate a rule, an or-gate a rule per input, a negated input written `\\+`.",
      parse: "119 clauses: the 8 flips and 111 rules; two queries and one observation.",
      ground: "The gates are optimized as an and-inverter graph before any diagram: 24 views inlined, 19 atoms merged, 4 proved equal by SAT sweeping; 88 atoms and 111 rules become 40 and 49.",
      rewrite: "The `program` layout keeps the views in place, the 81 derived atoms as written, and names `g76`, which crosses the one narrow cut, through a copy and an observed link.",
      layout: "`auto`'s first trial is `program`: a vtree right-linear over the flips, an OBDD in the order Alea.jl's BDD package built the circuit in; 9 leaves, the 8 flips and the copy.",
      compile: "81 components, a derived atom each, bottom-up in dependency order, each gate once. Each is stored as its negation (the dual), so the trace counts no disjunctions; the largest has 60 pairs.",
      evidence: "The observation and the link are conjoined into 40 pairs: P(e) = 0.5, the first die at least 4. Both queries are counted against it.",
      answers: "1/6 and 2/3, the answers the two dice give by hand (tests/it/examples.rs).",
    },
  },
];

for (const w of WALKS) {
  w.title = WALK_TITLES.find(([key]) => key === w.key)[1];
  if (!w.command.endsWith(` ${w.file}`)) throw new Error(`walk ${w.key}: its command does not end with its file`);
}
if (WALKS.length !== WALK_TITLES.length) throw new Error("a walkthrough without its title, or a title without its walkthrough");

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

// How it works: the walkthroughs, the one on show (the first where the
// address names none) at its step, the strip of the stages above it.
function howView(route) {
  const w = WALKS.find((x) => x.key === route.walk) ?? WALKS[0];
  const out = [crumbs(HOME, ...(route.walk ? [inSection("how"), [w.title]] : [[sectionName("how")]])),
    element("h2", {}, sectionName("how")),
    element("p", { class: "lede one" }, "A run of tddlog a stage at a time, from its trace (", element("code", {}, "tddlog --trace"), "); ",
      element("a", { href: `${BRANCH}main/docs/design.md#how-a-program-is-answered` }, "docs/design.md"), " describes each stage."),
    element("nav", { class: "walks", "aria-label": "Walkthroughs" }, ...WALKS.map((x) =>
      element("a", { href: `#/how/${x.key}`, title: x.what, "aria-current": x === w ? "page" : null }, x.title)))];
  const walk = element("div", { class: "walk" }, note("Loading the trace…"));
  out.push(walk, counted());
  walkTrace(w.key).then((trace) => {
    const steps = tddlogTrace.steps(trace);
    const i = Math.min(Math.max(route.step ?? 0, 0), steps.length - 1);
    const s = steps[i];
    const stage = STAGES.find(([key]) => key === s.key);
    const head = element("div", { class: "walk-head" }, caption(w.lede, "lede"),
      element("p", { class: "small run" }, element("code", {}, w.command.slice(0, -w.file.length),
        element("a", { href: `${BRANCH}main/${w.file}` }, w.file)),
        ...(w.example ? [element("a", { href: playgroundHref(w), class: "try-btn" }, "Open in playground")] : [])));
    const body = element("div", { class: "step" }, element("h3", {}, `${i + 1}. ${s.title}`,
      ...(stage ? [element("span", { class: "muted small" }, `${stage[1]}, in `, element("code", {}, stage[2]))] : [])));
    if (w.captions[s.key]) body.append(caption(w.captions[s.key]));
    const drawn = element("div", { class: "walk-trace" });
    drawn.innerHTML = tddlogTrace.stepHtml(s);
    body.append(drawn);
    const nav = element("p", { class: "step-nav" }, i > 0 ? element("a", { href: `#/how/${w.key}/${i}` }, "← Back") : "",
      i < steps.length - 1 ? element("a", { href: `#/how/${w.key}/${i + 2}` }, "Next →") : "");
    walk.replaceChildren(stepper(w, steps, i), head, body, nav);
  }, (e) => walk.replaceChildren(note(`The trace did not load: ${e.message}.`)));
  return out;
}

// How the results are counted (rule 16), the one place that says it whole:
// under a table, the views say it in one line (measureLine).
function counted() {
  const { floor, par, common } = data.measure;
  const items = [["answers", "Over all of a type's instances: a run out of time or memory, wrong, refused or not given counts as unanswered."],
    ["total", `Over the instances a system is given and either answers: an unanswered run counts as ${twice(par)} its time limit `
      + `(PAR-${par}), a time under ${floor} s as ${floor} s.`],
    ["speed-up", `The ratio of the totals over the instances both answer (not both under ${floor} s), with those only the other `
      + "answers, tddlog at its limit, and those only tddlog answers, the other at its limit where that bound is at least the "
      + `measured ratio; none with fewer than ${common} in common.`],
    ["X+tddlog", "X's own code, then tddlog's inference; its time includes X's after its warm-up, so that against X it differs only in the inference."]];
  return element("section", { class: "counted" }, element("h3", {}, "How the results are counted"),
    element("dl", { class: "classes" }, ...items.flatMap(([k, text]) => [element("dt", {}, k), element("dd", {}, text)])));
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

const DRAFT_VIEWS = { "draft-home": draftHome, baselines: baselinesView, baseline: baselineView, integrations: integrationsView,
  integration: integrationView, types: typesView, coverage: coverageView, how: howView };

Object.assign(extend, {
  load: async () => {
    const response = await fetch(`next/views.json${stamp}`);
    if (!response.ok) throw new Error(`views.json: ${response.status}`);
    draft.views = await response.json();
  },
  parse: (hash) => {
    const parts = hash.replace(/^\//, "").split("/").filter(Boolean).map((p) => decodeURIComponent(p));
    if (!parts.length) return { view: "draft-home" };
    const [first, key, at] = parts;
    // a baseline's page (#/tools/... before), or, at a solver's address, the page of its integration where it is tddlog's
    const baseline = (x) => (draft.views?.systems?.[x] ? { view: "baseline", system: x } : null);
    if (first === "baselines" || first === "tools") {
      if (parts.length === 1) return { view: "baselines" };
      const route = baseline(key);
      if (route && parts.length === 2) return route;
      return route && parts.length === 3 && BASELINE_AT.some(([id]) => id === at) ? { ...route, at } : null;
    }
    if (first === "system" && parts.length === 2) {
      return isMine(key) ? (data.integrations?.[integrationOf(key)] ? { view: "integration", key: integrationOf(key) } : { view: "integrations" }) : baseline(key);
    }
    if (first === "integrations") {
      if (parts.length === 1) return { view: "integrations" };
      if (parts.length !== 2) return null;
      // an integration the data does not have yet opens their page
      return data.integrations?.[key] ? { view: "integration", key } : { view: "integrations" };
    }
    if (parts.length === 1 && (first === "types" || first === "coverage" || first === "how")) return { view: first };
    if (first === "how" && WALKS.some((w) => w.key === key)) {
      if (parts.length === 2) return { view: "how", walk: key, step: 0 };
      if (parts.length === 3 && /^[1-9]\d{0,2}$/.test(at)) return { view: "how", walk: key, step: Number(at) - 1 };
    }
    return null;
  },
  href: (route) => ({ "draft-home": "#/", baselines: "#/baselines",
    baseline: `#/baselines/${encodeURIComponent(route.system)}${route.at ? `/${route.at}` : ""}`,
    integrations: "#/integrations", integration: `#/integrations/${route.key}`, types: "#/types", coverage: "#/coverage",
    how: route.walk ? `#/how/${route.walk}${route.step ? `/${route.step + 1}` : ""}` : "#/how" })[route.view] ?? null,
  draw: (route) => {
    if (!(route.view in DRAFT_VIEWS)) return null;
    const stale = staleViews();
    if (stale.length && route.view !== "draft-home" && route.view !== "how" && !route.view.startsWith("integration")) return [staleNote(stale)];
    return DRAFT_VIEWS[route.view](route);
  },
  title: (route) => ({ "draft-home": null, baselines: sectionName("baselines"), baseline: `tddlog against ${data.names[route.system]}`,
    integrations: sectionName("integrations"), integration: data.integrations?.[route.key]?.title, types: sectionName("types"),
    coverage: COVERAGE, how: route.walk ? `${sectionName("how")}: ${WALKS.find((w) => w.key === route.walk).title}` : sectionName("how") })[route.view] ?? null,
  category: (c, route, rows) => (staleViews().length ? [staleNote(staleViews())] : typeSplit(c, route, rows)),
  // a translator's or generator's name in a line on how a file was made, as a link to its source
  made: (html) => html.replace(/<code>([\w-]+\.(?:py|jl|rkt))(\b[^<]*)<\/code>/g, (whole, name, rest) =>
    (draft.views?.scripts.includes(name) ? `<a href="${BRANCH}baselines/scripts/${name}"><code>${name}${rest}</code></a>` : whole)),
});
