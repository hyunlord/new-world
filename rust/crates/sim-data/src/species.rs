use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{DataError, DataResult};
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct SpeciesDefinition {
    pub species_id: String,
    pub species_name: String,
    pub personality_model: String,
    pub personality_path: String,
    pub emotion_model: String,
    pub emotion_path: String,
    pub mortality_model: String,
    pub mortality_path: String,
    pub needs_model: String,
    pub needs_path: String,
    #[serde(default)]
    pub base_stats: Value,
    #[serde(default)]
    pub available_cultures: Vec<String>,
    pub species_name_key: String,
}

#[derive(Debug, Clone)]
pub struct SpeciesCatalog(HashMap<String, SpeciesDefinition>);

impl SpeciesCatalog {
    pub fn get(&self, species_id: &str) -> Option<&SpeciesDefinition> {
        self.0.get(species_id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load species definitions from `base_dir/species/*/species_definition.json`.
pub fn load_species_catalog(base_dir: &Path) -> DataResult<SpeciesCatalog> {
    let species_root = base_dir.join("species");
    let mut by_id = HashMap::new();

    if !species_root.exists() {
        return Ok(SpeciesCatalog(by_id));
    }

    let entries = std::fs::read_dir(&species_root).map_err(|source| DataError::Io {
        path: species_root.display().to_string(),
        source,
    })?;
    for entry in entries.flatten() {
        let species_dir = entry.path();
        if !species_dir.is_dir() {
            continue;
        }
        let def_path = species_dir.join("species_definition.json");
        if !def_path.exists() {
            continue;
        }
        let def: SpeciesDefinition = load_json(&def_path)?;
        by_id.insert(def.species_id.clone(), def);
    }

    Ok(SpeciesCatalog(by_id))
}
