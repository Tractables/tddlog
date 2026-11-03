// The site's front page in headless Chromium: each of its views (the home
// page, by baseline and a baseline's page, by integration and each
// integration's page, by program type and a type's page with its
// head-to-head and translated parts, the coverage, how it works, its strip
// of stages a trace's steps, each a link to its step, and each step of its
// walkthroughs, each a trace drawn with a caption over every step, the step
// on show marked) draws without an error, its views.json written from the
// same results as categories.json, with a member of tddlog's (tddlog or a
// pipeline) for each row a system was given, named where a baseline's page
// compares with it; a link to a section of a baseline's page scrolls to it;
// a benchmark's page has its playground link; an address of nothing is not
// found; the addresses these pages once had (#/tools, #/system, next/) open
// the same views; every page's bar names the sections as the menu does, in
// its order; the site's menu beside every view, the view on show marked in
// it, the ten baselines that answer most and the rest behind a control that
// the viewer's choice keeps, and on a phone the bar and the menu behind one
// toggle; no sideways scrolling at a desktop's width or a phone's, in both
// themes, with the screenshots where asked for.
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
  // the walkthroughs: each one's key, playground example (none for a run the playground does not make) and steps
  const WALKS = [["reach", "reachability"], ["gears", "gears"], ["tpch", "tpch"], ["network", "car"], ["markov", null], ["dice", "walk"],
    ["circuit", "circuit"], ["roulette", "roulette"]].map(([key, example]) =>
    [key, example, tddlogTrace.steps(JSON.parse(fs.readFileSync(path.join(site, "next", "walk", `${key}.json`), "utf8")))]);
  const integrations = Object.keys(cats.integrations ?? {});
  const checks = [
    ["#/", "draft-home", async () => {
      assert.equal(await page.locator("table.home-t tbody tr").count(), cats.categories.length - 1, "a row a type of program");
      assert.equal(await page.locator("table.home-t td.best a.tool").count(),
        cats.categories.filter((c) => c.key !== "all-programs").reduce((n, c) => n + c.named.length, 0), "each type's best baselines, a link each");
      assert.equal(await page.locator("ul.home-int li").count(), integrations.filter((k) => cats.integrations[k].system).length, "a line a pipeline");
      assert.equal(await page.locator("#tree .tree li.cat > ul").count(), 0, "no type open in the menu");
    }],
    ["#/baselines", "baselines", async () => {
      assert.equal(await page.locator("table.baselines-t tbody tr").count(), systems.length);
    }],
    ...["dice", "alea", "problog", "ace", "roulette"].filter((x) => systems.includes(x)).map((x) => [`#/baselines/${x}`, "baseline", async () => {
      for (const id of ["head", "translated", "types", "coverage", "run"]) assert.equal(await page.locator(`section.panel#${id}`).count(), 1, `${x}: ${id}`);
      const v = views.systems[x];
      if (v.n.s + v.n.m + v.n.b) assert.equal(await page.locator("#translated .way").count(), v.ways.length, `${x}: ways`);
      if (v.n.h) {
        await page.waitForSelector("#head figure svg");
        const line = await page.locator("#head .versus").first().textContent();
        for (const m of v.head.me) assert.ok(line.includes(cats.names[m]), `${x}: head-to-head names ${m}`);
      }
      assert.equal(await page.locator(`.panels-nav a[href="#/baselines/${x}/run"]`).count(), 1, `${x}: a link to how it was run`);
      assert.equal(await page.getByText(/its time limit \(PAR-/).count(), 1, `${x}: how the totals are counted, once`);
    }]),
    ["#/integrations", "integrations", async () => assert.equal(await page.locator("table.board tbody tr").count(), integrations.length)],
    ...integrations.map((k) => [`#/integrations/${k}`, "integration", async () => {
      const i = cats.integrations[k];
      if (!i.system) assert.equal(await page.locator("table.board tbody tr").count(), i.types.length, `${k}: a row a type`);
      else {
        await page.waitForSelector("#view figure svg");
        const line = await page.locator("#view .versus").first().textContent();
        assert.ok(line.includes(cats.names[i.pipeline]) && line.includes(cats.names[i.system]), `${k}: ${i.pipeline} against ${i.system}`);
        assert.equal(await page.locator("table.board tbody tr").count(), i.groups.length, `${k}: a row a family`);
      }
    }]),
    ["#/types", "types", async () => assert.equal(await page.locator("table.types-t tbody tr").count(), cats.categories.length)],
    ...cats.categories.map((c) => [`#/category/${c.key}`, "category", async () => {
      assert.equal(await page.locator("section.split").count(), 2, `${c.key}: head-to-head and translated`);
      assert.equal(await page.locator(".note", { hasText: "were written from other results" }).count(), 0, `${c.key}: views.json is stale`);
      // a system's name in the table opens its page
      for (const x of c.native) {
        const to = await page.locator("table.board tbody th a", { hasText: cats.names[x] }).first().getAttribute("href");
        assert.equal(to, `#/baselines/${x}`, `${c.key}: ${x}'s name opens its page`);
      }
    }]),
    ["#/coverage", "coverage", async () => assert.ok(await page.locator(".census").count() >= views.census.sources.length)],
    // how it works: the first walkthrough, its strip of stages its steps, each a link to its step
    ["#/how", "how", async () => {
      await page.waitForSelector(".walk-trace p.trace-api");
      const [key, , steps] = WALKS[0];
      assert.deepEqual(await page.$$eval(".pipe ol.pipeline a", (as) => as.map((a) => [a.textContent, a.getAttribute("href")])),
        steps.map((t, k) => [`${k + 1}. ${t.title}`, `#/how/${key}/${k + 1}`]), "the strip is the first walkthrough's steps");
      assert.equal(await page.locator("nav.walks a").count(), WALKS.length);
      assert.equal(await page.textContent("ol.pipeline a[aria-current]"), `1. ${steps[0].title}`);
    }],
    // each step of each walkthrough: its trace drawn, under its caption
    ...WALKS.flatMap(([key, example, steps]) => steps.map((step, i) => [`#/how/${key}/${i + 1}`, "how", async () => {
      await page.waitForSelector(".walk-trace p.trace-api");
      assert.match(await page.textContent(".step h3"), new RegExp(`^${i + 1}\\. ${step.title}`));
      assert.deepEqual(await page.$$eval("ol.pipeline a", (as) => as.map((a) => a.textContent)), steps.map((t, k) => `${k + 1}. ${t.title}`), `${key} ${i + 1}: the steps`);
      assert.equal(await page.textContent("ol.pipeline a[aria-current]"), `${i + 1}. ${step.title}`, `${key} ${i + 1}: the step marked`);
      assert.equal(await page.locator(".step > p").count(), 1, `${key} ${i + 1}: its caption`);
      assert.equal(await page.locator(example ? `a.try-btn[href^="playground/?example=${example}&"]` : "a.try-btn").count(), example ? 1 : 0);
      if (step.key === "layout") await page.waitForSelector(".walk-trace svg.trace-vtree");
    }])),
    ["#/how/reach/99", "how", async () => {
      await page.waitForSelector(".walk-trace p.trace-api");
      assert.match(await page.textContent(".step h3"), /^8\. Answers/, "past the last step, the last");
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
  // the menu: the other systems by their answers, the first ten shown, a baseline's page each
  const baselines = cats.systems.filter((s) => s.key !== "tddlog" && !s.key.endsWith("-tddlog")).sort((a, b) => b.answers - a.answers)
    .map((s) => `#/baselines/${s.key}`);
  const sections = await (async () => {
    await page.goto(base);
    await drawn(page);
    return page.evaluate(() => SECTIONS);
  })();
  for (const [hash, view, check] of checks) {
    await page.goto(base + hash);
    await drawn(page);
    await check();
    assert.deepEqual(await page.$$eval("#tree ul.baselines li.solver a", (as) => as.map((a) => a.getAttribute("href"))), baselines,
      `${hash}: the baselines in the menu`);
    // by integration where the data has the integrations, and the bar's link to it with it
    assert.deepEqual(await page.$$eval("#tree .tree > li.group > a", (as) => as.map((a) => a.textContent)),
      sections.filter(([key]) => key !== "integrations" || cats.integrations).map(([, name]) => name), `${hash}: the menu's sections`);
    assert.equal(await page.isVisible('.site-nav a[href="#/integrations"]'), Boolean(cats.integrations),
      `${hash}: the bar's link to the integrations`);
    const marked = await page.$$eval("#tree a[aria-current]", (as) => as.map((a) => a.getAttribute("href")));
    // nothing on the home page, an address of nothing, and the integrations' page where the menu has no such section
    const none = view === "missing" || view === "draft-home" || (view === "integrations" && !cats.integrations);
    assert.equal(marked.length, none ? 0 : 1, `${hash}: the menu marks ${marked.join(", ") || "nothing"}`);
    console.log(`ok ${hash} (${view})${marked.length ? `, marked ${marked[0]}` : ""}`);
  }
  for (const x of baselines) assert.ok(systems.includes(x.slice("#/baselines/".length)), `${x}: a baseline's page`);

  // the menu's first ten, the rest behind the control, which the next view keeps
  await page.goto(`${base}#/types`);
  await drawn(page);
  await page.evaluate(() => localStorage.removeItem("tddlog-baselines"));
  await page.reload();
  await drawn(page);
  const visible = () => page.$$eval("#tree ul.baselines li.solver", (lis) => lis.filter((li) => li.offsetParent !== null).length);
  assert.equal(await visible(), Math.min(10, baselines.length), "the menu shows ten baselines");
  if (baselines.length > 10) {
    await page.click("#tree li.toggle button");
    assert.equal(await visible(), baselines.length, "the control shows the rest");
    await page.goto(`${base}#/how`);
    await drawn(page);
    assert.equal(await visible(), baselines.length, "the next view keeps the choice");
    await page.click("#tree li.toggle button");
    assert.equal(await visible(), 10, "and the control folds them again");
  }
  console.log("ok the menu's baselines: ten, then the rest behind the control");

  // a link to a section of a baseline's page scrolls to it
  const x = systems.includes("dice") ? "dice" : systems[0];
  await page.goto(`${base}#/baselines/${x}`);
  await drawn(page);
  await page.click(`.panels-nav a[href="#/baselines/${x}/run"]`);
  await page.waitForFunction(() => document.getElementById("run").classList.contains("reached"));
  // in view: at the top, or as near it as the page's end lets it come
  assert.ok(await page.evaluate(() => {
    const top = document.getElementById("run").getBoundingClientRect().top;
    return top > -2 && (top < 40 || Math.ceil(window.scrollY + window.innerHeight) >= document.documentElement.scrollHeight - 2);
  }), "the section scrolled to");
  console.log(`ok #/baselines/${x}/run scrolls to its section`);

  // the addresses these pages once had
  for (const [from, to] of [["#/tools", "#/baselines"], [`#/tools/${x}`, `#/baselines/${x}`], [`#/system/${x}`, `#/baselines/${x}`],
    ...integrations.map((k) => [`#/system/${cats.integrations[k].pipeline ?? "tddlog"}`, `#/integrations/${k}`])]) {
    await page.goto(base + from);
    await page.waitForURL(base + to);
    await drawn(page);
    console.log(`ok ${from} opens ${to}`);
  }
  await page.goto(`${base}results/#/system/${x}`);
  await page.waitForURL(`${base}#/baselines/${x}`);
  await drawn(page);
  console.log(`ok results/#/system/${x} opens #/baselines/${x}`);
  await page.goto(`${base}next/#/tools`);
  await page.waitForURL(`${base}#/baselines`);
  await drawn(page);
  console.log("ok next/#/tools opens #/baselines");

  // every page's bar: the sections as the menu names them, in its order, then the playground, the docs and GitHub
  for (const at of ["", "results/", "playground/", "guide/", "404.html"]) {
    // 404.html reads its styles from the site's address (its base), which this server does not have: a page whose errors are not counted
    const p = at === "404.html" ? await browser.newPage() : page;
    await p.goto(base + at);
    assert.deepEqual(await p.$$eval(".site-nav > a, .site-nav > details > summary", (as) => as.map((a) => a.textContent.trim())),
      [...sections.map(([, name]) => name), "Playground", "Docs", "GitHub"], `${at || "the front page"}: the bar`);
    if (p !== page) await p.close();
  }
  console.log("ok every page's bar names the sections in the menu's order");

  await page.close();

  // both themes, at a desktop's width and a phone's, a page each
  const wide = ["#/", "#/baselines", `#/baselines/${x}`, "#/integrations", ...integrations.map((k) => `#/integrations/${k}`), "#/types",
    "#/category/functional-programs", "#/coverage", "#/how", "#/how/reach/5", "#/how/circuit/6"];
  for (const colorScheme of ["light", "dark"]) {
    for (const width of [1280, 390]) {
      for (const hash of wide) {
        const shot = await browser.newPage({ viewport: { width, height: 900 }, colorScheme });
        shot.on("pageerror", (e) => errors.push(`${shot.url()}: ${e.message}`));
        await shot.goto(base + hash);
        await drawn(shot);
        await shot.waitForTimeout(300);
        await noSidewaysScroll(shot, `${hash} at ${width} px (${colorScheme})`);
        // on a phone the bar and the menu are behind one toggle
        if (width < 600 && hash === "#/") {
          assert.equal(await shot.isVisible("#tree"), false, "the menu folded on a phone");
          assert.equal(await shot.isVisible(".site-nav"), false, "the bar folded on a phone");
          await shot.click("#menu");
          assert.equal(await shot.isVisible("#tree a[href='#/baselines']"), true, "the menu open on a phone");
          assert.equal(await shot.isVisible(".site-nav a[href='playground/']"), true, "the bar open on a phone");
          assert.equal(await shot.isVisible(".site-nav a[href='#/baselines']"), false, "its sections in the menu's headings alone");
          if (shots) await shot.screenshot({ path: path.join(shots, `next-menu-${colorScheme}-${width}.png`), fullPage: true });
          await shot.click("#menu");
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
