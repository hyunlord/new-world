use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentConfig {
    pub determination_window_days: u32,
    #[serde(default)]
    pub sensitivity_threshold_secure: f64,
    #[serde(default)]
    pub consistency_threshold_secure: f64,
    #[serde(default)]
    pub sensitivity_threshold_anxious: f64,
    #[serde(default)]
    pub consistency_threshold_disorganized: f64,
    #[serde(default)]
    pub abuser_is_caregiver_ace_min: f64,
    #[serde(default)]
    pub adult_effects: HashMap<String, Value>,
    #[serde(default)]
    pub protective_factor: Value,
}

/// Load attachment config from `base_dir/attachment_config.json`.
pub fn load_attachment_config(base_dir: &Path) -> DataResult<AttachmentConfig> {
    let path = base_dir.join("attachment_config.json");
    load_json(&path)
}
