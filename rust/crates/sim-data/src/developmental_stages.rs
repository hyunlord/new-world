use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct DevelopmentalStageDef {
    #[serde(default)]
    pub age_range: Vec<u32>,
    #[serde(default)]
    pub label_key: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct DevelopmentalStages(HashMap<String, DevelopmentalStageDef>);

impl DevelopmentalStages {
    pub fn get(&self, stage: &str) -> Option<&DevelopmentalStageDef> {
        self.0.get(stage)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load developmental stage definitions from `base_dir/developmental_stages.json`.
pub fn load_developmental_stages(base_dir: &Path) -> DataResult<DevelopmentalStages> {
    let path = base_dir.join("developmental_stages.json");
    let stages: HashMap<String, DevelopmentalStageDef> = load_json(&path)?;
    Ok(DevelopmentalStages(stages))
}
