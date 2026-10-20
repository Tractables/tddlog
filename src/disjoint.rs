//! Union-find over a parent array, `parent[x] == x` at a root: the find
//! that halves the path, and the union that puts the larger root under the
//! smaller, so that a set's root is its smallest member.

/// The root of `x`, each node on the way pointed at its grandparent.
pub(crate) fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

/// Joins the sets of `a` and `b` under the smaller of their roots, and
/// says whether they were apart.
pub(crate) fn union(parent: &mut [usize], a: usize, b: usize) -> bool {
    let (x, y) = (find(parent, a), find(parent, b));
    if x == y {
        return false;
    }
    parent[x.max(y)] = x.min(y);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_is_rooted_at_its_smallest_member() {
        let mut parent: Vec<usize> = (0..6).collect();
        assert!(union(&mut parent, 4, 2));
        assert!(union(&mut parent, 5, 4));
        assert!(!union(&mut parent, 2, 5));
        assert!(union(&mut parent, 3, 1));
        let roots: Vec<usize> = (0..6).map(|x| find(&mut parent, x)).collect();
        assert_eq!(roots, [0, 1, 2, 1, 2, 2]);
    }
}
