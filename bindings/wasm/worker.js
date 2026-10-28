"use strict";

// The solver runs here, off the page's thread: the page stays live during a
// run, and Cancel ends this worker.
let solver = null;
// What the module wrote to stderr during the current run: the reason, where
// it aborts.
let stderr = [];

// The page passes its version stamp to every dependency.
Promise.resolve().then(() => {
  importScripts("runtime.js" + location.search, "tddlog.js" + location.search);
  return createTddlog({
    locateFile: (file) => file + location.search,
    print: () => {},
    printErr: (line) => stderr.push(line),
  });
}).then((module) => {
  solver = module;
  postMessage({ ready: true });
}).catch((failure) => postMessage({ loadError: String(failure) }));

onmessage = (event) => {
  const { program, strategy, evidence, format } = event.data;
  stderr = [];
  try {
    const started = performance.now();
    const result = tddlogRuntime.solve(solver, program, strategy, evidence, format);
    postMessage({ result, elapsed: performance.now() - started });
  } catch (failure) {
    // The module may have aborted, and an aborted module fails every call
    // after it: the page replaces this worker.
    postMessage({ failure: tddlogRuntime.failure(failure, stderr) });
  }
};
