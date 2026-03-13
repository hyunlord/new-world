use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::item::{ItemId, ItemInstance, ItemOwner};

/// Central item registry. All item instances in the world live here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemStore {
    /// All item instances keyed by `ItemId`.
    items: BTreeMap<ItemId, ItemInstance>,
    /// Next item ID to allocate.
    next_id: u64,
    /// Reverse index: owner → item IDs.
    owner_index: BTreeMap<ItemOwner, Vec<ItemId>>,
    /// Ground tile index: tile → item IDs.
    ground_index: BTreeMap<(i32, i32), Vec<ItemId>>,
}

impl ItemStore {
    /// Creates an empty item store.
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            next_id: 1,
            owner_index: BTreeMap::new(),
            ground_index: BTreeMap::new(),
        }
    }

    /// Allocates a stable monotonic item identifier.
    pub fn allocate_id(&mut self) -> ItemId {
        let id = ItemId(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        id
    }

    /// Inserts one item and updates all reverse indices.
    pub fn insert(&mut self, item: ItemInstance) -> ItemId {
        let id = item.id;
        let owner = item.owner.clone();
        self.items.insert(id, item);
        self.push_owner_index(owner.clone(), id);
        if let ItemOwner::Ground(x, y) = owner {
            self.push_ground_index((x, y), id);
        }
        id
    }

    /// Removes one item and cleans up all reverse indices.
    pub fn remove(&mut self, id: ItemId) -> Option<ItemInstance> {
        let item = self.items.remove(&id)?;
        self.remove_owner_reference(&item.owner, id);
        if let ItemOwner::Ground(x, y) = item.owner {
            self.remove_ground_reference((x, y), id);
        }
        Some(item)
    }

    /// Returns an immutable reference to one item.
    pub fn get(&self, id: ItemId) -> Option<&ItemInstance> {
        self.items.get(&id)
    }

    /// Returns a mutable reference to one item.
    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut ItemInstance> {
        self.items.get_mut(&id)
    }

    /// Transfers ownership and updates reverse indices.
    pub fn transfer_owner(&mut self, id: ItemId, new_owner: ItemOwner) -> bool {
        let Some(old_owner) = self.items.get(&id).map(|item| item.owner.clone()) else {
            return false;
        };

        if old_owner == new_owner {
            return true;
        }

        self.remove_owner_reference(&old_owner, id);
        if let ItemOwner::Ground(x, y) = old_owner {
            self.remove_ground_reference((x, y), id);
        }

        if let Some(item) = self.items.get_mut(&id) {
            item.owner = new_owner.clone();
        } else {
            return false;
        }

        self.push_owner_index(new_owner.clone(), id);
        if let ItemOwner::Ground(x, y) = new_owner {
            self.push_ground_index((x, y), id);
        }

        true
    }

    /// Returns item IDs owned by the given owner.
    pub fn items_owned_by(&self, owner: &ItemOwner) -> &[ItemId] {
        self.owner_index
            .get(owner)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns item IDs lying on the ground at a given tile.
    pub fn items_at_ground(&self, x: i32, y: i32) -> &[ItemId] {
        self.ground_index
            .get(&(x, y))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Total item count.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true when the store is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn push_owner_index(&mut self, owner: ItemOwner, id: ItemId) {
        self.owner_index.entry(owner).or_default().push(id);
    }

    fn push_ground_index(&mut self, tile: (i32, i32), id: ItemId) {
        self.ground_index.entry(tile).or_default().push(id);
    }

    fn remove_owner_reference(&mut self, owner: &ItemOwner, id: ItemId) {
        let mut should_remove = false;
        if let Some(entries) = self.owner_index.get_mut(owner) {
            entries.retain(|entry_id| *entry_id != id);
            should_remove = entries.is_empty();
        }
        if should_remove {
            self.owner_index.remove(owner);
        }
    }

    fn remove_ground_reference(&mut self, tile: (i32, i32), id: ItemId) {
        let mut should_remove = false;
        if let Some(entries) = self.ground_index.get_mut(&tile) {
            entries.retain(|entry_id| *entry_id != id);
            should_remove = entries.is_empty();
        }
        if should_remove {
            self.ground_index.remove(&tile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{EntityId, SettlementId};
    use crate::item::{EquipSlot, ItemDerivedStats};

    fn sample_item(id: ItemId, owner: ItemOwner) -> ItemInstance {
        ItemInstance {
            id,
            template_id: "raw_wood".to_string(),
            material_id: "oak".to_string(),
            derived_stats: ItemDerivedStats::default(),
            current_durability: 100.0,
            quality: 0.5,
            owner,
            stack_count: 1,
            created_tick: 12,
            creator_id: None,
            equipped_slot: None,
        }
    }

    #[test]
    fn item_id_is_monotonic() {
        let mut store = ItemStore::new();
        let a = store.allocate_id();
        let b = store.allocate_id();
        let c = store.allocate_id();
        assert!(a < b && b < c);
    }

    #[test]
    fn item_store_insert_and_get() {
        let mut store = ItemStore::new();
        let id = store.allocate_id();
        let item = sample_item(id, ItemOwner::Settlement(SettlementId(7)));
        store.insert(item.clone());

        let stored = store.get(id).expect("item should exist");
        assert_eq!(stored.template_id, "raw_wood");
        assert_eq!(
            store.items_owned_by(&ItemOwner::Settlement(SettlementId(7))),
            &[id]
        );
    }

    #[test]
    fn item_store_remove_cleans_indices() {
        let mut store = ItemStore::new();
        let id = store.allocate_id();
        store.insert(sample_item(id, ItemOwner::Ground(3, 4)));

        let removed = store.remove(id);
        assert!(removed.is_some());
        assert!(store.get(id).is_none());
        assert!(store.items_at_ground(3, 4).is_empty());
        assert!(store.items_owned_by(&ItemOwner::Ground(3, 4)).is_empty());
    }

    #[test]
    fn item_store_transfer_owner_updates_indices() {
        let mut store = ItemStore::new();
        let id = store.allocate_id();
        store.insert(sample_item(id, ItemOwner::Agent(EntityId(2))));

        assert!(store.transfer_owner(id, ItemOwner::Ground(8, 9)));
        assert!(store
            .items_owned_by(&ItemOwner::Agent(EntityId(2)))
            .is_empty());
        assert_eq!(store.items_at_ground(8, 9), &[id]);
        assert_eq!(
            store.get(id).expect("item exists").owner,
            ItemOwner::Ground(8, 9)
        );
    }

    #[test]
    fn item_store_ground_index_tracks_tiles() {
        let mut store = ItemStore::new();
        let a = store.allocate_id();
        let b = store.allocate_id();
        store.insert(sample_item(a, ItemOwner::Ground(1, 2)));
        let mut second = sample_item(b, ItemOwner::Ground(1, 2));
        second.equipped_slot = Some(EquipSlot::OffHand);
        store.insert(second);

        let items = store.items_at_ground(1, 2);
        assert_eq!(items.len(), 2);
        assert!(items.contains(&a));
        assert!(items.contains(&b));
    }

    #[test]
    fn item_store_empty_helpers_reflect_state() {
        let mut store = ItemStore::new();
        assert!(store.is_empty());
        let id = store.allocate_id();
        store.insert(sample_item(id, ItemOwner::Ground(0, 0)));
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }
}
