// The programs the site shows, for every page that shows them (the guide,
// the playground and the results page).
//
// Syntax highlighting: each <pre data-lang="..."> is split into tokens by
// its language's rules, and each token is wrapped in a span whose class
// (t-com, t-str, ...) the stylesheet colours. The text itself is unchanged.
// The languages are the guide's: problog, shell (a session, "$ " before
// each command and the output under it, or commands alone), rust, python,
// c and toml.
//
// filledFile: the file of a benchmark's programs that the playground opens,
// which the results page's link to it names.
const tddlogCode = (() => {
  const plainWord = ["", /[A-Za-z_]\w*/y];
  const number = ["num", /\d+(?:\.\d+)?(?:[eE][-+]?\d+)?/y];
  const doubleQuoted = /"(?:[^"\\\n]|\\.)*"/y;
  const blockComment = ["com", /\/\*[\s\S]*?\*\//y];
  const lineComment = ["com", /\/\/[^\n]*/y];
  const call = ["fn", /[a-z_]\w*(?=\()/y];

  // Each language's rules in the order they are tried at a position; the
  // first that matches takes the token, and a rule with no class leaves it
  // plain (a plain word keeps the digits of a name out of the numbers).
  const languages = {
    problog: [
      ["com", /%[^\n]*/y],
      blockComment,
      ["str", doubleQuoted],
      ["str", /'(?:[^'\\\n]|\\.)*'/y],
      ["kw", /\b(?:query|evidence|true|false|is|not)\b/y],
      ["var", /[A-Z_]\w*/y],
      call,
      plainWord,
      number,
      ["op", /::|:-|\\\+|->|=:=|=\\=|\\==|==|=<|>=|\\=|[;=<>]/y],
    ],
    rust: [
      lineComment,
      blockComment,
      ["str", /"(?:[^"\\]|\\.)*"/y],
      ["kw", /\b(?:as|const|else|enum|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|static|struct|trait|type|use|where|while|true|false|Some|None|Ok|Err)\b/y],
      ["fn", /[a-z_]\w*!/y],
      call,
      ["typ", /[A-Z]\w*/y],
      plainWord,
      number,
    ],
    python: [
      ["com", /#[^\n]*/y],
      ["str", /[fFrRbB]?(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*')/y],
      ["kw", /\b(?:and|as|class|def|elif|else|except|for|from|if|import|in|is|lambda|not|or|pass|raise|return|try|while|with|yield|None|True|False)\b/y],
      call,
      ["typ", /[A-Z]\w*/y],
      plainWord,
      number,
    ],
    c: [
      lineComment,
      blockComment,
      ["kw", /#\w+/y],
      ["str", doubleQuoted],
      ["kw", /\b(?:bool|char|const|double|else|float|for|if|int|long|return|size_t|static|struct|unsigned|void|while|true|false|NULL)\b/y],
      ["num", /[A-Z][A-Z0-9]*_[A-Z0-9_]+\b/y],
      call,
      ["typ", /[A-Z]\w*/y],
      plainWord,
      number,
    ],
    toml: [
      ["com", /#[^\n]*/y],
      ["kw", /^\[[^\]\n]*\]/my],
      ["fn", /^[\w.-]+(?=\s*=)/my],
      ["str", doubleQuoted],
      plainWord,
      number,
    ],
  };

  function escape(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function span(cls, s) {
    return cls ? `<span class="t-${cls}">${escape(s)}</span>` : escape(s);
  }

  // [[class, text]] covering the whole of text, in order.
  function tokens(text, rules) {
    const out = [];
    let plain = "";
    for (let i = 0; i < text.length; ) {
      let hit = null;
      for (const [cls, re] of rules) {
        re.lastIndex = i;
        const m = re.exec(text);
        if (m && m[0]) { hit = [cls, m[0]]; break; }
      }
      if (!hit) { plain += text[i++]; continue; }
      i += hit[1].length;
      if (!hit[0]) { plain += hit[1]; continue; }
      if (plain) { out.push(["", plain]); plain = ""; }
      out.push(hit);
    }
    if (plain) out.push(["", plain]);
    return out;
  }

  // A command line: the command (the first word, and the first after
  // &&, | or ;), its flags, strings and redirections.
  function command(line) {
    let first = true;
    return line.split(/(\s+)/).map((part) => {
      if (!part || /^\s+$/.test(part)) return part;
      if (/^(?:&&|\|\|?|;)$/.test(part)) { first = true; return span("op", part); }
      if (/^(?:>>?|<)$/.test(part)) return span("op", part);
      const cls = first ? "fn" : /^-/.test(part) ? "kw" : /^["']/.test(part) ? "str" : "";
      first = false;
      return span(cls, part);
    }).join("");
  }

  // A session marks each command with "$ "; the lines under it are its
  // output, highlighted as ProbLog (what tddlog prints is atoms, numbers
  // and, with --ground, a program). A block with no "$ " is commands alone.
  function shell(text) {
    const session = /^\$ /m.test(text);
    return text.split("\n").map((line) => {
      if (!session) return command(line);
      if (!line.startsWith("$ ")) return tokens(line, languages.problog).map(([c, s]) => span(c, s)).join("");
      return span("prompt", "$") + " " + command(line.slice(2));
    }).join("\n");
  }

  function highlight(lang, text) {
    if (lang === "shell") return shell(text);
    const rules = languages[lang];
    if (!rules) throw new Error(`code.js: no rules for the language "${lang}"`);
    return tokens(text, rules).map(([c, s]) => span(c, s)).join("");
  }

  if (typeof document !== "undefined") {
    for (const pre of document.querySelectorAll("pre[data-lang]")) {
      pre.innerHTML = highlight(pre.dataset.lang, pre.textContent);
    }
  }

  // A family that is one program takes each instance's arguments in its
  // places ($1, $2, ...) and consults its data, which the playground cannot
  // give. For tddlog's file `file` of instance `inst` (in its tab `tab`, the
  // family's texts `texts`) that has places: the instance's program of its
  // own name in another ProbLog tab, the same program with its arguments in
  // their places and its consults inlined, as the ProbLog systems read it
  // (the one they read where there are several), as { tab, file }. Null
  // where `file` has no places or the results show no such program.
  function filledFile(inst, tab, file, texts) {
    if (!/\$[1-9]/.test(texts[file.sha] ?? "")) return null;
    const filled = (inst.tabs ?? []).filter((t) => t !== tab && t.lang === "ProbLog")
      .flatMap((t) => t.files.filter((x) => x.name === `${inst.name}.pl`).map((x) => ({ tab: t, file: x })));
    return filled.find(({ tab: t, file: x }) => (x.readers ?? t.readers).some((r) => r.startsWith("ProbLog"))) ?? filled[0] ?? null;
  }

  return { highlight, languages: ["shell", ...Object.keys(languages)], filledFile };
})();

if (typeof module !== "undefined") module.exports = tddlogCode;
