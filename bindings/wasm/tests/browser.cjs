// The assembled site in headless Chromium: every example's answers on the
// page against the ProbLog column beside them, P(evidence), an edited program
// through the address, an error, Cancel, the theme, the guide's links, the
// pages' links within the site, the
// results page (the overview; each kind of problems' plot, drawing the
// kind's systems over all of its instances with their answers and total
// times; each suite's plot and tddlog against each system; the hover layer
// and the keyboard's, the lines in colour, the table and its sorting, an
// address of nothing; where the site has the benchmarks' data, a family's
// times, programs and runs, the systems whose files were not captured, the
// link to the playground, the drawer's address, a gzipped file, the search,
// and a link to a benchmark's former page; the systems' colours, and a
// board on a phone), the API,
// Python and C pages, and no sideways scrolling at a desktop's width or a
// phone's.
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

const site = path.resolve(process.argv[2]);
const only = process.env.RESULTS_ONLY === "1";     // the results page alone, from a site that has only it
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css",
  ".wasm": "application/wasm", ".pl": "text/plain", ".json": "application/json",
  ".svg": "image/svg+xml", ".gz": "application/gzip",
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
// query (or the result is an error, where `first` is null).
function answered(page, first) {
  return page.waitForFunction((first) => {
    if (document.body.classList.contains("busy")) return false;
    const status = document.getElementById("status").textContent;
    if (first === null) return status === "failed" && document.querySelector("#answers .failed") !== null;
    if (status !== "ready") return false;
    const cell = document.querySelector("#answers td.query");
    return cell !== null && cell.textContent === first;
  }, first, { timeout: 60000 });
}

// The table on show: each row's query, tddlog's answer and ProbLog's.
function rows(page) {
  return page.$$eval("#answers tbody tr", (trs) => trs.map((tr) => [...tr.querySelectorAll("td.query, td.p")].map((td) => td.textContent)));
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

async function noSidewaysScroll(page, what) {
  const [scroll, width] = await page.evaluate(() => [document.documentElement.scrollWidth, window.innerWidth]);
  assert.ok(scroll <= width, `${what}: the page is ${scroll} px wide in a window of ${width}`);
}

(async () => {
  // The browser first: if it cannot start, no server is left listening
  // and node exits with the error.
  const browser = await chromium.launch();
  const server = await serve(site);
  const base = `http://127.0.0.1:${server.address().port}/`;
  const errors = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await context.newPage();
    page.on("pageerror", (error) => errors.push(String(error)));

    if (!only) {
    // The page opens on the first example and runs it, with the release
    // run's results at a glance above it.
    await page.goto(base);
    await answered(page, tddlogExamples[0].problog[0][0]);
    await page.waitForSelector("#glance:not([hidden])");
    const glance = (await page.textContent("#glance-stats")).replace(/(\d),(?=\d)/g, "$1").replace(/\s+/g, " ");
    assert.match(glance, /\d+ instances in the release run \d+ answered by tddlog \d+ answered by tddlog alone/, glance);

    // Every example: tddlog's answers agree with ProbLog's to the digits
    // shown, in the program's order.
    for (const [i, example] of tddlogExamples.entries()) {
      await page.click(`#examples button[data-example="${i}"]`);
      await answered(page, example.problog[0][0]);
      const shown = await rows(page);
      assert.deepEqual(shown.map(([query]) => query), example.problog.map(([query]) => query), `${example.key}: the queries`);
      for (const [query, tddlog, problog] of shown) assert.equal(tddlog, problog, `${example.key}: ${query}`);
      assert.match(page.url(), new RegExp(`example=${example.key}`), `${example.key}: the address`);
      console.log(`ok ${example.key}: ${shown.length} answers on the page agree with ProbLog's`);
    }

    // P(evidence), with ProbLog's beside it.
    await page.click('#examples button[data-example="0"]');
    await answered(page, tddlogExamples[0].problog[0][0]);
    await page.check("#evidence");
    await page.click("#run");
    await page.waitForSelector("#answers .evidence");
    await answered(page, tddlogExamples[0].problog[0][0]);
    const evidence = await page.$$eval("#answers .evidence b", (bs) => bs.map((b) => b.textContent));
    assert.equal(evidence[0], evidence[2], `P(e) ${evidence.join(" ")}`);
    assert.match(page.url(), /evidence=1/);
    console.log(`ok P(evidence) ${evidence[0]}`);

    // An edited program runs, goes in the address, and comes back from it.
    const program = "0.3::a.\n0.6::b.\nc :- a, \\+b.\nquery(c).\n";
    await page.fill("#program", program);
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

    // An error is shown as one, and the solver answers after it.
    await page.fill("#program", "0.5::a query(a).");
    await page.click("#run");
    await answered(page, null);
    assert.match(await page.textContent("#answers .failed"), /^parse error: /);
    await page.selectOption("#strategy", "decision");
    await page.fill("#program", "0.5::a. query(a).");
    await page.click("#run");
    await answered(page, null);
    assert.match(await page.textContent("#answers .failed"), /no fact of this program as a guard/);
    await page.selectOption("#strategy", "auto");
    await page.click("#run");
    await answered(page, "a");
    console.log("ok errors");

    // Cancel ends the run's worker, and a new one answers.
    await page.evaluate(() => {
      document.getElementById("run").click();
      document.getElementById("cancel").click();
    });
    assert.match(await page.textContent("#status"), /stopped|ready/);
    await page.click("#run");
    await answered(page, "a");
    console.log("ok Cancel");

    // The theme button switches the palette and keeps the choice.
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
    const tried = links.filter((h) => h.startsWith("../?")).map((h) => new URLSearchParams(h.slice(4)).get("example"));
    assert.ok(tried.length >= 5, `the guide links to ${tried.length} examples`);
    for (const key of tried) assert.ok(keys.has(key), `the guide links to ${key}, which is not an example`);
    const within = links.filter((h) => h.startsWith("#"));
    for (const h of within) assert.ok(await page.$(h), `the guide's link ${h} has no target`);
    console.log(`ok the guide: ${tried.length} links to examples, ${within.length} within the page`);

    // Every link from the playground, the guide and the results page to a
    // file of the site leads to one.
    const checked = new Set();
    for (const at of ["", "guide/", "results/"]) {
      await page.goto(`${base}${at}`);
      assert.ok(await page.$eval(".site-logo", (img) => img.complete && img.naturalWidth > 0),
        `${at || "the playground"}: the project logo loads`);
      if (at === "results/") await page.waitForSelector("#view .kind-card");
      const hrefs = await page.$$eval("a[href]", (anchors) => anchors.map((a) => a.href));
      for (const href of hrefs) {
        const url = new URL(href);
        url.hash = "";
        if (!url.href.startsWith(base) || url.search !== "" || checked.has(url.href)) continue;
        checked.add(url.href);
        const response = await context.request.get(url.href);
        assert.equal(response.status(), 200, `${at || "the playground"}: the link to ${url.href}`);
      }
    }
    console.log(`ok ${checked.size} links within the site`);
    }

    // The results page. The overview: a card a kind of problems, in the
    // README's order. A kind's view (#/kind/KEY) plots the kind's systems
    // over all of its instances, tddlog first and then by the instances
    // they answer; its legend gives "answers k of N, total T" for each, k
    // the system's answers in the kind's suites and T its total time over
    // the instances some system of the kind answers, a run that does not
    // answer counted at its suite's limit and a time under 0.2 s at 0.2 s,
    // as the README's legends write it; its axis goes to all N, from 0.2 s;
    // under it, one line for each system that was not given some of the
    // instances, and the README's sentences on where tddlog is slower where
    // the data has them. A suite's view (#/SUITE) plots the suite's systems,
    // its legend "answers k of N" in the same order, and tddlog against each
    // system given some of the same instances. A plot of more than five
    // lines draws tddlog's and the three that answer the most in colour.
    const data = ["suites.json", "ppl.json"].map((file) => path.join(site, "results", file))
      .filter((file) => fs.existsSync(file)).map((file) => JSON.parse(fs.readFileSync(file, "utf8")));
    const kinds = data.flatMap((d) => d.kinds).sort((a, b) => a.order - b.order);
    const bySuite = new Map(data.flatMap((d) => d.families.flatMap((f) => f.suites)).map((s) => [s.key, s]));
    const names = Object.assign({}, ...data.map((d) => d.names));
    const number = (v) => typeof v === "number";
    const count = (n) => n.toLocaleString("en");
    const duration = (t) => (t >= 36000 ? `${(t / 3600).toFixed(0)} h` : t >= 3600 ? `${(t / 3600).toFixed(1)} h`
      : t >= 60 ? `${(t / 60).toFixed(0)} min` : t >= 10 ? `${t.toFixed(0)} s` : `${t.toFixed(1)} s`);
    // A plot's lines in its legend's order: tddlog's, then by their answers.
    const byAnswers = (systems, answers) => [...systems].sort((a, b) => (b === "tddlog") - (a === "tddlog") || answers(b) - answers(a));
    const coloured = (order) => order.slice(0, order.length > 5 ? 4 : order.length);
    // The view an address names, once it is drawn: `ready` within it.
    const visit = async (hash, ready) => {
      await page.evaluate((h) => { location.hash = h; }, hash);
      await page.waitForSelector(`#view[data-route="${hash}"] ${ready}`);
    };
    const shown = () => page.$eval("#view figure.plot", (f) => ({
      systems: f.dataset.systems.split(" ").filter(Boolean),
      instances: Number(f.dataset.instances),
      lines: [...f.querySelectorAll("svg.cactus path.line")].map((p) => p.dataset.key),
      legend: [...f.querySelectorAll(".legend .n")].map((s) => s.textContent.replace(/\s/g, "")),
      coloured: [...f.querySelectorAll('.legend button[aria-pressed="true"]')].map((b) => b.dataset.system),
      axis: f.querySelector("svg.cactus").textContent.replace(/\s/g, ""),
    }));
    await page.goto(`${base}results/`);
    await page.waitForSelector("#view .kind-card");
    assert.equal(await page.title(), "tddlog results");
    const cards = await page.$$eval("#view .kinds > a", (as) => as.map((a) => [a.getAttribute("href"), a.querySelector("h3").textContent]));
    assert.deepEqual(cards, kinds.map((k) => [`#/kind/${k.key}`, k.title]), "the overview: a card a kind");
    for (const k of kinds) {
      const what = `the kind ${k.key}`;
      await visit(`#/kind/${k.key}`, "figure.plot svg.cactus path.line");
      const rows = k.suites.flatMap((key) => {
        const s = bySuite.get(key);
        return s.instances.map((row) => ({ limit: s.limit, v: Object.fromEntries(s.systems.map((x, i) => [x, row[i + 1]])) }));
      });
      const answers = (x) => rows.filter((r) => number(r.v[x])).length;
      const scored = rows.filter((r) => k.systems.some((x) => number(r.v[x])));
      const total = (x) => scored.reduce((t, r) => t + (number(r.v[x]) ? Math.max(r.v[x], 0.2) : r.limit), 0);
      assert.equal(k.systems[0], "tddlog", `${what}: tddlog first in the data`);
      const order = byAnswers(k.systems, answers);
      for (const x of order) {
        assert.equal(k.answers[x], answers(x), `${what}: ${x}'s answers in the data`);
        assert.ok(Math.abs(k.total[x] - total(x)) < 0.01, `${what}: ${x}'s total time in the data, ${k.total[x]} against ${total(x)}`);
      }
      assert.equal(k.instances, rows.length, `${what}: its instances in the data`);
      assert.equal(k.scored, scored.length, `${what}: the instances some system answers`);
      const fig = await shown();
      assert.equal(fig.instances, rows.length, `${what}: its instances`);
      assert.deepEqual(fig.systems, order, `${what}: its systems, tddlog first and then by their answers`);
      assert.deepEqual([...fig.lines].sort(), [...order].sort(), `${what}: its lines`);
      assert.deepEqual(fig.legend, order.map((x) => `answers${count(answers(x))}of${count(rows.length)},total${duration(k.total[x]).replace(/\s/g, "")}`), `${what}: its legend`);
      assert.deepEqual(fig.coloured, coloured(order), `${what}: its lines in colour`);
      assert.ok(fig.axis.includes(`all${count(rows.length)}instances`), `${what}: its axis`);
      assert.match(fig.axis, /≤0\.2/, `${what}: the time axis starts at 0.2 s`);
      const missing = order.filter((x) => rows.some((r) => r.v[x] === undefined || r.v[x] === null)).length;
      assert.equal((await page.$$("#view ul.cannot li")).length, missing, `${what}: the systems not given some of its instances`);
      const board = await page.$eval("#view table.board", (t) => [...t.tBodies[0].rows].map((r) => r.cells[1].textContent.trim().replace(/,/g, "")));
      assert.deepEqual(board.map(Number).sort((a, b) => b - a), order.map(answers).sort((a, b) => b - a), `${what}: its systems' answers`);
      assert.equal((await page.$$("#view ul.losses li")).length, k.losses?.length ?? 0, `${what}: where tddlog is slower`);
      assert.equal(await page.title(), `${k.title} · tddlog results`);
    }
    assert.ok(kinds.length >= 2, `${kinds.length} kinds`);
    console.log(`ok the kinds ${kinds.map((k) => `${k.key} ${k.instances}`).join(", ")}: every system of a kind over all of its instances, its answers and total time`);
    for (const s of bySuite.values()) {
      const what = `the suite ${s.key}`;
      await visit(`#/${s.key}`, "figure.plot svg.cactus path.line");
      const fig = await shown();
      assert.equal(fig.instances, s.instances.length, `${what}: its instances`);
      assert.deepEqual(fig.systems, s.systems, `${what}: its systems`);
      assert.deepEqual([...fig.lines].sort(), [...s.systems].sort(), `${what}: its lines`);
      // An answer counts within the suite's limit, as the plot draws it.
      const within = (x) => {
        const i = s.systems.indexOf(x) + 1;
        return s.instances.filter((row) => number(row[i]) && row[i] <= s.limit * 1.001).length;
      };
      const order = byAnswers(s.systems, within);
      assert.deepEqual(fig.legend, order.map((x) => `answers${count(within(x))}of${count(s.instances.length)}`), `${what}: its legend`);
      assert.deepEqual(fig.coloured, coloured(order), `${what}: its lines in colour`);
      const me = s.systems.indexOf("tddlog") + 1;
      const others = me === 0 ? [] : s.systems.filter((x, i) => x !== "tddlog"
        && s.instances.some((row) => row[me] !== null && row[i + 1] !== null));
      assert.equal((await page.$$("#view table.h2h tbody tr:not(.words)")).length, others.length, `${what}: tddlog against each system`);
      assert.equal(await page.title(), `${s.title} · tddlog results`);
    }
    console.log(`ok ${bySuite.size} suites: each one's systems, and tddlog against each`);

    // A suite's plot: the time under the pointer, and the time the arrow
    // keys choose; a grey line's name colours it; its table sorts by a
    // column, by the keyboard too, and again the other way.
    const [first] = bySuite.keys();
    await visit(`#/${first}`, "figure.plot svg.cactus rect.hit");
    const area = await page.$("#view svg.cactus rect.hit");
    await area.scrollIntoViewIfNeeded();
    const hit = await area.boundingBox();
    await page.mouse.move(hit.x + hit.width / 2, hit.y + hit.height / 2);
    assert.match(await page.textContent("#view .tip"), /^within [0-9.]+ s/);
    await page.mouse.move(0, 0);
    await page.focus("#view svg.cactus");
    await page.keyboard.press("Home");
    assert.match(await page.textContent("#view .tip"), /^within 0\.20 s/, "the keyboard's time limit");
    await page.click("#view details.instances summary");
    const sortBy = '#view table.times th[data-col="1"]';
    await page.click(`${sortBy} button`);
    assert.equal(await page.getAttribute(sortBy, "aria-sort"), "ascending");
    await page.focus(`${sortBy} button`);
    await page.keyboard.press("Enter");
    assert.equal(await page.getAttribute(sortBy, "aria-sort"), "descending");
    const sorted = await page.$$eval("#view table.times tbody tr", (trs) => trs.map((tr) => tr.cells[1].textContent)
      .filter((t) => /^[0-9.]+$/.test(t)).map(Number));
    assert.ok(sorted.every((t, i) => i === 0 || sorted[i - 1] >= t), `${first}: the times sorted the other way`);
    const many = [...bySuite.values()].find((s) => s.systems.length > 5);
    if (many) {
      await visit(`#/${many.key}`, "figure.plot svg.cactus path.line");
      const grey = await page.getAttribute('#view .legend button[aria-pressed="false"]', "data-system");
      assert.ok(await page.$(`#view svg.cactus path.line.grey[data-key="${grey}"]`), `${many.key}: ${grey}'s line grey`);
      await page.click(`#view .legend button[data-system="${grey}"]`);
      await page.waitForSelector(`#view .legend button[data-system="${grey}"][aria-pressed="true"]`);
      assert.equal(await page.$(`#view svg.cactus path.line.grey[data-key="${grey}"]`), null, `${many.key}: ${grey}'s line in colour`);
      await page.click("#view figure.plot button.every");
      assert.equal(await page.$$eval('#view .legend button[aria-pressed="false"]', (bs) => bs.length), 0, `${many.key}: every line in colour`);
    }
    // The former page's address of a family of suites, #networks.
    const old = data.flatMap((d) => d.families).find((f) => kinds.some((k) => k.key === f.key));
    if (old) {
      await page.evaluate((h) => { location.hash = h; }, old.key);
      await page.waitForSelector(`#view[data-route="#/kind/${old.key}"] figure.plot`);
    }
    // An address that names nothing of the results says so.
    await visit("#/no-such-suite", "h2");
    assert.equal(await page.textContent("#view h2"), "Not found");
    assert.equal(await page.title(), "Not found · tddlog results");
    console.log(`ok the results page's plot and table: ${first}${many ? `, ${many.key}'s colours` : ""}; an address of nothing`);

    // The README's sentences on where tddlog is slower, on a kind's page:
    // given to a kind that has none in the data.
    const plain = kinds.find((k) => !k.losses?.length);
    if (plain) {
      const routed = await context.newPage();
      routed.on("pageerror", (error) => errors.push(String(error)));
      const file = data[0].kinds.some((k) => k.key === plain.key) ? "suites.json" : "ppl.json";
      await routed.route((url) => url.pathname === `/results/${file}`, async (route) => {
        const response = await route.fetch();
        const json = await response.json();
        json.kinds.find((k) => k.key === plain.key).losses = ["Another system is faster on **these** instances."];
        await route.fulfill({ response, json });
      });
      await routed.goto(`${base}results/#/kind/${plain.key}`);
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
      const readsAs = (name) => (reader) => reader === name || reader.startsWith(`${name} `);
      const hashOf = (x) => `#/${x.suite}/${encodeURIComponent(x.key)}/${encodeURIComponent(x.name)}`;
      let nf = 0, nfiles = 0;
      let unseen = null, tried = null, kept = null, inline = null;
      for (const [suite, fams] of Object.entries(index.suites)) {
        const s = bySuite.get(suite);
        assert.ok(s, `benchmarks/index.json: the suite ${suite} is in the results' data`);
        assert.deepEqual(fams.flatMap((f) => f.instances).sort(), s.instances.map((row) => row[0]).sort(), `${suite}: its families hold its instances`);
        for (const f of fams) {
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
            const pl = inst.tabs.find((t) => t.readers.includes("tddlog"))?.files.filter((file) => file.name.endsWith(".pl")) ?? [];
            if (pl.length === 1 && !pl[0].held && !pl[0].part && !tried) tried = at;
            if (pl.length === 1 && (pl[0].held || pl[0].part) && !kept) kept = at;
          }
          nf++;
        }
      }
      assert.ok(family, "an instance with its programs");
      const hash = hashOf(family);
      await visit(hash, "#programs table.given");
      assert.equal(await page.title(), `${family.name} · ${family.suite}/${family.key} · tddlog results`);
      assert.equal((await page.$$("#programs table.given tbody tr")).length, family.tabs, `${family.name}: a row a language`);
      await page.waitForFunction(() => {
        const pane = document.querySelector("#programs .pane");
        return pane !== null && !pane.textContent.includes("Loading");
      });
      assert.ok(await page.$("#programs .pane .lines pre.src, #programs .pane p.held, #programs .pane p.note"), `${family.name}: the file on show`);

      // Every system that ran an instance reads one of its languages, or is
      // named as one whose files were not captured.
      for (const x of [family, unseen].filter(Boolean)) {
        await visit(hashOf(x), "#programs table.given");
        const readers = await page.$$eval("#programs table.given .chip", (chips) => chips.map((c) => c.textContent));
        const said = await page.$eval("#programs", (p) => p.querySelector("p.uncaptured")?.textContent ?? "");
        for (const n of new Set((x.inst.runs ?? []).map((r) => names[r.system] ?? r.system))) {
          assert.ok(readers.some(readsAs(n)) || said.includes(n), `${x.name}: ${n} ran it, and neither reads a language of it nor is named as not captured`);
        }
      }

      // A system's runs in a drawer the address names: it closes going back
      // and opens again going forward, closes on Escape, and when another
      // view is chosen.
      await visit(hash, "#programs table.given");
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
      await page.click("#view .crumbs a >> nth=-1");
      await page.waitForSelector(`#view[data-route="#/${family.suite}"]`);
      await page.waitForSelector("#drawer", { state: "hidden" });

      // The program tddlog read opens in the playground, from the instance's
      // view and from tddlog's runs; a file the site holds back or keeps in
      // part says why it does not.
      if (tried) {
        await visit(hashOf(tried), "#programs table.given");
        const link = `../#bench=${[tried.suite, tried.key, tried.name].map(encodeURIComponent).join("/")}`;
        assert.equal(await page.getAttribute("#programs p.try a", "href"), link, `${tried.name}: the link to the playground`);
        await page.click(`#view button[data-runs="${tried.suite}/${tried.name}/tddlog"]`);
        await page.waitForSelector("#drawer:not([hidden]) h4");
        assert.equal(await page.getAttribute("#drawer p.try a", "href"), link, `${tried.name}: the link in tddlog's runs`);
        // A file it reads: shown in the view, the drawer closed.
        await page.click("#drawer .files button.file");
        await page.waitForSelector("#drawer", { state: "hidden" });
        assert.ok(await page.$("#programs button.file.on"), `${tried.name}: the file tddlog reads on show`);
      }
      if (kept) {
        await visit(hashOf(kept), "#programs table.given");
        const notes = await page.$$eval("#programs p.note", (ps) => ps.map((p) => p.textContent));
        assert.ok(notes.some((t) => t.startsWith("Not in the playground")), `${kept.name}: why the playground cannot open it`);
        assert.equal(await page.$("#programs p.try"), null);
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
        await gz.waitForSelector(`#programs button.file.on[data-file="${inline.ti}:${inline.fi}"]`);
        await gz.waitForSelector("#programs .pane pre.src");
        assert.equal(await gz.$eval("#programs .pane pre.src", (pre) => pre.textContent.split("\n")[0]), inline.text.split("\n")[0],
          `${inline.name}: ${inline.file}, inflated`);
        assert.equal(new URL(gz.url()).hash, at, `${inline.name}: the address of ${inline.file}`);
        const [download] = await Promise.all([gz.waitForEvent("download"), gz.click("#programs .pane button.save")]);
        assert.equal(download.suggestedFilename(), inline.file.split("/").pop());
        assert.equal(fs.readFileSync(await download.path(), "utf8"), inline.text, `${inline.name}: ${inline.file}, saved`);
        await gz.close();
      }

      // The search finds an instance by its name; the arrow keys and Enter
      // choose it.
      await visit("#/", ".kind-card");
      await page.fill("#search", family.name);
      await page.waitForSelector("#found-list [role=option]");
      assert.equal(await page.getAttribute("#search", "aria-expanded"), "true");
      const option = await page.$$eval("#found-list [role=option]", (os, [name, title]) =>
        os.findIndex((o) => o.firstChild.textContent === name && o.lastChild.textContent === title), [family.name, bySuite.get(family.suite).title]);
      assert.ok(option >= 0, `the search finds ${family.name}`);
      for (let i = 0; i <= option; i++) await page.keyboard.press("ArrowDown");
      await page.keyboard.press("Enter");
      await page.waitForSelector(`#view[data-route="${hash}"] #programs`);

      await page.goto(`${base}results/benchmarks/${family.suite}/${encodeURIComponent(family.name)}.html`);
      await page.waitForSelector(`#view[data-route="${hash}"] #programs table.given`);
      console.log(`ok the benchmarks' data: ${nf} families, ${nfiles} files; ${family.suite}/${family.name}'s view, runs and search`
        + `${unseen ? `; ${unseen.name}'s files not captured` : ""}${tried ? `; ${tried.name} to the playground` : ""}`
        + `${kept ? `; ${kept.name} held back` : ""}${inline ? `; ${inline.file} gzipped` : ""}`);
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
        await shot.goto(`${base}?example=asia&evidence=1`);
        await answered(shot, "asia(yes)");
        await noSidewaysScroll(shot, `${colorScheme}, ${width} px`);
        if (shots) await shot.screenshot({ path: path.join(shots, `page-${colorScheme}-${width}.png`), fullPage: true });
        await shot.goto(`${base}guide/`);
        await noSidewaysScroll(shot, `the guide, ${colorScheme}, ${width} px`);
        assert.equal(await shot.title(), "tddlog guide");
        if (shots) await shot.screenshot({ path: path.join(shots, `guide-${colorScheme}-${width}.png`), fullPage: true });
        }
        // The results: the overview, a kind, a kind of many systems, a suite,
        // and a family with a system's runs open beside it. Each system's
        // colour has a contrast of 3:1 or more with the page and the card; on
        // a phone a board's rows stack, each value in the window and named by
        // its column.
        const crowded = kinds.find((k) => k.systems.length > 5) ?? kinds[0];
        const views = [["results", "", ".kind-card"], ["results-kind", `#/kind/${kinds[0].key}`, "figure.plot svg.cactus path.line"],
          ["results-many", `#/kind/${crowded.key}`, "figure.plot svg.cactus path.line"],
          ["results-suite", `#/${first}`, "figure.plot svg.cactus path.line"]];
        if (family) views.push(["results-family", `#/${family.suite}/${encodeURIComponent(family.key)}/${encodeURIComponent(family.name)}`, "#programs table.given"]);
        for (const [name, hash, ready] of views) {
          await shot.goto(`${base}results/${hash}`);
          await shot.waitForSelector(`#view[data-route="${hash || "#/"}"] ${ready}`);
          if (name === "results") {
            const keys = [...new Set(Object.keys(names).map((key) => key.replace(/-process$/, "")))];
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
          if (name === "results-kind" && width < 600) {
            const cells = await shot.$$eval("#view table.board.stack tbody tr:first-child td", (tds) => tds.filter((td) => td.textContent.trim() !== "")
              .map((td) => [td.getBoundingClientRect().right, getComputedStyle(td, "::before").content]));
            assert.ok(cells.length >= 2, `a kind's board on a phone: ${cells.length} values`);
            for (const [right, label] of cells) {
              assert.ok(right <= width, `a kind's board on a phone: a value ends at ${right} px`);
              assert.match(label, /:/, "a kind's board on a phone: each value named");
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
        for (const [binding, title] of [["python", "tddlog for Python"], ["c", "tddlog for C"]]) {
          const response = await shot.goto(`${base}${binding}/`);
          assert.equal(response.status(), 200, `the ${binding} page`);
          assert.match(await shot.title(), new RegExp(`^${title}`));
          const bar = await shot.$$eval(".announcement a", (anchors) => anchors.map((a) => a.textContent));
          assert.deepEqual(bar, ["Playground", "Guide", "Results", "API", "Python", "C", "GitHub"].filter((n) => n !== title.split(" ").pop()),
            `the ${binding} page's links to the site`);
          const [barWidth, barRoom] = await shot.$eval(".announcement", (bar) => [bar.scrollWidth, bar.clientWidth]);
          assert.ok(barWidth <= barRoom, `the ${binding} page's links, ${barWidth} px in ${barRoom}, ${colorScheme}, ${width} px`);
          if (shots) await shot.screenshot({ path: path.join(shots, `${binding}-${colorScheme}-${width}.png`) });
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
