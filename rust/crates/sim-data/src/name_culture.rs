//! Naming culture definitions loaded from `data/naming_cultures/*.json`.
//!
//! Each culture defines name pools, syllabic generation parameters,
//! and patronymic rules for agent name generation.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::loader::list_json_files;

// ── Structs ───────────────────────────────────────────────────────────────────

/// A complete naming culture definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameCulture {
    /// Unique identifier matching the file name (e.g. "proto_nature").
    pub culture_id: String,
    /// Human-readable name for UI display.
    pub display_name: String,
    /// Pre-built given name pools (may be empty for syllabic-only cultures).
    pub given_names: GivenNames,
    /// If true, names are generated syllabically (Markov-style combination).
    pub allow_markov_generation: bool,
    /// Syllable pools for syllabic name generation (required when `allow_markov_generation`).
    #[serde(default)]
    pub syllable_pools: Option<SyllablePools>,
    /// Number of syllables per generated name.
    #[serde(default)]
    pub syllable_count: Option<SyllableCount>,
    /// How patronymics are applied: "none", "prefix", "suffix".
    #[serde(default = "default_patronymic_rule")]
    pub patronymic_rule: String,
    /// Suffix/prefix strings for patronymic application.
    #[serde(default)]
    pub patronymic_config: Option<PatronymicConfig>,
    // Ignored fields from JSON — kept to avoid serde errors on unknown fields.
    #[serde(default, skip_serializing)]
    _description: Option<serde_json::Value>,
    #[serde(default, rename = "name_structure", skip_serializing)]
    _name_structure: Option<serde_json::Value>,
    #[serde(default, rename = "surname_rule", skip_serializing)]
    _surname_rule: Option<serde_json::Value>,
    #[serde(default, rename = "markov_config", skip_serializing)]
    _markov_config: Option<serde_json::Value>,
    #[serde(default, rename = "epithets", skip_serializing)]
    _epithets: Option<serde_json::Value>,
    #[serde(default, rename = "epithet_unlock_age", skip_serializing)]
    _epithet_unlock_age: Option<serde_json::Value>,
}

fn default_patronymic_rule() -> String {
    "none".to_string()
}

/// Male/female/neutral given name pools.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GivenNames {
    #[serde(default)]
    pub male: Vec<String>,
    #[serde(default)]
    pub female: Vec<String>,
    #[serde(default)]
    pub neutral: Vec<String>,
}

/// Syllable pools for Markov-style name generation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyllablePools {
    #[serde(default)]
    pub onset_male: Vec<String>,
    #[serde(default)]
    pub onset_female: Vec<String>,
    #[serde(default)]
    pub nucleus: Vec<String>,
    #[serde(default)]
    pub coda: Vec<String>,
    #[serde(default)]
    pub coda_final: Vec<String>,
}

/// Syllable count range for generated names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyllableCount {
    pub min: u32,
    pub max: u32,
}

impl Default for SyllableCount {
    fn default() -> Self {
        Self { min: 2, max: 3 }
    }
}

/// Patronymic suffix/prefix configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatronymicConfig {
    #[serde(default)]
    pub male_prefix: Option<String>,
    #[serde(default)]
    pub male_suffix: Option<String>,
    #[serde(default)]
    pub female_prefix: Option<String>,
    #[serde(default)]
    pub female_suffix: Option<String>,
}

// ── Loader ────────────────────────────────────────────────────────────────────

/// Load all naming cultures from `{base_path}/naming_cultures/*.json`.
///
/// Keys in the returned map are `culture_id` from the JSON.
/// Malformed files are skipped with a warning — never panics.
pub fn load_name_cultures(base_path: &Path) -> HashMap<String, NameCulture> {
    let dir = base_path.join("naming_cultures");
    let mut map = HashMap::new();

    let files = match list_json_files(&dir) {
        Ok(f) => f,
        Err(e) => {
            log::warn!("[sim-data] Could not list naming_cultures dir: {:?}", e);
            return map;
        }
    };

    for path in files {
        let content = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[sim-data] Could not read {:?}: {:?}", path, e);
                continue;
            }
        };
        let culture: NameCulture = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[sim-data] Malformed naming culture {:?}: {:?}", path, e);
                continue;
            }
        };
        log::debug!("[sim-data] Loaded naming culture '{}'", culture.culture_id);
        map.insert(culture.culture_id.clone(), culture);
    }

    log::info!("[sim-data] Loaded {} naming cultures", map.len());
    map
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate dir has parent")
            .parent()
            .expect("crates/ dir has parent")
            .parent()
            .expect("rust/ dir has parent")
            .join("data")
    }

    #[test]
    fn load_three_cultures() {
        let dir = data_dir();
        if !dir.exists() {
            eprintln!("Skipping: data dir not found at {:?}", dir);
            return;
        }
        let cultures = load_name_cultures(&dir);
        assert!(cultures.len() >= 3, "Expected at least 3 cultures, got {}", cultures.len());
        assert!(cultures.contains_key("proto_nature"), "Missing proto_nature");
        assert!(cultures.contains_key("proto_syllabic"), "Missing proto_syllabic");
        assert!(cultures.contains_key("tribal_totemic"), "Missing tribal_totemic");
    }

    #[test]
    fn proto_nature_has_name_pools() {
        let dir = data_dir();
        if !dir.exists() { return; }
        let cultures = load_name_cultures(&dir);
        let culture = &cultures["proto_nature"];
        assert!(!culture.given_names.male.is_empty(), "proto_nature male names should not be empty");
        assert!(!culture.given_names.female.is_empty(), "proto_nature female names should not be empty");
        assert!(!culture.allow_markov_generation, "proto_nature should not use markov generation");
    }

    #[test]
    fn proto_syllabic_has_syllable_pools() {
        let dir = data_dir();
        if !dir.exists() { return; }
        let cultures = load_name_cultures(&dir);
        let culture = &cultures["proto_syllabic"];
        assert!(culture.allow_markov_generation, "proto_syllabic should use markov generation");
        let pools = culture.syllable_pools.as_ref().expect("proto_syllabic should have syllable pools");
        assert!(!pools.nucleus.is_empty(), "Nucleus pool should not be empty");
    }

    #[test]
    fn tribal_totemic_has_patronymic() {
        let dir = data_dir();
        if !dir.exists() { return; }
        let cultures = load_name_cultures(&dir);
        let culture = &cultures["tribal_totemic"];
        assert_eq!(culture.patronymic_rule, "prefix");
        assert!(culture.patronymic_config.is_some(), "tribal_totemic should have patronymic_config");
    }
}
