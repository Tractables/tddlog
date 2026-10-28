"use strict";

// Shared by the page, the worker and the tests: the call into the module, and
// how the page writes what comes back.
const tddlogRuntime = {
  // The vtree strategies the command line's --vtree takes, for the page's
  // menu; `auto` first, the default. The module refuses any other name.
  strategies: ["auto", "td", "closed", "families", "families:0", "dynamic", "program", "program:fanin", "decision"],

  // The formats the module reads, for the page's menu: each name as the
  // module takes it, what the menu says, and the file extensions the
  // command line reads it by. The first is the default.
  formats: [
    ["problog", "ProbLog", []],
    ["dice", "Dice", [".dice"]],
    ["bif", "BIF network", [".bif"]],
    ["hugin", "Hugin network", [".net", ".hugin"]],
    ["xdsl", "GeNIe network", [".xdsl"]],
    ["uai", "UAI network", [".uai"]],
  ],

  // The format of a file named `name`, by its extension, as the command
  // line tells it.
  formatOf(name) {
    const found = this.formats.find(([, , extensions]) => extensions.some((e) => name.endsWith(e)));
    return (found ?? this.formats[0])[0];
  },

  // The answers of `program`, text in `format` (a network's with `observed`
  // the text of its evidence file, or null), under `strategy`, with P(e)
  // where `evidence` is true, as the object src/main.rs writes:
  //   answers   [{query, probability}] in the program's order, or null where
  //             the evidence is inconsistent;
  //   evidence  {probability, log10, z} where P(e) was asked for, else null;
  //             log10 is null where P(e) is zero, and z is true where it is
  //             a Markov network's partition function given the evidence;
  //   error     a message, where the program or the options are refused or
  //             the run fails (and then no answers);
  //   groundSeconds, compiling (the program handed to the last compile),
  //   notes (on the run as a whole) and parts (each part's run: the trial
  //   that answered, its sizes in pairs and its seconds).
  solve(module, program, strategy, evidence, format = "problog", observed = null) {
    if (typeof evidence !== "boolean") throw new Error("evidence must be true or false");
    const encoder = new TextEncoder();
    // ccall's string conversion uses the stack, which cannot hold large
    // programs; every string goes on the heap, and a missing evidence file
    // is a null pointer.
    const texts = [program, format, observed, strategy].map((text) => (text === null ? null : encoder.encode(text + "\0")));
    const at = [];
    let output = 0;
    try {
      for (const bytes of texts) {
        const ptr = bytes === null ? 0 : module._malloc(bytes.length) >>> 0;
        if (bytes !== null && !ptr) throw new Error("could not allocate the program");
        at.push(ptr);
        if (ptr) module.HEAPU8.set(bytes, ptr);
      }
      output = module._tddlog_solve_json(...at, evidence ? 1 : 0) >>> 0;
      if (!output) throw new Error("the solver returned nothing");
      return JSON.parse(module.UTF8ToString(output));
    } finally {
      if (output) module._tddlog_string_free(output);
      for (const ptr of at) if (ptr) module._free(ptr);
    }
  },

  // Why a call threw rather than returned: the module aborted, which it does
  // where an allocation the solver cannot refuse fails, or its stack ran out.
  // `stderr` is what the module wrote before it stopped.
  failure(error, stderr) {
    const said = [String(error), ...stderr].join("\n");
    if (/memory allocation of|Cannot enlarge memory|out of memory|OOM/i.test(said)) {
      return "tddlog ran out of memory. A page can use at most 4 GB, and less on some devices; the command-line tool has no such limit.";
    }
    if (/call stack|stack overflow|Maximum call stack/i.test(said)) {
      return "tddlog ran out of stack. The command-line tool runs it with the stack the system gives it.";
    }
    const last = stderr.filter((line) => line.trim() !== "").slice(-3).join(" ");
    return `the solver stopped: ${error}${last ? ` (${last})` : ""}`;
  },

  // Digits after the point, from 0 to 17, as --precision takes them.
  digits(value) {
    const number = Number(value);
    if (String(value).trim() === "" || !Number.isInteger(number) || number < 0 || number > 17) {
      throw new Error("digits must be an integer from 0 to 17");
    }
    return number;
  },

  // A probability as the command line writes it: `digits` after the point,
  // and in scientific notation where it is not zero and under 1e-4.
  probability(p, digits) {
    return p > 0 && p < 1e-4 ? p.toExponential(digits) : p.toFixed(digits);
  },

  // P(e) as the command line writes it, in scientific notation, with its
  // base-10 logarithm; a P(e) under the least normal float has its mantissa
  // and exponent read off the logarithm.
  evidence(e, digits) {
    if (e.log10 === null) return { scientific: (0).toExponential(digits), log10: "-inf" };
    let scientific;
    if (e.probability >= 2.2250738585072014e-308) {
      scientific = e.probability.toExponential(digits);
    } else {
      let exponent = Math.floor(e.log10);
      let mantissa = (10 ** (e.log10 - exponent)).toFixed(digits);
      // A mantissa that rounds to ten is one at the next power.
      if (mantissa.startsWith("10")) {
        mantissa = (1).toFixed(digits);
        exponent += 1;
      }
      scientific = `${mantissa}e${exponent}`;
    }
    return { scientific, log10: e.log10.toFixed(digits) };
  },
};

if (typeof module !== "undefined") module.exports = tddlogRuntime;
