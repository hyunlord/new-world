use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn project_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("missing crates/ parent")
        .parent()
        .expect("missing rust/ parent")
        .parent()
        .expect("missing project root parent")
        .join("data")
}

fn ron_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("crates")
        .join("sim-data")
        .join("data")
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
            "worldsim_sim_data_test_{}_{}_{}",
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

fn write_json_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create json parent dir");
    }
    fs::write(path, content).expect("failed to write json file");
}

#[test]
fn load_all_contains_r1_core_datasets() {
    let data_dir = project_data_dir();
    if !data_dir.exists() {
        eprintln!("Skipping: data dir not found at {:?}", data_dir);
        return;
    }

    let data = sim_data::load_all(&data_dir).expect("load_all failed");
    assert!(!data.emotions.is_empty(), "emotion presets empty");
    assert!(!data.tech.is_empty(), "tech catalog empty");
    assert!(!data.values.is_empty(), "value events empty");
    assert!(!data.stressors.is_empty(), "stressor events empty");
    assert!(!data.coping.is_empty(), "coping definitions empty");
    assert!(
        !data.mental_breaks.is_empty(),
        "mental break definitions empty"
    );
    assert!(!data.traits.is_empty(), "trait definitions empty");
    assert!(!data.species.is_empty(), "species definitions empty");
    assert!(!data.mortality.is_empty(), "mortality profiles empty");
    assert!(
        !data.developmental_stages.is_empty(),
        "developmental stages empty"
    );
    assert!(
        data.attachment.determination_window_days > 0,
        "attachment config invalid"
    );
    assert!(
        !data.occupation.categories.is_empty(),
        "occupation categories empty"
    );
    assert!(!data.occupation.jobs.is_empty(), "job profiles empty");
    assert_eq!(data.occupation.categories.default_job(), "laborer");
}

#[test]
fn data_registry_loads_authoritative_ron_datasets() {
    let data_dir = ron_data_dir();
    if !data_dir.exists() {
        eprintln!("Skipping: RON data dir not found at {:?}", data_dir);
        return;
    }

    let registry = sim_data::DataRegistry::load_from_directory(&data_dir)
        .expect("DataRegistry::load_from_directory failed");
    assert!(!registry.materials.is_empty(), "materials empty");
    assert!(!registry.furniture.is_empty(), "furniture empty");
    assert!(!registry.recipes.is_empty(), "recipes empty");
    assert!(!registry.structures.is_empty(), "structures empty");
    assert!(!registry.actions.is_empty(), "actions empty");
    assert!(registry.world_rules.is_some(), "world rules missing");
    assert!(registry.temperament_rules.is_some(), "temperament rules missing");
}

#[test]
fn stressor_loader_skips_comment_keys() {
    let data_dir = project_data_dir();
    if !data_dir.exists() {
        eprintln!("Skipping: data dir not found at {:?}", data_dir);
        return;
    }

    let stressors = sim_data::load_stressor_events(&data_dir).expect("load_stressor_events failed");
    assert!(
        stressors.get("_comment").is_none(),
        "comment key leaked into stressor map"
    );
    assert!(
        stressors.get("partner_death").is_some(),
        "expected stressor key not found"
    );
}

#[test]
fn species_loader_rejects_empty_species_id() {
    let temp = TempDirGuard::new("species_invalid");
    write_json_file(
        &temp
            .path
            .join("species")
            .join("human")
            .join("species_definition.json"),
        r#"{
            "species_id": "",
            "species_name": "Human",
            "personality_model": "hexaco",
            "personality_path": "res://data/species/human/personality/",
            "emotion_model": "plutchik",
            "emotion_path": "res://data/species/human/emotions/",
            "mortality_model": "siler",
            "mortality_path": "res://data/species/human/mortality/",
            "needs_model": "maslow_erg",
            "needs_path": "res://data/species/human/needs/",
            "base_stats": {},
            "available_cultures": [],
            "species_name_key": "DATA_SPECIES_HUMAN_NAME"
        }"#,
    );

    let result = sim_data::load_species_catalog(&temp.path);
    assert!(result.is_err(), "expected invalid species file to fail");
}

#[test]
fn mortality_loader_rejects_out_of_range_protection_factor() {
    let temp = TempDirGuard::new("mortality_invalid");
    write_json_file(
        &temp
            .path
            .join("species")
            .join("human")
            .join("mortality")
            .join("siler_parameters.json"),
        r#"{
            "model": "siler",
            "baseline": {"a1": 0.6, "b1": 1.3, "a2": 0.01, "a3": 0.00006, "b3": 0.09},
            "tech_modifiers": {"k1": 0.3, "k2": 0.2, "k3": 0.05},
            "care_protection": {"hunger_min": 0.4, "protection_factor": 1.2},
            "season_modifiers": {}
        }"#,
    );

    let result = sim_data::load_mortality_catalog(&temp.path);
    assert!(result.is_err(), "expected invalid mortality file to fail");
}

#[test]
fn developmental_loader_rejects_invalid_age_range_shape() {
    let temp = TempDirGuard::new("developmental_invalid");
    write_json_file(
        &temp.path.join("developmental_stages.json"),
        r#"{
            "infant": {
                "age_range": [0],
                "label_key": "STAGE_INFANT"
            }
        }"#,
    );

    let result = sim_data::load_developmental_stages(&temp.path);
    assert!(
        result.is_err(),
        "expected invalid developmental file to fail"
    );
}

#[test]
fn attachment_loader_rejects_out_of_range_threshold() {
    let temp = TempDirGuard::new("attachment_invalid");
    write_json_file(
        &temp.path.join("attachment_config.json"),
        r#"{
            "determination_window_days": 180,
            "sensitivity_threshold_secure": 1.2,
            "consistency_threshold_secure": 0.6,
            "sensitivity_threshold_anxious": 0.4,
            "consistency_threshold_disorganized": 0.35,
            "abuser_is_caregiver_ace_min": 4,
            "adult_effects": {},
            "protective_factor": {}
        }"#,
    );

    let result = sim_data::load_attachment_config(&temp.path);
    assert!(result.is_err(), "expected invalid attachment file to fail");
}

#[test]
fn occupation_loader_rejects_job_id_filename_mismatch() {
    let temp = TempDirGuard::new("occupation_invalid");
    write_json_file(
        &temp
            .path
            .join("occupations")
            .join("occupation_categories.json"),
        r#"{
            "categories": {"builder": ["construction"]},
            "default": "laborer"
        }"#,
    );
    write_json_file(
        &temp.path.join("jobs").join("builder.json"),
        r#"{
            "job_id": "wrong_id",
            "riasec": "RI",
            "hexaco_ideal": {"H": 0.0, "E": -0.1, "X": 0.0, "A": 0.1, "C": 0.3, "O": 0.1},
            "value_weights": {"HARD_WORK": 0.4},
            "primary_skill": "SKILL_CONSTRUCTION",
            "prestige": 0.45,
            "autonomy_level": 0.40,
            "danger_level": 0.25,
            "creativity_level": 0.35
        }"#,
    );

    let result = sim_data::load_occupation_data(&temp.path);
    assert!(result.is_err(), "expected invalid occupation data to fail");
}
