use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct TraitDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub trait_type: String,
    #[serde(default)]
    pub valence: String,
    #[serde(default)]
    pub condition: Value,
    #[serde(default)]
    pub effects: Value,
    #[serde(default)]
    pub opposite_actions: Vec<String>,
    #[serde(default)]
    pub synergies: Vec<String>,
    #[serde(default)]
    pub anti_synergies: Vec<String>,
    #[serde(default)]
    pub name_key: String,
    #[serde(default)]
    pub desc_key: String,
}

#[derive(Debug, Clone)]
pub struct TraitDefinitions(HashMap<String, TraitDefinition>);

impl TraitDefinitions {
    pub fn get(&self, id: &str) -> Option<&TraitDefinition> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load trait definitions from `base_dir/personality/trait_definitions_fixed.json`.
pub fn load_trait_definitions(base_dir: &Path) -> DataResult<TraitDefinitions> {
    let path = base_dir
        .join("personality")
        .join("trait_definitions_fixed.json");
    let defs: Vec<TraitDefinition> = load_json(&path)?;

    let mut by_id = HashMap::new();
    for def in defs {
        by_id.insert(def.id.clone(), def);
    }
    Ok(TraitDefinitions(by_id))
}
