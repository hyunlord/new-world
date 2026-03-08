use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sim_data::{DataLoadError, DataRegistry};

fn crate_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn write_ron_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create ron parent dir");
    }
    fs::write(path, content).expect("failed to write ron file");
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock error")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "worldsim_sim_data_ron_test_{}_{}_{}",
            name,
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn ron_registry_loads_sample_data_and_derives_stats() {
    let data_dir = crate_data_dir();
    let registry = DataRegistry::load_from_directory(&data_dir)
        .expect("expected sample RON registry to load successfully");

    let flint = registry
        .materials
        .get("flint")
        .expect("missing flint material");
    let stats = registry.derive_item_stats("knife", flint);

    assert_eq!(registry.materials.len(), 10);
    assert!(registry.recipes.contains_key("stone_knife"));
    assert!((stats.damage - 8.4).abs() < f64::EPSILON);
    assert!((stats.speed - (5.0 / 2.6)).abs() < f64::EPSILON);
    assert!((stats.durability - 182.0).abs() < f64::EPSILON);
}

#[test]
fn ron_registry_resolves_tag_threshold_queries() {
    let data_dir = crate_data_dir();
    let registry = DataRegistry::load_from_directory(&data_dir)
        .expect("expected sample RON registry to load successfully");
    let recipe = registry
        .recipes
        .get("stone_knife")
        .expect("missing stone_knife recipe");

    let matches = registry.find_materials_by_tag(&recipe.inputs[0]);
    let ids: Vec<&str> = matches.iter().map(|material| material.id.as_str()).collect();

    assert!(ids.contains(&"flint"));
    assert!(ids.contains(&"obsidian"));
    assert!(!ids.contains(&"granite"));
}

#[test]
fn ron_registry_reports_validation_errors_for_unknown_recipe_tags() {
    let temp = TempDirGuard::new("unknown_recipe_tag");
    write_ron_file(
        &temp.path.join("materials").join("stone.ron"),
        r#"[
    MaterialDef(
        id: "flint",
        display_name_key: "MAT_FLINT",
        category: Stone,
        tags: ["stone", "sharp"],
        properties: MaterialProperties(
            hardness: 7.0,
            density: 2.6,
            melting_point: None,
            rarity: 0.3,
            value: 2.5,
        ),
    ),
]"#,
    );
    write_ron_file(
        &temp.path.join("furniture").join("basic.ron"),
        "[]",
    );
    write_ron_file(
        &temp.path.join("structures").join("basic.ron"),
        "[]",
    );
    write_ron_file(
        &temp.path.join("actions").join("basic.ron"),
        "[]",
    );
    write_ron_file(
        &temp.path.join("recipes").join("invalid.ron"),
        r#"[
    RecipeDef(
        id: "bad_recipe",
        display_name_key: "RECIPE_BAD",
        inputs: [
            TagRequirement(
                tag: "missing_tag",
                min_hardness: None,
                min_density: None,
                max_rarity: None,
                amount: 1,
            ),
        ],
        requires: None,
        output: RecipeOutput(
            template: "knife",
            material_from_input: 0,
            count: None,
        ),
        time_ticks: 10,
        skill_tag: None,
        min_skill_level: None,
    ),
]"#,
    );

    let errors =
        DataRegistry::load_from_directory(&temp.path).expect_err("expected validation failure");

    assert!(errors.iter().any(|error| {
        matches!(
            error,
            DataLoadError {
                message,
                ..
            } if message.contains("missing_tag")
        )
    }));
}
