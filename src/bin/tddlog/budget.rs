//! The binary's memory budget: the smaller of its address-space limit and
//! the memory its cgroups leave it.
//!
//! tddlog refuses an allocation where the allocator refuses it: tididi
//! answers the operation as over budget, and `auto` gives up the trial
//! that asked and keeps the others. Under a limit on the address space
//! (`RLIMIT_AS`, `ulimit -v`) the allocator refuses at the limit. A
//! cgroup's `memory.max` bounds resident memory instead, and the kernel
//! kills a process that reaches it, with nothing refused before. So the
//! binary lowers its address-space limit to the room its cgroups leave
//! (`memory.max` less `memory.current`, the least over its cgroup and
//! those above it), and is refused there as under `ulimit -v`: resident
//! memory never exceeds the address space that holds it.

/// Lowers the process's address-space limit to the room its cgroups
/// leave, where that is less. Without a cgroup v2 hierarchy, or where no
/// cgroup sets `memory.max`, the limit stays as it is.
pub fn limit_to_cgroups() {
    #[cfg(target_os = "linux")]
    {
        let Ok(own) = std::fs::read_to_string("/proc/self/cgroup") else {
            return;
        };
        let read = |file: &str| std::fs::read_to_string(format!("/sys/fs/cgroup/{file}")).ok();
        let Some(room) = cgroup_room(&own, read) else {
            return;
        };
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: getrlimit writes the struct it is given; setrlimit reads
        // it. A soft limit below the hard one is always allowed.
        unsafe {
            if libc::getrlimit(libc::RLIMIT_AS, &mut limit) == 0 && room < limit.rlim_cur {
                limit.rlim_cur = room;
                libc::setrlimit(libc::RLIMIT_AS, &limit);
            }
        }
    }
}

/// The least, over the cgroup v2 that `own` (the text of
/// `/proc/self/cgroup`) names and every cgroup above it, of its
/// `memory.max` less its `memory.current`, or `None` where none of them
/// sets `memory.max`. `read` reads a file under the hierarchy's root by its
/// path relative to that root.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn cgroup_room(own: &str, read: impl Fn(&str) -> Option<String>) -> Option<u64> {
    let path = own.lines().find_map(|line| line.strip_prefix("0::"))?;
    let mut dir = path.trim().trim_matches('/');
    let mut room: Option<u64> = None;
    loop {
        let file = |name: &str| {
            let path = match dir {
                "" => name.to_string(),
                dir => format!("{dir}/{name}"),
            };
            read(&path)?.trim().parse::<u64>().ok()
        };
        // `max` for no limit, and no file at the root.
        if let (Some(max), Some(current)) = (file("memory.max"), file("memory.current")) {
            let left = max.saturating_sub(current);
            room = Some(room.map_or(left, |r| r.min(left)));
        }
        if dir.is_empty() {
            return room;
        }
        dir = dir.rfind('/').map_or("", |slash| &dir[..slash]);
    }
}

#[cfg(test)]
mod tests {
    use super::cgroup_room;
    use std::collections::HashMap;

    fn room(own: &str, files: &[(&str, &str)]) -> Option<u64> {
        let files: HashMap<&str, &str> = files.iter().copied().collect();
        cgroup_room(own, |path| files.get(path).map(|text| text.to_string()))
    }

    #[test]
    fn the_room_is_the_least_over_the_cgroup_and_those_above_it() {
        let own = "0::/user.slice/run.scope/run-1\n";
        let files = [
            ("user.slice/memory.max", "max\n"),
            ("user.slice/memory.current", "900\n"),
            ("user.slice/run.scope/memory.max", "1000\n"),
            ("user.slice/run.scope/memory.current", "300\n"),
            ("user.slice/run.scope/run-1/memory.max", "800\n"),
            ("user.slice/run.scope/run-1/memory.current", "50\n"),
        ];
        // 700 left in the scope, 750 in the command's own cgroup.
        assert_eq!(room(own, &files), Some(700));
        // The command's own cgroup alone sets a limit.
        assert_eq!(room(own, &files[4..]), Some(750));
        // A cgroup over its limit leaves nothing.
        assert_eq!(
            room(
                own,
                &[("user.slice/run.scope/run-1/memory.max", "40"), files[5]]
            ),
            Some(0)
        );
    }

    #[test]
    fn no_limit_leaves_no_room_to_take() {
        // No limit set, the root (which has no memory.max), and no cgroup
        // v2 line (a v1 hierarchy).
        let unlimited = [("a/memory.max", "max\n"), ("a/memory.current", "123\n")];
        assert_eq!(room("0::/a\n", &unlimited), None);
        assert_eq!(room("0::/\n", &[("memory.current", "5")]), None);
        assert_eq!(
            room(
                "4:memory:/a\n",
                &[("a/memory.max", "10"), ("a/memory.current", "1")]
            ),
            None
        );
        // A limit at the root of a cgroup namespace, as in a container.
        assert_eq!(
            room("0::/\n", &[("memory.max", "64"), ("memory.current", "4")]),
            Some(60)
        );
    }
}
