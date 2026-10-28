// Every example of the page, run through the built module, against
// ProbLog's answers and P(e) as examples.js records them; and the module's
// refusals. `node tests/examples.cjs <path to tddlog.js>`.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const runtime = require("../runtime.js");
const { tddlogExamples, tddlogExamplePath } = require("../examples.js");
const createTddlog = require(path.resolve(process.argv[2]));

const root = path.join(__dirname, "../../..");
// ProbLog's answers are recorded to 12 significant digits.
const TOLERANCE = 1e-9;

(async () => {
  const stderr = [];
  const solver = await createTddlog({ print: () => {}, printErr: (line) => stderr.push(line) });

  const served = tddlogExamples.map(tddlogExamplePath);
  assert.equal(new Set(served).size, served.length, "two examples are served at one path");
  assert.equal(new Set(tddlogExamples.map((e) => e.key)).size, tddlogExamples.length, "two examples have one key");

  for (const example of tddlogExamples) {
    const program = fs.readFileSync(path.join(root, example.file), "utf8");
    for (const strategy of ["auto", "closed"]) {
      for (const evidence of [false, true]) {
        const what = `${example.key} (${strategy}, P(e) ${evidence})`;
        const result = runtime.solve(solver, program, strategy, evidence, example.format ?? "problog");
        assert.equal(result.error, undefined, `${what}: ${result.error}`);
        assert.deepEqual(result.answers.map((a) => a.query), example.problog.map(([q]) => q), `${what}: the queries`);
        result.answers.forEach(({ query, probability }, i) => {
          const want = example.problog[i][1];
          assert.ok(Math.abs(probability - want) < TOLERANCE, `${what}: ${query} = ${probability} against ProbLog's ${want}`);
        });
        if (evidence) {
          const got = result.evidence.probability;
          assert.ok(Math.abs(got - example.evidence) < TOLERANCE, `${what}: P(e) = ${got} against ProbLog's ${example.evidence}`);
          assert.ok(Math.abs(result.evidence.log10 - Math.log10(example.evidence)) < 1e-8, `${what}: log10 P(e)`);
        } else {
          assert.equal(result.evidence, null, `${what}: P(e) not asked for`);
        }
        assert.ok(result.parts.length >= 1, `${what}: a part's run`);
        for (const part of result.parts) {
          if (strategy !== "auto") assert.equal(part.trial, strategy, `${what}: the layout asked for`);
          assert.ok(part.attempts >= 1 && part.ground.atoms > 0, `${what}: the part's run`);
        }
      }
    }
    console.log(`ok ${example.key}: ${example.problog.length} answers and P(e) agree with ProbLog`);
  }

  // Refusals come back as errors, and the module keeps answering after them.
  const refused = [
    ["0.5::a. query(a).", "nonesuch", /^options error: unknown vtree strategy/],
    ["0.5::a query(a).", "auto", /^parse error: /],
    ["0.5::a. query(a).", "decision", /no fact of this program as a guard/],
  ];
  for (const [program, strategy, message] of refused) {
    const result = runtime.solve(solver, program, strategy, false);
    assert.match(result.error ?? "", message, `${program} under ${strategy}`);
  }
  const inconsistent = runtime.solve(solver, "0.5::a. b :- a. evidence(a,true). evidence(b,false). query(a).", "auto", true);
  assert.equal(inconsistent.answers, null);
  assert.deepEqual(inconsistent.evidence, { probability: 0, log10: null, z: false });
  const again = runtime.solve(solver, fs.readFileSync(path.join(root, "examples/alarm.pl"), "utf8"), "auto", false);
  assert.equal(again.answers.length, 3);
  console.log("ok refusals");
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
