// The guide's syntax highlighting: every code block names a language
// code.js knows, highlighting keeps each block's text as it is, and the
// tokens that matter get their classes. `node --test tests/code.cjs`.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const code = require("../code.js");

const guide = fs.readFileSync(path.join(__dirname, "../guide/index.html"), "utf8");

function decode(html) {
  return html.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");
}

function text(html) {
  return decode(html.replace(/<\/?span[^>]*>/g, ""));
}

// The spans of class cls in the highlighted html, as text.
function spans(html, cls) {
  return [...html.matchAll(new RegExp(`<span class="t-${cls}">([^<]*)</span>`, "g"))].map((m) => decode(m[1]));
}

test("every code block of the guide names a known language and keeps its text", () => {
  assert.doesNotMatch(guide, /<pre>/, "a code block without data-lang");
  const blocks = [...guide.matchAll(/<pre data-lang="([a-z]+)">([\s\S]*?)<\/pre>/g)];
  assert.ok(blocks.length >= 20, `${blocks.length} code blocks`);
  for (const [, lang, body] of blocks) {
    assert.ok(code.languages.includes(lang), `the language ${lang}`);
    const source = decode(body);
    assert.equal(text(code.highlight(lang, source)), source, `a ${lang} block's text`);
  }
});

test("ProbLog: probabilities, operators, variables, predicates, comments", () => {
  const html = code.highlight("problog", "0.1::burglary.\ncalls(X) :- alarm, \\+ quiet(X). % a rule\nquery(n1).");
  assert.deepEqual(spans(html, "num"), ["0.1"]);
  assert.deepEqual(spans(html, "op"), ["::", ":-", "\\+"]);
  assert.deepEqual(spans(html, "var"), ["X", "X"]);
  assert.deepEqual(spans(html, "fn"), ["calls", "quiet"]);
  assert.deepEqual(spans(html, "kw"), ["query"]);
  assert.deepEqual(spans(html, "com"), ["% a rule"]);
});

test("shell: a session's prompt, command and flags, its output; commands alone", () => {
  const html = code.highlight("shell", "$ ./target/release/tddlog --ground win.pl > out\n0.5::win(3).");
  assert.deepEqual(spans(html, "prompt"), ["$"]);
  assert.deepEqual(spans(html, "fn"), ["./target/release/tddlog", "win"]);
  assert.deepEqual(spans(html, "kw"), ["--ground"]);
  assert.deepEqual(spans(html, "num"), ["0.5", "3"]);
  const alone = code.highlight("shell", "cd tddlog && cargo build --release");
  assert.deepEqual(spans(alone, "fn"), ["cd", "cargo"]);
  assert.deepEqual(spans(alone, "prompt"), []);
});

test("Rust, Python, C and TOML", () => {
  const rust = code.highlight("rust", 'let p = "a.\nb.";\nprintln!("{}", tddlog::Options::default());');
  assert.deepEqual(spans(rust, "kw"), ["let"]);
  assert.deepEqual(spans(rust, "str"), ['"a.\nb."', '"{}"']);
  assert.deepEqual(spans(rust, "fn"), ["println!", "default"]);
  assert.deepEqual(spans(rust, "typ"), ["Options"]);
  const python = code.highlight("python", 'import tddlog\nprint(f"{q}: {p:.6f}")  # done');
  assert.deepEqual(spans(python, "kw"), ["import"]);
  assert.deepEqual(spans(python, "str"), ['f"{q}: {p:.6f}"']);
  assert.deepEqual(spans(python, "com"), ["# done"]);
  const c = code.highlight("c", "TddlogOptions o;\nif (tddlog_solve(p) != TDDLOG_OK) return 1;");
  assert.deepEqual(spans(c, "typ"), ["TddlogOptions"]);
  assert.deepEqual(spans(c, "num"), ["TDDLOG_OK", "1"]);
  assert.deepEqual(spans(c, "kw"), ["if", "return"]);
  const toml = code.highlight("toml", '[dependencies]\ntddlog = { git = "x" }');
  assert.deepEqual(spans(toml, "kw"), ["[dependencies]"]);
  assert.deepEqual(spans(toml, "fn"), ["tddlog"]);
  assert.throws(() => code.highlight("cobol", "x"), /no rules/);
});
