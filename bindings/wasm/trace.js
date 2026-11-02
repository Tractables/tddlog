"use strict";

// A trace of a run drawn as its steps: the JSON `tddlog --trace` writes and
// the module returns where it is asked for one (src/trace.rs describes its
// stages). Shared by the playground, which draws the trace of the reader's
// own program, the draft site's walkthroughs (next/), which put a caption
// above each step, and the tests.
const tddlogTrace = (() => {
  // The format src/trace.rs writes (trace::VERSION).
  const VERSION = 1;

  const esc = (text) => String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const count = (n) => Number(n).toLocaleString("en-US");
  const plural = (n, one, many = `${one}s`) => `${count(n)} ${n === 1 ? one : many}`;
  const seconds = (s) => (s < 0.001 ? "<1 ms" : s < 1 ? `${(s * 1000).toFixed(0)} ms` : `${s.toFixed(2)} s`);
  const code = (text) => `<code>${esc(text)}</code>`;

  // What a library caller reads of each stage (src/solve, Progress), and
  // the code that does its work (docs/design.md, "How a program is
  // answered"): the TDD operations are tididi's, as the compile calls them.
  const API = {
    read: "tddlog::read(data, format, &ReadOptions) -> Input",
    parse: "tddlog::solve(&input.program, &options, progress): Progress::Parsed { program }",
    ground: "Progress::Grounded { ground }: tddlog::ground::ground",
    rewrite: "Progress::Answered(run): run.ground, by Kind::rewrite",
    layout: "run.vtree: auto's trials, each a vtree::Kind building a tididi::Vtree",
    compile: "run.stats, run.trace: tididi's Engine::and, or, negate_with, minimize and equivalent",
    evidence: "run.query_sizes, run.results: the evidence conjoined, then eval::count",
    answers: "Solution::answers",
    error: "Error",
  };

  // A text of the trace (`{text, lines, shown}`) as a block of code, with
  // what it leaves out said under it.
  function block(label, t, lang) {
    if (!t) return null;
    let more = "";
    if (t.lines === null) more = `the first ${count(t.shown)} lines`;
    else if (t.lines > t.shown) more = `the first ${count(t.shown)} of ${count(t.lines)} lines`;
    return { label, text: t.text, lang, more };
  }

  const ops = (o) => [["and", "conjunctions"], ["or", "disjunctions"], ["negate", "negations"], ["minimize", "minimizations"], ["equivalent", "equivalence tests"]]
    .filter(([k]) => o[k] > 0).map(([k, name]) => `${count(o[k])} ${name}`).join(", ") || "none";

  const sizes = (s) => `${plural(s.atoms, "atom")} (${count(s.facts)} facts, ${count(s.derived)} derived), ${plural(s.rules, "rule")}`;

  // The vtree as SVG: `tree` an inner node as [left, right], a leaf as its
  // fact's name, a subtree left closed as {leaves}.
  function vtreeSvg(vtree) {
    const leaves = [];
    const nodes = [];
    let depth = 0;
    function place(t, d) {
      depth = Math.max(depth, d);
      if (Array.isArray(t)) {
        const l = place(t[0], d + 1);
        const r = place(t[1], d + 1);
        const n = { x: (l.x + r.x) / 2, d, kids: [l, r] };
        nodes.push(n);
        return n;
      }
      const n = { x: leaves.length, d, leaf: typeof t === "string" ? t : null, closed: typeof t === "object" ? t.leaves : null };
      leaves.push(n);
      nodes.push(n);
      return n;
    }
    place(vtree.tree, 0);
    const label = Math.min(16, Math.max(...leaves.map((n) => (n.leaf ?? `${n.closed} leaves`).length)));
    const step = 22, dy = Math.max(10, Math.min(36, 300 / Math.max(depth, 1)));
    const w = Math.max(leaves.length * step, 120) + label * 3.5, top = 12, base = top + depth * dy, h = base + 8 + label * 6.2 + 8;
    const x = (n) => 11 + n.x * step, y = (n) => top + n.d * dy;
    let out = `<svg class="trace-vtree" viewBox="0 0 ${w + 4} ${h}" style="max-width:${w + 4}px" role="img" aria-label="The vtree: ${count(vtree.variables)} variables, depth ${count(vtree.depth)}">`;
    for (const n of nodes) {
      for (const k of n.kids ?? []) out += `<line x1="${x(n)}" y1="${y(n)}" x2="${x(k)}" y2="${y(k)}"/>`;
    }
    for (const n of nodes) {
      if (n.kids) out += `<circle class="inner" cx="${x(n)}" cy="${y(n)}" r="3"/>`;
      else if (n.closed !== null) {
        out += `<path class="closed" d="M${x(n)} ${y(n)} l-6 12 h12 z"><title>${esc(plural(n.closed, "leaf", "leaves"))} below</title></path>`;
        out += `<text class="leaf" transform="translate(${x(n) + 4} ${y(n) + 16}) rotate(60)">${esc(plural(n.closed, "leaf", "leaves"))}</text>`;
      } else {
        const name = n.leaf.length > 16 ? `${n.leaf.slice(0, 15)}…` : n.leaf;
        out += `<circle class="leaf" cx="${x(n)}" cy="${y(n)}" r="3.5"><title>${esc(n.leaf)}</title></circle>`;
        out += `<text class="leaf" transform="translate(${x(n) + 3} ${y(n) + 8}) rotate(60)">${esc(name)}</text>`;
      }
    }
    return out + "</svg>";
  }

  // The components in the order they compiled, a bar each, its height the
  // pairs of its largest formula, a cyclic one marked.
  function compileSvg(c) {
    const comps = c.components;
    if (!comps.length) return "";
    const most = Math.max(1, ...comps.map((k) => k.largest));
    const bw = Math.max(4, Math.min(24, Math.floor(640 / comps.length))), h = 120, w = comps.length * bw;
    let out = `<svg class="trace-bars" viewBox="0 0 ${w + 2} ${h + 18}" style="max-width:${Math.max(w + 2, 160)}px" role="img" aria-label="The pairs of each component's largest formula, in the order they compiled; the largest ${count(most)}">`;
    comps.forEach((k, i) => {
      const bh = Math.max(1, (k.largest / most) * h);
      const name = k.atoms.join(", ") + (k.size > k.atoms.length ? `, … (${count(k.size)} atoms)` : "");
      const what = `${name}: ${plural(k.largest, "pair")} at most, ${plural(k.updates, "update")}${k.cyclic ? (k.negation ? ", its well-founded model" : `, ${plural(k.rounds, "round")}`) : ""}`;
      out += `<rect class="${k.cyclic ? "cyclic" : "acyclic"}" x="${1 + i * bw}" y="${h - bh}" width="${bw - 1}" height="${bh}"><title>${esc(what)}</title></rect>`;
    });
    out += `<line x1="0" y1="${h + 0.5}" x2="${w + 2}" y2="${h + 0.5}"/>`;
    out += `<text x="1" y="${h + 14}">${esc(plural(comps.length, "component"))} in order</text>`;
    out += `<text x="${w + 1}" y="10" text-anchor="end">${esc(plural(most, "pair"))}</text>`;
    return out + "</svg>";
  }

  // The first rows of the compile's components as a table.
  function componentsTable(c, rows = 16) {
    const shown = c.components.slice(0, rows);
    if (!shown.length) return "";
    let out = '<div class="trace-scroll"><table class="trace-table"><thead><tr><th>#</th><th>atoms</th><th class="n">updates</th><th class="n">largest</th><th>formulas, in pairs</th></tr></thead><tbody>';
    shown.forEach((k, i) => {
      const atoms = k.atoms.map(code).join(" ") + (k.size > k.atoms.length ? ` … ${plural(k.size, "atom")}` : "");
      const kind = k.cyclic ? (k.negation ? " <span class=\"tag\">well-founded</span>" : ` <span class="tag">${plural(k.rounds, "round")}</span>`) : "";
      const each = k.sizes.slice(0, 8).map(([a, p, r]) => `${esc(a)} ${p === null ? "unchanged" : count(p)}${r ? ` (${r})` : ""}`).join(", ")
        + (k.updates > Math.min(8, k.sizes.length) ? ", …" : "");
      out += `<tr><td class="n">${i + 1}</td><td>${atoms}${kind}</td><td class="n">${count(k.updates)}</td><td class="n">${count(k.largest)}</td><td class="sizes">${each}</td></tr>`;
    });
    out += "</tbody></table></div>";
    const left = c.components.length - shown.length + c.rest.components;
    if (left > 0) out += `<p class="trace-more">and ${plural(left, "component")} more${c.rest.components ? `, ${count(c.rest.components)} of them counted together in the trace` : ""}.</p>`;
    return out;
  }

  function answersTable(answers, rows = 12) {
    if (answers === null) return '<p class="trace-more">The evidence is inconsistent: no query has an answer.</p>';
    if (!answers.length) return "";
    let out = '<div class="trace-scroll"><table class="trace-table"><thead><tr><th>query</th><th class="n">probability</th></tr></thead><tbody>';
    for (const a of answers.slice(0, rows)) out += `<tr><td>${code(a.query)}</td><td class="n">${a.probability.toFixed(6)}</td></tr>`;
    out += "</tbody></table></div>";
    if (answers.length > rows) out += `<p class="trace-more">and ${plural(answers.length - rows, "answer")} more.</p>`;
    return out;
  }

  // The steps of `trace`, one per stage, in the order of the run: {key,
  // part, title, api, facts: [[term, html]], figure: html, table: html,
  // code: [{label, text, lang, more}]}. A part's stages say which part they
  // are of where the queries were answered in parts.
  function steps(trace) {
    if (!trace || trace.trace !== VERSION) throw new Error(`not a trace of format ${VERSION}`);
    const stages = trace.stages;
    const parts = new Set(stages.filter((s) => s.part !== undefined).map((s) => s.part)).size;
    const notes = stages.filter((s) => s.stage === "note").map((s) => s.text);
    const handed = new Map(stages.filter((s) => s.stage === "compiling").map((s) => [s.part, s]));
    const lang = (format) => (format === "problog" ? "problog" : null);
    const out = [];
    let format = "problog";
    const add = (s, title, facts, more = {}) => out.push({
      key: s.stage, part: s.part ?? null, title: parts > 1 && s.part !== undefined ? `${title}, part ${s.part + 1} of ${parts}` : title,
      api: API[s.stage], facts, figure: "", table: "", code: [], ...more,
    });
    for (const s of stages) {
      switch (s.stage) {
        case "read": {
          format = s.format;
          const facts = [["Format", code(s.format)], ["Size", `${count(s.bytes)} bytes, ${plural(s.source.lines, "line")}`]];
          if (s.program) facts.push(["Read as", `a program of ${plural(s.program.lines, "line")} (${code("tddlog --program")} prints it)`]);
          for (const n of s.notes) facts.push(["Note", esc(n)]);
          add(s, "Read", facts, { code: [block("As given", s.source, lang(format)), s.program && block("Read as", s.program, "problog")].filter(Boolean) });
          break;
        }
        case "parse": {
          const kinds = [["probabilisticFacts", "probabilistic fact"], ["annotatedDisjunctions", "annotated disjunction"], ["probabilisticRules", "probabilistic rule"],
            ["facts", "fact"], ["rules", "rule"]].filter(([k]) => s[k] > 0).map(([k, name]) => plural(s[k], name));
          add(s, "Parse", [["Clauses", `${count(s.clauses)}: ${kinds.join(", ") || "none"}`], ["Queries", count(s.queries)], ["Evidence", count(s.evidence)],
            ["Time", seconds(s.seconds)]]);
          break;
        }
        case "ground": {
          const facts = [["As ground", `${plural(s.asGround.atoms, "atom")}, ${plural(s.asGround.rules, "rule")}`], ["After the passes", sizes(s)]];
          if (s.passes.length) facts.push(["Passes", s.passes.map((p) => `${esc(p.pass)} ${count(p.count)}`).join(", ")]);
          if (s.boundEnd.length) facts.push(["Bound end", s.boundEnd.map(code).join(" ")]);
          facts.push(["Components", `${count(s.components.count)} (${count(s.components.cyclic)} cyclic), the largest of ${plural(s.components.largest, "atom")}`]);
          facts.push(["Queries and observations", `${count(s.queries)} and ${count(s.observations)}`]);
          if (s.weight !== 1) facts.push(["Weight", `${s.weight}, a factor of P(q, e) and P(e) alike`]);
          for (const n of notes) facts.push(["Note", esc(n)]);
          facts.push(["Time", seconds(s.seconds)]);
          add(s, "Ground", facts, { code: [block("The ground program (tddlog --ground)", s.program, "problog")] });
          break;
        }
        case "rewrite": {
          const facts = [["Layout", code(s.kind)], ["Program", sizes(s)],
            ["Components", `${count(s.components.count)} (${count(s.components.cyclic)} cyclic), the largest of ${plural(s.components.largest, "atom")}`]];
          const h = handed.get(s.part);
          if (h) facts.unshift(["Handed to the compile", sizes(h)]);
          if (s.indicators.copies) facts.push(["Indicators", `${plural(s.indicators.copies, "copy", "copies")} of ${plural(s.indicators.atoms, "atom")}, ${plural(s.indicators.links, "link")}`]);
          facts.push(["Time", seconds(s.seconds)]);
          add(s, "Rewrite", facts, { code: [block("The program the layout compiles", s.program, "problog")] });
          break;
        }
        case "layout": {
          const facts = [["Strategy", code(s.strategy)], ["Layout", code(s.kind)]];
          if (s.strategy === "auto") facts.push(["Attempts", `${count(s.attempts)}, the trials in turns under doubling bounds`]);
          facts.push(["Vtree", `${plural(s.vtree.variables, "variable")}, depth ${count(s.vtree.depth)}`], ["Time", seconds(s.seconds) + (s.discardedSeconds > 0 ? `, and ${seconds(s.discardedSeconds)} in trials given up` : "")]);
          add(s, "Choose a layout", facts, { figure: vtreeSvg(s.vtree), code: s.notes.length ? [{ label: "auto's trials", text: s.notes.join("\n"), lang: null, more: "" }] : [] });
          break;
        }
        case "compile": {
          const t = s.totals;
          const facts = [["Formulation", `${code(s.formulation)}, projection ${code(s.projection)}`],
            ["Components", `${count(t.components)} (${count(t.cyclic)} cyclic${t.iterations ? `, ${plural(t.iterations, "round")} of fixpoint` : ""})`],
            ["Updates", count(t.updates)], ["Operations", ops(t.operations)],
            ["Largest formula", `${plural(t.largest, "pair")}${t.final ? `; the largest handed to the queries ${plural(t.final, "pair")}` : ""}`], ["Work", `${count(t.workUnits)} work units`], ["Time", seconds(s.seconds)]];
          add(s, "Compile", facts, { figure: compileSvg(s), table: componentsTable(s) });
          break;
        }
        case "evidence": {
          const facts = [["Observations", count(s.observations)], ["Evidence", `${plural(s.pairs, "pair")}`], ["Queries", `${count(s.queries)}, ${count(s.countedInCompile)} counted in the compile`],
            ["Largest query formula", plural(s.largest, "pair")], ["Work", `${count(s.workUnits)} work units`], ["Time", seconds(s.seconds)]];
          if (s.probability !== null) facts.splice(2, 0, ["P(e)", String(s.probability)]);
          const answered = stages.find((q) => q.stage === "queries" && q.part === s.part);
          add(s, "Evidence and queries", facts, { table: answered ? answersTable(answered.answers) : "" });
          break;
        }
        case "answers": {
          const facts = [];
          if (s.evidence) facts.push([s.evidence.z ? "Z" : "P(e)", `${s.evidence.probability.toExponential(6)}, log10 ${s.evidence.log10 === null ? "-inf" : s.evidence.log10.toFixed(6)}`]);
          add(s, "Answers", facts, { table: answersTable(s.answers, 50) });
          break;
        }
        case "error":
          add(s, "Error", [["Message", `<span class="bad">${esc(s.message)}</span>`], ...s.notes.map((n) => ["Note", esc(n)])]);
          break;
        default:
      }
    }
    return out;
  }

  // A step as HTML: its facts, its figure and table, and its code.
  function stepHtml(step) {
    let out = `<p class="trace-api"><span class="muted">API: </span>${code(step.api)}</p>`;
    if (step.facts.length) out += '<dl class="trace-facts">' + step.facts.map(([t, d]) => `<dt>${esc(t)}</dt><dd>${d}</dd>`).join("") + "</dl>";
    if (step.figure) out += `<div class="trace-figure">${step.figure}</div>`;
    out += step.table;
    for (const c of step.code) {
      const body = typeof tddlogCode !== "undefined" && c.lang ? tddlogCode.highlight(c.lang, c.text) : esc(c.text);
      out += `<p class="trace-label">${esc(c.label)}${c.more ? ` <span class="muted">(${esc(c.more)})</span>` : ""}</p><pre class="src trace-code">${body}</pre>`;
    }
    return out;
  }

  // The whole trace, each step one the reader opens (the playground's).
  function html(trace) {
    return steps(trace).map((s, i) => `<details class="trace-step"${i === 0 ? " open" : ""}><summary>${i + 1}. ${esc(s.title)}</summary>${stepHtml(s)}</details>`).join("");
  }

  return { VERSION, steps, stepHtml, html, vtreeSvg, compileSvg };
})();

if (typeof module !== "undefined") module.exports = tddlogTrace;
