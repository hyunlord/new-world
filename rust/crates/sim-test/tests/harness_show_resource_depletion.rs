//! V7 viz-A `show-resource-depletion` — FFI `amount` / `max` surface for the
//! depletion/regen marker visual.
//!
//! The renderer marker's `scale` AND `alpha` track each source's `amount / max`
//! ratio, so the FFI resource snapshot must carry the live tile counter
//! (`amount`) and the source ceiling (`max`) alongside the existing
//! `(x, y, kind)`. These are pure-Rust assertions on the marshalling (no Godot
//! runtime), the same surface `harness_s16_alpha0` locks for `(xs, ys, kinds)` —
//! here extended for `amounts` / `maxes`. The renderer wiring itself
//! (`world_renderer.gd::_update_resource_markers`) is verified by the pipeline's
//! Visual Verify + the source-token retargets in `harness_s16_gamma_visual`
//! (A11) and `harness_t7_10_b1_space_toggle` (A12), not here.
//!
//! Plan (plan_attempt 2) assertions implemented here (A11/A12 live in the two
//! retargeted GDScript-source-token harnesses):
//!
//!   A1  amounts/maxes present + aligned with rows (== rows.len(), == 6) ... Type A
//!   A2  amount + max correct for EVERY kind (food/water/sleep distinct) .. Type A
//!   A3  ratio reflects depletion (full == 1.0, depleted < full, ordered) . Type A
//!   A4  unregistered (infinite-sentinel 255) source → max == amount ..... Type A
//!   A5  (xs, ys, kinds) split contract preserved over the 6-row layout ... Type D
//!   A6  zero-amount sleep row exercises the .max(1) floor → ratio 0.0 .... Type A
//!   A7  over-capacity (50/45) + explicit zero-cap (50/0) → amount <= max . Type A
//!   A8  multi-kind index lockstep: amounts[i]/maxes[i] match (xs,ys,kinds) Type A
//!   A9  production scene registers finite capacities (max != 255, >= 1) .. Type A
//!   A10 live depletion surfaces (amount < max OR source count decreased) . Type A
//!
//! Note on A9/A10: the plan's metric names `make_stage1_engine(42, 20)`, but
//! the production stage-1 scene that actually seeds FINITE, depletable source
//! capacities is the dylib's own construction path,
//! [`init_production_engine`] (register systems → bootstrap 64-agent lattice +
//! INFINITE seeds → `seed_finite_resource_scarcity` overwrite to finite). A bare
//! `make_stage1_engine` helper seeds NO resource sources, so it cannot exhibit
//! the finite-capacity / depletion the plan's anti-circularity guards require.
//! `init_production_engine()` is the faithful realisation of "the production
//! stage-1 engine"; the LOCKED thresholds (>= 1 finite capacity; >= 1 depleted
//! row OR a reduced source count) are unchanged.
//!
//! Run: `cargo test -p sim-test --test harness_show_resource_depletion -- --nocapture`

use sim_bridge::ffi::world_node::init_production_engine;
use sim_bridge::ffi::{collect_resource_snapshot, resource_rows_amounts, resource_rows_split};
use sim_core::material::MaterialRegistry;
use sim_engine::{SimResources, RESOURCE_SOURCE_INFINITE};

const W: u32 = 64;
const H: u32 = 64;

/// Fresh empty resource backend for the hand-seeded collector assertions.
fn empty_resources() -> SimResources {
    SimResources::new(W, H, MaterialRegistry::new())
}

/// The canonical 6-row multi-kind fixture shared by A1, A5, A8: 3 food + 2
/// water + 1 sleep, each with a DISTINCT `(amount, source_max)` pair so no
/// value can be confused across registries, and all `amount <= max` so no
/// clamp perturbs the lockstep / order checks. Distinct coordinates per kind →
/// row count == 6 (no cross-kind collisions). Matches the layout pinned in the
/// plan's A8 metric (food (8,8)=30/45, (2,5)=10/45, (10,10)=45/45;
/// water (12,4)=18/40, (6,6)=40/40; sleep (3,15)=7/20).
fn seed_six_row_multi_kind(res: &mut SimResources) {
    // food (kind 0) — 3 tiles.
    res.set_food_tile(8, 8, 30);
    res.set_food_source_max(8, 8, 45);
    res.set_food_tile(2, 5, 10);
    res.set_food_source_max(2, 5, 45);
    res.set_food_tile(10, 10, 45);
    res.set_food_source_max(10, 10, 45);
    // water (kind 1) — 2 tiles.
    res.set_water_tile(12, 4, 18);
    res.set_water_source_max(12, 4, 40);
    res.set_water_tile(6, 6, 40);
    res.set_water_source_max(6, 6, 40);
    // sleep (kind 2) — 1 tile.
    res.set_sleep_tile(3, 15, 7);
    res.set_sleep_source_max(3, 15, 20);
}

// ─── A1: amounts/maxes present, length-aligned with rows (3+2+1 = 6) ────────
// Type A — parallel-array FFI contract. amounts[i]/maxes[i] index in lockstep
// with xs[i]/ys[i]/kinds[i]; the common length must equal rows.len() and the
// seeded tile count (3 food + 2 water + 1 sleep = 6). Structural only — value
// correctness is delegated to A2–A8.
#[test]
fn harness_resource_depletion_a1_amounts_maxes_present_and_aligned() {
    let mut res = empty_resources();
    seed_six_row_multi_kind(&mut res);

    let rows = collect_resource_snapshot(&res);
    let (xs, ys, kinds) = resource_rows_split(&rows);
    let (amounts, maxes) = resource_rows_amounts(&rows);

    // Type A — all five arrays + rows.len() must share one length == 6.
    assert_eq!(rows.len(), 6, "A1: 3 food + 2 water + 1 sleep == 6 rows");
    assert_eq!(amounts.len(), maxes.len(), "A1: amounts.len() == maxes.len()");
    assert_eq!(maxes.len(), xs.len(), "A1: maxes.len() == xs.len()");
    assert_eq!(xs.len(), ys.len(), "A1: xs.len() == ys.len()");
    assert_eq!(ys.len(), kinds.len(), "A1: ys.len() == kinds.len()");
    assert_eq!(kinds.len(), rows.len(), "A1: kinds.len() == rows.len()");
    assert_eq!(amounts.len(), 6, "A1: common length == seeded source count 6");
    println!("[viz-A A1] all arrays length {} == rows.len() == 6 ✓", amounts.len());
}

// ─── A2: amount + max correct for EVERY kind (cross-kind registry guard) ────
// Type A — closes the cross-kind registry-swap gaming vector. Each kind reads
// its OWN *_tiles map for amount and its OWN *_source_max registry for max.
// Values are all-distinct (30/45, 18/40, 7/20) so any cross-wire fails. ALL
// THREE kinds must pass.
#[test]
fn harness_resource_depletion_a2_amount_and_max_correct_for_every_kind() {
    let mut res = empty_resources();
    // Exactly one finite source of EACH kind, distinct non-symmetric pairs.
    res.set_food_tile(8, 8, 30);
    res.set_food_source_max(8, 8, 45);
    res.set_water_tile(12, 4, 18);
    res.set_water_source_max(12, 4, 40);
    res.set_sleep_tile(3, 15, 7);
    res.set_sleep_source_max(3, 15, 20);

    let rows = collect_resource_snapshot(&res);
    let (amounts, maxes) = resource_rows_amounts(&rows);

    // food (kind 0) → amount 30, max 45.
    let fi = rows
        .iter()
        .position(|r| r.kind == 0 && r.x == 8 && r.y == 8)
        .expect("A2: food row (8,8) present");
    assert_eq!(rows[fi].amount, 30, "A2: food amount == food_tiles value 30");
    assert_eq!(rows[fi].max, 45, "A2: food max == food_source_max 45");
    assert_eq!(amounts[fi], 30, "A2: amounts[food] mirrors row.amount 30");
    assert_eq!(maxes[fi], 45, "A2: maxes[food] mirrors row.max 45");

    // water (kind 1) → amount 18, max 40.
    let wi = rows
        .iter()
        .position(|r| r.kind == 1 && r.x == 12 && r.y == 4)
        .expect("A2: water row (12,4) present");
    assert_eq!(rows[wi].amount, 18, "A2: water amount == water_tiles value 18");
    assert_eq!(rows[wi].max, 40, "A2: water max == water_source_max 40");
    assert_eq!(amounts[wi], 18, "A2: amounts[water] mirrors row.amount 18");
    assert_eq!(maxes[wi], 40, "A2: maxes[water] mirrors row.max 40");

    // sleep (kind 2) → amount 7, max 20.
    let si = rows
        .iter()
        .position(|r| r.kind == 2 && r.x == 3 && r.y == 15)
        .expect("A2: sleep row (3,15) present");
    assert_eq!(rows[si].amount, 7, "A2: sleep amount == sleep_tiles value 7");
    assert_eq!(rows[si].max, 20, "A2: sleep max == sleep_source_max 20");
    assert_eq!(amounts[si], 7, "A2: amounts[sleep] mirrors row.amount 7");
    assert_eq!(maxes[si], 20, "A2: maxes[sleep] mirrors row.max 20");

    println!("[viz-A A2] food 30/45, water 18/40, sleep 7/20 — every kind correct ✓");
}

// ─── A3: ratio reflects depletion (full == 1.0, depleted < full) ────────────
// Type A — the feature's reason for existing. A depleted source reports
// amount < max (ratio < 1) while a full source reports amount == max (ratio
// 1.0); the two ratios must be strictly ordered.
#[test]
fn harness_resource_depletion_a3_ratio_reflects_depletion() {
    let mut res = empty_resources();
    // Two FOOD sources in one snapshot, same cap 45: one full, one depleted.
    res.set_food_tile(10, 10, 45); // FULL (amount == max)
    res.set_food_source_max(10, 10, 45);
    res.set_food_tile(2, 5, 10); // DEPLETED (amount << max)
    res.set_food_source_max(2, 5, 45);

    let rows = collect_resource_snapshot(&res);
    let full = rows.iter().find(|r| r.x == 10 && r.y == 10).expect("A3: full row");
    let dep = rows.iter().find(|r| r.x == 2 && r.y == 5).expect("A3: depleted row");
    let full_ratio = f64::from(full.amount) / f64::from(full.max);
    let dep_ratio = f64::from(dep.amount) / f64::from(dep.max);

    // Type A — full source ratio is exactly 1.0.
    assert_eq!(full.amount, full.max, "A3: full source amount == max");
    assert!((full_ratio - 1.0).abs() < 1e-9, "A3: full source ratio == 1.0");
    // Type A — depleted source amount < max and 0 < ratio < 1 (≈ 10/45).
    assert!(dep.amount < dep.max, "A3: depleted source amount < max");
    assert!(dep_ratio > 0.0 && dep_ratio < 1.0, "A3: 0 < depleted ratio < 1 (got {dep_ratio})");
    assert!((dep_ratio - (10.0 / 45.0)).abs() < 1e-9, "A3: depleted ratio == 10/45");
    // Type A — strict ordering: depleted < full.
    assert!(dep_ratio < full_ratio, "A3: depleted_ratio < full_ratio");
    println!("[viz-A A3] full={full_ratio} depleted={dep_ratio} ordered ✓");
}

// ─── A4: unregistered (infinite-sentinel 255) source → max == amount ────────
// Type A — documented default path. An unregistered/infinite source has no
// meaningful capacity, so max defaults to the live amount and the marker reads
// full (1.0). This is the path the 12 shared harnesses seed via
// RESOURCE_SOURCE_INFINITE. (The .max(1) floor is a no-op here — 255 >= 1 — and
// is exercised separately in A6.)
#[test]
fn harness_resource_depletion_a4_unregistered_source_defaults_max_to_amount() {
    let mut res = empty_resources();
    // Infinite-sentinel amount with NO source_max registered.
    res.set_food_tile(5, 5, RESOURCE_SOURCE_INFINITE);

    let rows = collect_resource_snapshot(&res);
    let row = rows
        .iter()
        .find(|r| r.kind == 0 && r.x == 5 && r.y == 5)
        .expect("A4: food row (5,5)");

    // Type A — max defaults to the live amount (255); ratio exactly 1.0.
    assert_eq!(row.amount, 255, "A4: amount is the infinite-sentinel 255");
    assert_eq!(row.max, row.amount, "A4: unregistered source max defaults to amount");
    assert_eq!(row.max, 255, "A4: max == amount == 255");
    let ratio = f64::from(row.amount) / f64::from(row.max);
    assert!((ratio - 1.0).abs() < 1e-9, "A4: ratio == 1.0");
    assert!(ratio <= 1.0, "A4: ratio never > 1 (no division by zero)");
    println!("[viz-A A4] sentinel 255 → max==amount==255, ratio 1.0 ✓");
}

// ─── A5: (xs, ys, kinds) split contract preserved over the 6-row layout ─────
// Type D — regression guard for the locked α0 (xs, ys, kinds) contract. Adding
// amount/max must NOT perturb marshalling, the (kind, x, y) sort, or kind
// encoding. The non-decreasing-kinds check is a verified invariant of the α0
// sort, not a new HashMap-iteration assumption.
#[test]
fn harness_resource_depletion_a5_xs_ys_kinds_split_contract_preserved() {
    let mut res = empty_resources();
    seed_six_row_multi_kind(&mut res);

    let rows = collect_resource_snapshot(&res);
    let (xs, ys, kinds) = resource_rows_split(&rows);

    // Type D — lengths all == rows.len() == 6.
    assert_eq!(rows.len(), 6, "A5: 6-row multi-kind layout");
    assert_eq!(xs.len(), rows.len(), "A5: xs.len() == rows.len()");
    assert_eq!(ys.len(), rows.len(), "A5: ys.len() == rows.len()");
    assert_eq!(kinds.len(), rows.len(), "A5: kinds.len() == rows.len()");
    // Type D — each split element mirrors its row field; kind domain {0,1,2}.
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(xs[i], r.x as i32, "A5: xs[{i}] mirrors row.x");
        assert_eq!(ys[i], r.y as i32, "A5: ys[{i}] mirrors row.y");
        assert_eq!(kinds[i], i32::from(r.kind), "A5: kinds[{i}] mirrors row.kind");
        assert!((0..=2).contains(&kinds[i]), "A5: kind {} ∈ {{0,1,2}}", kinds[i]);
    }
    // Type D — kinds sequence is non-decreasing (the locked α0 (kind, x, y) sort).
    for i in 1..kinds.len() {
        assert!(
            kinds[i - 1] <= kinds[i],
            "A5: kinds must be non-decreasing (locked α0 sort): {} > {}",
            kinds[i - 1],
            kinds[i]
        );
    }
    println!("[viz-A A5] (xs,ys,kinds) length + field correspondence + non-decreasing kinds ✓");
}

// ─── A6: zero-amount sleep row exercises the .max(1) floor → ratio 0.0 ──────
// Type A — the boundary input that forces the .max(1) floor to do real work
// (unlike A4's 255 no-op). A sleep tile at (1,1) with amount 0 and NO
// sleep_source_max: pre-floor capacity resolves to 0, the floor raises it to 1,
// so ratio is 0/1 == 0.0 — never a division by zero. Direct-inserted because
// set_sleep_tile(_,_,0) would REMOVE the entry.
#[test]
fn harness_resource_depletion_a6_zero_amount_row_exercises_floor() {
    let mut res = empty_resources();
    // Direct insert — bypass set_sleep_tile's amount==0 removal. NO source_max.
    res.sleep_tiles.insert((1, 1), 0);

    let rows = collect_resource_snapshot(&res);
    let row = rows
        .iter()
        .find(|r| r.kind == 2 && r.x == 1 && r.y == 1)
        .expect("A6: a 0-amount sleep tile in the map must still surface a row");

    // Type A — floor fires: 0.max(1) == 1; amount stays 0; ratio finite 0.0.
    assert_eq!(row.max, 1, "A6: unregistered 0-amount max == 0.max(1) == 1");
    assert!(row.max >= 1, "A6: max floored at >= 1 (got {})", row.max);
    assert_ne!(row.max, 0, "A6: max must NOT be 0");
    assert_eq!(row.amount, 0, "A6: amount stays 0");
    let ratio = f64::from(row.amount) / f64::from(row.max);
    assert!(ratio.is_finite(), "A6: ratio must be finite (no division by zero)");
    assert!((ratio - 0.0).abs() < 1e-9, "A6: ratio == 0/1 == 0.0");
    println!("[viz-A A6] sleep amount 0, max floored to 1, ratio 0.0 (no div-by-zero) ✓");
}

// ─── A7: over-capacity (50/45) + explicit zero-cap (50/0) → amount <= max ───
// Type A — a depletion display is bounded at "full": ratio > 1.0 is meaningless
// and overshoots the renderer's scale/alpha lerp. Two pathological inputs in
// INDEPENDENT snapshots: (7a) live counter exceeds the registered cap (regen
// overshoot); (7b) an explicit source_max of 0 (capacity lowered after seeding)
// floored to >= 1. Both must be clamped so amount <= max.
#[test]
fn harness_resource_depletion_a7_over_capacity_and_explicit_zero_cap_bounded() {
    // (7a) OVERSHOOT — live counter exceeds the registered cap.
    let mut res_a = empty_resources();
    res_a.set_food_tile(6, 6, 50);
    res_a.set_food_source_max(6, 6, 45);
    let rows_a = collect_resource_snapshot(&res_a);
    let over = rows_a.iter().find(|r| r.x == 6 && r.y == 6).expect("A7a: overshoot row");
    assert!(
        over.amount <= over.max,
        "A7a: overshoot amount {} must be <= max {}",
        over.amount,
        over.max
    );
    let over_ratio = f64::from(over.amount) / f64::from(over.max);
    assert!(over_ratio <= 1.0, "A7a: overshoot displayed ratio <= 1.0 (got {over_ratio})");

    // (7b) EXPLICIT ZERO CAP — source_max registered as 0 (distinct from absent).
    // Direct insert because set_food_source_max(_,_,0) would REMOVE the entry;
    // we want the explicit-0 → .max(1) floor path.
    let mut res_b = empty_resources();
    res_b.set_food_tile(7, 7, 50);
    res_b.food_source_max.insert((7, 7), 0);
    let rows_b = collect_resource_snapshot(&res_b);
    let zcap = rows_b.iter().find(|r| r.x == 7 && r.y == 7).expect("A7b: zero-cap row");
    assert!(zcap.max >= 1, "A7b: explicit-0 max floored to >= 1 (got {})", zcap.max);
    let zcap_ratio = f64::from(zcap.amount) / f64::from(zcap.max);
    assert!(
        zcap_ratio <= 1.0,
        "A7b: zero-cap displayed ratio <= 1.0 (no overflow, no division by zero; got {zcap_ratio})"
    );
    println!("[viz-A A7] overshoot {over_ratio} & zero-cap {zcap_ratio} both clamped <= 1.0 ✓");
}

// ─── A8: multi-kind index lockstep correspondence ───────────────────────────
// Type A — closes the cross-array index-scramble vector: a Generator could keep
// all arrays length-aligned (A1 green) yet emit amounts/maxes in a different row
// order than xs/ys/kinds, pairing the right ratio with the wrong coordinate.
// This matches each amount/max back to its tile BY COORDINATE, reading the LIVE
// backend maps directly (non-circular), spanning all three kinds.
#[test]
fn harness_resource_depletion_a8_multi_kind_index_lockstep_correspondence() {
    let mut res = empty_resources();
    seed_six_row_multi_kind(&mut res);

    let rows = collect_resource_snapshot(&res);
    let (xs, ys, kinds) = resource_rows_split(&rows);
    let (amounts, maxes) = resource_rows_amounts(&rows);

    assert_eq!(rows.len(), 6, "A8: 6-row multi-kind layout");
    for i in 0..rows.len() {
        let (x, y, kind) = (xs[i] as u32, ys[i] as u32, kinds[i]);
        // Resolve the identity (xs[i], ys[i], kinds[i]) against the LIVE backend
        // maps — independent of the parallel arrays under test. All seeded pairs
        // have amount <= max and a registered source_max, so the collector emits
        // the live counter and the registered max verbatim.
        let (live, smax) = match kind {
            0 => (
                *res.food_tiles.get(&(x, y)).expect("A8: food tile present at identity"),
                *res.food_source_max.get(&(x, y)).expect("A8: food source_max present"),
            ),
            1 => (
                *res.water_tiles.get(&(x, y)).expect("A8: water tile present at identity"),
                *res.water_source_max.get(&(x, y)).expect("A8: water source_max present"),
            ),
            2 => (
                *res.sleep_tiles.get(&(x, y)).expect("A8: sleep tile present at identity"),
                *res.sleep_source_max.get(&(x, y)).expect("A8: sleep source_max present"),
            ),
            other => panic!("A8: kind {other} ∉ {{0,1,2}} at index {i}"),
        };
        assert_eq!(
            amounts[i],
            i32::from(live),
            "A8: amounts[{i}] must equal the live counter of ({x},{y}) kind {kind}"
        );
        assert_eq!(
            maxes[i],
            i32::from(smax),
            "A8: maxes[{i}] must equal the reported max of ({x},{y}) kind {kind}"
        );
    }
    println!("[viz-A A8] all 6 indices: amounts[i]/maxes[i] match (xs[i],ys[i],kinds[i]) by coordinate ✓");
}

// ─── A9: production scene registers finite capacities ───────────────────────
// Type A — anti-circularity guard. A1–A8 hand-set the registry, so a production
// build that never populates *_source_max would pass them all yet show every
// in-game marker permanently full. Run the REAL production engine and prove the
// live snapshot carries >= 1 row with a registered finite capacity.
#[test]
fn harness_resource_depletion_a9_production_scene_registers_finite_capacities() {
    let mut engine = init_production_engine();
    for _ in 0..2000 {
        engine.tick();
    }
    let rows = collect_resource_snapshot(&engine.resources);
    // A finite, non-full registered capacity: max is neither the infinite
    // sentinel nor below the floor.
    let finite = rows
        .iter()
        .filter(|r| r.max != RESOURCE_SOURCE_INFINITE && r.max >= 1)
        .count();

    // Type A — existential: at least one finite, depletable source in the scene.
    assert!(
        finite >= 1,
        "A9: production snapshot must carry >= 1 row with a registered finite \
         capacity (max != {} sentinel and max >= 1); rows={}, finite={}",
        RESOURCE_SOURCE_INFINITE,
        rows.len(),
        finite
    );
    println!("[viz-A A9] live finite-capacity source rows = {finite} (>= 1) ✓");
}

// ─── A10: live depletion surfaces in the snapshot ───────────────────────────
// Type A — second anti-circularity guard: real gameplay consumption flows into
// the visualisation data. Disjunctive (partial-depletion row OR a reduced source
// count) because a source consumed to exactly 0 is removed from the *_tiles map
// (absent, not a zero row). Existential — defers rates/counts to the balance
// harness.
#[test]
fn harness_resource_depletion_a10_live_depletion_surfaces() {
    let mut engine = init_production_engine();
    // Tick-2000 baseline (matches A9's setup) for the source-count comparison.
    for _ in 0..2000 {
        engine.tick();
    }
    let count_2000 = collect_resource_snapshot(&engine.resources).len();
    // Continue to 4380 ticks (≈ 1 year) of real consumption.
    for _ in 2000..4380 {
        engine.tick();
    }
    let rows = collect_resource_snapshot(&engine.resources);
    let partial = rows.iter().any(|r| r.amount < r.max);
    let reaped = rows.len() < count_2000;

    // Type A — at least one of the two depletion signals must be observable.
    assert!(
        partial || reaped,
        "A10: after 4380 ticks depletion must surface — either a partially \
         depleted row (amount < max) OR fewer present source rows than at tick \
         2000 (count_2000={count_2000}, count_4380={}, partial={partial}, reaped={reaped})",
        rows.len()
    );
    println!(
        "[viz-A A10] depletion observable: partial={partial}, reaped={reaped} \
         (count 2000→4380: {count_2000}→{}) ✓",
        rows.len()
    );
}
