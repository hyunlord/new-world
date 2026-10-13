//! Lineage keys (Pass 9): a resident's identity for keyed random draws.
//!
//! `AgentId` is the engine's monotonic id: a birth added or removed anywhere
//! (or any other spawn) shifts every later id. A lineage key instead derives
//! from worldgen spawn order (founders) or from the first parent's key and its
//! birth count (newborns), so it never reads the global id counter. Attached
//! only when `AgentRules::keyed_draws` is set; inert otherwise.
use serde::{Deserialize, Serialize};

use crate::rng::{keyed_u64, STREAM_FOUNDER, STREAM_NEWBORN};

/// A resident's lineage key and the number of children it has parented as
/// the first parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    /// The key every keyed draw of this resident uses as its subject.
    pub key: u64,
    /// Children born with this resident as the first parent so far.
    pub births: u32,
}

impl Lineage {
    /// The founder spawned `founder_index`-th by worldgen in a world of `world_seed`.
    pub const fn founder(world_seed: u64, founder_index: u32) -> Self {
        Self { key: keyed_u64(world_seed, STREAM_FOUNDER, 0, 0, founder_index, 0), births: 0 }
    }

    /// The next child of this (first) parent; advances `births`.
    pub fn child(&mut self, world_seed: u64) -> Self {
        let child = Self { key: keyed_u64(world_seed, STREAM_NEWBORN, self.key, 0, self.births, 0), births: 0 };
        self.births += 1;
        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn founders_depend_on_seed_and_index_only() {
        assert_eq!(Lineage::founder(5, 3), Lineage::founder(5, 3));
        assert_ne!(Lineage::founder(5, 3).key, Lineage::founder(5, 4).key);
        assert_ne!(Lineage::founder(5, 3).key, Lineage::founder(6, 3).key);
    }

    #[test]
    fn children_follow_the_parent_and_its_birth_count() {
        let mut a = Lineage::founder(1, 0);
        let first = a.child(1);
        let second = a.child(1);
        assert_eq!(a.births, 2);
        assert_ne!(first.key, second.key);
        let mut again = Lineage::founder(1, 0);
        assert_eq!(again.child(1), first);
    }
}
