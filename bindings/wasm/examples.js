"use strict";

// The page's examples, shared by the page, the site's assembly and the tests.
// Each is a program the repository tests against ProbLog's answers or exact
// ones: `file` is its path in the repository, and the site serves it at
// examples/ under its file name. `key` names it in the address bar and `name`
// on its button; `format` names the format of a file that is not a ProbLog
// program (runtime.js's formats).
//
// `problog` holds ProbLog 2.3.0's answers (`-k sdd`, one query at a time, as
// the repository's differential tests take them; here to 12 significant
// digits), and `evidence` its P(e): the probability of one rule whose body is
// the evidence, queried without it, and 1 for a program without evidence.
// For the Dice program and the car network (its file and the program it is
// read as) they are computed by hand: Dice's own answers for the walk
// (tests/it/examples.rs), and the network's marginals by a sum over its
// worlds, which ProbLog's agree with (tests/it/networks.rs). Alea.jl's and
// Roulette's circuits hold them by hand too, from the programs they were
// built from (tests/it/examples.rs).
//
// The examples marked `larger` take a few seconds each (mastermind about
// 20), and ProbLog does not answer them within five minutes. Their
// `problog` holds exact answers,
// computed independently of tddlog and ProbLog as `exact` says; on the
// smaller sizes ProbLog answers, those computations agree with ProbLog and
// with tddlog.
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
    key: "car",
    name: "Bayesian network: car trouble",
    file: "examples/bn_car_evidence.pl",
    problog: [
      ["fuel(full)", 0.0872227771756], ["fuel(low)", 0.621332505662], ["fuel(empty)", 0.291444717162],
      ["plugs(ok)", 0.533974993546], ["plugs(fouled)", 0.466025006454],
      ["starter(ok)", 0.90144965579], ["starter(faulty)", 0.09855034421],
    ],
    evidence: 0.0226280992986,
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
    key: "tpch",
    name: "A TPC-H query over a database",
    file: "examples/tpch_q2.pl",
    problog: [["q2", 0.0177512033449]],
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
  {
    key: "walk",
    name: "Dice: a random walk",
    file: "tests/dice/walk.dice",
    format: "dice",
    problog: [
      ["x(0)", 0], ["x(1)", 0.245556733026], ["x(2)", 0], ["x(3)", 0.362357126637],
      ["x(4)", 0], ["x(5)", 0.190068881992], ["x(6)", 0], ["x(7)", 0.202017258345],
    ],
    evidence: 0.432898048,
  },
  {
    key: "circuit",
    name: "Alea.jl's circuit: two dice",
    file: "examples/alea_dice.pl",
    problog: [["q1", 0.166666666667], ["q2", 0.666666666667]],
    evidence: 0.5,
  },
  {
    key: "roulette",
    name: "Roulette's circuit: a faulty gate",
    file: "tests/roulette/lib/diagnosis.pl",
    problog: [["q1(1)", 1], ["q2(1)", 0.307692307692], ["q3(1)", 0.692307692308], ["q4(1)", 0]],
    evidence: 0.26,
  },
  {
    key: "car-net",
    name: "Hugin network: car trouble",
    file: "examples/car.net",
    format: "hugin",
    problog: [
      ["weather(cold)", 0.3], ["weather(mild)", 0.5], ["weather(hot)", 0.2],
      ["battery_age(new)", 0.7], ["battery_age(old)", 0.3], ["lights_left_on(yes)", 0.1], ["lights_left_on(no)", 0.9],
      ["battery(charged)", 0.7445], ["battery(weak)", 0.1699], ["battery(flat)", 0.0856],
      ["headlights(bright)", 0.73276], ["headlights(dim)", 0.165765], ["headlights(off)", 0.101475],
      ["starter(ok)", 0.97], ["starter(faulty)", 0.03],
      ["cranks(yes)", 0.74994095], ["cranks(slowly)", 0.11996505], ["cranks(no)", 0.130094],
      ["fuel(full)", 0.55], ["fuel(low)", 0.35], ["fuel(empty)", 0.1],
      ["fuel_gauge(full)", 0.53], ["fuel_gauge(low)", 0.354], ["fuel_gauge(empty)", 0.116],
      ["plugs(ok)", 0.909], ["plugs(fouled)", 0.091], ["spark(yes)", 0.839170165], ["spark(no)", 0.160829835],
      ["fires(yes)", 0.731336798797], ["fires(no)", 0.268663201202], ["starts(yes)", 0.653079337485], ["starts(no)", 0.346920662515],
    ],
    evidence: 1,
  },
  {
    key: "grid",
    name: "Reliability of a grid network",
    file: "examples/larger/grid_reliability.pl",
    larger: true,
    exact: "a dynamic program over the connectivity of the grid's last column, column by column. ProbLog 2.3.0 answers 5 by 5 routers in 121 s and does not finish 8 by 8 in five minutes.",
    problog: [["reaches(8,8)", 0.085163906983], ["reaches(8,1)", 0.108944533606]],
    evidence: 1,
  },
  {
    key: "dicesum",
    name: "The sum of 100 dice",
    file: "examples/larger/dice_sum.pl",
    larger: true,
    exact: "the distribution of the sum by convolution, in fractions. ProbLog 2.3.0 answers 5 dice in 43 s and does not finish 6 in five minutes.",
    problog: [["prime", 0.163737706468], ["sevens", 0.142857142857], ["high", 0.00182302430809]],
    evidence: 1,
  },
  {
    key: "blood",
    name: "Blood types in a family with loops",
    file: "examples/larger/blood_types.pl",
    larger: true,
    exact: "forward and backward over the generations, through every combination of a generation's genotypes. ProbLog 2.3.0 answers 3 generations of 4 in 1.9 s and does not finish 4 of 4 in five minutes.",
    problog: [
      ["blood(p(1,1),a)", 0.518137535803], ["blood(p(1,1),ab)", 0.0292381634438],
      ["blood(p(1,1),b)", 0.0573358853336], ["blood(p(1,1),o)", 0.39528841542],
      ["has(p(1,1),b)", 0.0865740487775], ["has(p(1,2),b)", 0.0877441576365], ["has(p(1,3),b)", 0.0883633033983],
      ["has(p(1,4),b)", 0.0883633033983], ["has(p(1,5),b)", 0.0877441576365], ["has(p(1,6),b)", 0.0865740487775],
      ["has(p(1,7),b)", 0.084397114572],
    ],
    evidence: 0.00890818390475,
  },
  {
    key: "mastermind",
    name: "Mastermind: 12 pegs, 12 colours",
    file: "examples/larger/mastermind.pl",
    larger: true,
    exact: "a search for the codes the scores allow, 2192 of them. ProbLog 2.3.0 answers 8 pegs of 10 colours after 8 guesses in 15 s and does not finish 10 of 10 after 10 in five minutes.",
    problog: [
      ["code(1,1)", 0], ["code(1,2)", 0], ["code(1,3)", 0], ["code(1,4)", 0], ["code(1,5)", 0], ["code(1,6)", 0],
      ["code(1,7)", 0], ["code(1,8)", 0.267335766423], ["code(1,9)", 0.22901459854], ["code(1,10)", 0.377737226277],
      ["code(1,11)", 0.125912408759], ["code(1,12)", 0],
    ],
    evidence: 2.45847387288e-10,
  },
  {
    key: "stones",
    name: "A game of stones: 4000 heaps",
    file: "examples/larger/stones.pl",
    larger: true,
    exact: "heap by heap over the last three heaps' outcomes, in fractions. ProbLog 2.3.0 answers a heap of 40 in 5.4 s and does not finish 100 in five minutes.",
    problog: [
      ["win(1)", 0.6], ["win(2)", 0.696], ["win(4)", 0.508271616], ["win(5)", 0.546749927424],
      ["win(10)", 0.593404502099], ["win(50)", 0.601997552254], ["win(4000)", 0.601997578692],
    ],
    evidence: 1,
  },
  {
    key: "robot",
    name: "A robot on a ring: 400 readings",
    file: "examples/larger/robot.pl",
    larger: true,
    exact: "the forward algorithm of the hidden Markov model. ProbLog 2.3.0 answers 5 readings in 4.2 s and does not finish 10 in five minutes.",
    problog: [
      ["at(400,1)", 0.000753783329927], ["at(400,2)", 0.0250366111806], ["at(400,3)", 0.0203417719893],
      ["at(400,4)", 0.00296430798699], ["at(400,5)", 0.135799230715], ["at(400,6)", 0.015962539863],
      ["at(400,7)", 0.112224825303], ["at(400,8)", 0.0126669608722], ["at(400,9)", 0.665833407746],
      ["at(400,10)", 0.00098898502858], ["at(400,11)", 0.00225543177311], ["at(400,12)", 0.00517214421171],
    ],
    evidence: 7.27128939706e-104,
  },
];

// Where the site serves an example's program.
function tddlogExamplePath(example) {
  return "examples/" + example.file.split("/").pop();
}

if (typeof module !== "undefined") module.exports = { tddlogExamples, tddlogExamplePath };
