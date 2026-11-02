// The site's draft (next/) in headless Chromium: each of its views (the
// home page, by tool and a tool's page, by type and a type's page with its
// head-to-head and translated parts, the coverage, how it works and each
// step of its walkthrough) draws without an error, its views.json written
// from the same results as categories.json; a benchmark's page has its
// playground link; an address of nothing is not found; no sideways
// scrolling at a desktop's width or a phone's, in both themes, with the
// screenshots where asked for.
//
//   node tests/next.cjs <site> [<directory for screenshots>]
//
// with Playwright and its Chromium installed (README.md).
const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { chromium } = require("playwright");

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
  const base = `http://127.0.0.1:${server.address().port}/next/`;
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
  const checks = [
    ["#/", "draft-home", async () => {
      assert.equal(await page.locator(".entries a.entry").count(), 3);
      await page.waitForSelector("#view figure.overview svg");
    }],
    ["#/tools", "tools", async () => assert.equal(await page.locator(".tool-card").count(), systems.length)],
    ...["dice", "alea", "problog", "ace", "roulette"].filter((x) => systems.includes(x)).map((x) => [`#/tools/${x}`, "tool", async () => {
      for (const id of ["head", "translated", "coverage", "run"]) assert.equal(await page.locator(`section.panel#${id}`).count(), 1, `${x}: ${id}`);
      const v = views.systems[x];
      if (v.n.s + v.n.m + v.n.b) assert.equal(await page.locator("#translated .way").count(), v.ways.length, `${x}: ways`);
      if (v.n.h) await page.waitForSelector("#head figure svg");
    }]),
    ["#/types", "types", async () => assert.equal(await page.locator("table.board tbody tr").count(), cats.categories.length)],
    ...cats.categories.map((c) => [`#/category/${c.key}`, "category", async () => {
      assert.equal(await page.locator("section.split").count(), 2, `${c.key}: head-to-head and translated`);
      assert.equal(await page.locator(".note", { hasText: "were written from other results" }).count(), 0, `${c.key}: views.json is stale`);
    }]),
    ["#/coverage", "coverage", async () => assert.ok(await page.locator(".census").count() >= views.census.sources.length)],
    ["#/how", "how", async () => {
      await page.waitForSelector(".pipe svg.pipeline");
      await page.waitForFunction(() => document.querySelector(".walk-code pre.src")?.textContent.includes("path(X,Y)"));
      assert.equal(await page.locator('a[href="../?example=reachability"]').count(), 1);
    }],
    ...[2, 3, 4, 5, 6].map((n) => [`#/how/${n}`, "how", async () => {
      await page.waitForFunction(() => document.querySelector(".walk-code pre.src") !== null);
      const text = await page.locator(".walk-code").textContent();
      assert.ok(!text.includes(": 404)"), `step ${n}: a walk file is missing`);
    }]),
    ["#/how/9", "missing", async () => assert.equal(await page.locator("#view h2", { hasText: "Not found" }).count(), 1)],
  ];
  // a benchmark's page, where the site has the benchmarks' data: its programs and the playground's link
  const benchIndex = path.join(site, "results", "benchmarks", "index.json");
  if (fs.existsSync(benchIndex)) {
    const row = fun.rows[0];
    checks.push([`#/${row[0]}/@${encodeURIComponent(row[1])}`, "family", async () => {
      await page.waitForSelector("#programs");
    }]);
  }
  for (const [hash, view, check] of checks) {
    await page.goto(base + hash);
    await drawn(page);
    await check();
    const nav = await page.locator(".site-nav a[aria-current]").count();
    assert.equal(nav, 1, `${hash}: one section of the header marked`);
    console.log(`ok ${hash} (${view})`);
  }

  await page.close();

  // both themes, at a desktop's width and a phone's, a page each
  const wide = ["#/", "#/tools", `#/tools/${systems.includes("dice") ? "dice" : systems[0]}`, "#/types", "#/category/functional-programs", "#/coverage", "#/how", "#/how/3"];
  for (const colorScheme of ["light", "dark"]) {
    for (const width of [1280, 390]) {
      for (const hash of wide) {
        const shot = await browser.newPage({ viewport: { width, height: 900 }, colorScheme });
        shot.on("pageerror", (e) => errors.push(`${shot.url()}: ${e.message}`));
        await shot.goto(base + hash);
        await drawn(shot);
        await shot.waitForTimeout(300);
        await noSidewaysScroll(shot, `${hash} at ${width} px (${colorScheme})`);
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
