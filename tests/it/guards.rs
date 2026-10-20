//! Derived atoms conditioned on the fact literals that guard their reads
//! (src/ground/guards.rs): circuits of if-then-elses, guarded products and
//! sums whose branches read their selectors again, as a bit-blasting front
//! end builds them, against their answers by enumeration of the worlds of
//! their facts, with the conditioning on and off and under the `decision`
//! vtree it records; and decision lists, whose chains of steps the
//! rewrite composes in balanced trees (src/ground/steps.rs).

use crate::common;

use common::{Lcg, assert_answers};
use tddlog::ground::GroundOptions;
use tddlog::{Kind, Progress, Projection, Strategy};

const FACTS: usize = 9;
const GATES: usize = 40;
const QUERIES: usize = 3;

/// A literal of the circuit: a fact or an earlier gate, positive or not.
#[derive(Clone, Copy)]
enum Lit {
    Fact(usize, bool),
    Gate(usize, bool),
}

impl Lit {
    fn text(self) -> String {
        match self {
            Lit::Fact(i, s) => format!("{}f{i}", if s { "" } else { "\\+" }),
            Lit::Gate(i, s) => format!("{}g{i}", if s { "" } else { "\\+" }),
        }
    }

    fn holds(self, world: usize, gates: &[bool]) -> bool {
        match self {
            Lit::Fact(i, s) => (world >> i & 1 == 1) == s,
            Lit::Gate(i, s) => gates[i] == s,
        }
    }
}

/// A gate: rules whose bodies are literals.
struct Circuit {
    probabilities: Vec<f64>,
    gates: Vec<Vec<Vec<Lit>>>,
    evidence: usize,
    queries: Vec<usize>,
}

impl Circuit {
    fn random(seed: u64) -> Self {
        Self::sized(seed, FACTS, GATES, 6)
    }

    /// A random circuit of `gates` gates over `facts` facts, whose gates
    /// read the `window` gates before them.
    fn sized(seed: u64, facts: usize, gates_n: usize, window: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut pick = |n: usize| ((rng.next() * n as f64) as usize).min(n - 1);
        let probabilities: Vec<f64> = (0..facts)
            .map(|i| 0.1 + 0.08 * ((i * 7 + seed as usize) % 10) as f64)
            .collect();
        let mut gates: Vec<Vec<Vec<Lit>>> = Vec::new();
        for g in 0..gates_n {
            // Mostly earlier gates, so that selectors are read again below.
            let operand = |pick: &mut dyn FnMut(usize) -> usize| {
                let sign = pick(4) != 0;
                if g >= 2 && pick(4) != 0 {
                    Lit::Gate(g - 1 - pick(g.min(window)), sign)
                } else {
                    Lit::Fact(pick(facts), sign)
                }
            };
            let s = pick(facts);
            let x = operand(&mut pick);
            let y = operand(&mut pick);
            gates.push(match pick(3) {
                // if f_s then x else y
                0 => vec![vec![Lit::Fact(s, true), x], vec![Lit::Fact(s, false), y]],
                // f_s and x and y
                1 => vec![vec![Lit::Fact(s, pick(2) == 0), x, y]],
                // x or (not f_s and y)
                _ => vec![vec![x], vec![Lit::Fact(s, false), y]],
            });
        }
        let mut c = Circuit {
            probabilities,
            gates,
            evidence: gates_n - QUERIES - 1,
            queries: (gates_n - QUERIES..gates_n).collect(),
        };
        // The last gate before the queries that some world makes true.
        while (0..1 << facts).all(|w| !c.eval(w)[c.evidence]) {
            c.evidence -= 1;
        }
        c
    }

    fn eval(&self, world: usize) -> Vec<bool> {
        let mut gates = Vec::with_capacity(self.gates.len());
        for rules in &self.gates {
            let v = rules
                .iter()
                .any(|body| body.iter().all(|l| l.holds(world, &gates)));
            gates.push(v);
        }
        gates
    }

    fn source(&self) -> String {
        let mut s = String::new();
        for (i, p) in self.probabilities.iter().enumerate() {
            s += &format!("{p}::f{i}.\n");
        }
        for (g, rules) in self.gates.iter().enumerate() {
            for body in rules {
                s += &format!(
                    "g{g} :- {}.\n",
                    body.iter().map(|l| l.text()).collect::<Vec<_>>().join(", ")
                );
            }
        }
        s += &format!("evidence(g{}).\n", self.evidence);
        for q in &self.queries {
            s += &format!("query(g{q}).\n");
        }
        s
    }

    /// Each query's probability given the evidence, by enumeration.
    fn answers(&self) -> Vec<(String, f64)> {
        let mut z = 0.0;
        let mut joint = vec![0.0; self.queries.len()];
        for w in 0..1usize << self.probabilities.len() {
            let p: f64 = self
                .probabilities
                .iter()
                .enumerate()
                .map(|(i, &p)| if w >> i & 1 == 1 { p } else { 1.0 - p })
                .product();
            let gates = self.eval(w);
            if !gates[self.evidence] {
                continue;
            }
            z += p;
            for (k, &q) in self.queries.iter().enumerate() {
                if gates[q] {
                    joint[k] += p;
                }
            }
        }
        self.queries
            .iter()
            .zip(joint)
            .map(|(q, j)| (format!("g{q}"), j / z))
            .collect()
    }
}

/// The program as ground for `decision`, conditioned on its guards, or as
/// ground by default.
fn ground(source: &str, guards: bool) -> tddlog::GroundProgram {
    let program = tddlog::syntax::parse_program(source).unwrap();
    if !guards {
        return tddlog::ground::ground(&program, &GroundOptions::default()).unwrap();
    }
    let options = tddlog::Options::with(Strategy::Trial(Kind::Decision)).ground_options();
    Kind::Decision.rewrite(tddlog::ground::ground(&program, &options).unwrap())
}

#[test]
fn circuits_conditioned_on_their_guards_keep_their_answers() {
    let (mut conditioned, mut families) = (0, 0);
    for seed in 0..24 {
        let circuit = Circuit::random(seed);
        let source = circuit.source();
        let expected = circuit.answers();
        let check = |what: &str, run: Result<tddlog::Run, tddlog::Error>| {
            let run = run.unwrap_or_else(|e| panic!("seed {seed}, {what}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {what}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        };
        // Every layout keeps the answers on the program as ground and on
        // the conditioned program, which `auto`'s trials may compile.
        for strategy in [
            Strategy::Trial(Kind::Closed),
            Strategy::Auto,
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Trial(Kind::Program { fan_in: true }),
            Strategy::Trial(Kind::Td),
            Strategy::Trial(Kind::Families { search: false }),
            Strategy::Trial(Kind::Families { search: true }),
        ] {
            let options = common::options(strategy, Projection::default());
            for (what, run) in [
                (format!("{strategy}"), common::run(&source, &options)),
                (
                    format!("{strategy}, guarded"),
                    tddlog::solve_ground(ground(&source, true), &options),
                ),
            ] {
                match run {
                    // `families` refuses a program it reads no atom of
                    // through an indicator, and `auto` skips the trial.
                    Err(tddlog::Error::Vtree(e)) if e.contains("reads no atom through one") => {
                        continue;
                    }
                    run => {
                        check(&what, run);
                        families +=
                            usize::from(matches!(strategy, Strategy::Trial(Kind::Families { .. })));
                    }
                }
            }
        }
        if ground(&source, true).stats.decisions.is_some() {
            conditioned += 1;
            let decision = common::options(Strategy::Trial(Kind::Decision), Projection::default());
            check("decision", common::run(&source, &decision));
            // A program conditioned already is compiled as it is.
            check(
                "decision, guarded",
                tddlog::solve_ground(ground(&source, true), &decision),
            );
        } else {
            assert!(
                common::run(
                    &source,
                    &common::options(Strategy::Trial(Kind::Decision), Projection::default())
                )
                .is_err(),
                "seed {seed}: `decision` without decisions"
            );
        }
        assert!(ground(&source, false).stats.decisions.is_none());
    }
    assert!(
        conditioned >= 12,
        "only {conditioned} of 24 circuits had their atoms conditioned on guards"
    );
    assert!(
        families > 0,
        "`families` read no circuit through indicators"
    );
}

/// Circuits past the size from which `auto` grounds a program again for a
/// `decision` trial (4 096 derived atoms, ground with views inlined), most
/// of them conditioned on their guards into more atoms than they have
/// derived: `auto` tries `decision` on those first, on the conditioned
/// program, which `program`, `closed` and `td` compile on the others, and
/// its answers are the enumeration's either way.
#[test]
fn auto_tries_decision_on_large_circuits_conditioned_on_their_guards() {
    let mut decided = 0;
    for seed in 1..=3 {
        let circuit = Circuit::sized(seed, 12, 12_000, 200);
        let source = circuit.source();
        let expected = circuit.answers();
        let program = tddlog::syntax::parse_program(&source).unwrap();
        let gp =
            tddlog::ground::ground(&program, &tddlog::Options::default().ground_options()).unwrap();
        assert!(
            gp.num_derived() >= 4096,
            "seed {seed}: {} derived atoms",
            gp.num_derived()
        );
        // `solve` grounds a program that `auto` may decide again for the
        // trial, and says whether it keeps one.
        let (mut trial, mut kind, mut conditioned, mut parts) = (None, None, None, 0);
        let solution =
            tddlog::solve(
                &source,
                &tddlog::Options::default(),
                &mut |progress| match progress {
                    Progress::Note(note) if note.starts_with("decision:") => {
                        trial = Some(note.contains(", a trial,"))
                    }
                    Progress::Answered(run) => {
                        (kind, conditioned) =
                            (Some(run.kind), Some(run.ground.stats.decisions.is_some()));
                        parts += 1;
                    }
                    _ => {}
                },
            )
            .unwrap();
        assert_answers(
            &format!("seed {seed}"),
            &solution.answers.expect("evidence is consistent"),
            &expected,
            1e-12,
        );
        let (noted, trial, kind) = (trial.is_some(), trial.unwrap_or(false), kind.unwrap());
        // `decision` answers where it is a trial, and the winner compiled
        // the conditioned program where the conditioning recorded decisions
        // on the whole program, if it is `program`, `closed` or `td`, whose
        // layouts the conditioning cannot mislead (`auto::Programs`).
        assert_eq!(kind == Kind::Decision, trial, "seed {seed}: {kind:?}");
        let recorded = noted && parts == 1 && ground(&source, true).stats.decisions.is_some();
        let takes_it = matches!(
            kind,
            Kind::Decision | Kind::Program { fan_in: false } | Kind::Closed | Kind::Td
        );
        assert_eq!(
            conditioned,
            Some(recorded && takes_it),
            "seed {seed}: {kind:?}"
        );
        if trial {
            decided += 1;
        }
    }
    assert!(decided >= 1, "no circuit had a `decision` trial");
}

/// A mux tree as Alea.jl builds one (coinBias, small): `decision` names
/// the contexts of the decisions' tree that rule bodies hold, and the bits
/// of the selected value that the comparators beside the tree read
/// (src/vtree/decision.rs). ProbLog's answers (ProbLog 2.3.0, one query
/// at a time).
#[test]
fn decision_names_the_contexts_and_values_of_a_mux_tree() {
    let source = std::fs::read_to_string("tests/regress/coin_bias_mux.pl").unwrap();
    let decision = common::options(Strategy::Trial(Kind::Decision), Projection::default());
    let run = common::run(&source, &decision).unwrap();
    let named: Vec<String> = run
        .ground
        .encoding
        .indicators
        .iter()
        .map(|i| run.ground.atoms[i.atom as usize].to_string())
        .collect();
    let contexts = named.iter().filter(|a| a.contains("$cx")).count();
    assert!(contexts >= 7, "{named:?}");
    assert!(named.len() > contexts, "no value bit named: {named:?}");
    assert_answers(
        "decision",
        run.results.as_ref().unwrap(),
        &[
            ("b(0)", 0.36526305),
            ("b(1)", 0.70025614),
            ("b(2)", 0.49552234),
            ("b(3)", 0.47751103),
        ],
        1e-6,
    );
}

/// The same tree, whose bias bits are each read alone with a flip outside
/// it: composed, the readers grow less than the joint of the bits' links,
/// so settling expands the tree's cut (`tp::settle`), and the answers are
/// ProbLog 2.3.0's.
#[test]
fn decision_expands_names_that_do_not_pay() {
    let source = std::fs::read_to_string("tests/regress/decision_bits_read_alone.pl").unwrap();
    let decision = common::options(Strategy::Trial(Kind::Decision), Projection::default());
    let run = common::run(&source, &decision).unwrap();
    assert_eq!(
        (run.query_sizes.names_kept, run.query_sizes.names_expanded),
        (0, 1),
        "the tree's cut is expanded"
    );
    assert_answers(
        "decision",
        run.results.as_ref().unwrap(),
        &[
            ("b(0)", 0.11788744),
            ("b(1)", 0.49437378),
            ("b(2)", 0.57050766),
            ("b(3)", 0.60458152),
        ],
        1e-6,
    );
}

/// An if-then-else whose branches read their selector again: under the
/// selector each branch reads one fact, and the rules read facts alone.
#[test]
fn branches_lose_the_selector_they_read() {
    let source = "0.3::s. 0.6::a. 0.2::b. 0.5::c.
        m :- s, a. m :- \\+s, b.
        n :- m, c. n :- \\+m, \\+c.
        t :- s, n. t :- \\+s, \\+n.
        query(t).";
    let program = tddlog::syntax::parse_program(source).unwrap();
    let options = GroundOptions {
        views: false,
        ..GroundOptions::default()
    };
    let gp = Kind::Decision.rewrite(tddlog::ground::ground(&program, &options).unwrap());
    // n under s and under not s, each over a and c or b and c.
    assert_eq!(gp.stats.guarded, 2, "{gp}");
    let text = gp.to_string();
    assert!(
        !text.contains(":- m") && !text.contains(", m") && !text.contains("\\+m"),
        "{text}"
    );
    // P(t) = P(s) P(n | s) + P(not s) P(not n | not s), with
    // n | s = a <-> c and n | not s = b <-> c.
    let (s, a, b, c) = (0.3, 0.6, 0.2, 0.5);
    let n_s = a * c + (1.0 - a) * (1.0 - c);
    let n_not_s = b * c + (1.0 - b) * (1.0 - c);
    let expected = [("t", s * n_s + (1.0 - s) * (1.0 - n_not_s))];
    for strategy in [
        Strategy::Trial(Kind::Decision),
        Strategy::Trial(Kind::Closed),
    ] {
        let run = tddlog::solve_ground(
            gp.clone(),
            &common::options(strategy, Projection::default()),
        )
        .unwrap();
        assert_answers(
            &format!("{strategy}"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

impl Circuit {
    /// An if-then-else whose branch reads its selector again, so that the
    /// conditioning applies, and a decision list of `steps` steps
    /// `l_k :- l_{k-1}, \+h_k. l_k :- c_k.`, `h_k` and `c_k` cubes of
    /// two fact literals, `h_k` a gate of its own; the queries are the
    /// branch, which a query keeps from being inlined, the if-then-else,
    /// the list's end and a step `cut` of the list, which splits it in two
    /// chains.
    fn decision_list(seed: u64, steps: usize, cut: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut pick = |n: usize| ((rng.next() * n as f64) as usize).min(n - 1);
        let probabilities: Vec<f64> = (0..FACTS)
            .map(|i| 0.1 + 0.08 * ((i * 5 + seed as usize) % 10) as f64)
            .collect();
        let cube = |pick: &mut dyn FnMut(usize) -> usize| {
            let a = pick(FACTS);
            let b = (a + 1 + pick(FACTS - 1)) % FACTS;
            vec![Lit::Fact(a, pick(2) == 0), Lit::Fact(b, pick(2) == 0)]
        };
        // g0 reads f0; g1 reads f0 and g0 under it.
        let mut gates = vec![
            vec![
                vec![Lit::Fact(0, true), Lit::Fact(1, true)],
                vec![Lit::Fact(2, true)],
            ],
            vec![
                vec![Lit::Fact(0, true), Lit::Gate(0, true)],
                vec![Lit::Fact(0, false), Lit::Fact(3, true)],
            ],
            // The evidence: some world makes it true.
            vec![vec![Lit::Fact(4, true)], vec![Lit::Fact(5, false)]],
        ];
        let start = gates.len();
        gates.push(vec![cube(&mut pick)]);
        let (mut prev, mut cut_at) = (start, start);
        for k in 1..=steps {
            let h = gates.len();
            gates.push(vec![cube(&mut pick)]);
            gates.push(vec![
                vec![Lit::Gate(prev, true), Lit::Gate(h, false)],
                cube(&mut pick),
            ]);
            prev = gates.len() - 1;
            if k == cut {
                cut_at = prev;
            }
        }
        let end = gates.len() - 1;
        Circuit {
            probabilities,
            gates,
            evidence: 2,
            queries: vec![0, 1, end, cut_at],
        }
    }
}

#[test]
fn decision_lists_composed_in_trees_keep_their_answers() {
    for seed in 0..12 {
        let circuit = Circuit::decision_list(seed, 9 + seed as usize % 7, 5);
        let source = circuit.source();
        let expected = circuit.answers();
        let gp = ground(&source, true);
        assert!(gp.stats.stepped > 0, "seed {seed}: no chain composed\n{gp}");
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Closed),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Trial(Kind::Program { fan_in: true }),
        ] {
            let run = tddlog::solve_ground(
                gp.clone(),
                &common::options(strategy, Projection::default()),
            )
            .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
        let run = common::run(
            &source,
            &common::options(Strategy::Trial(Kind::Decision), Projection::default()),
        )
        .unwrap();
        assert_answers(
            &format!("seed {seed}, run"),
            &run.results.unwrap(),
            &expected,
            1e-12,
        );
    }
}

/// A node of a circuit under construction: a constant or a literal.
#[derive(Clone, Copy)]
enum Node {
    True,
    False,
    Lit(Lit),
}

fn not(n: Node) -> Node {
    match n {
        Node::True => Node::False,
        Node::False => Node::True,
        Node::Lit(Lit::Fact(i, s)) => Node::Lit(Lit::Fact(i, !s)),
        Node::Lit(Lit::Gate(i, s)) => Node::Lit(Lit::Gate(i, !s)),
    }
}

impl Circuit {
    /// A gate of `bodies`, folded where a constant decides it, as Alea
    /// folds them.
    fn gate(&mut self, bodies: &[&[Node]]) -> Node {
        let mut rules = Vec::new();
        for body in bodies {
            if body.iter().any(|n| matches!(n, Node::False)) {
                continue;
            }
            let lits: Vec<Lit> = body
                .iter()
                .filter_map(|n| if let Node::Lit(l) = n { Some(*l) } else { None })
                .collect();
            if lits.is_empty() {
                return Node::True;
            }
            rules.push(lits);
        }
        match rules.as_slice() {
            [] => Node::False,
            [one] if one.len() == 1 => Node::Lit(one[0]),
            _ => {
                self.gates.push(rules);
                Node::Lit(Lit::Gate(self.gates.len() - 1, true))
            }
        }
    }

    fn ite(&mut self, s: Node, a: Node, b: Node) -> Node {
        self.gate(&[&[s, a], &[b, not(s)]])
    }

    fn flip(&mut self, p: f64) -> Node {
        self.probabilities.push(p);
        Node::Lit(Lit::Fact(self.probabilities.len() - 1, true))
    }

    /// A coin's bias as Alea bit-blasts coinBias, small: a piecewise
    /// number ([`Circuit::piecewise`]) and one toss `u < bias` over fresh
    /// flips, observed true. The queries are the bias's bits that are
    /// gates.
    fn coin_bias(seed: u64, k: usize, l: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let bias = c.piecewise(&mut rng, k, l);
        c.toss(&bias);
        c
    }

    /// A number as Alea bit-blasts a piecewise distribution, small: a
    /// `k`-bit index drawn by a tree of flips (bit d a mux over d's flips
    /// that the bits above select), the number's high bits the index and
    /// its `l` low bits flips of the piece the index selects, chosen by a
    /// decision list over the pieces.
    fn piecewise(&mut self, rng: &mut Lcg, k: usize, l: usize) -> Vec<Node> {
        let index = self.index(rng, k);
        let mut number = vec![Node::False; k + l];
        for i in (0..1usize << k).rev() {
            let eq = self.equals(&index, i);
            let mut piece = Circuit::constant(k, i);
            piece.extend((0..l).map(|_| self.flip(0.2 + 0.6 * rng.next())));
            number = piece
                .iter()
                .zip(&number)
                .map(|(&x, &y)| self.ite(eq, x, y))
                .collect::<Vec<_>>();
        }
        number
    }

    /// Alea.jl's genes, small: `t` genes, each occurring where two flips
    /// hold, and a reading that is a piecewise number where any gene
    /// occurs and fair flips where none does, observed equal to a
    /// constant. The queries are the genes. Returns the circuit and the
    /// gate `any`, the or of the genes.
    fn genes(seed: u64, k: usize, l: usize, t: usize) -> (Self, usize) {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let high = c.piecewise(&mut rng, k, l);
        let low: Vec<Node> = (0..k + l).map(|_| c.flip(0.5)).collect();
        let genes: Vec<Node> = (0..t)
            .map(|_| {
                let (p, u) = (c.flip(0.2 + 0.6 * rng.next()), c.flip(0.5));
                c.gate(&[&[p, u]])
            })
            .collect();
        let bodies: Vec<[Node; 1]> = genes.iter().map(|&g| [g]).collect();
        let refs: Vec<&[Node]> = bodies.iter().map(|b| &b[..]).collect();
        let Node::Lit(Lit::Gate(any, true)) = c.gate(&refs) else {
            panic!("the genes folded")
        };
        let reading: Vec<Node> = high
            .iter()
            .zip(&low)
            .map(|(&h, &w)| c.ite(Node::Lit(Lit::Gate(any, true)), h, w))
            .collect();
        let v = ((rng.next() * (1 << (k + l)) as f64) as usize).min((1 << (k + l)) - 1);
        let Node::Lit(Lit::Gate(e, true)) = c.equals(&reading, v) else {
            panic!("the observation folded")
        };
        c.evidence = e;
        c.queries = genes
            .iter()
            .filter_map(|n| {
                if let Node::Lit(Lit::Gate(g, true)) = n {
                    Some(*g)
                } else {
                    None
                }
            })
            .collect();
        (c, any)
    }

    /// A number as Alea.jl bit-blasts conjugate_ex3's Gaussians, small: a
    /// `k`-bit index drawn by a tree of flips, as in coinBias, a slope
    /// flip for each piece, made after the index, and `l` low bits that
    /// the pieces share, made last: the uniform's flips and the
    /// triangle's. The number's high bits are the index, and its low bit
    /// `j` is the uniform's or the triangle's by the slope of the piece
    /// the index selects, `ite(s_i, u_j, t_j)` in a decision list over the
    /// pieces. One toss `x < number` over fresh flips is observed true.
    /// The queries are the number's bits that are gates.
    fn slopes(seed: u64, k: usize, l: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let number = c.number(&mut rng, k, l);
        c.toss(&number);
        c
    }

    /// A mean and `n` noises, numbers drawn as in [`Circuit::slopes`] one
    /// after the other, and the sum of the mean and each noise observed
    /// equal to a constant, as Alea.jl's conjugate_ex3 observes two: a
    /// ripple-carry sum of Alea's gates (each bit's and, or and exclusive
    /// or of the two numbers' bits; the sum's bit the exclusive or of that
    /// with the carry, the carry out at least two of the three) whose last
    /// carry is observed false. The queries are the mean's bits that are
    /// gates.
    fn sums_observed(seed: u64, k: usize, l: usize, n: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let a = c.number(&mut rng, k, l);
        let mut seen = Vec::with_capacity(2 * n);
        for _ in 0..n {
            let b = c.number(&mut rng, k, l);
            let (sum, carry) = c.sum(&a, &b);
            let v = ((rng.next() * (1 << (k + l)) as f64) as usize).min((1 << (k + l)) - 1);
            seen.push(c.equals(&sum, v));
            seen.push(not(carry));
        }
        let Node::Lit(Lit::Gate(e, true)) = c.gate(&[&seen]) else {
            panic!("the observations folded")
        };
        c.evidence = e;
        c.queries = a
            .iter()
            .filter_map(|n| {
                if let Node::Lit(Lit::Gate(g, true)) = n {
                    Some(*g)
                } else {
                    None
                }
            })
            .collect();
        c
    }

    /// The ripple-carry sum of `a` and `b`, most significant bit first, of
    /// Alea's gates: each bit's and, or and exclusive or of the two
    /// numbers' bits, the sum's bit the exclusive or of that with the
    /// carry, the carry out at least two of the three. Returns the sum's
    /// bits and the last carry.
    fn sum(&mut self, a: &[Node], b: &[Node]) -> (Vec<Node>, Node) {
        let mut sum = vec![Node::False; a.len()];
        let mut carry = Node::False;
        for j in (0..a.len()).rev() {
            let (x, y) = (a[j], b[j]);
            let both = self.gate(&[&[x, y]]);
            let either = self.gate(&[&[x], &[y]]);
            let one = self.gate(&[&[x, not(y)], &[not(x), y]]);
            sum[j] = self.gate(&[&[one, not(carry)], &[not(one), carry]]);
            carry = self.gate(&[&[both], &[carry, either]]);
        }
        (sum, carry)
    }

    /// Alea.jl's altermu2, small: two uniform means of `k + l` bits and
    /// their sum `mu`, and `n` draws, each a number as
    /// [`Circuit::number`] draws it, with each draw plus `mu` (mod
    /// 2^(k+l)) observed equal to a constant. The queries are `mu`'s bits
    /// that are gates.
    fn means_observed(seed: u64, k: usize, l: usize, n: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let mu1: Vec<Node> = (0..k + l).map(|_| c.flip(0.5)).collect();
        let mu2: Vec<Node> = (0..k + l).map(|_| c.flip(0.5)).collect();
        let (mu, _) = c.sum(&mu1, &mu2);
        let mut seen = Vec::with_capacity(n);
        for _ in 0..n {
            let draw = c.number(&mut rng, k, l);
            let (y, _) = c.sum(&draw, &mu);
            let v = ((rng.next() * (1 << (k + l)) as f64) as usize).min((1 << (k + l)) - 1);
            seen.push(c.equals(&y, v));
        }
        let Node::Lit(Lit::Gate(e, true)) = c.gate(&[&seen]) else {
            panic!("the observations folded")
        };
        c.evidence = e;
        c.queries = mu
            .iter()
            .filter_map(|n| {
                if let Node::Lit(Lit::Gate(g, true)) = n {
                    Some(*g)
                } else {
                    None
                }
            })
            .collect();
        c
    }

    /// A Caesar cipher as Alea.jl bit-blasts caesar, small: a key and `n`
    /// letters, each a `k`-bit index drawn by a tree of flips, and each
    /// letter plus the key (mod 2^k, a ripple-carry sum) observed equal to
    /// a constant. The queries are the key's bits that are gates.
    fn caesar(seed: u64, k: usize, n: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let key = c.index(&mut rng, k);
        let mut seen = Vec::with_capacity(n);
        for _ in 0..n {
            let letter = c.index(&mut rng, k);
            let mut sum = vec![Node::False; k];
            let mut carry = Node::False;
            for j in (0..k).rev() {
                let (a, b) = (key[j], letter[j]);
                let x = c.gate(&[&[a, not(b)], &[not(a), b]]);
                sum[j] = c.gate(&[&[x, not(carry)], &[not(x), carry]]);
                carry = c.gate(&[&[a, b], &[a, carry], &[b, carry]]);
            }
            let v = ((rng.next() * (1 << k) as f64) as usize).min((1 << k) - 1);
            seen.push(c.equals(&sum, v));
        }
        let Node::Lit(Lit::Gate(e, true)) = c.gate(&[&seen]) else {
            panic!("the observations folded")
        };
        c.evidence = e;
        c.queries = key
            .iter()
            .filter_map(|n| {
                if let Node::Lit(Lit::Gate(g, true)) = n {
                    Some(*g)
                } else {
                    None
                }
            })
            .collect();
        c
    }

    /// A number chosen by an else-chain, as genes_t's selector trees
    /// choose a gene's value: the `i`-th of `n` pieces of `k` flips where
    /// the `i`-th selector holds and those before it do not, else a
    /// `k`-bit index drawn by a tree of flips, the branch that most reads
    /// are under. The selectors are made last, so that each lies within
    /// the facts of the branches it selects between, as the guards pass
    /// asks. One toss `x < number` over fresh flips is observed true. The
    /// queries are the number's bits that are gates.
    fn else_chain(seed: u64, n: usize, k: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut c = Circuit::empty();
        let pieces: Vec<Vec<Node>> = (0..n)
            .map(|_| (0..k).map(|_| c.flip(0.2 + 0.6 * rng.next())).collect())
            .collect();
        let mut number = c.index(&mut rng, k);
        let select: Vec<Node> = (0..n).map(|_| c.flip(0.2 + 0.6 * rng.next())).collect();
        for i in (0..n).rev() {
            number = pieces[i]
                .iter()
                .zip(&number)
                .map(|(&x, &y)| c.ite(select[i], x, y))
                .collect();
        }
        c.toss(&number);
        c
    }

    fn empty() -> Self {
        Circuit {
            probabilities: Vec::new(),
            gates: Vec::new(),
            evidence: 0,
            queries: Vec::new(),
        }
    }

    /// A `k`-bit index drawn by a tree of flips, as Alea.jl's `discrete`
    /// draws one: bit `d` a mux over `d`'s flips that the bits above
    /// select, most significant first.
    fn index(&mut self, rng: &mut Lcg, k: usize) -> Vec<Node> {
        let mut index: Vec<Node> = Vec::new();
        for d in 0..k {
            let mut level: Vec<Node> = (0..1 << d)
                .map(|_| self.flip(0.2 + 0.6 * rng.next()))
                .collect();
            for s in index.iter().rev() {
                level = level.chunks(2).map(|p| self.ite(*s, p[0], p[1])).collect();
            }
            index.push(level[0]);
        }
        index
    }

    /// A number as [`Circuit::slopes`] draws it: a `k`-bit index, a slope
    /// flip for each piece and `l` low bits of each kind that the pieces
    /// share, and the number's low bit `j` `ite(s_i, u_j, t_j)` by the
    /// slope of the piece the index selects, in a decision list over the
    /// pieces. Its bits, most significant first.
    fn number(&mut self, rng: &mut Lcg, k: usize, l: usize) -> Vec<Node> {
        let index = self.index(rng, k);
        let slope: Vec<Node> = (0..1 << k)
            .map(|_| self.flip(0.2 + 0.6 * rng.next()))
            .collect();
        let u: Vec<Node> = (0..l).map(|_| self.flip(0.5)).collect();
        let t: Vec<Node> = (0..l).map(|_| self.flip(0.2 + 0.6 * rng.next())).collect();
        let mut number = vec![Node::False; k + l];
        for i in (0..1usize << k).rev() {
            let eq = self.equals(&index, i);
            let mut piece = Circuit::constant(k, i);
            for j in 0..l {
                let low = self.ite(slope[i], u[j], t[j]);
                piece.push(low);
            }
            number = piece
                .iter()
                .zip(&number)
                .map(|(&x, &y)| self.ite(eq, x, y))
                .collect::<Vec<_>>();
        }
        number
    }

    /// Whether the index is `i`, a gate of one body.
    fn equals(&mut self, index: &[Node], i: usize) -> Node {
        let k = index.len();
        let cube: Vec<Node> = index
            .iter()
            .enumerate()
            .map(|(j, &b)| if i >> (k - 1 - j) & 1 == 1 { b } else { not(b) })
            .collect();
        self.gate(&[&cube])
    }

    /// The `k` bits of `i`, most significant first.
    fn constant(k: usize, i: usize) -> Vec<Node> {
        (0..k)
            .map(|j| {
                if i >> (k - 1 - j) & 1 == 1 {
                    Node::True
                } else {
                    Node::False
                }
            })
            .collect()
    }

    /// One toss `x < value` over fresh fair flips, observed, and the
    /// value's bits that are gates queried.
    fn toss(&mut self, value: &[Node]) {
        let x: Vec<Node> = value.iter().map(|_| self.flip(0.5)).collect();
        let mut lt = Node::False;
        for (&xi, &vi) in x.iter().zip(value).rev() {
            let eq = self.gate(&[&[xi, vi], &[not(xi), not(vi)]]);
            lt = self.gate(&[&[not(xi), vi], &[eq, lt]]);
        }
        let Node::Lit(Lit::Gate(e, true)) = lt else {
            panic!("the toss folded")
        };
        self.evidence = e;
        self.queries = value
            .iter()
            .filter_map(|n| {
                if let Node::Lit(Lit::Gate(g, true)) = n {
                    Some(*g)
                } else {
                    None
                }
            })
            .collect();
    }
}

/// A mux tree read by a comparator, as in coinBias: under `decision` the
/// index's flips decide between the pieces and the toss's flips decide
/// nothing; the answers are the enumeration's under it and the others.
#[test]
fn a_mux_tree_read_by_a_comparator_keeps_its_answers() {
    for seed in 0..2 {
        let circuit = Circuit::coin_bias(seed, 3, 1);
        assert_eq!(circuit.probabilities.len(), 19);
        let source = circuit.source();
        let expected = circuit.answers();
        assert!(
            ground(&source, true).stats.decisions.is_some(),
            "seed {seed}"
        );
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}

/// Alea.jl's genes, small: every bit of the reading reads `any`, the or of
/// the genes, beside the bits of the number that the decisions' tree
/// selects. `decision` names `any` for those readers (src/vtree/decision.rs);
/// expanded, its formula was in each reader of the tree, and settling built
/// it again with each composition. The answers are the enumeration's under
/// it and the others.
#[test]
fn decision_names_what_its_trees_readers_read_beside_it() {
    for seed in 0..2 {
        let (circuit, any) = Circuit::genes(seed, 3, 1, 2);
        assert_eq!(circuit.probabilities.len(), 23);
        let source = circuit.source();
        let expected = circuit.answers();
        assert!(
            ground(&source, true).stats.decisions.is_some(),
            "seed {seed}"
        );
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            if strategy == Strategy::Trial(Kind::Decision) {
                let named: Vec<String> = run
                    .ground
                    .encoding
                    .indicators
                    .iter()
                    .map(|i| run.ground.atoms[i.atom as usize].to_string())
                    .collect();
                assert!(
                    named.contains(&format!("g{any}")),
                    "seed {seed}: g{any} not named: {named:?}"
                );
            }
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}

/// Alea.jl's altermu2, small: each observation reads the bits of `mu`, the
/// sum of two means, beside the bits of a draw that the decisions' tree
/// selects. The bits' cones share the means' flips through the carries,
/// so `decision` does not name them (src/vtree/decision.rs): named, the
/// compile's largest formula grew 50-fold at 11 bits. The answers are the
/// enumeration's under it and the others.
#[test]
fn decision_names_nothing_beside_its_tree_that_shares_a_cone() {
    for seed in 0..2 {
        let circuit = Circuit::means_observed(seed, 2, 1, 1);
        assert_eq!(circuit.probabilities.len(), 15);
        let source = circuit.source();
        let expected = circuit.answers();
        assert!(
            ground(&source, true).stats.decisions.is_some(),
            "seed {seed}"
        );
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            if strategy == Strategy::Trial(Kind::Decision) {
                let named: Vec<String> = run
                    .ground
                    .encoding
                    .indicators
                    .iter()
                    .map(|i| run.ground.atoms[i.atom as usize].to_string())
                    .collect();
                for q in &circuit.queries {
                    assert!(
                        !named.contains(&format!("g{q}")),
                        "seed {seed}: g{q} named: {named:?}"
                    );
                }
            }
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}

/// altermu2 with several data points: under `decision` the means' flips
/// are the vtree's top chain, and each data point's observation, split
/// bit by bit by the evidence's conditioning, reads the chain beside its
/// own draw. The evidence walk groups a batch's factors by the draw they
/// read (src/query/walk.rs, `by_part`), where by the chain node they were
/// put off at it paired the same bit of different data points. The
/// answers are the enumeration's under it and the others.
#[test]
fn observations_of_several_draws_keep_their_answers_grouped_by_draw() {
    for seed in 0..2 {
        let circuit = Circuit::means_observed(seed, 1, 1, 3);
        assert!(
            circuit.probabilities.len() <= 22,
            "{}",
            circuit.probabilities.len()
        );
        let source = circuit.source();
        let expected = circuit.answers();
        assert!(
            ground(&source, true).stats.decisions.is_some(),
            "seed {seed}"
        );
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}

/// A number chosen by an else-chain whose last branch most reads are
/// under, as genes_t's selector trees: under `decision` each selector
/// after the first decides below those before it (src/ground/guards.rs),
/// and the answers are the enumeration's under it and the others.
#[test]
fn an_else_chain_keeps_its_answers() {
    for seed in 0..2 {
        let circuit = Circuit::else_chain(seed, 3, 2);
        assert_eq!(circuit.probabilities.len(), 14);
        let source = circuit.source();
        let expected = circuit.answers();
        assert!(
            ground(&source, true).stats.decisions.is_some(),
            "seed {seed}"
        );
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}

/// A number whose pieces' slopes select the low bits they share, as in
/// Alea.jl's conjugate_ex3: each slope flip is read only by an
/// if-then-else of the shared low bits, whose facts lie after the index's,
/// under the piece's index, which a decision list reads through its own
/// definition. `decision` puts each slope under its piece's index, the
/// context its reader is read in; on the root's spine with the shared low
/// bits, above the index's tree, a formula of the decision list held one
/// state for each combination of the slopes above it (k = 6: 2^64). The
/// answers are the enumeration's, and the formulas stay small.
#[test]
fn slopes_sit_under_the_index_their_readers_are_read_under() {
    for seed in 0..2 {
        let circuit = Circuit::slopes(seed, 3, 1);
        assert_eq!(circuit.probabilities.len(), 21);
        let source = circuit.source();
        let expected = circuit.answers();
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
    let source = Circuit::slopes(0, 6, 2).source();
    let decision = common::options(Strategy::Trial(Kind::Decision), Projection::default());
    let run = common::run(&source, &decision).unwrap();
    let largest = run.stats.max_pairs;
    assert!(largest < 20_000, "largest formula {largest} pairs");
}

/// A mean and noises drawn as in conjugate_ex3, the sum of the mean and
/// each noise observed equal to a constant, as Alea.jl's conjugate_ex3
/// observes two: the gates of each bit of a sum read a bit of each of two
/// numbers, whose trees of decisions are apart, and the low flips of each
/// number, which the decisions leave out, are read beside its tree only.
/// `decision` puts them above their number's tree (src/vtree/decision.rs);
/// on the root's spine, the gates of the sums sat at the root, and the
/// query walk joined their names' links there beside every value of the
/// three numbers (k = 4, l = 2: 21 669 pairs, 3 186 above). The answers
/// are the enumeration's, and the walk's formulas stay small.
#[test]
fn facts_the_tree_leaves_out_sit_above_the_part_that_reads_them() {
    for (seed, k, l, n, flips) in [
        (0, 2, 1, 1, 18),
        (1, 2, 1, 1, 18),
        (0, 1, 1, 2, 15),
        (2, 1, 1, 2, 15),
        (3, 1, 1, 2, 15),
    ] {
        let circuit = Circuit::sums_observed(seed, k, l, n);
        assert_eq!(circuit.probabilities.len(), flips);
        let source = circuit.source();
        let expected = circuit.answers();
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, n {n}, {strategy}: {e}\n{source}"));
            assert_answers(
                &format!("seed {seed}, n {n}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
    let source = Circuit::sums_observed(0, 4, 2, 2).source();
    let decision = common::options(Strategy::Trial(Kind::Decision), Projection::default());
    let run = common::run(&source, &decision).unwrap();
    let largest = run.query_sizes.largest;
    assert!(
        largest < 6_000,
        "largest formula of the walk {largest} pairs"
    );
}

/// A Caesar cipher as in Alea.jl's caesar: the key and the letters are
/// trees of decisions, and the sums read nothing outside them. `decision`
/// names nothing there (src/vtree/decision.rs): the contexts' names serve
/// the readers of the tree outside it, and alone they took caesar_huge's
/// compile from 0.45 G to 5.4 G work units. The answers are the
/// enumeration's.
#[test]
fn decision_names_nothing_where_nothing_outside_its_tree_reads_it() {
    for seed in 0..2 {
        let circuit = Circuit::caesar(seed, 3, 2);
        assert_eq!(circuit.probabilities.len(), 21);
        let source = circuit.source();
        let expected = circuit.answers();
        for strategy in [
            Strategy::Trial(Kind::Decision),
            Strategy::Trial(Kind::Program { fan_in: false }),
            Strategy::Auto,
        ] {
            let run = common::run(&source, &common::options(strategy, Projection::default()))
                .unwrap_or_else(|e| panic!("seed {seed}, {strategy}: {e}\n{source}"));
            if strategy == Strategy::Trial(Kind::Decision) {
                assert!(
                    run.ground.encoding.indicators.is_empty(),
                    "seed {seed}: {} names",
                    run.ground.encoding.indicators.len()
                );
            }
            assert_answers(
                &format!("seed {seed}, {strategy}"),
                &run.results.expect("evidence is consistent"),
                &expected,
                1e-12,
            );
        }
    }
}
