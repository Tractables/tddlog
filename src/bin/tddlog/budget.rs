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
//! (`memory.max` less `memory.current` less a reserve, the least over its
//! cgroup and those above it), and is refused there as under `ulimit -v`.
//!
//! The address space also holds memory the process has reserved and not
//! yet written, which is not resident: the capacity a `Vec` has grown into,
//! a zeroed table not yet filled, address space the allocator has mapped
//! ahead. At a limit equal to the room, that memory failed runs gigabytes
//! below the cgroup's limit (36.7 GiB resident under 41.9 GiB). So where
//! the allocator is refused at the limit, the limit follows the room: it is
//! raised by half the room the cgroups still leave, and a request is
//! refused once that half is under it ([`raise`]). While the reserved
//! memory stays unwritten, each raise lets resident memory take half of
//! what is left, and the run is refused short of the reserve. The reserve,
//! a 64th of each cgroup's `memory.max` ([`KEPT`]), is for what the kernel
//! charges the cgroup outside the process's mappings (page tables, kernel
//! stacks and the like) and for reserved memory written after all: raised
//! by the whole room, or by half of it down to nothing, resident memory
//! reached `memory.max` and the kernel killed the run. The run itself takes
//! a thread whose stack is mapped whole as it starts ([`stack_bytes`]), so
//! that the stack never has to grow against the address-space limit.

#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::sync::OnceLock;

/// The share of each cgroup's `memory.max` that its room keeps back, for
/// the kernel's charges outside the process's mappings: a 64th.
#[cfg(target_os = "linux")]
const KEPT: u64 = 64;

/// The cgroups that bound the process's memory, as it started
/// ([`limit_to_cgroups`]).
#[cfg(target_os = "linux")]
static ROOMS: OnceLock<Rooms> = OnceLock::new();

/// Lowers the process's address-space limit to the room its cgroups
/// leave, where that is less, and keeps the cgroups for [`raise`]. Without
/// a cgroup v2 hierarchy, or where no cgroup sets `memory.max`, the limit
/// stays as it is and is never raised.
pub fn limit_to_cgroups() {
    #[cfg(target_os = "linux")]
    {
        let Some((own, root)) = own_cgroup() else {
            return;
        };
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: getrlimit writes the struct it is given.
        if unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut limit) } != 0 {
            return;
        }
        let Some(rooms) = Rooms::open(&own, &root, limit.rlim_cur) else {
            return;
        };
        if let Some(room) = rooms.room()
            && room < limit.rlim_cur
        {
            limit.rlim_cur = room;
            // SAFETY: setrlimit reads the struct it is given. A soft limit
            // below the hard one is always allowed.
            unsafe { libc::setrlimit(libc::RLIMIT_AS, &limit) };
        }
        let _ = ROOMS.set(rooms);
    }
}

/// Raises the address-space limit, where the allocator has refused a
/// request of `bytes` at it, by half the room the cgroups still leave
/// ([`Rooms::raised`]), and says whether it did. Allocates nothing: the
/// allocator calls it.
pub fn raise(bytes: usize) -> bool {
    #[cfg(target_os = "linux")]
    {
        let Some(rooms) = ROOMS.get() else {
            return false;
        };
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: getrlimit writes the struct it is given; setrlimit reads
        // it. The raised limit is at most the soft limit the process
        // started with, which is at most the hard one.
        unsafe {
            if libc::getrlimit(libc::RLIMIT_AS, &mut limit) != 0 {
                return false;
            }
            let Some(raised) = rooms.raised(limit.rlim_cur, bytes as u64) else {
                return false;
            };
            limit.rlim_cur = raised;
            libc::setrlimit(libc::RLIMIT_AS, &limit) == 0
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = bytes;
        false
    }
}

/// The stack of the thread that runs the command: the main thread's limit
/// (`RLIMIT_STACK`, `ulimit -s`), or 8 MiB, the usual one, where it has
/// none. The thread's stack is mapped whole as it starts, so that under a
/// limit on the address space it never has to grow (`main`).
pub fn stack_bytes() -> usize {
    const USUAL: usize = 8 << 20;
    #[cfg(target_os = "linux")]
    {
        let mut limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: getrlimit writes the struct it is given.
        let read = unsafe { libc::getrlimit(libc::RLIMIT_STACK, &mut limit) } == 0;
        if read && limit.rlim_cur != libc::RLIM_INFINITY {
            return usize::try_from(limit.rlim_cur).unwrap_or(USUAL);
        }
    }
    USUAL
}

/// The text of `/proc/self/cgroup` and the root of the cgroup v2
/// hierarchy. In a debug build, the directory `TDDLOG_TEST_CGROUP` names
/// stands for the process's cgroup, at the root of a hierarchy of its own,
/// for tests/it/out_of_memory.rs; a release build reads no such variable.
#[cfg(target_os = "linux")]
fn own_cgroup() -> Option<(String, PathBuf)> {
    #[cfg(debug_assertions)]
    if let Some(dir) = std::env::var_os("TDDLOG_TEST_CGROUP") {
        return Some(("0::/\n".into(), dir.into()));
    }
    let own = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    Some((own, "/sys/fs/cgroup".into()))
}

/// The cgroups whose `memory.max` bounds the process, from its own up,
/// with their `memory.max` and `memory.current` kept open, so that the
/// room they leave is read again without allocating; and the soft
/// address-space limit the process started with, which a raise never
/// passes.
#[cfg(target_os = "linux")]
struct Rooms {
    levels: Vec<(File, File)>,
    ceiling: u64,
}

#[cfg(target_os = "linux")]
impl Rooms {
    /// The cgroup v2 that `own` (the text of `/proc/self/cgroup`) names and
    /// every cgroup above it, under the hierarchy's root `root`: those that
    /// set `memory.max`, or `None` where none does; `ceiling` is the soft
    /// address-space limit the process started with.
    fn open(own: &str, root: &Path, ceiling: u64) -> Option<Rooms> {
        let path = own.lines().find_map(|line| line.strip_prefix("0::"))?;
        let mut dir = path.trim().trim_matches('/');
        let mut levels = Vec::new();
        loop {
            // `max` for no limit, and no file at the root.
            let at = root.join(dir);
            if let (Ok(max), Ok(current)) = (
                File::open(at.join("memory.max")),
                File::open(at.join("memory.current")),
            ) && number(&max).is_some()
            {
                levels.push((max, current));
            }
            if dir.is_empty() {
                break;
            }
            dir = dir.rfind('/').map_or("", |slash| &dir[..slash]);
        }
        (!levels.is_empty()).then_some(Rooms { levels, ceiling })
    }

    /// The least, over the cgroups, of `memory.max` less `memory.current`
    /// less the reserve ([`KEPT`]), as they read now. Allocates nothing.
    fn room(&self) -> Option<u64> {
        self.levels
            .iter()
            .filter_map(|(max, current)| {
                let max = number(max)?;
                Some(
                    max.saturating_sub(number(current)?)
                        .saturating_sub(max / KEPT),
                )
            })
            .min()
    }

    /// The address-space limit `limit`, at which a request of `bytes` was
    /// refused, raised by half the room the cgroups leave, up to the limit
    /// the process started with; `None` where that half is under the
    /// request or the limit cannot rise.
    fn raised(&self, limit: u64, bytes: u64) -> Option<u64> {
        let half = self.room()? / 2;
        let raised = limit.saturating_add(half).min(self.ceiling);
        (half >= bytes && raised > limit).then_some(raised)
    }
}

/// The number a cgroup's file starts with, read from its start into a
/// buffer on the stack, or `None` (`max`). Allocates nothing.
#[cfg(target_os = "linux")]
fn number(file: &File) -> Option<u64> {
    use std::os::unix::fs::FileExt;
    let mut text = [0u8; 24];
    let n = file.read_at(&mut text, 0).ok()?;
    let digits = &text[..text[..n].iter().take_while(|b| b.is_ascii_digit()).count()];
    if digits.is_empty() {
        return None;
    }
    digits.iter().try_fold(0u64, |value, &d| {
        value.checked_mul(10)?.checked_add(u64::from(d - b'0'))
    })
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::Rooms;
    use std::path::{Path, PathBuf};

    /// A directory of its own for a test's cgroup files, one for each call,
    /// since the tests run in parallel.
    fn hierarchy(name: &str) -> PathBuf {
        static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let process = std::process::id();
        let dir = std::env::temp_dir().join(format!("tddlog-budget-{process}-{name}-{call}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(root: &Path, files: &[(&str, &str)]) {
        for (path, text) in files {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
    }

    fn room(own: &str, files: &[(&str, &str)]) -> Option<u64> {
        let root = hierarchy("room");
        write(&root, files);
        let room = Rooms::open(own, &root, u64::MAX).and_then(|rooms| rooms.room());
        std::fs::remove_dir_all(&root).unwrap();
        room
    }

    #[test]
    fn the_room_is_the_least_over_the_cgroup_and_those_above_it() {
        let own = "0::/user.slice/run.scope/run-1\n";
        let files = [
            ("user.slice/memory.max", "max\n"),
            ("user.slice/memory.current", "9000\n"),
            ("user.slice/run.scope/memory.max", "6400\n"),
            ("user.slice/run.scope/memory.current", "2000\n"),
            ("user.slice/run.scope/run-1/memory.max", "5120\n"),
            ("user.slice/run.scope/run-1/memory.current", "400\n"),
        ];
        // 4400 left in the scope less its reserve of 100, 4720 in the
        // command's own cgroup less 80.
        assert_eq!(room(own, &files), Some(4300));
        // The command's own cgroup alone sets a limit.
        assert_eq!(room(own, &files[4..]), Some(4640));
        // A cgroup over its limit, or into its reserve, leaves nothing.
        for current in ["700", "635"] {
            let files = [
                ("user.slice/run.scope/run-1/memory.max", "640"),
                ("user.slice/run.scope/run-1/memory.current", current),
            ];
            assert_eq!(room(own, &files), Some(0));
        }
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
            Some(59)
        );
    }

    #[test]
    fn a_refused_limit_rises_by_the_room_the_cgroups_still_leave() {
        let root = hierarchy("raise");
        write(
            &root,
            &[("memory.max", "6400\n"), ("memory.current", "300\n")],
        );
        let rooms = Rooms::open("0::/\n", &root, 12000).unwrap();
        // Refused at 6000 with 6000 left past the reserve of 100: raised by
        // half the room, as it reads now.
        assert_eq!(rooms.raised(6000, 3000), Some(9000));
        write(&root, &[("memory.current", "4300\n")]);
        assert_eq!(rooms.raised(9000, 1000), Some(10000));
        // Half a room under the request is no reason to raise.
        assert_eq!(rooms.raised(10000, 1001), None);
        // Nor past the limit the process started with.
        assert_eq!(rooms.raised(11500, 1000), Some(12000));
        assert_eq!(rooms.raised(12000, 1000), None);
        // Nor into the reserve.
        write(&root, &[("memory.current", "6300\n")]);
        assert_eq!(rooms.raised(10000, 1), None);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
