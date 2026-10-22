//! The colony's materials (M2 goal 5): the V7 material definitions the colony
//! uses, parsed once from the embedded RON files and integerized into
//! [`MaterialStats`] (permille) for run-time use.
//!
//! The 15-property `MaterialProperties` holds `f64` values; the run time sees
//! only the four integer fields below, converted once when the colony world is
//! built (`(value × scale).round()`, no transcendental; DECISIONS D4). The
//! files are embedded with `include_str!`, so a colony world never reads the
//! disk and the table is the same on every machine. The baseline world never
//! builds it.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::material::{parse_ron_str, MaterialDef, MaterialId};

/// The material files the colony uses (granite and limestone boulders, flint
/// and chert outcrops, oak logs).
const FILES: [&str; 5] = [
    include_str!("../../data/material/stone/granite.ron"),
    include_str!("../../data/material/stone/limestone.ron"),
    include_str!("../../data/material/stone/flint.ron"),
    include_str!("../../data/material/stone/chert.ron"),
    include_str!("../../data/material/wood/oak.ron"),
];

/// A material's run-time numbers, permille of each property's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialStats {
    /// Mohs hardness × 100 (Mohs 7 = 700; 1000 = Mohs 10).
    pub hardness: u16,
    /// Density as permille of 25,000 kg/m³ (flint 2,600 kg/m³ = 104).
    pub density: u16,
    /// Workability × 1000 (1000 = easy to shape).
    pub workability: u16,
    /// Flammability × 1000.
    pub flammability: u16,
}

impl MaterialStats {
    /// Integerize one definition (DECISIONS D4).
    pub fn from_def(def: &MaterialDef) -> Self {
        let p = &def.properties;
        let permille = |v: f64, scale: f64| (v * scale).round().clamp(0.0, 1000.0) as u16;
        Self {
            hardness: permille(p.hardness, 100.0),
            density: permille(p.density, 1000.0 / 25_000.0),
            workability: permille(p.workability, 1000.0),
            flammability: permille(p.flammability, 1000.0),
        }
    }

    /// Hardness on a 0–100 scale (Mohs × 10), the unit of the blade quality term.
    pub const fn hardness_percent(&self) -> i32 {
        self.hardness as i32 / 10
    }
}

/// The colony's material table: id → (stable name, stats), in id order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyMaterials {
    /// Every material by id.
    pub by_id: BTreeMap<MaterialId, (String, MaterialStats)>,
}

impl ColonyMaterials {
    /// Parse the embedded files. A file that fails to parse is skipped (a unit
    /// test asserts every file parses), so the run time never panics.
    pub fn load() -> Self {
        let mut by_id = BTreeMap::new();
        for text in FILES {
            for def in parse_ron_str(text).unwrap_or_default() {
                let name = def.name.to_lowercase();
                by_id.insert(def.id, (name, MaterialStats::from_def(&def)));
            }
        }
        Self { by_id }
    }

    /// Stats of `id`, if the colony knows it.
    pub fn stats(&self, id: MaterialId) -> Option<MaterialStats> {
        self.by_id.get(&id).map(|(_, stats)| *stats)
    }

    /// Lowercase name of `id` (`"flint"`), or `""`.
    pub fn name(&self, id: MaterialId) -> &str {
        self.by_id.get(&id).map_or("", |(name, _)| name.as_str())
    }
}

/// `flint`.
pub const FLINT: MaterialId = MaterialId::from_str_hash("flint");
/// `chert`.
pub const CHERT: MaterialId = MaterialId::from_str_hash("chert");
/// `granite` (hill boulders).
pub const GRANITE: MaterialId = MaterialId::from_str_hash("granite");
/// `limestone` (grass boulders).
pub const LIMESTONE: MaterialId = MaterialId::from_str_hash("limestone");
/// `oak` (every log until trees have species).
pub const OAK: MaterialId = MaterialId::from_str_hash("oak");
/// `clay` (river-bank clay, M4; no stats file: no craft reads its hardness).
pub const CLAY: MaterialId = MaterialId::from_str_hash("clay");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_file_parses_and_integerizes() {
        for text in FILES {
            assert_eq!(parse_ron_str(text).map(|d| d.len()).unwrap_or(0), 1);
        }
        let table = ColonyMaterials::load();
        assert_eq!(table.by_id.len(), 5);
        // flint.ron: hardness 7.0, density 2600, workability 0.5, flammability 0.
        let flint = table.stats(FLINT).unwrap_or(MaterialStats {
            hardness: 0,
            density: 0,
            workability: 0,
            flammability: 0,
        });
        assert_eq!(
            (
                flint.hardness,
                flint.density,
                flint.workability,
                flint.flammability
            ),
            (700, 104, 500, 0)
        );
        assert_eq!(flint.hardness_percent(), 70);
        assert_eq!(table.name(FLINT), "flint");
        // limestone.ron hardness 4.0; oak.ron flammability 0.8.
        assert_eq!(table.stats(LIMESTONE).map(|s| s.hardness), Some(400));
        assert_eq!(table.stats(OAK).map(|s| s.flammability), Some(800));
        for id in [GRANITE, CHERT] {
            assert!(table.stats(id).is_some());
        }
    }
}
