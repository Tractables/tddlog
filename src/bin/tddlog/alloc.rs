//! The binary's allocator: jemalloc, with transparent huge pages from the
//! first [`HUGE_FROM`] bytes until the kernel's time says otherwise.

use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// jemalloc rather than glibc's malloc: faster on the benchmark programs at
// the same peak memory.
#[global_allocator]
static ALLOC: HugeLater = HugeLater;

// Transparent huge pages for jemalloc's extents, which the query phase of
// the large networks faults in by the gigabyte: faster again, at a few
// percent more peak memory.
//
// No purging while requests succeed: memory the program frees stays with
// jemalloc for reuse. By default jemalloc gives freed pages back to the
// kernel over about 10 s, and a later request faults them in again,
// zeroed: g30_100 took 141 s instead of 179, with 1 M page faults instead
// of 12 M, holding 4.2 GB resident instead of 2.2 inside the same address
// space.
//
// A free extent serves every request that fits in it. By default jemalloc
// splits a free extent it has kept only for a request of at least a 64th
// of its size: freed blocks join into extents of gigabytes, the smaller
// requests after them took new pages, and without a limit on the address
// space the kept pages grew without bound. After 360 s coinBias_w16 held
// 57 GB under the default and 0.85 GB under this option, at the same
// speed; in longer runs the default passed 200 GB.
//
// Kept free pages fill a limit on the address space as live ones do, and
// jemalloc joins the small pieces among them only when it gives them
// back, so a request that fails is tried again after a purge ([`purge`]):
// where auto has retired a trial, the memory it held then serves the
// trials that follow. One arena for blocks of every size: by default
// blocks of 8 MiB and more come from an arena of their own, whose free
// address space no other request can take. `_RJEM_MALLOC_CONF` overrides
// each option.
#[unsafe(export_name = "_rjem_malloc_conf")]
static MALLOC_CONF: &[u8; 96] =
    b"thp:always,oversize_threshold:0,dirty_decay_ms:-1,muzzy_decay_ms:-1,lg_extent_max_active_fit:64\0";

/// jemalloc, with the process's transparent huge pages off until it has
/// asked for [`HUGE_FROM`] bytes. A small program's first huge pages are
/// zeroed whole for the few kilobytes it writes (bn_cancer asks for
/// 0.3 MB), while bn_hailfinder, which asks for 294 MB, runs 3 to 5%
/// faster with them. Every request counts towards the bound, and
/// jemalloc keeps its advice on every extent.
///
/// Once on, they go off for good where the kernel stalls
/// ([`kernel_stalls`]): a fault in an advised mapping compacts memory at
/// once where the system's defrag setting is `madvise` (the default on
/// Ubuntu), and with free memory fragmented the compaction fails, the
/// fault takes a small page after all, and the run spends much of its
/// time there (weather_w4 beside large programs: 1.3 s of system time
/// against 1.44 s of user time, without one huge page; rus_20_40_0_2 of
/// the UAI 2022 PR set: 111.0 s against 9.2 s).
struct HugeLater;

/// The bytes asked for before huge pages go on, and between two looks at
/// the processor times once they are on.
const HUGE_FROM: usize = 16 << 20;
/// The bytes still to be asked for before the allocator next turns to the
/// huge pages ([`huge_turn`]). One count on every allocation's path, for
/// both turns: lost updates only move a turn, the program being
/// single-threaded in effect.
static HUGE_NEXT: AtomicUsize = AtomicUsize::new(HUGE_FROM);
/// Whether huge pages are on.
static HUGE_ON: AtomicBool = AtomicBool::new(false);

/// Turns the process's transparent huge pages off or on; the kernel then
/// gives advised mappings small or huge pages at their next faults.
fn huge_pages(on: bool) {
    #[cfg(target_os = "linux")]
    // SAFETY: PR_SET_THP_DISABLE takes a flag and no pointers.
    unsafe {
        libc::prctl(libc::PR_SET_THP_DISABLE, libc::c_ulong::from(!on), 0, 0, 0);
    }
    #[cfg(not(target_os = "linux"))]
    let _ = on;
}

/// Off before the first allocation: the runtime allocates before `main`.
#[cfg(target_os = "linux")]
#[used]
#[unsafe(link_section = ".init_array")]
static HUGE_PAGES_OFF: extern "C" fn() = {
    extern "C" fn off() {
        huge_pages(false);
    }
    off
};

#[inline(always)]
fn note(n: usize) {
    let left = HUGE_NEXT.load(Ordering::Relaxed);
    if n < left {
        HUGE_NEXT.store(left - n, Ordering::Relaxed);
    } else {
        huge_turn();
    }
}

/// Huge pages on after the first [`HUGE_FROM`] bytes; then a look at the
/// processor times every [`HUGE_FROM`] bytes, which turns them off for
/// good where the kernel stalls, after which the count runs down from
/// `usize::MAX` and never comes back here. Allocates nothing.
#[cold]
#[inline(never)]
fn huge_turn() {
    let next = if !HUGE_ON.swap(true, Ordering::Relaxed) {
        huge_pages(true);
        HUGE_FROM
    } else if kernel_stalls() {
        huge_pages(false);
        usize::MAX
    } else {
        HUGE_FROM
    };
    HUGE_NEXT.store(next, Ordering::Relaxed);
}

/// Whether the kernel has taken more than a tenth of a second of the
/// process's processor time, and more than a nanosecond for each byte the
/// process has held. Supplying memory costs the kernel about 0.1 ns a byte
/// in huge pages and 0.5 in small ones; a run whose huge pages fail
/// compaction pays 1.4 to 10. The tenth of a second keeps the first looks,
/// whose few clock ticks split the time coarsely, from turning huge pages
/// off on noise.
fn kernel_stalls() -> bool {
    #[cfg(target_os = "linux")]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
        // SAFETY: getrusage writes the struct it is given and allocates
        // nothing.
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
            return false;
        }
        // SAFETY: getrusage returned 0, so it filled the struct.
        let usage = unsafe { usage.assume_init() };
        let t = usage.ru_stime;
        let system_micros = t.tv_sec * 1_000_000 + t.tv_usec;
        // A microsecond a kilobyte (`ru_maxrss` counts kilobytes) is a
        // nanosecond a byte.
        system_micros > 100_000 && system_micros > usage.ru_maxrss
    }
    #[cfg(not(target_os = "linux"))]
    false
}

/// Gives the free pages of every arena back to the kernel, before a failed
/// request is tried again. Allocates nothing.
#[cold]
#[inline(never)]
fn purge() {
    // SAFETY: the name is a NUL-terminated mallctl name (4096 stands for
    // every arena), and a purge reads and writes no value.
    unsafe {
        tikv_jemalloc_sys::mallctl(
            c"arena.4096.purge".as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        );
    }
}

/// `request`'s block, or where jemalloc refuses it, `request`'s block
/// after a [`purge`].
#[inline(always)]
fn or_after_purge(request: impl Fn() -> *mut u8) -> *mut u8 {
    let block = request();
    if !block.is_null() {
        return block;
    }
    purge();
    request()
}

// SAFETY: every method forwards to jemalloc with the caller's arguments; a
// request jemalloc refuses leaves `ptr` to the caller, so it is made
// again.
unsafe impl GlobalAlloc for HugeLater {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note(layout.size());
        or_after_purge(|| unsafe { tikv_jemallocator::Jemalloc.alloc(layout) })
    }
    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note(layout.size());
        or_after_purge(|| unsafe { tikv_jemallocator::Jemalloc.alloc_zeroed(layout) })
    }
    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note(new_size);
        or_after_purge(|| unsafe { tikv_jemallocator::Jemalloc.realloc(ptr, layout, new_size) })
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { tikv_jemallocator::Jemalloc.dealloc(ptr, layout) }
    }
}
