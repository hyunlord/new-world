use std::path::PathBuf;

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
    assert!(!data.mental_breaks.is_empty(), "mental break definitions empty");
    assert!(!data.traits.is_empty(), "trait definitions empty");
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
