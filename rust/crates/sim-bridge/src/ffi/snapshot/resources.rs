//! Resource-substrate snapshot collection and Godot dictionary marshalling.

use godot::prelude::*;
use sim_engine::SimResources;

// ────────────────────────────────────────────────────────────────────────
// V7 Section 16-α0: Resource-substrate snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the resource-substrate snapshot returned by
/// [`collect_resource_snapshot`]. `kind` encoding: `0 = Food`, `1 = Water`,
/// `2 = Sleep` (matches the `TargetKind` discriminant order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSnapshotRow {
    /// Tile-x coordinate of the source tile.
    pub x: u32,
    /// Tile-y coordinate of the source tile.
    pub y: u32,
    /// Resource kind: `0 = Food`, `1 = Water`, `2 = Sleep`, `3 = Deadwood`
    /// (Pass 5; registered spots persist at amount 0 between pickup and regrowth).
    pub kind: u8,
    /// Current stored amount on the tile (the `*_tiles` counter), CLAMPED to
    /// [`max`](Self::max). Drives the renderer's depletion/regen visual (marker
    /// scale + alpha = amount/max). A finite source decreases as agents consume
    /// and recovers via regen. Legacy scenes omit removed tiles; game rules
    /// retain registered food sources at zero so depleted bushes remain visible.
    /// Clamped to
    /// `max` so the displayed ratio never exceeds `1.0` (regen overshoot or a
    /// zero-cap source cannot make the marker read as more than full).
    pub amount: u8,
    /// Original source capacity (the `*_source_max` registry). The denominator
    /// for the depletion ratio. Guaranteed `>= 1` (a tile with no registered
    /// ceiling — e.g. an `RESOURCE_SOURCE_INFINITE` tile — reports `max ==
    /// amount`, so its ratio is `1.0` and the marker stays full).
    pub max: u8,
}

/// Pure-Rust collector over the three sparse tile maps on [`SimResources`].
///
/// **Sorted by `(kind, x, y)`** — `HashMap` iteration order is unspecified,
/// so the explicit sort is what makes the snapshot deterministic for the
/// renderer markers and the determinism harness assertion. Sim-test
/// exercises this collector directly (no Godot runtime required).
pub fn collect_resource_snapshot(resources: &SimResources) -> Vec<ResourceSnapshotRow> {
    let mut rows: Vec<ResourceSnapshotRow> = Vec::with_capacity(
        resources.food_tiles.len() + resources.water_tiles.len() + resources.sleep_tiles.len(),
    );
    // `amount` = the live tile counter (`*_tiles` value); `max` = the source
    // capacity (`*_source_max`), defaulting to `amount` (ratio 1.0) for any
    // tile with no registered ceiling (e.g. an infinite-sentinel source), and
    // floored at 1 so the renderer's `amount / max` is never a divide-by-zero.
    //
    // `amount` is finally CLAMPED to `max` (`amount.min(max)`): a depletion
    // display is bounded at "full", so the snapshot must never emit `amount >
    // max` (ratio > 1.0) — which the renderer's scale/alpha lerp would overshoot
    // beyond the full marker. Two pathological inputs need this clamp: regen
    // overshoot (live counter momentarily exceeds the registered cap) and an
    // explicitly-registered `max == 0` (floored to 1, so a raw amount would read
    // as e.g. 5/1 = 5.0). The clamp is read-only on the SNAPSHOT — the backend
    // tile counters are untouched (pure visualisation, no logic change).
    for (&(x, y), &amount) in resources.food_tiles.iter() {
        let max = resources
            .food_source_max
            .get(&(x, y))
            .copied()
            .unwrap_or(amount)
            .max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow {
            x,
            y,
            kind: 0,
            amount,
            max,
        });
    }
    if resources.rules.society.shelter_planning.is_some() {
        // The bush persists between harvest and regrowth even when its live
        // food counter is absent. The ceiling registry owns its existence.
        for (&(x, y), &capacity) in &resources.food_source_max {
            if !resources.food_tiles.contains_key(&(x, y)) {
                rows.push(ResourceSnapshotRow {
                    x,
                    y,
                    kind: 0,
                    amount: 0,
                    max: capacity.max(1),
                });
            }
        }
    }
    for (&(x, y), &amount) in resources.water_tiles.iter() {
        let max = resources
            .water_source_max
            .get(&(x, y))
            .copied()
            .unwrap_or(amount)
            .max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow {
            x,
            y,
            kind: 1,
            amount,
            max,
        });
    }
    for (&(x, y), &amount) in resources.sleep_tiles.iter() {
        let max = resources
            .sleep_source_max
            .get(&(x, y))
            .copied()
            .unwrap_or(amount)
            .max(1);
        let amount = amount.min(max);
        rows.push(ResourceSnapshotRow {
            x,
            y,
            kind: 2,
            amount,
            max,
        });
    }
    // Revitalize Pass 5 — every registered deadwood spot, at amount 0 while
    // emptied, so the pile stays visible between pickup and regrowth.
    for (&(x, y), &capacity) in &resources.wood_source_max {
        let max = capacity.max(1);
        let amount = resources
            .wood_tiles
            .get(&(x, y))
            .copied()
            .unwrap_or(0)
            .min(max);
        rows.push(ResourceSnapshotRow {
            x,
            y,
            kind: 3,
            amount,
            max,
        });
    }
    rows.sort_by_key(|r| (r.kind, r.x, r.y));
    rows
}

/// Pure-Rust marshalling split — the SINGLE source of the `(xs, ys, kinds)`
/// integer arrays. Extracted so the harness can verify the exact integers
/// the FFI emits WITHOUT a Godot runtime (`VarDictionary` /
/// `PackedInt32Array` require Godot). [`resource_rows_to_dict`] MUST build
/// its `PackedInt32Array`s from this — no duplicate marshalling logic.
pub fn resource_rows_split(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    let mut kinds = Vec::with_capacity(rows.len());
    for r in rows {
        xs.push(r.x as i32);
        ys.push(r.y as i32);
        kinds.push(r.kind as i32);
    }
    (xs, ys, kinds)
}

/// Companion to [`resource_rows_split`] for the depletion visual: the parallel
/// `(amounts, maxes)` integer arrays, in the SAME row order. Kept separate from
/// `resource_rows_split` so the locked `(xs, ys, kinds)` contract (and the
/// harness assertions on it) is untouched while [`resource_rows_to_dict`] gains
/// the `amounts` / `maxes` keys. `maxes[i] >= 1` (floored in
/// [`collect_resource_snapshot`]).
pub fn resource_rows_amounts(rows: &[ResourceSnapshotRow]) -> (Vec<i32>, Vec<i32>) {
    let mut amounts = Vec::with_capacity(rows.len());
    let mut maxes = Vec::with_capacity(rows.len());
    for r in rows {
        amounts.push(r.amount as i32);
        maxes.push(r.max as i32);
    }
    (amounts, maxes)
}

/// Marshal a [`ResourceSnapshotRow`] slice into the FFI dictionary shape
/// consumed by `WorldRenderer._render_resource_sources()`. Three parallel
/// `PackedInt32Array`s (`xs`, `ys`, `kinds`), lengths always equal to
/// `rows.len()`. Built solely from [`resource_rows_split`] (the single
/// marshalling path the harness tests).
pub(in crate::ffi) fn resource_rows_to_dict(rows: &[ResourceSnapshotRow]) -> VarDictionary {
    let (xv, yv, kv) = resource_rows_split(rows);
    let (av, mv) = resource_rows_amounts(rows);
    let n = rows.len();
    let mut xs = PackedInt32Array::new();
    let mut ys = PackedInt32Array::new();
    let mut kinds = PackedInt32Array::new();
    let mut amounts = PackedInt32Array::new();
    let mut maxes = PackedInt32Array::new();
    xs.resize(n);
    ys.resize(n);
    kinds.resize(n);
    amounts.resize(n);
    maxes.resize(n);
    for i in 0..n {
        xs[i] = xv[i];
        ys[i] = yv[i];
        kinds[i] = kv[i];
        amounts[i] = av[i];
        maxes[i] = mv[i];
    }
    let mut dict = VarDictionary::new();
    dict.set("xs", xs);
    dict.set("ys", ys);
    dict.set("kinds", kinds);
    dict.set("amounts", amounts);
    dict.set("maxes", maxes);
    dict
}

#[cfg(test)]
mod depleted_food_snapshot_tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn game_snapshot_keeps_depleted_bush_and_sorts_without_duplicates() {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::baseline(0);
        engine.resources.set_food_source_max(8, 4, 6);
        engine.resources.set_food_source_max(2, 3, 6);
        engine.resources.set_food_tile(8, 4, 3);
        let rows = collect_resource_snapshot(&engine.resources);
        assert_eq!(
            rows,
            vec![
                ResourceSnapshotRow {
                    x: 2,
                    y: 3,
                    kind: 0,
                    amount: 0,
                    max: 6
                },
                ResourceSnapshotRow {
                    x: 8,
                    y: 4,
                    kind: 0,
                    amount: 3,
                    max: 6
                },
            ]
        );
        assert!(!engine.resources.food_tiles.contains_key(&(2, 3)));
    }

    #[test]
    fn legacy_snapshot_omits_source_without_live_food() {
        let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
        engine.resources.set_food_source_max(2, 3, 6);
        assert!(collect_resource_snapshot(&engine.resources).is_empty());
    }
}
