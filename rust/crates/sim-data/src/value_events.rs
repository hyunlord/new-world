use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;
use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct ValueEvent {
    pub affected_values: HashMap<String, f64>,
    pub intensity: f64,
    pub description_key: String,
}

#[derive(Debug, Clone)]
pub struct ValueEvents(HashMap<String, ValueEvent>);

impl ValueEvents {
    pub fn get(&self, name: &str) -> Option<&ValueEvent> {
        self.0.get(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load value events from `base_dir/values/value_events.json`.
pub fn load_value_events(base_dir: &Path) -> DataResult<ValueEvents> {
    let path = base_dir.join("values").join("value_events.json");
    let raw: HashMap<String, ValueEvent> = load_json(&path)?;
    Ok(ValueEvents(raw))
}
