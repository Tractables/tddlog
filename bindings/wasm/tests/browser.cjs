// The assembled site in headless Chromium: every example's answers on the
// page against the ProbLog column beside them, P(evidence), an edited program
// through the address, an error, Cancel, the theme, the guide's links, the
// results page (every family's plot, a suite's, the hover layer, the table),
// the API, Python and C pages, and no sideways scrolling at a desktop's width
// or a phone's.
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
    }

    // The results page: a plot a family, a line a system; another suite's
    // plot on its button; the time under the pointer; the table sorts.
    await page.goto(`${base}results/`);
    await page.waitForSelector("section.family svg.cactus path.line");
    const families = await page.$$eval("section.family", (sections) => sections.filter((s) => s.id !== "overall").map((s) => [s.id, s.querySelectorAll("svg.cactus path.line").length]));
    assert.ok(families.length >= 2, `the results' families: ${JSON.stringify(families)}`);
    for (const [id, lines] of families) assert.ok(lines >= 2, `${id}: ${lines} lines`);
    // Every problem: one plot above the families over every instance of the
    // run, the instances of each family in the tab "All problems" first.
    // Both count what a system does not answer as unanswered, the legend
    // gives "N of M" per system, the plot's axis goes to all M, and the time
    // axis starts at 0.2 s.
    const legendTotal = async (selector) => {
      const items = await page.$$eval(`${selector} .legend .n`, (spans) => spans.map((s) => s.textContent.replace(/\s/g, "")));
      assert.ok(items.length >= 2, `${selector}: ${items.length} legend entries`);
      const totals = new Set(items.map((t) => t.match(/^(\d+)of(\d+)$/)?.[2]));
      assert.equal(totals.size, 1, `${selector}: one total in ${JSON.stringify(items)}`);
      return Number([...totals][0]);
    };
    const allText = async (selector) => (await page.textContent(`${selector} svg.cactus`)).replace(/\s+/g, " ");
    const overall = await legendTotal("#overall");
    assert.ok((await allText("#overall")).replace(/\s/g, "").includes(`all${overall}instances`), "the overall plot's axis");
    assert.match(await allText("#overall"), /≤0\.2/, "the time axis starts at 0.2 s");
    assert.ok((await page.$$("#overall ul.cannot li")).length >= 1, "the overall plot says who was not given what");
    let sum = 0;
    for (const [id] of families) {
      const first = await page.$(`#${id} .tabs button:first-child`);
      assert.equal(await first.getAttribute("data-suite"), "all", `${id}: the first tab`);
      assert.equal(await first.getAttribute("aria-pressed"), "true", `${id}: the first tab is shown`);
      assert.match(await first.textContent(), /^All problems \([\d ]+\)$/);
      const m = await legendTotal(`#${id}`);
      assert.equal(m, Number((await first.textContent()).match(/\(([\d ]+)\)/)[1].replace(/ /g, "")), `${id}: M of the legend and the tab`);
      const suites = await page.$$eval(`#${id} .tabs button:not(:first-child)`, (bs) => bs.map((b) => Number(b.textContent.match(/\(([\d ]+)\)/)[1].replace(/ /g, ""))));
      assert.equal(m, suites.reduce((a, b) => a + b, 0), `${id}: M is the instances of its suites`);
      sum += m;
    }
    assert.equal(overall, sum, "the overall plot has the instances of every family");
    console.log(`ok every problem: ${overall} instances in ${families.length} families, an All problems tab each`);

    const second = await page.$("section.family .tabs button:nth-child(2)");
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
