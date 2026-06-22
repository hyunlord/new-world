//! Covering harness for Direction-2 slice 2-5c — deposit/consume HUD feed.
//!
//! This slice is pure GDScript + locale (no Rust/sim/FFI change). The
//! headless-testable surfaces in cargo are therefore:
//!
//! - A7: the two new locale keys resolve (non-literal) in compiled en+ko.
//! - A8: each of the 4 (key × locale) templates has exactly two `%d`.
//! - A9: registry ↔ compiled (en, ko) set-consistency holds across the +2 keys
//!   (the a17 locale-lock regression class, count-independent).
//! - A10: recompile was additive — every pre-existing key is still present in
//!   both locales (0 dropped), incl. the pre-existing HUD feed-family key
//!   `HUD_STOCKPILE_FOOD`.
//! - A11: cross-phase guard for the 2-5a `food_stocks` snapshot contract this
//!   slice depends on. With >= 1 settlement formed, the settlement snapshot rows
//!   the HUD polls are sorted and parallel (the `ids` and `food_stocks`
//!   PackedArrays the GDScript reads are built from these rows in lockstep, so
//!   len-equality + index-alignment follow), and every food_stock is a
//!   non-negative integer.
//!
//! The behavioral feed-derivation logic (A1–A6) lives in `hud_status_panel.gd`
//! and is exercised by the Godot-headless harness at
//! `scripts/test/stockpile_feed_2_5c/harness_2_5c.gd` (cargo cannot drive
//! GDScript). No sim-core assertions are added — A11 only reads the unchanged
//! snapshot contract via the existing collector.
//!
//! Run:
//!   cargo test -p sim-test --test harness_stockpile_feed_2_5c -- --nocapture

use std::collections::HashSet;

use serde_json::Value;

use sim_bridge::ffi::collect_settlement_snapshot;
use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

// Locale artefacts embedded at compile time (deterministic, CWD-independent).
// Path is relative to THIS file: tests/ → sim-test/ → crates/ → rust/ → repo root.
const COMPILED_EN: &str = include_str!("../../../../localization/compiled/en.json");
const COMPILED_KO: &str = include_str!("../../../../localization/compiled/ko.json");
const KEY_REGISTRY: &str = include_str!("../../../../localization/key_registry.json");

const DEPOSIT_KEY: &str = "HUD_FEED_STOCKPILE_DEPOSIT";
const CONSUME_KEY: &str = "HUD_FEED_STOCKPILE_CONSUME";

// ── locale helpers ───────────────────────────────────────────────────────────

/// Parse the `strings` table of a compiled locale file into key→value pairs.
fn parse_strings(raw: &str, label: &str) -> serde_json::Map<String, Value> {
    let doc: Value =
        serde_json::from_str(raw).unwrap_or_else(|e| panic!("{label}: compiled JSON parse: {e}"));
    doc.get("strings")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{label}: compiled file must have a `strings` object"))
        .clone()
}

/// Look up a key's string value in a parsed `strings` table.
fn lookup<'a>(strings: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    strings.get(key).and_then(Value::as_str)
}

/// Count `%d` format specifiers in a template string.
fn count_percent_d(s: &str) -> usize {
    s.matches("%d").count()
}

// ── production-scene helper (mirrors harness_settlements_zero_regression) ──────

const W: u32 = 64;
const H: u32 = 64;
/// 2000 ticks per the plan's A11 — settlements form by ~tick 250 at seed 42
/// (observed 3 held to tick 3000); 2000 gives steady-state margin.
const RUN_TICKS: u64 = 2000;

/// The REAL production scene: bootstrap 64 agents + the 3 startup buildings at
/// (32,32)/(24,32)/(40,32) radius 8, then run RUN_TICKS ticks. Mirrors
/// `world_renderer.gd::_ready()` and harness_settlements_zero_regression.
fn production_scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    for _ in 0..RUN_TICKS {
        e.tick();
    }
    e
}

// ─── Assertion 7: both_new_locale_keys_resolve_in_en_and_ko ────────────────
#[test]
fn harness_2_5c_a7_both_new_locale_keys_resolve_in_en_and_ko() {
    // Type A — a missing key makes Locale.ltr return the literal key string,
    // producing visibly broken HUD text. All 4 lookups (2 keys × 2 locales)
    // must return a non-empty value that is NOT the key literal; en contains
    // "Food", ko contains "식량".
    let en = parse_strings(COMPILED_EN, "en");
    let ko = parse_strings(COMPILED_KO, "ko");

    for key in [DEPOSIT_KEY, CONSUME_KEY] {
        let ev = lookup(&en, key).unwrap_or_else(|| panic!("A7: en missing `{key}`"));
        let kv = lookup(&ko, key).unwrap_or_else(|| panic!("A7: ko missing `{key}`"));
        assert!(!ev.is_empty(), "A7: en `{key}` must be non-empty");
        assert!(!kv.is_empty(), "A7: ko `{key}` must be non-empty");
        assert_ne!(ev, key, "A7: en `{key}` must not equal the key literal");
        assert_ne!(kv, key, "A7: ko `{key}` must not equal the key literal");
        assert!(
            ev.contains("Food"),
            "A7: en `{key}` must contain \"Food\"; got {ev:?}"
        );
        assert!(
            kv.contains("식량"),
            "A7: ko `{key}` must contain \"식량\"; got {kv:?}"
        );
    }
    println!("[2-5c A7] both new keys resolve (non-literal) in en+ko; en has \"Food\", ko has \"식량\" ✓");
}

// ─── Assertion 8: both_new_templates_have_exactly_two_percent_d ────────────
#[test]
fn harness_2_5c_a8_both_new_templates_have_exactly_two_percent_d_in_each_locale() {
    // Type A — the call site formats with `% [sid, amount]` (exactly 2 args), so
    // each template must carry exactly two `%d`. Catches a ko/en asymmetry where
    // one locale drops a specifier (which would mis-format or raise at runtime).
    let en = parse_strings(COMPILED_EN, "en");
    let ko = parse_strings(COMPILED_KO, "ko");

    for (locale, strings) in [("en", &en), ("ko", &ko)] {
        for key in [DEPOSIT_KEY, CONSUME_KEY] {
            let v = lookup(strings, key).unwrap_or_else(|| panic!("A8: {locale} missing `{key}`"));
            let n = count_percent_d(v);
            assert_eq!(
                n, 2,
                "A8: {locale} `{key}` must contain exactly 2 `%d`; got {n} in {v:?}"
            );
        }
    }
    println!("[2-5c A8] all 4 (key × locale) templates contain exactly two %d ✓");
}

// ─── Assertion 9: registry_compiled_consistency_holds_across_plus_two_keys ─
#[test]
fn harness_2_5c_a9_registry_compiled_consistency_holds_across_plus_two_keys() {
    // Type D — the a17 locale-lock regression class (count-independent). The
    // active registry set (keys − removed_keys) must equal the compiled en and
    // ko key sets exactly (symmetric difference == 0 across all three), and the
    // +2 new keys must be present in all three.
    let reg: Value = serde_json::from_str(KEY_REGISTRY).expect("A9: registry JSON parse");
    let all_keys: HashSet<String> = reg
        .get("keys")
        .and_then(Value::as_array)
        .expect("A9: registry must have a `keys` array")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let removed: HashSet<String> = reg
        .get("removed_keys")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
        .unwrap_or_default();
    let active: HashSet<String> = all_keys.difference(&removed).cloned().collect();

    let en: HashSet<String> = parse_strings(COMPILED_EN, "en").keys().cloned().collect();
    let ko: HashSet<String> = parse_strings(COMPILED_KO, "ko").keys().cloned().collect();

    let ae: Vec<&String> = active.symmetric_difference(&en).collect();
    let ak: Vec<&String> = active.symmetric_difference(&ko).collect();
    let ek: Vec<&String> = en.symmetric_difference(&ko).collect();
    assert!(
        ae.is_empty(),
        "A9: registry-active vs en symmetric diff must be empty; got {} e.g. {:?}",
        ae.len(),
        &ae[..ae.len().min(8)]
    );
    assert!(
        ak.is_empty(),
        "A9: registry-active vs ko symmetric diff must be empty; got {} e.g. {:?}",
        ak.len(),
        &ak[..ak.len().min(8)]
    );
    assert!(
        ek.is_empty(),
        "A9: en vs ko symmetric diff must be empty; got {} e.g. {:?}",
        ek.len(),
        &ek[..ek.len().min(8)]
    );

    for key in [DEPOSIT_KEY, CONSUME_KEY] {
        assert!(active.contains(key), "A9: registry-active must contain `{key}`");
        assert!(en.contains(key), "A9: en must contain `{key}`");
        assert!(ko.contains(key), "A9: ko must contain `{key}`");
    }
    println!(
        "[2-5c A9] registry-active({}) == en({}) == ko({}); +2 keys present in all three ✓",
        active.len(),
        en.len(),
        ko.len()
    );
}

// ─── Assertion 10: preexisting_keys_not_dropped (additive recompile) ───────
#[test]
fn harness_2_5c_a10_preexisting_hud_feed_keys_not_dropped() {
    // Type D — recompiling after adding 2 keys must be additive: every
    // pre-existing key (active set minus the 2 new keys) must still resolve in
    // both en and ko (0 dropped). Catches an accidental key deletion / registry
    // truncation during recompile. Explicitly includes the pre-existing HUD
    // feed-family key HUD_STOCKPILE_FOOD (2-5a).
    let reg: Value = serde_json::from_str(KEY_REGISTRY).expect("A10: registry JSON parse");
    let all_keys: HashSet<String> = reg
        .get("keys")
        .and_then(Value::as_array)
        .expect("A10: registry `keys`")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let removed: HashSet<String> = reg
        .get("removed_keys")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
        .unwrap_or_default();
    let active: HashSet<String> = all_keys.difference(&removed).cloned().collect();

    let new_keys: HashSet<String> =
        [DEPOSIT_KEY.to_owned(), CONSUME_KEY.to_owned()].into_iter().collect();
    let preexisting: HashSet<String> = active.difference(&new_keys).cloned().collect();
    assert!(
        !preexisting.is_empty(),
        "A10: there must be pre-existing keys to guard"
    );

    let en: HashSet<String> = parse_strings(COMPILED_EN, "en").keys().cloned().collect();
    let ko: HashSet<String> = parse_strings(COMPILED_KO, "ko").keys().cloned().collect();

    let dropped_en: Vec<&String> = preexisting.iter().filter(|k| !en.contains(*k)).collect();
    let dropped_ko: Vec<&String> = preexisting.iter().filter(|k| !ko.contains(*k)).collect();
    assert!(
        dropped_en.is_empty(),
        "A10: {} pre-existing key(s) dropped from en e.g. {:?}",
        dropped_en.len(),
        &dropped_en[..dropped_en.len().min(8)]
    );
    assert!(
        dropped_ko.is_empty(),
        "A10: {} pre-existing key(s) dropped from ko e.g. {:?}",
        dropped_ko.len(),
        &dropped_ko[..dropped_ko.len().min(8)]
    );

    // Pre-existing HUD feed-family key (2-5a) must survive explicitly.
    assert!(
        en.contains("HUD_STOCKPILE_FOOD") && ko.contains("HUD_STOCKPILE_FOOD"),
        "A10: pre-existing HUD_STOCKPILE_FOOD must remain in both locales"
    );
    println!(
        "[2-5c A10] {} pre-existing keys all retained in en+ko; HUD_STOCKPILE_FOOD present ✓",
        preexisting.len()
    );
}

// ─── Assertion 11: food_stocks_snapshot_parallel_to_ids (cross-phase guard) ─
#[test]
fn harness_2_5c_a11_food_stocks_snapshot_parallel_to_ids() {
    // Type D — the 2-5a contract this whole slice depends on. The GDScript HUD
    // reads `ids` (PackedInt64Array) and `food_stocks` (PackedInt32Array) which
    // the FFI dict builder produces by iterating the collector's rows once, in
    // lockstep. So at the Rust level the guarantees reduce to: >= 1 row, rows
    // strictly ascending by settlement_id (the shared order both arrays inherit
    // ⇒ index alignment), and every food_stock a non-negative integer. The
    // parallel-array construction is simulated to assert len equality.
    let e = production_scene();
    let formed = e.resources.settlements.len();
    assert!(
        formed >= 1,
        "A11: production scene must form >= 1 settlement after {RUN_TICKS} ticks; got {formed}"
    );

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    assert!(
        !rows.is_empty(),
        "A11: snapshot must be non-empty with settlements present (vacuity guard)"
    );

    // Strictly ascending settlement_id == the order both PackedArrays inherit.
    let mut sort_violations = 0usize;
    for w in rows.windows(2) {
        if w[0].settlement_id >= w[1].settlement_id {
            sort_violations += 1;
        }
    }
    assert_eq!(
        sort_violations, 0,
        "A11: rows must be strictly ascending by settlement_id; {sort_violations} violation(s)"
    );

    // Non-negative food_stock for every row (i32; the GDScript reads these as the
    // food_stocks array — negatives would be a contract break).
    let neg = rows.iter().filter(|r| r.food_stock < 0).count();
    assert_eq!(
        neg, 0,
        "A11: every food_stock must be a non-negative integer; {neg} negative row(s)"
    );

    // Simulate the FFI dict builder: build the two parallel arrays from rows and
    // assert index alignment + len equality (what GDScript's bounds-guard relies on).
    let ids: Vec<i64> = rows.iter().map(|r| r.settlement_id as i64).collect();
    let food_stocks: Vec<i32> = rows.iter().map(|r| r.food_stock).collect();
    assert_eq!(
        ids.len(),
        food_stocks.len(),
        "A11: ids and food_stocks arrays must be the same length"
    );
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(ids[i], r.settlement_id as i64, "A11: ids[{i}] index-aligned");
        assert_eq!(food_stocks[i], r.food_stock, "A11: food_stocks[{i}] index-aligned");
    }
    println!(
        "[2-5c A11] {} settlement(s); {} snapshot rows: sorted, parallel ids/food_stocks, non-negative ✓",
        formed,
        rows.len()
    );
}
