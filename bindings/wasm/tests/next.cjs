// The site's front page in headless Chromium: each of its views (the home
// page, by tool and a tool's page, by type and a type's page with its
// head-to-head and translated parts, the coverage, how it works and each
// step of its four walkthroughs, each a trace drawn with a caption over every
// step) draws without an error, its views.json written
// from the same results as categories.json, with a member of tddlog's
// (tddlog or a pipeline) for each row a system was given, named where a
// tool's page compares with it; the tools' numbers each in a cell of its
// own; a benchmark's page has its playground link; an address of nothing
// is not found; a link to the draft it was at next/ opens the same view;
// the site's menu beside every view, the view on show marked in it, every
// other system in it by its answers, and on a phone behind its toggle; no
// sideways scrolling at a desktop's width or a phone's, in both themes,
// with the screenshots where asked for.
//
//   node tests/next.cjs <site> [<directory for screenshots>]
//
// with Playwright and its Chromium installed (README.md).
const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { chromium } = require("playwright");
const tddlogTrace = require("../trace.js");

const site = path.resolve(process.argv[2]);
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json",
  ".svg": "image/svg+xml", ".txt": "text/plain", ".pl": "text/plain", ".gz": "application/gzip",
};

// The site as GitHub Pages serves it (browser.cjs's serve).
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

async function noSidewaysScroll(page, what) {
  const [scroll, width] = await page.evaluate(() => [document.documentElement.scrollWidth, window.innerWidth]);
  assert.ok(scroll <= width, `${what}: the page is ${scroll} px wide in a window of ${width}`);
}

// Until the view on show has drawn: the address's view, not the note that it loads.
function drawn(page) {
  return page.waitForFunction(() => {
    const view = document.getElementById("view");
    return view.dataset.route !== undefined && !view.textContent.startsWith("Loading");
  }, null, { timeout: 30000 });
}

(async () => {
  const server = await serve(site);
  const base = `http://127.0.0.1:${server.address().port}/`;
  const views = JSON.parse(fs.readFileSync(path.join(site, "next", "views.json"), "utf8"));
  const cats = JSON.parse(fs.readFileSync(path.join(site, "results", "categories.json"), "utf8"));
  const browser = await chromium.launch();
  const errors = [];
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => errors.push(`${page.url()}: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && errors.push(`${page.url()}: ${m.text()}`));

  // the views, each with what it must show
  const systems = Object.keys(views.systems);
  assert.ok(systems.length > 10, "views.json has the systems");
  const fun = cats.categories.find((c) => c.key === "functional-programs");
  assert.equal(views.mine[0], "tddlog");
  for (const c of cats.categories) {
    assert.equal(views.categories[c.key].with.length, c.rows.length, `${c.key}: a member of mine per row`);
    for (const [x, v] of Object.entries(views.systems)) {
      if (v.kinds[c.key] !== undefined) assert.equal(v.with[c.key].length, c.rows.length, `${x} in ${c.key}: a member per row`);
    }
  }
  // the walkthroughs: each one's key, playground example and steps
  const WALKS = [["reach", "reachability"], ["network", "asia"], ["dice", "walk"], ["circuit", "circuit"]].map(([key, example]) =>
    [key, example, tddlogTrace.steps(JSON.parse(fs.readFileSync(path.join(site, "next", "walk", `${key}.json`), "utf8")))]);
  const checks = [
    ["#/", "draft-home", async () => {
      assert.equal(await page.locator(".entries a.entry").count(), 3);
      await page.waitForSelector("#view figure.overview svg");
    }],
    ["#/tools", "tools", async () => {
      assert.equal(await page.locator(".tool-card").count(), systems.length);
      assert.equal(await page.locator(".tool-card table.mini td.num").count(), 4 * systems.length, "two numbers a comparison, each in its cell");
      const text = await page.locator(".tool-grid").textContent();
      assert.doesNotMatch(text, /\d, \d/, "no numbers run together");
    }],
    ...["dice", "alea", "problog", "ace", "roulette"].filter((x) => systems.includes(x)).map((x) => [`#/tools/${x}`, "tool", async () => {
      for (const id of ["head", "translated", "coverage", "run"]) assert.equal(await page.locator(`section.panel#${id}`).count(), 1, `${x}: ${id}`);
      const v = views.systems[x];
      if (v.n.s + v.n.m + v.n.b) assert.equal(await page.locator("#translated .way").count(), v.ways.length, `${x}: ways`);
      if (v.n.h) {
        await page.waitForSelector("#head figure svg");
        const line = await page.locator("#head .cells").first().textContent();
        for (const m of v.head.me) assert.ok(line.includes(cats.names[m]), `${x}: head-to-head names ${m}`);
      }
    }]),
    ["#/types", "types", async () => assert.equal(await page.locator("table.board tbody tr").count(), cats.categories.length)],
    ...cats.categories.map((c) => [`#/category/${c.key}`, "category", async () => {
      assert.equal(await page.locator("section.split").count(), 2, `${c.key}: head-to-head and translated`);
      assert.equal(await page.locator(".note", { hasText: "were written from other results" }).count(), 0, `${c.key}: views.json is stale`);
    }]),
    ["#/coverage", "coverage", async () => assert.ok(await page.locator(".census").count() >= views.census.sources.length)],
    ["#/how", "how", async () => {
      await page.waitForSelector(".pipe svg.pipeline");
      assert.equal(await page.locator(".entries.walks a.entry").count(), WALKS.length);
      assert.equal(await page.locator('a[href="playground/?trace=1"]').count(), 1);
    }],
    // each step of each walkthrough: its trace drawn, under its caption
    ...WALKS.flatMap(([key, example, steps]) => steps.map((step, i) => [`#/how/${key}/${i + 1}`, "how", async () => {
      await page.waitForSelector(".walk-trace p.trace-api");
      assert.equal(await page.textContent(".step h4"), `${i + 1}. ${step.title}`);
      assert.equal(await page.locator(".step > p").count(), 1, `${key} ${i + 1}: its caption`);
      assert.equal(await page.locator(`a.try-btn[href^="playground/?example=${example}&"]`).count(), 1);
      if (step.key === "layout") await page.waitForSelector(".walk-trace svg.trace-vtree");
    }])),
    ["#/how/reach/99", "how", async () => {
      await page.waitForSelector(".walk-trace p.trace-api");
      assert.match(await page.textContent(".step h4"), /Answers$/, "past the last step, the last");
    }],
    ["#/how/nothing", "missing", async () => assert.equal(await page.locator("#view h2", { hasText: "Not found" }).count(), 1)],
  ];
  // a benchmark's page, where the site has the benchmarks' data: its programs and the playground's link
  const benchIndex = path.join(site, "results", "benchmarks", "index.json");
  if (fs.existsSync(benchIndex)) {
    const row = fun.rows[0];
    checks.push([`#/${row[0]}/@${encodeURIComponent(row[1])}`, "family", async () => {
      await page.waitForSelector("#programs");
    }]);
  }
  // the menu: every other system by its answers, a tool's page each
  const tools = cats.systems.filter((s) => s.key !== "tddlog" && !s.key.endsWith("-tddlog")).sort((a, b) => b.answers - a.answers)
    .map((s) => `#/tools/${s.key}`);
  for (const [hash, view, check] of checks) {
    await page.goto(base + hash);
    await drawn(page);
    await check();
    assert.deepEqual(await page.$$eval("#tree .tree > li.solver > a", (as) => as.map((a) => a.getAttribute("href"))), tools, `${hash}: the tools in the menu`);
    const marked = await page.$$eval("#tree a[aria-current]", (as) => as.map((a) => a.getAttribute("href")));
    assert.equal(marked.length, view === "missing" ? 0 : 1, `${hash}: the menu marks ${marked.join(", ") || "nothing"}`);
    console.log(`ok ${hash} (${view})${marked.length ? `, marked ${marked[0]}` : ""}`);
  }
  for (const x of tools) assert.ok(systems.includes(x.slice("#/tools/".length)), `${x}: a tool's page`);

  // the draft's former address
  await page.goto(`${base}next/#/tools`);
  await page.waitForURL(`${base}#/tools`);
  await drawn(page);
  console.log("ok next/#/tools opens #/tools");

  await page.close();

  // both themes, at a desktop's width and a phone's, a page each
  const wide = ["#/", "#/tools", `#/tools/${systems.includes("dice") ? "dice" : systems[0]}`, "#/types", "#/category/functional-programs", "#/coverage", "#/how", "#/how/reach/5", "#/how/circuit/6"];
  for (const colorScheme of ["light", "dark"]) {
    for (const width of [1280, 390]) {
      for (const hash of wide) {
        const shot = await browser.newPage({ viewport: { width, height: 900 }, colorScheme });
        shot.on("pageerror", (e) => errors.push(`${shot.url()}: ${e.message}`));
        await shot.goto(base + hash);
        await drawn(shot);
        await shot.waitForTimeout(300);
        await noSidewaysScroll(shot, `${hash} at ${width} px (${colorScheme})`);
        // on a phone the menu is behind its toggle
        if (width < 600 && hash === "#/") {
          assert.equal(await shot.isVisible("#tree"), false, "the menu folded on a phone");
          await shot.click("#browse");
          assert.equal(await shot.isVisible("#tree a[href='#/tools']"), true, "the menu open on a phone");
          if (shots) await shot.screenshot({ path: path.join(shots, `next-menu-${colorScheme}-${width}.png`), fullPage: true });
          await shot.click("#browse");
        }
        const name = hash.replace(/^#\/?/, "").replace(/\//g, "-") || "home";
        if (shots) await shot.screenshot({ path: path.join(shots, `next-${name}-${colorScheme}-${width}.png`), fullPage: true });
        await shot.close();
      }
    }
  }
  assert.deepEqual(errors, [], "no errors on the pages");
  console.log(`ok both themes at 1280 and 390 px${shots ? `, screenshots in ${shots}` : ""}`);
  await browser.close();
  server.close();
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
