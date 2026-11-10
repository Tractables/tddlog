"use strict";

// The site's front page (index.html): its sections, as the bar at the top
// of every page and the menu name them (results.js's SECTIONS), and how a
// program is answered. It is the results page's app (results.js, loaded
// before this script, whose helpers and data it uses) with these views
// added through its `extend`:
//   #/                     the home page: what tddlog is, each type of program against its best baselines, the integrations
//   #/baselines            by baseline: every other system, what it is native to and what it answers
//   #/baselines/SYSTEM[/TAB]
//                          tddlog against SYSTEM, a tab on show: its idiomatic programs (by family), its
//                          original programs (`original`), each type of program (`types`), coverage
//                          (`coverage`), and how it was run (`run`)
//   #/integrations         by integration: tddlog alone, and in a pipeline after another system's own code
//   #/integrations/KEY     one of them: tddlog alone's answers by type; X+tddlog against X on X's programs
//   #/types                by program type: results.js's opening view, the README's figures, each a link to its type
//   #/circuits             the circuits Alea.jl exports, every system on them (results.js's view of a category)
//   #/coverage             the programs each system's sources hold that tddlog does not run, and why
//   #/how[/WALK[/N]]       how a program is answered: a run's stages, a walkthrough at its step N
//   #/methods              how the runs were made and the results counted, a table
// #/tools/... and #/system/... are the addresses these pages once had. A
// type of program is results.js's view (#/category/C), with its comparisons
// head-to-head and translated before the rest; a family and an
// instance are results.js's views as they are.
//
// The data is views.json (written by the results' processing): each system's kind
// of comparison on each instance of each type (head-to-head, translated for
// it, for tddlog or for both, or a translation not idiomatic for it, out of
// a type's comparisons and in all programs'), and the comparisons over each kind, which
// are the processing's, and the census of the programs tddlog does not
// run; and categories.json's integrations. A system is
// compared on each instance with tddlog or one of its pipelines (views.json's
// mine): in a type, with the instance's subject (the type's with); on the
// system's own page, with its own pipeline where that ran (Roulette+tddlog
// for Roulette; the system's kinds and with). The plots draw that member's
// times. The page counts and draws; it computes no total or speed-up.

const BRANCH = "https://github.com/Tractables/tddlog/blob/";
const draft = { views: null };

// ------------------------------------------------------------- the data

// The rows of category c (catRows) that system x was given, each with its
// index in the category (i), the kind of their comparison (h, s, m, b, u or
// x) and `me`, the member of mine it is with: as the type compares them, or
// with own as x's page does.
function kindRows(c, x, own = false) {
  const v = own ? draft.views.systems[x] : draft.views.categories[c.key];
  const kinds = own ? v?.kinds?.[c.key] : v?.kinds?.[x];
  if (!kinds) return [];
  const ms = own ? v.with[c.key] : v.with;
  return catRows(c).map((r, i) => [r, kinds[i], ms[i], i]).filter(([, kind]) => kind !== "-")
    .map(([r, kind, m, i]) => ({ ...r, i, kind, me: draft.views.mine[Number(m)] }));
}


// The types (not all programs) a system was given.
const typesOf = (x) => data.types.filter((c) => c.systems.includes(x));

// A system's rows over the types, as its page compares them, each with its category.
function systemRows(x) {
  return typesOf(x).flatMap((c) => kindRows(c, x, true).map((r) => ({ ...r, c })));
}

// The types a system natively handles (as the data's
// categories name them).
const nativeTo = (x) => data.types.filter((c) => c.native.includes(x));

// An instance's address in the results app.
const instanceHref = (suite, name) => `#/${suite}/@${encodeURIComponent(name)}`;

// A script's source on the branch that holds it.
function scriptLink(name, path) {
  return path ? element("a", { href: `${BRANCH}baselines/${path}` }, element("code", {}, name)) : element("code", {}, name);
}

// ------------------------------------------------------------- the parts

// tddlog's speed-up in a table's cell: a dash, its title saying why, where there is nothing to
// measure (no instance both answer over the floor and none only one answers).
const speedCell = (v) => (v?.speedup == null
  ? element("span", { class: "few", title: `nothing to measure: no instance both answer over ${data.measure.floor} s and none only one answers` }, "–") : speedup(v));

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
    side(me, ["answers", count(v.tddlog_answers)], [`total (PAR-${data.measure.par})`, duration(v.tddlog_total), over], [speedName("speed-up"), speedup(v), speedup(v, true)]));
}

// The cactus plot of x and the members it is with over rows (kindRows' me,
// drawn as one line), beside the scatter plot of their times; the speed-up
// is in the cells above it (versusCells), once.
function pairPlot(rows, x, v, key) {
  const name = data.names[x], me = subject(v?.me);
  const both = { key: `${x}-${key}`, title: `${name} and ${me}`, noun: "instances", systems: ["tddlog", x],
    instances: rows.map((r) => [r.name, r.v[r.me], r.v[x]]), limit: Math.max(...rows.map((r) => r.limit)) };
  return element("div", { class: "pair" }, plotted(both, { ...data.names, tddlog: me }), scatter(rows, ["tddlog", x], x, false, (r) => r.me));
}

// The families rows are of, by how many each, each with its one link, its
// program's, which opens the family's view in its type on it; past the first
// eight, the rest folded.
const FAMILIES_SHOWN = 8;
function familyCounts(rows) {
  const list = familiesOfRows(rows).sort((a, b) => b.rows.length - a.rows.length);
  const table = (items) => scroller(stacked(element("table", { class: "board fams-t" }, element("thead", {}, element("tr", {},
    element("th", { scope: "col" }, "family"), element("th", { scope: "col", title: "Opens the family's view on its program" }, "program"),
    element("th", { scope: "col", class: "num" }, "instances"))),
  element("tbody", {}, ...items.map(({ c, g, rows: rs }) => element("tr", {},
    element("th", { scope: "row", title: g?.about ?? null }, g ? g.title : "another suite"),
    g ? programCell(c, g) : element("td"), element("td", { class: "num" }, count(rs.length))))))), "whole");
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
      `${speedup(i.vs[i.system], true)}, on the ${plural(i.instances, "instance")} ${data.names[i.system]} reads.`);
  });
  return [
    element("section", { class: "intro" },
      element("p", { class: "big" }, "tddlog computes exact probabilities for probabilistic logic programs, probabilistic databases, "
        + "Bayesian and Markov networks, and probabilistic functional programs, by compiling them into tree decision diagrams."),
      element("p", { class: "lede" }, `It answers ${count(all.answers.tddlog ?? 0)} of ${count(all.instances)} instances of `
        + `${inWords(data.types.length)} types of program, against ${count(others)} baselines. `,
        element("a", { href: "playground/" }, "The playground"), " runs it in the browser; ", element("a", { href: "guide/#getting-started" }, "the guide"), " installs it."),
      stampLine()),
    element("h2", { class: "home-h" }, "Each type of program against its best baselines"),
    scroller(stacked(element("table", { class: "board home-t" }, element("thead", {}, element("tr", {},
      element("th", { scope: "col" }, "type of program"), element("th", { scope: "col", class: "num" }, "instances"),
      element("th", { scope: "col", class: "num" }, "tddlog answers"),
      element("th", { scope: "col", title: "the baselines that answer most, each with tddlog's speed-up over it" }, "its best baselines, and tddlog's ", speedName("speed-up")))),
    body)), "whole"),
    ...(lines.length ? [element("h2", { class: "home-h" }, "tddlog as another system's backend"), element("ul", { class: "home-int" }, ...lines)] : []),
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

// A list of nodes in prose: "A", "A and B", "A, B and C".
const andNodes = (items) => items.flatMap((x, i) => [i === 0 ? "" : i === items.length - 1 ? " and " : ", ", x]);

// One line on what the two sides read on a baseline's idiomatic instances
// (views.json's read): the scripts that wrote their translations, and where
// each family's are shown.
function readLine(read) {
  const by = new Map([...read.system.by, ...read.mine.by]);
  const other = read.system.other || read.mine.other;
  if (!by.size && !other) return note("Both read the programs as their sources wrote them. Each family's page shows them.");
  const scripts = [...by].map(([n, path]) => scriptLink(n, path));
  return note(...(by.size ? ["Translations by ", ...andNodes(other ? [...scripts, "others"] : scripts)] : ["Some of the programs are translations"]),
    ". Each family's page shows what each side read.");
}

// A baseline's idiomatic instances by family (views.json's families, as
// the types name them, most first; those of few instances in one row, their
// names its title): the instances, each side's answers and total, and
// tddlog's speed-up; a family's one link, its program's, opens its view in its
// type on its program. Past the first FAMILY_ROWS, the rest
// show on asking. Under it, one line on what each
// side read.
const FAMILY_ROWS = 12;
function familyTable(v, name) {
  const me = subject(v.idiomatic.me), par = `over the instances ${name} is given and either answers (PAR-${data.measure.par})`;
  const num = (text) => element("td", { class: "num" }, text);
  const type = (c) => data.categories.find((y) => y.key === c);
  const group = ([c, key]) => type(c).groups.find((g) => g.key === key);
  // A title two types share (Mastermind) names its type too.
  const titles = v.families.filter((f) => f.family).map((f) => group(f.family).title);
  const title = (f) => titles.filter((t) => t === group(f).title).length > 1
    ? `${group(f).title} (${lower(type(f[0]).title)})` : group(f).title;
  const body = element("tbody", {}, ...v.families.map((f, i) => element("tr", { class: i < FAMILY_ROWS ? null : "more" },
    f.families ? element("th", { scope: "row", title: f.families.map((k) => group(k).title).join(", ") }, `other families (${f.families.length})`)
      : element("th", { scope: "row" }, title(f.family)),
    f.families ? element("td") : programCell(type(f.family[0]), group(f.family)),
    ...[count(f.n), count(f.vs.answers), duration(f.vs.total), count(f.vs.tddlog_answers), duration(f.vs.tddlog_total), speedCell(f.vs)].map(num))));
  const table = element("table", { class: `board fams-tr${v.families.length > FAMILY_ROWS ? " folded" : ""}` },
    head2(["family"], ["program", "Opens the family's view on its program, with the programs both read"], ["instances", null, "num"],
      { group: name, cols: [["answers", null, "num"], ["total", par, "num"]] },
      { group: me, cols: [["answers", null, "num"], ["total", par, "num"], [speedName("faster"), `${me}'s speed-up, a dash where there is nothing to measure`, "num"]] }), body);
  // past the first rows, the rest on asking
  const more = v.families.length > FAMILY_ROWS ? element("button", { type: "button", class: "plain more-rows", "aria-expanded": "false", onclick: () => {
    const open = table.classList.toggle("folded") === false;
    more.setAttribute("aria-expanded", String(open));
    more.textContent = open ? `Only the first ${FAMILY_ROWS} families` : `All ${count(v.families.length)} families`;
  } }, `All ${count(v.families.length)} families`) : null;
  return [scroller(stacked(table), "whole"), more,
    readLine(v.read)];
}

// The tabs of a baseline's page, by the ids a link names (#/baselines/X/TAB;
// the first, its every idiomatic program, at #/baselines/X): one on show at a
// time. ORIGINAL limits the page to its original programs.
const ORIGINAL = "original";
const BASELINE_TABS = [["idiomatic", "Idiomatic programs"], [ORIGINAL, "Original programs"], ["types", "In each type"],
  ["coverage", "Coverage"], ["run", "How it was run"]];
const IDIOMS_HREF = `${BRANCH}baselines/docs/baselines.md`;

// tddlog against one baseline, its numbers above a row of tabs: every
// idiomatic program it was given (its own language's, and the translations
// its users would write); its original programs, those its authors wrote and
// the others in its language that it read as their sources wrote them
// (views.json's originals); each type of program; coverage; how it was run.
function baselineView(route) {
  const x = route.system, s = data.systems.find((y) => y.key === x), v = draft.views.systems[x], name = data.names[x];
  const tab = route.tab ?? "idiomatic";
  const rows = systemRows(x).filter((r) => r.kind !== "x");
  const originals = new Set(Object.entries(v.originals ?? {}).flatMap(([c, is]) => is.map((i) => `${c}/${i}`)));
  const own = rows.filter((r) => originals.has(`${r.c.key}/${r.i}`));
  const site = s.link.replace(/^https?:\/\/(www\.)?/, "").replace(/\/$/, "");
  const page = { view: "baseline", system: x };
  const counts = { idiomatic: rows.length, [ORIGINAL]: own.length };
  const tabs = BASELINE_TABS.filter(([id]) => id !== ORIGINAL || own.length || tab === ORIGINAL);
  const out = [crumbs(HOME, inSection("baselines"), ...(tab === "idiomatic" ? [[name]]
    : [[name, page], [BASELINE_TABS.find(([id]) => id === tab)[1]]])),
    element("h2", { class: "solver" }, swatch(x), `tddlog against ${name}`),
    element("p", { class: "lede" }, `${s.about} `, element("a", { href: s.link }, site)),
    cells(["given", count(s.given), `instances of ${and(typesOf(x).map((c) => lower(c.title)))}`], ["answers", count(s.answers)],
      ["answers tddlog misses", count(v.beyond.length), `instances ${name} answers that tddlog does not`],
      ...(v.n.x ? [["not idiomatic", count(v.n.x), `translations a user of ${name}'s language would not write: in its answers, in no total`]] : [])),
    element("nav", { class: "tabs", "aria-label": `${name}'s page` }, ...tabs.map(([id, text]) => {
      const a = link(id === "idiomatic" ? page : { ...page, tab: id }, text, id in counts ? element("span", { class: "n" }, count(counts[id])) : null);
      if (id === tab) a.setAttribute("aria-current", "page");
      return a;
    }))];
  const panel = element("section", { class: "panel", id: tab });
  out.push(panel);
  if (tab === ORIGINAL) {
    panel.append(note(`Programs by ${name}'s authors, and others written in its language, read as written.`),
      versusCells(v.original, name, own.length), ...(own.length ? [pairPlot(own, x, v.original, "o"), familyCounts(own)] : []));
  } else if (tab === "idiomatic") {
    panel.append(note("Its own language's programs and the translations its users would write (",
      element("a", { href: IDIOMS_HREF }, "docs/baselines.md"), ", Idiomatic translations)."),
    versusCells(v.idiomatic, name, rows.length), ...(rows.length ? [pairPlot(rows, x, v.idiomatic, "i"), ...familyTable(v, name)] : []));
  } else if (tab === "types") {
    panel.append(...solverParts(x).types);
  } else if (tab === "coverage") {
    // what it answers that tddlog does not, what tddlog answers that it cannot be given, each in a cell;
    // then the programs of its own sources that tddlog does not run, and why
    const unexp = Object.entries(v.unexpressed).filter(([, [a, b]]) => a + b > 0);
    const [refused, missing] = unexp.reduce(([a, b], [, [c, d]]) => [a + c, b + d], [0, 0]);
    panel.append(cells(["it answers, tddlog does not", count(v.beyond.length), `instances ${name} answers that tddlog does not`],
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
      panel.append(element("details", { class: "beyond" }, element("summary", {}, `${count(list.length)} it answers where tddlog's run: ${what}`),
        element("ul", { class: "insts" }, ...list.map(([, suite, inst]) => element("li", {}, element("a", { href: instanceHref(suite, inst) }, inst))))));
    }
    if (unexp.length) {
      panel.append(fold(`What tddlog answers that ${name} cannot be given, by type`, ...unexp.flatMap(([cat, [r, m]], i) => [i ? "; " : "",
        `${data.categories.find((c) => c.key === cat)?.title ?? cat}: `, [r ? `${count(r)} it cannot express` : "", m ? `${count(m)} it was not given` : ""]
          .filter(Boolean).join(", ")]), "."));
    }
    const census = draft.views.census.sources.filter((src) => src.system === x);
    panel.append(element("h4", {}, `Programs of ${name}'s own that tddlog does not run`));
    if (census.length) panel.append(...census.map((src) => censusSource(src, false, true)));
    else panel.append(note(`The census has no source of ${name}'s; its programs are the suites'.`));
    panel.append(element("p", { class: "small" }, "Every system's: ", element("a", { href: "#/coverage" }, COVERAGE), "."));
  } else {
    const facts = solverParts(x).facts;
    facts.append(element("dt", {}, "Counted"), element("dd", {}, measureLine()),
      element("dt", {}, "Commands"), element("dd", {}, "Each instance's page shows the commands each system ran and the files it read."));
    panel.append(element("div", { class: "facts" }, facts));
  }
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
    scroller(stacked(element("table", { class: "board" }, head2(["integration"], ["what reads the program"], ["given", null, "num"],
      { group: "answers", cols: [["the integration", null, "num"], ["the system alone", "the system alone, on the same instances", "num"]] }),
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
  // plain tddlog, where it read its own program of some of the models (unanswered on the rest)
  const alone = i.systems.includes("tddlog");
  out.push(element("h2", { class: "solver", title: i.about ?? null }, swatch(me), `${i.title} against ${name}`),
    note(`${name}'s own code, then tddlog's inference; the time includes ${name}'s after its warm-up.`,
      ...(i.code ? [" The integration's code: ", element("a", { href: i.code }, i.code.replace("https://github.com/", ""))] : [])),
    versusCells(v, name, i.instances), note(measureLine()), integrationPlot(i, rows, `int-${k}`), ...whereWins(i, rows, x, me));
  if (alone) {
    const unread = i.classes.tddlog.unread;
    out.push(note(`Plain tddlog read another program of each model than ${name} did, with its own front ends`
      + (unread ? `; it counts as unanswered on the ${plural(unread, "instance")} it does not read.` : ".")));
  }
  out.push(...othersOf(i));
  // the families, as their types name them, under their types and domains, each with its one link, its program's
  const fams = element("tbody");
  const answers = [me, x, ...(alone ? ["tddlog"] : [])];
  for (const f of headings(familiesOfRows(rows))) {
    if (f.heading) {
      fams.append(headingRow(f, 3 + answers.length));
      continue;
    }
    fams.append(element("tr", {}, element("th", { scope: "row", title: f.g?.about ?? null }, f.g ? f.g.title : "another suite"),
      f.g ? programCell(f.c, f.g) : element("td"), element("td", { class: "num" }, count(f.rows.length)),
      ...answers.map((y) => element("td", { class: "num" }, count(f.rows.filter((r) => r.t[y] !== null).length)))));
  }
  out.push(element("h3", {}, "Families"), scroller(stacked(element("table", { class: "board" },
    head2(["family"], ["program", "Opens the family's view on its program"], ["instances", null, "num"],
      { group: "answers", cols: [[i.title, null, "num"], [name, null, "num"],
        ...(alone ? [["tddlog", "plain tddlog, on its own program of the model", "num"]] : [])] }), fams)), "whole"));
  return out;
}

// The families most named where an integration's system x is faster than
// the pipeline me.
const WINS_NAMED = 5;

// Where an integration's system x is faster than the pipeline me, in one
// line: its families by the instances x is faster on, as a scatter counts
// them (results.js's scatter: x answers and me does not, or me answers
// slower, an instance both answer in under FLOOR left out); [] where none.
function whereWins(i, rows, x, me) {
  const faster = (r) => r.t[x] !== null
    && (r.t[me] === null || (Math.max(r.t[me], r.t[x]) >= FLOOR && Math.max(r.t[me], FLOOR) > Math.max(r.t[x], FLOOR)));
  const by = i.groups.map((g) => [g, g.rows.filter((j) => faster(rows[j])).length]).filter(([, n]) => n > 0)
    .sort((a, b) => b[1] - a[1] || a[0].title.localeCompare(b[0].title));
  if (!by.length) return [];
  const named = by.slice(0, WINS_NAMED).map(([g, n]) => `${g.title} (${count(n)} of ${count(g.rows.length)})`);
  const more = by.length - named.length;
  return [note(`Where ${data.names[x]} is faster: ${and([...named, ...(more ? [`${more} more ${more === 1 ? "family" : "families"}`] : [])])}.`)];
}

// An integration's cactus plot and scatter: the pipeline against its
// system, and plain tddlog a third line where it read its own program of
// some of the models.
function integrationPlot(i, rows, key) {
  const x = i.system, me = i.pipeline;
  if (!i.systems.includes("tddlog")) return pairPlot(rows, x, i.vs[x], key);
  const three = { key: `${x}-${key}`, title: `${i.title}, ${data.names[x]} and tddlog`, noun: "instances", systems: [me, x, "tddlog"],
    instances: rows.map((r) => [r.name, r.v[me], r.v[x], r.v.tddlog]), limit: Math.max(...rows.map((r) => r.limit)) };
  return element("div", { class: "pair" }, plotted(three, { ...data.names, tddlog: "tddlog (its own program)" }),
    scatter(rows, ["tddlog", x], x, false, (r) => r.me));
}

// The pipeline's answered time split into the export's (the exporter's run up
// to what it hands tddlog) and tddlog's inference's, in cells; and, in an
// integration that is `ported` (Alea.jl+tddlog's), each other system on a
// program of its own of the model: its answers and total against the
// pipeline's.
function othersOf(i) {
  const me = i.pipeline, exporter = data.names[i.system];
  const num = (text) => element("td", { class: "num" }, text);
  const out = [];
  const s = i.splits?.[me];
  if (s) {
    out.push(cells([`${data.names[me]}'s answered time`, duration(s.seconds), `over the ${count(s.answers)} instances it answers`],
      [`${exporter}'s export`, duration(s.export), `${exporter}'s run up to what it hands tddlog`],
      ["tddlog's inference", duration(s.seconds - s.export)]));
  }
  const xs = i.systems.filter((y) => !isMine(y) && y !== i.system).sort((a, b) => i.answers[b] - i.answers[a]);
  if (!xs.length) return out;
  const par = `PAR-${data.measure.par}, over the instances the system is given and either answers`;
  // totals over no instance (neither answers any the system is given) are no time
  const total = (y, t) => (i.vs[y].total || i.vs[y].tddlog_total ? duration(t) : "–");
  const body = element("tbody", {}, ...xs.map((y) => element("tr", {}, element("th", { scope: "row" }, systemLink(y)),
    num(count(i.given[y])), num(count(i.answers[y])), num(total(y, i.vs[y].total)), num(total(y, i.vs[y].tddlog_total)), num(speedCell(i.vs[y])))));
  out.push(element("h3", {}, "On a program of their own of the model"),
    scroller(stacked(element("table", { class: "board" }, head2(["system"], ["given", null, "num"], ["answers", null, "num"],
      { group: "against " + data.names[me], cols: [["total", par, "num"], [data.names[me], par, "num"], [speedName("speed-up"), null, "num"]] }), body)), "whole"));
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
  const every = c.key === ALL;   // all programs: how every program was written, those of kind x too
  const out = [];
  // whom a translation was written for, by its kind; on all programs a translation not idiomatic for
  // the system is in the totals too, a column of its own (a type leaves it out of its totals, and its
  // board below counts it)
  const FOR = { s: ["it", "a translation for the system", "the system"], m: ["tddlog", "a translation for tddlog", "tddlog"],
    b: ["both", "a translation of the original for both", "both"] };
  for (const [id, title, keep, field] of [["head", "Head-to-head", "h", "head"], ["translated", "Translated", every ? "smbx" : "smb", "translated"]]) {
    const have = xs.filter((x) => n(x, keep) > 0);
    // the kinds of translation the type has: a column each where it has more than one
    const kinds = keep === "h" ? [] : [...keep].filter((k) => have.some((x) => n(x, k) > 0));
    const split = kinds.length > 1 ? kinds : [];
    const whom = [...new Set(kinds.map((k) => FOR[k === "x" ? "s" : k][2]))];
    const text = keep === "h" ? "both read the benchmark's program as its authors or its generator wrote it, or the same network in another network format"
      : whom.length === 1 && whom[0] === "the system" ? "the system read a translation and tddlog the benchmark's own program"
        : `one of them, or both, read a translation (for ${whom.length > 1 ? `${whom.slice(0, -1).join(", ")} or ${whom.at(-1)}` : whom[0]})`;
    const body = element("tbody");
    for (const x of have) {
      const w = v[field][x];
      const num = (text) => element("td", { class: "num" }, text);
      body.append(element("tr", { class: x === route.vs ? "vs" : null }, element("th", { scope: "row" }, systemLink(x)),
        num(count(n(x, keep))), ...split.map((k) => num(count(n(x, k)))),
        num(w ? count(w.answers) : ""), num(w ? duration(w.total) : ""), num(w ? count(w.tddlog_answers) : ""), num(w ? duration(w.tddlog_total) : ""),
        num(w ? speedCell(w) : "")));
    }
    const sec = element("section", { class: "panel split", id }, element("h3", {}, title), note(`Where ${text}.`));
    if (have.length) {
      const par = `PAR-${data.measure.par}, over the instances the system is given and either answers`;
      const cols = split.filter((k) => k !== "x").map((k) => [...FOR[k].slice(0, 2), "num"]);
      sec.append(scroller(stacked(element("table", { class: "board" }, head2(["system"], ["instances", null, "num"],
        ...(cols.length ? [{ group: "translated for", cols }] : []),
        ...(split.includes("x") ? [["not idiomatic", "a translation its users would not write, in these totals", "num"]] : []),
        { group: "the system", cols: [["answers", null, "num"], ["total", par, "num"]] },
        { group: "tddlog", cols: [["answers", null, "num"], ["total", par, "num"], [speedName("faster"), "tddlog's speed-up", "num"]] }), body)), "whole"));
      const x = have.includes(route.vs) ? route.vs : have[0];
      const rows = kindRows(c, x).filter((r) => keep.includes(r.kind));
      if (rows.length) {
        sec.append(element("p", { class: "small" }, `tddlog against ${data.names[x]} on ${plural(rows.length, "instance")}:`),
          pairPlot(rows, x, v[field][x], id));
      }
    } else sec.append(note("No system here."));
    out.push(sec);
  }
  out.push(element("h3", { class: "every" }, "Every instance of the type, each system's answers over all of them"));
  return out;
}

// ------------------------------------------------------------- coverage

// A source of the census: what it holds, how many programs and how many
// tddlog runs, each in a cell of its own, and why it does not run the others
// (folded on the census's page, open on a baseline's, where it is the point).
// A reason of the census as its badge, what it means on hover.
const reason = (k) => element("span", { class: `cls cls-${k.replace(/ /g, "-")}`, title: draft.views.census.classes[k] ?? null }, k);

function censusSource(src, tool = false, open = false) {
  const n = (x) => (typeof x === "number" ? count(x) : x);
  const rows = src.items.map((it) => element("tr", {}, element("td", {}, reason(it.class)),
    element("td", { class: "num" }, n(it.n)), element("td", { class: "about" }, it.programs), element("td", { class: "about" }, it.why)));
  const table = () => [scroller(stacked(element("table", { class: "board census-t" }, element("thead", {}, element("tr", {},
    element("th", { scope: "col" }, "why"), element("th", { scope: "col", class: "num" }, "programs"), element("th", { scope: "col" }, "which"),
    element("th", { scope: "col" }, "what stops tddlog"))), element("tbody", {}, ...rows))), "whole")];
  // counts in cells; a source counted in words, the words under the same labels
  const counted = typeof src.programs === "number" && typeof src.runs === "number";
  return element("div", { class: "census" },
    element("p", {}, element("b", {}, src.source), ...(tool ? [" (", systemLink(src.system), ")"] : []), `: ${src.what}.`),
    counted ? cells(["programs", count(src.programs)], ["tddlog runs", count(src.runs)], ["tddlog does not run", count(src.programs - src.runs)])
      : element("dl", { class: "classes words" }, element("dt", {}, "programs"), element("dd", {}, n(src.programs)),
        element("dt", {}, "tddlog runs"), element("dd", {}, n(src.runs))),
    ...(!rows.length ? [] : open ? table() : [lazy({ class: "census-fold" }, `Why tddlog does not run the others: ${plural(rows.length, "kind")}`, table)]));
}

function coverageView() {
  const cen = draft.views.census;
  // the census's account in its first sentence, the reasons in one line, each opening what it means
  const out = [crumbs(HOME, inSection("types"), [COVERAGE]), element("h2", {}, COVERAGE), brief(cen.about, "lede"),
    element("details", { class: "reasons" }, element("summary", {}, "Why tddlog does not run a program: ",
      ...Object.keys(cen.classes).map(reason)),
    element("dl", { class: "classes" }, ...Object.entries(cen.classes).flatMap(([k, text]) => [element("dt", {}, reason(k)), element("dd", {}, text)])))];
  // the run's side: each system, what it answers that tddlog does not, and what tddlog answers that it cannot be given
  const body = element("tbody");
  const sums = (v) => Object.values(v.unexpressed).reduce(([a, b], [c, d]) => [a + c, b + d], [0, 0]);
  const refusals = data.systems.some((s) => draft.views.systems[s.key] && sums(draft.views.systems[s.key])[0] > 0);
  for (const s of data.systems) {
    const v = draft.views.systems[s.key];
    if (!v) continue;
    const [refused, missing] = sums(v);
    // a count opens the system's coverage, which lists those instances
    const cell = (n) => element("td", { class: "num" }, n ? link({ view: "baseline", system: s.key, tab: "coverage" }, count(n)) : count(n));
    body.append(element("tr", {}, element("th", { scope: "row" }, systemLink(s.key)),
      cell(v.beyond.length), ...(refusals ? [cell(refused)] : []), cell(missing)));
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
  ["read", "a ProbLog program or LPAD, a Dice program, a netlist or a network file, read as one program", ["src/input.rs"]],
  ["parse", "the program's clauses, queries and evidence", ["src/syntax"]],
  ["ground", "the rules without variables that the queries and evidence need, simplified", ["src/ground"]],
  ["rewrite", "linear recursion written out, and copies of atoms where the layout reads them", ["src/ground/eliminate.rs", "src/indicators"]],
  ["layout", "a vtree over the facts: auto tries a few layouts, and the first to finish answers", ["src/vtree", "src/auto"]],
  ["compile", "a TDD per derived atom, component by component", ["src/tp"]],
  ["evidence", "the evidence conjoined, and each query counted against it", ["src/query", "src/eval"]],
  ["answers", "the probabilities, in the program's order", ["src/solve"]],
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
// fact a caption states is the trace's or docs/design.md's. A caption says
// what is particular to the program in a sentence or two: what the stage
// does is its line beside the heading, and its counts are the cells the
// trace draws under the caption.
const WALKS = [
  {
    key: "reach", what: "Recursion over a graph with cycles, written without recursion before the compile.",
    file: "examples/cycle_graph.pl", example: "reachability", evidence: false,
    command: "tddlog --trace reach.json examples/cycle_graph.pl",
    lede: "The probability that node 1 reaches each node of a graph with cycles, its 14 edges each with a probability of its own.",
    captions: {
      read: "A ProbLog file is the program as it stands: there is nothing to translate.",
      parse: "The 14 edges as probabilistic facts, the two rules of `path/2` (the second reads `path` itself) and six queries `path(1,Y)`.",
      ground: "Only the paths from node 1 are ground, since the queries need no others. They form one cyclic component of six atoms, round the graph.",
      rewrite: "The recursion is linear, so tddlog writes it out: the cycle's atoms are eliminated one at a time, as in Floyd and Warshall's algorithm. Eight acyclic components are left.",
      layout: "`auto` gives this program one trial, `dynamic`, and it finished in its first attempt. The vtree is right-linear over a min-fill order of the 12 edges.",
      compile: "Each atom's formula over the edges is built by TDD operations, in dependency order. The largest, `path(1,6)`, has 79 pairs. No cycle is left, so no fixpoint runs.",
      evidence: "There is no evidence, so each query was counted during the compile.",
      answers: "What `tddlog examples/cycle_graph.pl` prints. ProbLog 2.3.0 gives the same answers (tests/it/examples.rs).",
    },
  },
  {
    key: "tpch", what: "A database query: grounding evaluates its joins and selections, and its lineage is compiled.",
    file: "examples/tpch_q2.pl", example: "tpch", evidence: false,
    command: "tddlog --trace tpch.json examples/tpch_q2.pl",
    lede: "TPC-H's query 2 as a Boolean query: is a brass part of size 15 supplied from Europe? Five relations of 334 tuples, each an independent probabilistic fact.",
    captions: {
      read: "A ProbLog file: each relation's tuples as probabilistic facts, then the query.",
      parse: "The 334 tuples of `region`, `nation`, `supplier`, `part` and `partsupp`, and the one rule of `q2`.",
      ground: "Grounding evaluates the joins and selections as a database does: one part is brass of size 15, and two of its four suppliers are in Europe. What is left is the lineage, 2 rules over 8 tuples.",
      rewrite: "Nothing to rewrite: there is no recursion, and this layout reads no copies.",
      layout: "`auto`'s `td` trial finished in its first attempt: a vtree over the 8 tuples, from a tree decomposition of width 5.",
      compile: "`q2` is the disjunction of its two rules' conjunctions: one component of 30 pairs.",
      evidence: "There is no evidence, so `q2` was counted during the compile.",
      answers: "0.017751203. The product formula in the program's header, which reads each tuple once, and ProbLog 2.3.0 give the same (tests/it/examples.rs).",
    },
  },
  {
    key: "network", what: "A network file read as a program: its tables, then the evidence.",
    file: "examples/car.net", example: "car", evidence: true,
    command: "tddlog --trace network.json --evidence-probability --evidence examples/car.inst --query fuel,plugs,starter examples/car.net",
    lede: "A car that does not start on a cold morning, a Hugin network of 13 variables. The headlights are bright and the fuel gauge is low (examples/car.inst): is it the fuel, the plugs or the starter?",
    captions: {
      read: "The network and its evidence are read as one program, the one `tddlog --program` prints (examples/bn_car_evidence.pl). Each row of a table is an annotated disjunction over the values it gives a chance.",
      parse: "31 annotated disjunctions, and 11 rules for the rows certain of one value, such as `cranks(no) :- battery(flat), starter(faulty)`; four observations and seven queries.",
      ground: "A choice among n values becomes n − 1 switches, each given the ones before. The observed `weather(cold)` is fixed here, and its 0.3 is kept as a factor of P(e).",
      rewrite: "The `families:0` layout reads an atom of another family through a copy, a variable of its own tied to the atom by an observed link: 29 copies of 21 atoms, and 13 links beside 3 of the network's observations.",
      layout: "`auto`'s `families:0` trial finished in its first attempt: 23 families in 12 groups, laid out by a tree decomposition of width 2. The vtree's 65 leaves are 36 switches and 29 copies.",
      compile: "37 components of one atom each, none cyclic. The links hold the tables' rows. The largest formula, 106 pairs, is `cranks(yes)` as `starts(no)` reads it.",
      evidence: "The 16 observations are conjoined along the vtree into 438 pairs, whose count is P(e) = 0.0226. Each query is counted against it: P(q | e) = WMC(q ∧ e) / WMC(e).",
      answers: "An empty tank goes from 0.10 to 0.29, fouled plugs from 0.20 on a cold day to 0.47, a faulty starter from 0.03 to 0.10. A sum over the worlds and ProbLog give the same answers (tests/it/networks.rs).",
    },
  },
  {
    key: "markov", what: "A UAI Markov network: its factors as observed atoms, and Z from P(e).",
    file: "tests/regress/potts.uai", example: null, evidence: false,
    command: "tddlog --trace markov.json --no-queries --evidence-probability tests/regress/potts.uai",
    lede: "A UAI Markov network: five variables of three values and six Potts factors over pairs, a weighted graph colouring. Asked nothing (`--no-queries`), tddlog answers Z.",
    captions: {
      read: "Each variable is an annotated disjunction, and each factor an observed atom `fJ(ok)` with a rule per nonzero entry. Fixing `x0` breaks the colours' symmetry and leaves a third of Z (docs/design.md).",
      parse: "The fact `x0(v_0)`, four annotated disjunctions, 50 rules (19 of them probabilistic) and seven observations, `f0` to `f6`.",
      ground: "The observations fix `x0` and `x2` before any diagram is built, and rows of one weight share a class atom: 41 rules over 42 atoms become 33 over 29.",
      rewrite: "As for a Bayesian network, the `families:0` layout reads an atom of another family through a copy tied to it by an observed link: 22 copies of 13 atoms.",
      layout: "`auto`'s `families:0` trial finished in its first attempt: 14 families in 10 groups, laid out by a tree decomposition of width 2. The vtree's 32 leaves are 10 switches and 22 copies.",
      compile: "27 components of one atom each, in dependency order. The largest, `f3(ok)` at 34 pairs, is the one factor nothing fixed.",
      evidence: "Only P(e) is wanted, so as the 14 observations are conjoined along the vtree, each variable is summed out once nothing later reads it: 11 summing joins leave one pair. P(e) = 5/81, about 0.0617.",
      answers: "Z = P(e) × 10^c, with c the offset on the program's first line: 112.5, the 225/2 that tests/it/networks.rs takes from an enumeration of the worlds.",
    },
  },
  {
    key: "dice", what: "A probabilistic functional program: integers held as bits, a derived atom per bit and step.",
    file: "tests/dice/walk.dice", example: "walk", evidence: true,
    command: "tddlog --trace dice.json --evidence-probability tests/dice/walk.dice",
    lede: "A random walk on 0..7 in Dice: ten steps of plus or minus one from 3, absorbed at either end, observed not to end at 0. tddlog reads Dice itself and holds an integer as bits.",
    captions: {
      read: "The Dice front end evaluates the program symbolically into Datalog: `x` is three bits per step, each step's flip is `0.4::x_c_3(T)`, the count is a fact `steps(10)`, and each value has a query `x(V)`.",
      parse: "The flip as a probabilistic rule, the count and the start as facts, and 41 rules; eight queries and the observation `obs`.",
      ground: "The steps are ground level by level, constants are proved as each level completes, and equal gates are merged, some proved equal by a SAT solver (`swept`): 216 atoms become 90.",
      rewrite: "Nothing to rewrite: no recursion is left once the steps are ground, and `closed` reads no copies.",
      layout: "`closed` is `auto`'s first trial where the facts are few and the derived atoms per fact many (here 10 and 80). It puts new facts on a right-linear spine, here a chain over the ten flips.",
      compile: "Each of the 80 gates is a component, compiled in dependency order, one step after the other. The formulas grow with the steps, up to 59 pairs.",
      evidence: "The observation is conjoined into 41 pairs, whose count is P(e) = 0.432898: the probability that the walk does not end at 0.",
      answers: "Ten steps from 3 end on an odd number unless absorbed at 0, which is observed away, so the even values have probability 0. Dice gives the same answers (tests/it/examples.rs).",
    },
  },
  {
    key: "circuit", what: "Alea.jl+tddlog: Alea.jl builds a circuit of the program, and tddlog optimizes it and compiles it gate by gate.",
    file: "examples/alea_dice.pl", example: "circuit", evidence: true,
    command: "tddlog --trace circuit.json --evidence-probability examples/alea_dice.pl",
    lede: "Two dice, the first at least 4, as the circuit Alea.jl builds (8 flips, 78 gates), in ProbLog. The queries: the sum is 7 (`q1`), the first shows more (`q2`).",
    captions: {
      read: "Alea.jl+tddlog: Alea.jl builds the circuit, and its tddlog backend writes it as this program. A flip is a probabilistic fact `f1` to `f8`, an and-gate a rule, an or-gate a rule per input, and a negated input is written `\\+`.",
      parse: "The 8 flips and 111 rules; two queries and one observation.",
      ground: "Before any diagram is built, the gates are optimized as an and-inverter graph: 24 views inlined, 19 atoms merged, 4 proved equal by SAT sweeping. 88 atoms and 111 rules become 40 and 49.",
      rewrite: "The `program` layout keeps the views in place, the 81 derived atoms as written. It names one, `g76`, which crosses the one narrow cut, through a copy and an observed link.",
      layout: "`auto`'s first trial here is `program`: a vtree right-linear over the flips, an OBDD in the order Alea.jl's BDD package built the circuit in. Its 9 leaves are the 8 flips and the copy.",
      compile: "81 components, a derived atom each, bottom-up in dependency order, each gate once; the largest has 60 pairs. Each formula is stored as its negation, so the operations count no disjunctions.",
      evidence: "The observation and the link are conjoined into 40 pairs: P(e) = 0.5, the first die at least 4. Both queries are counted against it.",
      answers: "1/6 and 2/3, the answers the two dice give by hand (tests/it/examples.rs).",
    },
  },
  {
    key: "roulette", what: "Roulette+tddlog: Roulette builds a circuit of the program, and tddlog does the inference.",
    file: "tests/roulette/lib/diagnosis.pl", example: "roulette", evidence: true,
    command: "tddlog --trace roulette.json --evidence-probability tests/roulette/lib/diagnosis.pl",
    lede: "A fault diagnosis in Roulette (tests/roulette/lib/diagnosis.rkt): an and gate fails with probability 0.1, an or gate with 0.2, and the output is observed wrong. Which gate failed?",
    captions: {
      read: "Roulette+tddlog: Roulette runs the program symbolically into a circuit, and its tddlog engine writes the circuit as this program: a flip per random choice, a rule per gate.",
      parse: "4 flips (the inputs, of probability 1 and 0, and the faults `f3` and `f4`), 15 rules and a fact; the output `g9` is observed, and there are 4 queries.",
      ground: "The inputs are constants, so the gates fold away: 16 rules over 17 atoms become 6 over 7, and the output is the exclusive or of the faults. The observation makes `q4(1)` false.",
      rewrite: "Nothing to rewrite: there is no recursion, and this layout reads no copies.",
      layout: "`auto`'s `td` trial finished in its first attempt: a vtree of two leaves, the faults.",
      compile: "5 components of one atom each, in dependency order; the largest, `g9`, has 2 pairs.",
      evidence: "The observations, `g9` true and `q4(1)` false, are conjoined into 2 pairs: P(e) = 0.26, the probability that exactly one gate fails.",
      answers: "The and gate failed with probability 0.08 / 0.26 and the or gate with 0.18 / 0.26, never both: the answers computed by hand in the program (tests/it/examples.rs).",
    },
  },
  {
    key: "gears", what: "Negation through a loop: atoms that support only each other are false.",
    file: "examples/gearwheels.pl", example: "gears", evidence: false,
    command: "tddlog --trace gears.json examples/gearwheels.pl",
    lede: "Two gear wheels that turn each other, and a button for each (Bogaerts and Van den Broeck 2015, Example 4.13). The program is not stratified, so its meaning is its well-founded model.",
    captions: {
      read: "A ProbLog file is the program as it stands: there is nothing to translate.",
      parse: "The two buttons as probabilistic facts and 8 rules, two of which read `turns1(0)` and `turns2(0)` under negation; three queries.",
      ground: "At time 0, `turns1(0)` and `turns2(0)` support only each other, so both are false. At time 1, `turns1(1)` and `turns2(1)` form one cycle.",
      rewrite: "Each rule reads one atom of the cycle, so the cycle is written out: `turns1(1)` reads the two buttons, and `turns2(1)` reads its button and `turns1(1)`.",
      layout: "`auto` gives this program one trial, `dynamic`, and it finished in its first attempt. The vtree's two leaves are the buttons.",
      compile: "`turns1(1)` first, then `turns2(1)`, which reads it: 3 pairs each. No cycle is left, so no fixpoint runs.",
      evidence: "There is no evidence: the queries were counted during the compile, and `turns1(0)` is false.",
      answers: "0 at time 0, and 1 − 0.7 × 0.4 = 0.72 for each wheel at time 1. ProbLog 2.3.0 gives the same answers (tests/it/examples.rs).",
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

// The guide's terms a caption links, the first use of each in a step:
// what matches, and the term's id in the guide (guide/#term-ID).
const TERMS = [[/\bvtrees?\b/, "vtree"], [/\bpairs\b/, "pairs"], [/\btrials?\b/, "trial"], [/\blayouts?\b/, "layout"],
  [/\bcomponents?\b/, "component"], [/\bTDDs?\b/, "tdd"], [/P\(e\)/, "evidence-probability"]];

// A text with the terms not yet in `seen` linked to the guide, each once.
function withTerms(text, seen) {
  const out = [];
  let rest = text;
  while (rest) {
    let first = null;
    for (const [re, id] of TERMS) {
      const m = seen.has(id) ? null : re.exec(rest);
      if (m && (!first || m.index < first.m.index)) first = { m, id };
    }
    if (!first) break;
    seen.add(first.id);
    out.push(rest.slice(0, first.m.index), element("a", { href: `guide/#term-${first.id}`, class: "term" }, first.m[0]));
    rest = rest.slice(first.m.index + first.m[0].length);
  }
  return [...out, rest];
}

// A caption as a paragraph, `code` between backticks; with `seen`, the
// guide's terms outside the code linked (withTerms).
const caption = (text, cls, seen) => element("p", cls ? { class: cls } : {},
  ...text.split("`").map((part, i) => (i % 2 ? element("code", {}, part) : seen ? withTerms(part, seen) : part)));

const playgroundHref = (w) => `playground/?example=${w.example}${w.evidence ? "&evidence=1" : ""}&trace=1`;

// A strip that scrolls sideways (on a phone) scrolled to show its marked
// item, once it is laid out.
function showMarked(strip) {
  requestAnimationFrame(() => {
    const on = strip.querySelector("[aria-current]");
    if (!on || strip.scrollWidth <= strip.clientWidth) return;
    const [a, b] = [on.getBoundingClientRect(), strip.getBoundingClientRect()];
    if (a.left < b.left || a.right > b.right) strip.scrollLeft += a.left - b.left - 24;
  });
}

// How it works: the walkthroughs, the one on show (the first where the
// address names none) at its step, the strip of the stages above it.
function howView(route) {
  const w = WALKS.find((x) => x.key === route.walk) ?? WALKS[0];
  const out = [crumbs(HOME, ...(route.walk ? [inSection("how"), [w.title]] : [[sectionName("how")]])),
    element("h2", {}, sectionName("how")),
    element("p", { class: "lede one" }, "Pick a program and step through tddlog's run on it, drawn from the run's own trace (", element("code", {}, "tddlog --trace"), "). ",
      element("a", { href: `${BRANCH}main/docs/design.md#how-a-program-is-answered` }, "docs/design.md"), " describes each stage."),
    element("nav", { class: "walks", "aria-label": "Walkthroughs" }, ...WALKS.map((x) =>
      element("a", { href: `#/how/${x.key}`, title: x.what, "aria-current": x === w ? "page" : null }, x.title)))];
  showMarked(out[out.length - 1]);
  const walk = element("div", { class: "walk" }, note("Loading the trace…"));
  out.push(walk, note("How the runs were made and the results counted: ", link({ view: "methods" }, METHODS), "."));
  walkTrace(w.key).then((trace) => {
    const steps = tddlogTrace.steps(trace);
    const i = Math.min(Math.max(route.step ?? 0, 0), steps.length - 1);
    const s = steps[i];
    const stage = STAGES.find(([key]) => key === s.key);
    const head = element("div", { class: "walk-head" }, caption(w.lede, "lede"),
      element("p", { class: "small run" }, element("code", {}, w.command.slice(0, -w.file.length),
        element("a", { href: `${BRANCH}main/${w.file}` }, w.file)),
        ...(w.example ? [element("a", { href: playgroundHref(w), class: "try-btn" }, "Open in playground")] : [])));
    // the stage's line beside the heading, its code a link each; the guide's terms linked once in the step
    const seen = new Set();
    const body = element("div", { class: "step" }, element("h3", {}, `${i + 1}. ${s.title}`,
      ...(stage ? [element("span", { class: "muted small" }, ...withTerms(stage[1], seen), ", in ",
        ...stage[2].flatMap((src, k) => [k ? " and " : "", element("a", { href: `${BRANCH}main/${src}` }, element("code", {}, src))]))] : [])));
    if (w.captions[s.key]) body.append(caption(w.captions[s.key], null, seen));
    const drawn = element("div", { class: "walk-trace" });
    drawn.innerHTML = tddlogTrace.stepHtml(s);
    body.append(drawn);
    const nav = element("p", { class: "step-nav" }, i > 0 ? element("a", { href: `#/how/${w.key}/${i}` }, "← Back") : "",
      i < steps.length - 1 ? element("a", { href: `#/how/${w.key}/${i + 2}` }, "Next →") : "");
    const strip = stepper(w, steps, i);
    walk.replaceChildren(strip, head, body, nav);
    showMarked(strip);
  }, (e) => walk.replaceChildren(note(`The trace did not load: ${e.message}.`)));
  return out;
}

// The methods table (#/methods), the one place that says how the runs were
// made and how the results are counted: a row a fact, the
// runs' from the data (data/systems.json's `methods`), each type's time
// limits from its rows, the measure from data.measure and the data's stamp.
// Under a table, the views say the measure in one line (measureLine), and
// every speed-up's header links here (results.js's speedName).
function methodsView() {
  const { floor, par } = data.measure;
  const limits = data.categories.filter((c) => c.key !== ALL).map((c) =>
    `${c.title} ${[...new Set(c.rows.map((r) => r[2]))].sort((a, b) => a - b).join(" or ")} s`);
  const { date, source } = data.stamp;
  const group = (title, rows) => element("tbody", {}, element("tr", { class: "group" }, element("th", { scope: "colgroup", colspan: 2 }, title)),
    ...rows.map(([what, fact]) => element("tr", {}, element("th", { scope: "row" }, what), element("td", {}, fact))));
  return [crumbs(HOME, inSection("how"), [METHODS]), element("h2", {}, METHODS),
    element("table", { class: "board methods" },
      group("How the runs were made", data.methods.flatMap((row) => (row[0] === "memory" ? [row, ["time limits", limits.join("; ")]] : [row]))),
      group("How the results are counted", [
        ["answers", "A run out of time or memory, wrong, refused or not given is unanswered; a type's tables count a system's answers on the "
          + "instances it is given as its users would write them, its plots every instance"],
        [`total (PAR-${par})`, `Over the instances a system is given (in a type, as its users would write them) and either answers: an unanswered run at ${twice(par)} its time limit, `
          + `a time under ${floor} s at ${floor} s`],
        ["speed-up", `The ratio of the totals over the instances both answer (not both under ${floor} s) and over each instance only one of the two `
          + `answers, the other at its limit, where that raises the multiplier of the system that answered it, so that `
          + `a system's answers the other misses only help it; no minimum number of instances in common`],
        ["X+tddlog", "X's own code, then tddlog's inference; its time includes X's after its warm-up, so that against X it differs only in the inference"],
        ["circuits", "Every circuit Alea.jl exports, read by every system and timed without its export; the functional programs among them are also in their type"],
        ["data", element("span", {}, `Processed ${date}, on public main `,
          element("a", { href: `https://github.com/Tractables/tddlog/commit/${source}` }, source))]]))];
}

// ------------------------------------------------------------- the routes

// FNV-1a (32 bits) of a category's rows' SUITE/NAME, a line each, as
// the processing computes it.
function fingerprint(rows) {
  let h = 0x811c9dc5;
  for (const b of new TextEncoder().encode(rows.map((r) => `${r[0]}/${r[1]}`).join("\n"))) h = Math.imul(h ^ b, 0x01000193) >>> 0;
  return h;
}

// The types whose rows views.json was not written from (its kinds
// would be another instance's).
function staleViews() {
  if (draft.stale === undefined) {
    draft.stale = data.categories.filter((c) => draft.views.categories[c.key]?.rows !== fingerprint(c.rows)).map((c) => c.title);
  }
  return draft.stale;
}
const staleNote = (stale) => note(`These views (views.json) were written from other results than these (${and(stale)}); `
  + "the results' processing writes them again from the results page's data.");

const DRAFT_VIEWS = { "draft-home": draftHome, baselines: baselinesView, baseline: baselineView, integrations: integrationsView,
  integration: integrationView, types: home, coverage: coverageView, how: howView, methods: methodsView };

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
      // a tab, not a section scrolled to: another tab is another view
      if (!route || parts.length !== 3 || !BASELINE_TABS.some(([id]) => id === at)) return null;
      return at === "idiomatic" ? route : { ...route, tab: at };
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
    if (parts.length === 1 && ["types", "coverage", "how", "methods"].includes(first)) return { view: first };
    if (first === "how" && WALKS.some((w) => w.key === key)) {
      if (parts.length === 2) return { view: "how", walk: key, step: 0 };
      if (parts.length === 3 && /^[1-9]\d{0,2}$/.test(at)) return { view: "how", walk: key, step: Number(at) - 1 };
    }
    return null;
  },
  href: (route) => ({ "draft-home": "#/", baselines: "#/baselines",
    baseline: `#/baselines/${encodeURIComponent(route.system)}${route.tab ? `/${route.tab}` : ""}`,
    integrations: "#/integrations", integration: `#/integrations/${route.key}`, types: "#/types", coverage: "#/coverage", methods: "#/methods",
    how: route.walk ? `#/how/${route.walk}${route.step ? `/${route.step + 1}` : ""}` : "#/how" })[route.view] ?? null,
  draw: (route) => {
    if (!(route.view in DRAFT_VIEWS)) return null;
    const stale = staleViews();
    if (stale.length && !["draft-home", "types", "how", "methods"].includes(route.view) && !route.view.startsWith("integration")) return [staleNote(stale)];
    return DRAFT_VIEWS[route.view](route);
  },
  title: (route) => ({ "draft-home": null, baselines: sectionName("baselines"), baseline: `tddlog against ${data.names[route.system]}`,
    integrations: sectionName("integrations"), integration: data.integrations?.[route.key]?.title, types: sectionName("types"),
    coverage: COVERAGE, methods: METHODS, how: route.walk ? `${sectionName("how")}: ${WALKS.find((w) => w.key === route.walk).title}` : sectionName("how") })[route.view] ?? null,
  // the circuits are no type: views.json splits only the types
  category: (c, route, rows) => (c.key === APART ? [] : staleViews().length ? [staleNote(staleViews())] : typeSplit(c, route, rows)),
  // a translator's or generator's name in a line on how a file was made, as a link to its source
  made: (html) => html.replace(/<code>([\w-]+\.(?:py|jl|rkt))(\b[^<]*)<\/code>/g, (whole, name, rest) =>
    (draft.views?.scripts.includes(name) ? `<a href="${BRANCH}baselines/scripts/${name}"><code>${name}${rest}</code></a>` : whole)),
});
