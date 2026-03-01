//! sim-data: JSON data loaders for simulation content.
pub mod attachment_config;
pub mod coping;
pub mod developmental_stages;
pub mod emotion_presets;
pub mod error;
pub mod loader;
pub mod mental_breaks;
pub mod mortality;
pub mod species;
pub mod stressor_events;
pub mod tech;
pub mod trait_defs;
pub mod value_events;

pub use attachment_config::{load_attachment_config, AttachmentConfig};
pub use coping::{load_coping_definitions, CopingDef, CopingDefinitions};
pub use developmental_stages::{load_developmental_stages, DevelopmentalStageDef, DevelopmentalStages};
pub use emotion_presets::{load_emotion_presets, EmotionPreset, EmotionPresets};
pub use error::{DataError, DataResult};
pub use mental_breaks::{load_mental_breaks, MentalBreakCatalog, MentalBreakDef};
pub use mortality::{load_mortality_catalog, MortalityCatalog, MortalityProfile};
pub use species::{load_species_catalog, SpeciesCatalog, SpeciesDefinition};
pub use stressor_events::{load_stressor_events, StressorEventDef, StressorEvents};
pub use tech::{load_tech_catalog, TechCatalog, TechDef};
pub use trait_defs::{load_trait_definitions, TraitDefinition, TraitDefinitions};
pub use value_events::{load_value_events, ValueEvent, ValueEvents};

/// Aggregated simulation data snapshot loaded from JSON files.
#[derive(Debug, Clone)]
pub struct DataBundle {
    pub emotions: EmotionPresets,
    pub tech: TechCatalog,
    pub values: ValueEvents,
    pub stressors: StressorEvents,
    pub coping: CopingDefinitions,
    pub mental_breaks: MentalBreakCatalog,
    pub traits: TraitDefinitions,
    pub species: SpeciesCatalog,
    pub mortality: MortalityCatalog,
    pub developmental_stages: DevelopmentalStages,
    pub attachment: AttachmentConfig,
}

/// Load all currently-supported data from a base data directory.
/// If any individual loader fails, returns that error.
pub fn load_all(base_dir: &std::path::Path) -> DataResult<DataBundle> {
    let emotions = load_emotion_presets(base_dir)?;
    let tech = load_tech_catalog(base_dir)?;
    let values = load_value_events(base_dir)?;
    let stressors = load_stressor_events(base_dir)?;
    let coping = load_coping_definitions(base_dir)?;
    let mental_breaks = load_mental_breaks(base_dir)?;
    let traits = load_trait_definitions(base_dir)?;
    let species = load_species_catalog(base_dir)?;
    let mortality = load_mortality_catalog(base_dir)?;
    let developmental_stages = load_developmental_stages(base_dir)?;
    let attachment = load_attachment_config(base_dir)?;
    Ok(DataBundle {
        emotions,
        tech,
        values,
        stressors,
        coping,
        mental_breaks,
        traits,
        species,
        mortality,
        developmental_stages,
        attachment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_all_from_project_data() {
        let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap() // crates/
            .parent()
            .unwrap() // rust/
            .parent()
            .unwrap() // lead/ (project root)
            .join("data");

        if !data_dir.exists() {
            eprintln!("Skipping: data dir not found at {:?}", data_dir);
            return;
        }

        let data = load_all(&data_dir).expect("load_all failed");
        assert!(data.emotions.len() > 0, "no emotion presets loaded");
        assert!(data.tech.len() > 0, "no tech definitions loaded");
        assert!(data.values.len() > 0, "no value events loaded");
        assert!(data.stressors.len() > 0, "no stressor events loaded");
        assert!(data.coping.len() > 0, "no coping definitions loaded");
        assert!(
            data.mental_breaks.len() > 0,
            "no mental break definitions loaded"
        );
        assert!(data.traits.len() > 0, "no trait definitions loaded");
        assert!(data.species.len() > 0, "no species definitions loaded");
        assert!(data.mortality.len() > 0, "no mortality profiles loaded");
        assert!(
            data.developmental_stages.len() > 0,
            "no developmental stage definitions loaded"
        );
        assert!(
            data.attachment.determination_window_days > 0,
            "attachment config not loaded"
        );
    }
}
