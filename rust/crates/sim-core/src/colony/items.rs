//! Physical items (M1): kinds, ground stacks, carried loads and the conservation ledger.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What an item is. M1 has logs only; M2 adds materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// A felled tree's trunk section; the hut's material.
    Log,
}

impl ItemKind {
    /// Stable code for the bridge's packed columns (0 = log).
    pub const fn code(self) -> u8 {
        match self {
            Self::Log => 0,
        }
    }

    /// Stable lowercase name for logs, scripts and summaries.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Log => "log",
        }
    }
}

/// Items of one kind lying on one tile (one stack per tile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemStack {
    /// What the stack holds.
    pub kind: ItemKind,
    /// How many; a stored stack is never 0.
    pub count: u16,
}

/// Items an agent holds in its hands (an ECS component).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Carry {
    /// What is carried.
    pub kind: ItemKind,
    /// How many; never 0 while the component exists.
    pub count: u16,
}

/// Every item ever created and consumed, per kind (7.6). Conservation:
/// `created == on the ground + carried + delivered to sites + consumed`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemLedger {
    /// Items brought into the world (a felled tree's logs).
    pub created: BTreeMap<ItemKind, u64>,
    /// Items used up (a finished hut's logs).
    pub consumed: BTreeMap<ItemKind, u64>,
}

impl ItemLedger {
    /// Record `n` new items of `kind`.
    pub fn create(&mut self, kind: ItemKind, n: u64) {
        *self.created.entry(kind).or_insert(0) += n;
    }

    /// Record `n` items of `kind` used up.
    pub fn consume(&mut self, kind: ItemKind, n: u64) {
        *self.consumed.entry(kind).or_insert(0) += n;
    }

    /// Items of `kind` created so far.
    pub fn created_of(&self, kind: ItemKind) -> u64 {
        self.created.get(&kind).copied().unwrap_or(0)
    }

    /// Items of `kind` consumed so far.
    pub fn consumed_of(&self, kind: ItemKind) -> u64 {
        self.consumed.get(&kind).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_counts_per_kind_and_codes_are_stable() {
        let mut ledger = ItemLedger::default();
        assert_eq!(ledger.created_of(ItemKind::Log), 0);
        ledger.create(ItemKind::Log, 4);
        ledger.create(ItemKind::Log, 3);
        ledger.consume(ItemKind::Log, 2);
        assert_eq!(ledger.created_of(ItemKind::Log), 7);
        assert_eq!(ledger.consumed_of(ItemKind::Log), 2);
        assert_eq!((ItemKind::Log.code(), ItemKind::Log.name()), (0, "log"));
    }
}
