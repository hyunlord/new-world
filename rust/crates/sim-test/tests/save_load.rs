//! Save and load (M1.5 §7, contract clause C1): `save → load → tick` equals
//! `tick` without the save, two saves of one state are byte-identical, and a
//! save of another format version is rejected.
//!
//! The colony runs apply the U-script (`tools/v8/u_script.json`, its orders at
//! tick 360) the way the colony probe does: resolve against the engine at the
//! entry's tick, `commands::submit`, then tick.

use sim_engine::SimEngine;
use sim_systems::save::{self, SaveError, FORMAT_VERSION};
use sim_test::probe::colony::{colony_fingerprint, load_script, resolve, ScriptEntry};
use sim_test::probe::fingerprint;

/// Ticks before the save.
const SAVE_AT: u64 = 2_000;
/// Ticks after it.
const AFTER: u64 = 1_000;
/// Ticks after the save at which the loaded engine is compared.
const CHECKS: [u64; 7] = [0, 1, 2, 10, 120, 500, AFTER];

fn script() -> Vec<ScriptEntry> {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/u_script.json");
    let entries = load_script(&path).expect("u_script.json loads");
    assert!(!entries.is_empty(), "the U-script has entries");
    entries
}

/// Advance `engine` to tick `to`, applying the script entries due on the way.
fn advance(engine: &mut SimEngine, script: &[ScriptEntry], to: u64) {
    while engine.current_tick() < to {
        let tick = engine.current_tick();
        let mut avoid = Vec::new();
        for entry in script.iter().filter(|e| e.tick == tick) {
            for cmd in resolve(engine, &entry.spec, &mut avoid).expect("script resolves") {
                let _ = sim_systems::commands::submit(engine, cmd);
            }
        }
        engine.tick();
    }
}

/// Everything compared between a continuous and a loaded engine at a tick.
fn observe(engine: &SimEngine) -> (String, String, usize, usize) {
    (
        colony_fingerprint(engine),
        fingerprint(engine),
        engine.resources.chronicle.len(),
        engine.resources.chronicle.person_count(),
    )
}

fn assert_same(label: &str, a: &SimEngine, b: &SimEngine) {
    assert_eq!(a.current_tick(), b.current_tick(), "{label}: tick");
    assert_eq!(observe(a), observe(b), "{label}: fingerprints");
    assert!(
        a.resources.chronicle == b.resources.chronicle,
        "{label}: chronicle entries or people differ"
    );
}

/// Tick `continuous` and a save → load copy of it in lockstep from `SAVE_AT`
/// and compare them; at the end compare full saves of both (every resource,
/// every entity in archetype order, the allocator, every system's state).
fn round_trip(
    label: &str,
    mut continuous: SimEngine,
    mut saved: SimEngine,
    script: &[ScriptEntry],
) {
    let bytes = save::save(&mut saved).expect("save");
    let mut loaded = save::load(&bytes).expect("load");
    for &after in &CHECKS {
        let to = SAVE_AT + after;
        advance(&mut continuous, script, to);
        advance(&mut saved, script, to);
        advance(&mut loaded, script, to);
        assert_same(&format!("{label} +{after} (loaded)"), &continuous, &loaded);
        assert_same(&format!("{label} +{after} (saved)"), &continuous, &saved);
    }
    let end_continuous = save::save(&mut continuous).expect("save continuous");
    let end_loaded = save::save(&mut loaded).expect("save loaded");
    let end_saved = save::save(&mut saved).expect("save saved");
    assert!(
        end_continuous == end_loaded,
        "{label}: full state at {} differs after load",
        SAVE_AT + AFTER
    );
    assert!(
        end_continuous == end_saved,
        "{label}: full state differs after save"
    );
}

#[test]
fn save_load_roundtrip() {
    let script = script();
    std::thread::scope(|scope| {
        for seed in 0..8u64 {
            let script = &script;
            scope.spawn(move || {
                let mut a = sim_test::colony_engine(seed);
                let mut b = sim_test::colony_engine(seed);
                advance(&mut a, script, SAVE_AT);
                advance(&mut b, script, SAVE_AT);
                // The run did something worth saving: the orders were applied
                // and logs moved (observed: 3 commands; trees felled > 0).
                assert_eq!(a.resources.colony.command_log.len(), 3, "seed {seed}");
                assert!(a.resources.colony.tallies.trees_felled > 0, "seed {seed}");
                round_trip(&format!("colony seed {seed}"), a, b, script);
            });
        }
    });
}

#[test]
fn save_load_roundtrip_baseline() {
    // The baseline world runs the V7 systems the colony leaves off (fission
    // and relations rules on; their state is saved too).
    std::thread::scope(|scope| {
        for seed in 0..2u64 {
            scope.spawn(move || {
                let mut a = sim_test::baseline_engine(seed);
                let mut b = sim_test::baseline_engine(seed);
                advance(&mut a, &[], SAVE_AT);
                advance(&mut b, &[], SAVE_AT);
                round_trip(&format!("baseline seed {seed}"), a, b, &[]);
            });
        }
    });
}

fn deaths(engine: &SimEngine) -> u64 {
    let t = engine.resources.tallies;
    t.deaths_starvation
        + t.deaths_dehydration
        + t.deaths_combat
        + t.deaths_old_age
        + t.deaths_exposure
}

#[test]
fn save_load_roundtrip_through_deaths_and_births() {
    // Baseline seed 0, saved on the tick after its first death after tick
    // 20,000 (observed: deaths by tick 30,000 = 1, births = 20; by 100,000
    // 41 and 96): the save holds a non-empty hecs free list (asserted), and
    // births after the load pop it. Two chained save → load steps; full saves
    // compared at the end (entity ids and generations, archetype order, the
    // allocator). Breaking the free-list generations in `save::world::build`
    // fails this test (checked once on purpose).
    let mut a = sim_test::baseline_engine(0);
    let mut b = sim_test::baseline_engine(0);
    advance(&mut a, &[], 20_000);
    advance(&mut b, &[], 20_000);
    let before = deaths(&a);
    while deaths(&a) == before && a.current_tick() < 100_000 {
        a.tick();
        b.tick();
    }
    assert!(
        deaths(&a) > before,
        "a death between ticks 20,000 and 100,000"
    );
    let births_at_save = a.resources.tallies.births;
    let first_save = b.current_tick();
    for (step, to) in [(1, first_save + 3_000), (2, first_save + 12_000)] {
        let bytes = save::save(&mut b).expect("save");
        if step == 1 {
            let saved = save::world::capture(&b.world).expect("capture");
            assert!(saved.free_len() >= 1, "the save holds a free id");
        }
        b = save::load(&bytes).expect("load");
        for check in [b.current_tick() + 1, b.current_tick() + 60, to] {
            advance(&mut a, &[], check);
            advance(&mut b, &[], check);
            assert_same(&format!("baseline chain {step} tick {check}"), &a, &b);
        }
    }
    assert!(
        a.resources.tallies.births > births_at_save,
        "births after the first load ({} -> {})",
        births_at_save,
        a.resources.tallies.births
    );
    let end_a = save::save(&mut a).expect("save a");
    let end_b = save::save(&mut b).expect("save b");
    assert!(end_a == end_b, "full state differs after two loads");
}

#[test]
fn save_is_deterministic() {
    let script = script();
    let mut a = sim_test::colony_engine(3);
    let mut b = sim_test::colony_engine(3);
    advance(&mut a, &script, 1_500);
    advance(&mut b, &script, 1_500);
    let first = save::save(&mut a).expect("save");
    let twin = save::save(&mut b).expect("save twin");
    // Two engines hold their hash maps under different hash seeds; the
    // sorted writing makes their saves equal anyway.
    assert!(first == twin, "two saves of one state differ");
    let again = save::save(&mut a).expect("second save");
    assert!(first == again, "a second save of the same engine differs");
    let mut loaded = save::load(&first).expect("load");
    let reloaded = save::save(&mut loaded).expect("save loaded");
    assert!(first == reloaded, "save → load → save differs");
    assert!(
        first.len() > 100_000,
        "a colony save holds the world ({} bytes)",
        first.len()
    );
}

#[test]
fn bridge_save_files_round_trip() {
    // The pure functions behind `WorldSimNode::save_world` / `load_world`.
    let script = script();
    let dir = std::env::temp_dir().join(format!("worldsim-save-test-{}", std::process::id()));
    let path = dir.join("saves").join("0-1200.wsave");
    let mut engine = sim_test::colony_engine(0);
    advance(&mut engine, &script, 1_200);
    let written = sim_bridge::ffi::save_engine_to_file(&mut engine, &path).expect("save file");
    assert_eq!(
        std::fs::metadata(&path).map(|m| m.len()).ok(),
        Some(written as u64)
    );
    let mut loaded = sim_bridge::ffi::load_engine_from_file(&path).expect("load file");
    advance(&mut engine, &script, 1_500);
    advance(&mut loaded, &script, 1_500);
    assert_same("bridge file", &engine, &loaded);
    assert!(sim_bridge::ffi::load_engine_from_file(&dir.join("missing.wsave")).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_rejects_other_version() {
    let mut engine = sim_test::colony_engine(0);
    engine.tick();
    let bytes = save::save(&mut engine).expect("save");
    assert!(save::load(&bytes).is_ok());
    let mut other = bytes.clone();
    other[8..12].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
    match save::load(&other) {
        Err(SaveError::Version { found, expected }) => {
            assert_eq!((found, expected), (FORMAT_VERSION + 1, FORMAT_VERSION));
        }
        Err(e) => panic!("wrong error: {e}"),
        Ok(_) => panic!("a save of another version loaded"),
    }
    let mut not_a_save = bytes.clone();
    not_a_save[0] = b'X';
    assert!(matches!(save::load(&not_a_save), Err(SaveError::NotASave)));
    assert!(matches!(
        save::load(&bytes[..bytes.len() / 2]),
        Err(SaveError::Decode(_))
    ));
}

/// Tick `engine` (no orders) until `stop` holds, at most `limit` ticks.
fn run_until(engine: &mut SimEngine, limit: u64, stop: impl Fn(&SimEngine) -> bool) -> bool {
    for _ in 0..limit {
        if stop(engine) {
            return true;
        }
        engine.tick();
    }
    stop(engine)
}

/// Save at `engine`'s current tick, load, and tick the loaded and a twin
/// continuous engine `ticks` further; both must stay equal.
fn compare_after_load(label: &str, mut twin: SimEngine, mut engine: SimEngine, ticks: u64) {
    let bytes = save::save(&mut engine).expect("save");
    let mut loaded = save::load(&bytes).expect("load");
    for step in 0..ticks {
        twin.tick();
        loaded.tick();
        if step % 240 == 0 || step + 1 == ticks {
            assert_same(&format!("{label} +{}", step + 1), &twin, &loaded);
        }
    }
    let a = save::save(&mut twin).expect("save twin");
    let b = save::save(&mut loaded).expect("save loaded");
    assert!(a == b, "{label}: full state differs after load");
}

#[test]
fn save_load_roundtrip_through_a_pregnancy_and_a_due_birth() {
    // Review MINOR-3: save while a woman is pregnant, and on the tick after
    // FertilitySystem queued a birth (12:00) before SettlementSystem spawns it
    // (12:01); the loaded engine must give the same births as the continuous one.
    let pregnant = |e: &SimEngine| {
        e.world
            .query::<&sim_core::components::Pregnant>()
            .iter()
            .count()
            > 0
    };
    let queued = |e: &SimEngine| !e.resources.due_births.is_empty();
    let mut a = sim_test::colony_engine(1);
    let mut b = sim_test::colony_engine(1);
    assert!(
        run_until(&mut a, 40 * 1_440, pregnant),
        "no pregnancy in 40 days"
    );
    assert!(run_until(&mut b, 40 * 1_440, pregnant));
    let births = a.resources.tallies.births;
    compare_after_load("seed 1 mid-pregnancy", a, b, 1_000);
    let mut a = sim_test::colony_engine(1);
    let mut b = sim_test::colony_engine(1);
    assert!(
        run_until(&mut a, 60 * 1_440, queued),
        "no queued birth in 60 days"
    );
    assert!(run_until(&mut b, 60 * 1_440, queued));
    assert_eq!(births, 0, "the first save was before any birth");
    let before = a.resources.tallies.births;
    let waiting = a.resources.due_births.len() as u64;
    let mut probe = save::load(&save::save(&mut a).expect("save")).expect("load");
    assert_eq!(probe.resources.due_births.len() as u64, waiting);
    probe.tick();
    assert_eq!(
        probe.resources.tallies.births,
        before + waiting,
        "the queued births survived the load"
    );
    compare_after_load("seed 1 due birth queued", a, b, 2_000);
}
