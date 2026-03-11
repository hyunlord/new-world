use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::defs::{
    ActionDef, ActionEffect, FurnitureDef, InfluenceChannelRule, InfluenceEmission, MaterialDef,
    RecipeDef, StructureDef,
};
use crate::loader::{load_ron_directory, DataLoadError};
use crate::tag_index::TagIndex;
use crate::validator::{validate_registry, Severity};
use crate::{MaterialProperties, TagRequirement, TemperamentRules, ValidationError, WorldRuleset};

/// Immutable registry of data-driven simulation definitions.
#[derive(Debug, Clone)]
pub struct DataRegistry {
    /// Base directory that was loaded.
    pub(crate) base_path: PathBuf,
    /// Material definitions keyed by id.
    pub materials: HashMap<String, Arc<MaterialDef>>,
    /// Furniture definitions keyed by id.
    pub furniture: HashMap<String, Arc<FurnitureDef>>,
    /// Recipe definitions keyed by id.
    pub recipes: HashMap<String, Arc<RecipeDef>>,
    /// Structure definitions keyed by id.
    pub structures: HashMap<String, Arc<StructureDef>>,
    /// Action definitions keyed by id.
    pub actions: HashMap<String, Arc<ActionDef>>,
    /// Optional world rules schema bundle.
    pub world_rules: Option<WorldRuleset>,
    /// Optional temperament rules schema bundle.
    pub temperament_rules: Option<TemperamentRules>,
    /// Reverse material tag index.
    pub(crate) tag_index: TagIndex,
}

/// Derived item stats based on material properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerivedStats {
    /// Damage scalar.
    pub damage: f64,
    /// Speed scalar.
    pub speed: f64,
    /// Durability scalar.
    pub durability: f64,
}

impl DataRegistry {
    /// Load a registry from a data directory, collecting all load errors.
    pub fn load_from_directory(base_path: &Path) -> Result<Self, Vec<DataLoadError>> {
        let mut errors = Vec::new();

        let materials = load_defs(
            &base_path.join("materials"),
            "MaterialDef",
            "materials",
            |def: &MaterialDef| def.id.as_str(),
            &mut errors,
        );
        let furniture = load_defs(
            &base_path.join("furniture"),
            "FurnitureDef",
            "furniture",
            |def: &FurnitureDef| def.id.as_str(),
            &mut errors,
        );
        let recipes = load_defs(
            &base_path.join("recipes"),
            "RecipeDef",
            "recipes",
            |def: &RecipeDef| def.id.as_str(),
            &mut errors,
        );
        let structures = load_defs(
            &base_path.join("structures"),
            "StructureDef",
            "structures",
            |def: &StructureDef| def.id.as_str(),
            &mut errors,
        );
        let actions = load_defs(
            &base_path.join("actions"),
            "ActionDef",
            "actions",
            |def: &ActionDef| def.id.as_str(),
            &mut errors,
        );
        let world_rules = load_optional_singleton::<WorldRuleset>(
            &base_path.join("world_rules"),
            "WorldRuleset",
            &mut errors,
        );
        let temperament_rules = load_optional_singleton::<TemperamentRules>(
            &base_path.join("temperament"),
            "TemperamentRules",
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(errors);
        }

        let tag_index = TagIndex::build(&materials);
        let registry = Self {
            base_path: base_path.to_path_buf(),
            materials,
            furniture,
            recipes,
            structures,
            actions,
            world_rules,
            temperament_rules,
            tag_index,
        };

        let validation_errors = validate_registry(&registry);
        let has_fatal_error = validation_errors
            .iter()
            .any(|error| matches!(error.severity, Severity::Error));
        for error in &validation_errors {
            match error.severity {
                Severity::Error => log::error!("{error}"),
                Severity::Warning => log::warn!("{error}"),
            }
        }
        if has_fatal_error {
            return Err(validation_errors
                .into_iter()
                .filter(|error| matches!(error.severity, Severity::Error))
                .map(validation_to_load_error)
                .collect());
        }

        log::info!(
            "Loaded {} materials, {} recipes, {} furniture, {} structures",
            registry.materials.len(),
            registry.recipes.len(),
            registry.furniture.len(),
            registry.structures.len()
        );

        Ok(registry)
    }

    /// Find materials matching a tag requirement.
    pub fn find_materials_by_tag(&self, req: &TagRequirement) -> Vec<Arc<MaterialDef>> {
        self.tag_index.query_with_threshold(req)
    }

    /// Derive item stats from material properties.
    pub fn derive_item_stats(&self, _template: &str, material: &MaterialDef) -> DerivedStats {
        derive_stats_from_properties(&material.properties)
    }

    /// Returns a wall-blocking hint derived from a material's density.
    pub fn material_wall_blocking_hint(&self, material_id: &str) -> Option<f64> {
        self.materials
            .get(material_id)
            .map(|material| (material.properties.density * 0.15).clamp(0.0, 1.0))
    }

    /// Returns influence emissions configured for the given furniture id.
    pub fn furniture_influence_emissions(
        &self,
        furniture_id: &str,
    ) -> Option<&[InfluenceEmission]> {
        self.furniture
            .get(furniture_id)
            .map(|furniture| furniture.influence_emissions.as_slice())
    }

    /// Returns influence emissions configured for a completed structure.
    pub fn structure_completion_influence(
        &self,
        structure_id: &str,
    ) -> Option<&[InfluenceEmission]> {
        self.structures
            .get(structure_id)
            .map(|structure| structure.influence_when_complete.as_slice())
    }

    /// Returns declarative action effects configured for the given action.
    pub fn action_effects(&self, action_id: &str) -> Option<&[ActionEffect]> {
        self.actions
            .get(action_id)
            .map(|action| action.effects.as_slice())
    }

    /// Returns the authoritative world-rules schema bundle when present.
    pub fn world_rules_ref(&self) -> Option<&WorldRuleset> {
        self.world_rules.as_ref()
    }

    /// Returns influence-channel metadata overrides from the world rules when present.
    pub fn influence_channel_rules(&self) -> Option<&[InfluenceChannelRule]> {
        self.world_rules
            .as_ref()
            .map(|ruleset| ruleset.influence_channels.as_slice())
    }

    /// Returns the authoritative temperament-rules schema bundle when present.
    pub fn temperament_rules_ref(&self) -> Option<&TemperamentRules> {
        self.temperament_rules.as_ref()
    }
}

fn load_defs<T, F>(
    dir: &Path,
    def_type: &'static str,
    scope_name: &str,
    id_of: F,
    errors: &mut Vec<DataLoadError>,
) -> HashMap<String, Arc<T>>
where
    T: DeserializeOwned,
    F: Fn(&T) -> &str,
{
    let defs = match load_ron_directory::<T>(dir) {
        Ok(defs) => defs,
        Err(mut load_errors) => {
            errors.append(&mut load_errors);
            return HashMap::new();
        }
    };

    let mut by_id = HashMap::new();
    for def in defs {
        let id = id_of(&def).trim().to_string();
        if id.is_empty() {
            errors.push(DataLoadError {
                file: dir.to_path_buf(),
                line: None,
                message: format!("{def_type} id in {scope_name} must not be empty"),
            });
            continue;
        }
        if by_id.contains_key(&id) {
            errors.push(DataLoadError {
                file: dir.to_path_buf(),
                line: None,
                message: format!("duplicate {def_type} id '{id}'"),
            });
            continue;
        }
        by_id.insert(id, Arc::new(def));
    }

    by_id
}

fn load_optional_singleton<T>(
    dir: &Path,
    def_type: &'static str,
    errors: &mut Vec<DataLoadError>,
) -> Option<T>
where
    T: DeserializeOwned,
{
    let defs = match load_ron_directory::<T>(dir) {
        Ok(defs) => defs,
        Err(mut load_errors) => {
            errors.append(&mut load_errors);
            return None;
        }
    };
    if defs.len() > 1 {
        errors.push(DataLoadError {
            file: dir.to_path_buf(),
            line: None,
            message: format!("expected at most one {def_type}, found {}", defs.len()),
        });
        return None;
    }
    defs.into_iter().next()
}

fn derive_stats_from_properties(properties: &MaterialProperties) -> DerivedStats {
    DerivedStats {
        damage: properties.hardness * 1.2,
        speed: 5.0 / properties.density,
        durability: properties.hardness * properties.density * 10.0,
    }
}

fn validation_to_load_error(error: ValidationError) -> DataLoadError {
    DataLoadError {
        file: PathBuf::from(format!("{}:{}", error.def_type, error.def_id)),
        line: None,
        message: error.message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_item_stats_uses_material_properties() {
        let properties = MaterialProperties {
            hardness: 7.0,
            density: 2.6,
            melting_point: None,
            rarity: 0.3,
            value: 2.5,
        };

        let stats = derive_stats_from_properties(&properties);

        assert!((stats.damage - 8.4).abs() < f64::EPSILON);
        assert!((stats.speed - (5.0 / 2.6)).abs() < f64::EPSILON);
        assert!((stats.durability - 182.0).abs() < f64::EPSILON);
    }

    #[test]
    fn load_from_directory_reads_optional_singletons() {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let registry = DataRegistry::load_from_directory(&data_dir)
            .expect("expected sample crate data to load");

        assert!(registry.world_rules.is_some());
        assert!(registry.temperament_rules.is_some());
    }

    #[test]
    fn foundation_helpers_expose_loaded_schema_hooks() {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let registry = DataRegistry::load_from_directory(&data_dir)
            .expect("expected sample crate data to load");

        let emissions = registry
            .furniture_influence_emissions("fire_pit")
            .expect("expected fire_pit emissions");

        assert!(!emissions.is_empty());
        assert!(registry.material_wall_blocking_hint("flint").is_some());
        assert!(registry.structure_completion_influence("lean_to_structure").is_some());
        assert!(registry.action_effects("forage").is_some());
        assert!(registry.world_rules_ref().is_some());
        assert!(
            registry
                .influence_channel_rules()
                .expect("expected influence channel overrides")
                .len()
                >= 3
        );
        assert!(registry.temperament_rules_ref().is_some());
    }
}
