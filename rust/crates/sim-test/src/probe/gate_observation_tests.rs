//! Regression tests for evidence completeness and observer neutrality.
use super::*;
use serde_json::Value;
use sim_core::chronicle::ChronicleEntry;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "gate-observer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
    fn read(&self, scene: &str, occurrence: &str) -> Value {
        serde_json::from_slice(
            &std::fs::read(
                self.0
                    .join("gate_observations")
                    .join(format!("{scene}_s0_{occurrence}.json")),
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
fn pending_and_short_run_cannot_claim_terminal_completion() {
    let dir = Temp::new();
    let mut engine = crate::scenes::colony_scene(0);
    let initial = engine.world.query::<&Agent>().iter().count();
    let watch = GateObservation::begin(&engine, &dir.0, "colony", 0, "primary", 2).expect("begin");
    let pending = dir.read("colony", "primary");
    assert_eq!(pending["initial_population"], initial);
    assert_eq!(pending["initial_tick"], 0);
    assert_eq!(pending["terminal_complete"], false);
    assert_eq!(pending["terminal_tick"], Value::Null);
    assert!(GateObservation::begin(&engine, &dir.0, "colony", 0, "primary", 2).is_err());
    engine.tick();
    watch.finish(&engine, true).expect("finish");
    let terminal = dir.read("colony", "primary");
    assert_eq!(terminal["terminal_tick"], 1);
    assert_eq!(terminal["requested_terminal_tick"], 2);
    assert_eq!(terminal["terminal_complete"], false);
}

#[test]
fn baseline_capability_is_explicit_and_observation_is_read_only() {
    let dir = Temp::new();
    let engine = crate::scenes::baseline_scene(0);
    let before = super::super::fingerprint(&engine);
    let watch =
        GateObservation::begin(&engine, &dir.0, "baseline", 0, "primary", 0).expect("begin");
    watch.finish(&engine, true).expect("finish");
    assert_eq!(before, super::super::fingerprint(&engine));
    let record = dir.read("baseline", "primary");
    assert_eq!(record["knowledge_capability"]["enabled"], false);
    assert_eq!(record["knowledge_lost"], Value::Null);
    assert_eq!(record["history_complete"], true);
    assert_eq!(record["terminal_complete"], true);
}

#[test]
fn complete_native_losses_preserve_optional_holder_and_reject_bad_node() {
    let mut engine = crate::scenes::colony_scene(0);
    for subject in [None, Some(42)] {
        let mut entry = ChronicleEntry::new(0, ChronicleKind::KnowledgeLost);
        entry.subject = subject;
        engine.resources.chronicle.push(entry);
    }
    let losses = knowledge_losses(&engine).expect("loss rows");
    assert_eq!(losses.len(), 2);
    assert_eq!(losses[0].holder, None);
    assert_eq!(losses[1].holder, Some(42));
    assert_eq!(losses[0].node, defs().nodes[0].name);
    let mut bad = ChronicleEntry::new(0, ChronicleKind::KnowledgeLost);
    bad.value = -1;
    engine.resources.chronicle.push(bad);
    assert!(knowledge_losses(&engine).is_err());
}

#[test]
fn loaded_tick_without_prior_history_proof_remains_incomplete() {
    let dir = Temp::new();
    let mut engine = crate::scenes::colony_scene(0);
    for _ in 0..100 {
        engine.tick();
    }
    let watch = GateObservation::begin(&engine, &dir.0, "colony", 0, "primary", 0).expect("begin");
    watch.finish(&engine, true).expect("finish");
    let record = dir.read("colony", "primary");
    assert_eq!(record["terminal_complete"], true);
    assert_eq!(record["history_complete"], false);
    assert_eq!(record["history_source"], "prior_history_unverified");
}

#[test]
fn native_colony_twins_emit_separate_receipts_without_summary_changes() {
    let ordinary = Temp::new();
    let observed = Temp::new();
    let parse = |path: &Path, enabled: bool| {
        let mut argv = vec![
            "--days".into(),
            "1".into(),
            "--workers".into(),
            "1".into(),
            "--check-determinism".into(),
            "--out".into(),
            path.display().to_string(),
        ];
        if enabled {
            argv.push("--gate-observations".into());
        }
        super::super::colony::ColonyArgs::parse(argv).expect("args")
    };
    super::super::colony::run(&parse(&ordinary.0, false)).expect("ordinary");
    super::super::colony::run(&parse(&observed.0, true)).expect("observed");
    let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
    let environment_enabled =
        super::super::ProbeArgs::gate_observations_enabled(false, setting.as_deref())
            .expect("valid gate setting");
    assert_eq!(
        ordinary.0.join("gate_observations").exists(),
        environment_enabled
    );
    assert_eq!(
        std::fs::read(ordinary.0.join("summary.json")).expect("ordinary summary"),
        std::fs::read(observed.0.join("summary.json")).expect("observed summary")
    );
    for role in ["primary", "determinism_twin"] {
        let record = observed.read("colony", role);
        assert_eq!(record["occurrence"], role);
        assert_eq!(record["terminal_tick"], 1440);
        assert_eq!(record["terminal_complete"], true);
        assert_eq!(record["history_complete"], true);
    }
}

#[test]
fn baseline_twins_emit_native_capabilities_without_changing_raw_samples() {
    let ordinary = Temp::new();
    let observed = Temp::new();
    let parse = |path: &Path, enabled: bool| {
        let mut argv = vec![
            "--days".into(),
            "1".into(),
            "--workers".into(),
            "1".into(),
            "--check-determinism".into(),
            "--out".into(),
            path.display().to_string(),
        ];
        if enabled {
            argv.push("--gate-observations".into());
        }
        super::super::ProbeArgs::parse(argv).expect("args")
    };
    super::super::runner::run(&parse(&ordinary.0, false)).expect("ordinary");
    super::super::runner::run(&parse(&observed.0, true)).expect("observed");
    let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
    let environment_enabled =
        super::super::ProbeArgs::gate_observations_enabled(false, setting.as_deref())
            .expect("valid gate setting");
    assert_eq!(
        ordinary.0.join("gate_observations").exists(),
        environment_enabled
    );
    assert_eq!(
        std::fs::read(ordinary.0.join("baseline_s0.jsonl")).expect("ordinary samples"),
        std::fs::read(observed.0.join("baseline_s0.jsonl")).expect("observed samples")
    );
    for role in ["primary", "determinism_twin"] {
        let record = observed.read("baseline", role);
        assert_eq!(record["occurrence"], role);
        assert_eq!(record["terminal_complete"], true);
        assert_eq!(record["history_complete"], true);
        assert_eq!(record["knowledge_capability"]["enabled"], false);
        assert_eq!(record["knowledge_lost"], Value::Null);
    }
}

#[test]
fn failed_loop_and_nonmonotonic_history_cannot_be_complete_evidence() {
    let dir = Temp::new();
    let mut engine = crate::scenes::colony_scene(0);
    let watch = GateObservation::begin(&engine, &dir.0, "colony", 0, "primary", 0).expect("begin");
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(1, ChronicleKind::KnowledgeLost));
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::KnowledgeLost));
    watch.finish(&engine, false).expect("finish");
    let record = dir.read("colony", "primary");
    assert_eq!(record["terminal_complete"], false);
    assert_eq!(record["history_complete"], false);
    assert_eq!(
        record["knowledge_lost"].as_array().expect("losses").len(),
        2
    );
}

#[test]
fn debug_kill_has_actual_tick_1800_pre_tick_and_resolved_command_evidence() {
    let out = Temp::new();
    std::fs::create_dir_all(&out.0).expect("dir");
    let engine = crate::scenes::colony_scene(0);
    let agent = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .min()
        .expect("founder");
    let script = serde_json::json!([{"tick":1800,"cmd":"debug_kill","agent":agent}]);
    let path = out.0.join("script.json");
    std::fs::write(&path, serde_json::to_vec(&script).expect("script json")).expect("script");
    let args = super::super::colony::ColonyArgs::parse(vec![
        "--days".into(),
        "2".into(),
        "--workers".into(),
        "1".into(),
        "--out".into(),
        out.0.display().to_string(),
        "--commands".into(),
        path.display().to_string(),
        "--gate-observations".into(),
        "--check-determinism".into(),
    ])
    .expect("args");
    super::super::colony::run(&args).expect("run");
    for role in ["primary", "determinism_twin"] {
        let record = out.read("colony", role);
        let checkpoints = record["checkpoints"].as_array().expect("checkpoints");
        assert_eq!(checkpoints.len(), 2);
        assert_eq!(checkpoints[0]["phase"], "before_tick_commands");
        assert_eq!(checkpoints[1]["phase"], "before_debug_kill");
        assert_eq!(checkpoints[1]["resolved_command_index"], 0);
        assert_eq!(checkpoints[1]["debug_kill_agent"], agent);
        for checkpoint in checkpoints {
            assert_eq!(checkpoint["tick"], 1800);
            assert_eq!(checkpoint["script_index"], 0);
            assert_eq!(checkpoint["script_spec"], script[0]);
            assert_eq!(checkpoint["history_complete"], true);
        }
    }
}
