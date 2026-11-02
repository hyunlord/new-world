//! Mirror lifecycle regressions with injected paths; never mutates process environment.
use super::*;
use crate::probe::gate_observation::GateObservation;
use serde_json::Value;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "gate-mirror-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).expect("new temp");
        Self(root)
    }
    fn records(&self) -> Vec<Value> {
        let mut paths: Vec<_> = std::fs::read_dir(&self.0)
            .expect("mirror root")
            .map(|entry| entry.expect("entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        paths
            .iter()
            .map(|path| serde_json::from_slice(&std::fs::read(path).expect("bytes")).expect("json"))
            .collect()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pending_checkpoint_and_terminal_survive_local_directory_cleanup() {
    let mirror = Temp::new();
    let local = Temp::new();
    let engine = crate::colony_engine(0);
    let mut watch = GateObservation::begin_with_mirror(
        &engine,
        &local.0,
        "colony",
        0,
        "primary",
        0,
        Some(&mirror.0),
    )
    .expect("begin");
    let pending = mirror.records();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["schema"], "worldsim_gate_test_observation_v1");
    assert_eq!(pending[0]["process_id"], std::process::id());
    assert_eq!(pending[0]["observation"]["terminal_complete"], false);
    assert_eq!(pending[0]["observation"]["terminal_tick"], Value::Null);
    watch
        .checkpoint(
            &engine,
            0,
            &serde_json::json!({"tick":0,"cmd":"debug_kill","agent":1}),
            Some((0, 1)),
        )
        .expect("checkpoint");
    assert_eq!(
        mirror.records()[0]["observation"]["checkpoints"]
            .as_array()
            .expect("checkpoints")
            .len(),
        1
    );
    watch.finish(&engine, true).expect("finish");
    let expected_path = local.0.canonicalize().expect("absolute local");
    std::fs::remove_dir_all(&local.0).expect("test caller cleanup");
    let terminal = mirror.records();
    assert_eq!(terminal.len(), 1);
    assert_eq!(terminal[0]["observation"]["terminal_complete"], true);
    assert_eq!(
        terminal[0]["local_output"],
        expected_path.to_str().expect("path")
    );
    assert_eq!(terminal[0]["world_sequence"], pending[0]["world_sequence"]);
}

#[test]
fn repeated_world_at_same_local_path_owns_a_distinct_external_instance() {
    let mirror = Temp::new();
    let local = Temp::new();
    for _ in 0..2 {
        let engine = crate::colony_engine(0);
        let watch = GateObservation::begin_with_mirror(
            &engine,
            &local.0,
            "colony",
            0,
            "primary",
            0,
            Some(&mirror.0),
        )
        .expect("begin");
        watch.finish(&engine, true).expect("finish");
        std::fs::remove_dir_all(local.0.join("gate_observations"))
            .expect("caller resets local outputs");
    }
    let records = mirror.records();
    assert_eq!(records.len(), 2);
    assert_ne!(records[0]["world_sequence"], records[1]["world_sequence"]);
    assert_eq!(records[0]["local_sidecar"], records[1]["local_sidecar"]);
    assert!(records
        .iter()
        .all(|row| row["observation"]["terminal_complete"] == true));
}

#[test]
fn abandoned_observation_leaves_pending_mirror_and_unnamed_thread_is_not_invented() {
    let mirror = Temp::new();
    let local = Temp::new();
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let engine = crate::colony_engine(0);
                let _watch = GateObservation::begin_with_mirror(
                    &engine,
                    &local.0,
                    "colony",
                    0,
                    "primary",
                    5,
                    Some(&mirror.0),
                )
                .expect("begin");
            })
            .join()
            .expect("thread");
    });
    std::fs::remove_dir_all(&local.0).expect("local cleanup");
    let record = &mirror.records()[0];
    assert_eq!(record["thread_name"], Value::Null);
    assert_eq!(record["observation"]["terminal_complete"], false);
    assert_eq!(record["observation"]["terminal_tick"], Value::Null);
}

#[test]
fn mirror_rejects_relative_missing_and_checkout_roots() {
    let local = Temp::new();
    let engine = crate::colony_engine(0);
    for (index, root) in [
        PathBuf::from("relative"),
        local.0.join("missing"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    ]
    .iter()
    .enumerate()
    {
        let result = GateObservation::begin_with_mirror(
            &engine,
            &local.0.join(index.to_string()),
            "colony",
            0,
            "primary",
            0,
            Some(root),
        );
        assert!(result.is_err());
    }
}
