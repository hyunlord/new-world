//! sim-data: JSON data loaders for simulation content.
pub mod error;
pub mod loader;
pub mod emotion_presets;
pub mod tech;
pub mod value_events;

pub use error::{DataError, DataResult};
pub use emotion_presets::{EmotionPreset, EmotionPresets, load_emotion_presets};
pub use tech::{TechDef, TechCatalog, load_tech_catalog};
pub use value_events::{ValueEvent, ValueEvents, load_value_events};

/// Load all data from a base data directory.
/// Returns a tuple of (emotion_presets, tech_catalog, value_events).
/// If any individual load fails, returns that error.
pub fn load_all(base_dir: &std::path::Path) -> DataResult<(EmotionPresets, TechCatalog, ValueEvents)> {
    let emotions = load_emotion_presets(base_dir)?;
    let tech = load_tech_catalog(base_dir)?;
    let values = load_value_events(base_dir)?;
    Ok((emotions, tech, values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_all_from_project_data() {
        let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()  // crates/
            .parent().unwrap()  // rust/
            .parent().unwrap()  // lead/ (project root)
            .join("data");

        if !data_dir.exists() {
            eprintln!("Skipping: data dir not found at {:?}", data_dir);
            return;
        }

        let (emotions, tech, values) = load_all(&data_dir).expect("load_all failed");
        assert!(emotions.len() > 0, "no emotion presets loaded");
        assert!(tech.len() > 0, "no tech definitions loaded");
        assert!(values.len() > 0, "no value events loaded");
    }
}
