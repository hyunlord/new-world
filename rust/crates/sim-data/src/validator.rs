use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{DataRegistry, StructureRequirement};

/// Validation severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Validation failure that should abort loading.
    Error,
    /// Validation warning that should be logged but not abort loading.
    Warning,
}

/// Registry validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Severity of the validation issue.
    pub severity: Severity,
    /// Definition type for the issue.
    pub def_type: &'static str,
    /// Definition identifier associated with the issue.
    pub def_id: String,
    /// Human-readable explanation.
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {} {}: {}",
            self.severity, self.def_type, self.def_id, self.message
        )
    }
}

/// Validate the loaded data registry and return all issues.
pub fn validate_registry(registry: &DataRegistry) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    validate_recipe_tags(registry, &mut errors);
    validate_structure_furniture(registry, &mut errors);
    validate_structure_runtime_fields(registry, &mut errors);
    validate_action_tool_tags(registry, &mut errors);
    validate_recipe_cycles(registry, &mut errors);
    validate_material_ranges(registry, &mut errors);
    validate_display_name_keys(registry, &mut errors);
    errors
}

fn validate_recipe_tags(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    for recipe in registry.recipes.values() {
        for requirement in &recipe.inputs {
            if registry.find_materials_by_tag(requirement).is_empty() {
                errors.push(ValidationError {
                    severity: Severity::Error,
                    def_type: "RecipeDef",
                    def_id: recipe.id.clone(),
                    message: format!(
                        "tag requirement '{}' does not match any material",
                        requirement.tag
                    ),
                });
            }
        }
    }
}

fn validate_structure_furniture(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    for structure in registry.structures.values() {
        for requirement in structure
            .required_components
            .iter()
            .chain(structure.optional_components.iter())
        {
            if let StructureRequirement::Furniture { id, .. } = requirement {
                if !registry.furniture.contains_key(id) {
                    errors.push(ValidationError {
                        severity: Severity::Error,
                        def_type: "StructureDef",
                        def_id: structure.id.clone(),
                        message: format!("references missing furniture id '{id}'"),
                    });
                }
            }
        }
    }
}

fn validate_structure_runtime_fields(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    for structure in registry.structures.values() {
        if structure.build_ticks == 0 {
            errors.push(ValidationError {
                severity: Severity::Error,
                def_type: "StructureDef",
                def_id: structure.id.clone(),
                message: "build_ticks must be > 0".to_string(),
            });
        }

        for (resource_tag, amount) in &structure.resource_costs {
            if resource_tag.trim().is_empty() {
                errors.push(ValidationError {
                    severity: Severity::Error,
                    def_type: "StructureDef",
                    def_id: structure.id.clone(),
                    message: "resource_costs keys must not be empty".to_string(),
                });
            }
            if *amount < 0.0 {
                errors.push(ValidationError {
                    severity: Severity::Error,
                    def_type: "StructureDef",
                    def_id: structure.id.clone(),
                    message: format!(
                        "resource_costs['{resource_tag}'] must be >= 0.0"
                    ),
                });
            }
        }
    }
}

fn validate_action_tool_tags(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    let outputs: HashSet<&str> = registry
        .recipes
        .values()
        .map(|recipe| recipe.output.template.as_str())
        .collect();

    for action in registry.actions.values() {
        if let Some(tool_tag) = &action.tool_tag {
            if !outputs.contains(tool_tag.as_str()) {
                errors.push(ValidationError {
                    severity: Severity::Error,
                    def_type: "ActionDef",
                    def_id: action.id.clone(),
                    message: format!(
                        "tool_tag '{tool_tag}' does not match any recipe output template"
                    ),
                });
            }
        }
    }
}

fn validate_recipe_cycles(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    let mut template_to_recipes: HashMap<&str, Vec<&str>> = HashMap::new();
    for recipe in registry.recipes.values() {
        template_to_recipes
            .entry(recipe.output.template.as_str())
            .or_default()
            .push(recipe.id.as_str());
    }

    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for recipe in registry.recipes.values() {
        let mut edges = Vec::new();
        for requirement in &recipe.inputs {
            if let Some(targets) = template_to_recipes.get(requirement.tag.as_str()) {
                edges.extend(targets.iter().copied());
            }
        }
        graph.insert(recipe.id.as_str(), edges);
    }

    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    for recipe_id in graph.keys().copied() {
        if detect_cycle(recipe_id, &graph, &mut visited, &mut active) {
            errors.push(ValidationError {
                severity: Severity::Error,
                def_type: "RecipeDef",
                def_id: recipe_id.to_string(),
                message: "recipe dependency cycle detected".to_string(),
            });
        }
    }
}

fn detect_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    active: &mut HashSet<&'a str>,
) -> bool {
    if active.contains(node) {
        return true;
    }
    if !visited.insert(node) {
        return false;
    }

    active.insert(node);
    let has_cycle = graph
        .get(node)
        .into_iter()
        .flatten()
        .copied()
        .any(|neighbor| detect_cycle(neighbor, graph, visited, active));
    active.remove(node);
    has_cycle
}

fn validate_material_ranges(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    for material in registry.materials.values() {
        let properties = &material.properties;
        if !(0.0..=10.0).contains(&properties.hardness) {
            errors.push(ValidationError {
                severity: Severity::Error,
                def_type: "MaterialDef",
                def_id: material.id.clone(),
                message: "hardness must be in 0.0..=10.0".to_string(),
            });
        }
        if properties.density <= 0.0 {
            errors.push(ValidationError {
                severity: Severity::Error,
                def_type: "MaterialDef",
                def_id: material.id.clone(),
                message: "density must be > 0.0".to_string(),
            });
        }
        if !(0.0..=1.0).contains(&properties.rarity) {
            errors.push(ValidationError {
                severity: Severity::Error,
                def_type: "MaterialDef",
                def_id: material.id.clone(),
                message: "rarity must be in 0.0..=1.0".to_string(),
            });
        }
    }
}

fn validate_display_name_keys(registry: &DataRegistry, errors: &mut Vec<ValidationError>) {
    let locale_keys = load_locale_keys(&registry.base_path);
    if locale_keys.is_empty() {
        return;
    }

    for material in registry.materials.values() {
        push_missing_display_key(
            "MaterialDef",
            &material.id,
            &material.display_name_key,
            &locale_keys,
            errors,
        );
    }
    for furniture in registry.furniture.values() {
        push_missing_display_key(
            "FurnitureDef",
            &furniture.id,
            &furniture.display_name_key,
            &locale_keys,
            errors,
        );
    }
    for recipe in registry.recipes.values() {
        push_missing_display_key(
            "RecipeDef",
            &recipe.id,
            &recipe.display_name_key,
            &locale_keys,
            errors,
        );
    }
    for structure in registry.structures.values() {
        push_missing_display_key(
            "StructureDef",
            &structure.id,
            &structure.display_name_key,
            &locale_keys,
            errors,
        );
    }
    for action in registry.actions.values() {
        push_missing_display_key(
            "ActionDef",
            &action.id,
            &action.display_name_key,
            &locale_keys,
            errors,
        );
    }
}

fn push_missing_display_key(
    def_type: &'static str,
    def_id: &str,
    display_name_key: &str,
    locale_keys: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if !locale_keys.contains(display_name_key) {
        errors.push(ValidationError {
            severity: Severity::Warning,
            def_type,
            def_id: def_id.to_string(),
            message: format!("display_name_key '{display_name_key}' is missing from localization"),
        });
    }
}

fn load_locale_keys(base_path: &Path) -> HashSet<String> {
    let Some(project_root) = find_project_root(base_path) else {
        return HashSet::new();
    };

    let mut keys = HashSet::new();
    for locale in ["en", "ko"] {
        let locale_dir = project_root.join("localization").join(locale);
        let Ok(entries) = fs::read_dir(&locale_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                        Ok(map) => keys.extend(map.into_keys()),
                        Err(error) => {
                            log::warn!(
                                "failed to parse localization file {}: {}",
                                path.display(),
                                error
                            );
                        }
                    }
                }
                Err(error) => {
                    log::warn!(
                        "failed to read localization file {}: {}",
                        path.display(),
                        error
                    );
                }
            }
        }
    }
    keys
}

fn find_project_root(base_path: &Path) -> Option<PathBuf> {
    base_path.ancestors().find_map(|ancestor| {
        let localization_dir = ancestor.join("localization");
        (localization_dir.join("en").exists() && localization_dir.join("ko").exists())
            .then(|| ancestor.to_path_buf())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::{
        ActionCategory, ActionDef, DataRegistry, MaterialCategory, MaterialDef, MaterialProperties,
        RecipeDef, RecipeOutput, StructureDef, TagIndex, TagRequirement,
    };

    fn sample_registry() -> DataRegistry {
        let material = Arc::new(MaterialDef {
            id: "flint".to_string(),
            display_name_key: "MAT_FLINT".to_string(),
            category: MaterialCategory::Stone,
            tags: ["stone".to_string(), "sharp".to_string()]
                .into_iter()
                .collect(),
            properties: MaterialProperties {
                hardness: 7.0,
                density: 2.6,
                melting_point: None,
                rarity: 0.3,
                value: 2.5,
            },
        });

        let materials = HashMap::from([(material.id.clone(), Arc::clone(&material))]);
        let recipes = HashMap::from([(
            "stone_knife".to_string(),
            Arc::new(RecipeDef {
                id: "stone_knife".to_string(),
                display_name_key: "RECIPE_STONE_KNIFE".to_string(),
                inputs: vec![TagRequirement {
                    tag: "sharp".to_string(),
                    min_hardness: Some(4.0),
                    min_density: None,
                    max_rarity: None,
                    amount: 1,
                }],
                requires: None,
                output: RecipeOutput {
                    template: "knife".to_string(),
                    material_from_input: 0,
                    count: None,
                },
                time_ticks: 60,
                skill_tag: None,
                min_skill_level: None,
            }),
        )]);
        let actions = HashMap::from([(
            "prepare_hide".to_string(),
            Arc::new(ActionDef {
                id: "prepare_hide".to_string(),
                display_name_key: "ACTION_PREPARE_HIDE".to_string(),
                category: ActionCategory::Craft,
                preconditions: Vec::new(),
                effects: Vec::new(),
                base_duration_ticks: 10,
                tool_tag: Some("knife".to_string()),
                skill_tag: None,
                animation_key: None,
            }),
        )]);

        DataRegistry {
            base_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
            materials,
            furniture: HashMap::new(),
            recipes,
            structures: HashMap::<String, Arc<StructureDef>>::new(),
            actions,
            world_rules: None,
            temperament_rules: None,
            tag_index: TagIndex::build(&HashMap::from([(
                "flint".to_string(),
                Arc::clone(&material),
            )])),
        }
    }

    #[test]
    fn validator_accepts_basic_registry() {
        let registry = sample_registry();
        let errors = validate_registry(&registry);

        assert!(errors
            .iter()
            .all(|error| error.severity == Severity::Warning));
    }

    #[test]
    fn validator_reports_missing_recipe_tag_matches() {
        let mut registry = sample_registry();
        registry.recipes.insert(
            "bad_recipe".to_string(),
            Arc::new(RecipeDef {
                id: "bad_recipe".to_string(),
                display_name_key: "RECIPE_BAD".to_string(),
                inputs: vec![TagRequirement {
                    tag: "missing".to_string(),
                    min_hardness: None,
                    min_density: None,
                    max_rarity: None,
                    amount: 1,
                }],
                requires: None,
                output: RecipeOutput {
                    template: "tool".to_string(),
                    material_from_input: 0,
                    count: None,
                },
                time_ticks: 10,
                skill_tag: None,
                min_skill_level: None,
            }),
        );

        let errors = validate_registry(&registry);
        assert!(errors.iter().any(|error| error.message.contains("missing")));
    }
}
