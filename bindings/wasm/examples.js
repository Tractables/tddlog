"use strict";

// The page's examples, shared by the page, the site's assembly and the tests.
// Each is a program the repository already tests against ProbLog: `file` is
// its path in the repository, and the site serves it at examples/<name>.pl.
// `key` names it in the address bar and `name` on its button.
//
// `problog` holds ProbLog 2.3.0's answers (`-k sdd`, one query at a time, as
// the repository's differential tests take them; here to 12 significant
// digits), and `evidence` its P(e): the probability of one rule whose body is
// the evidence, queried without it, and 1 for a program without evidence.
// tests/examples.cjs checks the built module against both.
const tddlogExamples = [
  {
    key: "alarm",
    name: "Burglary and alarm",
    file: "examples/alarm.pl",
    problog: [["burglary", 0.357142857143], ["earthquake", 0.714285714286], ["calls(mary)", 0.4]],
    evidence: 0.196,
  },
  {
    key: "coins",
    name: "Tossing coins",
    file: "tests/problog/3_tossing_coin.pl",
    problog: [["someHeads", 0.9744]],
    evidence: 1,
  },
  {
    key: "dice",
    name: "A die until it shows six",
    file: "examples/dice.pl",
    problog: [["on(0,1)", 1], ["on(1,1)", 1], ["on(2,1)", 0.166666666667]],
    evidence: 0.0277777777778,
  },
  {
    key: "cards",
    name: "Cards, annotated disjunctions",
    file: "tests/problog/10_cards.pl",
    problog: [
      ["doublecard", 0.25], ["samecard(q,h)", 0.0625], ["samecard(q,s)", 0.0625],
      ["samecard(k,h)", 0.0625], ["samecard(k,s)", 0.0625],
    ],
    evidence: 1,
  },
  {
    key: "asia",
    name: "Bayesian network: asia",
    file: "examples/bn_asia_evidence.pl",
    problog: [
      ["asia(yes)", 0.0116784200427], ["asia(no)", 0.988321579957],
      ["tub(yes)", 0.0540212892219], ["tub(no)", 0.945978710778],
      ["smoke(yes)", 0.513207093653], ["smoke(no)", 0.486792906347],
      ["lung(yes)", 0.252297229882], ["lung(no)", 0.747702770118],
      ["bronc(yes)", 0.193211096486], ["bronc(no)", 0.806788903514],
      ["either(yes)", 0.303694627914], ["either(no)", 0.696305372086],
    ],
    evidence: 0.0396199356,
  },
  {
    key: "smokers",
    name: "Smokers in a social network",
    file: "tests/problog/8_smokers_network.pl",
    problog: [
      ["smokes(1)", 0.508771929825], ["smokes(2)", 1], ["smokes(3)", 0.44], ["smokes(4)", 0.44],
      ["asthma(1)", 0.20350877193], ["asthma(2)", 0.4], ["asthma(3)", 0.176], ["asthma(4)", 0.176],
    ],
    evidence: 0.2736,
  },
  {
    key: "reachability",
    name: "Reachability in a graph with cycles",
    file: "examples/cycle_graph.pl",
    problog: [
      ["path(1,2)", 0.632256], ["path(1,3)", 0.460356], ["path(1,4)", 0.399624],
      ["path(1,5)", 0.754432], ["path(1,6)", 0.5716992], ["path(1,7)", 0.199812],
    ],
    evidence: 1,
  },
  {
    key: "adder",
    name: "Adding integers in binary",
    file: "examples/adder.pl",
    problog: [["sum(1)", 0.491304347826], ["sum(2)", 0.496869565217], ["at_least_8", 0.505913043478], ["x(0)", 0.608695652174]],
    evidence: 0.46,
  },
  {
    key: "win",
    name: "A game: negation through recursion",
    file: "examples/win.pl",
    problog: [["win(1)", 0.5625], ["win(2)", 0.25], ["win(3)", 0.5]],
    evidence: 1,
  },
  {
    key: "gears",
    name: "Gear wheels: a loop under negation",
    file: "examples/gearwheels.pl",
    problog: [["turns1(0)", 0], ["turns1(1)", 0.72], ["turns2(1)", 0.72]],
    evidence: 1,
  },
  {
    key: "digits",
    name: "Sum of two digits",
    file: "tests/lit/mnist_sum.pl",
    problog: [
      ["sum(0)", 0.00140544], ["sum(1)", 0.00374336], ["sum(2)", 0.022827], ["sum(3)", 0.03859655],
      ["sum(4)", 0.00591345], ["sum(5)", 0.02648234], ["sum(6)", 0.04219473], ["sum(7)", 0.02116928],
      ["sum(8)", 0.05089386], ["sum(9)", 0.08413355], ["sum(10)", 0.04095259], ["sum(11)", 0.50704523],
      ["sum(12)", 0.02676586], ["sum(13)", 0.02359843], ["sum(14)", 0.02757976], ["sum(15)", 0.03501714],
      ["sum(16)", 0.00668589], ["sum(17)", 0.00121972], ["sum(18)", 0.03377582],
    ],
    evidence: 1,
  },
];

// Where the site serves an example's program.
function tddlogExamplePath(example) {
  return "examples/" + example.file.split("/").pop();
}

if (typeof module !== "undefined") module.exports = { tddlogExamples, tddlogExamplePath };
