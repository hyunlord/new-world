//! `MaterialBlockingCache` — pre-computed `Material × Channel → blocking`
//! lookup table.
//!
//! Phase 0 Section 2.9.2 base. Mirrors the T6.6.2 `warm_cache` pattern:
//! pay one-time `O(materials × channels)` work, get O(1) lookup forever.
//!
//! v0.1.1 ISSUE 7 fix: this struct is passed by reference to every
//! propagation routine — there is no global static.

use ahash::AHashMap;

use crate::influence::channel::InfluenceChannel;
use crate::material::{MaterialId, MaterialRegistry};

/// Lookup table for the wall-blocking coefficient of each
/// `(MaterialId, InfluenceChannel)` pair.
///
/// Sized for 105 materials × 8 channels = 840 entries by default; growth
/// is bounded by `registry.count() * COUNT`.
pub struct MaterialBlockingCache {
    cache: AHashMap<(MaterialId, InfluenceChannel), f32>,
}

impl MaterialBlockingCache {
    /// Build the cache by querying every material in the registry for its
    /// blocking coefficient on every channel.
    pub fn build(registry: &MaterialRegistry) -> Self {
        let mut cache =
            AHashMap::with_capacity(registry.count() * InfluenceChannel::COUNT);

        for material_id in registry.all_ids() {
            // `get` returns `Option<&MaterialDef>`; only proceed for hits.
            // Missing entries fall back to `0.0` via `get` below.
            if let Some(material) = registry.get(material_id) {
                for channel in InfluenceChannel::all() {
                    cache.insert(
                        (material_id, *channel),
                        material.influence_blocking(*channel),
                    );
                }
            }
        }

        Self { cache }
    }

    /// Build an empty cache (useful for tests that want to drive
    /// propagation through an explicitly empty wall world).
    pub fn empty() -> Self {
        Self {
            cache: AHashMap::new(),
        }
    }

    /// Lookup the blocking coefficient (`0.0` = no blocking, `1.0` =
    /// total block). Missing entries return `0.0`.
    #[inline]
    pub fn get(&self, mat: MaterialId, ch: InfluenceChannel) -> f32 {
        *self.cache.get(&(mat, ch)).unwrap_or(&0.0)
    }

    /// Number of `(material, channel)` entries currently cached.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// `true` if the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MaterialCategory, MaterialDef};
    use crate::material::properties::test_support::valid_props;

    fn sample_material(name: &str) -> MaterialDef {
        MaterialDef {
            id: MaterialId::from_str_hash(name),
            name: name.to_string(),
            category: MaterialCategory::Stone,
            properties: valid_props(),
            tier: 1,
            natural_in: vec![],
            mod_source: None,
        }
    }

    #[test]
    fn test_empty_cache_returns_zero() {
        let cache = MaterialBlockingCache::empty();
        let mid = MaterialId::from_str_hash("does_not_exist");
        assert_eq!(cache.get(mid, InfluenceChannel::Light), 0.0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_build_from_registry_populates_all_pairs() {
        let mut registry = MaterialRegistry::new();
        registry.register(sample_material("alpha"), None).expect("reg");
        registry.register(sample_material("beta"), None).expect("reg");

        let cache = MaterialBlockingCache::build(&registry);
        // 2 materials × 8 channels = 16 entries.
        assert_eq!(cache.len(), 2 * InfluenceChannel::COUNT);

        let mid_alpha = MaterialId::from_str_hash("alpha");
        for ch in InfluenceChannel::all() {
            // Each pair must be present (lookup returns the cached value).
            // Missing keys would also return 0.0, so we additionally
            // assert `len()` above.
            let _ = cache.get(mid_alpha, *ch);
        }
    }

    #[test]
    fn test_get_missing_material_returns_zero() {
        let mut registry = MaterialRegistry::new();
        registry.register(sample_material("alpha"), None).expect("reg");
        let cache = MaterialBlockingCache::build(&registry);
        let mid_missing = MaterialId::from_str_hash("missing");
        assert_eq!(cache.get(mid_missing, InfluenceChannel::Light), 0.0);
    }

    #[test]
    fn test_light_blocking_is_one_for_solid() {
        let mut registry = MaterialRegistry::new();
        registry.register(sample_material("granite"), None).expect("reg");
        let cache = MaterialBlockingCache::build(&registry);
        let mid = MaterialId::from_str_hash("granite");
        // Light is binary 1.0 for any solid wall material.
        assert_eq!(cache.get(mid, InfluenceChannel::Light), 1.0);
    }

    #[test]
    fn test_danger_and_social_pass_through() {
        let mut registry = MaterialRegistry::new();
        registry.register(sample_material("granite"), None).expect("reg");
        let cache = MaterialBlockingCache::build(&registry);
        let mid = MaterialId::from_str_hash("granite");
        // Danger and Social spread regardless of walls.
        assert_eq!(cache.get(mid, InfluenceChannel::Danger), 0.0);
        assert_eq!(cache.get(mid, InfluenceChannel::Social), 0.0);
    }
}
