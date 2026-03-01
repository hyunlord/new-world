use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct MentalBreakDef {
    pub id: String,
    pub severity: String,
    #[serde(default)]
    pub behavior_override: Value,
    #[serde(default)]
    pub duration_base_ticks: u32,
    #[serde(default)]
    pub duration_variance_ticks: u32,
    #[serde(default)]
    pub stress_catharsis_factor: f64,
    #[serde(default)]
    pub energy_cost: f64,
    #[serde(default)]
    pub personality_weights: HashMap<String, f64>,
    #[serde(default)]
    pub trait_modifiers: HashMap<String, f64>,
    #[serde(default)]
    pub scar_chance_base: f64,
    #[serde(default)]
    pub scar_id: String,
    pub name_key: String,
    pub desc_key: String,
}

#[derive(Debug, Clone)]
pub struct MentalBreakCatalog(HashMap<String, MentalBreakDef>);

impl MentalBreakCatalog {
    pub fn get(&self, id: &str) -> Option<&MentalBreakDef> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load mental break definitions from `base_dir/mental_breaks.json`.
pub fn load_mental_breaks(base_dir: &Path) -> DataResult<MentalBreakCatalog> {
    let path = base_dir.join("mental_breaks.json");
    let defs: HashMap<String, MentalBreakDef> = load_json(&path)?;
    Ok(MentalBreakCatalog(defs))
}
