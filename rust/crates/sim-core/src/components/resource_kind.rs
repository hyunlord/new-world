//! `ResourceKind` carry/store key (Direction-2 slice 2-1).
//!
//! Unified key for resources that agents carry in an [`Inventory`] and that
//! settlements hold in their `stockpile`. Spans need-resources (Food/Water)
//! and raw materials (Wood/Stone) under one orderable key.
//!
//! Deliberately separate from:
//! - `MaterialId` — a material is a *substance* with derived item stats; food
//!   is not a material, so reusing `MaterialId` would be a category error.
//! - `TargetKind` — a behaviour *goal*, not a carriable quantity.
//!
//! `Ord` is REQUIRED: `ResourceKind` keys a `BTreeMap` (in both `Inventory`
//! and `Settlement::stockpile`), so iteration order must be deterministic —
//! the derived `Ord` on a fieldless enum is its declaration order
//! (`Food < Water < Wood < Stone`).
//!
//! Sleep is intentionally NOT a variant: rest is a *place* (non-depleting),
//! not a carry/store target. Direction-3 may add a `Material(MaterialId)`
//! bridge variant; it is deliberately excluded from slice 2-1.
//!
//! [`Inventory`]: crate::components::inventory::Inventory

use serde::{Deserialize, Serialize};

/// Carry/store resource key. Declaration order defines the `Ord` total order
/// used for deterministic `BTreeMap` iteration: `Food < Water < Wood < Stone`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Edible resource satisfying the Hunger need.
    Food,
    /// Drinkable resource satisfying the Thirst need.
    Water,
    /// Raw material — gathered, carried, stockpiled, used in construction.
    Wood,
    /// Raw material — gathered, carried, stockpiled, used in construction.
    Stone,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn ord_is_declaration_order() {
        assert!(ResourceKind::Food < ResourceKind::Water);
        assert!(ResourceKind::Water < ResourceKind::Wood);
        assert!(ResourceKind::Wood < ResourceKind::Stone);
    }

    #[test]
    fn btreemap_iterates_in_ord_order_regardless_of_insertion() {
        let mut m: BTreeMap<ResourceKind, u32> = BTreeMap::new();
        m.insert(ResourceKind::Stone, 1);
        m.insert(ResourceKind::Food, 1);
        m.insert(ResourceKind::Wood, 1);
        m.insert(ResourceKind::Water, 1);
        let keys: Vec<ResourceKind> = m.keys().copied().collect();
        assert_eq!(
            keys,
            vec![
                ResourceKind::Food,
                ResourceKind::Water,
                ResourceKind::Wood,
                ResourceKind::Stone
            ]
        );
    }

    #[test]
    fn serde_round_trip() {
        let k = ResourceKind::Wood;
        let encoded = ron::to_string(&k).unwrap();
        let decoded: ResourceKind = ron::from_str(&encoded).unwrap();
        assert_eq!(decoded, k);
    }
}
