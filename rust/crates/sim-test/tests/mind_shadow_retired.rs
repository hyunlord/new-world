//! The retired CLI must refuse before content selection and before producing evidence.
use std::process::Command;

#[test]
fn retired_cli_refuses_without_bootstrap_or_output() {
    let output_path =
        std::env::temp_dir().join(format!("worldsim-retired-shadow-{}", std::process::id()));
    assert!(!output_path.exists(), "test output path must be absent");
    let output = Command::new(env!("CARGO_BIN_EXE_worldsim_probe"))
        .args(["mind-shadow", "--seeds", "0..1", "--days", "20", "--out"])
        .arg(&output_path)
        // This would fail if content bootstrap were reached before the refusal.
        .env("WORLDSIM_CONTENT_PACK", "")
        .output()
        .expect("launch CLI");
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        sim_test::mind_shadow::RETIREMENT_NOTICE
    );
    assert!(output.stdout.is_empty());
    assert!(!output_path.exists());
}

#[test]
fn retired_cli_help_explains_frozen_manifest_without_live_syntax() {
    let output = Command::new(env!("CARGO_BIN_EXE_worldsim_probe"))
        .args(["mind-shadow", "--help"])
        .env("WORLDSIM_CONTENT_PACK", "")
        .output()
        .expect("launch help");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        sim_test::mind_shadow::RETIREMENT_NOTICE
    );
    assert!(output.stderr.is_empty());
}
