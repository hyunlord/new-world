//! Native boundaries for the opt-in ASCII receipt; execution requires the native lane.
use super::*;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "ascii-observer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
    fn read(&self) -> Value {
        serde_json::from_slice(
            &std::fs::read(self.0.join("gate_observations/colony_s100_primary.json"))
                .expect("receipt"),
        )
        .expect("JSON")
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn gate_flag_requires_output_and_rejects_duplicates() {
    // Given an ASCII invocation whose default output is stdout.
    let argv = ["--ascii", "--seeds", "100..101", "--tick", "900"];
    // When the observer is requested with incomplete or duplicate output options.
    for extra in [
        vec!["--gate-observations"],
        vec![
            "--gate-observations",
            "--gate-observations",
            "--out",
            "grid.txt",
        ],
    ] {
        // Then parsing fails before constructing a world.
        assert!(AsciiArgs::parse_with_gate_setting(
            argv.into_iter().chain(extra).map(String::from),
            None
        )
        .is_err());
    }
    let plain = AsciiArgs::parse_with_gate_setting(argv.map(String::from), None)
        .expect("default arguments");
    assert!(!plain.gate_observations);
    let observed = AsciiArgs::parse_with_gate_setting(
        argv.into_iter()
            .chain(["--gate-observations", "--out", "grid.txt"])
            .map(String::from),
        None,
    )
    .expect("observed arguments");
    assert!(observed.gate_observations);
}

#[test]
fn original_u_script_and_renderer_outputs_are_identical_with_observer() {
    // Given the renderer-agreement seed, original script, and tick900 endpoint.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/u_script.json");
    let plain = Temp::new();
    let observed = Temp::new();
    std::fs::create_dir_all(&plain.0).expect("plain output");
    std::fs::create_dir_all(&observed.0).expect("observed output");
    let args = |dir: &Temp, enabled| AsciiArgs {
        seed: 100,
        tick: 900,
        rect: None,
        commands: Some(path.clone()),
        legend: true,
        out: Some(dir.0.join("grid.txt")),
        people_json: Some(dir.0.join("people.json")),
        gate_observations: enabled,
    };
    // When both actual CLI paths run.
    run(&args(&plain, false)).expect("plain renderer");
    run(&args(&observed, true)).expect("observed renderer");
    // Then rendered artifacts are byte-identical and the native receipt retains the replay.
    for file in ["grid.txt", "people.json"] {
        assert_eq!(
            std::fs::read(plain.0.join(file)).expect("plain artifact"),
            std::fs::read(observed.0.join(file)).expect("observed artifact")
        );
    }
    assert!(!plain.0.join("gate_observations").exists());
    let receipt = observed.read();
    assert_eq!(receipt["terminal_tick"], 900);
    assert_eq!(receipt["terminal_complete"], true);
    let script = load_script(&path).expect("script");
    let checkpoints = receipt["checkpoints"].as_array().expect("checkpoints");
    assert_eq!(checkpoints.len(), script.len());
    for (checkpoint, entry) in checkpoints.iter().zip(&script) {
        assert_eq!(checkpoint["tick"], entry.tick);
        assert_eq!(checkpoint["script_spec"], entry.spec);
    }
}

#[test]
fn tick_boundary_and_actual_debug_kill_replay_preserve_public_engine_state() {
    // Given commands before and exactly at the requested endpoint.
    let dir = Temp::new();
    let script = [
        ScriptEntry {
            tick: 0,
            spec: json!({"cmd":"debug_kill","agent":1}),
        },
        ScriptEntry {
            tick: 1,
            spec: json!({"cmd":"set_steward","on":true}),
        },
        ScriptEntry {
            tick: 2,
            spec: json!({"cmd":"set_steward","on":false}),
        },
    ];
    let initial = crate::scenes::colony_scene(100);
    let founders = initial
        .world
        .query::<&sim_core::components::Agent>()
        .iter()
        .count();
    let (mut plain, plain_stumps) = engine_at(100, &script, 2).expect("public engine API");
    // When the same replay is observed.
    let (mut observed, stumps) =
        engine_at_observed(100, &script, 2, Some(&dir.0)).expect("observed replay");
    // Then state, command history, and founding origin remain native and unchanged.
    assert_eq!(
        super::super::fingerprint(&plain),
        super::super::fingerprint(&observed)
    );
    assert_eq!(
        super::super::colony::colony_fingerprint(&plain),
        super::super::colony::colony_fingerprint(&observed)
    );
    assert_eq!(
        format!("{:?}", plain.resources.colony.command_log),
        format!("{:?}", observed.resources.colony.command_log)
    );
    assert_eq!(plain_stumps.tiles(), stumps.tiles());
    let receipt = dir.read();
    assert_eq!(receipt["initial_population"], founders);
    assert_eq!(receipt["initial_tick"], 0);
    assert_eq!(receipt["terminal_tick"], 2);
    assert_eq!(
        receipt["checkpoints"]
            .as_array()
            .expect("checkpoints")
            .len(),
        3
    );
    assert_eq!(receipt["checkpoints"][1]["phase"], "before_debug_kill");
    assert_eq!(receipt["checkpoints"][1]["debug_kill_agent"], 1);
    let actual_losses = observed
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|entry| entry.kind == sim_core::chronicle::ChronicleKind::KnowledgeLost)
        .count();
    assert_eq!(
        receipt["knowledge_lost"]
            .as_array()
            .expect("enabled loss history")
            .len(),
        actual_losses
    );
    assert_eq!(
        sim_systems::save::save(&mut plain).expect("plain state"),
        sim_systems::save::save(&mut observed).expect("observed state")
    );
}

#[test]
fn replay_error_retains_actual_incomplete_endpoint() {
    // Given a command resolution error at tick1.
    let dir = Temp::new();
    let script = [ScriptEntry {
        tick: 1,
        spec: json!({"cmd":"unknown_ascii_command"}),
    }];
    // When replay fails.
    assert!(engine_at_observed(100, &script, 3, Some(&dir.0)).is_err());
    // Then the receipt cannot claim the planned endpoint.
    let receipt = dir.read();
    assert_eq!(receipt["requested_terminal_tick"], 3);
    assert_eq!(receipt["terminal_tick"], 1);
    assert_eq!(receipt["terminal_complete"], false);
}

#[test]
fn zero_tick_snapshot_is_retained_without_applying_tick_zero_commands() {
    // Given a render-only tick0 request with a command that would fail if applied.
    let dir = Temp::new();
    let script = [ScriptEntry {
        tick: 0,
        spec: json!({"cmd":"unknown_ascii_command"}),
    }];
    // When no ticks are requested.
    let _ = engine_at_observed(100, &script, 0, Some(&dir.0)).expect("tick0 render");
    // Then native tick0 is recorded without inventing replay or positive duration.
    let receipt = dir.read();
    assert_eq!(receipt["requested_ticks"], 0);
    assert_eq!(receipt["terminal_tick"], 0);
    assert_eq!(receipt["checkpoints"], json!([]));
    assert_eq!(receipt["history_complete"], true);
}

#[test]
fn environment_opt_in_preserves_cli_and_rejects_invalid_settings() {
    use std::ffi::OsStr;

    let argv = [
        "--ascii", "--seeds", "100..101", "--tick", "900", "--out", "grid.txt",
    ];
    let plain =
        AsciiArgs::parse_with_gate_setting(argv.map(String::from), None).expect("default off");
    assert!(!plain.gate_observations);
    let enabled = AsciiArgs::parse_with_gate_setting(argv.map(String::from), Some(OsStr::new("1")))
        .expect("environment opt-in");
    let explicit = AsciiArgs::parse_with_gate_setting(
        argv.into_iter()
            .chain(["--gate-observations"])
            .map(String::from),
        None,
    )
    .expect("explicit CLI");
    assert_eq!(enabled, explicit);
    let both = AsciiArgs::parse_with_gate_setting(
        argv.into_iter()
            .chain(["--gate-observations"])
            .map(String::from),
        Some(OsStr::new("1")),
    )
    .expect("one flag plus environment is not a duplicate");
    assert_eq!(both, explicit);
    for setting in [None, Some(OsStr::new("1"))] {
        assert!(AsciiArgs::parse_with_gate_setting(
            argv.into_iter()
                .chain(["--gate-observations", "--gate-observations"])
                .map(String::from),
            setting,
        )
        .is_err());
    }
    for value in ["", "0", "true", "2"] {
        for flags in [vec![], vec!["--gate-observations"]] {
            let error = AsciiArgs::parse_with_gate_setting(
                argv.into_iter().chain(flags).map(String::from),
                Some(OsStr::new(value)),
            )
            .expect_err("CLI must not hide an invalid environment setting");
            assert_eq!(error, "WORLDSIM_GATE_OBSERVER must be absent or 1");
        }
    }
    assert!(AsciiArgs::parse_with_gate_setting(
        ["--ascii", "--seeds", "100..101", "--tick", "900"].map(String::from),
        Some(OsStr::new("1")),
    )
    .is_err());
}
