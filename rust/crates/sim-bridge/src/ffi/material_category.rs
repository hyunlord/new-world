//! M-ART2 bridge helper: the registry category name of a `MaterialId` (MART2_DECISIONS D6), for `get_space`'s
//! `wall_categories` / `floor_categories` — the art loader's fallback when its data does not list a material
//! (`art/materials/ADDING_A_STYLE.md`, "M-ART2 loading contract").
//!
//! Lookup order: the world's live `SimResources::material_registry` (mods and future worlds that register
//! materials), then the embedded catalogue of every sim-core material file (`build.rs` embeds
//! `sim-core/data/material/**/*.ron`; parsed once with a `OnceLock`). The game's worlds build an empty registry
//! today, so the catalogue answers for granite, limestone, oak, …; a `MaterialId` neither knows (the packed-earth
//! floor's `packed_earth` has no material file) has no category (`""`). Names are the `MaterialCategory`
//! variants (`Stone`, `Wood`, `Animal`, `Mineral`, `Plant`); every `Mod(u8)` collapses to `Mod`. Pure reads.
use sim_core::material::{parse_ron_str, MaterialCategory, MaterialId};
use sim_engine::SimResources;
use std::collections::BTreeMap;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/material_files.rs"));

/// The category name the bridge sends for `category` (`Mod(_)` → `"Mod"`).
pub const fn category_name(category: MaterialCategory) -> &'static str {
    match category {
        MaterialCategory::Stone => "Stone",
        MaterialCategory::Wood => "Wood",
        MaterialCategory::Animal => "Animal",
        MaterialCategory::Mineral => "Mineral",
        MaterialCategory::Plant => "Plant",
        MaterialCategory::Mod(_) => "Mod",
    }
}

/// The embedded catalogue: every material id of the sim-core files with its category, in id order. A file that
/// fails to parse is skipped (a unit test asserts every file parses), so a read never panics.
pub fn catalogue() -> &'static BTreeMap<MaterialId, MaterialCategory> {
    static CATALOGUE: OnceLock<BTreeMap<MaterialId, MaterialCategory>> = OnceLock::new();
    CATALOGUE.get_or_init(|| {
        MATERIAL_FILES
            .iter()
            .flat_map(|(_, text)| parse_ron_str(text).unwrap_or_default())
            .map(|def| (def.id, def.category))
            .collect()
    })
}

/// The number of embedded material files (tests compare it with the directory).
pub fn embedded_file_count() -> usize {
    MATERIAL_FILES.len()
}

/// The category of `id`: the live registry first, then the catalogue; `None` when neither knows it.
pub fn category_of(res: &SimResources, id: MaterialId) -> Option<MaterialCategory> {
    res.material_registry
        .get(id)
        .map(|def| def.category)
        .or_else(|| catalogue().get(&id).copied())
}

/// [`category_of`] as the bridge's name, `""` for none (also for a tile without a material).
pub fn category_label(res: &SimResources, id: Option<MaterialId>) -> &'static str {
    id.and_then(|id| category_of(res, id))
        .map_or("", category_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_material_file_parses_with_one_category() {
        assert!(embedded_file_count() >= 100, "{}", embedded_file_count());
        for (path, text) in MATERIAL_FILES {
            let defs = parse_ron_str(text).unwrap_or_default();
            assert!(!defs.is_empty(), "{path} parses");
        }
        let n: usize = MATERIAL_FILES
            .iter()
            .map(|(_, t)| parse_ron_str(t).map_or(0, |d| d.len()))
            .sum();
        assert_eq!(catalogue().len(), n, "ids are unique");
    }

    #[test]
    fn names_follow_the_category_and_mod_collapses() {
        assert_eq!(category_name(MaterialCategory::Mod(7)), "Mod");
        assert_eq!(category_name(MaterialCategory::Mod(0)), "Mod");
        assert_eq!(
            catalogue().get(&MaterialId::from_str_hash("granite")),
            Some(&MaterialCategory::Stone)
        );
        assert_eq!(
            catalogue().get(&MaterialId::from_str_hash("oak")),
            Some(&MaterialCategory::Wood)
        );
        assert_eq!(
            catalogue().get(&MaterialId::from_str_hash("packed_earth")),
            None
        );
    }
}
