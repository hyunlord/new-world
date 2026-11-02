//! Observer regressions and the explicitly separate matched-horizon companion workload.
use crate::probe::gate_observation::GateObservation;
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "m6a-gate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("new temp");
        Self(path)
    }
    fn read(&self, scene: &str) -> Value {
        serde_json::from_slice(
            &std::fs::read(
                self.0
                    .join("gate_observations")
                    .join(format!("{scene}_s0_primary.json")),
            )
            .expect("receipt"),
        )
        .expect("json")
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn additive_gate_flag_preserves_valued_parser_and_rejects_duplicates() {
    let original = vec!["--out".into(), "/tmp/new-out".into()];
    let (args, gate) = super::cli::gate_arguments_with_setting(&original, None).expect("ordinary");
    assert!(!gate);
    assert_eq!(args, original);
    let mut enabled = original.clone();
    enabled.push("--gate-observations".into());
    let (args, gate) = super::cli::gate_arguments_with_setting(&enabled, None).expect("observed");
    assert!(gate);
    assert_eq!(args, original);
    assert!(super::cli::arguments(&args, &["--out"]).is_ok());
    enabled.push("--gate-observations".into());
    assert!(super::cli::gate_arguments_with_setting(&enabled, None).is_err());
    assert!(super::cli::arguments(&["--out".into()], &["--out"]).is_err());
}

#[test]
fn food_chain_observes_early_endpoint_without_changing_requested_maximum() {
    let out = Temp::new();
    let mut engine = crate::colony_engine(0);
    let watch = GateObservation::begin_food_chain(&engine, &out.0, 100).expect("begin");
    let pending = out.read("m6a-data-only");
    assert_eq!(pending["schema_version"], 2);
    assert_eq!(pending["terminal_complete"], false);
    assert_eq!(pending["event_horizon"]["condition_met"], Value::Null);
    for _ in 0..20 {
        engine.tick();
    }
    watch
        .finish_food_chain(&engine, true, true)
        .expect("finish");
    let record = out.read("m6a-data-only");
    assert_eq!(record["requested_ticks"], 100);
    assert_eq!(record["requested_terminal_tick"], 100);
    assert_eq!(record["terminal_tick"], 20);
    assert_eq!(record["terminal_complete"], true);
    assert_eq!(record["event_horizon"]["condition_met"], true);
    assert_eq!(record["event_horizon"]["reached_limit"], false);
}

#[test]
fn interrupted_or_unmet_early_event_stays_incomplete_and_limit_is_distinct() {
    for (now, loop_complete, condition_met, complete) in [
        (20, false, true, false),
        (20, true, false, false),
        (100, true, false, true),
        (101, true, true, false),
    ] {
        let out = Temp::new();
        let mut engine = crate::colony_engine(0);
        let watch = GateObservation::begin_food_chain(&engine, &out.0, 100).expect("begin");
        for _ in 0..now {
            engine.tick();
        }
        watch
            .finish_food_chain(&engine, loop_complete, condition_met)
            .expect("finish");
        let record = out.read("m6a-data-only");
        assert_eq!(record["terminal_complete"], complete);
        assert_eq!(record["event_horizon"]["reached_limit"], now == 100);
        assert_eq!(record["event_horizon"]["condition_met"], condition_met);
    }
}

#[test]
fn fixed_horizon_retains_v1_without_event_fields() {
    let out = Temp::new();
    let engine = crate::colony_engine(0);
    let watch = GateObservation::begin(&engine, &out.0, "m6a", 0, "primary", 0).expect("begin");
    watch.finish(&engine, true).expect("finish");
    let record = out.read("m6a");
    assert_eq!(record["schema_version"], 1);
    assert!(record.get("event_horizon").is_none());
}

#[test]
fn m6a_fixed_producer_preserves_existing_outputs_and_emits_one_actual_world() {
    let root = Temp::new();
    let ordinary = root.0.join("ordinary");
    let observed = root.0.join("observed");
    for (out, gate) in [(&ordinary, false), (&observed, true)] {
        let mut args = vec![
            "--seed".into(),
            "0".into(),
            "--days".into(),
            "1".into(),
            "--out".into(),
            out.display().to_string(),
        ];
        if gate {
            args.push("--gate-observations".into());
        }
        super::run(&args).expect("actual m6a run");
    }
    for name in ["events.jsonl", "summary.json", "chronicle.json"] {
        assert_eq!(
            std::fs::read(ordinary.join(name)).expect("ordinary"),
            std::fs::read(observed.join(name)).expect("observed")
        );
    }
    let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
    let environment_enabled =
        crate::probe::ProbeArgs::gate_observations_enabled(false, setting.as_deref())
            .expect("valid gate setting");
    assert_eq!(
        ordinary.join("gate_observations").exists(),
        environment_enabled
    );
    let files: Vec<_> = std::fs::read_dir(observed.join("gate_observations"))
        .expect("sidecars")
        .collect();
    assert_eq!(files.len(), 1);
    let record: Value = serde_json::from_slice(
        &std::fs::read(observed.join("gate_observations/m6a_s0_primary.json")).expect("receipt"),
    )
    .expect("json");
    assert_eq!(record["initial_tick"], 0);
    assert_eq!(record["terminal_tick"], 1440);
    assert_eq!(record["terminal_complete"], true);
}

#[test]
fn matched_horizon_parser_is_explicit_bounded_and_observer_only() {
    let original = vec!["--out".into(), "/tmp/companion-output".into()];
    let (_, ticks) = super::cli::data_only_arguments(&original, false).expect("primary");
    assert_eq!(ticks, None);
    for value in ["0", "-1", "+1", "1.5", "", "18446744073709551616"] {
        let mut args = original.clone();
        args.extend(["--matched-horizon-ticks".into(), value.into()]);
        assert!(super::cli::data_only_arguments(&args, true).is_err());
    }
    let mut args = original;
    args.extend(["--matched-horizon-ticks".into(), "20".into()]);
    assert!(super::cli::data_only_arguments(&args, false).is_err());
    assert_eq!(
        super::cli::data_only_arguments(&args, true)
            .expect("companion")
            .1,
        Some(20)
    );
    args.extend(["--matched-horizon-ticks".into(), "21".into()]);
    assert!(super::cli::data_only_arguments(&args, true).is_err());
    assert_eq!(super::data_only::duration(None, 100).expect("primary"), 100);
    assert_eq!(
        super::data_only::duration(Some(20), 100).expect("short companion"),
        20
    );
    assert_eq!(
        super::data_only::duration(Some(100), 100).expect("maximum"),
        100
    );
    assert!(super::data_only::duration(Some(0), 100).is_err());
    assert!(super::data_only::duration(Some(101), 100).is_err());
}

#[test]
fn selected_pack_companion_runs_beyond_primary_event_in_a_fresh_process() {
    const CHILD: &str = "WORLDSIM_M6A_COMPANION_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let pack = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/m6a_apple/pack.ron");
        let child = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "m6a::gate_tests::selected_pack_companion_runs_beyond_primary_event_in_a_fresh_process", "--nocapture"])
            .env(CHILD, "1")
            .env("WORLDSIM_CONTENT_PACK", pack)
            .env("WORLDSIM_GATE_OBSERVER", "1")
            .output()
            .expect("fresh-process selected pack");
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        assert!(
            String::from_utf8_lossy(&child.stdout).contains("1 passed"),
            "exact child test must execute: {}",
            String::from_utf8_lossy(&child.stdout)
        );
        return;
    }
    crate::content_boot::from_environment().expect("production selected-pack bootstrap");
    let root = Temp::new();
    let primary = root.0.join("primary");
    super::run_data_only(&["--out".into(), primary.display().to_string()])
        .expect("original selected-pack chain oracle");
    let report: Value = serde_json::from_slice(
        &std::fs::read(primary.join("data-only.json")).expect("primary report"),
    )
    .expect("primary json");
    assert_eq!(report["passed"], true);
    let setup_commands = report["commands"].clone();
    let ticks = report["end_tick"]
        .as_u64()
        .expect("actual primary endpoint")
        + 1;
    let companion = root.0.join("companion");
    super::run_data_only(&[
        "--out".into(),
        companion.display().to_string(),
        "--matched-horizon-ticks".into(),
        ticks.to_string(),
    ])
    .expect("companion exact horizon");
    assert!(!companion.join("data-only.json").exists());
    let report: Value = serde_json::from_slice(
        &std::fs::read(companion.join("data-only-companion.json")).expect("companion report"),
    )
    .expect("companion json");
    assert_eq!(report["end_tick"], ticks);
    assert_eq!(report["commands"], setup_commands);
    assert_eq!(report["horizon_complete"], true);
    assert_eq!(report["primary_oracle_satisfied"], false);
    assert!(report.get("passed").is_none());
    let observer: Value = serde_json::from_slice(
        &std::fs::read(companion.join("gate_observations/m6a-data-only-companion_s0_primary.json"))
            .expect("companion observer"),
    )
    .expect("observer json");
    assert_eq!(observer["schema_version"], 1);
    assert_eq!(observer["requested_ticks"], ticks);
    assert_eq!(observer["terminal_tick"], ticks);
    assert_eq!(observer["terminal_complete"], true);
}
