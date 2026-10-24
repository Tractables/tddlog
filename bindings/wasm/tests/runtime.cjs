// The browser runtime without the module: the worker's calls into a stand-in
// for it, and how the page writes a result. `node --test tests/runtime.cjs`.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const path = require("node:path");
const { test } = require("node:test");

const root = path.join(__dirname, "..");
global.tddlogRuntime = require("../runtime.js");
const page = require("../app.js");

// worker.js in a context of its own, its module `solver`.
function worker(solver) {
  const messages = [];
  const context = vm.createContext({
    location: { search: "" }, performance, TextEncoder, JSON,
    createTddlog: (settings) => {
      solver.printErr = settings.printErr;
      return Promise.resolve(solver);
    },
    postMessage: (message) => messages.push(message),
  });
  context.importScripts = (...files) => {
    for (const file of files) if (file !== "tddlog.js") vm.runInContext(fs.readFileSync(path.join(root, file), "utf8"), context);
  };
  vm.runInContext(fs.readFileSync(path.join(root, "worker.js"), "utf8"), context);
  return { context, messages };
}

const settle = () => new Promise((resolve) => setImmediate(resolve));

test("the program and the strategy go on the heap and every allocation is freed", async () => {
  const freed = [];
  const heap = new Uint8Array(4 * 1024 * 1024);
  let next = 16;
  const solver = {
    HEAPU8: heap,
    _malloc: (n) => { const at = next; next += n; return at; },
    _free: (ptr) => freed.push(ptr),
    _tddlog_solve_json: (program, strategy, evidence) => {
      assert.equal(heap[program], "%".charCodeAt(0));
      assert.equal(new TextDecoder().decode(heap.subarray(strategy, strategy + 6)), "closed");
      assert.equal(evidence, 1);
      return 8;
    },
    _tddlog_string_free: (ptr) => freed.push(ptr),
    UTF8ToString: () => '{"answers":[],"evidence":null}',
  };
  const { context, messages } = worker(solver);
  await settle();
  // Objects made in the worker's context have its prototypes, so they are
  // compared as JSON.
  assert.equal(JSON.stringify(messages[0]), '{"ready":true}');
  context.onmessage({ data: { program: "%" + "x".repeat(3 * 1024 * 1024), strategy: "closed", evidence: true } });
  assert.equal(JSON.stringify(messages.at(-1).result), '{"answers":[],"evidence":null}', JSON.stringify(messages.at(-1)));
  assert.equal(freed.length, 3);
});

test("a module that aborts is reported, with the reason it wrote", async () => {
  const freed = [];
  const solver = {
    HEAPU8: new Uint8Array(128),
    _malloc: () => 16, _free: (ptr) => freed.push(ptr),
    _tddlog_solve_json() {
      this.printErr("memory allocation of 1073741824 bytes failed");
      throw new Error("Aborted()");
    },
    _tddlog_string_free: (ptr) => freed.push(ptr),
  };
  const { context, messages } = worker(solver);
  await settle();
  context.onmessage({ data: { program: "0.5::a.", strategy: "auto", evidence: false } });
  assert.match(messages.at(-1).failure, /ran out of memory\. A page can use at most 4 GB/);
  assert.equal(freed.length, 2, "both inputs freed");
});

test("load failures are reported to the page", async () => {
  for (const [importScripts, createTddlog] of [
    [() => { throw new Error("module unavailable"); }, undefined],
    [() => {}, () => Promise.reject(new Error("wasm unavailable"))],
  ]) {
    const messages = [];
    const context = vm.createContext({ location: { search: "" }, postMessage: (m) => messages.push(m), importScripts, createTddlog });
    vm.runInContext(fs.readFileSync(path.join(root, "worker.js"), "utf8"), context);
    await settle();
    assert.match(messages[0].loadError, /unavailable/);
  }
});

test("evidence must be a boolean, digits an integer from 0 to 17", () => {
  assert.throws(() => tddlogRuntime.solve({}, "", "auto", "false"), /true or false/);
  for (const value of ["", "-1", "1.5", "18", "x"]) assert.throws(() => tddlogRuntime.digits(value), /digits/);
  for (const value of ["0", "6", "17"]) assert.equal(tddlogRuntime.digits(value), Number(value));
});

test("probabilities are written as the command line writes them", () => {
  assert.equal(tddlogRuntime.probability(0.35714285714285715, 6), "0.357143");
  assert.equal(tddlogRuntime.probability(0, 6), "0.000000");
  assert.equal(tddlogRuntime.probability(5.7e-15, 3), "5.700e-15");
  assert.deepEqual(tddlogRuntime.evidence({ probability: 0.196, log10: Math.log10(0.196) }, 6), { scientific: "1.960000e-1", log10: "-0.707744" });
  assert.deepEqual(tddlogRuntime.evidence({ probability: 0, log10: null }, 2), { scientific: "0.00e+0", log10: "-inf" });
  // Under the least normal float the mantissa comes from the logarithm.
  assert.deepEqual(tddlogRuntime.evidence({ probability: 0, log10: -400.30103 }, 3), { scientific: "5.000e-401", log10: "-400.301" });
});

test("an edited program goes through the address and back", async () => {
  const program = "0.3::a.\nb :- a, \\+c.\nquery(b). % é\n".repeat(20);
  const code = await page.encodeProgram(program);
  assert.match(code, /^[zt][A-Za-z0-9_-]+$/);
  assert.equal(await page.decodeProgram(code), program);
  await assert.rejects(page.decodeProgram("q123"), /no program/);
});

test("the answers are escaped, and ProbLog's sit beside an example's", () => {
  const result = {
    answers: [{ query: "a(<b>)", probability: 0.25 }, { query: "\\+c", probability: 0 }],
    evidence: { probability: 0.5, log10: Math.log10(0.5) },
    groundSeconds: 0.001, compiling: null, notes: ["note & more"], parts: [],
  };
  const reference = page.exampleReference({ problog: [["a(<b>)", 0.25]], evidence: 0.5 });
  const html = page.answersHtml(result, 4, reference);
  assert.match(html, /a\(&lt;b&gt;\)/);
  assert.match(html, /<th class="p">ProbLog<\/th>/);
  assert.match(html, /<td class="p reference">0\.2500<\/td>/);
  assert.match(html, /P\(evidence\) <b>5\.0000e-1<\/b>, log10 <b>-0\.3010<\/b>; ProbLog <b>5\.0000e-1<\/b>/);
  assert.match(html, /<p class="note">ProbLog: ProbLog 2\.3\.0&#39;s answers/, "the reference's caption, visible");
  assert.doesNotMatch(page.answersHtml(result, 4, null), /ProbLog/);
  assert.match(page.answersHtml({ ...result, error: "parse error: <x>" }, 4, null), /parse error: &lt;x&gt;/);
  assert.match(page.workHtml(result, "auto"), /note &amp; more/);
  assert.equal(page.said(result, 110), "done in 110 ms: 2 answers");
  assert.equal(page.said({ ...result, error: "parse error: line 3: x\nmore" }, 0), "failed: parse error: line 3: x");
  assert.match(page.summarise({ ...result, parts: [] }, 0.5), /&lt;1 ms<\/b> in the browser/);
});

test("a long table shows its first 500 rows, and all on asking", () => {
  const answers = Array.from({ length: 600 }, (_, i) => ({ query: `a(${i})`, probability: 0.5 }));
  const result = { answers, evidence: null, groundSeconds: 0, compiling: null, notes: [], parts: [] };
  const rows = (html) => html.match(/<tr><td class="query">/g).length;
  assert.equal(rows(page.answersHtml(result, 6, null)), 500);
  assert.match(page.answersHtml(result, 6, null), /<button type="button" class="all">Show all 600<\/button>/);
  assert.equal(rows(page.answersHtml(result, 6, null, true)), 600);
  assert.doesNotMatch(page.answersHtml(result, 6, null, true), /Show all/);
});

test("an address names a benchmark by three plain names", () => {
  assert.deepEqual(page.benchPath("std/grid/grid4"), ["std", "grid", "grid4"]);
  assert.deepEqual(page.benchPath("uai_pr/csp/uai_CSP_12.1-x"), ["uai_pr", "csp", "uai_CSP_12.1-x"]);
  for (const bad of ["../x", "std/grid", "std/grid/grid4/x", "std/../grid4", "std/grid/..", "std/grid/.x", "std/gr id/x", "std/grid/%2e", "", "std//x"]) {
    assert.equal(page.benchPath(bad), null, bad);
  }
});

test("a benchmark's program: the one .pl file tddlog read, and the answers beside it", () => {
  const file = (name, more = {}) => ({ name, bytes: 10, sha: "0", ...more });
  const tab = (readers, ...files) => ({ lang: "ProbLog", readers, files });
  const one = tab(["tddlog", "ProbLog -k sdd"], file("g.pl"));
  assert.equal(page.tddlogFile({ tabs: [tab(["PITA"], file("g.lpad.pl")), one] }).file.name, "g.pl");
  assert.match(page.tddlogFile({ tabs: [tab(["Dice"], file("g.dice"))] }).refused, /do not show the program tddlog was given/);
  assert.match(page.tddlogFile({}).refused, /do not show/);
  assert.match(page.tddlogFile({ tabs: [tab(["tddlog"], file("g.pl"), file("g.pl"))] }).refused, /2 forms/);

  const ref = page.benchReference({ cols: ["reference", "tddlog", "PITA"], ref: "ProbLog -k ddnnf",
    rows: [["p", "0.1676566", "0.1676565957", ""], ["q", "nan", "0.5", "0.5"], ["r", "", "1", "1"]] });
  assert.equal(ref.column, "Reference");
  assert.equal(ref.caption, "Reference: ProbLog -k ddnnf, in the release run.");
  assert.deepEqual([...ref.values], [["p", 0.1676566]]);
  assert.equal(ref.evidence, null);
  const own = page.benchReference({ cols: ["tddlog", "Ace Xlog"], ref: null, log10z: true, rows: [["log10 Z", "-1.041392685", "-1.04"]] });
  assert.equal(own.column, "Command line");
  assert.deepEqual(own.values, new Map());
  assert.equal(own.evidence.log10, -1.041392685);
  assert.equal(page.benchReference({ cols: ["Alea.jl"], ref: null, rows: [] }), null);
  assert.equal(page.benchReference(undefined), null);
});

test("a benchmark runs as it opens where it is short and tddlog was quick natively", () => {
  const records = [{ families: [{ suites: [{ key: "std", systems: ["ace", "tddlog"], instances: [["grid4", 0.1, 0.02], ["grid9", 1, "timeout"], ["grid5", 1, null]] }] }] }];
  assert.equal(page.nativeRun(records, "std", "grid4"), 0.02);
  assert.equal(page.nativeRun(records, "std", "grid9"), "timeout");
  assert.equal(page.nativeRun(records, "std", "nothing"), undefined);
  assert.equal(page.nativeRun(records, "other", "grid4"), undefined);
  assert.equal(page.holdBack(650, 0.02), "");
  assert.equal(page.holdBack(650, 2), "");
  assert.match(page.holdBack(650, 14.2), /^tddlog took 14\.20 s natively in the release run; the browser is slower and has 4 GB at most\. Press Run/);
  assert.match(page.holdBack(650, "timeout"), /did not answer it natively in the release run \(timeout\)\. Press Run/);
  assert.match(page.holdBack(650, null), /no time of tddlog's for it/);
  assert.match(page.holdBack(2 * 1024 * 1024, 0.5), /^The program is 2\.0 MB\. Press Run/);
});

test("a benchmark's text is its file, by size and SHA-256, after any header", async () => {
  const program = "0.5::a.\nquery(a). % é\n";
  const bytes = new TextEncoder().encode(program);
  const sha = Buffer.from(await crypto.subtle.digest("SHA-256", bytes)).toString("hex");
  const file = { bytes: bytes.length, sha };
  assert.equal(await page.holdsFile(program, file), true);
  assert.equal(await page.holdsFile("% A licence's header.\n" + program, file), true);
  assert.equal(await page.holdsFile(program.replace("0.5", "0.6"), file), false);
  assert.equal(await page.holdsFile(program.slice(1), file), false);
});
