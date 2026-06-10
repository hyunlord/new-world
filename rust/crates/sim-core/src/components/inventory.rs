//! `Inventory` agent component (Direction-2 slice 2-1).
//!
//! A count map of carriable resources with a simple total-count capacity cap.
//! MVP uses `BTreeMap<ResourceKind, u32>` counts (not stateful `ItemInstance`s)
//! per the Gate-0 "counts, not items" decision — `ItemStore`/`ItemId` arrive in
//! Direction-3. `BTreeMap` keeps iteration deterministic.
//!
//! Slice 2-1 ships ONLY the data structure + its pure methods. Attaching
//! `Inventory` to agents at bootstrap/birth, and the pickup/store cascade, are
//! slice 2-2 scope — none of that exists here.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::components::resource_kind::ResourceKind;

/// Maximum total count (summed across all kinds) an [`Inventory`] can hold.
///
/// A single shared budget, NOT a per-kind cap (Gate-0 Q4 "simple total-count
/// cap"). Used by 2-2 partial-pickup logic via the overflow return of
/// [`Inventory::add`].
pub const INVENTORY_CAPACITY: u32 = 10;

/// Per-agent carriable-resource counts.
///
/// `items` maps each held [`ResourceKind`] to its count. Zero counts are never
/// stored — [`Inventory::remove`] drops a key the moment it reaches 0, keeping
/// the map minimal and iteration deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// Held resource counts, keyed by [`ResourceKind`] for deterministic
    /// iteration. Never holds a zero-count entry.
    pub items: BTreeMap<ResourceKind, u32>,
}

impl Inventory {
    /// Total count summed across all kinds. Uses `saturating_add` defensively;
    /// the capacity cap keeps the real total well below the `u32` ceiling.
    pub fn total(&self) -> u32 {
        self.items
            .values()
            .copied()
            .fold(0u32, |acc, c| acc.saturating_add(c))
    }

    /// Count held for `kind` (0 if the kind is absent).
    pub fn get(&self, kind: ResourceKind) -> u32 {
        self.items.get(&kind).copied().unwrap_or(0)
    }

    /// Add up to the remaining capacity (`INVENTORY_CAPACITY - total()`),
    /// incrementing `items[kind]`. Returns the overflow `n - taken` — the
    /// amount that did NOT fit (used by 2-2 partial pickup).
    ///
    /// Adding 0, or adding to a full inventory, is a no-op that returns `n`
    /// (no key is created when nothing is taken).
    pub fn add(&mut self, kind: ResourceKind, n: u32) -> u32 {
        let room = INVENTORY_CAPACITY.saturating_sub(self.total());
        let taken = n.min(room);
        if taken > 0 {
            *self.items.entry(kind).or_insert(0) += taken;
        }
        n - taken
    }

    /// Remove up to the held amount of `kind`. Returns the amount actually
    /// removed (`min(n, held)`). When the entry reaches 0 the key is dropped
    /// (no lingering zero entries). A zero `n`, or an absent key, is a no-op
    /// returning 0 without dropping a still-positive key.
    pub fn remove(&mut self, kind: ResourceKind, n: u32) -> u32 {
        match self.items.get_mut(&kind) {
            Some(count) => {
                let removed = n.min(*count);
                *count -= removed;
                if *count == 0 {
                    self.items.remove(&kind);
                }
                removed
            }
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let inv = Inventory::default();
        assert_eq!(inv.total(), 0);
        assert!(inv.items.is_empty());
        assert_eq!(inv.get(ResourceKind::Food), 0);
    }

    #[test]
    fn add_respects_capacity_and_returns_overflow() {
        let mut inv = Inventory::default();
        assert_eq!(inv.add(ResourceKind::Food, INVENTORY_CAPACITY), 0);
        assert_eq!(inv.total(), INVENTORY_CAPACITY);
        // full → all overflows
        assert_eq!(inv.add(ResourceKind::Water, 4), 4);
        assert_eq!(inv.total(), INVENTORY_CAPACITY);
    }

    #[test]
    fn add_accumulates_into_existing_key() {
        let mut inv = Inventory::default();
        inv.add(ResourceKind::Food, 2);
        inv.add(ResourceKind::Food, 3);
        assert_eq!(inv.get(ResourceKind::Food), 5);
    }

    #[test]
    fn add_zero_is_noop_no_key() {
        let mut inv = Inventory::default();
        assert_eq!(inv.add(ResourceKind::Food, 0), 0);
        assert!(inv.items.is_empty());
    }

    #[test]
    fn remove_returns_actual_and_drops_zero() {
        let mut inv = Inventory::default();
        inv.add(ResourceKind::Food, 5);
        assert_eq!(inv.remove(ResourceKind::Food, 5), 5);
        assert!(!inv.items.contains_key(&ResourceKind::Food));
    }

    #[test]
    fn remove_zero_keeps_positive_key() {
        let mut inv = Inventory::default();
        inv.add(ResourceKind::Wood, 3);
        assert_eq!(inv.remove(ResourceKind::Wood, 0), 0);
        assert_eq!(inv.get(ResourceKind::Wood), 3);
    }

    #[test]
    fn serde_round_trip() {
        let mut inv = Inventory::default();
        inv.add(ResourceKind::Food, 3);
        inv.add(ResourceKind::Wood, 2);
        let encoded = ron::to_string(&inv).unwrap();
        let decoded: Inventory = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, inv);
    }
}
