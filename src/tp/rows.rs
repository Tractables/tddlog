//! A network's table stated as its rows.
//!
//! The link between a table and a reader of it ([`crate::indicators`]) is
//! a loop of bodies over facts alone: groups of copies of which exactly one
//! holds, one group for each input of the table read a copy per value and
//! one for its output, variables of no group (switches, and the bits of
//! inputs read in binary), and a body per row of the table and value of
//! the output, that the row's inputs and the switches that pick that value
//! hold and the output's copy of it does not. Where the output is read in
//! binary, its copies are the bits of its value, in no group, and a body
//! fixes one of them: that the row's inputs and switches hold and the bit
//! does not where the value has it, or does where it has not. Its formula,
//! `⋀ groups ∧ ⋀_r ¬body_r`, holds where each group holds one value, and
//! the output the value that the inputs and switches force. Those
//! assignments, each fixing every variable, are the formula's models, and
//! tididi builds the canonical diagram of a set of models in one pass
//! ([`TddBackend::of_models`](super::TddBackend::of_models)).
//!
//! [`table_models`] reads the models off the loop's bodies and groups, and
//! declines where they are not a table's or too many.

use tididi::Literal;
use tididi::vtree::VarId;

/// No value of a group: where the group is at most one, none of its
/// variables holds. It sorts after every variable, as it counts after them.
const NONE: u32 = u32::MAX;

/// An input a cube does not name, until the cube is spread over the
/// input's values.
const FREE: u32 = u32::MAX - 1;

/// The models of `⋀ groups ∧ ⋀_r ¬cubes_r` over local variables `1..=n`,
/// `n.div_ceil(64)` words apiece, bit `k - 1` for variable `k`, where that
/// is a table: `None` where it is not, or where reading them takes more
/// than `max` rows, the cubes spread over the inputs they leave free and
/// the assignments no cube reads, or makes more than `max` models.
///
/// Two variables of no group of which the cubes say that not both hold,
/// and, where they also say that not neither holds, exactly one, are a
/// group of two; every other variable of no group but `outputs` is an
/// input of two values. The output is `outputs`, its value read in binary,
/// where they are given, and else the group of which cubes negate a
/// variable. A cube reads at most one variable of the output and forces
/// it where the rest of the cube holds: a variable of a group the cube
/// negates holds, and a variable of `outputs` takes the value the cube
/// does not read; a cube that reads none rules the rest out. Each cube
/// reads at most one variable of every group that is an input, and that
/// positively, and an input of two values either way. An assignment of the
/// inputs at which no cube rules out and the variables the cubes force do
/// not disagree is a model with each output value those leave, and one
/// that no cube reads leaves the output free.
pub(super) fn table_models(
    cubes: &[Vec<Literal>],
    groups: &[(Vec<VarId>, bool)],
    outputs: &[VarId],
    n: usize,
    max: usize,
) -> Option<Vec<u64>> {
    let mut columns = Columns {
        group_of: vec![u32::MAX; n + 1],
        member: vec![0; n + 1],
        members: Vec::with_capacity(n),
        all: Vec::with_capacity(n),
    };
    for (g, exactly) in groups {
        if g.iter().any(|v| columns.group_of[v.0 as usize] != u32::MAX) {
            return None;
        }
        columns.push(g, *exactly);
    }
    let mut out_of = vec![false; n + 1];
    for v in outputs {
        if columns.group_of[v.0 as usize] != u32::MAX {
            return None;
        }
        out_of[v.0 as usize] = true;
    }
    // Pairs of variables in no group, both of the output or neither: both
    // positive, `not both`; both negative, `not neither`.
    let mut both: Vec<(u32, u32)> = Vec::new();
    let mut neither: Vec<(u32, u32)> = Vec::new();
    let pair = |c: &[Literal], sign: bool| match *c {
        [a, b]
            if a.sign == sign
                && b.sign == sign
                && a.var != b.var
                && out_of[a.var.0 as usize] == out_of[b.var.0 as usize] =>
        {
            Some((a.var.0.min(b.var.0), a.var.0.max(b.var.0)))
        }
        _ => None,
    };
    for c in cubes {
        if c.iter()
            .any(|l| columns.group_of[l.var.0 as usize] != u32::MAX)
        {
            continue;
        }
        if let Some(p) = pair(c, true) {
            both.push(p);
        } else if let Some(p) = pair(c, false) {
            neither.push(p);
        }
    }
    both.sort_unstable();
    both.dedup();
    neither.sort_unstable();
    for &(a, b) in &both {
        if columns.group_of[a as usize] != u32::MAX || columns.group_of[b as usize] != u32::MAX {
            return None;
        }
        columns.push(
            &[VarId(a), VarId(b)],
            neither.binary_search(&(a, b)).is_ok(),
        );
    }
    let paired = |c: &[Literal]| {
        let p = pair(c, true).or_else(|| pair(c, false));
        p.is_some_and(|(a, b)| {
            let g = columns.group_of[a as usize];
            g != u32::MAX
                && g == columns.group_of[b as usize]
                && columns.all[g as usize].1 == 2
                && both.binary_search(&(a, b)).is_ok()
        })
    };
    // The cubes besides those of the groups of two.
    let cubes: Vec<&[Literal]> = cubes
        .iter()
        .map(Vec::as_slice)
        .filter(|c| !paired(c))
        .collect();

    // The output: `outputs` where the cubes leave them in no group, a group
    // of their own, and else the group of which cubes negate a variable.
    let binary = !outputs.is_empty()
        && outputs
            .iter()
            .all(|v| columns.group_of[v.0 as usize] == u32::MAX);
    let output = if binary {
        if outputs.len() >= 64 {
            return None;
        }
        columns.push(outputs, false);
        columns.all.len() - 1
    } else {
        let mut output: Option<u32> = None;
        for c in &cubes {
            for l in c.iter().filter(|l| !l.sign) {
                let g = columns.group_of[l.var.0 as usize];
                if g != u32::MAX && *output.get_or_insert(g) != g {
                    return None;
                }
            }
        }
        output? as usize
    };
    // Every other variable of no group: an input of two values, the
    // variable true and false.
    let first_bit = columns.all.len();
    for v in 1..=n {
        if columns.group_of[v] == u32::MAX {
            columns.push(&[VarId(v as u32)], false);
        }
    }
    let Columns {
        group_of,
        member,
        members,
        all,
    } = columns;
    let bit = |g: usize| g >= first_bit;
    let width = all.len();
    // A cube spreads over two values or more of each input it does not
    // name: where that alone passes `max`, decline before reading them.
    let most = max.max(1).ilog2() as usize;
    if cubes.iter().any(|c| width > c.len() + 1 + most) {
        return None;
    }
    // The values of each group: its variables, and none where it is at
    // most one; those of `outputs`, its codes.
    let count = |g: usize| match g == output && binary {
        true => 1u64 << all[g].1,
        false => all[g].1 as u64 + u64::from(!all[g].2),
    };
    let value_of = |g: usize, k: u64| if k == all[g].1 as u64 { NONE } else { k as u32 };

    // Each cube as `width + 1` values: the value of each input it names,
    // free for those it does not, none for the output, and then the cube's
    // place. What it says of the output is in `forces`: the bits of the
    // output's value it fixes and their values, all of them for a group,
    // and `None` where it rules its inputs out.
    let mut read: Vec<u32> = Vec::with_capacity(cubes.len() * (width + 1));
    let mut forces: Vec<Option<(u64, u64)>> = Vec::with_capacity(cubes.len());
    for (i, c) in cubes.iter().enumerate() {
        let at = read.len();
        read.resize(at + width + 1, FREE);
        let (values, place) = read[at..].split_at_mut(width);
        place[0] = i as u32;
        let mut force = None;
        for &l in c.iter() {
            let g = group_of[l.var.0 as usize] as usize;
            let k = member[l.var.0 as usize];
            if g == output {
                if force.is_some() || (l.sign && !binary) {
                    return None;
                }
                force = Some(if binary {
                    (1 << k, u64::from(!l.sign) << k)
                } else {
                    (u64::MAX, u64::from(k))
                });
            } else if values[g] != FREE || !(l.sign || bit(g)) {
                return None;
            } else {
                values[g] = if l.sign { k } else { NONE };
            }
        }
        values[output] = NONE;
        forces.push(force);
    }

    // Each cube spread over the values of the inputs it does not name, in
    // the same layout.
    let mut spread = 0u64;
    for cube in read.chunks(width + 1) {
        let each = (0..width)
            .filter(|&g| cube[g] == FREE)
            .try_fold(1u64, |t, g| t.checked_mul(count(g)))?;
        spread = spread.checked_add(each)?;
    }
    if spread > max as u64 {
        return None;
    }
    let mut rows: Vec<u32> = Vec::with_capacity(spread as usize * (width + 1));
    let mut free = Vec::new();
    let mut at = Vec::new();
    for cube in read.chunks(width + 1) {
        free.clear();
        free.extend((0..width).filter(|&g| cube[g] == FREE));
        at.clear();
        at.resize(free.len(), 0u64);
        loop {
            let row = rows.len();
            rows.extend_from_slice(cube);
            for (i, &g) in free.iter().enumerate() {
                rows[row + g] = value_of(g, at[i]);
            }
            if !step(&mut at, |i| count(free[i]), 0..free.len()) {
                break;
            }
        }
    }
    let key = |r: u32| &rows[r as usize * (width + 1)..][..width];
    let mut order: Vec<u32> = (0..spread as u32).collect();
    order.sort_unstable_by(|&a, &b| key(a).cmp(key(b)));

    // The inputs no cube reads leave the output free.
    let inputs: Vec<usize> = (0..width).filter(|&g| g != output).collect();
    let total = inputs
        .iter()
        .try_fold(1u64, |t, &g| t.checked_mul(count(g)))?;
    let named = order.chunk_by(|&a, &b| key(a) == key(b)).count() as u64;
    let outputs = count(output);
    if spread.checked_add((total - named).checked_mul(outputs)?)? > max as u64 {
        return None;
    }
    let words = n.div_ceil(64).max(1);
    let mut models = Vec::new();
    // The models at `values` with each value of the output that agrees
    // with `value` on the bits `fixed`: for a group, the value fixed or,
    // where none is, each of them; false past `max` models.
    let mut put = |values: &[u32], (fixed, value): (u64, u64)| {
        let open = match (binary, fixed) {
            (true, _) => (outputs - 1) & !fixed,
            (false, 0) => outputs - 1,
            (false, _) => 0,
        };
        let mut next = 0u64;
        loop {
            let out = if binary { value | next } else { value + next };
            let at = models.len();
            models.resize(at + words, 0u64);
            for (g, &(start, len, _)) in all.iter().enumerate() {
                for (k, &v) in members[start..start + len].iter().enumerate() {
                    let holds = match g == output {
                        true if binary => out >> k & 1 == 1,
                        true => k as u64 == out,
                        false => k as u32 == values[g],
                    };
                    let i = v.0 as usize - 1;
                    models[at + i / 64] |= u64::from(holds) << (i % 64);
                }
            }
            if next == open {
                return models.len() <= max * words;
            }
            // The next value of the bits left open, or the next value.
            next = if binary {
                next.wrapping_sub(open) & open
            } else {
                next + 1
            };
        }
    };
    let place = |r: u32| rows[r as usize * (width + 1) + width] as usize;
    let mut runs = order.chunk_by(|&a, &b| key(a) == key(b)).peekable();
    at.clear();
    at.resize(inputs.len(), 0);
    let mut values = vec![NONE; width];
    loop {
        for (i, &g) in inputs.iter().enumerate() {
            values[g] = value_of(g, at[i]);
        }
        // The assignments come in ascending order, as the runs do.
        let fixed = match runs.next_if(|run| key(run[0]) == values.as_slice()) {
            // The bits the cubes fix, where they agree and rule out none.
            Some(run) => run.iter().try_fold((0u64, 0u64), |(fixed, value), &r| {
                let (more, to) = forces[place(r)]?;
                ((value ^ to) & fixed & more == 0).then_some((fixed | more, value | to))
            }),
            None => Some((0, 0)),
        };
        if let Some(fixed) = fixed
            && !put(&values, fixed)
        {
            return None;
        }
        // The next assignment, the last input fastest.
        if !step(&mut at, |i| count(inputs[i]), (0..inputs.len()).rev()) {
            break;
        }
    }
    debug_assert!(runs.next().is_none());
    Some(models)
}

/// The columns of a table's rows: each group as the start and length of
/// its run of `members` and whether it is exactly one, and of each
/// variable its group and place in it.
struct Columns {
    group_of: Vec<u32>,
    member: Vec<u32>,
    members: Vec<VarId>,
    all: Vec<(usize, usize, bool)>,
}

impl Columns {
    fn push(&mut self, g: &[VarId], exactly: bool) {
        for (k, &v) in g.iter().enumerate() {
            (self.group_of[v.0 as usize], self.member[v.0 as usize]) =
                (self.all.len() as u32, k as u32);
        }
        self.all.push((self.members.len(), g.len(), exactly));
        self.members.extend_from_slice(g);
    }
}

/// Steps the digits `at` to their next combination, digit `i` below
/// `radix(i)` and the digits taken in `order`, fastest first: false past
/// the last combination.
fn step(
    at: &mut [u64],
    radix: impl Fn(usize) -> u64,
    mut order: impl Iterator<Item = usize>,
) -> bool {
    order.any(|i| {
        at[i] += 1;
        if at[i] < radix(i) {
            return true;
        }
        at[i] = 0;
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(v: i32) -> Literal {
        Literal::new(VarId(v.unsigned_abs()), v > 0)
    }

    fn cubes(of: &[&[i32]]) -> Vec<Vec<Literal>> {
        of.iter()
            .map(|c| c.iter().map(|&v| lit(v)).collect())
            .collect()
    }

    fn group(vars: &[u32], exactly: bool) -> (Vec<VarId>, bool) {
        (vars.iter().map(|&v| VarId(v)).collect(), exactly)
    }

    /// The models of `⋀ groups ∧ ⋀ ¬cubes` over `1..=n`, every assignment
    /// tried.
    fn enumerated(cubes: &[Vec<Literal>], groups: &[(Vec<VarId>, bool)], n: usize) -> Vec<u64> {
        (0..1u64 << n)
            .filter(|&m| {
                let holds = |v: VarId| m >> (v.0 - 1) & 1 == 1;
                groups.iter().all(
                    |(g, exactly)| match g.iter().filter(|&&v| holds(v)).count() {
                        0 => !exactly,
                        k => k == 1,
                    },
                ) && !cubes
                    .iter()
                    .any(|c| c.iter().all(|l| holds(l.var) == l.sign))
            })
            .collect()
    }

    fn same_models(cubes: &[Vec<Literal>], groups: &[(Vec<VarId>, bool)], n: usize) {
        same_models_out(cubes, groups, &[], n);
    }

    fn same_models_out(
        cubes: &[Vec<Literal>],
        groups: &[(Vec<VarId>, bool)],
        outputs: &[u32],
        n: usize,
    ) {
        let outputs: Vec<VarId> = outputs.iter().map(|&v| VarId(v)).collect();
        let mut got = table_models(cubes, groups, &outputs, n, 1 << n).expect("a table");
        got.sort_unstable();
        assert_eq!(got, enumerated(cubes, groups, n));
    }

    /// The cubes that force each bit of `code` over `bits` where `inputs`
    /// hold: one a bit, negating the bit where it is set.
    fn forcing(inputs: &[i32], bits: &[i32], code: u32) -> Vec<Vec<i32>> {
        bits.iter()
            .enumerate()
            .map(|(j, &b)| {
                inputs
                    .iter()
                    .copied()
                    .chain([if code >> j & 1 == 1 { -b } else { b }])
                    .collect()
            })
            .collect()
    }

    /// A class link: `p` of three values (1 to 3) and `q` of two (4 and 5,
    /// a group of two that the cubes state) force a class (6 to 8).
    #[test]
    fn a_class_link_is_its_rows() {
        let mut of: Vec<Vec<i32>> = vec![vec![4, 5], vec![-4, -5]];
        for p in 0..3 {
            for q in 0..2 {
                of.push(vec![1 + p, 4 + q, -(6 + (p + 2 * q) % 3)]);
            }
        }
        let of: Vec<&[i32]> = of.iter().map(Vec::as_slice).collect();
        let groups = [group(&[1, 2, 3], true), group(&[6, 7, 8], true)];
        same_models(&cubes(&of), &groups, 8);
    }

    /// An input read in binary (1 and 2, code 0 ruled out) and one of three
    /// values (3 to 5) force a value of an output that is at most one (6
    /// and 7); the code 3 with the last value is read by no cube and leaves
    /// the output free.
    #[test]
    fn an_input_read_in_binary_is_two_columns() {
        let mut of: Vec<Vec<i32>> = vec![vec![-1, -2]];
        for code in 1..4 {
            for r in 0..3 {
                if (code, r) != (3, 2) {
                    let bit = |j: i32| if code >> j & 1 == 1 { j + 1 } else { -(j + 1) };
                    of.push(vec![bit(0), bit(1), 3 + r, -(6 + (code + r) % 2)]);
                }
            }
        }
        let of: Vec<&[i32]> = of.iter().map(Vec::as_slice).collect();
        let groups = [group(&[3, 4, 5], true), group(&[6, 7], false)];
        same_models(&cubes(&of), &groups, 7);
    }

    /// A reader link: the class (1 and 2) and the switches of its row (3
    /// for the first, 4 for the second) pick the reader's value (5 to 7);
    /// each row leaves the other's switch free.
    #[test]
    fn switches_are_columns_of_the_rows() {
        let of: [&[i32]; 4] = [&[1, 3, -5], &[1, -3, -6], &[2, 4, -6], &[2, -4, -7]];
        let groups = [group(&[1, 2], true), group(&[5, 6, 7], true)];
        same_models(&cubes(&of), &groups, 7);
    }

    /// Cubes that force two values of the output at one input rule it out.
    #[test]
    fn cubes_that_disagree_leave_no_model() {
        let of: [&[i32]; 4] = [&[1, -4], &[1, 3, -5], &[2, -5], &[2, -3, -6]];
        let groups = [group(&[1, 2], true), group(&[4, 5, 6], true)];
        same_models(&cubes(&of), &groups, 6);
    }

    /// Cubes that negate variables of two groups, a cube that reads the
    /// output positively, a variable in two groups, and more models than
    /// allowed are no table.
    #[test]
    fn what_is_no_table_is_declined() {
        let groups = [group(&[1, 2, 3], true), group(&[4, 5, 6], true)];
        assert!(table_models(&cubes(&[&[-1, 4], &[2, -5]]), &groups, &[], 6, 64).is_none());
        assert!(table_models(&cubes(&[&[1, 4, -5], &[2, -5]]), &groups, &[], 6, 64).is_none());
        let overlapping = [group(&[1, 2, 3], true), group(&[3, 4, 5], true)];
        assert!(table_models(&cubes(&[&[1, -4]]), &overlapping, &[], 5, 64).is_none());
        let of: [&[i32]; 3] = [&[1, -4], &[2, -5], &[3, -6]];
        assert!(table_models(&cubes(&of), &groups, &[], 6, 64).is_some());
        assert!(table_models(&cubes(&of), &groups, &[], 6, 2).is_none());
    }
    /// A table of an input of three values (1 to 3) and one read in binary
    /// (4 and 5, code 0 ruled out by a cube of its bits alone) whose output
    /// is read in binary too (6 to 8), a cube a bit; the code 3 with the
    /// last value is read by no cube and leaves the output free.
    #[test]
    fn an_output_read_in_binary_is_its_bits() {
        let mut of: Vec<Vec<i32>> = vec![vec![-4, -5]];
        for code in 1..4u32 {
            for r in 0..3u32 {
                if (code, r) != (3, 2) {
                    let bit = |j: i32| if code >> j & 1 == 1 { j + 4 } else { -(j + 4) };
                    of.extend(forcing(
                        &[bit(0), bit(1), 1 + r as i32],
                        &[6, 7, 8],
                        1 + (code * 3 + r) % 7,
                    ));
                }
            }
        }
        let of: Vec<&[i32]> = of.iter().map(Vec::as_slice).collect();
        same_models_out(&cubes(&of), &[group(&[1, 2, 3], true)], &[6, 7, 8], 8);
    }

    /// A reader's link with its output in binary (5 and 6): the class (1
    /// and 2) and the switch of its row (3 for the first, 4 for the
    /// second) pick a code; a row that fixes one bit alone leaves the
    /// other free, and two that fix a bit differently leave no model.
    #[test]
    fn bits_of_the_output_are_fixed_one_at_a_time() {
        let mut of: Vec<Vec<i32>> = Vec::new();
        of.extend(forcing(&[1, 3], &[5, 6], 1));
        of.extend(forcing(&[1, -3], &[5, 6], 2));
        of.extend(forcing(&[2, 4], &[5, 6], 3));
        of.push(vec![2, -4, -5]);
        let of: Vec<&[i32]> = of.iter().map(Vec::as_slice).collect();
        let groups = [group(&[1, 2], true)];
        same_models_out(&cubes(&of), &groups, &[5, 6], 6);
        let mut clash = of.clone();
        clash.push(&[1, 3, 5]);
        same_models_out(&cubes(&clash), &groups, &[5, 6], 6);
    }

    /// Two bits of the output that the cubes state a group of two are the
    /// output as a group; a cube of an input and a bit of the output is no
    /// pair; and a cube that reads two bits of the output is no table.
    #[test]
    fn outputs_are_bits_where_no_group_holds_them() {
        let groups = [group(&[1, 2], true)];
        let of: [&[i32]; 4] = [&[3, 4], &[-3, -4], &[1, -3], &[2, -4]];
        same_models_out(&cubes(&of), &groups, &[3, 4], 4);
        let of: [&[i32]; 4] = [&[1, 3], &[1, -4], &[-1, -3], &[-1, 4]];
        same_models_out(&cubes(&of), &[], &[3, 4], 4);
        let of: [&[i32]; 2] = [&[1, -3, 4], &[2, -4]];
        assert!(table_models(&cubes(&of), &groups, &[VarId(3), VarId(4)], 4, 64).is_none());
    }
}
