use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::defs::{
    ActionDef, ActionEffect, AgentConstants, FurnitureDef, GlobalConstants, InfluenceChannelRule,
    InfluenceEmission, MaterialDef, RecipeDef, StructureDef,
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
    /// Raw world rulesets as loaded from the filesystem, sorted by priority
    /// ascending (lowest priority first, highest last). Diagnostic view —
    /// authoritative merged result lives in [`DataRegistry::world_rules`].
    pub world_rules_raw: Vec<WorldRuleset>,
    /// Cached merged world ruleset, computed once at load time from
    /// `world_rules_raw`. `None` only when no rulesets were loaded.
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
        let world_rules_raw = load_all_world_rules(
            &base_path.join("world_rules"),
            &mut errors,
        );
        let world_rules = merge_world_rules(&world_rules_raw);
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
            world_rules_raw,
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

    /// Returns a wall-blocking hint for an influence channel propagating
    /// through a wall of this material.
    ///
    /// Coefficients follow the Building System architecture note in
    /// `CLAUDE.md` (§Architecture Decisions — Building System):
    /// - Stone walls block ~90% of incoming influence.
    /// - Wood walls block ~50%.
    ///
    /// Other categories fall back to a density-derived heuristic so future
    /// material categories get a sensible default without code changes.
    /// The returned value is always clamped to `[0.0, 1.0]`.
    pub fn material_wall_blocking_hint(&self, material_id: &str) -> Option<f64> {
        use crate::MaterialCategory;
        self.materials.get(material_id).map(|material| {
            let coefficient = match material.category {
                MaterialCategory::Stone | MaterialCategory::Metal | MaterialCategory::Mineral => 0.9,
                MaterialCategory::Wood => 0.5,
                _ => (material.properties.density * 0.15).clamp(0.0, 1.0),
            };
            coefficient.clamp(0.0, 1.0)
        })
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

    /// Returns the immutable structure definition for the given runtime building id.
    pub fn structure_def(&self, structure_id: &str) -> Option<&StructureDef> {
        self.structures.get(structure_id).map(Arc::as_ref)
    }

    /// Returns the configured construction time for a structure.
    pub fn structure_build_ticks(&self, structure_id: &str) -> Option<u64> {
        self.structure_def(structure_id).map(|structure| structure.build_ticks)
    }

    /// Returns the configured resource cost for a structure and resource tag.
    pub fn structure_resource_cost(&self, structure_id: &str, resource_tag: &str) -> Option<f64> {
        self.structure_def(structure_id)
            .and_then(|structure| structure.resource_costs.get(resource_tag))
            .copied()
    }

    /// Returns wall hit points derived from authoritative material properties.
    pub fn material_wall_hit_points(&self, material_id: &str) -> Option<f64> {
        self.materials.get(material_id).map(|material| {
            material.properties.hardness * material.properties.density * 10.0
        })
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

/// Loads every `WorldRuleset` RON file in `dir` and its immediate subdirectories
/// (1-level recursion for `scenarios/`, `oracle/`, etc.), then sorts the result
/// by `priority` ascending.
///
/// Lowest-priority ruleset appears first in the returned vec; highest-priority
/// appears last — the expected input order for [`merge_world_rules`].
fn load_all_world_rules(dir: &Path, errors: &mut Vec<DataLoadError>) -> Vec<WorldRuleset> {
    let mut all_rules: Vec<WorldRuleset> = Vec::new();

    match load_ron_directory::<WorldRuleset>(dir) {
        Ok(defs) => all_rules.extend(defs),
        Err(mut load_errors) => errors.append(&mut load_errors),
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut subdirs: Vec<PathBuf> = entries
            .flatten()
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                if file_type.is_dir() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect();
        subdirs.sort();
        for subdir in subdirs {
            match load_ron_directory::<WorldRuleset>(&subdir) {
                Ok(defs) => all_rules.extend(defs),
                Err(mut load_errors) => errors.append(&mut load_errors),
            }
        }
    }

    all_rules.sort_by_key(|ruleset| ruleset.priority);
    all_rules
}

/// Merges a sorted slice of [`WorldRuleset`] by priority into a single
/// authoritative ruleset.
///
/// The slice must be pre-sorted in ascending priority order (lowest first,
/// highest last). Merge semantics per field type:
/// - `Option<T>` fields (GlobalConstants / AgentConstants subfields): higher
///   priority `Some(x)` overrides lower priority values; `None` is transparent
///   (does not clobber a prior `Some`).
/// - Vec fields with an identity key (resource_modifiers by `target`,
///   special_resources by `name`, influence_channels by `channel`): dedup by
///   key with last-writer-wins (highest priority wins).
/// - `special_zones`: pure append (no dedup — distinct rulesets may contribute
///   independent zones).
/// - Scalar fields (`name`, `priority`): highest-priority ruleset wins.
///
/// Returns `None` when the input slice is empty.
pub fn merge_world_rules(rulesets: &[WorldRuleset]) -> Option<WorldRuleset> {
    if rulesets.is_empty() {
        return None;
    }

    let mut merged = WorldRuleset {
        name: String::new(),
        priority: 0,
        resource_modifiers: Vec::new(),
        special_zones: Vec::new(),
        special_resources: Vec::new(),
        agent_constants: None,
        influence_channels: Vec::new(),
        global_constants: None,
    };

    for ruleset in rulesets {
        // Scalar fields: last writer wins (highest priority at end of slice).
        merged.name = ruleset.name.clone();
        merged.priority = ruleset.priority;

        // Resource modifiers: dedup by target, then append. Last writer wins.
        for modifier in &ruleset.resource_modifiers {
            merged
                .resource_modifiers
                .retain(|existing| existing.target != modifier.target);
            merged.resource_modifiers.push(modifier.clone());
        }

        // Special zones: pure append (no dedup).
        merged.special_zones.extend(ruleset.special_zones.clone());

        // Special resources: dedup by name, then append.
        for resource in &ruleset.special_resources {
            merged
                .special_resources
                .retain(|existing| existing.name != resource.name);
            merged.special_resources.push(resource.clone());
        }

        // Influence channels: dedup by channel name, then append.
        for channel_rule in &ruleset.influence_channels {
            merged
                .influence_channels
                .retain(|existing| existing.channel != channel_rule.channel);
            merged.influence_channels.push(channel_rule.clone());
        }

        // GlobalConstants: field-wise merge (Some overlay wins, None transparent).
        merged.global_constants = merge_global_constants(
            merged.global_constants.as_ref(),
            ruleset.global_constants.as_ref(),
        );

        // AgentConstants: field-wise merge (Some overlay wins, None transparent).
        merged.agent_constants = merge_agent_constants(
            merged.agent_constants.as_ref(),
            ruleset.agent_constants.as_ref(),
        );
    }

    Some(merged)
}

fn merge_global_constants(
    base: Option<&GlobalConstants>,
    overlay: Option<&GlobalConstants>,
) -> Option<GlobalConstants> {
    let overlay = match overlay {
        Some(o) => o,
        None => return base.cloned(),
    };
    let base = base.cloned().unwrap_or(GlobalConstants {
        season_mode: None,
        hunger_decay_mul: None,
        warmth_decay_mul: None,
        food_regen_mul: None,
        wood_regen_mul: None,
        farming_enabled: None,
        temperature_bias: None,
        disaster_frequency_mul: None,
    });
    Some(GlobalConstants {
        season_mode: overlay.season_mode.clone().or(base.season_mode),
        hunger_decay_mul: overlay.hunger_decay_mul.or(base.hunger_decay_mul),
        warmth_decay_mul: overlay.warmth_decay_mul.or(base.warmth_decay_mul),
        food_regen_mul: overlay.food_regen_mul.or(base.food_regen_mul),
        wood_regen_mul: overlay.wood_regen_mul.or(base.wood_regen_mul),
        farming_enabled: overlay.farming_enabled.or(base.farming_enabled),
        temperature_bias: overlay.temperature_bias.or(base.temperature_bias),
        disaster_frequency_mul: overlay
            .disaster_frequency_mul
            .or(base.disaster_frequency_mul),
    })
}

fn merge_agent_constants(
    base: Option<&AgentConstants>,
    overlay: Option<&AgentConstants>,
) -> Option<AgentConstants> {
    let overlay = match overlay {
        Some(o) => o,
        None => return base.cloned(),
    };
    let base = base.cloned().unwrap_or(AgentConstants {
        mortality_mul: None,
        skill_xp_mul: None,
        body_potential_mul: None,
        fertility_mul: None,
        lifespan_mul: None,
        move_speed_mul: None,
    });
    Some(AgentConstants {
        mortality_mul: overlay.mortality_mul.or(base.mortality_mul),
        skill_xp_mul: overlay.skill_xp_mul.or(base.skill_xp_mul),
        body_potential_mul: overlay.body_potential_mul.or(base.body_potential_mul),
        fertility_mul: overlay.fertility_mul.or(base.fertility_mul),
        lifespan_mul: overlay.lifespan_mul.or(base.lifespan_mul),
        move_speed_mul: overlay.move_speed_mul.or(base.move_speed_mul),
    })
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
        assert!(emissions.iter().any(|emission| emission.channel == "danger"));
        assert!(emissions.iter().any(|emission| emission.channel == "social"));
        assert!(registry.material_wall_blocking_hint("flint").is_some());
        assert_eq!(registry.structure_build_ticks("stockpile"), Some(36));
        assert_eq!(registry.structure_build_ticks("campfire"), Some(24));
        assert_eq!(registry.structure_resource_cost("shelter", "wood"), Some(4.0));
        assert_eq!(registry.structure_resource_cost("shelter", "stone"), Some(1.0));
        assert_eq!(registry.material_wall_hit_points("oak"), Some(30.0));
        assert!(registry.structure_completion_influence("shelter").is_some());
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
