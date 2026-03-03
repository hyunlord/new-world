use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{DataError, DataResult};
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
    let config: AttachmentConfig = load_json(&path)?;
    validate_attachment_config(&config, &path)?;
    Ok(config)
}

fn validate_attachment_config(config: &AttachmentConfig, path: &Path) -> DataResult<()> {
    let p = path.display().to_string();

    if config.determination_window_days == 0 {
        return Err(DataError::InvalidField {
            field: "determination_window_days".to_string(),
            path: p.clone(),
            reason: "must be > 0".to_string(),
        });
    }

    validate_unit_range(
        config.sensitivity_threshold_secure,
        "sensitivity_threshold_secure",
        &p,
    )?;
    validate_unit_range(
        config.consistency_threshold_secure,
        "consistency_threshold_secure",
        &p,
    )?;
    validate_unit_range(
        config.sensitivity_threshold_anxious,
        "sensitivity_threshold_anxious",
        &p,
    )?;
    validate_unit_range(
        config.consistency_threshold_disorganized,
        "consistency_threshold_disorganized",
        &p,
    )?;
    if config.abuser_is_caregiver_ace_min < 0.0 {
        return Err(DataError::InvalidField {
            field: "abuser_is_caregiver_ace_min".to_string(),
            path: p,
            reason: "must be >= 0".to_string(),
        });
    }

    Ok(())
}

fn validate_unit_range(value: f64, field: &str, path: &str) -> DataResult<()> {
    if !(0.0..=1.0).contains(&value) {
        return Err(DataError::InvalidField {
            field: field.to_string(),
            path: path.to_string(),
            reason: "must be in [0, 1]".to_string(),
        });
    }
    Ok(())
}
