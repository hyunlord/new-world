//! Allocator byte counter (M5 goal §4, §5 item 4): measurement only.
//!
//! [`Counting`] is a `GlobalAlloc` that forwards every call to
//! [`std::alloc::System`] and keeps process-wide counters of the live heap
//! bytes and of the allocations made. It is installed as the
//! `#[global_allocator]` of `worldsim_perf` only (the standing benchmark,
//! `sim_test::bench`, reads [`live_bytes`] beside the RSS); the game, the
//! bridge and the probe never install it, so in every other binary the
//! counters stay 0. Nothing here feeds the simulation: the counters are
//! relaxed atomics read only by the benchmark.
//!
//! The `GlobalAlloc` contract requires `unsafe`; the counter lives in
//! `sim-test`, not in `sim-core` (the goal's `sim_core::alloc`), because
//! `sim-core` forbids `unsafe` code (M5 DECISIONS D4). Every unsafe call
//! forwards the caller's arguments to `System` unchanged.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static LIVE: AtomicU64 = AtomicU64::new(0);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

/// The counting allocator: forwards to [`System`] and tracks live bytes.
/// Use it only as `#[global_allocator] static ALLOC: Counting = Counting;`
/// in a measurement binary.
#[derive(Debug, Default, Clone, Copy)]
pub struct Counting;

fn add(bytes: usize) {
    LIVE.fetch_add(bytes as u64, Ordering::Relaxed);
}

fn sub(bytes: usize) {
    LIVE.fetch_sub(bytes as u64, Ordering::Relaxed);
}

// SAFETY: every method forwards its arguments unchanged to `System`, which
// upholds the `GlobalAlloc` contract; the counters never touch the memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's layout is passed through unchanged.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            add(layout.size());
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's layout is passed through unchanged.
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            add(layout.size());
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was allocated by this allocator (hence `System`) with `layout`.
        unsafe { System.dealloc(ptr, layout) };
        sub(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller's pointer, layout and size are passed through unchanged.
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            // On failure the old block stays allocated and the counters stay put.
            if new_size >= layout.size() {
                add(new_size - layout.size());
            } else {
                sub(layout.size() - new_size);
            }
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        new
    }
}

/// Live heap bytes held through [`Counting`] (0 when it is not the global
/// allocator of this process).
pub fn live_bytes() -> u64 {
    LIVE.load(Ordering::Relaxed)
}

/// Allocations (including reallocations) made through [`Counting`] since the
/// process started (0 when it is not the global allocator).
pub fn allocations() -> u64 {
    ALLOCATIONS.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_counter_tracks_alloc_realloc_and_dealloc() {
        // The test binary does not install `Counting`, so call it directly;
        // other tests in the process do not touch these counters.
        let before = live_bytes();
        let calls = allocations();
        let a = Counting;
        let Ok(layout) = Layout::from_size_align(64, 8) else {
            panic!("layout");
        };
        // SAFETY: a non-zero-sized layout; the block is freed below with the
        // layout it currently has.
        unsafe {
            let p = a.alloc(layout);
            assert!(!p.is_null());
            assert_eq!(live_bytes() - before, 64);
            let q = a.realloc(p, layout, 200);
            assert!(!q.is_null());
            assert_eq!(live_bytes() - before, 200);
            let Ok(bigger) = Layout::from_size_align(200, 8) else {
                panic!("layout");
            };
            let r = a.realloc(q, bigger, 40);
            assert!(!r.is_null());
            assert_eq!(live_bytes() - before, 40);
            let Ok(smaller) = Layout::from_size_align(40, 8) else {
                panic!("layout");
            };
            a.dealloc(r, smaller);
            let z = a.alloc_zeroed(layout);
            assert!(!z.is_null());
            assert_eq!(*z, 0);
            assert_eq!(live_bytes() - before, 64);
            a.dealloc(z, layout);
        }
        assert_eq!(live_bytes(), before);
        assert_eq!(allocations() - calls, 4);
    }
}
