"use strict";

// The page: the examples, a benchmark of the results, the address bar, the
// editor, the worker the solver runs in, and the result.

// A line on each strategy under the menu, from the command line's --help.
const STRATEGY_NOTES = {
  auto: "Trials of the layouts below over rewrites of the program, under bounds that double until one finishes.",
  td: "Right-linear over a fact order from a tree decomposition.",
  closed: "Components in an order found by local search, each on a right-linear spine.",
  families: "A tree decomposition of the program read through indicators.",
  "families:0": "families without the decomposer's search.",
  dynamic: "Right-linear, changed during the compile by a search of rotations.",
  program: "Right-linear in the order the program states its facts.",
  "program:fanin": "Right-linear in the fan-in order of the rules from the queries and evidence, interleaved over them.",
  decision: "The tree of the decisions the grounder read facts as guards in, its contexts and the atoms read outside it named; fails for a program with none.",
};

// A program whose code in the address would be longer than this goes in no
// address; Download saves it instead.
const ADDRESS_MOST = 64 * 1024;
// A program longer than this is shown without colours, which would slow
// down typing in it.
const PLAIN_ABOVE = 256 * 1024;
// The answers shown before "Show all".
const ROWS_SHOWN = 500;
// A benchmark of the results runs as it opens where its program is at most
// AUTO_BYTES long and tddlog answered it natively within AUTO_SECONDS.
const AUTO_BYTES = 1024 * 1024;
const AUTO_SECONDS = 2;

// The host keeps the page's files in caches for a while, so the workflow
// stamps the references in index.html with a hash of the files the pages load
// (app.js?v=<hash>), and the stamp goes on to every file the page loads: a
// new page never runs with an older file a browser still holds. Opened by
// hand there is no stamp.
const stamp = typeof document === "undefined" || document.currentScript === null
  ? ""
  : new URL(document.currentScript.src).search;

// ------------------------------------------------------------- the address

// An edited program goes in the address after `#program=`, so that a link
// reproduces it: deflated where the browser can (`z`), as it is otherwise
// (`t`), then base64url. The fragment never reaches the server, so the
// length of a request line does not bound the program.
function base64url(bytes) {
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64url(text) {
  const binary = atob(text.replace(/-/g, "+").replace(/_/g, "/"));
  return Uint8Array.from(binary, (c) => c.charCodeAt(0));
}

async function through(bytes, stream) {
  return new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(stream)).arrayBuffer());
}

async function encodeProgram(text) {
  const bytes = new TextEncoder().encode(text);
  try {
    return "z" + base64url(await through(bytes, new CompressionStream("deflate")));
  } catch (failure) {
    return "t" + base64url(bytes);
  }
}

async function decodeProgram(code) {
  const bytes = fromBase64url(code.slice(1));
  if (code[0] === "t") return new TextDecoder().decode(bytes);
  if (code[0] === "z") return new TextDecoder().decode(await through(bytes, new DecompressionStream("deflate")));
  throw new Error("the address holds no program");
}

// ------------------------------------------------------------- formatting

function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

const count = (n) => n.toLocaleString("en-US");

function duration(seconds) {
  if (seconds < 0.001) return "<1 ms";
  if (seconds < 1) return `${Math.round(seconds * 1000)} ms`;
  return `${seconds.toFixed(2)} s`;
}

// The spaces lay nothing out (the figures are flex items) but keep the
// text readable as text: "1,306 instances in the release run 1,198 ...".
const stat = (figure, word) => `<span class="stat"><b>${figure}</b> ${word}</span> `;

// The figures of a result: the queries, the layout that answered, the
// largest formula of the run, and the time the call took in this browser.
// The guide says what a layout and a pair are.
function summarise(result, elapsed) {
  const parts = result.parts;
  const largest = Math.max(0, ...parts.map((p) => Math.max(p.largestPairs, p.queryPairs)));
  const layouts = [...new Set(parts.map((p) => p.trial))].join(", ");
  let out = "";
  if (result.answers !== null) out += stat(count(result.answers.length), result.answers.length === 1 ? "query" : "queries");
  if (layouts) out += stat(escapeHtml(layouts), '<a href="guide/#terms">layout</a>');
  if (parts.length > 0) out += stat(count(largest), '<a href="guide/#terms">pairs</a> at most');
  out += stat(escapeHtml(duration(elapsed / 1000)), "in the browser");
  return out;
}

// The line the status announces when a run ends.
function said(result, elapsed) {
  if (result.error !== undefined) return `failed: ${result.error.split("\n")[0]}`;
  const time = duration(elapsed / 1000);
  if (result.answers === null) return `done in ${time}: the evidence is inconsistent`;
  const n = result.answers.length;
  return `done in ${time}: ${n === 0 ? "no query" : `${count(n)} ${n === 1 ? "answer" : "answers"}`}`;
}

// The answers beside an example's: ProbLog's.
function exampleReference(example) {
  return {
    column: "ProbLog",
    caption: "ProbLog: ProbLog 2.3.0's answers (-k sdd, one query at a time), as the repository's tests record them.",
    values: new Map(example.problog),
    evidence: { probability: example.evidence, log10: Math.log10(example.evidence) },
  };
}

// The digits after the point a cell of the results' answers was written
// with (in scientific notation, its mantissa's), where the cell is in the
// notation the page writes its value in (tddlogRuntime.probability); else
// undefined.
function writtenDigits(cell, value) {
  const [mantissa, exponent] = cell.toLowerCase().split("e");
  if ((exponent !== undefined) !== (value > 0 && value < 1e-4)) return undefined;
  const point = mantissa.indexOf(".");
  return point === -1 ? 0 : mantissa.length - point - 1;
}

// The answers beside a benchmark's: the release run's reference where it
// has one, else tddlog's own there, from the command line. A log10 Z row is
// the logarithm of P(evidence).
function benchReference(answers) {
  if (!answers) return null;
  const at = answers.cols.indexOf(answers.ref ? "reference" : "tddlog");
  if (at === -1) return null;
  const values = new Map();
  const written = new Map();
  let evidence = null;
  for (const [query, ...cells] of answers.rows) {
    const value = cells[at] === "" ? NaN : Number(cells[at]);
    if (!Number.isFinite(value)) continue;
    if (answers.log10z) evidence = { probability: 10 ** value, log10: value };
    else {
      values.set(query, value);
      written.set(query, writtenDigits(cells[at], value));
    }
  }
  return answers.ref
    ? { column: "Reference", caption: `Reference: ${answers.ref}, in the release run.`, values, written, evidence }
    : { column: "Command line", caption: "Command line: tddlog's answers in the release run.", values, written, evidence };
}

// The answers as a table, each with its probability as a bar, and beside
// them the `reference` where the program has one: null, or { column,
// caption, values (a Map from each query to its probability), written
// (where given, a Map from a query to the digits after the point its value
// was written with, which the column does not exceed), evidence (a
// P(evidence) as tddlog gives it, or null) }. The first ROWS_SHOWN rows
// unless `all`.
function answersHtml(result, digits, reference, all = false) {
  if (result.error !== undefined) return `<pre class="failed">${escapeHtml(result.error)}</pre>`;
  let out = "";
  if (result.answers === null) {
    out += '<p class="note">The evidence is inconsistent: its probability is zero, and no query has an answer.</p>';
  } else if (result.answers.length === 0) {
    out += '<p class="note">The program asks no query.</p>';
  } else {
    const shown = all ? result.answers : result.answers.slice(0, ROWS_SHOWN);
    out += '<table><thead><tr><th>Query</th><th class="p">Probability</th><th></th>';
    if (reference !== null) out += `<th class="p">${escapeHtml(reference.column)}</th>`;
    out += "</tr></thead><tbody>";
    for (const { query, probability } of shown) {
      const width = Math.max(0, Math.min(1, probability)) * 100;
      out += `<tr><td class="query">${escapeHtml(query)}</td>`;
      out += `<td class="p" title="${probability}">${tddlogRuntime.probability(probability, digits)}</td>`;
      out += `<td class="bar"><i class="${width === 0 ? "zero" : ""}" style="width:${width}%"></i></td>`;
      if (reference !== null) {
        const p = reference.values.get(query);
        const written = reference.written?.get(query);
        out += `<td class="p reference">${p === undefined ? "" : tddlogRuntime.probability(p, written === undefined ? digits : Math.min(digits, written))}</td>`;
      }
      out += "</tr>";
    }
    out += "</tbody></table>";
    if (shown.length < result.answers.length) {
      out += `<p><button type="button" class="all">Show all ${count(result.answers.length)}</button></p>`;
    }
  }
  if (result.evidence !== null) {
    const e = tddlogRuntime.evidence(result.evidence, digits);
    // A Markov network's is its partition function given the evidence.
    let line = `${result.evidence.z ? "Z" : "P(evidence)"} <b>${e.scientific}</b>, log10 <b>${e.log10}</b>`;
    if (reference?.evidence) {
      const r = tddlogRuntime.evidence(reference.evidence, digits);
      line += `; ${escapeHtml(reference.column)} <b>${r.scientific}</b>, log10 <b>${r.log10}</b>`;
    }
    out += `<p class="evidence">${line}</p>`;
  }
  if (reference !== null) out += `<p class="note">${escapeHtml(reference.caption)}</p>`;
  return out;
}

function groundText(g) {
  return `${count(g.atoms)} atoms (${count(g.facts)} facts, ${count(g.derived)} derived), ${count(g.rules)} rules`;
}

// What tddlog did: for each part of the program answered on its own, the
// layout that answered and how auto came to it, the program and the
// formulas in pairs, and the seconds of each stage.
function workHtml(result, strategy) {
  const terms = (rows) => "<dl>" + rows.map(([t, d]) => `<dt>${t}</dt><dd>${d}</dd>`).join("") + "</dl>";
  let out = "";
  const parts = result.parts;
  if (parts.length === 0 && result.compiling !== null) {
    out += "<h3>What tddlog did</h3>" + terms([["Ground program", groundText(result.compiling)]]);
  }
  parts.forEach((p, i) => {
    out += `<h3>${parts.length > 1 ? `Part ${i + 1} of ${parts.length}` : "What tddlog did"}</h3>`;
    const layout = strategy === "auto"
      ? `<code>${escapeHtml(p.trial)}</code>, the trial <code>auto</code> answered with, after ${count(p.attempts)} ${p.attempts === 1 ? "compilation" : "compilations"}`
      : `<code>${escapeHtml(p.trial)}</code>`;
    const stages = [
      `ground ${duration(result.groundSeconds)}`,
      `rewrites and vtree ${duration(p.rewriteSeconds + p.vtreeSeconds)}`,
      `compile ${duration(p.compileSeconds)}`,
      `evidence and queries ${duration(p.querySeconds)}`,
    ];
    if (p.discardedSeconds > 0) stages.push(`trials given up ${duration(p.discardedSeconds)}`);
    out += terms([
      ["Layout", layout],
      ["Ground program", groundText(p.ground)],
      ["Vtree", `over ${count(p.leaves)} ${p.leaves === 1 ? "variable" : "variables"}`],
      ["Compile", `${count(p.components)} components (${count(p.cyclic)} cyclic); largest formula ${count(p.largestPairs)} pairs, ${count(p.finalPairs)} at the end`],
      ["Evidence and queries", `evidence ${count(p.evidencePairs)} pairs; largest formula ${count(p.queryPairs)} pairs`],
      ["Time", escapeHtml(stages.join(", "))],
    ]);
  });
  if (result.notes.length > 0) {
    out += "<h3>Notes</h3><dl>" + result.notes.map((n) => `<dd>${escapeHtml(n)}</dd>`).join("") + "</dl>";
  }
  return out;
}

// ------------------------------------------------------ a benchmark's program

// The suite, family and instance an address names
// (#bench=SUITE/FAMILY/INSTANCE), each of letters, digits, _, . and - and
// not starting with . or -; else null.
function benchPath(value) {
  const parts = String(value).split("/");
  return parts.length === 3 && parts.every((p) => /^[A-Za-z0-9_][A-Za-z0-9_.-]*$/.test(p)) ? parts : null;
}

// The file tddlog read of an instance of the results: the one .pl file of
// the tab tddlog is a reader of. Else why there is none to open.
function tddlogFile(instance) {
  const files = (instance.tabs ?? []).filter((tab) => tab.readers.includes("tddlog"))
    .flatMap((tab) => tab.files.filter((file) => file.name.endsWith(".pl")).map((file) => ({ tab, file })));
  if (files.length === 1) return files[0];
  return {
    refused: files.length === 0
      ? "the results do not show the program tddlog was given."
      : `the results show ${files.length} forms of the program; its results say which one tddlog was scored on.`,
  };
}

// Why a benchmark does not run as it opens, or "" where it does. `native`
// is tddlog's time in the release run, why it has none ("timeout", ...), or
// undefined where the records do not have it.
function holdBack(bytes, native) {
  const why = [];
  if (bytes > AUTO_BYTES) why.push(`The program is ${(bytes / (1024 * 1024)).toFixed(1)} MB.`);
  if (typeof native === "string") why.push(`tddlog did not answer it natively in the release run (${native}).`);
  else if (typeof native !== "number") why.push("The release run has no time of tddlog's for it.");
  else if (native > AUTO_SECONDS) why.push(`tddlog took ${duration(native)} natively in the release run; the browser is slower and has 4 GB at most.`);
  return why.length === 0 ? "" : `${why.join(" ")} Press Run to try it here.`;
}

// tddlog's record of an instance in the release run: its seconds, why it
// has none, or undefined where `records` (the results' suites) lack it.
function nativeRun(records, suite, instance) {
  const s = records.flatMap((r) => r.families.flatMap((f) => f.suites)).find((x) => x.key === suite);
  const row = s?.instances.find(([name]) => name === instance);
  const own = s === undefined ? -1 : s.systems.indexOf("tddlog");
  return row === undefined || own === -1 ? undefined : row[own + 1];
}

// The text of a file the site keeps on its own, gzipped where its name ends
// in .gz.
async function keptText(src) {
  const response = await fetch(`results/benchmarks/${src}`);
  if (!response.ok) throw new Error(`${src}: ${response.status}`);
  if (!src.endsWith(".gz")) return response.text();
  return new Response(response.body.pipeThrough(new DecompressionStream("gzip"))).text();
}

// Whether a text ends with the file the results name, by its size and
// SHA-256: the site may put a header above a file (a licence's).
async function holdsFile(text, file) {
  const bytes = new TextEncoder().encode(text);
  if (bytes.length < file.bytes) return false;
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes.subarray(bytes.length - file.bytes)));
  return Array.from(digest, (b) => b.toString(16).padStart(2, "0")).join("") === file.sha;
}

// The program tddlog was given for an instance in the release run, with the
// answers that go beside it, or why the page cannot run it (`refused`, HTML:
// a held file's reason comes from the results).
async function benchmark([suite, family, instance]) {
  const response = await fetch(`results/benchmarks/${suite}/${family}.json${stamp}`);
  if (response.status === 404) return { refused: "the results have no such benchmark." };
  if (!response.ok) throw new Error(`${suite}/${family}.json: ${response.status}`);
  const fam = await response.json();
  const inst = fam.instances.find((i) => i.name === instance);
  if (inst === undefined) return { refused: "the results have no such benchmark." };
  const chosen = tddlogFile(inst);
  if (chosen.refused) return chosen;
  const { tab, file } = chosen;
  if (file.held) return { refused: file.held };
  if (file.part) {
    return { refused: `the site keeps the first ${count(file.part)} of its ${count(file.bytes)} bytes, which do not run on their own.` };
  }
  if (fam.texts[file.sha] === undefined && !file.src) return { refused: "the results do not keep its text." };
  const text = fam.texts[file.sha] ?? await keptText(file.src);
  if (!(await holdsFile(text, file))) return { refused: "the text the site keeps does not match the file's SHA-256." };
  return { tab, file, text, reference: benchReference(inst.answers), log10z: inst.answers?.log10z === true };
}

// ------------------------------------------------------- the results at a glance

// The release run's records, which the results page reads (results/): each
// suite's instances, with each system's time or why it has none. Loaded
// once, for the glance and for a benchmark's time.
let recordsLoaded = null;
function records() {
  const load = async (file) => {
    const response = await fetch(`results/${file}${stamp}`);
    if (!response.ok) throw new Error(`${file}: ${response.status}`);
    return response.json();
  };
  recordsLoaded ??= Promise.all(["suites.json", "ppl.json"].map((file) => load(file).catch(() => null)))
    .then((parts) => parts.filter(Boolean));
  return recordsLoaded;
}

// The release run's instances, those tddlog answers and those it alone
// answers.
async function glance() {
  let all = 0, answered = 0, alone = 0;
  for (const suite of (await records()).flatMap((p) => p.families.flatMap((f) => f.suites))) {
    const own = suite.systems.indexOf("tddlog");
    for (const [, ...times] of suite.instances) {
      all += 1;
      if (typeof times[own] !== "number") continue;
      answered += 1;
      if (times.every((t, i) => i === own || typeof t !== "number")) alone += 1;
    }
  }
  if (all === 0) return;
  document.getElementById("glance-stats").innerHTML = stat(count(all), "instances in the release run")
    + stat(count(answered), "answered by tddlog") + stat(count(alone), "answered by tddlog alone");
  document.getElementById("glance").hidden = false;
}

// --------------------------------------------------------------- the page

if (typeof document !== "undefined") {
  const element = (id) => document.getElementById(id);
  const editor = element("program");
  const shade = element("program-hl");
  const sourceNote = element("source");

  // The strategies, auto first.
  for (const name of tddlogRuntime.strategies) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    element("strategy").append(option);
  }
  const showStrategy = () => {
    element("strategy-note").textContent = STRATEGY_NOTES[element("strategy").value] ?? "";
  };
  element("strategy").addEventListener("change", showStrategy);

  // The formats, ProbLog first; an example, a file or the address sets it,
  // and the reader may change it.
  for (const [name, label] of tddlogRuntime.formats) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = label;
    element("format").append(option);
  }
  const format = element("format");

  // ------------------------------------------------------------ the editor

  // The program in colours: the textarea's text is transparent, and the
  // same text in code.js's colours lies under it, moved as it scrolls. A
  // long program is shown plain. A parse error's line is marked under it.
  let errorLine = 0;
  function follow() {
    shade.style.transform = `translate(${-editor.scrollLeft}px, ${-editor.scrollTop}px)`;
  }
  function paint() {
    const plain = editor.value.length > PLAIN_ABOVE;
    editor.parentElement.classList.toggle("plain", plain);
    // The newline keeps a last empty line, where the caret can be, as tall
    // in the colours as in the textarea.
    let html = plain ? "" : tddlogCode.highlight("problog", editor.value) + "\n";
    if (html && errorLine > 0) html = `<span class="error-line" style="top:${(errorLine - 1) * 1.5}em"></span>` + html;
    shade.innerHTML = html;
    follow();
  }
  let painting = 0;
  editor.addEventListener("scroll", follow, { passive: true });

  // A button per example; the one whose program is in the editor, unedited,
  // is pressed.
  const examples = element("examples");
  tddlogExamples.forEach(({ name }, i) => {
    const chip = document.createElement("button");
    chip.type = "button";
    chip.textContent = name;
    chip.dataset.example = i;
    chip.setAttribute("aria-pressed", "false");
    examples.append(chip);
  });

  // Where the program in the editor comes from while it is unedited: an
  // example ({ example: its index }) or a benchmark of the results
  // ({ bench: SUITE/FAMILY/INSTANCE }), each with the answers that go
  // beside tddlog's (`reference`); null for a program of the reader's own.
  let source = null;
  // The name Download gives the program: the file it came from.
  let fileName = "program.pl";
  function choose(from) {
    source = from;
    for (const chip of examples.children) {
      chip.setAttribute("aria-pressed", String(from?.example === Number(chip.dataset.example)));
    }
  }

  // Every program the editor shows comes through here, with its format.
  function setProgram(text, from, name, as = "problog") {
    format.value = as;
    editor.value = text;
    editor.scrollTop = 0;
    editor.scrollLeft = 0;
    errorLine = 0;
    paint();
    choose(from);
    fileName = name;
  }

  // Where a benchmark came from, or why it did not open (HTML).
  function showSource(html) {
    sourceNote.innerHTML = html;
    sourceNote.hidden = html === "";
  }

  // A parse error names its line ("parse error: line 3: ..."), which is
  // marked, scrolled to, and the caret put at its start, while the editor
  // holds the program that ran.
  function markError(error, program) {
    const at = /^parse error: line (\d+):/.exec(error ?? "");
    if (at === null || editor.value !== program) return;
    errorLine = Number(at[1]);
    paint();
    const start = editor.value.split("\n", errorLine - 1).join("\n").length + (errorLine > 1 ? 1 : 0);
    editor.setSelectionRange(start, start);
    const height = parseFloat(getComputedStyle(editor).lineHeight);
    editor.scrollTop = Math.max(0, (errorLine - 1) * height - editor.clientHeight / 3);
  }

  // Choosing an example runs it once the solver is there: the point of the
  // buttons is to see what comes out.
  let loading = 0;
  async function loadExample(i) {
    const ticket = ++loading;
    let text;
    try {
      const response = await fetch(tddlogExamplePath(tddlogExamples[i]) + stamp);
      if (!response.ok) throw new Error(`${response.status}`);
      text = await response.text();
    } catch (failure) {
      status(`the example did not load: ${failure}`);
      return;
    }
    if (ticket !== loading) return;
    const example = tddlogExamples[i];
    setProgram(text, { example: i, reference: exampleReference(example) }, tddlogExamplePath(example).split("/").pop(), example.format);
    showSource("");
    requestRun();
  }
  examples.addEventListener("click", (event) => {
    const chip = event.target.closest("button");
    if (chip !== null) loadExample(Number(chip.dataset.example));
  });

  // A benchmark of the results (#bench=SUITE/FAMILY/INSTANCE, the results
  // page's "Open in the playground"): the program tddlog was given in the
  // release run, run as it opens where that is quick, with the release
  // run's answers beside tddlog's. A file the site does not show whole
  // opens nothing, and the note says why.
  async function openBench(value) {
    const ticket = ++loading;
    const path = benchPath(value);
    const back = path === null ? "results/" : `results/#/${path.join("/")}`;
    const name = path === null ? "" : `<b>${escapeHtml(path[2])}</b> from ${escapeHtml(`${path[0]}/${path[1]}`)}: `;
    setProgram("", null, "program.pl");
    showSource(`${name}loading.`);
    let bench, native;
    try {
      bench = path === null ? { refused: "The address names no benchmark (#bench=SUITE/FAMILY/INSTANCE)." } : await benchmark(path);
      if (!bench.refused) native = nativeRun(await records(), path[0], path[2]);
    } catch (failure) {
      bench = { refused: escapeHtml(`it did not load (${failure.message}).`) };
    }
    if (ticket !== loading) return;
    if (bench.refused) {
      showSource(`${name}${bench.refused} <a href="${back}">${path === null ? "The results" : "Its results"}</a>.`);
      return;
    }
    setProgram(bench.text, { bench: path.join("/"), reference: bench.reference }, bench.file.name);
    if (bench.log10z) element("evidence").checked = true;
    const wait = holdBack(bench.file.bytes, native);
    showSource(`${name}the program tddlog was given in the release run (${escapeHtml(bench.tab.lang)}, ${count(bench.file.bytes)} bytes). `
      + `<a href="${back}">Back to its results</a>.${wait ? ` ${escapeHtml(wait)}` : ""}`);
    if (!wait) requestRun();
  }

  // A file opened or dropped on the editor.
  async function openFile(file) {
    const ticket = ++loading;
    const text = await file.text();
    if (ticket !== loading) return;
    setProgram(text, null, file.name, tddlogRuntime.formatOf(file.name));
    showSource("");
    saveSoon();
  }
  element("open").addEventListener("click", () => element("file").click());
  element("file").addEventListener("change", () => {
    const [file] = element("file").files;
    if (file !== undefined) openFile(file);
    element("file").value = "";
  });
  editor.parentElement.addEventListener("dragover", (event) => {
    if (event.dataTransfer.types.includes("Files")) event.preventDefault();
  });
  editor.parentElement.addEventListener("drop", (event) => {
    const [file] = event.dataTransfer.files;
    if (file === undefined) return;
    event.preventDefault();
    openFile(file);
  });
  element("download").addEventListener("click", () => {
    const link = document.createElement("a");
    link.href = URL.createObjectURL(new Blob([editor.value], { type: "text/plain" }));
    link.download = fileName;
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 1000);
  });

  // Once edited, the text is no longer the example or benchmark it started
  // from; the address follows the edits a second after the last.
  editor.addEventListener("input", () => {
    loading++;
    if (source?.bench !== undefined) sourceNote.append(" Edited since.");
    if (source !== null) choose(null);
    errorLine = 0;
    painting ||= requestAnimationFrame(() => {
      painting = 0;
      paint();
    });
    saveSoon();
  });
  // Another format reads the text as another program.
  format.addEventListener("change", () => {
    loading++;
    if (source?.bench !== undefined) sourceNote.append(" Edited since.");
    if (source !== null) choose(null);
    saveSoon();
  });
  editor.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      requestRun();
    }
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && running) cancel();
  });

  // ----------------------------------------------------------- the address

  // The address carries the choices, so a result can be linked to:
  // ?example=<key>&strategy=<s>&evidence=1&digits=<n>, an unedited benchmark
  // after #bench=, and an edited program after #program=, with &format=<f>
  // where it is not ProbLog. It is read once
  // when the page opens, and written at every run and after edits.
  function readAddress() {
    const params = new URLSearchParams(location.search);
    const strategy = params.get("strategy");
    if (tddlogRuntime.strategies.includes(strategy)) element("strategy").value = strategy;
    element("evidence").checked = params.get("evidence") === "1";
    if (params.get("digits") !== null) element("digits").value = params.get("digits");
    const hash = new URLSearchParams(location.hash.slice(1));
    return {
      example: tddlogExamples.findIndex(({ key }) => key === params.get("example")),
      format: tddlogRuntime.formats.some(([name]) => name === params.get("format")) ? params.get("format") : "problog",
      program: hash.get("program"),
      bench: hash.get("bench"),
    };
  }
  // A write waits for the program's code; a later write takes its place.
  let addressing = 0;
  async function writeAddress(settings) {
    const ticket = ++addressing;
    const params = new URLSearchParams();
    if (source?.example !== undefined) params.set("example", tddlogExamples[source.example].key);
    if (settings.strategy !== "auto") params.set("strategy", settings.strategy);
    if (settings.evidence) params.set("evidence", "1");
    if (settings.digits !== 6) params.set("digits", String(settings.digits));
    if (source === null && settings.format !== "problog") params.set("format", settings.format);
    const query = params.toString();
    let hash = "";
    if (source?.bench !== undefined) hash = `#bench=${source.bench}`;
    else if (source === null) {
      const code = await encodeProgram(settings.program);
      if (ticket !== addressing) return;
      if (code.length <= ADDRESS_MOST) hash = `#program=${code}`;
    }
    const tooLong = source === null && hash === "";
    history.replaceState(null, "", location.pathname + (query ? `?${query}` : "") + hash);
    element("link").hidden = tooLong;
    element("too-long").hidden = !tooLong;
  }
  let saving = 0;
  function saveSoon() {
    clearTimeout(saving);
    saving = setTimeout(() => {
      try {
        writeAddress(settings());
      } catch (failure) {
        // digits out of range: the run says so
      }
    }, 1000);
  }
  function flash(button, text) {
    const label = button.textContent;
    button.textContent = text;
    setTimeout(() => {
      button.textContent = label;
    }, 1500);
  }
  function copy(button, text) {
    navigator.clipboard.writeText(text).then(() => flash(button, "Copied"), () => flash(button, "Not copied"));
  }
  element("link").addEventListener("click", () => copy(element("link"), location.href));
  // The answers on show as tab-separated text, as the page writes them.
  element("copy").addEventListener("click", () => {
    const { result, settings: { digits } } = shown;
    const lines = [["query", "probability"], ...(result.answers ?? []).map(({ query, probability }) => [query, tddlogRuntime.probability(probability, digits)])];
    if (result.evidence !== null) lines.push([result.evidence.z ? "Z" : "P(evidence)", tddlogRuntime.evidence(result.evidence, digits).scientific]);
    copy(element("copy"), lines.map((line) => line.join("\t")).join("\n") + "\n");
  });

  // ------------------------------------------------------------ the solver

  // The solver runs in a worker, so the page stays live during a run, the
  // clock can count the seconds, and a run can be stopped: Cancel ends the
  // worker and starts another, which takes a moment to load the module. A
  // run asked for before the worker is ready, or while another runs, starts
  // once the worker is.
  let worker = null;
  let ready = false;
  let running = false;
  let runWhenReady = false;
  let ticking = 0;
  // The results at a glance load once the solver is ready, when the page
  // is idle.
  let glanced = false;
  // The settings of the run on show, and its result, which a change of
  // digits or "Show all" writes again.
  let shown = null;
  let allRows = false;

  // The status is announced: one line at a time, and the clock beside it
  // is not.
  const status = (text) => {
    element("status").textContent = text;
    element("clock").textContent = "";
  };

  function startSolver() {
    ready = false;
    element("run").disabled = true;
    worker = new Worker("worker.js" + stamp);
    worker.onmessage = (event) => {
      const data = event.data;
      if (data.ready === true) {
        ready = true;
        element("run").disabled = false;
        if (!running) status("ready");
        if (!glanced) {
          glanced = true;
          (window.requestIdleCallback ?? setTimeout)(() => glance().catch(() => {}));
        }
        if (runWhenReady) {
          runWhenReady = false;
          run();
        }
      } else if (data.loadError !== undefined) {
        loadFailed(data.loadError);
      } else if (data.failure !== undefined) {
        // The module may have aborted, and an aborted module fails every
        // call after it, so the worker is replaced as Cancel does.
        const settings = shown === null ? null : shown.settings;
        stop();
        show({ error: data.failure, groundSeconds: 0, compiling: null, notes: [], parts: [], answers: null, evidence: null }, 0, settings);
        status("failed; loading the solver again");
      } else {
        finish(data.result, data.elapsed);
      }
    };
    worker.onerror = (event) => loadFailed(event.message);
  }

  function loadFailed(message) {
    worker.terminate();
    ready = false;
    runWhenReady = false;
    settle();
    element("run").disabled = true;
    console.error("The solver did not load", message);
    status("the solver did not load; reload the page to retry");
  }

  function requestRun() {
    if (running) {
      stop();
      runWhenReady = true;
    } else if (ready) {
      run();
    } else {
      runWhenReady = true;
    }
  }

  function stop() {
    worker.terminate();
    settle();
    startSolver();
  }

  function cancel() {
    if (!running) return;
    stop();
    status("stopped; loading the solver");
  }

  function settle() {
    running = false;
    clearInterval(ticking);
    document.body.classList.remove("busy");
    element("cancel").hidden = true;
    element("run").hidden = false;
  }

  function settings() {
    return {
      program: editor.value,
      format: format.value,
      strategy: element("strategy").value,
      evidence: element("evidence").checked,
      digits: tddlogRuntime.digits(element("digits").value),
      reference: source === null ? null : source.reference,
    };
  }

  function run() {
    let chosenSettings;
    try {
      chosenSettings = settings();
    } catch (failure) {
      status(failure.message);
      return;
    }
    clearTimeout(saving);
    writeAddress(chosenSettings);
    running = true;
    allRows = false;
    shown = { settings: chosenSettings, result: null, elapsed: 0 };
    const started = performance.now();
    status("running");
    ticking = setInterval(() => {
      element("clock").textContent = `${Math.round((performance.now() - started) / 1000)} s`;
    }, 1000);
    element("run").hidden = true;
    element("cancel").hidden = false;
    document.body.classList.add("busy");
    const { program, strategy, evidence, format } = chosenSettings;
    worker.postMessage({ program, strategy, evidence, format });
  }

  element("run").addEventListener("click", requestRun);
  element("cancel").addEventListener("click", cancel);

  function show(result, elapsed, chosenSettings) {
    shown = { settings: chosenSettings, result, elapsed };
    const digits = chosenSettings === null ? 6 : chosenSettings.digits;
    const strategy = chosenSettings === null ? "auto" : chosenSettings.strategy;
    const reference = chosenSettings === null ? null : chosenSettings.reference;
    element("summary").innerHTML = result.error === undefined ? summarise(result, elapsed) : "";
    element("answers").innerHTML = answersHtml(result, digits, reference, allRows);
    element("work").innerHTML = workHtml(result, strategy);
    element("copy").hidden = chosenSettings === null || result.error !== undefined || (!result.answers?.length && result.evidence === null);
    const trials = result.parts.flatMap((p) => p.trials);
    element("trials").hidden = trials.length === 0;
    element("trial-notes").textContent = trials.join("\n");
  }

  function finish(result, elapsed) {
    settle();
    show(result, elapsed, shown.settings);
    markError(result.error, shown.settings.program);
    status(said(result, elapsed));
  }

  element("answers").addEventListener("click", (event) => {
    if (event.target.closest("button.all") === null) return;
    allRows = true;
    show(shown.result, shown.elapsed, shown.settings);
  });

  // Digits change how the answers on show are written, not the answers.
  element("digits").addEventListener("input", () => {
    if (shown === null || shown.result === null) return;
    try {
      const digits = tddlogRuntime.digits(element("digits").value);
      show(shown.result, shown.elapsed, { ...shown.settings, digits });
    } catch (failure) {
      status(failure.message);
    }
  });

  // The page opens on the benchmark or the program in the address, else the
  // example it names, else the first.
  startSolver();
  const opening = readAddress();
  showStrategy();
  if (opening.bench !== null) {
    openBench(opening.bench);
  } else if (opening.program !== null) {
    decodeProgram(opening.program).then(
      (text) => {
        const extension = tddlogRuntime.formats.find(([name]) => name === opening.format)[2][0] ?? ".pl";
        setProgram(text, null, `program${extension}`, opening.format);
        requestRun();
      },
      () => loadExample(0),
    );
  } else {
    loadExample(opening.example === -1 ? 0 : opening.example);
  }
}

if (typeof module !== "undefined") {
  module.exports = {
    encodeProgram, decodeProgram, answersHtml, workHtml, summarise, said, escapeHtml,
    exampleReference, benchReference, benchPath, tddlogFile, holdBack, nativeRun, holdsFile,
  };
}
