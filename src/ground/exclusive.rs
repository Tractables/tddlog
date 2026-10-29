//! Sets of derived atoms of which at most one is true in any world.
//!
//! A ground-program analysis shared by the rewrites that exploit it: the
//! indicators ([`crate::indicators`]) read the other members of a set
//! negatively, and row classes ([`super::classes`]) merge the chains of
//! rows whose bodies read different members of one set.

use std::hash::BuildHasher;

use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use super::interned::Interned;
use super::{AtomId, GLit, GroundProgram, RulesByHead};

/// The sets of two or more derived atoms of which at most one is true in
/// any world, found from the rules alone, in the order their components are
/// compiled.
///
/// Two atoms are exclusive when every rule for the one and every rule for
/// the other have bodies that cannot both hold: a literal of one is the
/// negation of a literal of the other, as the stick-breaking switches of one
/// row, or they read two different atoms of a set already found, as the
/// parents' values of two rows. The atoms are taken in dependency order,
/// so that the sets of an atom's parents are final when it comes, and an
/// atom joins the first set it is exclusive with among those whose atoms
/// read atoms of the same sets: the values of one Bayesian network variable
/// read the same parents, if not the same values of them (a genotype reads
/// one value of each parental allele). Among them siblings with the same
/// parents are not exclusive, and end up in sets of their own. An atom
/// that reads no derived atom is compared with the atoms of its own unit
/// in the sense of [`super::deps::components`]
/// ([`super::structure::Structure::base_of`]), which holds the values of a
/// table without parents.
///
/// The atoms of a predicate with several atoms join only sets of that
/// predicate, and atoms alone in their predicate only sets of such atoms.
/// Relevance can leave one value of a table, as `dewpoints(highevrywher)`,
/// the evidence of hailfinder_ev; it excluded `scnrelplfcst(g)`, which
/// joined it and was parted from the other values of scnrelplfcst, and
/// then no two rows of plainsfcst, which read them, clashed.
///
/// Nor are the atoms of a predicate of several atoms that the evidence
/// makes false, every one, and that no rule reads: each is a set of its
/// own. They join only sets of their predicate, and such a set says
/// nothing, none of its atoms ever being true and no body reading one. A
/// weighted CNF's clauses are such a predicate, each `violated(j)` observed
/// false, which compared with each other would be most of the run on a
/// large one. Values that conditioning settles false beside one it settles
/// true (`s(a)` beside `s(b)` in a test below) keep their sets.
pub(super) fn exclusive_sets(
    gp: &GroundProgram,
    by_head: &RulesByHead,
    base_of: &[usize],
    sccs: &[Vec<AtomId>],
) -> Found {
    let n = gp.num_atoms();
    // The atoms of each set, in the order they joined it ([`Chains`]),
    // rather than a list per set: most atoms are alone in theirs.
    let mut sets: Chains<AtomId> = Chains::default();
    let mut set_of = vec![usize::MAX; n];
    // Each atom's key, its class and the sets it reads, numbered.
    let mut key_ids: Interned<usize> = Interned::with_capacity(0);
    // The class of each atom: its predicate's index if the predicate has
    // other atoms, `usize::MAX` if not. Each atom's predicate is looked up
    // once, not again when its key is made.
    let mut predicates: FxHashMap<(&str, usize), usize> = FxHashMap::default();
    let mut predicate_of = vec![usize::MAX; n];
    let mut atoms_of: Vec<usize> = Vec::new();
    for &a in sccs.iter().flatten() {
        let next = predicates.len();
        let id = *predicates
            .entry(gp.atoms[a as usize].signature())
            .or_insert(next);
        if id == atoms_of.len() {
            atoms_of.push(0);
        }
        atoms_of[id] += 1;
        predicate_of[a as usize] = id;
    }
    let class = |a: AtomId| match predicate_of[a as usize] {
        id if atoms_of[id] > 1 => id,
        _ => usize::MAX,
    };
    // The sets of each key.
    let mut by_key: Chains<usize> = Chains::default();
    // Each atom's rules by the atoms of sets they read positively, once
    // its parents' sets are final: where every rule of two atoms reads one
    // atom of each of the same sets, two rules that read different atoms of
    // one set clash, and only the rules that read the same ones need their
    // literals compared, not every pair of two values' rules (bn_pathfinder's
    // f74 has over a thousand rules per value).
    let mut rows: Vec<Option<Rows>> = vec![None; n];
    // Each rule is stored once at most, with at most its body's literals as
    // the atoms it reads, so the store is sized once rather than grown.
    let mut store = RowStore::default();
    store.rules.reserve_exact(gp.rules.len());
    store.run_ends.reserve_exact(gp.rules.len());
    store.key_hashes.reserve_exact(gp.rules.len());
    store
        .selectors
        .reserve_exact(gp.rules.iter().map(|r| r.body.len()).sum());
    // The sets the rows read, numbered ([`Rows::sig`]).
    let mut sigs: FxHashMap<Vec<usize>, u32> = FxHashMap::default();
    // Per set, the numbers of the sets its atoms' rows read, distinct, and
    // `None` for an atom without rows; and the runs of its atoms' rows by
    // the set, the number and the hash of the atoms they read: the head of
    // a chain through `run_chain` of each run's atom, first rule and next
    // run. An atom is compared with only the runs that read what one of its
    // rows reads, not merged with every atom's rows in turn.
    let mut sigs_of_set: Chains<Option<u32>> = Chains::default();
    let mut run_heads: FxHashMap<(usize, u32, u64), u32> = FxHashMap::default();
    let mut run_chain: Vec<(AtomId, u32, u32)> = Vec::new();
    // Each body that another rule can have, its literals sorted and
    // distinct, by number, with the sets whose rules hold it. A body with a
    // literal that no other rule reads is no other rule's, and rules no set
    // out for another atom. On a network nearly every body has one, a
    // switch of its row, so few bodies are hashed and copied to be
    // numbered.
    let mut body_ids = Bodies::with_capacity(gp.rules.len());
    // How many rules read each literal, 2 standing for two or more.
    let lit = |l: &GLit| 2 * l.atom() as usize + usize::from(l.is_positive());
    let mut readers = vec![0u8; 2 * n];
    for l in gp.rules.iter().flat_map(|r| &r.body) {
        readers[lit(l)] = (readers[lit(l)] + 1).min(2);
    }
    // The predicates of several atoms each observed false and read by no
    // rule, whose atoms are not compared.
    let mut false_unread = vec![false; n];
    for &(_, id, value) in &gp.evidence {
        if let Some(x) = id
            .filter(|&x| !value && readers[2 * x as usize] == 0 && readers[2 * x as usize + 1] == 0)
        {
            false_unread[x as usize] = true;
        }
    }
    let mut silent: Vec<bool> = atoms_of.iter().map(|&k| k > 1).collect();
    for &a in sccs.iter().flatten() {
        silent[predicate_of[a as usize]] &= false_unread[a as usize];
    }
    let mut marks = Marks::new(n);
    // Buffers reused across atoms and rules: a body, the rest of it that
    // other rules can clash with, and what `rows_of` reads of each rule.
    let mut sorted: Vec<GLit> = Vec::new();
    let mut kept: Vec<GLit> = Vec::new();
    let mut scratch = RowScratch::default();
    // And per atom: the sets it reads and its key, its rules' bodies that
    // do not clash with themselves, distinct, and those an earlier atom
    // had, and the runs of a set's rows that read what a row of it reads.
    let mut read_sets: Vec<usize> = Vec::new();
    // The sets the atom at hand reads, each marked with the atom's stamp
    // as it is listed, so that it is listed once, without sorting the sets
    // of every literal of its rules. Atoms are numbered from zero, so an
    // atom's number plus one is a stamp fresh for each.
    let mut read_mark = vec![0usize; n];
    let mut key: Vec<usize> = Vec::new();
    let mut rule_bodies: Vec<(usize, u32)> = Vec::new();
    let mut bodies: Vec<u32> = Vec::new();
    let mut shared: Vec<(usize, u32)> = Vec::new();
    // The bodies of the atom at hand that rule out a set of its key, and
    // the sets a shared body is found to rule out as the atom is compared.
    let mut holding: Vec<u32> = Vec::new();
    let mut failed: Vec<(u32, usize)> = Vec::new();
    let mut runs: Vec<(usize, usize)> = Vec::new();
    // The atoms of a set whose rows read other sets than the atom's, kept
    // from one call of `exclusive_with` to the next rather than a list
    // allocated per call.
    let mut rest: Vec<AtomId> = Vec::new();
    // Per set, the atoms every rule of every one of its atoms reads
    // positively, sorted, as a range of `common`, which only shrinks, and
    // its key; per atom, the sets that held it so
    // when they were made, in the order they were. An atom with a rule that
    // is one negated atom `\+x` clashes with a rule only where that rule
    // reads `x`, so it can join only a set all of whose rules read `x`
    // (dice_digitRecognition's observations each have a rule `\+y(d)`).
    let mut common: Vec<AtomId> = Vec::new();
    let mut common_of: Vec<(usize, usize)> = Vec::new();
    let mut key_of_set: Vec<usize> = Vec::new();
    let mut sets_reading: FxHashMap<AtomId, usize> = FxHashMap::default();
    let mut reading: Chains<usize> = Chains::default();
    let mut common_a: Vec<AtomId> = Vec::new();
    // Per key and literal, the sets of the key whose first atom's first
    // rule has a literal that clashes with it ([`Marks::clashes`]): its
    // negation, or another atom of a set it reads positively ([`clash_code`]).
    // An atom joins a set only where each of its rules clashes with that
    // rule, so the sets it can join are among those listed under the
    // literals of any one of its rules, not every set of its key:
    // dice_caesar-iter's bits of 3000 steps, two sets a step, all of one
    // key, would each be compared with every set before it. A key's sets
    // are listed from its second on, and all compared where it has one
    // (most keys, whose listing cost hybit_weekend's grounding 5 per cent),
    // or a set whose first atom has no rules, or whose first rule reads an
    // atom whose set is not known yet.
    let mut clashing: FxHashMap<(usize, usize), usize> = FxHashMap::default();
    let mut clash_lists: Chains<usize> = Chains::default();
    let mut unlisted: Vec<bool> = Vec::new();
    let mut candidates: Vec<usize> = Vec::new();
    for &a in sccs.iter().flatten() {
        if silent[predicate_of[a as usize]] {
            let i = sets.add();
            sigs_of_set.add();
            key_of_set.push(usize::MAX);
            common_of.push((common.len(), common.len()));
            sets.push(i, a);
            set_of[a as usize] = i;
            continue;
        }
        // The sets of the atoms it reads that came before it; an atom that
        // reads none is compared with the atoms of its own unit only, the
        // values of a table without parents.
        let stamp = a as usize + 1;
        read_sets.clear();
        for l in by_head[a as usize]
            .iter()
            .flat_map(|&ri| gp.rules[ri].body.iter())
        {
            let i = set_of[l.atom() as usize];
            if i != usize::MAX && read_mark[i] != stamp {
                read_mark[i] = stamp;
                read_sets.push(i);
            }
        }
        read_sets.sort_unstable();
        key.clear();
        key.push(class(a));
        if read_sets.is_empty() {
            key.push(usize::MAX - base_of[a as usize]);
        } else {
            key.extend_from_slice(&read_sets);
        }
        let rows_a = rows_of(
            gp,
            &by_head[a as usize],
            &set_of,
            &mut scratch,
            &mut sigs,
            &mut store,
        );
        let sig_a = rows_a.as_ref().map(|x| x.sig);
        // A body that does not clash with itself, found in a rule of an
        // atom of a set, keeps every atom with that body out of the set, so
        // only the sets holding none of this atom's bodies are compared
        // (dice_digitRecognition's observations are alone in their
        // predicates and share one key, and would each be compared with all
        // the sets before it). So does a body that does not clash with a
        // rule of the set ([`Bodies::rule_out`]).
        let known = body_ids.len();
        // The atoms each rule of `a` reads positively, and a rule that is
        // one negated atom, if there is one, read off each body as it is
        // sorted to be numbered, not in a pass of their own.
        let mut required: Option<AtomId> = None;
        common_a.clear();
        rule_bodies.clear();
        for (k, &ri) in by_head[a as usize].iter().enumerate() {
            // The rule's body sorted and distinct, as nearly every body
            // is already, copied only where it is not.
            let body: &[GLit] = if gp.rules[ri].body.is_sorted_by(|x, y| x < y) {
                &gp.rules[ri].body
            } else {
                sorted.clear();
                sorted.extend_from_slice(&gp.rules[ri].body);
                sorted.sort_unstable();
                sorted.dedup();
                &sorted
            };
            if let [GLit::Neg(x)] = body[..] {
                required.get_or_insert(x);
            }
            if k == 0 {
                common_a.extend(body.iter().filter(|l| l.is_positive()).map(|l| l.atom()));
                common_a.dedup();
            } else {
                common_a.retain(|&x| body.binary_search(&GLit::Pos(x)).is_ok());
            }
            // What another rule can clash with of the body: its literals
            // but those of a fact that no other rule reads, which no set
            // holds either. A rule clashes with another atom's rule where
            // the rest of its body does, so two rules with the same rest
            // stand for each other ([`Bodies`]). A choice read under
            // exclusive contexts is read through a fact per context, read
            // once: dice_caesar-iter's 3000 encoded letters, each alone in
            // its predicate, have the same four rests, a value of the key
            // each, and are not compared with each other.
            let once = |l: &GLit| {
                let x = l.atom() as usize;
                readers[2 * x] + readers[2 * x + 1] == 1
                    && by_head[x].is_empty()
                    && set_of[x] == usize::MAX
            };
            let rest_of: &[GLit] = if body.iter().any(once) {
                kept.clear();
                kept.extend(body.iter().filter(|l| !once(l)));
                &kept
            } else {
                body
            };
            // A body with a literal no other rule reads is not marked to
            // tell whether it clashes with itself: on a network nearly
            // every body is such. Nor is an empty rest, which would stand
            // for every rule that reads only such facts.
            if !rest_of.is_empty() && rest_of.iter().all(|l| readers[lit(l)] == 2) {
                let clashes = marks.mark(rest_of, &set_of);
                marks.clear();
                if !clashes {
                    rule_bodies.push((ri, body_ids.id(rest_of)));
                }
            }
        }
        bodies.clear();
        bodies.extend(rule_bodies.iter().map(|&(_, b)| b));
        bodies.sort_unstable();
        bodies.dedup();
        // The bodies of `a` that an earlier atom had, one rule each: those
        // a later atom may have too.
        shared.clear();
        shared.extend(
            rule_bodies
                .iter()
                .copied()
                .filter(|&(_, b)| (b as usize) < known),
        );
        shared.sort_unstable_by_key(|&(_, b)| b);
        shared.dedup_by_key(|&mut (_, b)| b);
        let key_id = key_ids.id(&key).0 as usize;
        if by_key.len() <= key_id {
            by_key.add();
            unlisted.push(false);
        }
        // A body that rules out every set of the key leaves `a` a set of
        // its own: dice_digitRecognition's observations of the digits 1
        // to 9 each have a rule `y(0)`, which every set of the key holds or
        // does not clash with.
        let alone = bodies
            .iter()
            .any(|&b| body_ids.rules_out_all(b, key_id, by_key.len_of(key_id)));
        // Each set that comes up is asked whether it holds one of the
        // bodies of `a` that rule out a set of the key, rather than every
        // set holding one marked first: each leaf's adder of bb_wmi_star
        // has the bits of the centre for bodies, and its atoms of one key
        // marked the sets of every leaf before them.
        holding.clear();
        holding.extend(
            bodies
                .iter()
                .copied()
                .filter(|&b| body_ids.rules_out_some(b, key_id)),
        );
        let ruled_out = |i: usize| holding.iter().any(|&b| body_ids.holds(b, i));
        // Whether `a` is exclusive with every atom of a set: whether every
        // rule of `a` clashes with every rule of each atom, or, where the
        // two atoms' rows read the same sets, with every rule of the same
        // row. Each rule of `a` is marked once per set and tested against
        // all the set's atoms, not marked anew for each atom. The rule
        // marked last stays marked for the next set, which is most often
        // with the same rule: an atom of one rule is compared with every
        // candidate set.
        let mut exclusive_with = |i: usize| -> bool {
            let exclusive = 'test: {
                rest.clear();
                if sigs_of_set.iter(i).any(|s| s.is_none() || s != sig_a) {
                    rest.extend(sets.iter(i).filter(|&b| {
                        sig_a.is_none() || rows[b as usize].as_ref().map(|y| y.sig) != sig_a
                    }));
                }
                if !rest.is_empty() {
                    for &ra in &by_head[a as usize] {
                        marks.mark_rule(ra, &gp.rules[ra].body, &set_of);
                        if !rest.iter().all(|&b| {
                            by_head[b as usize]
                                .iter()
                                .all(|&rb| marks.clashes(&gp.rules[rb].body, &set_of))
                        }) {
                            break 'test false;
                        }
                    }
                }
                let Some(x) = rows_a
                    .as_ref()
                    .filter(|x| sigs_of_set.iter(i).any(|s| s == Some(x.sig)))
                else {
                    break 'test true;
                };
                // Each run of the rows of `a`, and the runs of the set's atoms
                // whose rows read the same sets and the same atoms of them, as
                // ranges of the store's rules.
                let mut j = 0;
                while j < x.len {
                    let j_end = x.run_end(&store, j);
                    runs.clear();
                    let key = x.key(&store, j);
                    let mut link = run_heads.get(&(i, x.sig, x.key_hash(&store, j))).copied();
                    while let Some(l) = link {
                        let (b, k, next) = run_chain[l as usize];
                        let y = rows[b as usize].as_ref().expect("an indexed atom has rows");
                        if y.key(&store, k as usize) == key {
                            runs.push((
                                y.start + k as usize,
                                y.start + y.run_end(&store, k as usize),
                            ));
                        }
                        link = (next != u32::MAX).then_some(next);
                    }
                    if !runs.is_empty() {
                        for &ra in &x.rules(&store)[j..j_end] {
                            marks.mark_rule(ra, &gp.rules[ra].body, &set_of);
                            if !runs.iter().all(|&(s, e)| {
                                store.rules[s..e]
                                    .iter()
                                    .all(|&rb| marks.clashes(&gp.rules[rb].body, &set_of))
                            }) {
                                break 'test false;
                            }
                        }
                    }
                    j = j_end;
                }
                true
            };
            // A shared body of `a` that does not clash with a rule of the
            // set rules the set out for every atom with that body.
            if !exclusive {
                for &(ra, b) in &shared {
                    marks.mark_rule(ra, &gp.rules[ra].body, &set_of);
                    if !sets.iter(i).all(|m| {
                        by_head[m as usize]
                            .iter()
                            .all(|&rb| marks.clashes(&gp.rules[rb].body, &set_of))
                    }) {
                        failed.push((b, i));
                    }
                }
            }
            exclusive
        };
        let joined = match required {
            _ if alone => None,
            Some(x) => sets_reading
                .get(&x)
                .into_iter()
                .flat_map(|&l| reading.iter(l))
                .filter(|&i| {
                    key_of_set[i] == key_id
                        && !ruled_out(i)
                        && common[common_of[i].0..common_of[i].1]
                            .binary_search(&x)
                            .is_ok()
                })
                .find(|&i| exclusive_with(i)),
            None => {
                // The rule of `a` under whose literals the fewest sets are
                // listed, where they are fewer than the key's sets.
                let listed = |ri: usize| {
                    let clashing = &clashing;
                    let set_of = &set_of;
                    gp.rules[ri].body.iter().flat_map(move |&l| {
                        [Some(lit(&l)), clash_code(l, set_of, n)]
                            .into_iter()
                            .flatten()
                            .filter_map(move |code| clashing.get(&(key_id, code)).copied())
                    })
                };
                let fewest = by_head[a as usize]
                    .iter()
                    .filter(|_| by_key.len_of(key_id) > 1 && !unlisted[key_id])
                    .map(|&ri| (listed(ri).map(|k| clash_lists.len_of(k)).sum::<usize>(), ri))
                    .min()
                    .filter(|&(count, _)| count < by_key.len_of(key_id));
                match fewest {
                    Some((_, ri)) => {
                        candidates.clear();
                        candidates.extend(
                            listed(ri)
                                .flat_map(|k| clash_lists.iter(k))
                                .filter(|&i| !ruled_out(i)),
                        );
                        candidates.sort_unstable();
                        candidates.dedup();
                        candidates.iter().copied().find(|&i| exclusive_with(i))
                    }
                    None => by_key
                        .iter(key_id)
                        .filter(|&i| !ruled_out(i))
                        .find(|&i| exclusive_with(i)),
                }
            }
        };
        marks.clear();
        for (b, i) in failed.drain(..) {
            body_ids.rule_out(b, key_id, i);
        }
        let i = match joined {
            Some(i) => {
                let (start, end) = common_of[i];
                let mut kept = start;
                for k in start..end {
                    if common_a.binary_search(&common[k]).is_ok() {
                        common[kept] = common[k];
                        kept += 1;
                    }
                }
                common_of[i].1 = kept;
                sets.push(i, a);
                if !sigs_of_set.iter(i).any(|s| s == sig_a) {
                    sigs_of_set.push(i, sig_a);
                }
                i
            }
            None => {
                let i = sets.add();
                sets.push(i, a);
                sigs_of_set.add();
                sigs_of_set.push(i, sig_a);
                by_key.push(key_id, i);
                key_of_set.push(key_id);
                for &x in &common_a {
                    let l = *sets_reading.entry(x).or_insert_with(|| reading.add());
                    reading.push(l, i);
                }
                common_of.push((common.len(), common.len() + common_a.len()));
                common.extend_from_slice(&common_a);
                // List set `j`, whose first atom is `b`.
                let mut list = |j: usize, b: AtomId| match by_head[b as usize]
                    .first()
                    .map(|&r| &gp.rules[r].body)
                {
                    Some(body)
                        if body.iter().all(|l| {
                            !l.is_positive()
                                || set_of[l.atom() as usize] != usize::MAX
                                || predicate_of[l.atom() as usize] == usize::MAX
                        }) =>
                    {
                        for &l in body {
                            for code in [Some(lit(&l.negated())), clash_code(l, &set_of, n)]
                                .into_iter()
                                .flatten()
                            {
                                let k = *clashing
                                    .entry((key_id, code))
                                    .or_insert_with(|| clash_lists.add());
                                clash_lists.push(k, j);
                            }
                        }
                    }
                    _ => unlisted[key_id] = true,
                };
                match by_key.len_of(key_id) {
                    1 => {}
                    2 => {
                        let first = by_key.iter(key_id).next().expect("the key's first set");
                        list(first, sets.iter(first).next().expect("a set's first atom"));
                        list(i, a);
                    }
                    _ => list(i, a),
                }
                i
            }
        };
        if let Some(x) = &rows_a {
            let mut j = 0;
            while j < x.len {
                let head = run_heads
                    .entry((i, x.sig, x.key_hash(&store, j)))
                    .or_insert(u32::MAX);
                run_chain.push((a, j as u32, *head));
                *head = (run_chain.len() - 1) as u32;
                j = x.run_end(&store, j);
            }
        }
        rows[a as usize] = rows_a;
        set_of[a as usize] = i;
        // An atom that has a body a set holds is ruled out of it, and its
        // bodies are distinct, so a set takes each body once.
        for &body in &bodies {
            body_ids.rule_out(body, key_id, i);
        }
    }
    let sets = (0..sets.len())
        .filter(|&i| sets.len_of(i) >= 2)
        .map(|i| sets.iter(i).collect())
        .collect();
    Found {
        sets,
        predicate_of,
        atoms_of,
    }
}

/// The code under which [`exclusive_sets`] lists the literals that clash
/// with `l` for reading another atom of its set, for a positive literal of
/// an atom of a set: `2 * n` plus the set, past the literals' codes.
fn clash_code(l: GLit, set_of: &[usize], n: usize) -> Option<usize> {
    let s = set_of[l.atom() as usize];
    if l.is_positive() && s != usize::MAX {
        Some(2 * n + s)
    } else {
        None
    }
}

/// What [`exclusive_sets`] finds: the sets, and the predicates it numbers
/// ([`super::structure::Structure::predicate_of`]).
pub(super) struct Found {
    pub(super) sets: Vec<Vec<AtomId>>,
    pub(super) predicate_of: Vec<usize>,
    pub(super) atoms_of: Vec<usize>,
}

/// Lists that only grow, their items end to end in one buffer, each chained
/// to the next of its list ([`exclusive_sets`]).
struct Chains<T> {
    /// Per list its first and its last entry, `u32::MAX` while it is
    /// empty, and its length.
    heads: Vec<(u32, u32, u32)>,
    /// Per entry its item and the next entry of its list.
    entries: Vec<(T, u32)>,
}

impl<T> Default for Chains<T> {
    fn default() -> Self {
        Chains {
            heads: Vec::new(),
            entries: Vec::new(),
        }
    }
}

impl<T: Copy> Chains<T> {
    /// How many lists there are.
    fn len(&self) -> usize {
        self.heads.len()
    }

    /// How many items list `i` has.
    fn len_of(&self, i: usize) -> usize {
        self.heads[i].2 as usize
    }

    /// A new empty list, and its number.
    fn add(&mut self) -> usize {
        self.heads.push((u32::MAX, u32::MAX, 0));
        self.heads.len() - 1
    }

    /// Add `x` to the end of list `i`.
    fn push(&mut self, i: usize, x: T) {
        let e = self.entries.len() as u32;
        self.entries.push((x, u32::MAX));
        let (first, last, len) = &mut self.heads[i];
        if *first == u32::MAX {
            *first = e;
        } else {
            self.entries[*last as usize].1 = e;
        }
        *last = e;
        *len += 1;
    }

    /// The items of list `i`, in the order they were added.
    fn iter(&self, i: usize) -> impl Iterator<Item = T> + '_ {
        let mut e = self.heads[i].0;
        std::iter::from_fn(move || {
            let &(x, next) = self.entries.get(e as usize)?;
            e = next;
            Some(x)
        })
    }
}

/// Distinct bodies numbered in the order they are first seen
/// ([`Interned`]), with the sets that hold each ([`exclusive_sets`]): each
/// body with a set it rules out, and per body and key how many sets of the
/// key it rules out, rather than a list in a map per set and body: on a
/// network nearly every body is a rule's own. A body is what other rules
/// can clash with of a rule's, without the facts no other rule reads.
struct Bodies {
    bodies: Interned<GLit>,
    /// Each body with a set it rules out.
    held: FxHashSet<(u32, u32)>,
    /// Per body and key, how many sets of the key the body rules out.
    counts: FxHashMap<(u32, u32), u32>,
    /// Whether `held` takes no more entries ([`Bodies::rule_out`]).
    full: bool,
}

impl Bodies {
    fn with_capacity(bodies: usize) -> Self {
        Bodies {
            bodies: Interned::with_capacity(bodies),
            held: FxHashSet::default(),
            counts: FxHashMap::default(),
            full: false,
        }
    }

    /// How many bodies there are.
    fn len(&self) -> usize {
        self.bodies.len()
    }

    /// Whether body `b` rules out `set`.
    fn holds(&self, b: u32, set: usize) -> bool {
        self.held.contains(&(b, set as u32))
    }

    /// Whether body `b` rules out a set under `key`.
    fn rules_out_some(&self, b: u32, key: usize) -> bool {
        self.counts.contains_key(&(b, key as u32))
    }

    /// Record that body `b` rules out `set`, under `key`: an atom with a
    /// rule of that body cannot join it. A set rules out the bodies of its
    /// atoms' rules, and the bodies that do not clash with a rule of one
    /// of its atoms; a set only takes atoms, so it rules them out for
    /// good, and a body that rules a set out keeps the atoms that have it
    /// from being compared with it, so it is recorded once.
    ///
    /// The record only spares comparisons: a set it would rule out is
    /// compared with the atom, and the comparison finds the atom not
    /// exclusive with it. Where `held` or `counts` cannot grow, because
    /// the allocator refuses it, nothing more is recorded and the sets found
    /// are the same. The entries grow with the bodies times the sets: a
    /// large weighted CNF would ask for gigabytes.
    fn rule_out(&mut self, b: u32, key: usize, set: usize) {
        if (self.held.len() == self.held.capacity() || self.counts.len() == self.counts.capacity())
            && !self.grow()
        {
            return;
        }
        if self.held.insert((b, set as u32)) {
            *self.counts.entry((b, key as u32)).or_insert(0) += 1;
        }
    }

    /// Room for one more entry of `held` and of `counts`, as an insertion
    /// makes it; false once there is none ([`rule_out`](Self::rule_out)).
    #[cold]
    fn grow(&mut self) -> bool {
        self.full =
            self.full || self.held.try_reserve(1).is_err() || self.counts.try_reserve(1).is_err();
        !self.full
    }

    /// Whether body `b` rules out all of the `sets` sets under `key`.
    fn rules_out_all(&self, b: u32, key: usize, sets: usize) -> bool {
        sets > 0
            && self
                .counts
                .get(&(b, key as u32))
                .is_some_and(|&held| held as usize == sets)
    }

    /// The number of `body`, numbered anew if it is new.
    fn id(&mut self, body: &[GLit]) -> u32 {
        self.bodies.id(body).0
    }
}

/// The literals of one rule body, marked to tell in one pass over another
/// body whether the two clash ([`exclusive_sets`]): whether a literal of
/// one is the negation of a literal of the other, or they read different
/// atoms of one set positively, without comparing every pair of literals.
struct Marks {
    /// Per atom, bit 1 if the body reads it positively, bit 2 negatively.
    polarity: Vec<u8>,
    /// Per set, the atom the body reads positively, [`Marks::MANY`] for two
    /// or more, [`Marks::NONE`] for none.
    reads: Vec<AtomId>,
    atoms: Vec<AtomId>,
    sets: Vec<usize>,
    /// The rule whose body is marked, if one is.
    rule: Option<usize>,
}

impl Marks {
    const NONE: AtomId = AtomId::MAX;
    const MANY: AtomId = AtomId::MAX - 1;

    /// For `n` atoms, and so at most `n` sets.
    fn new(n: usize) -> Self {
        Marks {
            polarity: vec![0; n],
            reads: vec![Self::NONE; n],
            atoms: Vec::new(),
            sets: Vec::new(),
            rule: None,
        }
    }

    /// Mark the body of rule `ri`, unless it is the one marked.
    fn mark_rule(&mut self, ri: usize, body: &[GLit], set_of: &[usize]) {
        if self.rule != Some(ri) {
            self.clear();
            self.mark(body, set_of);
            self.rule = Some(ri);
        }
    }

    /// Mark `body`, and say whether it clashes with itself: whether it
    /// reads an atom both ways, or two atoms of one set positively, in the
    /// marking pass rather than a second pass by [`Marks::clashes`].
    fn mark(&mut self, body: &[GLit], set_of: &[usize]) -> bool {
        let mut clashes = false;
        for &l in body {
            let a = l.atom();
            let polarity = self.polarity[a as usize];
            if polarity == 0 {
                self.atoms.push(a);
            }
            let bit = if l.is_positive() { 1 } else { 2 };
            clashes |= polarity & !bit != 0;
            self.polarity[a as usize] = polarity | bit;
            let s = set_of[a as usize];
            if l.is_positive() && s != usize::MAX {
                match self.reads[s] {
                    Self::NONE => {
                        self.reads[s] = a;
                        self.sets.push(s);
                    }
                    b if b == a => {}
                    _ => {
                        self.reads[s] = Self::MANY;
                        clashes = true;
                    }
                }
            }
        }
        clashes
    }

    /// Whether `body` clashes with the marked body.
    fn clashes(&self, body: &[GLit], set_of: &[usize]) -> bool {
        body.iter().any(|&m| {
            let a = m.atom();
            let polarity = self.polarity[a as usize];
            if m.is_positive() {
                let s = set_of[a as usize];
                polarity & 2 != 0
                    || (s != usize::MAX && self.reads[s] != Self::NONE && self.reads[s] != a)
            } else {
                polarity & 1 != 0
            }
        })
    }

    fn clear(&mut self) {
        self.rule = None;
        for a in self.atoms.drain(..) {
            self.polarity[a as usize] = 0;
        }
        for s in self.sets.drain(..) {
            self.reads[s] = Self::NONE;
        }
    }
}

/// An atom's rules keyed by the atoms of sets each reads positively
/// ([`exclusive_sets`]), kept in a [`RowStore`].
#[derive(Clone, Copy)]
struct Rows {
    /// The number of the sets every rule reads one atom of among the rows
    /// of the program, and how many sets that is.
    sig: u32,
    width: usize,
    /// The rules, as a range of the store's rules from `start`, and their
    /// atoms of the sets from `selectors`.
    start: usize,
    len: usize,
    selectors: usize,
}

/// The rules of every atom's [`Rows`], end to end, grouped per atom by the
/// atoms they read of its sets, so that the rules that read the same atoms
/// are a run; and those atoms, as many per rule as the atom's rows read
/// sets, in the same order. The runs are chained in one map per program,
/// not a map per atom from the atoms read to the rules, nor two lists per
/// atom, each of which costs allocations per row or per atom.
#[derive(Default)]
struct RowStore {
    rules: Vec<usize>,
    selectors: Vec<AtomId>,
    /// Per rule the end of its run, from the start of its atom's rules,
    /// kept rather than found anew by comparing atoms each time a run is
    /// walked.
    run_ends: Vec<u32>,
    /// Per rule the hash of the atoms it reads, worked out once to group
    /// the rules and kept to chain and find the runs.
    key_hashes: Vec<u64>,
}

impl Rows {
    /// The rules, in the store's order.
    fn rules<'s>(&self, store: &'s RowStore) -> &'s [usize] {
        &store.rules[self.start..self.start + self.len]
    }

    /// The atoms rule `i` reads of the sets, in the sets' order.
    fn key<'s>(&self, store: &'s RowStore, i: usize) -> &'s [AtomId] {
        let at = self.selectors + i * self.width;
        &store.selectors[at..at + self.width]
    }

    /// The end of the run of rules from `i` that read the same atoms.
    fn run_end(&self, store: &RowStore, i: usize) -> usize {
        store.run_ends[self.start + i] as usize
    }

    /// The hash of the atoms rule `i` reads.
    fn key_hash(&self, store: &RowStore, i: usize) -> u64 {
        store.key_hashes[self.start + i]
    }
}

/// The buffers [`rows_of`] reuses from one call to the next: what it reads
/// of the first rule, per set the position of its atom in what a rule
/// reads, and the groups of the rules that read the same atoms, the
/// last group in each bucket of the hashes of those atoms and per group the
/// one before it in its bucket, its first rule and its size, and per rule
/// its group and its hash. The buckets are as many as twice the rules at
/// hand, or more, a power of two, and only those are cleared, not a map
/// at the size the most rules made it.
#[derive(Default)]
struct RowScratch {
    read: Vec<(usize, AtomId)>,
    sets: Vec<usize>,
    slot: Vec<u32>,
    selectors: Vec<AtomId>,
    order: Vec<usize>,
    buckets: Vec<u32>,
    same_group: Vec<u32>,
    group_of: Vec<u32>,
    groups: Vec<(usize, usize)>,
    hashes: Vec<u64>,
}

/// The [`Rows`] of the rules `rules`, added to `store`, or `None` where
/// they do not all read one atom of each of the same sets.
fn rows_of(
    gp: &GroundProgram,
    rules: &[usize],
    set_of: &[usize],
    scratch: &mut RowScratch,
    sigs: &mut FxHashMap<Vec<usize>, u32>,
    store: &mut RowStore,
) -> Option<Rows> {
    let RowScratch {
        read,
        sets,
        slot,
        selectors,
        order,
        buckets,
        same_group,
        group_of,
        groups,
        hashes,
    } = scratch;
    sets.clear();
    selectors.clear();
    read.clear();
    read.extend(
        rules
            .iter()
            .take(1)
            .flat_map(|&ri| gp.rules[ri].body.iter())
            .filter(|l| l.is_positive() && set_of[l.atom() as usize] != usize::MAX)
            .map(|l| (set_of[l.atom() as usize], l.atom())),
    );
    read.sort_unstable();
    read.dedup();
    if read.windows(2).any(|w| w[0].0 == w[1].0) {
        return None;
    }
    sets.extend(read.iter().map(|&(s, _)| s));
    selectors.extend(read.iter().map(|&(_, b)| b));
    let w = sets.len();
    for (j, &s) in sets.iter().enumerate() {
        if slot.len() <= s {
            slot.resize(s + 1, u32::MAX);
        }
        slot[s] = j as u32;
    }
    // Each later rule's atoms go straight to the slots of the first rule's
    // sets: an atom of a set the first rule does not read, a second atom of
    // a set, or a set left without one, is a rule that does not read the
    // same sets. A slot left by an earlier call holds a stale position,
    // which the first rule's sets tell apart. No rule's atoms are
    // collected, sorted and compared with the first rule's sets.
    for &ri in rules.iter().skip(1) {
        let at = selectors.len();
        selectors.resize(at + w, AtomId::MAX);
        let mut filled = 0;
        for l in gp.rules[ri].body.iter().filter(|l| l.is_positive()) {
            let s = set_of[l.atom() as usize];
            if s == usize::MAX {
                continue;
            }
            let j = match slot.get(s) {
                Some(&j) if (j as usize) < w && sets[j as usize] == s => j as usize,
                _ => return None,
            };
            let x = &mut selectors[at + j];
            if *x == AtomId::MAX {
                *x = l.atom();
                filled += 1;
            } else if *x != l.atom() {
                return None;
            }
        }
        if filled != w {
            return None;
        }
    }
    // The rules that read the same atoms, grouped by the hash of those
    // atoms, the groups in the order they first come and the rules of each
    // in order, without a sort: where the rules of an atom's values each
    // read other atoms, a sort only finds runs of one rule.
    let size = (2 * rules.len()).next_power_of_two().max(2);
    let shift = u64::BITS - size.trailing_zeros();
    buckets.clear();
    buckets.resize(size, u32::MAX);
    same_group.clear();
    group_of.clear();
    groups.clear();
    hashes.clear();
    for k in 0..rules.len() {
        let key = &selectors[k * w..(k + 1) * w];
        let h = FxBuildHasher.hash_one(key);
        hashes.push(h);
        let last = &mut buckets[(h >> shift) as usize];
        let mut g = *last;
        while g != u32::MAX
            && (hashes[groups[g as usize].0] != h
                || selectors[groups[g as usize].0 * w..(groups[g as usize].0 + 1) * w] != *key)
        {
            g = same_group[g as usize];
        }
        if g == u32::MAX {
            g = groups.len() as u32;
            groups.push((k, 0));
            same_group.push(*last);
            *last = g;
        }
        groups[g as usize].1 += 1;
        group_of.push(g);
    }
    // Each group's rules from where the groups before it end.
    let mut end = 0;
    for (start, size) in groups.iter_mut() {
        *start = end;
        end += *size;
    }
    order.clear();
    order.resize(rules.len(), 0);
    for (k, &g) in group_of.iter().enumerate() {
        let (at, _) = &mut groups[g as usize];
        order[*at] = k;
        *at += 1;
    }
    let x = Rows {
        sig: 0,
        width: w,
        start: store.rules.len(),
        len: rules.len(),
        selectors: store.selectors.len(),
    };
    store.rules.extend(order.iter().map(|&k| rules[k]));
    for &k in order.iter() {
        store
            .selectors
            .extend_from_slice(&selectors[k * w..(k + 1) * w]);
    }
    // The placement moved each group's start to its end, where the run of
    // each of its rules ends.
    store
        .run_ends
        .extend(order.iter().map(|&k| groups[group_of[k] as usize].0 as u32));
    store.key_hashes.extend(order.iter().map(|&k| hashes[k]));
    let sig = match sigs.get(sets.as_slice()) {
        Some(&sig) => sig,
        None => {
            let sig = sigs.len() as u32;
            sigs.insert(sets.clone(), sig);
            sig
        }
    };
    Some(Rows { sig, ..x })
}

#[cfg(test)]
mod tests {
    use crate::ground::structure::Structure;

    fn sets_of(source: &str) -> Vec<Vec<String>> {
        let program = crate::syntax::parse_program(source).expect("parses");
        let gp =
            crate::ground::ground(&program, &crate::GroundOptions::default()).expect("grounds");
        let mut sets: Vec<Vec<String>> = Structure::of(&gp)
            .sets
            .iter()
            .map(|set| {
                let mut names: Vec<String> = set
                    .iter()
                    .map(|&a| gp.atoms[a as usize].to_string())
                    .collect();
                names.sort();
                names
            })
            .collect();
        sets.sort();
        sets
    }

    #[test]
    fn the_values_of_a_predicate_stay_together() {
        // d(x) holds only where s(b) does, so it excludes r(a) and r(c).
        // As evidence it comes first, the one relevant value of d, as
        // dewpoints(highevrywher) in hailfinder_ev; r(a) and r(c) joining
        // it would part them from r(b). t is queried so that the evidence,
        // which settles the r's, is not dropped as detached.
        let sets = sets_of(
            "0.5::t.
             0.2::s(a); 0.3::s(b); 0.5::s(c) :- t.
             0.4::s(a); 0.6::s(b) :- \\+t.
             0.3::d(x); 0.7::d(y) :- s(b).
             r(a) :- s(a). r(b) :- s(b). r(c) :- s(c).
             evidence(d(x)). query(t). query(r(a)). query(r(b)). query(r(c)).",
        );
        let names = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(
            sets,
            vec![
                names(&["r(a)", "r(b)", "r(c)"]),
                names(&["s(a)", "s(b)", "s(c)"])
            ]
        );
    }

    #[test]
    fn a_predicate_observed_false_that_nothing_reads_joins_no_set() {
        // The clauses of `x(1) or x(2)` and `not x(1) or not x(2)`: the
        // two violated atoms exclude each other, and once both are
        // observed false neither is compared.
        let clauses = "0.5::x(1). 0.5::x(2).
             violated(1) :- \\+x(1), \\+x(2). violated(2) :- x(1), x(2).";
        let queried = sets_of(&format!(
            "{clauses} query(violated(1)). query(violated(2))."
        ));
        assert_eq!(
            queried,
            vec![vec!["violated(1)".to_string(), "violated(2)".to_string()]]
        );
        let observed = sets_of(&format!(
            "{clauses} evidence(violated(1), false). evidence(violated(2), false). query(x(1))."
        ));
        assert!(observed.is_empty(), "{observed:?}");
    }

    #[test]
    fn atoms_alone_in_their_predicate_form_sets() {
        let sets = sets_of(
            "0.5::c. 0.4::d.
             heads :- c, d. tails :- \\+c, d.
             query(heads). query(tails).",
        );
        assert_eq!(sets, vec![vec!["heads".to_string(), "tails".to_string()]]);
    }

    #[test]
    fn an_atom_with_a_negated_rule_joins_a_set_whose_rules_read_that_atom() {
        // v's rule is `\+y(0)` alone, and every rule of u reads y(0): they
        // exclude each other. a and b, the observations of the Dice
        // program's digits, each with a rule `\+y(d)`, do not.
        let sets = sets_of(
            "0.3::y(0); 0.7::y(1). 0.1::o1. 0.1::o2. 0.1::o3.
             u :- y(0), \\+o3. v :- \\+y(0).
             a :- y(0), \\+o1. a :- \\+y(0).
             b :- y(1), \\+o2. b :- \\+y(1).
             query(u). query(v). query(a). query(b).",
        );
        let names = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(sets.contains(&names(&["u", "v"])), "{sets:?}");
        assert!(
            !sets
                .iter()
                .any(|s| s.contains(&"a".to_string()) && s.contains(&"b".to_string())),
            "{sets:?}"
        );
    }

    #[test]
    fn a_body_that_does_not_clash_with_a_set_keeps_its_atoms_out() {
        // p and q exclude each other. s fails against {p, q} on its rule
        // `y(1), o2`, and its rule `y(0)`, which r had before it, does not
        // clash with p's: that rules {p, q} out for t, whose rule `y(0)`
        // every set then rules out, and t is left alone. u has no body an
        // earlier atom had, fails against {p, q} and joins r.
        let sets = sets_of(
            "0.2::y(0); 0.3::y(1); 0.5::y(2). 0.5::o1. 0.5::o2.
             p :- y(0), o1.
             q :- y(1), \\+o1. q :- y(0), \\+o1.
             r :- y(2), o2. r :- y(0).
             s :- y(1), o2. s :- y(0).
             t :- y(0). t :- y(2), \\+o2.
             u :- y(1), \\+o2.
             query(p). query(q). query(r). query(s). query(t). query(u).",
        );
        let names = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(sets.contains(&names(&["p", "q"])), "{sets:?}");
        assert!(sets.contains(&names(&["r", "u"])), "{sets:?}");
        for x in ["s", "t"] {
            assert!(
                !sets.iter().any(|set| set.contains(&x.to_string())),
                "{x} in {sets:?}"
            );
        }
    }

    #[test]
    fn rules_that_differ_in_facts_read_once_stand_for_each_other() {
        // f1 to f6 are read once each: a rule clashes with another atom's
        // where the rest of its body, a value of k, does. e1 and e2 read
        // the same values and are not exclusive, and e2 is not compared
        // with e1 once e1's set holds those rests, nor a with either; b
        // reads the other value than a and joins it. f7 is read by g and,
        // negated, by h, which keep it and exclude each other.
        let sets = sets_of(
            "0.3::k(0); 0.7::k(1). 0.5::f1. 0.5::f2. 0.5::f3. 0.5::f4. 0.5::f5. 0.5::f6. 0.5::f7.
             e1 :- k(0), f1. e1 :- k(1), f2.
             e2 :- k(0), f3. e2 :- k(1), f4.
             a :- k(0), f5. b :- k(1), f6.
             g :- k(0), f7. h :- k(0), \\+f7.
             query(e1). query(e2). query(a). query(b). query(g). query(h).",
        );
        let names = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(sets.contains(&names(&["a", "b"])), "{sets:?}");
        assert!(sets.contains(&names(&["g", "h"])), "{sets:?}");
        assert!(
            !sets
                .iter()
                .any(|set| set.contains(&"e1".to_string()) || set.contains(&"e2".to_string())),
            "{sets:?}"
        );
    }

    #[test]
    fn an_atom_is_compared_with_the_sets_whose_first_rule_clashes_with_its_own() {
        // The steps' atoms read k, all of one key, and each step's own
        // switch: b2 and b3, whose key holds two sets and more, are
        // compared with the set listed under their switch, their step's,
        // and not with the sets before it. d's key holds one set, c's,
        // which is compared without a listing.
        let sets = sets_of(
            "0.5::k. 0.5::s1. 0.5::s2. 0.5::s3. 0.3::y(0); 0.7::y(1).
             a1 :- s1, k. b1 :- \\+s1, k.
             a2 :- s2, k. b2 :- \\+s2, k.
             a3 :- s3, k. b3 :- \\+s3, k.
             c :- y(0), k. d :- y(1), k.
             query(a1). query(b1). query(a2). query(b2). query(a3). query(b3). query(c). query(d).",
        );
        let names = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        for set in [["a1", "b1"], ["a2", "b2"], ["a3", "b3"], ["c", "d"]] {
            assert!(sets.contains(&names(&set)), "{set:?} in {sets:?}");
        }
    }

    #[test]
    fn a_body_holds_the_sets_of_each_key_apart() {
        // A body that rules out sets under two keys in turn, as the centre's
        // bits of bb_wmi_star are read under each leaf's key: each set is
        // asked alone, and the body rules out every set of a key that its
        // sets there number.
        let mut bodies = super::Bodies::with_capacity(0);
        let b = bodies.id(&[crate::ground::GLit::Pos(0)]);
        for (key, set) in [(0, 0), (1, 1), (0, 2), (1, 3), (1, 4)] {
            bodies.rule_out(b, key, set);
        }
        let held: Vec<usize> = (0..6).filter(|&i| bodies.holds(b, i)).collect();
        assert_eq!(held, [0, 1, 2, 3, 4]);
        assert!(bodies.rules_out_some(b, 0) && bodies.rules_out_some(b, 1));
        assert!(!bodies.rules_out_some(b, 2));
        assert!(bodies.rules_out_all(b, 0, 2));
        assert!(bodies.rules_out_all(b, 1, 3));
        assert!(!bodies.rules_out_all(b, 1, 4));
        assert!(!bodies.rules_out_all(b, 2, 0));
    }
}
