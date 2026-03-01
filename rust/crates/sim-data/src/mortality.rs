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
        validate_mortality_profile(&profile, &profile_path)?;
        by_species.insert(species_id, profile);
    }

    Ok(MortalityCatalog(by_species))
}

fn validate_mortality_profile(profile: &MortalityProfile, path: &Path) -> DataResult<()> {
    if profile.model.trim().is_empty() {
        return Err(DataError::MissingField {
            field: "model".to_string(),
            path: path.display().to_string(),
        });
    }

    let p = path.display().to_string();
    let a1 = required_number(&profile.baseline, "a1", &p, "baseline")?;
    let b1 = required_number(&profile.baseline, "b1", &p, "baseline")?;
    let a2 = required_number(&profile.baseline, "a2", &p, "baseline")?;
    let a3 = required_number(&profile.baseline, "a3", &p, "baseline")?;
    let b3 = required_number(&profile.baseline, "b3", &p, "baseline")?;
    if a1 < 0.0 || a2 < 0.0 || a3 < 0.0 || b1 <= 0.0 || b3 <= 0.0 {
        return Err(DataError::InvalidField {
            field: "baseline".to_string(),
            path: p.clone(),
            reason: "expected a1/a2/a3 >= 0 and b1/b3 > 0".to_string(),
        });
    }

    let k1 = required_number(&profile.tech_modifiers, "k1", &p, "tech_modifiers")?;
    let k2 = required_number(&profile.tech_modifiers, "k2", &p, "tech_modifiers")?;
    let k3 = required_number(&profile.tech_modifiers, "k3", &p, "tech_modifiers")?;
    if k1 < 0.0 || k2 < 0.0 || k3 < 0.0 {
        return Err(DataError::InvalidField {
            field: "tech_modifiers".to_string(),
            path: p.clone(),
            reason: "expected k1/k2/k3 >= 0".to_string(),
        });
    }

    let hunger_min = required_number(
        &profile.care_protection,
        "hunger_min",
        &p,
        "care_protection",
    )?;
    let protection_factor = required_number(
        &profile.care_protection,
        "protection_factor",
        &p,
        "care_protection",
    )?;
    if !(0.0..=1.0).contains(&hunger_min) || !(0.0..=1.0).contains(&protection_factor) {
        return Err(DataError::InvalidField {
            field: "care_protection".to_string(),
            path: p,
            reason: "expected hunger_min/protection_factor in [0, 1]".to_string(),
        });
    }

    Ok(())
}

fn required_number(value: &Value, key: &str, path: &str, parent_field: &str) -> DataResult<f64> {
    let object = value.as_object().ok_or_else(|| DataError::InvalidField {
        field: parent_field.to_string(),
        path: path.to_string(),
        reason: "expected object".to_string(),
    })?;
    let number =
        object
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| DataError::MissingField {
                field: format!("{}.{}", parent_field, key),
                path: path.to_string(),
            })?;
    Ok(number)
}
