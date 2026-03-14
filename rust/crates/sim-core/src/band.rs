use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{BandId, EntityId};

/// A hunter-gatherer band or provisional party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Band {
    /// Stable band identifier.
    pub id: BandId,
    /// Display name of the band.
    pub name: String,
    /// Current member entity IDs.
    pub members: Vec<EntityId>,
    /// Current leader when elected.
    pub leader: Option<EntityId>,
    /// Tick when the provisional group formed.
    pub provisional_since: u64,
    /// Tick when the band was promoted from provisional to established.
    pub promoted_tick: Option<u64>,
    /// True when the band has been promoted from provisional status.
    pub is_promoted: bool,
}

impl Band {
    /// Creates a new provisional band.
    pub fn new(id: BandId, name: String, members: Vec<EntityId>, tick: u64) -> Self {
        Self {
            id,
            name,
            members,
            leader: None,
            provisional_since: tick,
            promoted_tick: None,
            is_promoted: false,
        }
    }

    /// Returns the number of members.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Returns true if the entity is a member of the band.
    pub fn contains(&self, entity: EntityId) -> bool {
        self.members.contains(&entity)
    }

    /// Adds one member if they are not already present.
    pub fn add_member(&mut self, entity: EntityId) {
        if !self.contains(entity) {
            self.members.push(entity);
            self.members.sort_by_key(|member| member.0);
        }
    }

    /// Removes one member if present.
    pub fn remove_member(&mut self, entity: EntityId) {
        self.members.retain(|member| *member != entity);
    }
}

/// Central registry of all provisional and promoted bands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BandStore {
    bands: BTreeMap<BandId, Band>,
    next_id: u64,
}

impl BandStore {
    /// Creates an empty band store.
    pub fn new() -> Self {
        Self {
            bands: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Allocates a stable monotonic band ID.
    pub fn allocate_id(&mut self) -> BandId {
        let id = BandId(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        id
    }

    /// Inserts or replaces one band.
    pub fn insert(&mut self, band: Band) {
        self.bands.insert(band.id, band);
    }

    /// Returns one band by ID.
    pub fn get(&self, id: BandId) -> Option<&Band> {
        self.bands.get(&id)
    }

    /// Returns one mutable band by ID.
    pub fn get_mut(&mut self, id: BandId) -> Option<&mut Band> {
        self.bands.get_mut(&id)
    }

    /// Removes and returns one band by ID.
    pub fn remove(&mut self, id: BandId) -> Option<Band> {
        self.bands.remove(&id)
    }

    /// Returns all bands in stable ID order.
    pub fn all(&self) -> impl Iterator<Item = &Band> {
        self.bands.values()
    }

    /// Returns all mutable bands in stable ID order.
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut Band> {
        self.bands.values_mut()
    }

    /// Returns the number of stored bands.
    pub fn len(&self) -> usize {
        self.bands.len()
    }

    /// Returns true when the store is empty.
    pub fn is_empty(&self) -> bool {
        self.bands.is_empty()
    }

    /// Returns the ID of the first band containing the entity.
    pub fn find_band_for(&self, entity: EntityId) -> Option<BandId> {
        self.bands
            .iter()
            .find(|(_, band)| band.contains(entity))
            .map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_tracks_members_without_duplicates() {
        let mut band = Band::new(BandId(1), "Oak".to_string(), vec![EntityId(2)], 12);
        band.add_member(EntityId(3));
        band.add_member(EntityId(2));
        assert_eq!(band.member_count(), 2);
        assert!(band.contains(EntityId(3)));
    }

    #[test]
    fn band_remove_member_drops_entity() {
        let mut band = Band::new(
            BandId(1),
            "Oak".to_string(),
            vec![EntityId(2), EntityId(3)],
            12,
        );
        band.remove_member(EntityId(2));
        assert!(!band.contains(EntityId(2)));
        assert_eq!(band.member_count(), 1);
    }

    #[test]
    fn band_store_allocates_monotonic_ids() {
        let mut store = BandStore::new();
        let a = store.allocate_id();
        let b = store.allocate_id();
        assert!(a < b);
    }

    #[test]
    fn band_store_insert_find_and_remove() {
        let mut store = BandStore::new();
        let id = store.allocate_id();
        store.insert(Band::new(id, "Oak".to_string(), vec![EntityId(7)], 3));

        assert_eq!(store.find_band_for(EntityId(7)), Some(id));
        assert!(store.get(id).is_some());
        assert!(store.remove(id).is_some());
        assert!(store.get(id).is_none());
    }
}
