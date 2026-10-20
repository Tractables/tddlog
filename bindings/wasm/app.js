"use strict";

// The page: the examples, the address bar, the worker the solver runs in,
// and the result.

// A line on each strategy under the menu, from the command line's --help.
const STRATEGY_NOTES = {
  auto: "Trials of the layouts below over rewrites of the program, under bounds that double until one finishes.",
  td: "Right-linear over a fact order from a tree decomposition.",
  closed: "Components in an order found by local search, each on a right-linear spine.",
  families: "A tree decomposition of the program read through indicators.",
  "families:0": "families without the decomposer's search.",
  dynamic: "Right-linear, changed during the compile by a search of rotations.",
  program: "Right-linear in the order the program states its facts.",
  "program:fanin": "Right-linear in the fan-in order of the rules from the queries and evidence, interleaved over them.",
  decision: "The tree of the decisions the grounder read facts as guards in, its contexts and the atoms read outside it named; fails for a program with none.",
};

// The host keeps the page's files in caches for a while, so the workflow
// stamps the references in index.html with the commit (app.js?v=<sha>), and
// the stamp goes on to every file the page loads: a new page never runs with
// an older file a browser still holds. Opened by hand there is no stamp.
const stamp = typeof document === "undefined" || document.currentScript === null
  ? ""
  : new URL(document.currentScript.src).search;

// ------------------------------------------------------------- the address

// An edited program goes in the address after `#program=`, so that a link
// reproduces it: deflated where the browser can (`z`), as it is otherwise
// (`t`), then base64url. The fragment never reaches the server, so the
// length of a request line does not bound the program.
function base64url(bytes) {
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64url(text) {
  const binary = atob(text.replace(/-/g, "+").replace(/_/g, "/"));
  return Uint8Array.from(binary, (c) => c.charCodeAt(0));
}

async function through(bytes, stream) {
  return new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(stream)).arrayBuffer());
}

async function encodeProgram(text) {
  const bytes = new TextEncoder().encode(text);
  try {
    return "z" + base64url(await through(bytes, new CompressionStream("deflate")));
  } catch (failure) {
    return "t" + base64url(bytes);
  }
}

async function decodeProgram(code) {
  const bytes = fromBase64url(code.slice(1));
  if (code[0] === "t") return new TextDecoder().decode(bytes);
  if (code[0] === "z") return new TextDecoder().decode(await through(bytes, new DecompressionStream("deflate")));
  throw new Error("the address holds no program");
}

// ------------------------------------------------------------- formatting

function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

const count = (n) => n.toLocaleString("en-US");

function duration(seconds) {
  if (seconds < 0.001) return "&lt;1 ms";
  if (seconds < 1) return `${Math.round(seconds * 1000)} ms`;
  return `${seconds.toFixed(2)} s`;
}

// The spaces lay nothing out (the figures are flex items) but keep the
// text readable as text: "1,306 instances in the release run 1,198 ...".
const stat = (figure, word) => `<span class="stat"><b>${figure}</b> ${word}</span> `;

// The figures of a result: the queries, the layout that answered, the
// largest formula of the run, and the time the call took in this browser.
function summarise(result, elapsed) {
  const parts = result.parts;
  const largest = Math.max(0, ...parts.map((p) => Math.max(p.largestPairs, p.queryPairs)));
  const layouts = [...new Set(parts.map((p) => p.trial))].join(", ");
  let out = "";
  if (result.answers !== null) out += stat(count(result.answers.length), result.answers.length === 1 ? "query" : "queries");
  if (layouts) out += stat(escapeHtml(layouts), "layout");
  if (parts.length > 0) out += stat(count(largest), "pairs at most");
  out += stat(duration(elapsed / 1000), "");
  return out;
}

// The answers as a table, each with its probability as a bar, and ProbLog's
// where the program is an example as the page loaded it.
function answersHtml(result, digits, example) {
  if (result.error !== undefined) return `<pre class="failed">${escapeHtml(result.error)}</pre>`;
  let out = "";
  if (result.answers === null) {
    out += '<p class="note">The evidence is inconsistent: its probability is zero, and no query has an answer.</p>';
  } else if (result.answers.length === 0) {
    out += '<p class="note">The program asks no query.</p>';
  } else {
    const reference = example === null ? null : new Map(example.problog);
    out += '<table><thead><tr><th>Query</th><th class="p">Probability</th><th></th>';
    if (reference !== null) out += '<th class="p" title="ProbLog 2.3.0, as the repository\'s tests record it">ProbLog</th>';
    out += "</tr></thead><tbody>";
    for (const { query, probability } of result.answers) {
      const width = Math.max(0, Math.min(1, probability)) * 100;
      out += `<tr><td class="query">${escapeHtml(query)}</td>`;
      out += `<td class="p" title="${probability}">${tddlogRuntime.probability(probability, digits)}</td>`;
      out += `<td class="bar"><i class="${width === 0 ? "zero" : ""}" style="width:${width}%"></i></td>`;
      if (reference !== null) {
        const p = reference.get(query);
        out += `<td class="p reference">${p === undefined ? "" : tddlogRuntime.probability(p, digits)}</td>`;
      }
      out += "</tr>";
    }
    out += "</tbody></table>";
  }
  if (result.evidence !== null) {
    const e = tddlogRuntime.evidence(result.evidence, digits);
    let line = `P(evidence) <b>${e.scientific}</b>, log10 <b>${e.log10}</b>`;
    if (example !== null) line += `; ProbLog <b>${example.evidence.toExponential(digits)}</b>`;
    out += `<p class="evidence">${line}</p>`;
  }
  return out;
}

function groundText(g) {
  return `${count(g.atoms)} atoms (${count(g.facts)} facts, ${count(g.derived)} derived), ${count(g.rules)} rules`;
}

// What tddlog did: for each part of the program answered on its own, the
// layout that answered and how auto came to it, the program and the
// formulas in pairs, and the seconds of each stage.
function workHtml(result, strategy) {
  const terms = (rows) => "<dl>" + rows.map(([t, d]) => `<dt>${t}</dt><dd>${d}</dd>`).join("") + "</dl>";
  let out = "";
  const parts = result.parts;
  if (parts.length === 0 && result.compiling !== null) {
    out += "<h3>What tddlog did</h3>" + terms([["Ground program", groundText(result.compiling)]]);
  }
  parts.forEach((p, i) => {
    out += `<h3>${parts.length > 1 ? `Part ${i + 1} of ${parts.length}` : "What tddlog did"}</h3>`;
    const layout = strategy === "auto"
      ? `<code>${escapeHtml(p.trial)}</code>, the trial <code>auto</code> answered with, after ${count(p.attempts)} ${p.attempts === 1 ? "compilation" : "compilations"}`
      : `<code>${escapeHtml(p.trial)}</code>`;
    const stages = [
      `ground ${duration(result.groundSeconds)}`,
      `rewrites and vtree ${duration(p.rewriteSeconds + p.vtreeSeconds)}`,
      `compile ${duration(p.compileSeconds)}`,
      `evidence and queries ${duration(p.querySeconds)}`,
    ];
    if (p.discardedSeconds > 0) stages.push(`trials given up ${duration(p.discardedSeconds)}`);
    out += terms([
      ["Layout", layout],
      ["Ground program", groundText(p.ground)],
      ["Vtree", `over ${count(p.leaves)} ${p.leaves === 1 ? "variable" : "variables"}`],
      ["Compile", `${count(p.components)} components (${count(p.cyclic)} cyclic); largest formula ${count(p.largestPairs)} pairs, ${count(p.finalPairs)} at the end`],
      ["Evidence and queries", `evidence ${count(p.evidencePairs)} pairs; largest formula ${count(p.queryPairs)} pairs`],
      ["Time", stages.join(", ")],
    ]);
  });
  if (result.notes.length > 0) {
    out += "<h3>Notes</h3><dl>" + result.notes.map((n) => `<dd>${escapeHtml(n)}</dd>`).join("") + "</dl>";
  }
  return out;
}

// --------------------------------------------------------------- the page

if (typeof document !== "undefined") {
  const element = (id) => document.getElementById(id);
  const editor = element("program");

  // The strategies, auto first.
  for (const name of tddlogRuntime.strategies) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    element("strategy").append(option);
  }
  const showStrategy = () => {
    element("strategy-note").textContent = STRATEGY_NOTES[element("strategy").value] ?? "";
  };
  element("strategy").addEventListener("change", showStrategy);

  // A button per example; the one whose program is in the editor, unedited,
  // is marked, and its ProbLog answers go beside tddlog's.
  const examples = element("examples");
  tddlogExamples.forEach(({ name }, i) => {
    const chip = document.createElement("button");
    chip.type = "button";
    chip.textContent = name;
    chip.dataset.example = i;
    examples.append(chip);
  });
  let chosen = -1;
  const markExample = (i) => {
    chosen = i;
    for (const chip of examples.children) chip.classList.toggle("chosen", chip.dataset.example === String(i));
  };

  // Choosing an example runs it once the solver is there: the point of the
  // buttons is to see what comes out.
  let loading = 0;
  async function loadExample(i) {
    const ticket = ++loading;
    let text;
    try {
      const response = await fetch(tddlogExamplePath(tddlogExamples[i]) + stamp);
      if (!response.ok) throw new Error(`${response.status}`);
      text = await response.text();
    } catch (failure) {
      element("status").textContent = `the example did not load: ${failure}`;
      return;
    }
    if (ticket !== loading) return;
    editor.value = text;
    markExample(i);
    requestRun();
  }
  examples.addEventListener("click", (event) => {
    const chip = event.target.closest("button");
    if (chip !== null) loadExample(Number(chip.dataset.example));
  });

  // Once edited, the text is no longer the example it started from.
  editor.addEventListener("input", () => {
    loading++;
    markExample(-1);
  });
  editor.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      requestRun();
    }
  });

  // The address carries the choices, so a result can be linked to:
  // ?example=<key>&strategy=<s>&evidence=1&digits=<n>, and an edited program
  // after #program=. It is read once when the page opens and written at
  // every run.
  function readAddress() {
    const params = new URLSearchParams(location.search);
    const strategy = params.get("strategy");
    if (tddlogRuntime.strategies.includes(strategy)) element("strategy").value = strategy;
    element("evidence").checked = params.get("evidence") === "1";
    if (params.get("digits") !== null) element("digits").value = params.get("digits");
    const program = new URLSearchParams(location.hash.slice(1)).get("program");
    return { example: tddlogExamples.findIndex(({ key }) => key === params.get("example")), program };
  }
  async function writeAddress(settings) {
    const params = new URLSearchParams();
    if (chosen !== -1) params.set("example", tddlogExamples[chosen].key);
    if (settings.strategy !== "auto") params.set("strategy", settings.strategy);
    if (settings.evidence) params.set("evidence", "1");
    if (settings.digits !== 6) params.set("digits", String(settings.digits));
    const query = params.toString();
    let hash = "";
    if (chosen === -1) hash = "#program=" + (await encodeProgram(settings.program));
    history.replaceState(null, "", location.pathname + (query ? `?${query}` : "") + hash);
    element("link").hidden = false;
  }
  function flash(button, text) {
    const label = button.textContent;
    button.textContent = text;
    setTimeout(() => {
      button.textContent = label;
    }, 1500);
  }
  element("link").addEventListener("click", () => {
    navigator.clipboard.writeText(location.href).then(
      () => flash(element("link"), "Copied"),
      () => flash(element("link"), "Not copied"),
    );
  });

  // The solver runs in a worker, so the page stays live during a run, the
  // status can count the seconds, and a run can be stopped: Cancel ends the
  // worker and starts another, which takes a moment to load the module. A
  // run asked for before the worker is ready, or while another runs, starts
  // once the worker is.
  let worker = null;
  let ready = false;
  let running = false;
  let runWhenReady = false;
  let ticking = 0;
  // The settings of the run on show, and its result, which a change of
  // digits writes again.
  let shown = null;

  function startSolver() {
    ready = false;
    element("run").disabled = true;
    worker = new Worker("worker.js" + stamp);
    worker.onmessage = (event) => {
      const data = event.data;
      if (data.ready === true) {
        ready = true;
        element("run").disabled = false;
        if (!running) element("status").textContent = "ready";
        if (runWhenReady) {
          runWhenReady = false;
          run();
        }
      } else if (data.loadError !== undefined) {
        loadFailed(data.loadError);
      } else if (data.failure !== undefined) {
        // The module may have aborted, and an aborted module fails every
        // call after it, so the worker is replaced as Cancel does.
        const settings = shown === null ? null : shown.settings;
        stop();
        show({ error: data.failure, groundSeconds: 0, compiling: null, notes: [], parts: [], answers: null, evidence: null }, 0, settings);
        element("status").textContent = "failed; loading the solver again";
      } else {
        finish(data.result, data.elapsed);
      }
    };
    worker.onerror = (event) => loadFailed(event.message);
  }

  function loadFailed(message) {
    worker.terminate();
    ready = false;
    runWhenReady = false;
    settle();
    element("run").disabled = true;
    console.error("The solver did not load", message);
    element("status").textContent = "the solver did not load; reload the page to retry";
  }

  function requestRun() {
    if (running) {
      stop();
      runWhenReady = true;
    } else if (ready) {
      run();
    } else {
      runWhenReady = true;
    }
  }

  function stop() {
    worker.terminate();
    settle();
    startSolver();
  }

  function settle() {
    running = false;
    clearInterval(ticking);
    document.body.classList.remove("busy");
    element("cancel").hidden = true;
    element("run").hidden = false;
  }

  function settings() {
    return {
      program: editor.value,
      strategy: element("strategy").value,
      evidence: element("evidence").checked,
      digits: tddlogRuntime.digits(element("digits").value),
      example: chosen === -1 ? null : tddlogExamples[chosen],
    };
  }

  function run() {
    let chosenSettings;
    try {
      chosenSettings = settings();
    } catch (failure) {
      element("status").textContent = failure.message;
      return;
    }
    writeAddress(chosenSettings);
    running = true;
    shown = { settings: chosenSettings, result: null, elapsed: 0 };
    const started = performance.now();
    element("status").textContent = "running";
    ticking = setInterval(() => {
      element("status").textContent = `running, ${Math.round((performance.now() - started) / 1000)} s`;
    }, 1000);
    element("run").hidden = true;
    element("cancel").hidden = false;
    document.body.classList.add("busy");
    const { program, strategy, evidence } = chosenSettings;
    worker.postMessage({ program, strategy, evidence });
  }

  element("run").addEventListener("click", requestRun);
  element("cancel").addEventListener("click", () => {
    if (!running) return;
    stop();
    element("status").textContent = "stopped; loading the solver";
  });

  function show(result, elapsed, chosenSettings) {
    shown = { settings: chosenSettings, result, elapsed };
    const digits = chosenSettings === null ? 6 : chosenSettings.digits;
    const strategy = chosenSettings === null ? "auto" : chosenSettings.strategy;
    const example = chosenSettings === null ? null : chosenSettings.example;
    element("summary").innerHTML = result.error === undefined ? summarise(result, elapsed) : "";
    element("answers").innerHTML = answersHtml(result, digits, example);
    element("work").innerHTML = workHtml(result, strategy);
    const trials = result.parts.flatMap((p) => p.trials);
    element("trials").hidden = trials.length === 0;
    element("trial-notes").textContent = trials.join("\n");
  }

  function finish(result, elapsed) {
    settle();
    show(result, elapsed, shown.settings);
    element("status").textContent = result.error === undefined ? "ready" : "failed";
  }

  // Digits change how the answers on show are written, not the answers.
  element("digits").addEventListener("input", () => {
    if (shown === null || shown.result === null) return;
    try {
      const digits = tddlogRuntime.digits(element("digits").value);
      show(shown.result, shown.elapsed, { ...shown.settings, digits });
    } catch (failure) {
      element("status").textContent = failure.message;
    }
  });

  // The results at a glance load beside the solver; without them the
  // card stays hidden.
  glance().catch(() => {});
  // The page opens on the program in the address, else the example it
  // names, else the first.
  startSolver();
  const opening = readAddress();
  showStrategy();
  if (opening.program !== null) {
    decodeProgram(opening.program).then(
      (text) => {
        editor.value = text;
        requestRun();
      },
      () => loadExample(0),
    );
  } else {
    loadExample(opening.example === -1 ? 0 : opening.example);
  }
}

// ------------------------------------------------------- the results at a glance

// The release run's instances, those tddlog answers and those it alone
// answers, from the records the results page reads (results/). A record
// is an instance's name and each system's time, or why it has none.
async function glance() {
  const load = async (file) => {
    const response = await fetch(`results/${file}${stamp}`);
    if (!response.ok) throw new Error(`${file}: ${response.status}`);
    return response.json();
  };
  const parts = (await Promise.all(["suites.json", "ppl.json"].map((file) => load(file).catch(() => null)))).filter(Boolean);
  let all = 0, answered = 0, alone = 0;
  for (const suite of parts.flatMap((p) => p.families.flatMap((f) => f.suites))) {
    const own = suite.systems.indexOf("tddlog");
    for (const [, ...times] of suite.instances) {
      all += 1;
      if (typeof times[own] !== "number") continue;
      answered += 1;
      if (times.every((t, i) => i === own || typeof t !== "number")) alone += 1;
    }
  }
  if (all === 0) return;
  document.getElementById("glance-stats").innerHTML = stat(count(all), "instances in the release run")
    + stat(count(answered), "answered by tddlog") + stat(count(alone), "answered by tddlog alone");
  document.getElementById("glance").hidden = false;
}

if (typeof module !== "undefined") {
  module.exports = { encodeProgram, decodeProgram, answersHtml, workHtml, summarise, escapeHtml };
}
