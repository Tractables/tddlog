// The assembled site in headless Chromium: in the playground
// (playground/), every example's answers on the page against the ProbLog
// column beside them, P(evidence), the editor's colours under its text (the
// same text, laid out alike, following the scroll; plain for a long program
// and under forced colours), an edited program through the address (none
// for a program too long for it), a former address of the playground (the
// site's root) sent to it, an error and its line, Cancel and Escape, a long table, a file opened,
// dropped and downloaded, the answers copied, a benchmark of the results
// opened through #bench= (its answers against the release run's reference;
// a held, a partial, a slow and a malformed one; a gzipped text with a
// header above the file, and one that does not match), the theme, the
// guide's links, the pages' links within the site, the
// results page (the overview, each type against its native systems; the
// menu of the site's sections and their entries; each category's plot, drawing its systems over all of its
// instances with their answers and total times, its board and its
// filters; each baseline's page from its old address; each suite's address, which opens its
// type's view; the hover layer
// and the keyboard's, the lines in colour, an
// address of nothing; where the site has the benchmarks' data, a family's
// times, programs and runs, the systems whose files were not captured, the
// link to the playground, the drawer's address, a gzipped file, the search,
// and a link to a benchmark's former page; the systems' colours, and a
// category's board on a phone), the API,
// Python and C pages, and no sideways scrolling at a desktop's width or a
// phone's; on a phone the editor on the first screen; the code's colours'
// contrast in both themes.
//
//   node tests/browser.cjs <site> [<directory for screenshots>]
//
// RESULTS_ONLY=1 tests the results page alone (a site of that page, its
// styles, theme.js and code.js, and 404.html).
//
// with Playwright and its Chromium installed (README.md).
const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const zlib = require("node:zlib");
const { chromium } = require("playwright");
const { tddlogExamples } = require("../examples.js");
// the file of a benchmark the playground opens, as the pages choose it
const { filledFile } = require("../code.js");

const site = path.resolve(process.argv[2]);
const only = process.env.RESULTS_ONLY === "1";     // the results page alone, from a site that has only it
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css",
  ".wasm": "application/wasm", ".pl": "text/plain", ".json": "application/json",
  ".svg": "image/svg+xml", ".txt": "text/plain", ".gz": "application/gzip",
};

// The site as GitHub Pages serves it: a directory's index.html at its path,
// and 404.html for an address it does not have.
function serve(root) {
  const server = http.createServer((request, response) => {
    let file = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    if (file.endsWith("/")) file += "index.html";
    const full = path.join(root, file);
    if (!full.startsWith(root) || !fs.existsSync(full) || fs.statSync(full).isDirectory()) {
      const missing = path.join(root, "404.html");
      response.writeHead(404, fs.existsSync(missing) ? { "content-type": "text/html" } : {});
      response.end(fs.existsSync(missing) ? fs.readFileSync(missing) : undefined);
      return;
    }
    response.writeHead(200, { "content-type": TYPES[path.extname(full)] ?? "application/octet-stream" });
    fs.createReadStream(full).pipe(response);
  });
  return new Promise((resolve) => server.listen(0, "127.0.0.1", () => resolve(server)));
}

// Until the run on show is over and its answers have `first` as their first
// query (any, where `first` is undefined; or the result is an error, where
// `first` is null).
function answered(page, first) {
  return page.waitForFunction((first) => {
    if (document.body.classList.contains("busy")) return false;
    const status = document.getElementById("status").textContent;
    if (first === null) return status.startsWith("failed") && document.querySelector("#answers .failed") !== null;
    if (!status.startsWith("done")) return false;
    const cell = document.querySelector("#answers td.query");
    return first === undefined || (cell !== null && cell.textContent === first);
  }, first, { timeout: 60000 });
}

// The table on show: each row's query, tddlog's answer and the reference's.
function rows(page) {
  return page.$$eval("#answers tbody tr", (trs) => trs.map((tr) => [...tr.querySelectorAll("td.query, td.p")].map((td) => td.textContent)));
}

// Each row's query, tddlog's answer as the page holds it (its cell's
// title) and the reference's cell.
function unrounded(page) {
  return page.$$eval("#answers tbody tr", (trs) => trs.map((tr) => {
    const [query, answer, reference] = tr.querySelectorAll("td.query, td.p");
    return [query.textContent, Number(answer.title), reference?.textContent ?? ""];
  }));
}

// A probability with as many digits after the point as `cell` has, in its
// notation.
function writtenAs(p, cell) {
  const [mantissa, exponent] = cell.split("e");
  const point = mantissa.indexOf(".");
  const digits = point === -1 ? 0 : mantissa.length - point - 1;
  return exponent === undefined ? p.toFixed(digits) : p.toExponential(digits);
}

// Until the page has drawn twice: after an input, the colours are painted.
function frame(page) {
  return page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

// A long program put in the editor at once, as a paste: fill inserts it as
// typed text, which for thousands of lines takes Chromium longer than the
// test waits.
function paste(page, text) {
  return page.$eval("#program", (editor, text) => {
    editor.value = text;
    editor.dispatchEvent(new Event("input", { bubbles: true }));
  }, text);
}

// The editor's text and the colours' under it.
function layers(page) {
  return page.evaluate(() => [document.getElementById("program").value, document.getElementById("program-hl").textContent]);
}

// The contrast of two colours #rrggbb, as WCAG defines it.
function contrast(a, b) {
  const luminance = (hex) => {
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
      .map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  };
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

// No sideways scrolling; where there is, the first elements that stick out
// (outside any box that clips them) say why.
async function noSidewaysScroll(page, what) {
  const [scroll, width, out] = await page.evaluate(() => {
    const clipped = (e) => {
      for (let p = e.parentElement; p !== null; p = p.parentElement) if (getComputedStyle(p).overflowX !== "visible") return true;
      return false;
    };
    const out = [...document.body.querySelectorAll("*")]
      .filter((e) => e.getBoundingClientRect().right > window.innerWidth + 0.5 && !clipped(e)).slice(0, 3)
      .map((e) => `${e.tagName.toLowerCase()}${e.id ? `#${e.id}` : ""}${typeof e.className === "string" && e.className ? `.${e.className.replace(/ /g, ".")}` : ""} to ${e.getBoundingClientRect().right}`);
    return [document.documentElement.scrollWidth, window.innerWidth, out];
  });
  assert.ok(scroll <= width, `${what}: the page is ${scroll} px wide in a window of ${width} (${out.join(", ")})`);
}

// The benchmarks of the results the playground can open, where the site
// has their data: for each instance, the one .pl file tddlog read (the
// instance's program where the family's has places: code.js's filledFile,
// `filled`), whether the site keeps it whole and its text, and tddlog's
// time in the release run.
function benchmarksOf(root) {
  const dir = path.join(root, "results", "benchmarks");
  if (!fs.existsSync(path.join(dir, "index.json"))) return [];
  const index = JSON.parse(fs.readFileSync(path.join(dir, "index.json"), "utf8"));
  const native = new Map();
  for (const file of ["suites.json", "ppl.json"].map((f) => path.join(root, "results", f)).filter((f) => fs.existsSync(f))) {
    for (const s of JSON.parse(fs.readFileSync(file, "utf8")).families.flatMap((f) => f.suites)) {
      const own = s.systems.indexOf("tddlog") + 1;
      for (const row of s.instances) native.set(`${s.key}/${row[0]}`, own === 0 ? undefined : row[own]);
    }
  }
  const out = [];
  for (const [suite, fams] of Object.entries(index.suites)) {
    for (const f of fams) {
      const famFile = path.join(dir, suite, `${f.key}.json`);
      const fam = JSON.parse(fs.readFileSync(famFile, "utf8"));
      for (const inst of fam.instances) {
        const files = (inst.tabs ?? []).filter((t) => t.readers.includes("tddlog"))
          .flatMap((t) => t.files.filter((x) => x.name.endsWith(".pl")).map((x) => [t, x]));
        if (files.length !== 1) continue;
        const opened = filledFile(inst, files[0][0], files[0][1], fam.texts);
        const file = opened?.file ?? files[0][1];
        const inline = fam.texts[file.sha];
        const readable = !file.held && !file.part && (inline !== undefined || Boolean(file.src));
        const text = () => {
          if (inline !== undefined) return inline;
          const raw = fs.readFileSync(path.join(dir, file.src));
          return (file.src.endsWith(".gz") ? zlib.gunzipSync(raw) : raw).toString("utf8");
        };
        out.push({ suite, family: f.key, name: inst.name, path: `${suite}/${f.key}/${inst.name}`, famFile, file, filled: opened !== null, readable, text,
          inline: inline !== undefined, native: native.get(`${suite}/${inst.name}`), reference: Boolean(inst.answers?.ref) });
      }
    }
  }
  return out;
}

(async () => {
  // The browser first: if it cannot start, no server is left listening
  // and node exits with the error.
  const browser = await chromium.launch();
  const server = await serve(site);
  const base = `http://127.0.0.1:${server.address().port}/`;
  const play = `${base}playground/`;
  const errors = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, permissions: ["clipboard-read", "clipboard-write"] });
    const page = await context.newPage();
    page.on("pageerror", (error) => errors.push(String(error)));

    if (!only) {
    // The page opens on the first example and runs it, with the results at
    // a glance under it: all programs as categories.json counts them, the
    // numbers of the site's home and the README, with their stamp.
    await page.goto(play);
    await answered(page, tddlogExamples[0].problog[0][0]);
    await page.waitForSelector("#glance:not([hidden])");
    const glance = (await page.textContent("#glance-stats")).replace(/(\d),(?=\d)/g, "$1").replace(/\s+/g, " ");
    const cats = JSON.parse(fs.readFileSync(path.join(site, "results", "categories.json"), "utf8"));
    const every = cats.categories.find((c) => c.key === "all-programs");
    assert.match(glance, new RegExp(`${every.instances} instances ${every.answers.tddlog} answered by tddlog \\d+ answered by tddlog and no baseline`), glance);
    assert.equal(await page.textContent("#glance-asof"), `Results as of ${cats.stamp.date}, public main ${cats.stamp.source}.`);
    assert.equal(await page.$("[aria-live]"), null, "only the status is announced");

    // Every example: tddlog's answers agree with ProbLog's to the digits
    // shown, in the program's order; its button is pressed, and the
    // program's colours lie under its text.
    for (const [i, example] of tddlogExamples.entries()) {
      await page.click(`#examples button[data-example="${i}"]`);
      await answered(page, example.problog[0][0]);
      const shown = await rows(page);
      assert.deepEqual(shown.map(([query]) => query), example.problog.map(([query]) => query), `${example.key}: the queries`);
      for (const [query, tddlog, problog] of shown) assert.equal(tddlog, problog, `${example.key}: ${query}`);
      assert.match(page.url(), new RegExp(`example=${example.key}`), `${example.key}: the address`);
      const pressed = await page.$$eval("#examples button", (bs) => bs.map((b) => b.getAttribute("aria-pressed")));
      assert.deepEqual(pressed, tddlogExamples.map((_, j) => String(j === i)), `${example.key}: its button pressed`);
      const [text, shade] = await layers(page);
      assert.equal(shade, text + "\n", `${example.key}: the colours' text`);
      console.log(`ok ${example.key}: ${shown.length} answers on the page agree with ProbLog's`);
    }

    // P(evidence), with ProbLog's beside it; the alarm's tokens in colour.
    await page.click('#examples button[data-example="0"]');
    await answered(page, tddlogExamples[0].problog[0][0]);
    const tokens = await page.$$eval("#program-hl span", (spans) => spans.map((s) => `${s.className} ${s.textContent}`));
    for (const token of ["t-kw query", "t-num 0.1", "t-op ::", "t-op :-", "t-var X"]) assert.ok(tokens.includes(token), `the alarm's ${token}`);
    await page.check("#evidence");
    await page.click("#run");
    await page.waitForSelector("#answers .evidence");
    await answered(page, tddlogExamples[0].problog[0][0]);
    const evidence = await page.$$eval("#answers .evidence b", (bs) => bs.map((b) => b.textContent));
    assert.equal(evidence[0], evidence[2], `P(e) ${evidence.join(" ")}`);
    assert.match(page.url(), /evidence=1/);
    assert.match(await page.textContent("#summary"), /in the browser/);
    console.log(`ok P(evidence) ${evidence[0]}`);

    // The trace of the run, step by step, its last step the answers on
    // show; the address keeps it, and a run without it hides it.
    await page.check("#trace");
    await page.click("#run");
    await answered(page, tddlogExamples[0].problog[0][0]);
    await page.waitForSelector("#trace-card:not([hidden]) details.trace-step");
    const titles = await page.$$eval("#trace-steps details.trace-step > summary", (ss) => ss.map((s) => s.textContent));
    for (const t of ["1. Read", "2. Parse", "3. Ground"]) assert.equal(titles.includes(t), true, `the trace's ${t}: ${titles.join(", ")}`);
    for (const t of ["Rewrite", "Choose a layout", "Compile", "Evidence and queries"]) assert.ok(titles.some((u) => u.includes(t)), `the trace's ${t}: ${titles.join(", ")}`);
    assert.match(titles.at(-1), /\. Answers$/);
    await page.click("#trace-steps details.trace-step:has(svg.trace-vtree) > summary");
    assert.ok(await page.isVisible("#trace-steps svg.trace-vtree"), "the vtree drawn");
    const traced = await page.$$eval("#trace-steps details.trace-step:last-child tbody tr", (trs) => trs.map((tr) => [...tr.cells].map((td) => td.textContent)));
    assert.deepEqual(traced, (await rows(page)).map(([query, p]) => [query, p]), "the trace's answers are those on show");
    assert.match(page.url(), /trace=1/);
    await page.uncheck("#trace");
    await page.click("#run");
    await answered(page, tddlogExamples[0].problog[0][0]);
    assert.equal(await page.isHidden("#trace-card"), true, "no trace unless asked for");
    assert.doesNotMatch(page.url(), /trace=1/);
    console.log(`ok the trace: ${titles.length} steps`);

    // An edited program is painted in the next frame, runs, goes in the
    // address, and comes back from it.
    const program = "0.3::a.\n0.6::b.\nc :- a, \\+b.\nquery(c).\n";
    await page.fill("#program", program);
    await frame(page);
    assert.deepEqual(await layers(page), [program, program + "\n"], "the colours after an edit");
    assert.equal(await page.getAttribute('#examples button[data-example="0"]', "aria-pressed"), "false");
    await page.click("#run");
    await answered(page, "c");
    assert.deepEqual(await rows(page), [["c", "0.120000"]]);
    await page.waitForFunction(() => location.hash.startsWith("#program="));
    const link = page.url();
    assert.doesNotMatch(link, /example=/);
    const again = await context.newPage();
    await again.goto(link);
    await answered(again, "c");
    assert.equal(await again.inputValue("#program"), program);
    assert.equal(await again.isChecked("#evidence"), true);
    await again.close();
    console.log("ok an edited program through the address");

    // The playground was once the site's root: an address of it there opens
    // the same program in the playground.
    const former = await context.newPage();
    await former.goto(link.replace(play, base));
    await answered(former, "c");
    assert.equal(former.url(), link, "a former address of the playground");
    await former.goto(`${base}?example=win`);
    await former.waitForURL(`${play}?example=win`);
    await former.close();
    console.log("ok a former address of the playground");

    // An error is shown as one, a parse error's line is marked and the
    // caret put there, and the solver answers after it.
    const broken = "0.3::a.\n0.6::b.\nc :- a b.\nquery(c).\n";
    await page.fill("#program", broken);
    await page.click("#run");
    await answered(page, null);
    assert.match(await page.textContent("#answers .failed"), /^parse error: line 3: /);
    assert.match(await page.textContent("#status"), /^failed: parse error: line 3: /);
    const marked = await page.evaluate(() => [document.getElementById("program").selectionStart, document.querySelector("#program-hl .error-line")?.style.top]);
    assert.deepEqual(marked, ["0.3::a.\n0.6::b.\n".length, "3em"], "the error's line");
    await page.selectOption("#strategy", "decision");
    await page.fill("#program", "0.5::a. query(a).");
    await frame(page);
    assert.equal(await page.$("#program-hl .error-line"), null, "the mark goes with an edit");
    await page.click("#run");
    await answered(page, null);
    assert.match(await page.textContent("#answers .failed"), /no fact of this program as a guard/);
    await page.selectOption("#strategy", "auto");
    await page.click("#run");
    await answered(page, "a");
    console.log("ok errors");

    // Cancel, and Escape, end the run's worker, and a new one answers.
    await page.evaluate(() => {
      document.getElementById("run").click();
      document.getElementById("cancel").click();
    });
    assert.match(await page.textContent("#status"), /stopped|ready/);
    await page.click("#run");
    await answered(page, "a");
    await page.evaluate(() => {
      document.getElementById("run").click();
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });
    assert.match(await page.textContent("#status"), /^stopped/);
    await page.click("#run");
    await answered(page, "a");
    console.log("ok Cancel and Escape");

    // The colours lie under the text: both layers lay it out alike, and the
    // colours follow the textarea to either end of a long, wide program
    // (its last line the longest).
    const style = await page.evaluate(() => {
      const props = ["fontFamily", "fontSize", "lineHeight", "fontVariantLigatures", "fontKerning", "letterSpacing", "wordSpacing", "tabSize", "whiteSpace",
        "paddingTop", "paddingRight", "paddingBottom", "paddingLeft", "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth"];
      const of = (id) => { const s = getComputedStyle(document.getElementById(id)); return Object.fromEntries(props.map((p) => [p, s[p]])); };
      return [of("program"), of("program-hl")];
    });
    assert.deepEqual(style[1], style[0], "the colours' layout is the textarea's");
    const wide = Array.from({ length: 120 }, (_, i) => `0.5::p${i}(${"x".repeat(i === 119 ? 400 : 5)}).`).join("\n");
    await page.fill("#program", wide);
    await frame(page);
    const ends = [];
    for (const end of [false, true]) {
      ends.push(await page.evaluate(async (end) => {
        const editor = document.getElementById("program");
        editor.scrollLeft = end ? editor.scrollWidth : 0;
        editor.scrollTop = end ? editor.scrollHeight : 0;
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        const shade = document.getElementById("program-hl");
        const s = getComputedStyle(editor);
        const box = editor.getBoundingClientRect();
        const left = box.left + editor.clientLeft + parseFloat(s.paddingLeft);
        const top = box.top + editor.clientTop + parseFloat(s.paddingTop);
        const right = box.left + editor.clientLeft + editor.clientWidth;
        const bottom = box.top + editor.clientTop + editor.clientHeight - parseFloat(s.paddingBottom);
        const texts = [];
        for (const walk = document.createTreeWalker(shade, NodeFilter.SHOW_TEXT); walk.nextNode();) texts.push(walk.currentNode);
        const char = (node, i) => {
          const range = document.createRange();
          range.setStart(node, i);
          range.setEnd(node, i + 1);
          const r = range.getBoundingClientRect();
          return { left: r.left, right: r.right, top: r.top, bottom: r.bottom };
        };
        const last = texts[texts.length - 1];
        return { transform: shade.style.transform, scroll: [editor.scrollLeft, editor.scrollTop], left, top, right, bottom,
          padRight: parseFloat(s.paddingRight), first: char(texts[0], 0), last: char(last, last.length - 2) };
      }, end));
    }
    for (const at of ends) assert.equal(at.transform, `translate(${-at.scroll[0]}px, ${-at.scroll[1]}px)`, "the colours follow the scroll");
    const [start, finish] = ends;
    assert.ok(finish.scroll[0] > 0 && finish.scroll[1] > 0, `scrolled to ${finish.scroll}`);
    assert.ok(Math.abs(start.first.left - start.left) <= 1, `the first character at ${start.first.left}, the text from ${start.left}`);
    const leading = start.first.top - start.top;
    assert.ok(finish.last.right <= finish.right + 1 && finish.last.right >= finish.right - finish.padRight - 3,
      `the last character ends at ${finish.last.right}, the scroll at ${finish.right}`);
    assert.ok(Math.abs(finish.bottom - finish.last.bottom - leading) <= 1.5,
      `the last line ends at ${finish.last.bottom}, the scroll at ${finish.bottom} less ${leading}`);
    await page.emulateMedia({ forcedColors: "active" });
    const forced = await page.evaluate(() => [getComputedStyle(document.getElementById("program-hl")).display,
      getComputedStyle(document.getElementById("program")).webkitTextFillColor]);
    assert.equal(forced[0], "none", "no colours under forced colours");
    assert.notEqual(forced[1], "rgba(0, 0, 0, 0)", "the text under forced colours");
    await page.emulateMedia({ forcedColors: "none" });
    console.log("ok the colours under the text, to both ends of a long, wide program");

    // A program too long to colour is shown plain, and one too long for the
    // address leaves it, Download taking the link's place.
    let seed = 7;
    const random = () => (seed = (seed * 48271) % 2147483647).toString(36);
    const long = Array.from({ length: 15000 }, () => `0.5::f${random()}${random()}.`).join("\n");
    await paste(page, long);
    await frame(page);
    const plain = await page.evaluate(() => [document.querySelector(".editor").classList.contains("plain"),
      getComputedStyle(document.getElementById("program")).webkitTextFillColor]);
    assert.equal(plain[0], true, `a program of ${long.length} characters is plain`);
    assert.notEqual(plain[1], "rgba(0, 0, 0, 0)", "a plain program's text");
    await page.waitForSelector("#too-long:not([hidden])");
    assert.doesNotMatch(page.url(), /#/, "no program in the address");
    assert.equal(await page.isHidden("#link"), true);
    console.log("ok a long program: plain, and not in the address");

    // A long table shows 500 rows and all on asking; the answers copy as
    // tab-separated text.
    await page.uncheck("#evidence");
    const many = Array.from({ length: 600 }, (_, i) => `0.5::a(${i}). query(a(${i})).`).join("\n");
    await paste(page, many);
    await page.click("#run");
    await answered(page, "a(0)");
    assert.equal((await page.$$("#answers tbody tr")).length, 500);
    await page.click("#answers button.all");
    assert.equal((await page.$$("#answers tbody tr")).length, 600);
    await page.click("#copy");
    const tsv = (await page.evaluate(() => navigator.clipboard.readText())).split("\n");
    assert.deepEqual(tsv.slice(0, 2), ["query\tprobability", "a(0)\t0.500000"]);
    assert.equal(tsv.length, 602);
    console.log("ok a long table, and the answers copied");

    // A file opened, or dropped on the editor, comes in whole; Download
    // saves the program under its name.
    const mine = "0.2::m.\nquery(m). % é\n";
    await page.setInputFiles("#file", { name: "mine.pl", mimeType: "text/plain", buffer: Buffer.from(mine) });
    await page.waitForFunction((mine) => document.getElementById("program").value === mine, mine);
    const [download] = await Promise.all([page.waitForEvent("download"), page.click("#download")]);
    assert.equal(download.suggestedFilename(), "mine.pl");
    assert.equal(fs.readFileSync(await download.path(), "utf8"), mine);
    await page.evaluate(() => {
      const files = new DataTransfer();
      files.items.add(new File(["0.1::d.\nquery(d).\n"], "dropped.pl", { type: "text/plain" }));
      document.getElementById("program").dispatchEvent(new DragEvent("drop", { dataTransfer: files, bubbles: true, cancelable: true }));
    });
    await page.waitForFunction(() => document.getElementById("program").value === "0.1::d.\nquery(d).\n");
    console.log("ok a file opened, dropped and downloaded");

    // A benchmark of the results opens through #bench=SUITE/FAMILY/INSTANCE,
    // where the site has their data: the program tddlog was given, run where
    // it is short and was quick natively, its answers agreeing with the
    // release run's reference; again after a reload. A held or partial file,
    // a slow benchmark and a malformed or unknown address open nothing to
    // run, and say why.
    const benches = benchmarksOf(site);
    // A page's address changed only after the # does not load it again.
    const visit = async (p, address) => {
      await p.goto("about:blank");
      await p.goto(`${play}${address}`);
    };
    if (benches.length > 0) {
      const quick = (b) => b.readable && b.file.bytes <= 1024 * 1024 && typeof b.native === "number" && b.native <= 2;
      const whole = benches.find((b) => b.path === "std/grids/grid4" && quick(b) && b.reference && b.inline)
        ?? benches.find((b) => quick(b) && b.reference && b.inline);
      assert.ok(whole, "a benchmark to open");
      const opened = async (p, bench) => {
        await visit(p, `#bench=${bench.path}`);
        await answered(p);
        const shown = await rows(p);
        // the reference has no more digits than the results wrote it with
        const referenced = (await unrounded(p)).filter(([, , reference]) => reference !== "");
        assert.ok(referenced.length > 0, `${bench.path}: answers with a reference`);
        for (const [query, tddlog, reference] of referenced) assert.equal(writtenAs(tddlog, reference), reference, `${bench.path}: ${query}`);
        return shown;
      };
      const text = whole.text();
      const shown = await opened(page, whole);
      assert.equal(await page.inputValue("#program"), text);
      const opening = `${whole.name} from ${whole.suite}/${whole.family}: `;
      assert.ok((await page.textContent("#source")).startsWith(`${opening}the program tddlog was given`), await page.textContent("#source"));
      // a family's program with places opens with this instance's arguments in them, and says so
      const places = "the program tddlog was given in the release run, with this instance's arguments in its places";
      assert.equal((await page.textContent("#source")).includes(places), whole.filled, `${whole.path}: the note on its places`);
      // places are filled outside the comments, as tddlog binds them (a family's comment may name its places)
      if (whole.filled) assert.doesNotMatch((await page.inputValue("#program")).replace(/%.*$/gm, ""), /\$[1-9]/, `${whole.path}: a place left in the program`);
      assert.equal(await page.getAttribute("#source a", "href"), `${base}results/#/${whole.path}`);
      assert.ok(page.url().endsWith(`#bench=${whole.path}`), page.url());
      await page.reload();
      await answered(page);
      assert.deepEqual(await rows(page), shown, `${whole.path}: again after a reload`);
      console.log(`ok ${whole.path} in the playground: ${shown.length} answers agree with the reference`);

      const fetched = benches.filter((b) => quick(b) && b.reference && !b.inline).sort((a, b) => a.file.bytes - b.file.bytes)[0];
      if (fetched) {
        await opened(page, fetched);
        console.log(`ok ${fetched.path}, its text a file of its own`);
      }

      // A gzipped text, with a header above the file, opens; a text that is
      // not the file does not.
      const family = JSON.parse(fs.readFileSync(whole.famFile, "utf8"));
      const routed = async (body) => {
        const p = await context.newPage();
        p.on("pageerror", (error) => errors.push(String(error)));
        await p.route(`**/results/benchmarks/${whole.suite}/${whole.family}.json*`, (route) => {
          const fam = structuredClone(family);
          // (the file the playground opens: tddlog's, or the instance's program filled in for it)
          const file = fam.instances.find((i) => i.name === whole.name).tabs.flatMap((t) => t.files).find((f) => f.sha === whole.file.sha);
          delete fam.texts[file.sha];
          file.src = "files/routed.txt.gz";
          return route.fulfill({ json: fam });
        });
        await p.route("**/results/benchmarks/files/routed.txt.gz", (route) => route.fulfill({ body: zlib.gzipSync(body), contentType: "application/gzip" }));
        return p;
      };
      const header = "% A header the site puts above the file: not part of it.\n";
      const gz = await routed(header + text);
      await opened(gz, whole);
      assert.equal(await gz.inputValue("#program"), header + text);
      await gz.close();
      // its last digit changed: a header the site puts above a file (Biomine's, Ace's) is not part of it
      const wrong = await routed(text.replace(/\d(?=\D*$)/, (d) => String((Number(d) + 1) % 10)));
      await visit(wrong, `#bench=${whole.path}`);
      await wrong.waitForFunction(() => /does not match/.test(document.getElementById("source").textContent));
      await wrong.close();
      console.log("ok a gzipped text with a header opens, a text that is not the file does not");

      // Nothing to run: the note says why, with a link to the results.
      const refused = async (address, said) => {
        await visit(page, `#bench=${address}`);
        await page.waitForFunction(() => {
          const note = document.getElementById("source");
          return !note.hidden && !note.textContent.endsWith("loading.") && document.getElementById("status").textContent === "ready";
        });
        const note = await page.textContent("#source");
        assert.match(note, said, address);
        assert.doesNotMatch(note, /the program tddlog was given/, address);
        assert.ok((await page.getAttribute("#source a:last-of-type", "href")).startsWith(`${base}results/`), `${address}: a link to the results`);
        assert.equal(await page.inputValue("#program"), "", `${address}: nothing to run`);
        await page.waitForTimeout(300);
        assert.equal(await page.textContent("#answers"), "Press Run.", `${address}: no run`);
      };
      const named = (b) => new RegExp(`^${b.name.replace(/\./g, "\\.")} from ${b.suite}/${b.family}: `);
      const held = benches.find((b) => b.file.held);
      if (held) await refused(held.path, named(held));
      const part = benches.find((b) => b.file.part && !b.file.held);
      if (part) await refused(part.path, /keeps the first [\d,]+ of its [\d,]+ bytes/);
      await refused("../x", /^The address names no benchmark/);
      await refused("std/grid/..", /^The address names no benchmark/);
      await refused(`${whole.suite}/${whole.family}/nothing`, /no such benchmark/);
      const slow = benches.find((b) => b.readable && !quick(b));
      if (slow) {
        await visit(page, `#bench=${slow.path}`);
        await page.waitForFunction(() => /Press Run to try it here\.$/.test(document.getElementById("source").textContent)
          && document.getElementById("status").textContent === "ready");
        await page.waitForTimeout(300);
        assert.equal(await page.textContent("#answers"), "Press Run.", `${slow.path}: no run`);
        assert.equal(await page.inputValue("#program"), slow.text(), `${slow.path}: its program`);
      }
      console.log(`ok refused: ${[held?.path, part?.path, "../x", "std/grid/.."].filter(Boolean).join(", ")}; not run as it opens: ${slow?.path ?? "none"}`);
    }

    // The theme button switches the palette and keeps the choice.
    await visit(page, "");
    await answered(page, tddlogExamples[0].problog[0][0]);
    const background = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    const before = await background();
    await page.click("#theme");
    assert.notEqual(await background(), before);
    await page.reload();
    assert.notEqual(await background(), before, "the choice is kept");
    await page.click("#theme");
    assert.equal(await background(), before);
    console.log("ok the theme");
    await noSidewaysScroll(page, "1280 px");

    // The guide: each link to the playground names an example, and each
    // link within the page an element of it.
    await page.goto(`${base}guide/`);
    const links = await page.$$eval("a[href]", (anchors) => anchors.map((a) => a.getAttribute("href")));
    const keys = new Set(tddlogExamples.map((e) => e.key));
    const tried = links.filter((h) => h.startsWith("../playground/?")).map((h) => new URLSearchParams(h.split("?")[1]).get("example"));
    assert.ok(tried.length >= 5, `the guide links to ${tried.length} examples`);
    for (const key of tried) assert.ok(keys.has(key), `the guide links to ${key}, which is not an example`);
    const within = links.filter((h) => h.startsWith("#"));
    for (const h of within) assert.ok(await page.$(h), `the guide's link ${h} has no target`);
    console.log(`ok the guide: ${tried.length} links to examples, ${within.length} within the page`);

    // The guide's headings each link to themselves; the terms the
    // playground and the results page use are there to link to; the
    // install line is the README's; and at a desktop's width the contents
    // stay in view beside the text.
    const headings = await page.$$eval(".guide-text :is(h2, h3, h4)", (hs) => hs.map((h) => {
      const a = h.querySelector("a.anchor");
      return [h.textContent, a && a.getAttribute("href"), h.id || h.parentElement.id];
    }));
    for (const [text, href, id] of headings) assert.equal(href, `#${id}`, `the guide's heading ${text}: its link to itself`);
    for (const term of ["vtree", "pairs", "layout", "trial", "component", "part"]) {
      assert.ok(await page.$(`dt#term-${term}`), `the guide's terms: ${term}`);
    }
    const install = "cargo install --locked --git https://github.com/Tractables/tddlog tddlog";
    assert.ok((await page.textContent("#getting-started")).includes(install), "the guide's install line");
    assert.ok(fs.readFileSync(path.join(__dirname, "../../../README.md"), "utf8").includes(install), "the README's install line");
    assert.equal(await page.$eval("nav.contents", (n) => getComputedStyle(n).position), "sticky");
    await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
    const top = await page.$eval("nav.contents", (n) => n.getBoundingClientRect().top);
    assert.ok(top >= 0 && top < 100, `the guide's contents at the end of the text: ${top} px from the top`);
    console.log(`ok the guide: ${headings.length} headings with links, the terms, the install line, the contents in view`);

    // The page for an address the site does not have: a 404, with the
    // site's styles, logo and links, whose links lead to the site's pages.
    const guideBackground = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    const missing = await page.goto(`${base}no/such/page`);
    assert.equal(missing.status(), 404);
    assert.equal(await page.title(), "tddlog: not found");
    assert.ok(await page.$eval(".site-logo", (img) => img.complete && img.naturalWidth > 0), "the not-found page: the logo");
    assert.equal(await page.evaluate(() => getComputedStyle(document.body).backgroundColor), guideBackground, "the not-found page: the site's styles");
    const away = await page.$$eval("main a[href]", (anchors) => anchors.map((a) => a.href).filter((h) => h.startsWith(location.origin)));
    for (const href of new Set(away)) {
      assert.equal((await context.request.get(href)).status(), 200, `the not-found page: the link to ${href}`);
    }
    console.log(`ok the not-found page: ${new Set(away).size} links within the site`);

    // Each page's head: the icon, and for a page of its own what a shared
    // link to it shows; the picture is 1200 by 630. The sitemap's pages are
    // the site's.
    for (const at of ["", "playground/", "guide/", "results/"]) {
      await page.goto(`${base}${at}`);
      const meta = await page.evaluate(() => {
        const get = (selector, name) => document.querySelector(selector)?.getAttribute(name) ?? null;
        return {
          icon: document.querySelector('link[rel="icon"]')?.href ?? null,
          title: get('meta[property="og:title"]', "content"), description: get('meta[property="og:description"]', "content"),
          image: get('meta[property="og:image"]', "content"), url: get('meta[property="og:url"]', "content"),
          canonical: get('link[rel="canonical"]', "href"), card: get('meta[name="twitter:card"]', "content"),
        };
      });
      const what = at || "the front page";
      assert.ok(meta.icon && (await context.request.get(meta.icon)).status() === 200, `${what}: the icon`);
      assert.ok(meta.title && meta.description, `${what}: the title and description a shared link shows`);
      assert.equal(meta.url, `https://tractables.github.io/tddlog/${at}`, `${what}: its address`);
      assert.equal(meta.canonical, meta.url, `${what}: its canonical address`);
      assert.equal(meta.image, "https://tractables.github.io/tddlog/social.png", `${what}: the picture a shared link shows`);
      assert.equal(meta.card, "summary_large_image", `${what}: the card a shared link shows`);
    }
    const social = fs.readFileSync(path.join(site, "social.png"));
    assert.deepEqual([social.readUInt32BE(16), social.readUInt32BE(20)], [1200, 630], "social.png is 1200 by 630");
    const sitemap = [...fs.readFileSync(path.join(site, "sitemap.xml"), "utf8").matchAll(/<loc>https:\/\/tractables\.github\.io\/tddlog\/([^<]*)<\/loc>/g)].map((m) => m[1]);
    assert.ok(sitemap.length >= 5, `the sitemap lists ${sitemap.length} pages`);
    for (const address of sitemap) assert.equal((await context.request.get(`${base}${address}`)).status(), 200, `the sitemap's page ${address}`);
    console.log(`ok the pages' icon and shared-link tags; the sitemap's ${sitemap.length} pages`);

    // Every link from the front page, the playground, the guide and the
    // results page to a file of the site leads to one.
    const checked = new Set();
    for (const at of ["", "playground/", "guide/", "results/"]) {
      await page.goto(`${base}${at}`);
      assert.ok(await page.$eval(".site-logo", (img) => img.complete && img.naturalWidth > 0),
        `${at || "the front page"}: the project logo loads`);
      await page.waitForSelector(at === "" ? "#view table.home-t" : at === "results/" ? "#view figure.overview svg" : "body");
      if (at === "") {
        // the home: the five types in its lede, its numbers those of the data's stamp
        const home = JSON.parse(fs.readFileSync(path.join(site, "results", "categories.json"), "utf8"));
        assert.match(await page.textContent("#view p.big"), /probabilistic logic programs, probabilistic databases, Bayesian and Markov networks/);
        assert.equal((await page.textContent("#view p.stamp")).replace(/\s+/g, " "),
          `Results as of ${home.stamp.date}, public main ${home.stamp.source}.`, "the home: the data's stamp");
      }
      // an SVG link's href is an SVGAnimatedString
      const hrefs = await page.$$eval("a[href]", (anchors) => anchors.map((a) => (typeof a.href === "string" ? a.href : new URL(a.href.baseVal, location.href).href)));
      for (const href of hrefs) {
        const url = new URL(href);
        url.hash = "";
        if (!url.href.startsWith(base) || url.search !== "" || checked.has(url.href)) continue;
        checked.add(url.href);
        const response = await context.request.get(url.href);
        assert.equal(response.status(), 200, `${at || "the front page"}: the link to ${url.href}`);
      }
    }
    console.log(`ok ${checked.size} links within the site`);
    }

    // The results page, its categories from categories.json (the README's
    // figures draw the same). The overview draws the README's figures, in
    // the README's order: for each type of program a row for each of its
    // best native systems and one for the others, then each category's
    // headline, its cactus plot of the systems drawn (tddlog and the native
    // systems, tddlog first, then by their answers, tddlog's and the five best
    // native systems' lines in colour and the others grey)
    // and its scatter plot of tddlog against the fastest of them; then each
    // system on its own benchmarks. A category's view
    // (#/category/KEY) plots the same, its legend "answers k of N, tddlog R
    // faster" for each system drawn, k the data's over all N instances and R
    // tddlog's speed-up over it (rule 16, checked against an oracle); its axis
    // goes to all N, linear from 0; its board lists tddlog and the native
    // systems of the category, its table of families narrows it to a family
    // and its filters to a system's own benchmarks, each system's name
    // opening its page (a baseline's, or tddlog's integration's, on the front
    // page); the menu holds the site's sections. A suite's address (#/SUITE)
    // opens the view of its type, narrowed to the family of its instances
    // where that is one.
    const read = (file) => JSON.parse(fs.readFileSync(path.join(site, "results", file), "utf8"));
    const data = ["suites.json", "ppl.json"].filter((file) => fs.existsSync(path.join(site, "results", file))).map(read);
    const cats = read("categories.json");
    const categories = [...cats.categories].sort((a, b) => a.order - b.order);
    const ALL = "all-programs";
    const types = categories.filter((c) => c.key !== ALL);
    const bySuite = new Map(data.flatMap((d) => d.families.flatMap((f) => f.suites)).map((s) => [s.key, s]));
    const names = Object.assign({}, ...data.map((d) => d.names), cats.names);
    const number = (v) => typeof v === "number";
    const count = (n) => n.toLocaleString("en");
    // tddlog and its pipelines (Alea.jl+tddlog: another system's front end,
    // then tddlog), the subjects of the comparisons. A row's subject is the
    // first of them with a record in the systems' order: tddlog in a type,
    // its record "unread" (unanswered and not given) where only another
    // system's front end reads the program; on system x's own page too, never
    // X+tddlog (results.js's and panels.py's).
    const isMine = (x) => x === "tddlog" || x.endsWith("-tddlog");
    const given = (r, x) => r.v[x] !== null && r.v[x] !== "unread";
    const myKey = (r) => Object.keys(r.v).find((x) => isMine(x) && r.v[x] !== null);
    const subjectsOf = (rows) => [...new Set(rows.map(myKey))].filter(Boolean)
      .sort((a, b) => (b === "tddlog") - (a === "tddlog") || a.localeCompare(b));
    const answeredBy = (rows) => rows.filter((r) => number(r.v[myKey(r)])).length;
    const subject = (keys) => {
      const xs = (keys.length ? keys : ["tddlog"]).map((x) => names[x]);
      return xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} and ${xs.at(-1)}`;
    };
    // A plot's lines in its legend's order: tddlog's and its pipelines', then
    // by their answers.
    const byAnswers = (systems, answers) => [...systems].sort((a, b) => isMine(b) - isMine(a) || answers(b) - answers(a));
    // a category's line in colour: tddlog's, its pipelines' and its five best native systems'
    const lit = (c, x) => c.mine.includes(x) || c.named.includes(x);
    // The view an address names, once it is drawn: `ready` within it.
    const visit = async (hash, ready) => {
      await page.evaluate((h) => { location.hash = h; }, hash);
      await page.waitForSelector(`#view[data-route="${hash}"] ${ready}`);
      // every part of a view an element: no "null" of a part left out
      assert.ok(!(await page.$eval("#view", (v) => [...v.childNodes].some((n) => n.nodeType === Node.TEXT_NODE && n.textContent.trim()))),
        `${hash}: no stray text in the view`);
    };
    // (a plot is drawn once its box has a width, after the view is in place)
    // the plots of the view, or those `figures` selects
    const plots = async (figures = "#view figure.plot") => (await page.waitForFunction((sel) => [...document.querySelectorAll(sel)].every((f) => f.querySelector("svg.cactus")), figures))
      && page.$$eval(figures, (fs) => fs.map((f) => ({
      systems: f.dataset.systems.split(" ").filter(Boolean),
      instances: Number(f.dataset.instances),
      lines: [...f.querySelectorAll("svg.cactus path.line")].map((p) => p.dataset.key),
      legend: [...f.querySelectorAll(".legend .n")].map((s) => s.textContent.replace(/\s/g, "")),
      coloured: [...f.querySelectorAll('.legend button[aria-pressed="true"]')].map((b) => b.dataset.system),
      axis: f.querySelector("svg.cactus").textContent.replace(/\s/g, ""),
      time: [...f.querySelectorAll('svg.cactus text.tick[text-anchor="middle"]')].map((t) => [Number(t.getAttribute("x")), Number(t.textContent)]),
    })));
    const shown = async () => (await plots())[0];
    const linear = (fig, what) => {
      const [[xa, ta], ...later] = fig.time;
      const [xb, tb] = later.at(-1);
      assert.equal(ta, 0, `${what}: the time axis starts at 0`);
      for (const [x, t] of later) assert.ok(Math.abs((x - xa) / (xb - xa) - t / tb) < 0.01, `${what}: the time axis is linear`);
    };
    // A category's instances: each one's limit and every system's record.
    const rowsOf = (c) => c.rows.map((r) => ({ suite: r[0], name: r[1], limit: r[2], cls: r[3],
      foreign: new Set(c.systems.filter((x, j) => r[3][j] === "x")),
      v: Object.fromEntries(c.systems.map((x, j) => [x, r[4 + j]])) }));
    // How many rows the fastest of xs answers.
    const virtual = (rows, xs) => rows.filter((r) => xs.some((x) => number(r.v[x]))).length;
    // A category's other systems by their answers, then tddlog's speed-up
    // over them, the least first and a system without one last.
    const ranked = (c, xs) => xs.filter((x) => !isMine(x))
      .sort((a, b) => c.answers[b] - c.answers[a] || (c.vs[a].speedup ?? Number.MAX_VALUE) - (c.vs[b].speedup ?? Number.MAX_VALUE));
    // How many times less time tddlog takes, as the README writes it.
    const ratio = (a, b) => `${a / b >= 10 ? (a / b).toFixed(0) : (a / b).toFixed(1)}×`;
    const speed = (r) => (r >= 1 ? ratio(r, 1) : `${ratio(1, r)} slower`);
    // A speed-up's text: nothing to measure, answers none where the other
    // system answers no instance (its speed-up is the measure's ceiling), or
    // the ratio.
    const speedText = (v) => (v.speedup === null ? "nothing to measure" : v.answers === 0 ? "answers none" : speed(v.speedup));
    // tddlog against a system by rule 16 over runs [limit, tddlog's record,
    // its record], the instances it is given: the two totals over those
    // either answers, a run that does not answer counted as twice its limit
    // (PAR-2), and the speed-up F: the ratio of the totals over those both
    // answer (but those both answer in under 0.2 s) and over each instance
    // only one answers, the other at the limit, where that raises the
    // multiplier of the one that answered it (an instance only tddlog answers
    // where the limit over its time is at least F, one only the system answers
    // where the limit over its time is at least 1 / F); none only where there
    // is nothing to measure, no instance both answer over 0.2 s and none only
    // one answers (no minimum in common). F is the root of the left side of
    // that balance, which decreases in F, found here by bisection (panels.py
    // walks its pieces); with none both answer over 0.2 s and only one side's
    // instances, the left side is 0 past the largest of tddlog's bounds, or
    // before the least of the system's, and F is that bound. A time under
    // 0.2 s counts as 0.2 s. (0.2 and 2: the data's measure, panels.py's
    // parameters.)
    const { floor, par: times } = cats.measure;
    const compare = (runs) => {
      const f = (t) => Math.max(t, floor);
      const par = (limit, v) => (number(v) ? f(v) : times * limit);
      const either = runs.filter(([, t, x]) => number(t) || number(x));
      const both = either.filter(([, t, x]) => number(t) && number(x) && Math.max(t, x) >= floor);
      const out = { given: runs.length, answers: runs.filter(([, , x]) => number(x)).length,
        tddlog_answers: runs.filter(([, t]) => number(t)).length, either: either.length,
        total: either.reduce((s, [l, , x]) => s + par(l, x), 0), tddlog_total: either.reduce((s, [l, t]) => s + par(l, t), 0),
        both: both.length, ratio: null, speedup: null };
      const failures = either.filter(([, t, x]) => number(x) && !number(t)).map(([l, , x]) => [f(x), l]);
      const timeouts = either.filter(([, t, x]) => number(t) && !number(x)).map(([l, t]) => [f(t), l]);
      if (!(both.length || failures.length || timeouts.length)) return out;
      if (!both.length && !failures.length) return { ...out, speedup: Math.max(...timeouts.map(([t, l]) => l / t)) };
      if (!both.length && !timeouts.length) return { ...out, speedup: Math.min(...failures.map(([x, l]) => x / l)) };
      const mine = both.reduce((s, [, t]) => s + f(t), 0), theirs = both.reduce((s, [, , x]) => s + f(x), 0);
      const g = (F) => theirs - F * mine + timeouts.reduce((s, [t, l]) => s + Math.max(0, l - F * t), 0)
        + failures.reduce((s, [x, l]) => s + Math.min(0, x - F * l), 0);
      let [lo, hi] = [1e-12, 1e12];
      for (let k = 0; k < 200; k++) {
        const m = Math.sqrt(lo * hi);
        if (g(m) > 0) lo = m; else hi = m;
      }
      return { ...out, ratio: both.length ? theirs / mine : null, speedup: Math.sqrt(lo * hi) };
    };
    // A comparison of the data's is the oracle's: its counts, its totals to
    // 0.01 s, its ratio and speed-up to their four digits.
    const same = (v, w, what) => {
      for (const k of ["given", "answers", "tddlog_answers", "either", "both"]) assert.equal(v[k], w[k], `${what}: ${k}`);
      for (const k of ["total", "tddlog_total"]) assert.ok(Math.abs(v[k] - w[k]) < 0.01, `${what}: ${k}, ${v[k]} against ${w[k]}`);
      for (const k of ["ratio", "speedup"]) {
        assert.equal(v[k] === null, w[k] === null, `${what}: a ${k} or none`);
        if (w[k] !== null) assert.ok(Math.abs(v[k] / w[k] - 1) < 1e-3, `${what}: ${k}, ${v[k]} against ${w[k]}`);
      }
    };
    // Each row's subject (key) against x over rows, those that give x a run of a program idiomatic
    // for it unless every: a translation that is not (the letter x) is in its answers and out of a
    // type's comparison, in a family's and all programs'.
    const versus = (rows, x, every = false, key = myKey) => compare(rows.filter((r) => r.v[x] !== null && (every || !r.foreign.has(x)))
      .map((r) => [r.limit, r.v[key(r)], r.v[x]]));
    // The circuits' places (the instances whose circuit an exporter's code
    // exports) and, on each, the systems that read the circuit
    // (categories.json's circuits).
    const circuitRows = cats.circuits ? rowsOf(cats.circuits) : [];
    const places = new Set(circuitRows.map((r) => `${r.suite}/${r.name}`));
    const onCircuit = new Set(circuitRows.flatMap((r) => cats.circuits.systems.filter((x, j) => r.cls[j] !== "-")
      .map((x) => `${r.suite}/${r.name}/${x}`)));
    // x's mate on an instance of suite s (panels.py's mate): tddlog where x
    // reads the circuit (a system's process where the system does), the
    // exporter's pipeline elsewhere on the places, and tddlog off them. On
    // a suite's Alea.jl programs, whose circuits the places copy, tddlog
    // reads a port of the program, and Alea.jl is compared with it there.
    const reads = (s, name, x) => onCircuit.has(`${s.key}/${name}/${x.replace(/-process$/, "")}`);
    const mate = (s, name, x) => (reads(s, name, x) || !places.has(`${s.key}/${name}`)
      ? "tddlog" : cats.circuits.pipeline);
    // A suite's runs of x: [limit, its mate's record, x's record] where x
    // and its mate were given the row. The suite's other systems are those with such a run.
    const suiteRuns = (s, x) => {
      const j = s.systems.indexOf(x) + 1;
      return s.instances.flatMap((row) => {
        const m = s.systems.indexOf(mate(s, row[0], x)) + 1;
        return m > 0 && row[m] !== null && row[j] !== null ? [[s.limit, row[m], row[j]]] : [];
      });
    };
    const suiteOthers = (s) => s.systems.filter((x) => !isMine(x) && suiteRuns(s, x).length);
    // A scatter plot's dots under and over the diagonal: each row's subject
    // (key) against the fastest of `others`, the instances both answer in
    // under 0.2 s and those neither answers left out.
    const sides = (rows, others, key = myKey) => {
      let faster = 0, slower = 0;
      for (const r of rows) {
        const theirs = others.filter((x) => number(r.v[x])).map((x) => r.v[x]);
        const t = number(r.v[key(r)]) ? r.v[key(r)] : null, b = theirs.length ? Math.min(...theirs) : null;
        if ((t === null && b === null) || (t !== null && b !== null && Math.max(t, b) < 0.2)) continue;
        const x = b === null ? Infinity : Math.max(b, 0.2), y = t === null ? Infinity : Math.max(t, 0.2);
        if (y < x) faster++;
        else if (y > x) slower++;
      }
      return [String(faster), String(slower)];
    };
    // The data: each category's answers, systems given, native and drawn,
    // and each system's comparison with tddlog, over the category and
    // narrowed to each family and each system's own benchmarks, are its rows'.
    for (const c of categories) {
      const what = `the category ${c.key}`;
      const rows = rowsOf(c);
      assert.deepEqual(c.systems.slice(0, c.mine.length), c.mine, `${what}: tddlog and its pipelines first in the data`);
      assert.deepEqual(c.systems.filter(isMine), c.mine, `${what}: tddlog and its pipelines`);
      assert.equal(c.instances, rows.length, `${what}: its instances`);
      assert.ok(rows.every(myKey), `${what}: tddlog or a pipeline of it given each instance`);
      for (const x of c.systems) {
        assert.equal(c.answers[x], virtual(rows, [x]), `${what}: ${x}'s answers`);
        assert.equal(c.given[x], rows.filter((r) => given(r, x)).length, `${what}: the instances ${x} was given`);
        if (!isMine(x)) same(c.vs[x], versus(rows, x, c.key === ALL), `${what}: tddlog against ${x}`);
      }
      const parts = [...c.groups.map((g) => [g.key, g.rows.map((i) => rows[i])]),
        ...c.systems.map((x, j) => [`own:${x}`, rows.filter((r) => r.cls[j] === "o")]).filter(([, sub]) => sub.length)];
      assert.deepEqual(Object.keys(c.narrowed).sort(), parts.map(([key]) => key).sort(), `${what}: narrowed to each family and own benchmarks`);
      for (const [key, sub] of parts) {
        const given = c.systems.filter((x) => !isMine(x) && sub.some((r) => r.v[x] !== null));
        assert.deepEqual(Object.keys(c.narrowed[key]).sort(), [...given].sort(), `${what} narrowed to ${key}: its systems`);
        for (const x of given) same(c.narrowed[key][x], versus(sub, x, true), `${what} narrowed to ${key}: tddlog against ${x}`);
      }
      // ranked by their answers in the comparison beside them (in a type, the instances as their users would write them)
      const byVs = (xs) => [...xs].sort((a, b) => c.vs[b].answers - c.vs[a].answers || (c.vs[a].speedup ?? Number.MAX_VALUE) - (c.vs[b].speedup ?? Number.MAX_VALUE));
      assert.deepEqual(c.native, byVs(c.native), `${what}: the native systems by their answers, then speed-up`);
      if (c.key === ALL) assert.deepEqual([...c.native].sort(), c.systems.filter((x) => !isMine(x)).sort(), `${what}: every system`);
      const shown = c.native.filter((x) => c.given[x] >= cats.measure.drawn);
      assert.deepEqual(c.drawn, [...c.mine, ...shown], `${what}: the systems drawn`);
      const { leads } = c.others;
      assert.deepEqual([...c.named, ...shown.filter((x) => !c.named.includes(x) && !leads.includes(x))], [...c.named, ...c.others.systems],
        `${what}: the named systems, then the others`);
      assert.deepEqual([...c.named, ...c.others.systems, ...leads].sort(), [...shown].sort(), `${what}: each shown system named, among the others or led`);
      const rest = shown.slice(c.named.length);
      assert.ok(c.named.length === shown.length || (c.named.length === cats.measure.named && rest.length > 1), `${what}: the best named, or all of them`);
      // the others tddlog leads most below their row, the largest speed-ups first: three, or
      // every one of them where they are five or fewer (panels.py's LEADS and FEW)
      const byLead = [...rest].sort((a, b) => (c.vs[a].speedup === null) - (c.vs[b].speedup === null) || (c.vs[b].speedup ?? 0) - (c.vs[a].speedup ?? 0));
      assert.deepEqual(leads, rest.length <= 5 ? byLead : byLead.filter((x) => c.vs[x].speedup !== null).slice(0, 3), `${what}: the others tddlog leads most`);
      if (c.others.systems.length) {
        const vs = c.others.systems.map((x) => c.vs[x]);
        const fast = vs.map((v) => v.speedup).filter((r) => r !== null);
        assert.equal(c.others.speedup, fast.length ? Math.min(...fast) : null, `${what}: the others' least speed-up`);
        assert.equal(c.others.total, Math.min(...vs.map((v) => v.total)), `${what}: the others' least total`);
        assert.deepEqual(c.others.answers, [Math.min(...c.others.systems.map((x) => c.vs[x].answers)),
          Math.max(...c.others.systems.map((x) => c.vs[x].answers))], `${what}: the range of the others' answers`);
        assert.deepEqual(c.others.tddlog, [Math.min(...vs.map((v) => v.tddlog_total)), Math.max(...vs.map((v) => v.tddlog_total))],
          `${what}: the range of tddlog's totals against the others`);
      }
      assert.equal(c.groups.reduce((n, g) => n + g.rows.length, 0), c.instances, `${what}: each instance in one family`);
    }
    // Each system on its own benchmarks, and each suite's systems, against tddlog.
    for (const o of cats.own) assert.ok(o.vs.given <= o.instances, `${o.system}'s own benchmarks: given at most its instances`);
    for (const s of bySuite.values()) {
      if (!s.systems.some(isMine)) continue;
      const vs = cats.suites[s.key];
      const others = suiteOthers(s);
      assert.deepEqual(Object.keys(vs).sort(), [...others].sort(), `the suite ${s.key}: tddlog against each system`);
      for (const x of others) same(vs[x], compare(suiteRuns(s, x)), `the suite ${s.key}: tddlog against ${x}`);
    }
    assert.equal(categories.at(-1).key, ALL, "all programs, the last category");
    assert.equal(categories.at(-1).instances, types.reduce((n, c) => n + c.instances, 0), "all programs: the types' instances");
    // The overview: for each type, a link to it with its instances and
    // tddlog's answers, a row for each named native system, with its answers
    // and tddlog's speed-up over it (nothing to measure, narrow: none),
    // and a row for the others. Then each
    // category's cactus plot of its systems drawn, tddlog's and the named
    // systems' lines in colour, and its scatter plot of tddlog against the fastest of them; then each
    // system on its own benchmarks.
    await page.goto(`${base}results/`);
    await page.waitForSelector("#view figure.overview svg a.row");
    assert.equal(await page.title(), "tddlog results");
    const heads = await page.$$eval("#view svg.overview a.kind", (as) => as.map((a) => [a.getAttribute("href"), a.textContent]));
    assert.deepEqual(heads, types.map((c) => {
      const rows = rowsOf(c), ms = subjectsOf(rows);
      return [`#/category/${c.key}`, `${c.title}, ${count(c.instances)} ${c.noun}: ${subject(ms)} ${ms.length > 1 ? "answer" : "answers"} ${count(answeredBy(rows))}`];
    }), "the overview: a type a group");
    const overviewRows = await page.$$eval("#view svg.overview a.row.other", (as) => as.map((a) => [a.getAttribute("href"),
      ...["answers", "ratio"].map((c) => (a.querySelector(`text.${c}`)?.textContent ?? "").replace(/^none$/, "nothing to measure"))]));
    assert.deepEqual(overviewRows, types.flatMap((c) => c.named.map((x) => [`#/category/${c.key}/vs/${x}`, count(c.vs[x].answers),
      speedText(c.vs[x])])), "the overview: each named system's answers and tddlog's speed-up");
    assert.equal((await page.$$("#view svg.overview a.row.others")).length, types.filter((c) => c.others.systems.length).length,
      "the overview: a row of the others");
    const leadRows = await page.$$eval("#view svg.overview a.row.lead", (as) => as.map((a) => [a.getAttribute("href"),
      ...["answers", "ratio"].map((c) => (a.querySelector(`text.${c}`)?.textContent ?? "").replace(/^none$/, "nothing to measure"))]));
    assert.deepEqual(leadRows, types.flatMap((c) => c.others.leads.map((x) => [`#/category/${c.key}/vs/${x}`, count(c.vs[x].answers),
      speedText(c.vs[x])])), "the overview: the others tddlog leads most, each with its answers and speed-up");
    // no bound of 0 s on the others' row (a total of 0: no instance it or tddlog answers)
    assert.equal((await page.$$eval("#view svg.overview a.row.others text.both", (ts) => ts.map((t) => t.textContent)))
      .filter((t) => /\b0(\.0)? s\b/.test(t)).length, 0, "the overview: no others' bound of 0 s");
    assert.equal((await page.textContent("#view p.stamp")).replace(/\s+/g, " "),
      `Results as of ${cats.stamp.date}, public main ${cats.stamp.source}.`, "the overview: the data's stamp");
    const sections = await page.$$eval("#view section.kind-figs", (ss) => ss.map((s) => [s.querySelector("h3 a").getAttribute("href"),
      s.querySelector("figure.plot").dataset.systems, s.querySelector("figure.scatter").dataset.faster, s.querySelector("figure.scatter").dataset.slower,
      [...s.querySelectorAll("figure.plot svg.cactus path.line.grey")].map((p) => p.dataset.key).sort()]));
    assert.deepEqual(sections, categories.map((c) => [`#/category/${c.key}`, [...c.mine, ...ranked(c, c.drawn)].join(" "),
      ...sides(rowsOf(c), c.drawn.filter((x) => !isMine(x))), c.drawn.filter((x) => !lit(c, x)).sort()]), "the overview: each category's cactus plot and its scatter plot");
    assert.equal((await page.$$("#view table.board.own tbody tr")).length, cats.own.length, "the overview: each system on its own benchmarks");
    // every speed-up's header a link to the methods table (rule 20): the overview's, and the tables' and cells' on this page
    assert.equal(await page.locator('#view svg a.defined[href="../#/methods"]').count(), 1, "the overview: its speed-up's header links to the methods");
    assert.deepEqual(await page.$$eval("#view thead th, #view .cell > span", (hs) => hs.filter((h) => /\b(faster|speed-up)\b/.test(h.textContent)
      && !h.querySelector('a[href="../#/methods"]')).map((h) => h.textContent)), [], "the overview: every speed-up's header links to the methods");
    console.log(`ok the overview: ${overviewRows.length} native systems' speed-ups, each category's plots and the own benchmarks`);
    // The menu, the site's: its sections, each a heading with its entries:
    // every other system by its answers; the integrations; the
    // categories and the programs tddlog does not run; the walkthroughs and the
    // methods table.
    // by integration where the data has the integrations
    const sectionNames = (await page.evaluate(() => SECTIONS)).map(([, name]) => name);
    // and the circuits where it has them
    const menuNames = (await page.evaluate(() => SECTIONS)).filter(([key]) => (key !== "integrations" || cats.integrations)
      && (key !== "circuits" || cats.circuits)).map(([, name]) => name);
    const groups = await page.$$eval("#tree .tree > li.group", (lis) => lis.map((li) => [li.querySelector(":scope > a").textContent,
      ...[...li.querySelectorAll(":scope > ul > li > a")].map((a) => a.getAttribute("href"))]));
    const withInt = cats.integrations ? groups : [groups[0], [""], ...groups.slice(1)];
    const menu = cats.circuits ? withInt : [...withInt.slice(0, 3), [""], ...withInt.slice(3)];
    const baselines = cats.systems.filter((s) => !isMine(s.key)).sort((a, b) => b.answers - a.answers).map((s) => `../#/baselines/${s.key}`);
    assert.deepEqual(groups.map(([name]) => name), menuNames, "the menu: the bar's sections");
    assert.deepEqual(menu[0].slice(1), baselines, "the menu: the baselines");
    assert.deepEqual(menu[1].slice(1), ["tddlog", "alea", "roulette"].filter((k) => cats.integrations?.[k]).map((k) => `../#/integrations/${k}`),
      "the menu: the integrations");
    assert.deepEqual(menu[2].slice(1), [...categories.map((c) => `#/category/${c.key}`), "../#/coverage"],
      "the menu: the categories and the programs tddlog does not run");
    // the circuits' families only where the view is among the circuits, as a type's under the type on show
    assert.deepEqual(menu[3].slice(1), [], "the menu: the circuits, without their families on the overview");
    const walks = fs.readdirSync(path.join(site, "next", "walk")).filter((f) => f.endsWith(".json")).map((f) => `../#/how/${f.slice(0, -5)}`);
    assert.deepEqual(menu[4].slice(-1), ["../#/methods"], "the menu: the methods table, last");
    assert.deepEqual(menu[4].slice(1, -1).sort(), walks.sort(), "the menu: every walkthrough");
    assert.deepEqual(await page.$$eval("#tree li.legend", (ls) => ls.map((l) => l.textContent)),
      ["instances answered", "instances"], "the menu: what its counts count");
    if (cats.circuits) {
      await visit("#/circuits", "figure.plot svg.cactus path.line");
      const circuits = await page.$$eval("#tree .tree > li.group", (lis) => [...lis.find((li) => li.querySelector(":scope > a").dataset.section === "#/circuits")
        .querySelectorAll(":scope > ul > li > a")].map((a) => a.getAttribute("href")));
      assert.deepEqual(circuits, cats.circuits.groups.map((g) => `#/category/circuits/in/${encodeURIComponent(g.key)}`), "the menu: the circuits' families among the circuits");
    }
    for (const c of categories) {
      const what = `the category ${c.key}`;
      await visit(`#/category/${c.key}`, "figure.plot svg.cactus path.line");
      const rows = rowsOf(c);
      const order = [...c.mine, ...ranked(c, c.drawn)];
      const fig = await shown();
      assert.equal(fig.instances, rows.length, `${what}: its instances`);
      assert.deepEqual(fig.systems, order, `${what}: its systems drawn, tddlog first and then by their answers`);
      assert.deepEqual([...fig.lines].sort(), [...order].sort(), `${what}: its lines`);
      assert.deepEqual(fig.legend, byAnswers(order, (x) => c.answers[x])
        .map((x) => `answers${count(c.answers[x])}of${count(rows.length)}${isMine(x) || (c.vs[x].answers === 0 && c.vs[x].speedup !== null) ? ""
          : c.vs[x].speedup === null ? ",nothingtomeasure"
          : `,${subject(c.vs[x].me)}${speed(c.vs[x].speedup)}${c.vs[x].speedup >= 1 ? "faster" : ""}`.replace(/\s/g, "")}`), `${what}: its legend`);
      assert.deepEqual([...fig.coloured].sort(), order.filter((x) => lit(c, x)).sort(), `${what}: tddlog's and the named systems' lines in colour`);
      assert.ok(fig.axis.includes(`all${count(rows.length)}instances`), `${what}: its axis`);
      linear(fig, what);
      const board = await page.$eval("#view table.board", (t) => [...t.tBodies[0].rows].map((r) => [r.cells[0].textContent.trim(),
        r.cells[1].textContent.trim().replace(/,/g, "")]));
      assert.deepEqual(board, [...c.mine, ...c.native].map((x) => [names[x], String(c.answers[x])]), `${what}: its board`);
      const pageOf = (x) => (isMine(x) ? `../#/integrations/${x === "tddlog" ? x : x.replace(/-tddlog$/, "")}` : `../#/baselines/${x}`);
      assert.deepEqual(await page.$eval("#view table.board", (t) => [...t.tBodies[0].rows].map((r) => r.cells[0].querySelector("a").getAttribute("href"))),
        [...c.mine, ...c.native].map(pageOf), `${what}: each system's name opens its page`);
      // how many each system was not given, in the board's last column, empty for none
      assert.deepEqual(await page.$eval("#view table.board", (t) => [...t.tBodies[0].rows].map((r) => r.cells[r.cells.length - 1].textContent.replace(/,/g, ""))),
        [...c.mine, ...c.native].map((x) => {
          const n = rows.filter((r) => r.v[x] === null).length;
          return n ? String(n) : "";
        }), `${what}: the instances each system was not given`);
      assert.equal((await page.$$("#view ul.losses li")).length, c.losses?.length ?? 0, `${what}: where tddlog is slower`);
      assert.deepEqual(await page.$eval("#view figure.scatter", (f) => [f.dataset.faster, f.dataset.slower]),
        sides(rows, c.drawn.filter((x) => !isMine(x))), `${what}: tddlog against the fastest other system`);
      // each member of tddlog's labelled, a subject given every instance not repeated
      const subjects = subjectsOf(rows);
      const answers = [[`${subject(subjects)} answers`, `${count(answeredBy(rows))} of ${count(c.instances)}`],
        ...(c.mine.length < 2 ? [] : c.mine.filter((x) => !subjects.includes(x) || c.given[x] < c.instances).map((x) =>
          [subjects.includes(x) ? `${names[x]} answers, of those it reads` : `${names[x]} answers`, `${count(c.answers[x])} of ${count(c.given[x])}`]))];
      const headline = await page.$$eval("#view .cells.headline .cell", (cs) => cs.map((x) => [x.querySelector("span").textContent, x.querySelector("b").textContent]));
      assert.deepEqual(headline.slice(0, answers.length), answers, `${what}: its headline's answers, each in a cell`);
      const filters = c.systems.filter((x) => c.classes[x].own > 0).length;
      assert.equal((await page.$$("#view nav.filters a")).length, filters ? filters + 1 : 0, `${what}: its filters`);
      // a row a family, its one link its program's (all programs': a type's, its name), those of a domain
      // together under its heading after the others
      const domains = [...new Set(c.groups.map((g) => g.domain).filter(Boolean))];
      const ordered = [...c.groups.filter((g) => !g.domain), ...domains.flatMap((d) => c.groups.filter((g) => g.domain === d))];
      assert.deepEqual(await page.$$eval("#view table.families tbody a", (as) => as.map((a) => a.getAttribute("href"))),
        ordered.map((g) => `#/category/${c.key}/in/${encodeURIComponent(g.key)}`), `${what}: its families`);
      assert.equal(await page.locator("#view table.families tbody tr.heading.domain").count(), domains.length, `${what}: a heading a domain`);
      assert.equal(await page.title(), `${c.title} · tddlog results`);
    }
    // the line nearest the pointer picked out: one line, in its colour, its name at its end and first in the tip
    const greyed = categories.find((c) => c.drawn.some((x) => !lit(c, x)));
    if (greyed) {
      await visit(`#/category/${greyed.key}`, "figure.plot svg.cactus path.line.grey");
      const key = greyed.drawn.find((x) => !lit(greyed, x));
      await page.$eval("#view svg.cactus", (s) => s.scrollIntoView({ block: "center" }));
      const to = await page.$eval("#view svg.cactus", (s, k) => {
        const r = s.getBoundingClientRect(), hit = s.querySelector("rect.hit"), end = s.querySelector(`text.end.quiet[data-key="${k}"]`);
        return { x: r.left + Number(hit.getAttribute("x")) + Number(hit.getAttribute("width")) - 1, y: r.top + Number(end.getAttribute("y")) - 4 };
      }, key);
      await page.mouse.move(to.x, to.y);
      const picked = await page.$$eval("#view svg.cactus path.line.hl", (ps) => ps.map((p) => p.dataset.key));
      assert.equal(picked.length, 1, `${greyed.key}: one line picked out under the pointer`);
      assert.equal(await page.$eval(`#view svg.cactus text.end.hl[data-key="${picked[0]}"]`, (t) => getComputedStyle(t).visibility), "visible",
        `${greyed.key}: the picked line's name at its end`);
      assert.ok((await page.textContent("#view .tip span.picked")).startsWith(names[picked[0]]), `${greyed.key}: the picked line first in the tip`);
      await page.mouse.move(0, 0);
    }
    console.log(`ok the categories ${categories.map((c) => `${c.key} ${c.instances}`).join(", ")}: every system of a category over all of its instances`);
    // A category narrowed to a family, from its table of families, and to a
    // system's own benchmarks: the address names it, the plot and the board
    // count its instances alone, and the menu marks the family. An address
    // that narrowed a category to a suite opens the category whole.
    for (const c of [categories.find((x) => x.groups.length > 1), categories.find((x) => x.systems.some((y) => x.classes[y].own > 0))]) {
      const g = c.groups.find((x) => x.rows.length > 1) ?? c.groups[0];
      const x = c.systems.find((y) => c.classes[y].own > 0);
      for (const [only, from] of [[g.key, "table.families"], ...(x ? [[`own:${x}`, "nav.filters"]] : [])]) {
        const h = `#/category/${c.key}/in/${encodeURIComponent(only)}`;
        await visit(`#/category/${c.key}`, from);
        await page.click(`#view ${from} a[href="${h}"]`);
        await page.waitForFunction((h) => location.hash === h && document.querySelector("#view nav.filters a[aria-current]")?.getAttribute("href") === h, h);
        const j = only.startsWith("own:") ? c.systems.indexOf(only.slice(4)) : -1;
        const rows = rowsOf(c).filter((r, i) => (j >= 0 ? r.cls[j] === "o" : g.rows.includes(i)));
        assert.equal((await shown()).instances, rows.length, `${c.key} narrowed to ${only}: its instances`);
        const tddlog = await page.$eval("#view table.board tr.mine td", (td) => td.textContent.trim().replace(/,/g, ""));
        assert.equal(tddlog, String(virtual(rows, [c.mine.find((m) => rows.some((r) => r.v[m] !== null))])), `${c.key} narrowed to ${only}: the answers of the first of tddlog's`);
        if (j < 0) {
          assert.equal(await page.getAttribute(`#tree a[href="${h}"]`, "aria-current"), "page", `${c.key} narrowed to ${only}: the menu`);
          assert.equal((await page.$$("#tree .tree > li > ul > li > ul > li > ul")).length, 0, `${c.key} narrowed to ${only}: no pages in the menu`);
          assert.equal(await page.textContent("#view h2"), g.title, `${c.key} narrowed to ${only}: its title`);
          // the family's program first, once, where the site has the benchmarks' data: an instance
          // picked among the family's own, its programs and tddlog's command; no links by suite.
          // Another instance redraws it in place, the address naming it.
          assert.equal(await page.$("#view p.pages"), null, `${c.key} narrowed to ${only}: no pages by suite`);
          if (g.source && await page.$("#view section#programs:not([hidden])")) {
            await page.waitForSelector("#view section#programs .viewer .pane, #view section#programs p.note:not(:first-child)");
            const order = await page.$$eval("#view > *", (es) => es.map((e) => (e.id === "programs" ? "P" : e.tagName === "H3" ? "H" : "")).join(""));
            assert.ok(order.indexOf("P") >= 0 && order.indexOf("P") < order.indexOf("H"), `${c.key} narrowed to ${only}: its program first, ${order}`);
            // the instance picker, not the menu of the instance's programs beside it
            const menu = '#view section#programs select[aria-label="Instance"]';
            const seg = '#view section#programs .seg[aria-label="Instance"]';
            const picks = await page.$$eval(`${menu} option, ${seg} a`, (os) => os.length);
            assert.ok(g.rows.length === 1 ? picks === 0 : picks > 1 && picks <= g.rows.length, `${c.key} narrowed to ${only}: ${picks} instances to pick`);
            if (picks > 1) {
              if (await page.$(menu)) await page.selectOption(menu, "1");
              else await page.click(`${seg} a:nth-child(2)`);
              await page.waitForFunction((h) => location.hash.startsWith(`${h}/program/`), h);
              await page.waitForSelector(`${seg} [aria-current], ${menu}`);
              assert.equal(await page.$eval("#view h2", (e) => e.textContent), g.title, `${c.key} narrowed to ${only}: another instance in place`);
            }
          }
        }
        console.log(`ok ${c.key} narrowed to ${only}, ${rows.length} instances`);
      }
    }
    {
      const c = categories[0];
      const suite = [...new Set(c.rows.map((r) => r[0]))].find((k) => !c.groups.some((g) => g.key === k));
      const whole = `#/category/${c.key}`;
      await page.evaluate((h) => { location.hash = h; }, `${whole}/in/${suite}`);
      await page.waitForFunction((h) => location.hash === h && document.querySelector(`#view[data-route="${h}"] figure.plot svg.cactus path.line`)
        && (document.querySelector("#view nav.filters a[aria-current]")?.getAttribute("href") ?? h) === h, whole);
      assert.equal((await shown()).instances, c.instances, "a category narrowed to a suite opens whole: its instances");
    }
    // A category's times set against one system, from the scatter plot's
    // menu: the address names it, its row is marked and the scatter plot
    // sets tddlog against it alone; the menu sets them against the fastest
    // other system again, in place.
    {
      const c = categories.find((x) => x.systems.length > 5) ?? categories[0];
      const rows = rowsOf(c);
      const x = ranked(c, c.drawn).at(-1);
      await visit(`#/category/${c.key}`, "table.board");
      await page.selectOption("#view select.vs", x);
      await page.waitForFunction((x) => document.querySelector("#view select.vs")?.value === x && document.querySelector("#view table.board tr.vs"), x);
      assert.equal(new URL(page.url()).hash, `#/category/${c.key}/vs/${x}`, `tddlog against ${x}: the address`);
      const vs = await page.$eval("#view", (v) => ({ marked: v.querySelector("table.board tr.vs th").textContent,
        sides: [v.querySelector("figure.scatter").dataset.faster, v.querySelector("figure.scatter").dataset.slower] }));
      assert.equal(vs.marked, names[x], `tddlog against ${x}: its row marked`);
      assert.deepEqual(vs.sides, sides(rows, [x]), `tddlog against ${x}: the scatter plot`);
      await page.selectOption("#view select.vs", "");
      await page.waitForFunction((h) => location.hash === h && document.querySelector("#view select.vs")?.value === "", `#/category/${c.key}`);
      assert.deepEqual(await page.$eval("#view figure.scatter", (f) => [f.dataset.faster, f.dataset.slower]),
        sides(rows, c.drawn.filter((y) => !isMine(y))), "tddlog against the fastest other system again");
      console.log(`ok tddlog against ${x} on ${c.key}, from the scatter plot's menu and back`);
    }
    // Each solver: its answers over every instance of the categories it was
    // given, each instance in one class. A baseline's page (the front page's
    // #/baselines/KEY; #/system/KEY here opens it), its tab of the types: a row for each category,
    // and its plot of the systems drawn and this one, tddlog's line and its
    // own in colour; how it was given the instances, every one of them
    // counted; its paper and its link.
    for (const s of cats.systems) {
      const what = `the solver ${s.key}`;
      const me = isMine(s.key);
      const cs = types.filter((c) => c.systems.includes(s.key));
      assert.deepEqual(s.categories, cs.map((c) => c.key), `${what}: the categories it was given`);
      const rows = cs.flatMap(rowsOf);
      assert.equal(s.answers, virtual(rows, [s.key]), `${what}: its answers`);
      assert.equal(s.given, cs.reduce((n, c) => n + c.given[s.key], 0), `${what}: the instances it was given`);
      // own, native, translated, or translated but not idiomatic
      assert.equal(cs.reduce((n, c) => n + rowsOf(c).filter((r) => "ontx".includes(r.cls[c.systems.indexOf(s.key)])).length, 0), s.given,
        `${what}: each instance it was given in one class`);
      if (me) continue;
      const hash = `#/baselines/${s.key}`;
      await page.goto(`${base}results/#/system/${s.key}`);
      await page.waitForSelector(`#view[data-route="${hash}"] section#idiomatic`);
      assert.equal(new URL(page.url()).hash, hash, `${what}: its old address opens its page`);
      // its tab of the types
      await page.click(`#view nav.tabs a[href="${hash}/types"]`);
      await page.waitForSelector(`#view[data-route="${hash}/types"] section#types table.board`);
      await page.click("#view details.cat-figs-fold > summary");
      // its plot in each type (the page's other panels have theirs)
      const figs = (await plots("#view #types figure.plot")).map((f) => [f.systems.join(" "), f.instances, [...f.coloured].sort().join(" ")]);
      assert.deepEqual(figs, cs.map((c) => {
        const drawn = [...new Set([...c.drawn, s.key])];
        return [[...drawn.filter(isMine), ...ranked(c, drawn)].join(" "), c.instances, [...new Set([...c.mine, s.key])].sort().join(" ")];
      }), `${what}: its plots`);
      for (const fig of await plots()) linear(fig, what);
      assert.equal(await page.$$eval("#view section#types > div table.board tbody tr", (trs) => trs.length), cs.length, `${what}: a row for each category`);
      assert.equal(await page.getAttribute("#view p.lede a", "href"), s.link, `${what}: its link`);
      assert.equal(await page.getAttribute(`#tree a[href="${hash}"]`, "aria-current"), "page", `${what}: marked in the menu`);
      // its paper, in how it was run
      await page.click(`#view nav.tabs a[href="${hash}/run"]`);
      await page.waitForSelector(`#view[data-route="${hash}/run"] div.facts`);
      if (s.cite) assert.ok((await page.textContent("#view div.facts")).includes(s.cite.title), `${what}: its paper`);
      assert.ok((await page.title()).startsWith(`tddlog against ${names[s.key]} · `), `${what}: its page's title`);
    }
    console.log(`ok ${cats.systems.length} solvers: ${cats.systems.map((s) => `${s.key} ${s.answers}`).join(", ")}; the baselines' pages`);
    await page.goto(`${base}results/`);
    await page.waitForSelector("#view figure.overview svg");
    // A suite's address (#/SUITE): a suite gathered instances and has no
    // view of its own. Its address opens the type that holds most of its
    // instances (its copies counted), narrowed to their family where that
    // is one family; the address of a suite in no type opens the circuits
    // likewise.
    {
      const held = (c, k) => c.rows.filter((r) => r[0] === k).length
        + Object.values(c.copies ?? {}).flat().filter((copy) => copy.startsWith(`${k}/`)).length;
      const familiesIn = (c, k) => c.groups.filter((g) => g.rows.some((i) => {
        const key = `${c.rows[i][0]}/${c.rows[i][1]}`;
        return [key, ...(c.copies?.[key] ?? [])].some((x) => x.startsWith(`${k}/`));
      }));
      const opened = [];
      for (const s of bySuite.values()) {
        const [type] = types.filter((c) => held(c, s.key) > 0).sort((a, b) => held(b, s.key) - held(a, s.key));
        const c = type ?? (cats.circuits && held(cats.circuits, s.key) > 0 ? cats.circuits : null);
        if (!c) continue;   // in an integration's rows alone: its address opens the integration's page
        const gs = familiesIn(c, s.key);
        const g = gs.length === 1 ? gs[0] : null;
        const whole = type ? `#/category/${c.key}` : "#/circuits";
        const want = g ? `#/category/${c.key}/in/${encodeURIComponent(g.key)}` : whole;
        await page.evaluate((h) => { location.hash = h; }, `#/${s.key}`);
        await page.waitForFunction(([want, route, title]) => location.hash === want
          && document.querySelector(`#view[data-route="${route}"] figure.plot`) !== null
          && document.querySelector("#view h2")?.textContent === title, [want, whole, g ? g.title : c.title]);
        assert.equal(await page.$("#view table.h2h"), null, `the suite ${s.key}: no view of its own`);
        opened.push(`${s.key} ${want}`);
      }
      assert.ok(opened.length > 0, "a suite's address opens its type's view");
      console.log(`ok ${opened.length} of ${bySuite.size} suites' addresses, each to its type's view: ${opened.join(", ")}`);
    }

    // A plot: the time under the pointer, and the time the arrow keys
    // choose; a grey line's name colours it, and a button every line.
    await visit(`#/category/${categories[0].key}`, "figure.plot svg.cactus rect.hit");
    const area = await page.$("#view figure.plot svg.cactus rect.hit");
    await area.scrollIntoViewIfNeeded();
    const hit = await area.boundingBox();
    await page.mouse.move(hit.x + hit.width / 2, hit.y + hit.height / 2);
    assert.match(await page.textContent("#view figure.plot .tip"), /^within [0-9.]+ s/);
    await page.mouse.move(0, 0);
    await page.focus("#view figure.plot svg.cactus");
    await page.keyboard.press("Home");
    assert.match(await page.textContent("#view figure.plot .tip"), /^within 0\.20 s/, "the keyboard's time limit");
    // a failure's short meaning, as a key of a grid of times gives it
    assert.deepEqual(await page.evaluate(() => ["memout", "skipped", "wrong"].map((k) => FAILURES[k][1].split(":")[0])),
      ["over the memory limit", "after a smaller size failed", "off the reference"], "the failures' short meanings");
    if (greyed) {
      await visit(`#/category/${greyed.key}`, "figure.plot svg.cactus path.line.grey");
      // (the category's own plot, the first of its view)
      const fig = page.locator("#view figure.plot").first();
      const greys = await fig.locator('.legend button[aria-pressed="false"]').count();
      const grey = await fig.locator('.legend button[aria-pressed="false"]').first().getAttribute("data-system");
      assert.ok(await fig.locator(`svg.cactus path.line.grey[data-key="${grey}"]`).count() > 0, `${greyed.key}: ${grey}'s line grey`);
      await fig.locator(`.legend button[data-system="${grey}"]`).click();
      await fig.locator(`.legend button[data-system="${grey}"][aria-pressed="true"]`).waitFor();
      assert.equal(await fig.locator(`svg.cactus path.line.grey[data-key="${grey}"]`).count(), 0, `${greyed.key}: ${grey}'s line in colour`);
      // (the button colours every line while some are grey, and greys them again once none is)
      if (greys > 1) {
        await fig.locator("button.every").click();
        assert.equal(await fig.locator('.legend button[aria-pressed="false"]').count(), 0, `${greyed.key}: every line in colour`);
      }
    }
    // The former pages' addresses: a family of suites (#programs) and a kind
    // of problems (#/kind/networks) open the category that holds them now.
    for (const [old, now] of [["programs", "logic-programs"], ["/kind/networks", "bayesian-networks"]]) {
      await page.evaluate((h) => { location.hash = h; }, old);
      await page.waitForSelector(`#view[data-route="#/category/${now}"] figure.plot`);
    }
    // An address that names nothing of the results says so.
    await visit("#/no-such-suite", "h2");
    assert.equal(await page.textContent("#view h2"), "Not found");
    assert.equal(await page.title(), "Not found · tddlog results");
    console.log(`ok the results page's plot: ${categories[0].key}${greyed ? `, ${greyed.key}'s colours` : ""}; the former addresses; an address of nothing`);

    // The README's sentences on where tddlog is slower, on a category's
    // page: given to a category that has none in the data.
    const plain = categories.find((c) => !c.losses?.length);
    if (plain) {
      const routed = await context.newPage();
      routed.on("pageerror", (error) => errors.push(String(error)));
      await routed.route((url) => url.pathname === "/results/categories.json", async (route) => {
        const response = await route.fetch();
        const json = await response.json();
        json.categories.find((c) => c.key === plain.key).losses = ["Another system is faster on **these** instances."];
        await route.fulfill({ response, json });
      });
      await routed.goto(`${base}results/#/category/${plain.key}`);
      await routed.waitForSelector("#view ul.losses li");
      assert.equal(await routed.textContent("#view ul.losses li"), "Another system is faster on these instances.");
      await routed.close();
      console.log(`ok ${plain.key}: where tddlog is slower, where the data says`);
    }

    // The benchmarks' data, where the site has it: every suite's families
    // hold its instances, each once, and each family's file its instances,
    // and each program's text (in the family's file, or a file of its own)
    // or why it is not shown. A family's view: its times, an instance's
    // programs, the systems whose files were not captured named, a link to
    // open the program tddlog read in the playground (or why it cannot), a
    // system's runs in the drawer, which the address names; a file of its
    // own, gzipped, and saved under its name; the search; a link to a
    // benchmark's former page leads to its view.
    const benchmarks = path.join(site, "results", "benchmarks");
    let family = null;
    if (fs.existsSync(path.join(benchmarks, "index.json"))) {
      const index = JSON.parse(fs.readFileSync(path.join(benchmarks, "index.json"), "utf8"));
      // (tddlog's run of an instance given to a pipeline reads as the pipeline)
      const readsAs = (name) => (reader) => reader === name || reader.startsWith(`${name} `) || (name === "tddlog" && /^\S+\+tddlog( |$)/.test(reader));
      const hashOf = (x) => `#/${x.suite}/${encodeURIComponent(x.key)}/${encodeURIComponent(x.name)}`;
      let nf = 0, nfiles = 0;
      let unseen = null, tried = null, kept = null, inline = null, multi = null;
      const observed = (name) => /\.(inst|evid)$/.test(name);
      // (a copy of another suite's instance, categories.json's copies, may have no page of its own)
      const copied = new Set(categories.flatMap((c) => Object.values(c.copies ?? {}).flat()));
      // each instance's family on the types' views and the circuits' (categories.json's groups), a
      // copy's its instance's: the benchmarks' pages list it in that family, under its key and title
      const familyAt = new Map();
      for (const c of [...types, ...(cats.circuits ? [cats.circuits] : [])]) {
        for (const g of c.groups) {
          for (const i of g.rows) {
            const key = `${c.rows[i][0]}/${c.rows[i][1]}`;
            for (const k of [key, ...(c.copies?.[key] ?? [])]) familyAt.set(k, g);
          }
        }
      }
      for (const [suite, fams] of Object.entries(index.suites)) {
        const s = bySuite.get(suite);
        assert.ok(s, `benchmarks/index.json: the suite ${suite} is in the results' data`);
        const held = fams.flatMap((f) => f.instances);
        const placed = new Set(held);
        assert.deepEqual(held.sort(), s.instances.map((row) => row[0]).filter((i) => placed.has(i) || !copied.has(`${suite}/${i}`)).sort(),
          `${suite}: its families hold its instances, but copies without a page`);
        for (const f of fams) {
          for (const i of f.instances) {
            const g = familyAt.get(`${suite}/${i}`);
            assert.ok(g, `${suite}/${i}: in a family of the types' views`);
            assert.deepEqual([f.key, f.title], [g.key, g.title], `${suite}/${i}: its family on the types' views`);
          }
          const fam = JSON.parse(fs.readFileSync(path.join(benchmarks, suite, `${f.key}.json`), "utf8"));
          assert.deepEqual(fam.instances.map((i) => i.name), f.instances, `${suite}/${f.key}: its instances`);
          for (const inst of fam.instances) {
            const at = { suite, key: f.key, name: inst.name };
            for (const [ti, t] of (inst.tabs ?? []).entries()) {
              for (const [fi, file] of t.files.entries()) {
                if (file.src) assert.ok(fs.existsSync(path.join(benchmarks, file.src)), `${suite}/${inst.name}: ${file.src}`);
                else if (!file.held && !file.archive) assert.ok(file.sha in fam.texts, `${suite}/${inst.name}: the text of ${file.name}`);
                if (!inline && (ti || fi) && !file.src && file.sha in fam.texts && fam.texts[file.sha].length > 0) {
                  inline = { ...at, ti, fi, file: file.name, text: fam.texts[file.sha] };
                }
                nfiles++;
              }
            }
            if (!inst.tabs) continue;
            if (family === null || (suite === "geni" && family.suite !== "geni")) family = { ...at, tabs: inst.tabs.length, inst };
            const readers = inst.tabs.flatMap((t) => t.readers);
            const ran = [...new Set((inst.runs ?? []).map((r) => names[r.system] ?? r.system))];
            if (!unseen && ran.some((n) => !readers.some(readsAs(n)))) unseen = { ...at, inst };
            // a tab whose data lists its evidence before the model it observes
            const ti = inst.tabs.findIndex((t) => !t.readers.some(readsAs("tddlog")) && t.files.length > 1
              && observed(t.files[0].name) && t.files.some((x) => !observed(x.name)));
            if (!multi && ti >= 0) multi = { ...at, ti, inst };
            const mine = inst.tabs.find((t) => t.readers.includes("tddlog"));
            // the file the playground opens: the instance's program where the family's has places (code.js)
            const read = mine?.files.filter((file) => file.name.endsWith(".pl")) ?? [];
            const filled = read.map((file) => filledFile(inst, mine, file, fam.texts));
            // tddlog's command says what the text says of its places, for a text the site keeps on its own
            for (const [k, file] of read.entries()) {
              if (file.sha in fam.texts) assert.deepEqual(filledFile(inst, mine, file, {}), filled[k], `${suite}/${inst.name}: the places of ${file.name}`);
            }
            const pl = read.map((file, k) => filled[k]?.file ?? file);
            if (pl.length === 1 && !pl[0].held && !pl[0].part && !tried) tried = { ...at, filled: filled[0] !== null, runs: (inst.runs ?? []).filter((r) => r.system === "tddlog") };
            if (pl.length === 1 && (pl[0].held || pl[0].part) && !kept) kept = at;
          }
          nf++;
        }
      }
      assert.ok(family, "an instance with its programs");
      const hash = hashOf(family);
      await visit(hash, "#programs select.version");
      assert.equal(await page.title(), `${family.name} · ${family.suite}/${family.key} · tddlog results`);
      // the menu marks the family of its instance, as it marks a category
      // narrowed to that family
      {
        const key = `${family.suite}/${family.name}`;
        const placed = types.flatMap((c) => c.groups.filter((g) => g.rows.some((i) => {
          const k = `${c.rows[i][0]}/${c.rows[i][1]}`;
          return k === key || (c.copies?.[k] ?? []).includes(key);
        })).map((g) => `#/category/${c.key}/in/${encodeURIComponent(g.key)}`));
        const marked = await page.$$eval('#tree a[aria-current="page"]', (as) => as.map((a) => a.getAttribute("href")));
        assert.deepEqual(marked, placed.slice(-1), `${family.name}: its family marked in the menu`);
      }
      assert.equal((await page.$$("#programs select.version option")).length, family.tabs, `${family.name}: an entry a language`);
      // The program tddlog read shows first; a translation another system
      // read warns that it may make the comparison unfair, the original does not.
      const mine = family.inst.tabs.findIndex((t) => t.readers.includes("tddlog"));
      assert.equal(await page.$eval("#programs select.version", (s) => Number(s.value)), Math.max(mine, 0), `${family.name}: tddlog's program first`);
      const warned = family.inst.tabs.findIndex((t, ti) => ti !== Math.max(mine, 0) && t.translated && t.readers.some((r) => r !== "reference"));
      if (warned >= 0 && family.inst.tabs[0].original) {
        await page.selectOption("#programs select.version", String(warned));
        await page.waitForSelector(`#programs .viewer[data-file^="${warned}:"] .pane p.unfair`);
        assert.match(page.url(), new RegExp(`/tab/${warned}(/\\d+)?$`), `${family.name}: the address names the program`);
        // beside the original, the warning is on the translation's pane alone
        await page.click("#programs .viewer label.check input");
        await page.waitForSelector("#programs .panes.two");
        assert.deepEqual(await page.$$eval("#programs .panes > .pane", (ps) => ps.map((p) => p.querySelector(":scope > p.unfair") !== null)),
          [false, true], `${family.name}: the warning above the translation, not the original`);
        await page.click("#programs .viewer label.check input");
        await page.waitForSelector("#programs .panes:not(.two)");
        await page.selectOption("#programs select.version", "0");
        await page.waitForSelector('#programs .viewer[data-file^="0:"]');
        assert.equal(await page.$("#programs p.unfair"), null, `${family.name}: the original is not a translation`);
      }
      await page.waitForFunction(() => {
        const pane = document.querySelector("#programs .pane");
        return pane !== null && !pane.textContent.includes("Loading");
      });
      assert.ok(await page.$("#programs .pane .lines pre.src, #programs .pane p.held, #programs .pane p.note"), `${family.name}: the file on show`);

      // Every system that ran an instance reads one of its languages, ran a
      // translation of one, or is named as one whose files were not
      // captured, unless it was handed no file (results.js's uncaptured):
      // its grounding failed, it was never handed a program (a translation
      // past the limit) or it was skipped.
      for (const x of [family, unseen].filter(Boolean)) {
        await visit(hashOf(x), "#programs select.version");
        // each entry of the menu names its readers after "read by"
        const readers = (await page.$$eval("#programs select.version option", (os) => os.map((o) => o.textContent)))
          .flatMap((o) => o.split(/(?:: |, )read by /)[1]?.split(/, | and /) ?? []);
        const translated = x.inst.tabs.flatMap((t) => (t.translations ?? []).map(([who]) => who));
        const said = await page.$eval("#programs", (p) => p.querySelector("p.uncaptured")?.textContent ?? "");
        for (const n of new Set((x.inst.runs ?? []).filter((r) => !r.ungrounded && !r.unhanded && !r.skipped).map((r) => names[r.system] ?? r.system))) {
          assert.ok(readers.some(readsAs(n)) || translated.some(readsAs(n)) || said.includes(n),
            `${x.name}: ${n} ran it, and neither reads a language of it, translates one nor is named as not captured`);
        }
      }

      // A tab's files are tabs on its pane, the model before its evidence;
      // the arrow keys go from one to the next, and the address names it.
      if (multi) {
        await visit(hashOf(multi), "#programs select.version");
        await page.selectOption("#programs select.version", String(multi.ti));
        const files = multi.inst.tabs[multi.ti].files;
        const order = [...files.keys()].sort((a, b) => observed(files[a].name) - observed(files[b].name));
        await page.waitForSelector(`#programs .viewer[data-file="${multi.ti}:${order[0]}"]`);
        assert.deepEqual(await page.$$eval("#programs .pane [role=tab]", (bs) => bs.map((b) => b.firstChild.textContent)),
          order.map((k) => files[k].name.split("/").pop()), `${multi.name}: the files' tabs, the evidence last`);
        await page.focus('#programs .pane [role=tab][aria-selected="true"]');
        await page.keyboard.press("ArrowRight");
        await page.waitForSelector(`#programs .viewer[data-file="${multi.ti}:${order[1]}"]`);
        assert.match(page.url(), new RegExp(`/tab/${multi.ti}${order[1] ? `/${order[1]}` : ""}$`), `${multi.name}: the address names the file`);
        assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("aria-selected")), "true", `${multi.name}: the focus stays on the tabs`);
      }

      // A system's runs in a drawer the address names: it closes going back
      // and opens again going forward, closes on Escape, and when another
      // view is chosen.
      await visit(hash, "#programs select.version");
      await page.click("#view table.grid tbody tr.on td button");
      await page.waitForSelector("#drawer:not([hidden]) h4");
      assert.ok((await page.textContent("#drawer h3")).length > 0, "the drawer names its system");
      assert.match(page.url(), /\/runs\/[^/]+$/, "the drawer's address");
      await page.evaluate(() => history.back());
      await page.waitForSelector("#drawer", { state: "hidden" });
      await page.evaluate(() => history.forward());
      await page.waitForSelector("#drawer:not([hidden]) h4");
      await page.keyboard.press("Escape");
      await page.waitForSelector("#drawer", { state: "hidden" });
      assert.doesNotMatch(page.url(), /\/runs\//);
      await page.click("#view table.grid tbody tr.on td button");
      await page.waitForSelector("#drawer:not([hidden]) h4");
      // the drawer may lie over the crumbs on a narrow window: the link followed as a keyboard would
      await page.locator("#view .crumbs a").last().evaluate((a) => a.click());
      await page.waitForSelector(`#view[data-route^="#/category/"]`);
      await page.waitForSelector("#drawer", { state: "hidden" });

      // The program tddlog read opens in the playground, from the instance's
      // view and from tddlog's runs; a file the site holds back or keeps in
      // part says why it does not.
      if (tried) {
        await visit(hashOf(tried), "#programs select.version");
        const link = `../playground/#bench=${[tried.suite, tried.key, tried.name].map(encodeURIComponent).join("/")}`;
        assert.equal(await page.getAttribute("#programs .pick .try a", "href"), link, `${tried.name}: the link to the playground`);
        assert.equal((await page.textContent("#programs .pick .try")).includes("with this instance's arguments in its places"), tried.filled,
          `${tried.name}: the link says whether the program opens with this instance's arguments in its places`);
        await page.click(`#view button[data-runs="${tried.suite}/${tried.name}/tddlog"]`);
        await page.waitForSelector("#drawer:not([hidden]) h4");
        assert.equal(await page.getAttribute("#drawer .try a", "href"), link, `${tried.name}: the link in tddlog's runs`);
        // Each group of runs says which run it is, beside its form: the
        // release run, or a later run by its place among the later runs
        // (the data gives a run's job, and no date or build); the latest first.
        if (tried.runs.length) {
          const jobs = [...new Set(tried.runs.map((r) => r.later).filter(Boolean))];
          const which = (r) => `${r.label}, ${!r.later ? "the release run" : jobs.length > 1 ? `later run ${jobs.indexOf(r.later) + 1} of ${jobs.length}` : "a later run"}`;
          const told = await page.$$eval("#drawer details.run summary", (ss) => ss.map((s) => s.querySelector("b").textContent + s.querySelector(".which").textContent));
          assert.equal(told[0], which(tried.runs.at(-1)), `${tried.name}: tddlog's latest run named`);
          const all = new Set(tried.runs.map(which));
          for (const t of told) assert.ok(all.has(t), `${tried.name}: tddlog's runs, "${t}" is one of ${[...all].join("; ")}`);
        }
        // A file it reads: shown in the view, the drawer closed.
        const read = await page.$eval("#drawer .files button.file", (b) => b.firstChild.textContent);
        await page.click("#drawer .files button.file");
        await page.waitForSelector("#drawer", { state: "hidden" });
        assert.ok((await page.textContent("#programs .pane code.fname")).endsWith(read), `${tried.name}: the file tddlog reads on show`);
      }
      if (kept) {
        await visit(hashOf(kept), "#programs select.version");
        assert.match(await page.textContent("#programs .pick .no-try"), /^Not in the playground/, `${kept.name}: why the playground cannot open it`);
        assert.equal(await page.$("#programs .try"), null);
      }

      // A file of its own, gzipped (files/….txt.gz, as the data writes a
      // large one): the page inflates it, the address names it
      // (…/tab/T/F), and Save hands the reader its text under its name.
      if (inline) {
        const gz = await context.newPage();
        gz.on("pageerror", (error) => errors.push(String(error)));
        await gz.route((url) => url.pathname === `/results/benchmarks/${inline.suite}/${inline.key}.json`, async (route) => {
          const response = await route.fetch();
          const json = await response.json();
          const file = json.instances.find((i) => i.name === inline.name).tabs[inline.ti].files[inline.fi];
          file.src = "files/test.txt.gz";
          delete json.texts[file.sha];
          await route.fulfill({ response, json });
        });
        await gz.route((url) => url.pathname === "/results/benchmarks/files/test.txt.gz", (route) =>
          route.fulfill({ status: 200, contentType: "application/gzip", body: zlib.gzipSync(inline.text) }));
        const at = `${hashOf(inline)}/tab/${inline.ti}${inline.fi ? `/${inline.fi}` : ""}`;
        await gz.goto(`${base}results/${at}`);
        await gz.waitForSelector(`#programs .viewer[data-file="${inline.ti}:${inline.fi}"]`);
        await gz.waitForSelector("#programs .pane pre.src");
        assert.equal(await gz.$eval("#programs .pane pre.src", (pre) => pre.textContent.split("\n")[0]), inline.text.split("\n")[0],
          `${inline.name}: ${inline.file}, inflated`);
        assert.equal(new URL(gz.url()).hash, at, `${inline.name}: the address of ${inline.file}`);
        const [download] = await Promise.all([gz.waitForEvent("download"), gz.click("#programs .pane button.save")]);
        assert.equal(download.suggestedFilename(), inline.file.split("/").pop());
        assert.equal(fs.readFileSync(await download.path(), "utf8"), inline.text, `${inline.name}: ${inline.file}, saved`);
        await gz.close();
      }

      // The search finds a family by its title, once, as its type's view
      // narrowed to it, and names no suite; it finds an instance by its
      // name, with the family that holds it; the arrow keys and Enter choose
      // what it finds.
      const found = (name, what) => page.$$eval("#found-list [role=option]", (os, [name, what]) =>
        os.map((o, i) => (o.firstChild.textContent === name && o.lastChild.textContent === what ? i : -1)).filter((i) => i >= 0), [name, what]);
      const choose = async (option) => {
        for (let i = 0; i <= option; i++) await page.keyboard.press("ArrowDown");
        await page.keyboard.press("Enter");
      };
      await visit("#/", "figure.overview svg");
      {
        const c = types[0], g = c.groups[0];
        await page.fill("#search", g.title);
        await page.waitForSelector("#found-list [role=option]");
        assert.equal(await page.getAttribute("#search", "aria-expanded"), "true");
        const options = await found(g.title, `family, ${c.title}`);
        assert.equal(options.length, c.groups.filter((x) => x.title === g.title).length, `the search finds the family ${g.title} of ${c.key} once`);
        assert.deepEqual(await page.$$eval("#found-list [role=option]", (os) => os.map((o) => o.lastChild.textContent).filter((t) => /^suite\b/.test(t))), [],
          "the search names no suite");
        await choose(options[0]);
        // (the family's view, drawn: its address and its title)
        await page.waitForFunction(([h, title]) => location.hash === h && document.querySelector("#view h2")?.textContent === title,
          [`#/category/${c.key}/in/${encodeURIComponent(g.key)}`, g.title]);
      }
      await visit("#/", "figure.overview svg");
      await page.fill("#search", family.name);
      await page.waitForSelector("#found-list [role=option]");
      {
        // the family of the types' views (or the circuits') that holds the instance, else its family of the benchmarks' data
        const key = `${family.suite}/${family.name}`;
        const holds = (c) => c.groups.filter((g) => g.rows.some((i) => {
          const k = `${c.rows[i][0]}/${c.rows[i][1]}`;
          return k === key || (c.copies?.[k] ?? []).includes(key);
        }));
        const group = types.flatMap(holds).at(-1) ?? (cats.circuits ? holds(cats.circuits).at(-1) : undefined);
        const what = group ? group.title : index.suites[family.suite].find((f) => f.key === family.key).title;
        const [option] = await found(family.name, what);
        assert.ok(option !== undefined, `the search finds ${family.name}, of ${what}`);
        await choose(option);
      }
      await page.waitForSelector(`#view[data-route="${hash}"] #programs`);

      await page.goto(`${base}results/benchmarks/${family.suite}/${encodeURIComponent(family.name)}.html`);
      await page.waitForSelector(`#view[data-route="${hash}"] #programs select.version`);
      console.log(`ok the benchmarks' data: ${nf} families, ${nfiles} files; ${family.suite}/${family.name}'s view, runs and search`
        + `${unseen ? `; ${unseen.name}'s files not captured` : ""}${tried ? `; ${tried.name} to the playground` : ""}`
        + `${kept ? `; ${kept.name} held back` : ""}${inline ? `; ${inline.file} gzipped` : ""}${multi ? `; ${multi.name}'s files as tabs` : ""}`);
    }
    await context.close();

    // Both themes at a desktop's width and a phone's: nothing scrolls
    // sideways, and the screenshots, where asked for.
    if (shots) fs.mkdirSync(shots, { recursive: true });
    for (const colorScheme of ["light", "dark"]) {
      for (const width of [1280, 390]) {
        const view = await browser.newContext({ viewport: { width, height: width > 600 ? 900 : 844 }, colorScheme, deviceScaleFactor: width > 600 ? 1 : 2 });
        const shot = await view.newPage();
        shot.on("pageerror", (error) => errors.push(String(error)));
        if (!only) {
        await shot.goto(`${play}?example=car&evidence=1`);
        await answered(shot, "fuel(full)");
        await noSidewaysScroll(shot, `${colorScheme}, ${width} px`);
        if (shots) await shot.screenshot({ path: path.join(shots, `page-${colorScheme}-${width}.png`), fullPage: true });
        // The code's colours, the muted text and the examples' buttons
        // read against their backgrounds (WCAG AA, 4.5:1).
        const contrast = await shot.evaluate(() => {
          const luminance = (color) => {
            const [r, g, b] = color.match(/[\d.]+/g).slice(0, 3).map((v) => {
              const c = Number(v) / 255;
              return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
            });
            return 0.2126 * r + 0.7152 * g + 0.0722 * b;
          };
          const ratio = (a, b) => {
            const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
            return (hi + 0.05) / (lo + 0.05);
          };
          const code = getComputedStyle(document.querySelector(".editor")).backgroundColor;
          const out = {};
          for (const cls of ["com", "str", "num", "kw", "op", "var", "fn", "typ"]) {
            const span = document.createElement("span");
            span.className = `t-${cls}`;
            document.getElementById("program-hl").append(span);
            out[`t-${cls}`] = ratio(getComputedStyle(span).color, code);
            span.remove();
          }
          out.muted = ratio(getComputedStyle(document.querySelector(".hint")).color, code);
          const chip = getComputedStyle(document.querySelector('#examples button[aria-pressed="false"]'));
          out.example = ratio(chip.color, chip.backgroundColor);
          return out;
        });
        for (const [what, r] of Object.entries(contrast)) assert.ok(r >= 4.5, `${colorScheme}: ${what} at ${r.toFixed(2)}:1`);
        // On a phone the examples are one row, and the editor is on the
        // first screen.
        if (width < 600) {
          const [rows, top] = await shot.evaluate(() => [new Set([...document.querySelectorAll("#examples button")].map((b) => b.offsetTop)).size,
            document.querySelector(".editor").getBoundingClientRect().top + scrollY]);
          assert.equal(rows, 1, `${colorScheme}: the examples in one row`);
          assert.ok(top < 844, `${colorScheme}: the editor ${top} px down`);
        }
        // The editor, a selection in it, in each theme.
        if (shots) {
          await shot.evaluate(() => document.getElementById("program").setSelectionRange(250, 420));
          await shot.focus("#program");
          await shot.locator(".editor").screenshot({ path: path.join(shots, `editor-${colorScheme}-${width}.png`) });
        }
        await shot.goto(`${base}guide/`);
        await noSidewaysScroll(shot, `the guide, ${colorScheme}, ${width} px`);
        assert.equal(await shot.title(), "tddlog guide");
        if (shots) await shot.screenshot({ path: path.join(shots, `guide-${colorScheme}-${width}.png`), fullPage: true });
        }
        // The results: the overview, a category, a category of many systems,
        // and a family with a system's runs open
        // beside it. Each system's colour has a contrast of 3:1 or more with
        // the page and the card; on a phone a category's board's rows stack,
        // each value in the window and named by its column.
        const crowded = categories.find((c) => c.drawn.length > 5) ?? categories[0];
        const views = [["results", "", "figure.overview svg"], ["results-category", `#/category/${categories[0].key}`, "figure.plot svg.cactus path.line"],
          ["results-many", `#/category/${crowded.key}`, "figure.plot svg.cactus path.line"]];
        if (family) views.push(["results-family", `#/${family.suite}/${encodeURIComponent(family.key)}/${encodeURIComponent(family.name)}`, "#programs select.version"]);
        for (const [name, hash, ready] of views) {
          await shot.goto(`${base}results/${hash}`);
          await shot.waitForSelector(`#view[data-route="${hash || "#/"}"] ${ready}`);
          if (name === "results") {
            // (a pipeline of tddlog's is drawn in tddlog's colour, its line dashed)
            const keys = [...new Set(Object.keys(names).map((key) => key.replace(/-process$/, "")))]
              .filter((key) => key === "tddlog" || !isMine(key));
            const colours = await shot.evaluate((keys) => {
              const style = getComputedStyle(document.documentElement);
              const get = (name) => style.getPropertyValue(name).trim();
              return { page: get("--bg"), card: get("--card"), systems: keys.map((key) => [key, get(`--s-${key}`)]) };
            }, keys);
            for (const [key, colour] of colours.systems) {
              assert.match(colour, /^#[0-9a-f]{6}$/i, `${key} has a colour of its own, ${colorScheme}`);
              for (const on of [colours.page, colours.card]) {
                assert.ok(contrast(colour, on) >= 3, `${key}'s ${colour} on ${on}: ${contrast(colour, on).toFixed(2)}:1, ${colorScheme}`);
              }
            }
          }
          if (name === "results-category" && width < 600) {
            const cells = await shot.$$eval("#view table.board.stack tbody tr:first-child td", (tds) => tds.filter((td) => td.textContent.trim() !== "")
              .map((td) => [td.getBoundingClientRect().right, getComputedStyle(td, "::before").content]));
            assert.ok(cells.length >= 2, `a category's board on a phone: ${cells.length} values`);
            for (const [right, label] of cells) {
              assert.ok(right <= width, `a category's board on a phone: a value ends at ${right} px`);
              assert.match(label, /:/, "a category's board on a phone: each value named");
            }
          }
          if (name === "results-family") {
            await shot.click("#view table.grid tbody tr.on td button");
            await shot.waitForSelector("#drawer:not([hidden]) h4");
          }
          await noSidewaysScroll(shot, `${name}, ${colorScheme}, ${width} px`);
          assert.match(await shot.title(), /tddlog results$/);
          if (shots) await shot.screenshot({ path: path.join(shots, `${name}-${colorScheme}-${width}.png`), fullPage: true });
        }
        if (!only) {
        const api = await shot.goto(`${base}api/tddlog/index.html`);
        assert.equal(api.status(), 200, "the API documentation");
        // The Python and C pages: the site's links above each page, a
        // description, and the binding's other pages in the sidebar.
        const docs = [["python", "tddlog for Python", [["examples.html", "Examples"], ["reference.html", "Reference"]]],
          ["c", "tddlog for C", [["example.html", "The example"], ["header.html", "The header"]]]];
        for (const [binding, title, pages] of docs) {
          const response = await shot.goto(`${base}${binding}/`);
          assert.equal(response.status(), 200, `the ${binding} page`);
          assert.match(await shot.title(), new RegExp(`^${title}`));
          const bar = await shot.$$eval(".announcement a", (anchors) => anchors.map((a) => a.textContent));
          assert.deepEqual(bar, ["tddlog", ...sectionNames, "Playground", "Guide", "Rust API", "Python", "C", "GitHub"].filter((n) => n !== title.split(" ").pop()),
            `the ${binding} page's links to the site`);
          const [barWidth, barRoom, barHeight, barHeightRoom] = await shot.$eval(".announcement",
            (bar) => [bar.scrollWidth, bar.clientWidth, bar.scrollHeight, bar.clientHeight]);
          assert.ok(barWidth <= barRoom && barHeight <= barHeightRoom,
            `the ${binding} page's links, ${barWidth} by ${barHeight} px in ${barRoom} by ${barHeightRoom}, ${colorScheme}, ${width} px`);
          const sidebar = await shot.$$eval(".sidebar-tree a", (anchors) => anchors.map((a) => [a.getAttribute("href"), a.textContent.trim()]));
          assert.deepEqual(sidebar, pages, `the ${binding} page's sidebar`);
          if (shots) await shot.screenshot({ path: path.join(shots, `${binding}-${colorScheme}-${width}.png`) });
          if (colorScheme !== "light" || width !== 1280) continue;
          for (const [file] of [["index.html"], ...pages]) {
            await shot.goto(`${base}${binding}/${file}`);
            assert.ok(await shot.$eval('meta[name="description"]', (m) => m.content.length > 40), `${binding}/${file}: its description`);
            assert.ok(await shot.$('link[rel~="icon"]'), `${binding}/${file}: its icon`);
          }
          // The theme chosen on the site's other pages is this page's, and
          // one chosen here is theirs.
          await shot.evaluate(() => localStorage.setItem("tddlog-theme", "dark"));
          await shot.goto(`${base}${binding}/`);
          assert.equal(await shot.$eval("body", (b) => b.dataset.theme), "dark", `the ${binding} page takes the site's theme`);
          await shot.locator(".theme-toggle:visible").first().click();
          const [theme, kept] = await shot.evaluate(() => [document.body.dataset.theme, localStorage.getItem("tddlog-theme")]);
          assert.equal(kept, theme === "auto" ? null : theme, `the ${binding} page hands its theme, ${theme}, to the site`);
          await shot.evaluate(() => localStorage.clear());
        }
        }
        await view.close();
      }
    }
    console.log(`ok both themes at 1280 and 390 px${shots ? `, screenshots in ${shots}` : ""}`);
    assert.deepEqual(errors, [], "errors on the page");
  } finally {
    await browser.close();
    server.close();
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
