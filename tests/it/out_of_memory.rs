//! A run that the allocator refuses memory, under a limit on the address
//! space (`ulimit -v`, or the room a cgroup's `memory.max` leaves, to which
//! the binary lowers it), ends with the error of a refused TDD operation and
//! exit status 1, also where the refused request is one that cannot give up,
//! on which std aborts. Under a cgroup, the limit rises by half the room the
//! cgroup still leaves before a request is refused, so that memory reserved
//! and not written does not fail a run below the cgroup's limit.
//!
//! A debug build of the binary makes requests that cannot give up where
//! `TDDLOG_TEST_ALLOCATE` lists them, and takes the directory
//! `TDDLOG_TEST_CGROUP` names for its cgroup; a release build has neither,
//! and these tests run on debug builds only.
#![cfg(all(target_os = "linux", debug_assertions))]

use std::os::unix::process::CommandExt;
use std::process::{Command, Output};

const GIB: u64 = 1 << 30;

/// The binary on a one-fact program, with the environment `env`, under the
/// soft address-space limit `limit` where one is given, else under its hard
/// limit.
fn run(name: &str, env: &[(&str, String)], limit: Option<libc::rlim_t>) -> Output {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("{name}.pl"));
    std::fs::write(&path, "0.5::a.\nquery(a).\n").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_tddlog"));
    command.arg(&path).envs(env.iter().map(|(k, v)| (k, v)));
    // SAFETY: getrlimit and setrlimit are async-signal-safe and allocate
    // nothing.
    unsafe {
        command.pre_exec(move || {
            let mut rl = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::getrlimit(libc::RLIMIT_AS, &mut rl) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            rl.rlim_cur = limit.unwrap_or(rl.rlim_max);
            if libc::setrlimit(libc::RLIMIT_AS, &rl) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    command.output().unwrap()
}

/// A directory that stands for the binary's cgroup, its `memory.max` and
/// `memory.current` as given.
fn cgroup(name: &str, max: u64, current: u64) -> String {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("memory.max"), format!("{max}\n")).unwrap();
    std::fs::write(dir.join("memory.current"), format!("{current}\n")).unwrap();
    dir.to_str().unwrap().to_string()
}

/// Asserts that `out` is the run's end as a refused request of `bytes`
/// that cannot give up: std's line for the request, then the error of a
/// refused TDD operation and its exit status, not SIGABRT.
fn assert_refused(out: &Output, bytes: u64) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    let refused = tddlog::Error::Compile(tddlog::tp::CompileError::Tdd(
        tididi::OperationError::OverBudget,
    ));
    assert!(
        stderr.contains(&format!("memory allocation of {bytes} bytes failed")),
        "{stderr}"
    );
    assert!(stderr.ends_with(&format!("{refused}\n")), "{stderr}");
    assert_eq!(out.status.code(), Some(1), "{:?}: {stderr}", out.status);
    assert!(out.stdout.is_empty());
}

#[test]
fn a_refused_request_that_cannot_give_up_ends_the_run_with_the_allocation_error() {
    // Room for the binary and its run's stack, half the request.
    let out = run(
        "out_of_memory",
        &[("TDDLOG_TEST_ALLOCATE", GIB.to_string())],
        Some(GIB / 2),
    );
    assert_refused(&out, GIB);
}

#[test]
fn memory_reserved_within_the_cgroups_room_is_not_refused() {
    // Blocks of 1 GiB and 0.5 GiB, reserved and not written, in a cgroup
    // of 1.5 GiB: the binary lowers its limit to the room, 1.5 GiB less
    // the reserve of a 64th, is refused the second block there, and raises
    // the limit by half the room the cgroup still shows.
    let dir = cgroup("out_of_memory_room", 3 * GIB / 2, 0);
    let out = run(
        "out_of_memory_room",
        &[
            ("TDDLOG_TEST_ALLOCATE", format!("{GIB},{}", GIB / 2)),
            ("TDDLOG_TEST_CGROUP", dir),
        ],
        None,
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{:?}: {stderr}", out.status);
    assert!(String::from_utf8_lossy(&out.stdout).contains("a:"));
}

#[test]
fn a_request_over_the_cgroups_room_is_refused() {
    // Under 1 GiB of room left, half of it under the request of 1 GiB: no
    // raise.
    let dir = cgroup("out_of_memory_full", 2 * GIB, GIB);
    let out = run(
        "out_of_memory_full",
        &[
            ("TDDLOG_TEST_ALLOCATE", GIB.to_string()),
            ("TDDLOG_TEST_CGROUP", dir),
        ],
        None,
    );
    assert_refused(&out, GIB);
}
