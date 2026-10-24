// The assembled site in headless Chromium: every example's answers on the
// page against the ProbLog column beside them, P(evidence), an edited program
// through the address, an error, Cancel, the theme, the guide's links, the
// pages' links within the site, the
// results page (the overview; each kind of problems' plot, drawing the
// kind's systems over all of its instances with their answers and total
// times; each suite's plot and tddlog against each system; the hover layer,
// the table; where the site has the benchmarks' data, a family's times,
// programs and runs, and a link to a benchmark's former page), the API,
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
const { chromium } = require("playwright");
const { tddlogExamples } = require("../examples.js");

const site = path.resolve(process.argv[2]);
const only = process.env.RESULTS_ONLY === "1";     // the results page alone, from a site that has only it
const shots = process.argv[3] ? path.resolve(process.argv[3]) : null;
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".css": "text/css",
  ".wasm": "application/wasm", ".pl": "text/plain", ".json": "application/json",
  ".svg": "image/svg+xml",
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
    for (const at of ["", "guide/", "results/"]) {
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
      const what = at || "the playground";
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
    // instances. A suite's view (#/SUITE) plots the suite's systems, its
    // legend "answers k of N", and tddlog against each system given some of
    // the same instances.
    const data = ["suites.json", "ppl.json"].map((file) => path.join(site, "results", file))
      .filter((file) => fs.existsSync(file)).map((file) => JSON.parse(fs.readFileSync(file, "utf8")));
    const kinds = data.flatMap((d) => d.kinds).sort((a, b) => a.order - b.order);
    const bySuite = new Map(data.flatMap((d) => d.families.flatMap((f) => f.suites)).map((s) => [s.key, s]));
    const number = (v) => typeof v === "number";
    const duration = (t) => (t >= 36000 ? `${(t / 3600).toFixed(0)} h` : t >= 3600 ? `${(t / 3600).toFixed(1)} h`
      : t >= 60 ? `${(t / 60).toFixed(0)} min` : t >= 10 ? `${t.toFixed(0)} s` : `${t.toFixed(1)} s`);
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
      const order = ["tddlog", ...k.systems.slice(1).sort((a, b) => answers(b) - answers(a))];
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
      assert.deepEqual(fig.legend, order.map((x) => `answers${answers(x)}of${rows.length},total${duration(k.total[x]).replace(/\s/g, "")}`), `${what}: its legend`);
      assert.ok(fig.axis.includes(`all${rows.length}instances`), `${what}: its axis`);
      assert.match(fig.axis, /≤0\.2/, `${what}: the time axis starts at 0.2 s`);
      const missing = order.filter((x) => rows.some((r) => r.v[x] === undefined || r.v[x] === null)).length;
      assert.equal((await page.$$("#view ul.cannot li")).length, missing, `${what}: the systems not given some of its instances`);
      const board = await page.$eval("#view table.board", (t) => [...t.tBodies[0].rows].map((r) => r.cells[1].textContent.trim()));
      assert.deepEqual(board.map(Number).sort((a, b) => b - a), order.map(answers).sort((a, b) => b - a), `${what}: its systems' answers`);
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
      const answers = s.systems.map((x, i) => s.instances.filter((row) => number(row[i + 1]) && row[i + 1] <= s.limit * 1.001).length);
      assert.deepEqual(fig.legend, answers.map((a) => `answers${a}of${s.instances.length}`), `${what}: its legend`);
      const me = s.systems.indexOf("tddlog") + 1;
      const others = me === 0 ? [] : s.systems.filter((x, i) => x !== "tddlog"
        && s.instances.some((row) => row[me] !== null && row[i + 1] !== null));
      assert.equal((await page.$$("#view table.h2h tbody tr:not(.words)")).length, others.length, `${what}: tddlog against each system`);
    }
    console.log(`ok ${bySuite.size} suites: each one's systems, and tddlog against each`);

    // A suite's plot: the time under the pointer; its table sorts by a column.
    const [first] = bySuite.keys();
    await visit(`#/${first}`, "figure.plot svg.cactus rect.hit");
    const area = await page.$("#view svg.cactus rect.hit");
    await area.scrollIntoViewIfNeeded();
    const hit = await area.boundingBox();
    await page.mouse.move(hit.x + hit.width / 2, hit.y + hit.height / 2);
    assert.match(await page.textContent("#view .tip"), /^within [0-9.]+ s/);
    await page.click("#view details.instances summary");
    await page.click('#view table.times th[data-col="1"]');
    assert.equal(await page.getAttribute('#view table.times th[data-col="1"]', "aria-sort"), "ascending");
    // The former page's address of a family of suites, #networks.
    const old = data.flatMap((d) => d.families).find((f) => kinds.some((k) => k.key === f.key));
    if (old) {
      await page.evaluate((h) => { location.hash = h; }, old.key);
      await page.waitForSelector(`#view[data-route="#/kind/${old.key}"] figure.plot`);
    }
    console.log(`ok the results page's plot and table: ${first}`);

    // The benchmarks' data, where the site has it: every suite's families
    // hold its instances, each once, and each family's file its instances,
    // and each program's text (in the family's file, or a file of its own)
    // or why it is not shown. A family's view: its times, an instance's
    // programs, a system's runs in the drawer; a link to a benchmark's former
    // page leads to its view.
    const benchmarks = path.join(site, "results", "benchmarks");
    let family = null;
    if (fs.existsSync(path.join(benchmarks, "index.json"))) {
      const index = JSON.parse(fs.readFileSync(path.join(benchmarks, "index.json"), "utf8"));
      let nf = 0, nfiles = 0;
      for (const [suite, fams] of Object.entries(index.suites)) {
        const s = bySuite.get(suite);
        assert.ok(s, `benchmarks/index.json: the suite ${suite} is in the results' data`);
        assert.deepEqual(fams.flatMap((f) => f.instances).sort(), s.instances.map((row) => row[0]).sort(), `${suite}: its families hold its instances`);
        for (const f of fams) {
          const fam = JSON.parse(fs.readFileSync(path.join(benchmarks, suite, `${f.key}.json`), "utf8"));
          assert.deepEqual(fam.instances.map((i) => i.name), f.instances, `${suite}/${f.key}: its instances`);
          for (const inst of fam.instances) {
            for (const t of inst.tabs ?? []) {
              for (const file of t.files) {
                if (file.src) assert.ok(fs.existsSync(path.join(benchmarks, file.src)), `${suite}/${inst.name}: ${file.src}`);
                else if (!file.held && !file.archive) assert.ok(file.sha in fam.texts, `${suite}/${inst.name}: the text of ${file.name}`);
                nfiles++;
              }
            }
            if (inst.tabs && (family === null || (suite === "geni" && family.suite !== "geni"))) {
              family = { suite, key: f.key, name: inst.name, tabs: inst.tabs.length };
            }
          }
          nf++;
        }
      }
      assert.ok(family, "an instance with its programs");
      const hash = `#/${family.suite}/${encodeURIComponent(family.key)}/${encodeURIComponent(family.name)}`;
      await visit(hash, "#programs table.given");
      assert.equal((await page.$$("#programs table.given tbody tr")).length, family.tabs, `${family.name}: a row a language`);
      await page.waitForFunction(() => {
        const pane = document.querySelector("#programs .pane");
        return pane !== null && !pane.textContent.includes("Loading");
      });
      assert.ok(await page.$("#programs .pane .lines pre.src, #programs .pane p.held, #programs .pane p.note"), `${family.name}: the file on show`);
      await page.click("#view table.grid tbody tr.on td button");
      await page.waitForSelector("#drawer:not([hidden]) h4");
      assert.ok((await page.textContent("#drawer h3")).length > 0, "the drawer names its system");
      await page.keyboard.press("Escape");
      await page.waitForSelector("#drawer", { state: "hidden" });
      await page.goto(`${base}results/benchmarks/${family.suite}/${encodeURIComponent(family.name)}.html`);
      await page.waitForSelector(`#view[data-route="${hash}"] #programs table.given`);
      console.log(`ok the benchmarks' data: ${nf} families, ${nfiles} files; ${family.suite}/${family.name}'s view and runs`);
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
        // The results: the overview, a kind, a suite, and a family with a
        // system's runs open beside it.
        const views = [["results", "", ".kind-card"], ["results-kind", `#/kind/${kinds[0].key}`, "figure.plot svg.cactus path.line"],
          ["results-suite", `#/${first}`, "figure.plot svg.cactus path.line"]];
        if (family) views.push(["results-family", `#/${family.suite}/${encodeURIComponent(family.key)}/${encodeURIComponent(family.name)}`, "#programs table.given"]);
        for (const [name, hash, ready] of views) {
          await shot.goto(`${base}results/${hash}`);
          await shot.waitForSelector(`#view[data-route="${hash || "#/"}"] ${ready}`);
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
          assert.deepEqual(bar, ["Playground", "Guide", "Results", "API", "Python", "C", "GitHub"].filter((n) => n !== title.split(" ").pop()),
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
