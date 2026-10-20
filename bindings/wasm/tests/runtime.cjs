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
  const example = { problog: [["a(<b>)", 0.25]], evidence: 0.5 };
  const html = page.answersHtml(result, 4, example);
  assert.match(html, /a\(&lt;b&gt;\)/);
  assert.match(html, /<th class="p"[^>]*>ProbLog<\/th>/);
  assert.match(html, /P\(evidence\) <b>5\.0000e-1<\/b>/);
  assert.doesNotMatch(page.answersHtml(result, 4, null), /ProbLog/);
  assert.match(page.answersHtml({ ...result, error: "parse error: <x>" }, 4, null), /parse error: &lt;x&gt;/);
  assert.match(page.workHtml(result, "auto"), /note &amp; more/);
});
