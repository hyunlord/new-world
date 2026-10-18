//! Pass 18 sub-system timing, compiled only with the `subprofile` feature
//! (never on by default). Scopes read `Instant` and add elapsed nanoseconds
//! to a thread-local accumulator; nothing here reads or writes simulation
//! state. A driver calls [`take`] after every tick to collect that tick's
//! totals. Counters (`calls` without time) use [`count`].
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::time::Instant;

/// One scope's totals within the current tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeTotal {
    /// Summed elapsed nanoseconds (0 for pure counters).
    pub ns: u64,
    /// Number of times the scope ran, or the counter's summed value.
    pub calls: u64,
}

thread_local! {
    static TOTALS: RefCell<BTreeMap<&'static str, ScopeTotal>> = const { RefCell::new(BTreeMap::new()) };
    static MOVES: Cell<u64> = const { Cell::new(0) };
}

/// A started scope; [`Timer::stop`] records it.
#[derive(Debug)]
pub struct Timer(Instant);

impl Timer {
    /// Start timing.
    pub fn start() -> Self {
        Self(Instant::now())
    }

    /// Record the elapsed time under `name`.
    pub fn stop(self, name: &'static str) {
        let ns = u64::try_from(self.0.elapsed().as_nanos()).unwrap_or(u64::MAX);
        TOTALS.with(|totals| {
            let mut totals = totals.borrow_mut();
            let entry = totals.entry(name).or_default();
            entry.ns = entry.ns.saturating_add(ns);
            entry.calls += 1;
        });
    }
}

/// Add `n` to the counter `name` (no time).
pub fn count(name: &'static str, n: u64) {
    TOTALS.with(|totals| totals.borrow_mut().entry(name).or_default().calls += n);
}

/// Note one archetype move when `moved` (an insert of a component type the
/// entity lacked, or a remove of one it had).
pub fn note_move(moved: bool) {
    if moved {
        MOVES.with(|moves| moves.set(moves.get() + 1));
    }
}

/// Move the moves noted since the last flush to the counter `name`.
pub fn flush_moves(name: &'static str) {
    let n = MOVES.with(|moves| moves.replace(0));
    count(name, n);
}

/// This tick's totals; resets the accumulator.
pub fn take() -> BTreeMap<&'static str, ScopeTotal> {
    MOVES.with(|moves| moves.set(0));
    TOTALS.with(|totals| std::mem::take(&mut *totals.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_and_counters_accumulate_until_taken() {
        let _ = take();
        Timer::start().stop("a");
        Timer::start().stop("a");
        count("b", 3);
        note_move(true);
        note_move(false);
        note_move(true);
        flush_moves("m");
        let totals = take();
        assert_eq!(totals["a"].calls, 2);
        assert_eq!(totals["b"], ScopeTotal { ns: 0, calls: 3 });
        assert_eq!(totals["m"].calls, 2);
        assert!(take().is_empty());
    }
}
