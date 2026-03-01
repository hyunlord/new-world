use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct CopingDef {
    pub name_key: String,
    pub desc_key: String,
    pub effect_key: String,
    #[serde(rename = "type")]
    pub coping_type: String,
    pub focus: String,
    #[serde(default)]
    pub base_rate: f64,
    #[serde(default)]
    pub hexaco_weights: HashMap<String, f64>,
    #[serde(default)]
    pub break_weights: HashMap<String, f64>,
    #[serde(default)]
    pub control_appraisal_min: f64,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub synergies: Vec<String>,
    #[serde(default)]
    pub cooldown_ticks: u32,
    #[serde(default)]
    pub prerequisite: String,
}

#[derive(Debug, Clone)]
pub struct CopingDefinitions(HashMap<String, CopingDef>);

impl CopingDefinitions {
    pub fn get(&self, id: &str) -> Option<&CopingDef> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load coping definitions from `base_dir/coping_definitions.json`.
pub fn load_coping_definitions(base_dir: &Path) -> DataResult<CopingDefinitions> {
    let path = base_dir.join("coping_definitions.json");
    let defs: HashMap<String, CopingDef> = load_json(&path)?;
    Ok(CopingDefinitions(defs))
}
