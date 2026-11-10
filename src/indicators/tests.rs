use std::cell::Cell;
use std::collections::TryReserveError;

use crate::ground::{GroundOptions, ground};
use crate::syntax::parse_program;

thread_local! {
    /// The requests for the copies' room ([`super::Copies::make`]) granted
    /// before every later one is refused, where set: the tests' allocator
    /// refusing memory.
    static GRANTS: Cell<Option<usize>> = const { Cell::new(None) };
}

/// A request for the copies' room granted, or refused where [`GRANTS`] has
/// run out.
pub(super) fn grant() -> Result<(), TryReserveError> {
    GRANTS.with(|grants| match grants.get() {
        Some(0) => Err(refusal()),
        left => {
            grants.set(left.map(|k| k - 1));
            Ok(())
        }
    })
}

/// The allocator's refusal: room for more bytes than any allocation may
/// have, which is refused before anything is asked of the allocator.
fn refusal() -> TryReserveError {
    Vec::<u8>::new()
        .try_reserve(usize::MAX)
        .expect_err("more than isize::MAX bytes")
}

/// A chain of families, each reading the atom before it through a copy.
const SOURCE: &str = "0.3::a. 0.6::b. 0.2::d. 0.7::e. h :- a, b. q :- h, d. r :- q, e. query(r).";

/// The rewrite refused room for its copies, at each of its requests in turn
/// (the rewrite's atoms and their kinds, the copies' ties and their map),
/// gives up with the refusal, so that `auto` disables the trial whose
/// layout asked for it; where a request grew a list infallibly, a refusal
/// aborted the run. Granted every request, it is the rewrite made without
/// a limit.
#[test]
fn a_rewrite_refused_room_for_its_copies_gives_up() {
    // Its derived atoms read as written, not inlined (`GroundOptions::views`)
    // nor read as facts (`contexts`).
    let options = GroundOptions {
        views: false,
        contexts: false,
        ..GroundOptions::default()
    };
    let gp = ground(&parse_program(SOURCE).expect("parses"), &options).expect("grounds");
    let whole = super::introduce(&gp).expect("rewrites");
    assert!(
        !whole.encoding.is_empty(),
        "the rewrite reads copies: {whole}"
    );
    let whole = whole.to_string();
    for granted in 0.. {
        GRANTS.set(Some(granted));
        let rewrite = super::introduce(&gp);
        GRANTS.set(None);
        match rewrite {
            Err(_) => assert!(granted < 4, "refused with {granted} requests granted"),
            Ok(rewrite) => {
                assert_eq!(granted, 4, "the copies' requests");
                assert_eq!(rewrite.to_string(), whole);
                break;
            }
        }
    }
}

/// The rewrite holds no room past its lists' lengths: they grow by doubling
/// as its rules, links and evidence are written, and the compile holds the
/// rewrite to its end, where on a loop of 100 000 steps the room past the
/// rules and the evidence came to megabytes.
#[test]
fn the_rewrite_holds_no_room_past_its_lists() {
    let options = GroundOptions {
        views: false,
        contexts: false,
        ..GroundOptions::default()
    };
    let gp = ground(&parse_program(SOURCE).expect("parses"), &options).expect("grounds");
    let rewrite = super::introduce(&gp).expect("rewrites");
    assert!(!rewrite.encoding.is_empty(), "the rewrite reads copies");
    let room = [
        ("atoms", rewrite.atoms.capacity(), rewrite.atoms.len()),
        ("kinds", rewrite.kinds.capacity(), rewrite.kinds.len()),
        ("rules", rewrite.rules.capacity(), rewrite.rules.len()),
        (
            "evidence",
            rewrite.evidence.capacity(),
            rewrite.evidence.len(),
        ),
        (
            "indicators",
            rewrite.encoding.indicators.capacity(),
            rewrite.encoding.indicators.len(),
        ),
    ];
    for (list, capacity, len) in room {
        assert_eq!(capacity, len, "the room of the rewrite's {list}");
    }
}
