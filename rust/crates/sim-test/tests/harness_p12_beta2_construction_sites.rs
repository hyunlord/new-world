//! V7 Phase 12-β.2 (A3) — ConstructionSite rendering + progress alpha harness.
//!
//! Static-inspection + smoke-test harness verifying the Phase 12-β.2 A3
//! implementation:
//!   - `rust/crates/sim-bridge/src/ffi/world_node.rs` — adds
//!     `ConstructionSnapshotRow`, `collect_construction_snapshot`,
//!     `construction_rows_to_dict`, and a `#[func] get_construction_snapshot`
//!     method on `WorldSimNode`.
//!   - `rust/crates/sim-bridge/src/ffi/mod.rs` — re-exports the new symbols.
//!   - `scripts/ui/world_renderer.gd` (modified) — polls the new snapshot,
//!     maintains a keyed dictionary of Sprite2D nodes, reaps despawned sites.
//!   - Regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α, and
//!     Phase 12-β.1 invariants.
//!
//! Plan locked thresholds: see `.harness/plans/p12-beta2-a3-construction-sites/plan_final.md`.

use hecs::World;
use sim_bridge::ffi::{collect_construction_snapshot, ConstructionSnapshotRow};
use sim_core::components::{BuildingBlueprint, ConstructionSite, Position};

// ── helpers ───────────────────────────────────────────────────────────────

// ─── Assertion 6: collector_smoke_test_returns_matching_row ───────────────
#[test]
fn harness_p12_beta2_a6_collector_smoke_test_returns_matching_row() {
    // Type: A — spawn one (ConstructionSite, Position) entity with known
    // values; verify all 5 fields and Vec length on the returned row.
    let mut world = World::new();
    let blueprint = BuildingBlueprint::new(0, 1, 1, 100);
    let mut site = ConstructionSite::new(blueprint, Position { x: 7, y: 11 });
    site.progress = 42;
    let entity = world.spawn((site, Position { x: 7, y: 11 }));

    let rows: Vec<ConstructionSnapshotRow> = collect_construction_snapshot(&world);

    assert_eq!(
        rows.len(),
        1,
        "A6.1: expected exactly 1 row; got {}",
        rows.len()
    );
    let row = rows[0];
    assert_eq!(row.x, 7, "A6.2: row.x must equal 7; got {}", row.x);
    assert_eq!(row.y, 11, "A6.3: row.y must equal 11; got {}", row.y);
    assert_eq!(
        row.progress, 42,
        "A6.4: row.progress must equal 42; got {}",
        row.progress
    );
    assert_eq!(
        row.required_progress, 100,
        "A6.5: row.required_progress must equal 100; got {}",
        row.required_progress
    );
    assert_eq!(
        row.entity_bits,
        entity.to_bits().get(),
        "A6.6: row.entity_bits must equal entity.to_bits().get()"
    );
    println!("[P12-β.2 A6] smoke test — row pass-through invariants hold ✓");
}

// ─── Assertion 7: empty_world_returns_empty_vec ───────────────────────────
#[test]
fn harness_p12_beta2_a7_empty_world_returns_empty_vec() {
    // Type: A — empty world → Vec length == 0 (not panic).
    let world = World::new();
    let rows = collect_construction_snapshot(&world);
    assert_eq!(
        rows.len(),
        0,
        "A7: empty world must yield an empty Vec; got {}",
        rows.len()
    );
    println!("[P12-β.2 A7] empty world returns empty Vec ✓");
}
