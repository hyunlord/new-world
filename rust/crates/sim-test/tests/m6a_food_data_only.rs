//! Fresh selected-content process proves a data-added species across the real food chain.
use std::path::Path;

#[test]
fn apple_addition_runs_in_a_fresh_process_without_simulation_code_changes() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = std::env::temp_dir().join(format!("worldsim-m6a-apple-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh external evidence root");
    let output = root.join("observed");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_worldsim_probe"))
        .args(["m6a-data-only", "--out"])
        .arg(&output)
        .env(
            "WORLDSIM_CONTENT_PACK",
            checkout.join("rust/crates/sim-test/data/m6a_apple/pack.ron"),
        )
        .output()
        .expect("selected content process");
    std::fs::write(root.join("stdout.log"), &result.stdout).expect("stdout evidence");
    std::fs::write(root.join("stderr.log"), &result.stderr).expect("stderr evidence");
    assert!(
        result.status.success(),
        "apple fixture failed; evidence at {}",
        root.display()
    );
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("data-only.json")).expect("report"))
            .expect("JSON");
    assert_eq!(value["passed"], true);
    assert_eq!(value["category_fallback"], true);
    assert!(value["intake_evidence"]
        .as_array()
        .is_some_and(|events| !events.is_empty()));
}
