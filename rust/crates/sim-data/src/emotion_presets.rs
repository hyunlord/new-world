use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;
use crate::error::DataResult;
use crate::loader::load_json;

#[derive(Debug, Clone, Deserialize)]
pub struct EmotionPreset {
    pub description: String,
    pub category: String,
    pub intensity: u32,
    pub goal_congruence: f64,
    pub novelty: f64,
    pub controllability: f64,
    pub agency: f64,
    pub norm_violation: f64,
    pub pathogen: f64,
    pub social_bond: f64,
    pub future_relevance: f64,
    #[serde(default)]
    pub is_trauma: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct EmotionPresetsFile {
    #[serde(rename = "_comment", default)]
    _comment: Option<String>,
    events: HashMap<String, EmotionPreset>,
}

#[derive(Debug, Clone)]
pub struct EmotionPresets(HashMap<String, EmotionPreset>);

impl EmotionPresets {
    pub fn get(&self, name: &str) -> Option<&EmotionPreset> {
        self.0.get(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Load emotion presets from `base_dir/emotions/event_presets.json`.
pub fn load_emotion_presets(base_dir: &Path) -> DataResult<EmotionPresets> {
    let path = base_dir.join("emotions").join("event_presets.json");
    let file: EmotionPresetsFile = load_json(&path)?;
    Ok(EmotionPresets(file.events))
}
