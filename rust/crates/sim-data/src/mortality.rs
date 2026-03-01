use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{DataError, DataResult};
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct MortalityProfile {
    pub model: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub baseline: Value,
    #[serde(default)]
    pub tech_modifiers: Value,
    #[serde(default)]
    pub care_protection: Value,
    #[serde(default)]
    pub season_modifiers: Value,
}

#[derive(Debug, Clone)]
pub struct MortalityCatalog(HashMap<String, MortalityProfile>);

impl MortalityCatalog {
    pub fn get(&self, species_id: &str) -> Option<&MortalityProfile> {
        self.0.get(species_id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load mortality profiles from `base_dir/species/*/mortality/siler_parameters.json`.
pub fn load_mortality_catalog(base_dir: &Path) -> DataResult<MortalityCatalog> {
    let species_root = base_dir.join("species");
    let mut by_species = HashMap::new();

    if !species_root.exists() {
        return Ok(MortalityCatalog(by_species));
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
        let species_id = species_dir
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_owned();
        if species_id.is_empty() {
            continue;
        }

        let profile_path = species_dir.join("mortality").join("siler_parameters.json");
        if !profile_path.exists() {
            continue;
        }
        let profile: MortalityProfile = load_json(&profile_path)?;
        by_species.insert(species_id, profile);
    }

    Ok(MortalityCatalog(by_species))
}
