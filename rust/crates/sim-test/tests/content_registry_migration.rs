//! Genuine historical-source save continuation. Fixtures are external and never regenerated here.
//! Run with WORLDSIM_MUX_FIXTURES=<stage-b> and --ignored; missing inputs are hard failures.
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use sim_core::colony::content::{bootstrap, BootSelection};
use sim_core::colony::{BillMode, Command, CommandRecord, RecipeKind};
use sim_engine::SimEngine;
use sim_systems::{commands::submit, save};

const M55: &str = "ae443aa4a498680ef56211faa6b993533019a6d2";
const MART2: &str = "32cd61a77cb26886be5f13b9924fa55b7d64cbe0";
const STAGE_A: &str = "ca9f531ca6ef700ca223d99dba14d834b94b45da";

fn fixtures() -> PathBuf {
    bootstrap(BootSelection::Bundled).expect("install bundled catalogue before old-ID decoding");
    let path = PathBuf::from(
        std::env::var_os("WORLDSIM_MUX_FIXTURES")
            .expect("WORLDSIM_MUX_FIXTURES must name the external stage-b fixture root"),
    );
    assert!(
        path.is_absolute() && path.is_dir(),
        "invalid fixture root: {}",
        path.display()
    );
    path
}

fn bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path)
        .unwrap_or_else(|error| panic!("required fixture {}: {error}", path.display()))
}

fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&bytes(path))
        .unwrap_or_else(|error| panic!("invalid fixture {}: {error}", path.display()))
}

fn old_save(path: &Path, version: u32, tick: u64, enabled: bool) -> SimEngine {
    let input = bytes(path);
    assert_eq!(
        input.get(..8),
        Some(b"WORLDSIM".as_slice()),
        "{} magic",
        path.display()
    );
    assert_eq!(
        input.get(8..12),
        Some(version.to_le_bytes().as_slice()),
        "{} version",
        path.display()
    );
    let engine =
        save::load(&input).unwrap_or_else(|error| panic!("load {}: {error}", path.display()));
    assert_eq!(engine.resources.world_seed, 5, "{} seed", path.display());
    assert_eq!(engine.current_tick(), tick, "{} tick", path.display());
    assert_eq!(
        engine.resources.steward.enabled,
        enabled,
        "{} steward",
        path.display()
    );
    if version == 6 {
        assert_eq!(
            engine.resources.steward,
            sim_core::colony::steward::StewardState::default(),
            "v6 must install the complete OFF default state"
        );
    }
    engine
}

fn records_json(records: &[CommandRecord]) -> Value {
    Value::Array(records.iter().map(|record| {
        let mut value = sim_test::probe::colony::command_json(record.tick, &record.command);
        value["result"] = json!({"accepted":record.result.accepted,"count":record.result.count,
            "reason":record.result.reason.map(|reason| reason.name())});
        value
    }).collect())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedResult {
    accepted: bool,
    count: u32,
    reason: Option<String>,
}

// This fixture contains concrete add_bill records only. Unknown shapes fail rather than
// being re-resolved through the probe's symbolic script scheduler.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedBill {
    cmd: String,
    tick: u64,
    workshop: u32,
    recipe: String,
    mode: String,
    count: u16,
    result: CapturedResult,
}

fn replay(engine: &mut SimEngine, rows: &[CapturedBill]) {
    let mut next = 0;
    for tick in 2000..3000 {
        assert_eq!(engine.current_tick(), tick);
        while next < rows.len() && rows[next].tick == tick {
            let row = &rows[next];
            assert_eq!(
                row.cmd, "add_bill",
                "only captured concrete schema is supported"
            );
            let mode = match row.mode.as_str() {
                "make" => BillMode::Make(row.count),
                "maintain" => BillMode::Maintain(row.count),
                other => panic!("unknown captured bill mode {other}"),
            };
            let command = Command::AddBill {
                workshop: row.workshop,
                recipe: RecipeKind::from_name(&row.recipe).expect("legacy recipe alias resolves"),
                mode,
            };
            let result = submit(engine, command);
            assert_eq!(
                result.accepted, row.result.accepted,
                "record{next} tick{tick}"
            );
            assert_eq!(result.count, row.result.count, "record{next} tick{tick}");
            assert_eq!(
                result.reason.map(|reason| reason.name()),
                row.result.reason.as_deref(),
                "record{next} tick{tick}"
            );
            next += 1;
        }
        engine.tick();
    }
    assert_eq!(
        next,
        rows.len(),
        "every captured command consumed exactly once"
    );
    assert_eq!(engine.current_tick(), 3000);
}

fn new_bytes(engine: &mut SimEngine) -> Vec<u8> {
    let encoded = save::save(engine).expect("new-format save");
    assert_eq!(
        encoded.get(8..12),
        Some(8u32.to_le_bytes().as_slice()),
        "writer must emit format8"
    );
    encoded
}

fn identical(actual: &[u8], expected: &[u8], context: &str) {
    let first = actual.iter().zip(expected).position(|(a, b)| a != b);
    assert!(
        actual == expected,
        "{context}: complete save bytes differ; sizes {}/{}, first differing shared byte {:?}",
        actual.len(),
        expected.len(),
        first
    );
}

fn historical(label: &str, pin: &str) {
    let root = fixtures();
    let directory = root.join("historical").join(label);
    assert_eq!(
        String::from_utf8(bytes(&directory.join("source-commit.txt")))
            .expect("source pin UTF8")
            .trim(),
        pin
    );
    let receipt = json_file(&root.join("historical/acquisition-receipt.json"));
    assert_eq!(receipt[label]["source"], pin);
    assert_eq!(receipt[label]["continuation_records"], 2);
    let prefix = json_file(&directory.join("prefix-before-2000.json"));
    let continuation = bytes(&directory.join("continuation-2000-3000.json"));
    let rows: Vec<CapturedBill> =
        serde_json::from_slice(&continuation).expect("concrete captured command schema");
    assert_eq!(rows.len(), 2, "frozen acquisition has two commands");
    assert!(rows.windows(2).all(|pair| pair[0].tick <= pair[1].tick));
    assert!(rows.iter().all(|row| (2000..3000).contains(&row.tick)));
    let future_path = directory.join("tick-3000.save");
    let mut future = old_save(&future_path, 6, 3000, false);
    let expected = new_bytes(&mut future);
    for resave_before_continuation in [false, true] {
        let mut engine = old_save(&directory.join("tick-2000.save"), 6, 2000, false);
        assert_eq!(
            records_json(&engine.resources.colony.command_log),
            prefix,
            "{label} preserved prefix"
        );
        if resave_before_continuation {
            let encoded = new_bytes(&mut engine);
            engine = save::load(&encoded).expect("new-format reload");
            assert_eq!(engine.current_tick(), 2000);
            assert_eq!(
                records_json(&engine.resources.colony.command_log),
                prefix,
                "{label} prefix after new reload"
            );
        }
        replay(&mut engine, &rows);
        let actual = new_bytes(&mut engine);
        identical(
            &actual,
            &expected,
            &format!("{label} source {pin}, resave={resave_before_continuation}"),
        );
    }
}

#[test]
#[ignore = "requires external historical source fixtures"]
fn genuine_m55_v6_continuation_and_new_format_roundtrip() {
    historical("m55", M55);
}

#[test]
#[ignore = "requires external historical source fixtures"]
fn genuine_mart2_v6_continuation_and_new_format_roundtrip() {
    historical("mart2", MART2);
}

#[test]
#[ignore = "requires external historical source fixtures"]
fn published_stage_a_v7_steward_continues_without_resubmitting_auto_commands() {
    let root = fixtures();
    let directory = root.join("base");
    assert_eq!(
        json_file(&directory.join("provenance.json"))["head"],
        STAGE_A
    );
    let captured = json_file(&directory.join("on-commands-6000/colony_s5_commands.json"));
    let captured = captured.as_array().expect("captured command array");
    let interval: Vec<_> = captured
        .iter()
        .filter(|row| (5000..6000).contains(&row["tick"].as_u64().expect("record tick")))
        .collect();
    assert_eq!(
        interval.len(),
        3,
        "frozen A continuation has three automatic attempts"
    );
    assert!(
        interval
            .iter()
            .all(|row| row["source"] == "Steward" && row["phase"] == "after_assignment"),
        "fixture interval must contain no external player commands"
    );
    let prefix = Value::Array(
        captured
            .iter()
            .filter(|row| row["tick"].as_u64().expect("tick") < 5000)
            .cloned()
            .collect(),
    );
    let mut future = old_save(&directory.join("on-tick-6000.save"), 7, 6000, true);
    let expected = new_bytes(&mut future);
    for resave_before_continuation in [false, true] {
        let mut engine = old_save(&directory.join("on-tick-5000.save"), 7, 5000, true);
        assert_eq!(
            records_json(&engine.resources.colony.command_log),
            prefix,
            "A preserved automatic prefix"
        );
        if resave_before_continuation {
            let encoded = new_bytes(&mut engine);
            engine = save::load(&encoded).expect("new-format ON reload");
        }
        for tick in 5000..6000 {
            assert_eq!(engine.current_tick(), tick);
            engine.tick();
        }
        assert_eq!(engine.current_tick(), 6000);
        assert!(engine.resources.steward.enabled);
        let actual_interval: Vec<_> = engine
            .resources
            .colony
            .command_log
            .iter()
            .filter(|record| record.tick >= 5000)
            .cloned()
            .collect();
        assert_eq!(
            records_json(&actual_interval),
            Value::Array(interval.iter().map(|row| (*row).clone()).collect()),
            "A automatic attempts regenerate once with exact results"
        );
        identical(
            &new_bytes(&mut engine),
            &expected,
            &format!("Stage A {STAGE_A} ON, resave={resave_before_continuation}"),
        );
    }
}
