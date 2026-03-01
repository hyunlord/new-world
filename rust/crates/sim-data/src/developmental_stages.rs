use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{DataError, DataResult};
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
    validate_developmental_stages(&stages, &path)?;
    Ok(DevelopmentalStages(stages))
}

fn validate_developmental_stages(
    stages: &HashMap<String, DevelopmentalStageDef>,
    path: &Path,
) -> DataResult<()> {
    let p = path.display().to_string();
    for (stage_key, stage) in stages {
        if stage.label_key.trim().is_empty() {
            return Err(DataError::MissingField {
                field: format!("{}.label_key", stage_key),
                path: p.clone(),
            });
        }
        if stage.age_range.len() != 2 {
            return Err(DataError::InvalidField {
                field: format!("{}.age_range", stage_key),
                path: p.clone(),
                reason: "expected [start_age, end_age]".to_string(),
            });
        }
        let start = stage.age_range[0];
        let end = stage.age_range[1];
        if start >= end {
            return Err(DataError::InvalidField {
                field: format!("{}.age_range", stage_key),
                path: p.clone(),
                reason: "expected start_age < end_age".to_string(),
            });
        }
    }
    Ok(())
}
