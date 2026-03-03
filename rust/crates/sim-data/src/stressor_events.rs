use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{DataError, DataResult};
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct StressorEventDef {
    pub category: String,
    #[serde(default)]
    pub is_loss: bool,
    #[serde(default)]
    pub base_instant: f64,
    #[serde(default)]
    pub base_per_tick: f64,
    #[serde(default)]
    pub base_decay_rate: f64,
    #[serde(default)]
    pub personality_modifiers: Value,
    #[serde(default)]
    pub relationship_scaling: Value,
    #[serde(default)]
    pub context_modifiers: Value,
    #[serde(default)]
    pub emotion_inject: Value,
    pub name_key: String,
    pub description_key: String,
}

#[derive(Debug, Clone)]
pub struct StressorEvents(HashMap<String, StressorEventDef>);

impl StressorEvents {
    pub fn get(&self, name: &str) -> Option<&StressorEventDef> {
        self.0.get(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load stressor events from `base_dir/stressor_events.json`.
/// Comment-like keys (`_comment*`) are skipped.
pub fn load_stressor_events(base_dir: &Path) -> DataResult<StressorEvents> {
    let path = base_dir.join("stressor_events.json");
    let raw: HashMap<String, Value> = load_json(&path)?;

    let mut events = HashMap::new();
    for (key, value) in raw {
        if key.starts_with("_comment") {
            continue;
        }
        let event: StressorEventDef =
            serde_json::from_value(value).map_err(|source| DataError::Json {
                path: format!("{}#{}", path.display(), key),
                source,
            })?;
        events.insert(key, event);
    }

    Ok(StressorEvents(events))
}
