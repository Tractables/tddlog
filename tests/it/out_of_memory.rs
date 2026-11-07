//! A run that the allocator refuses memory, under a limit on the address
//! space (`ulimit -v`, or the room a cgroup's `memory.max` leaves, to which
//! the binary lowers it), ends with the error of a refused TDD operation and
//! exit status 1, also where the refused request is one that cannot give up,
//! on which std aborts. A debug build of the binary makes such a request
//! where `TDDLOG_TEST_ALLOCATE` asks for one; a release build has no such
//! request, and these tests run on debug builds only.
#![cfg(all(target_os = "linux", debug_assertions))]

use std::os::unix::process::CommandExt;
use std::process::Command;

/// The address space the run is given: room for the binary and its run's
/// stack, half the request below.
const LIMIT: libc::rlim_t = 512 << 20;

#[test]
fn a_refused_request_that_cannot_give_up_ends_the_run_with_the_allocation_error() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("out_of_memory.pl");
    std::fs::write(&path, "0.5::a.\nquery(a).\n").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_tddlog"));
    command
        .arg(&path)
        .env("TDDLOG_TEST_ALLOCATE", (1u64 << 30).to_string());
    // SAFETY: setrlimit is async-signal-safe and allocates nothing.
    unsafe {
        command.pre_exec(|| {
            let limit = libc::rlimit {
                rlim_cur: LIMIT,
                rlim_max: libc::RLIM_INFINITY,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &limit) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    let out = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    // std's line for the request, then the error of a refused TDD operation
    // and its exit status, not SIGABRT.
    let refused = tddlog::Error::Compile(tddlog::tp::CompileError::Tdd(
        tididi::OperationError::OverBudget,
    ));
    assert!(
        stderr.contains("memory allocation of 1073741824 bytes failed"),
        "{stderr}"
    );
    assert!(stderr.ends_with(&format!("{refused}\n")), "{stderr}");
    assert_eq!(out.status.code(), Some(1), "{:?}: {stderr}", out.status);
    assert!(out.stdout.is_empty());
}
