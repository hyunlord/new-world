//! `MaterialRegistry` — owns defs by id, by mod, plus derive/explain caches.
//!
//! All public lookups are O(1) average via `ahash::AHashMap`.

use ahash::AHashMap;

use crate::material::definition::MaterialDef;
use crate::material::derivation::{derive_all, AutoDerivedStats, DerivedStatKind};
use crate::material::error::MaterialError;
use crate::material::explanation::{explain, Explanation};
use crate::material::id::MaterialId;

/// Material registry. Owns defs and lazily caches per-material derivations
/// and per-stat explanations.
#[derive(Default)]
pub struct MaterialRegistry {
    defs: AHashMap<MaterialId, MaterialDef>,
    by_mod: AHashMap<String, Vec<MaterialId>>,
    derive_cache: AHashMap<MaterialId, AutoDerivedStats>,
    explain_cache: AHashMap<DerivedStatKind, Explanation>,

    /// Test-only counter incremented every time `derive` actually computes
    /// (cache miss). Used by harness assertion A23 to observe cache hits.
    #[cfg(test)]
    derive_call_count: usize,
}

impl MaterialRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a definition. Fails with `DuplicateId` if the id is already
    /// occupied; the existing def is preserved.
    pub fn register(
        &mut self,
        def: MaterialDef,
        mod_id: Option<&str>,
    ) -> Result<(), MaterialError> {
        if self.defs.contains_key(&def.id) {
            return Err(MaterialError::DuplicateId(def.id));
        }
        let id = def.id;
        if let Some(m) = mod_id {
            self.by_mod.entry(m.to_string()).or_default().push(id);
        }
        self.defs.insert(id, def);
        Ok(())
    }

    /// Lookup by id.
    pub fn get(&self, id: MaterialId) -> Option<&MaterialDef> {
        self.defs.get(&id)
    }

    /// Lazy-cached derivation. First call computes, subsequent calls hit
    /// the cache. Returns `None` if id is unknown.
    pub fn derive(&mut self, id: MaterialId) -> Option<&AutoDerivedStats> {
        if !self.defs.contains_key(&id) {
            return None;
        }
        if !self.derive_cache.contains_key(&id) {
            // Compute once.
            let def = self.defs.get(&id)?;
            let stats = derive_all(&def.properties);
            self.derive_cache.insert(id, stats);
            #[cfg(test)]
            {
                self.derive_call_count += 1;
            }
        }
        self.derive_cache.get(&id)
    }

    /// Lazy-cached explanation for a derived stat.
    pub fn explain(&mut self, stat: DerivedStatKind) -> &Explanation {
        self.explain_cache.entry(stat).or_insert_with(|| explain(stat))
    }

    /// Eagerly populate `derive_cache` and `explain_cache` for every
    /// currently-registered material and every stat kind.
    pub fn warm_cache(&mut self) {
        let ids: Vec<MaterialId> = self.defs.keys().copied().collect();
        for id in ids {
            let _ = self.derive(id);
        }
        for k in DerivedStatKind::all_variants() {
            let _ = self.explain(*k);
        }
    }

    /// Remove every def whose `mod_id` matches. Returns the number removed.
    /// Also flushes `derive_cache` entries for the removed ids.
    pub fn unload_mod(&mut self, mod_id: &str) -> usize {
        let Some(ids) = self.by_mod.remove(mod_id) else {
            return 0;
        };
        let mut removed = 0;
        for id in &ids {
            if self.defs.remove(id).is_some() {
                removed += 1;
            }
            self.derive_cache.remove(id);
        }
        removed
    }

    /// Iterator over every registered id.
    pub fn all_ids(&self) -> impl Iterator<Item = MaterialId> + '_ {
        self.defs.keys().copied()
    }

    /// Number of registered definitions.
    pub fn count(&self) -> usize {
        self.defs.len()
    }

    /// Test-only sentinel injection. Lets the harness verify that `derive`
    /// actually hits the cache (A23 option A).
    #[cfg(test)]
    pub(crate) fn cache_insert(&mut self, id: MaterialId, stats: AutoDerivedStats) {
        self.derive_cache.insert(id, stats);
    }

    /// Test-only counter accessor (A23 option B).
    #[cfg(test)]
    pub(crate) fn derive_call_count(&self) -> usize {
        self.derive_call_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::category::MaterialCategory;
    use crate::material::properties::test_support::valid_props;

    fn def_for(name: &str) -> MaterialDef {
        MaterialDef {
            id: MaterialId::from_str_hash(name),
            name: name.to_string(),
            category: MaterialCategory::Mineral,
            properties: valid_props(),
            tier: 1,
            natural_in: vec![],
            mod_source: None,
        }
    }

    #[test]
    fn register_and_get_roundtrip() {
        let mut r = MaterialRegistry::new();
        let def = def_for("test_iron");
        let id = def.id;
        r.register(def, None).expect("register ok");
        let back = r.get(id).expect("must exist");
        assert_eq!(back.id, MaterialId::from_str_hash("test_iron"));
        assert_eq!(back.name, "test_iron");
        assert_eq!(back.category, MaterialCategory::Mineral);
    }

    #[test]
    fn duplicate_id_rejected_no_overwrite() {
        let mut r = MaterialRegistry::new();
        let mut first = def_for("x");
        first.name = "first".to_string();
        let id = first.id;
        r.register(first, None).expect("first ok");

        let mut second = def_for("x");
        second.name = "second".to_string();
        let err = r.register(second, None).expect_err("duplicate must fail");
        match err {
            MaterialError::DuplicateId(d) => assert_eq!(d, id),
            other => panic!("expected DuplicateId, got {other:?}"),
        }
        assert_eq!(r.get(id).unwrap().name, "first");
    }

    #[test]
    fn derive_cache_hit_via_sentinel() {
        let mut r = MaterialRegistry::new();
        let def = def_for("sentinel_x");
        let id = def.id;
        r.register(def, None).unwrap();

        // First derive — computes and caches.
        let _ = r.derive(id).unwrap();

        // Inject sentinel pattern that derive_all would never produce.
        let bits = f64::from_bits(0xCAFE_BABE_DEAD_BEEF);
        let sentinel = AutoDerivedStats {
            axe_damage_blunt: bits,
            axe_damage_cut: bits,
            axe_durability: bits,
            axe_speed: bits,
            sword_damage_cut: bits,
            sword_durability: bits,
            spear_damage_pierce: bits,
            dagger_damage_cut: bits,
            armor_blunt: bits,
            armor_sharp: bits,
            armor_heat: bits,
            wall_strength: bits,
            wall_insulation: bits,
            wall_aesthetic: bits,
            floor_aesthetic: bits,
            blocking_warmth: bits,
            blocking_light: bits,
            blocking_noise: bits,
            craft_time_factor: bits,
            craft_quality_factor: bits,
            sharp_damage_factor: bits,
            blunt_damage_factor: bits,
            max_hit_points_factor: bits,
        };
        r.cache_insert(id, sentinel);

        // Second derive — must hit the cache and return sentinel.
        let stats = r.derive(id).unwrap();
        for k in DerivedStatKind::all_variants() {
            assert_eq!(k.read(stats).to_bits(), bits.to_bits());
        }
    }

    #[test]
    fn derive_cache_hit_via_call_counter() {
        let mut r = MaterialRegistry::new();
        let def = def_for("counter_x");
        let id = def.id;
        r.register(def, None).unwrap();

        assert_eq!(r.derive_call_count(), 0);
        let _ = r.derive(id);
        assert_eq!(r.derive_call_count(), 1);
        for _ in 0..10 {
            let _ = r.derive(id);
        }
        assert_eq!(r.derive_call_count(), 1);
    }

    #[test]
    fn unload_mod_scoped_with_cache_flush() {
        let mut r = MaterialRegistry::new();
        let mod_a_ids: Vec<MaterialId> = (0..3)
            .map(|i| {
                let mut d = def_for(&format!("a{i}"));
                d.mod_source = Some("test_mod_a".into());
                let id = d.id;
                r.register(d, Some("test_mod_a")).unwrap();
                let _ = r.derive(id);
                id
            })
            .collect();
        let mod_b_ids: Vec<MaterialId> = (0..2)
            .map(|i| {
                let mut d = def_for(&format!("b{i}"));
                d.mod_source = Some("test_mod_b".into());
                let id = d.id;
                r.register(d, Some("test_mod_b")).unwrap();
                id
            })
            .collect();

        assert_eq!(r.count(), 5);
        let removed = r.unload_mod("test_mod_a");
        assert_eq!(removed, 3);
        assert_eq!(r.count(), 2);

        for id in &mod_b_ids {
            assert!(r.get(*id).is_some());
        }
        for id in &mod_a_ids {
            assert!(r.get(*id).is_none());
            assert!(!r.derive_cache.contains_key(id), "cache leak after unload");
        }
    }

    #[test]
    fn unload_unknown_mod_returns_zero() {
        let mut r = MaterialRegistry::new();
        let removed = r.unload_mod("nonexistent_mod_xyz");
        assert_eq!(removed, 0);
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn bulk_register_100_with_collision_precheck() {
        let names: Vec<String> = (0..100).map(|i| format!("synth_{i}")).collect();
        let hashes: Vec<u32> = names.iter().map(|n| MaterialId::from_str_hash(n).raw()).collect();
        let unique: std::collections::HashSet<u32> = hashes.iter().copied().collect();
        assert_eq!(unique.len(), 100, "djb2 collision in synth_ set — fix the harness data, not the assertion");

        let mut r = MaterialRegistry::new();
        for name in &names {
            r.register(def_for(name), None).unwrap();
        }
        assert_eq!(r.count(), 100);
        assert_eq!(r.all_ids().count(), 100);
        for id in r.all_ids().collect::<Vec<_>>() {
            assert!(r.get(id).is_some());
        }
    }

    #[test]
    fn warm_cache_populates_both_caches() {
        let mut r = MaterialRegistry::new();
        for n in ["a", "b", "c"] {
            r.register(def_for(n), None).unwrap();
        }
        r.warm_cache();
        for id in r.all_ids().collect::<Vec<_>>() {
            assert!(r.derive_cache.contains_key(&id));
        }
        for k in DerivedStatKind::all_variants() {
            assert!(r.explain_cache.contains_key(k));
        }
    }
}
