//! Endpoint receipt regressions; production timing remains a separate gate.
use super::*;
use sim_core::chronicle::ChronicleEntry;

#[test]
fn flag_is_explicit_and_invalid_values_fail() {
    // Given an optional external flag; when parsed; then only explicit 1 enables.
    assert!(!enabled(None).expect("absent"));
    assert!(enabled(Some(std::ffi::OsStr::new("1"))).expect("enabled"));
    assert!(enabled(Some(std::ffi::OsStr::new("0"))).is_err());
    assert!(enabled(Some(std::ffi::OsStr::new("true"))).is_err());
}

#[test]
fn disabled_knowledge_is_not_an_empty_exercised_history() {
    // Given a baseline engine; when observed at construction; then capability is explicit.
    let engine = crate::baseline_engine(0);
    let start = Initial::read(&engine);
    let report = start.finish(&engine, 0).expect("endpoint");
    assert_eq!(report["knowledge_capability"]["enabled"], false);
    assert!(report["knowledge_lost"].is_null());
    assert_eq!(report["terminal_complete"], true);
    assert_eq!(report["initial_population"], crate::perf::alive(&engine));
}

#[test]
fn colony_losses_preserve_duplicate_rows_and_optional_holders() {
    // Given duplicate native losses including an anonymous one; when observed; then neither is dropped.
    let mut engine = crate::colony_engine(0);
    let start = Initial::read(&engine);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::KnowledgeLost).value(0));
    engine.resources.chronicle.push(
        ChronicleEntry::new(0, ChronicleKind::KnowledgeLost)
            .value(0)
            .subject(u64::MAX),
    );
    let before = crate::probe::fingerprint(&engine);
    let report = start.finish(&engine, 0).expect("endpoint");
    let rows = report["knowledge_lost"].as_array().expect("loss list");
    assert_eq!(rows.len(), 2);
    assert!(rows[0]["holder"].is_null());
    assert_eq!(rows[1]["holder"], u64::MAX);
    assert_eq!(crate::probe::fingerprint(&engine), before);
}

#[test]
fn unknown_knowledge_node_fails_closed() {
    // Given an invalid native node; when observed; then no complete receipt is returned.
    let mut engine = crate::colony_engine(0);
    let start = Initial::read(&engine);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(0, ChronicleKind::KnowledgeLost).value(-1));
    assert!(start.finish(&engine, 0).is_err());
}

#[test]
fn early_terminal_and_unverified_prior_history_are_incomplete() {
    // Given a nonzero initial tick without save provenance; when observed; then history/horizon remain false.
    let mut engine = crate::colony_engine(0);
    engine.tick();
    let start = Initial::read(&engine);
    let report = start.finish(&engine, 10).expect("endpoint");
    assert_eq!(report["history_complete"], false);
    assert_eq!(report["terminal_complete"], false);
    assert_eq!(report["terminal_tick"], 1);
    assert_eq!(report["requested_terminal_tick"], 11);
}

#[test]
fn future_chronicle_entries_invalidate_history() {
    // Given an event beyond the engine endpoint; when observed; then history is incomplete.
    let mut engine = crate::colony_engine(0);
    let start = Initial::read(&engine);
    engine
        .resources
        .chronicle
        .push(ChronicleEntry::new(100, ChronicleKind::KnowledgeLost).value(0));
    let report = start.finish(&engine, 0).expect("endpoint");
    assert_eq!(report["history_complete"], false);
}

#[test]
fn sidecar_keeps_raw_bytes_and_refuses_replacement() {
    // Given a raw report path; when saved twice; then first evidence remains and second fails.
    let root = std::env::temp_dir().join(format!("worldsim-perf-observer-{}", std::process::id()));
    std::fs::create_dir(&root).expect("unique directory");
    let raw = root.join("runs").join("n200_r1.json");
    let bytes = "{\"value\": 1}\n";
    write(&raw, bytes, Some(&json!({"terminal_complete": true}))).expect("first receipt");
    let path = root.join("gate_observations/n200_r1.json.safety.json");
    let first = std::fs::read(&path).expect("receipt");
    let receipt: Value = serde_json::from_slice(&first).expect("json");
    assert_eq!(receipt["raw_report_bytes"], bytes);
    assert!(write(&raw, "changed", Some(&json!({}))).is_err());
    assert_eq!(std::fs::read(&path).expect("retained"), first);
    assert!(!root.join("runs").exists());
    std::fs::remove_dir_all(&root).expect("cleanup own test directory");
}
