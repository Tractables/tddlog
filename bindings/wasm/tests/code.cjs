// The pages' text, without a browser: the guide's syntax highlighting
// (every code block names a language code.js knows, highlighting keeps each
// block's text as it is, and the tokens that matter get their classes), the
// guide's programs against the repository's files, and the site's
// navigation, the same on every page, and the file of a benchmark the
// playground opens. `node --test tests/code.cjs`.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const code = require("../code.js");

const page = (file) => fs.readFileSync(path.join(__dirname, "..", file), "utf8");
const guide = page("guide/index.html");

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
  const blocks = [...guide.matchAll(/<pre data-lang="([a-z]+)"[^>]*>([\s\S]*?)<\/pre>/g)];
  assert.ok(blocks.length >= 20, `${blocks.length} code blocks`);
  for (const [, lang, body] of blocks) {
    assert.ok(code.languages.includes(lang), `the language ${lang}`);
    const source = decode(body);
    assert.equal(text(code.highlight(lang, source)), source, `a ${lang} block's text`);
  }
});

// A worked example shows its whole program, marked data-file, as the
// repository has it after the comment at its top.
test("every program the guide shows whole is its file in the repository", () => {
  const blocks = [...guide.matchAll(/<pre data-lang="problog" data-file="([^"]+)">([\s\S]*?)<\/pre>/g)];
  assert.ok(blocks.length >= 6, `${blocks.length} programs`);
  for (const [, file, body] of blocks) {
    const lines = fs.readFileSync(path.join(__dirname, "../../..", file), "utf8").split("\n");
    while (lines.length > 0 && lines[0].startsWith("%")) lines.shift();
    assert.equal(decode(body), lines.join("\n").trimEnd(), file);
    assert.ok(guide.includes(`<a href="https://github.com/Tractables/tddlog/blob/main/${file}"><code>${file}</code></a>`),
      `the guide links ${file}`);
  }
});

// Each page's bar: the site's sections (results.js's SECTIONS, the names
// its menu gives them), the playground, the docs in a list under their
// name, and GitHub, the same in the same order, the page's own marked as
// the current one (the front page, which the logo leads to, is none of
// them; the results page is By program type's).
test("every page has the site's bar, its own page marked", () => {
  const sections = JSON.parse(page("results/results.js").match(/^const SECTIONS = (\[.*\]);$/m)[1]);
  const names = [...sections.map(([, name]) => name), "Playground", "Guide", "Rust API", "Python", "C", "GitHub"];
  for (const [file, current] of [["index.html", null], ["playground/index.html", "Playground"], ["guide/index.html", "Guide"],
    ["results/index.html", "By program type"], ["404.html", null]]) {
    const nav = page(file).match(/<nav class="site-nav" id="site-nav" aria-label="Site">([\s\S]*?)<\/nav>/);
    assert.ok(nav, `${file}: the bar`);
    assert.match(nav[1], /<details class="docs"><summary[^>]*>Docs<\/summary><div>\s*<a href="[^"]+"[^>]*>Guide</, `${file}: the docs' list`);
    const links = [...nav[1].matchAll(/<a href="([^"]+)"(?: data-section="[^"]+")?( aria-current="page")?>([^<]+)<\/a>/g)];
    assert.deepEqual(links.map((l) => l[3]), names, `${file}: the bar's links`);
    assert.deepEqual(links.filter((l) => l[2]).map((l) => l[3]), current ? [current] : [], `${file}: the current page`);
    const up = file.includes("/") ? "../" : "";
    const where = { ...Object.fromEntries(sections.map(([key, name]) => [name, `${up}#/${key}`])), Playground: `${up}playground/`,
      Guide: `${up}guide/`, "Rust API": `${up}api/`, Python: `${up}python/`, C: `${up}c/` };
    for (const [, href, , name] of links) {
      const expected = name === current && !name.startsWith("By ") ? "./" : where[name] ?? "https://github.com/Tractables/tddlog";
      assert.equal(href, expected, `${file}: ${name}`);
    }
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

// A family's program with places ($1, ...) opens in the playground as the
// instance's program of its own name, the one a ProbLog system read; a
// program without places, or with no such program beside it, as it is.
test("filledFile: the instance's program in place of a family's with places", () => {
  const family = { name: "grid.pl", sha: "f" };
  const mine = { lang: "ProbLog", readers: ["tddlog"], files: [family] };
  const pita = { lang: "ProbLog", readers: ["PITA"], files: [{ name: "grid4.pl", sha: "p" }] };
  const problog = { lang: "ProbLog", readers: ["ProbLog -k sdd", "aspmc"], files: [{ name: "grid4.pl", sha: "q" }] };
  const texts = { f: "size($1).\n", p: "size(4).\n", q: "size(4).\n" };
  const inst = { name: "grid4", tabs: [mine, pita, problog] };
  assert.deepEqual(code.filledFile(inst, mine, family, texts), { tab: problog, file: problog.files[0] });
  assert.deepEqual(code.filledFile({ name: "grid4", tabs: [mine, pita] }, mine, family, texts), { tab: pita, file: pita.files[0] });
  assert.equal(code.filledFile(inst, mine, family, { ...texts, f: "size(4).\n" }), null);
  assert.equal(code.filledFile({ name: "grid4", tabs: [mine] }, mine, family, texts), null);
  // a text the site keeps on its own: tddlog's command says whether it has places
  const kept = { q: texts.q };
  const given = { ...mine, cmds: [["tddlog", "tddlog --stats grid.pl 4"]] };
  assert.deepEqual(code.filledFile({ name: "grid4", tabs: [given, problog] }, given, family, kept), { tab: problog, file: problog.files[0] });
  const bare = { ...mine, cmds: [["tddlog", "tddlog --stats grid.pl"]] };
  assert.equal(code.filledFile({ name: "grid4", tabs: [bare, problog] }, bare, family, kept), null);
  assert.equal(code.filledFile(inst, mine, family, kept), null);
});
