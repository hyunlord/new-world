use crate::error::DataResult;
use crate::loader::{list_json_files_recursive, load_json};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TechAnyOf {
    Flat(Vec<String>),
    Groups(Vec<Vec<String>>),
}

impl Default for TechAnyOf {
    fn default() -> Self {
        Self::Flat(Vec::new())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechPrereqLogic {
    #[serde(default)]
    pub all_of: Vec<String>,
    #[serde(default)]
    pub any_of: TechAnyOf,
    #[serde(default)]
    pub soft: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TechDef {
    pub id: String,
    pub display_key: String,
    pub description_key: String,
    pub era: String,
    pub tier: u32,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub knowledge_type: String,
    pub prereq_logic: TechPrereqLogic,
}

#[derive(Debug, Clone)]
pub struct TechCatalog(HashMap<String, TechDef>);

impl TechCatalog {
    pub fn get(&self, id: &str) -> Option<&TechDef> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load all tech definitions from `base_dir/tech/**/*.json`.
pub fn load_tech_catalog(base_dir: &Path) -> DataResult<TechCatalog> {
    let tech_dir = base_dir.join("tech");
    let files = list_json_files_recursive(&tech_dir)?;
    let mut map = HashMap::new();
    for path in &files {
        let def: TechDef = load_json(path)?;
        map.insert(def.id.clone(), def);
    }
    Ok(TechCatalog(map))
}
