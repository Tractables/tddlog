// The assembled site in headless Chromium: every example's answers on the
// page against the ProbLog column beside them, P(evidence), an edited program
// through the address, an error, Cancel, the theme, the guide's links, the
// pages' links within the site, the
// results page (each kind of problems' plot, drawing the kind's systems over
// all of its instances with their answers and total times, and every
// suite's; the hover layer, the table), the API, Python and C pages, and no
// sideways scrolling at a desktop's width or a phone's.
//
//   node tests/browser.cjs <site> [<directory for screenshots>]
//
// RESULTS_ONLY=1 tests the results page alone (a site of that page, its
// styles and theme.js).
//
// with Playwright and its Chromium installed (README.md).
const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { chromium } = require("playwright");
const { tddlogExamples } = require("../examples.js");

const site = path.resolve(process.argv[2]);
const only = process.env.RESULTS_ONLY === "1";     // the results page alone, from a site that has only it
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css",
  ".wasm": "application/wasm", ".pl": "text/plain", ".json": "application/json",
};

// The site as GitHub Pages serves it: a directory's index.html at its path.
function serve(root) {
  const server = http.createServer((request, response) => {
    let file = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    if (file.endsWith("/")) file += "index.html";
    const full = path.join(root, file);
    if (!full.startsWith(root) || !fs.existsSync(full) || fs.statSync(full).isDirectory()) {
      response.writeHead(404);
      response.end();
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
      if (at === "results/") await page.waitForSelector("section.family svg.cactus path.line");
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

    // The results page: a plot a family, a line a system; another suite's
    // plot on its button; the time under the pointer; the table sorts.
    await page.goto(`${base}results/`);
    await page.waitForSelector("section.family svg.cactus path.line");
    const families = await page.$$eval("section.family", (sections) => sections.map((s) => [s.id, s.querySelectorAll("svg.cactus path.line").length]));
    assert.ok(families.length >= 2, `the results' families: ${JSON.stringify(families)}`);
    for (const [id, lines] of families) assert.ok(lines >= 2, `${id}: ${lines} lines`);
    // A family's tabs: its kinds of problems (the data's kinds whose suites
    // are the family's, in the README's order), the first shown, then its
    // suites. A kind's plot draws the kind's systems over all of its
    // instances, tddlog first and then by the instances they answer; its
    // legend gives "answers k of N, total T" for each, k the system's
    // answers in the kind's suites and T its total time over the instances
    // some system of the kind answers, a run that does not answer counted at
    // its suite's limit and a time under 0.2 s at 0.2 s, as the README's
    // legends write it; its axis goes to all N, from 0.2 s; under it, one
    // line for each system that was not given some of the instances. A
    // suite's plot draws the suite's systems, its legend "answers k of N".
    const data = ["suites.json", "ppl.json"].map((file) => path.join(site, "results", file))
      .filter((file) => fs.existsSync(file)).map((file) => JSON.parse(fs.readFileSync(file, "utf8")));
    const kinds = data.flatMap((d) => d.kinds).sort((a, b) => a.order - b.order);
    const bySuite = new Map(data.flatMap((d) => d.families.flatMap((f) => f.suites)).map((s) => [s.key, s]));
    const number = (v) => typeof v === "number";
    const duration = (t) => (t >= 36000 ? `${(t / 3600).toFixed(0)} h` : t >= 3600 ? `${(t / 3600).toFixed(1)} h`
      : t >= 60 ? `${(t / 60).toFixed(0)} min` : t >= 10 ? `${t.toFixed(0)} s` : `${t.toFixed(1)} s`);
    const shown = (selector) => page.$eval(`${selector} figure.plot`, (f) => ({
      systems: f.dataset.systems.split(" ").filter(Boolean),
      instances: Number(f.dataset.instances),
      lines: [...f.querySelectorAll("svg.cactus path.line")].map((p) => p.dataset.key),
      legend: [...f.querySelectorAll(".legend .n")].map((s) => s.textContent.replace(/\s/g, "")),
      axis: f.querySelector("svg.cactus").textContent.replace(/\s/g, ""),
    }));
    let drawn = 0;
    for (const f of data.flatMap((d) => d.families)) {
      const mine = kinds.filter((k) => k.suites.every((key) => f.suites.some((s) => s.key === key)));
      const tabs = await page.$$eval(`#${f.key} .tabs button`, (bs) => bs.map((b) => [b.dataset.kind ? `kind ${b.dataset.kind}` : `suite ${b.dataset.suite}`, b.getAttribute("aria-pressed")]));
      assert.deepEqual(tabs.map(([t]) => t), [...mine.map((k) => `kind ${k.key}`), ...f.suites.map((s) => `suite ${s.key}`)], `${f.key}: its tabs`);
      assert.equal(tabs[0][1], "true", `${f.key}: its first tab is shown`);
      for (const k of mine) {
        const what = `${f.key}, the kind ${k.key}`;
        await page.click(`#${f.key} .tabs button[data-kind="${k.key}"]`);
        const rows = k.suites.flatMap((key) => {
          const s = bySuite.get(key);
          return s.instances.map((row) => ({ limit: s.limit, v: Object.fromEntries(s.systems.map((x, i) => [x, row[i + 1]])) }));
        });
        const answers = (x) => rows.filter((r) => number(r.v[x])).length;
        const scored = rows.filter((r) => k.systems.some((x) => number(r.v[x])));
        const total = (x) => scored.reduce((t, r) => t + (number(r.v[x]) ? Math.max(r.v[x], 0.2) : r.limit), 0);
        assert.equal(k.systems[0], "tddlog", `${what}: tddlog first in the data`);
        const order = ["tddlog", ...k.systems.slice(1).sort((a, b) => answers(b) - answers(a))];
        for (const x of order) {
          assert.equal(k.answers[x], answers(x), `${what}: ${x}'s answers in the data`);
          assert.ok(Math.abs(k.total[x] - total(x)) < 0.01, `${what}: ${x}'s total time in the data, ${k.total[x]} against ${total(x)}`);
        }
        assert.equal(k.instances, rows.length, `${what}: its instances in the data`);
        assert.equal(k.scored, scored.length, `${what}: the instances some system answers`);
        const fig = await shown(`#${f.key}`);
        assert.equal(fig.instances, rows.length, `${what}: its instances`);
        assert.deepEqual(fig.systems, order, `${what}: its systems, tddlog first and then by their answers`);
        assert.deepEqual([...fig.lines].sort(), [...order].sort(), `${what}: its lines`);
        assert.deepEqual(fig.legend, order.map((x) => `answers${answers(x)}of${rows.length},total${duration(k.total[x]).replace(/\s/g, "")}`), `${what}: its legend`);
        assert.ok(fig.axis.includes(`all${rows.length}instances`), `${what}: its axis`);
        assert.match(fig.axis, /≤0\.2/, `${what}: the time axis starts at 0.2 s`);
        const missing = order.filter((x) => rows.some((r) => r.v[x] === undefined || r.v[x] === null)).length;
        assert.equal((await page.$$(`#${f.key} ul.cannot li`)).length, missing, `${what}: the systems not given some of its instances`);
        drawn++;
      }
      for (const s of f.suites) {
        const what = `${f.key}, the suite ${s.key}`;
        await page.click(`#${f.key} .tabs button[data-suite="${s.key}"]`);
        const fig = await shown(`#${f.key}`);
        assert.equal(fig.instances, s.instances.length, `${what}: its instances`);
        assert.deepEqual(fig.systems, s.systems, `${what}: its systems`);
        assert.deepEqual([...fig.lines].sort(), [...s.systems].sort(), `${what}: its lines`);
        // An answer counts within the suite's limit, as the plot draws it.
        const answers = s.systems.map((x, i) => s.instances.filter((row) => number(row[i + 1]) && row[i + 1] <= s.limit * 1.001).length);
        assert.deepEqual(fig.legend, answers.map((a) => `answers${a}of${s.instances.length}`), `${what}: its legend`);
      }
    }
    assert.equal(drawn, kinds.length, "every kind is a tab of its family");
    assert.ok(drawn >= 2, `${drawn} kinds`);
    console.log(`ok the kinds ${kinds.map((k) => `${k.key} ${k.instances}`).join(", ")}: every system of a kind over all of its instances, its answers and total time`);

    const second = await page.$("section.family .tabs button[data-suite]");
    const key = await second.getAttribute("data-suite");
    await second.click();
    assert.equal(await second.getAttribute("aria-pressed"), "true");
    const area = await page.$("section.family svg.cactus rect.hit");
    await area.scrollIntoViewIfNeeded();
    const hit = await area.boundingBox();
    await page.mouse.move(hit.x + hit.width / 2, hit.y + hit.height / 2);
    assert.match(await page.textContent("section.family .tip"), /^within [0-9.]+ s/);
    await page.click("section.family details.instances summary");
    await page.click('section.family table.times th[data-col="1"]');
    assert.equal(await page.getAttribute('section.family table.times th[data-col="1"]', "aria-sort"), "ascending");
    console.log(`ok the results page: ${families.map(([id, n]) => `${id} ${n} lines`).join(", ")}; ${key}`);
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
        await shot.goto(`${base}results/`);
        await shot.waitForSelector("section.family svg.cactus path.line");
        await noSidewaysScroll(shot, `the results, ${colorScheme}, ${width} px`);
        assert.equal(await shot.title(), "tddlog results");
        if (shots) await shot.screenshot({ path: path.join(shots, `results-${colorScheme}-${width}.png`), fullPage: true });
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
