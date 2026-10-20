//! Separator copies (src/vtree/separators.rs) through the command line:
//! on clause programs whose `td` fold copies separator facts, `P(e)` and
//! the answers equal a count by enumeration, the copies are made, and
//! `--ground` prints them; a star is planned without reading its units.
//! The rules other than the default are tested in the crate
//! (src/vtree/tests/copies.rs).
use std::process::{Command, Output};

/// The command line with `args`.
fn tddlog(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tddlog"))
        .args(args)
        .output()
        .unwrap()
}

/// A fixed sequence of numbers in `0..n`.
struct Numbers(u64);

impl Numbers {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

/// A clause program over facts in clusters of three, each cluster but the
/// first tied to an earlier one: two clauses of three literals within a
/// cluster and three across to the earlier one, each `violated(j)`
/// observed false. The interaction graph is a tree of small clusters, so
/// `td`'s fold is a tree of bags with siblings to copy for. The program
/// queries `x(0)` and `q`, which reads facts of two clusters.
struct Program {
    probabilities: Vec<f64>,
    clauses: Vec<Vec<(usize, bool)>>,
    q: Vec<(usize, bool)>,
}

impl Program {
    fn random(seed: u64, clusters: usize) -> Program {
        let mut numbers = Numbers(seed);
        let n = 3 * clusters;
        let probabilities = (0..n)
            .map(|_| [0.2, 0.3, 0.5, 0.6, 0.9][numbers.below(5)])
            .collect();
        let mut clauses = Vec::new();
        let mut clause = |numbers: &mut Numbers, vars: &[usize]| {
            let mut lits: Vec<(usize, bool)> = Vec::new();
            while lits.len() < 3 {
                let v = vars[numbers.below(vars.len())];
                if lits.iter().all(|&(w, _)| w != v) {
                    lits.push((v, numbers.below(2) == 0));
                }
            }
            clauses.push(lits);
        };
        for c in 0..clusters {
            let own = [3 * c, 3 * c + 1, 3 * c + 2];
            for _ in 0..2 {
                clause(&mut numbers, &own);
            }
            if c > 0 {
                let p = numbers.below(c);
                let both = [3 * c, 3 * c + 1, 3 * c + 2, 3 * p, 3 * p + 1, 3 * p + 2];
                for _ in 0..3 {
                    clause(&mut numbers, &both);
                }
            }
        }
        let q = vec![(1, true), (n - 1, false)];
        Program {
            probabilities,
            clauses,
            q,
        }
    }

    fn source(&self) -> String {
        let lit = |&(v, positive): &(usize, bool)| {
            if positive {
                format!("x({v})")
            } else {
                format!("\\+x({v})")
            }
        };
        let mut s = String::new();
        for (v, p) in self.probabilities.iter().enumerate() {
            s += &format!("{p}::x({v}).\n");
        }
        for (j, c) in self.clauses.iter().enumerate() {
            s += &format!(
                "violated({j}) :- {}.\nevidence(violated({j}), false).\n",
                c.iter().map(lit).collect::<Vec<_>>().join(", ")
            );
        }
        s += &format!(
            "q :- {}.\nquery(x(0)).\nquery(q).\n",
            self.q.iter().map(lit).collect::<Vec<_>>().join(", ")
        );
        s
    }

    /// `P(e)`, `P(x(0) | e)` and `P(q | e)` by enumeration.
    fn exact(&self) -> (f64, f64, f64) {
        let n = self.probabilities.len();
        let (mut e, mut x0, mut q) = (0.0, 0.0, 0.0);
        for world in 0u32..1 << n {
            let holds = |&(v, positive): &(usize, bool)| (world >> v & 1 == 1) == positive;
            if self.clauses.iter().any(|c| c.iter().all(holds)) {
                continue;
            }
            let w: f64 = (0..n)
                .map(|v| {
                    if world >> v & 1 == 1 {
                        self.probabilities[v]
                    } else {
                        1.0 - self.probabilities[v]
                    }
                })
                .product();
            e += w;
            if world & 1 == 1 {
                x0 += w;
            }
            if self.q.iter().all(holds) {
                q += w;
            }
        }
        (e, x0 / e, q / e)
    }
}

/// The answers of a run: `x(0)`, `q` and `log10 P(e)`.
fn answers(out: &Output) -> (f64, f64, f64) {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = std::str::from_utf8(&out.stdout).unwrap();
    let field = |prefix: &str, i: usize| -> f64 {
        let line = stdout
            .lines()
            .find(|l| l.starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix} in {stdout}"));
        line.split('\t')
            .nth(i)
            .unwrap()
            .trim_start_matches("log10 ")
            .parse()
            .unwrap()
    };
    (field("x(0):", 1), field("q:", 1), field("P(e):", 2))
}

/// The copies a run's statistics report.
fn copies(out: &Output) -> usize {
    let stderr = String::from_utf8_lossy(&out.stderr);
    let note = stderr
        .split("separator copies: ")
        .nth(1)
        .unwrap_or_else(|| panic!("no copies note in {stderr}"));
    note.split(' ').next().unwrap().parse().unwrap()
}

#[test]
fn copies_keep_the_evidence_probability_and_the_answers() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let mut copied = 0;
    for seed in 1..=6 {
        let program = Program::random(seed, 6);
        let path = dir.join(format!("separator_copies_{seed}.pl"));
        std::fs::write(&path, program.source()).unwrap();
        let path = path.to_str().unwrap();
        let (e, x0, q) = program.exact();
        let out = tddlog(&[
            "--vtree",
            "td",
            "--evidence-probability",
            "--precision",
            "10",
            "--stats",
            path,
        ]);
        let (a, b, l) = answers(&out);
        let made = copies(&out);
        copied += made;
        // `--ground` prints the program `td` compiles, its copies among
        // the facts.
        let ground = tddlog(&["--ground", "--vtree", "td", "--evidence-probability", path]);
        assert!(
            ground.status.success(),
            "{}",
            String::from_utf8_lossy(&ground.stderr)
        );
        let printed = String::from_utf8_lossy(&ground.stdout);
        assert_eq!(
            printed
                .lines()
                .filter(|l| l.starts_with("0.5::'$sc'("))
                .count(),
            made,
            "seed {seed}: {printed}"
        );
        assert!(
            (a - x0).abs() < 1e-8 && (b - q).abs() < 1e-8,
            "seed {seed}: {a} {b} against {x0} {q}"
        );
        assert!(
            (l - e.log10()).abs() < 1e-8,
            "seed {seed}: log10 P(e) {l} against {}",
            e.log10()
        );
    }
    assert!(copied > 0, "no copies made");
}

#[test]
fn a_star_is_ruled_out_by_its_bounds() {
    // One atom reads every fact, a rule for each, as a TPC-H selection
    // does: it crosses every unit, and a unit's side holds every fact
    // outside it. The bounds on the side rule each unit out without
    // reading it, where reading them all was cubic in the facts.
    // P(q) = 1 - prod(1 - p) by hand.
    let n = 4000;
    let p = |i: usize| 1e-4 * (1 + i % 3) as f64;
    let mut source: String = (0..n)
        .map(|i| format!("{}::f({i}).\nq :- f({i}).\n", p(i)))
        .collect();
    source += "query(q).\n";
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("separator_copies_star.pl");
    std::fs::write(&path, source).unwrap();
    let out = tddlog(&[
        "--vtree",
        "td",
        "--precision",
        "12",
        "--stats",
        path.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let want = 1.0 - (0..n).map(|i| 1.0 - p(i)).product::<f64>();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let got: f64 = stdout
        .lines()
        .find_map(|l| l.strip_prefix("q:\t"))
        .unwrap_or_else(|| panic!("no q in {stdout}"))
        .parse()
        .unwrap();
    assert!((got - want).abs() < 1e-10, "P(q) {got} against {want}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let note = stderr
        .split("separator copies: ")
        .nth(1)
        .unwrap_or_else(|| panic!("no copies note in {stderr}"));
    let candidates: usize = note
        .split(" of ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(candidates > 1000, "{note}");
    assert!(
        note.contains("0 copies atop 0 of") && note.contains(", 0 read in full"),
        "{note}"
    );
}
