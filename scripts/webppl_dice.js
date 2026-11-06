// webppl_dice.js PROGRAM.wppl: runs a WebPPL program that dice2webppl.py
// wrote, whose last statement is `Infer({method: 'enumerate'}, model)`, the
// exact distribution of the Dice program's value, through WebPPL's own API
// (src/main.js: parsePackageCode, compile, prepare), and prints
//   webppl_c K VALUE P         the marginal of each value of each component K,
//                              as the program's `// component K TYPE` lines
//                              give them: VALUE true or false for a Boolean,
//                              0 to 2^W - 1 for an int W (0 where it has no
//                              mass), past 2^24 values only those with mass
//   webppl_seconds S           WebPPL's compile and run of the program
//   webppl_compile_seconds S   the compile alone, the program with WebPPL's
//                              header (its library, written in WebPPL, which
//                              WebPPL compiles with every program)
//   webppl_header_seconds S    the compile of an empty program: the header's
//                              share of a compile
// The clock starts once node has started, WebPPL's modules are loaded and its
// header is parsed, and after a warm-up, the compile and run of a small model,
// so that V8 has compiled WebPPL's own code before it is timed, as a Julia
// system is timed warm. A probability is printed as JavaScript prints a
// number, the shortest decimal that reads back as the same double.
// With FACTS, a facts file that problog2ppl.py wrote beside a logic program's
// WebPPL program (a line `relation<TAB>probability<TAB>argument...` a fact),
// the program reads the facts as the global lpFacts, {relation: [[argument
// ..., probability], ...]}, and their reading is part of webppl_seconds, as
// the other systems' reading of their facts is part of their time.
// WebPPL is found as require('webppl'): NODE_PATH names the node_modules
// directory it is installed in. Run as node's script:
//   NODE_PATH=WEBPPL/node_modules node webppl_dice.js PROGRAM.wppl [FACTS]
'use strict';
const fs = require('fs');

function fail(msg) {
  process.stderr.write('webppl_dice.js: ' + msg + '\n');
  process.exit(1);
}
process.on('uncaughtException', function(e) { fail(e && e.stack ? e.stack : String(e)); });

if (process.argv.length !== 3 && process.argv.length !== 4) {
  fail('usage: webppl_dice.js PROGRAM.wppl [FACTS]');
}
const file = process.argv[2];
const factsFile = process.argv[3];
const code = fs.readFileSync(file, 'utf8');
const MAX_LISTED = 24;   // the widest int whose every value is printed
const widths = [];   // a component's int width, or null for a Boolean
for (const line of code.split('\n')) {
  const m = /^\/\/ component (\d+) (?:bool|int (\d+))$/.exec(line);
  if (m) {
    if (Number(m[1]) !== widths.length) {
      fail(file + ': component ' + m[1] + ' out of order');
    }
    widths.push(m[2] === undefined ? null : Number(m[2]));
  }
}
if (!widths.length) {
  fail(file + ' has no `// component K TYPE` line');
}

const webppl = require('webppl');
const bundles = webppl.parsePackageCode([], false);

function now() {
  return Number(process.hrtime.bigint()) / 1e9;
}

// compile and run TEXT, then k(the program's value, compile seconds, run seconds)
function compileRun(text, filename, k) {
  const t0 = now();
  const compiled = webppl.compile(text, {bundles: bundles, filename: filename});
  const t1 = now();
  webppl.prepare(compiled, function(s, x) {
    const t2 = now();
    setImmediate(function() { k(x, t1 - t0, t2 - t1); });
  }, {}).run();
}

function leaves(v, out) {
  if (Array.isArray(v)) {
    v.forEach(function(x) { leaves(x, out); });
  } else {
    out.push(v);
  }
  return out;
}

function report(dist, compileS, runS) {
  if (!dist || typeof dist.getDist !== 'function') {
    fail(file + ': the program\'s value is not a distribution');
  }
  const masses = widths.map(function() { return new Map(); });
  const rows = dist.getDist();
  for (const key of Object.keys(rows)) {
    const vs = leaves(rows[key].val, []);
    if (vs.length !== widths.length) {
      fail(file + ': a value of ' + vs.length + ' components, ' + key + ', where ' + widths.length + ' are declared');
    }
    vs.forEach(function(v, k) {
      const w = widths[k];
      const i = w === null ? (v === true ? 1 : v === false ? 0 : -1) :
        (Number.isInteger(v) && v >= 0 && v < 2 ** w ? v : -1);
      if (i < 0) {
        fail(file + ': component ' + k + ' takes ' + JSON.stringify(v) + ', not a ' + (w === null ? 'Boolean' : 'int ' + w));
      }
      masses[k].set(i, (masses[k].get(i) || 0) + rows[key].prob);
    });
  }
  const t0 = now();
  webppl.compile('', {bundles: bundles, filename: 'empty'});
  const header = now() - t0;
  const out = [];
  masses.forEach(function(m, k) {
    const w = widths[k];
    if (w === null) {
      out.push('webppl_c ' + k + ' true ' + (m.get(1) || 0), 'webppl_c ' + k + ' false ' + (m.get(0) || 0));
    } else if (w <= MAX_LISTED) {
      for (let v = 0; v < 2 ** w; v++) {
        out.push('webppl_c ' + k + ' ' + v + ' ' + (m.get(v) || 0));
      }
    } else {
      Array.from(m.keys()).sort(function(a, b) { return a - b; }).forEach(function(v) {
        out.push('webppl_c ' + k + ' ' + v + ' ' + m.get(v));
      });
    }
  });
  out.push('webppl_seconds ' + (compileS + runS), 'webppl_compile_seconds ' + compileS, 'webppl_header_seconds ' + header);
  process.stdout.write(out.join('\n') + '\n');
}

const WARM = [
  'var model = function() {',
  '  var a = flip(0.3);',
  '  var b = a ? categorical({ps: [0.2, 0.8], vs: [0, 1]}) : randomInteger(2);',
  '  condition(a || b == 1);',
  '  return [a, (b + 1) % 2];',
  '};',
  "Infer({method: 'enumerate'}, model);"].join('\n');

// the facts of a facts file as lpFacts holds them: an argument that starts
// as a number does is a number (problog2ppl.py writes no symbol that does)
function readFacts(path) {
  const facts = {};
  for (const line of fs.readFileSync(path, 'utf8').split('\n')) {
    if (!line) {
      continue;
    }
    const f = line.split('\t');
    const row = f.slice(2).map(function(s) { return /^[-+]?(\d|\.\d)/.test(s) ? Number(s) : s; });
    row.push(Number(f[1]));
    (facts[f[0]] = facts[f[0]] || []).push(row);
  }
  return facts;
}

compileRun(WARM, 'warm-up', function() {
  const t0 = now();
  if (factsFile !== undefined) {
    global.lpFacts = readFacts(factsFile);
  }
  const load = now() - t0;
  compileRun(code, file, function(dist, compileS, runS) { report(dist, compileS, runS + load); });
});
