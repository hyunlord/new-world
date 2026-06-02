# Fix moving settlement marker — expose a fixed formation_tile + distinct sprite

HEAD: 658bb706. sim-core + sim-systems + sim-bridge + GDScript + harness (`--full` lane).

## Section 1: Implementation Intent

**Why this exists.** On screen the settlement marker (a campfire-like disc)
drifts around, so the user asks "why is the building moving?". Buildings do NOT
move — they are bootstrap campfire sprites at fixed `BOOTSTRAP_X/LEFT/RIGHT`,
drawn once. The MARKER moves: `world_renderer._update_settlement_furniture` draws
a hearth sprite at the snapshot's `centroid_xs/ys`, and
`collect_settlement_snapshot` computes that centroid as the live MEAN of member
positions. Members wander (Brownian / Seeking / migration), so the centroid —
and the marker — shakes every tick. Worse, the marker is a `furniture/hearth`
sprite, visually identical to the `buildings/campfire` bootstrap sprite, creating
the "moving building" illusion.

**Why it was a centroid.** `Settlement` (sim-core) has no fixed-position field —
only `member_agents / member_buildings / population_stats / community_history /
founded_at`. So the FFI could only DERIVE a position from member positions (the
moving mean).

**Key enabler.** `SettlementSystem` already stores the formation tile in a
private `formation_tiles: HashMap<SettlementId,(u32,u32)>` (set at formation,
removed at dissolution), but the FFI only reads `resources.settlements`, not the
system's private fields. The fix promotes the formation tile onto the
`Settlement` struct itself so the FFI exposes it.

**Approach (fixed + distinct).** (1) Add `Settlement.formation_tile`. (2) Set it
to the formation candidate at `run_formation_scan`. (3) ADD `formation_x/y` to
the snapshot row + `formation_xs/ys` to the dict ALONGSIDE the existing centroid
(centroid is kept — it is independently unit-tested as a mean computation, and
removing it would force a large locked-test rewrite for no benefit). (4) The
GDScript marker reads `formation_xs/ys` (fixed) and uses a distinct
`gathering_marker` sprite.

**Tradeoffs.** Keeping centroid leaves it computed-but-unused-by-the-marker —
mild, but it preserves A6/A8 centroid-computation coverage and keeps the change
additive (only the sprite-path lock A9 needs re-pointing). A full centroid→
formation rename was rejected: it would re-point A2/A5/A6/A8 and destroy the
centroid-mean test coverage.

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly six files. Add a fixed formation_tile to Settlement, set it at
formation, expose it additively through the FFI, draw the marker at it with a
distinct sprite, and test fixedness. Formation logic, building render, member
tracking, and the centroid computation are NOT changed.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-core/src/components/settlement.rs` | Add `pub formation_tile: (u32, u32)` field to `Settlement` (place after `settlement_id`). Initialize it to `(0, 0)` in `new_with_id` (the 28 `new_with_id` callers are unaffected; no struct-literal construction exists). Doc the field as "tile where the settlement formed — fixed for its lifetime; the FFI marker anchor". The `Clone/Debug/PartialEq/Serialize/Deserialize` derives extend automatically; the existing serde round-trip test stays symmetric. |
| 2 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | In `run_formation_scan`, immediately after `let mut settlement = Settlement::new_with_id(new_id, tick);`, add `settlement.formation_tile = candidate;` (the same `candidate` already stored in `self.formation_tiles.insert(new_id, candidate)`). No other change — formation thresholds/logic untouched. |
| 3 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | (a) `SettlementSnapshotRow`: ADD `pub formation_x: i32` and `pub formation_y: i32` (keep the 5 existing fields incl. centroid_x/y). (b) `collect_settlement_snapshot`: for each settlement set `formation_x = settlement.formation_tile.0 as i32`, `formation_y = settlement.formation_tile.1 as i32` (centroid computation unchanged). (c) `settlement_rows_to_dict`: ADD `formation_xs` + `formation_ys` `PackedInt32Array`s and `dict.set("formation_xs", …)` / `dict.set("formation_ys", …)` (keep centroid_xs/ys keys). |
| 4 | `scripts/ui/world_renderer.gd` | In `_update_settlement_furniture`, change `var xs = snap.get("centroid_xs", …)` / `centroid_ys` to `snap.get("formation_xs", …)` / `formation_ys` (marker now anchors to the fixed formation tile). Change `const FURNITURE_SPRITE_PATH` from `res://assets/sprites/furniture/hearth/1.png` to `res://assets/sprites/buildings/gathering_marker/1.png` (distinct from the campfire building sprite). No other renderer change; the `ids`/reaper/`Z_FURNITURE` logic is unchanged. |
| 5 | `rust/crates/sim-test/tests/harness_p12_gamma_settlement_furniture.rs` | **A9 re-point (authorized):** A9.2 asserts `FURNITURE_SPRITE_PATH` references `assets/sprites/furniture/hearth/`; re-point to `assets/sprites/buildings/gathering_marker/` (const name `FURNITURE_SPRITE_PATH` and A9.1 unchanged; the marker is just a different asset). **ADD assertions (authorized, additive):** SettlementSnapshotRow declares `formation_x`+`formation_y`; the dict marshaller sets `formation_xs`+`formation_ys`; `world_renderer._update_settlement_furniture` reads `formation_xs`/`formation_ys` (not centroid). Do NOT change A1-A8, A10-A12 (A2/A5 are presence checks → still green with the added fields/keys; A6/A8 centroid-value tests stay green since centroid is kept). |
| 6 | `rust/crates/sim-test/tests/harness_settlement_marker_fixed.rs` | **New** harness, ≥6 assertions (Section 3 T6) proving the marker position is the FIXED formation tile and does NOT move when members move. |

**Scope boundary — NOT in this ticket:** settlement formation thresholds/logic
(`run_formation_scan` predicate untouched); building bootstrap render
(`BUILDING_SPRITE_PATH` campfire — untouched); centroid computation (kept,
unchanged); the `ids`/reaper/Z logic; migration/gathering/freeze; the
`get_settlement_snapshot` `#[func]` signature (only its row/dict gain fields).

## Section 3: How to Implement

**T1/T2 — fixed tile.** `formation_tile` defaults to `(0,0)` (settlements built
in unit tests via `new_with_id` that never form keep `(0,0)`; that is fine — only
formed settlements get a real tile, set in `run_formation_scan`). The candidate
IS the formation anchor; it equals `self.formation_tiles[new_id]` (kept in sync,
same value).

**T3 — additive FFI.** Mirror the existing centroid plumbing: where the row has
`centroid_x/centroid_y` add `formation_x/formation_y`; where the dict sets
`centroid_xs/centroid_ys` add `formation_xs/formation_ys`. `entity_bits` /
`settlement_id` / `member_count` unchanged. `collect_settlement_snapshot` already
takes `(world, settlements)` (from the prior fix) — read `settlement.formation_tile`
inside the existing `settlements.values()` loop.

**T6 — fixedness regression `harness_settlement_marker_fixed.rs`.** Production
scene (`SimEngine::new` → `register_default_runtime_systems` →
`bootstrap_spawn_agents` → enqueue buildings (32,32)/(24,32)/(40,32) r8 →
`BuildingStampSystem::tick`). Run until ≥1 settlement forms (~300 ticks).
Assertions:
1. ≥1 settlement formed and every formed settlement's `formation_tile != (0,0)`
   (it was set at formation).
2. snapshot exposes `formation_x/y` for each row equal to that settlement's
   `formation_tile`.
3. ★ FIXEDNESS: capture each settlement's `formation_x/y` at tick T, run ~200
   more ticks (members move — verify at least one member position changed), then
   re-capture: `formation_x/y` is IDENTICAL (the marker does not move).
4. CONTRAST: the centroid (mean of members) DID change over those 200 ticks for
   at least one settlement (proves the bug existed and formation_tile is the fix).
5. `formation_x/y` is within the world bounds and within
   `SETTLEMENT_PROXIMITY_RADIUS` of the settlement's member centroid at formation
   (sanity: the anchor is inside the cluster, not garbage).
6. determinism: two identical runs give identical `formation_x/y` for the same
   settlement_id.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | sim-core `formation_tile` field | 🔴 DIRECT | — |
| T2 | settlement_system set at formation | 🔴 DIRECT | T1 |
| T3 | world_node additive FFI | 🔴 DIRECT | T1 |
| T4 | world_renderer marker + sprite | 🟢 DISPATCH | T3 |
| T5 | p12_gamma A9 + additive asserts | 🟢 DISPATCH | T3,T4 |
| T6 | new fixedness harness | 🟢 DISPATCH | T1,T2,T3 |

T1-T3 DIRECT (shared struct/FFI). 3/6 dispatchable.

## Section 5: Localization Checklist

No new localization keys. (Sprite path + numeric fields only; no user-visible text.)

## Section 6: Verification & Notion

**Gate (no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cargo test -p sim-test harness_p12_gamma -- --nocapture 2>&1 | tail   # A9 re-pointed, A1-A8/A10-A12 green
cargo test -p sim-test harness_p10 -- --nocapture 2>&1 | tail          # formation/births untouched
cargo test -p sim-test harness_settlements_zero -- --nocapture 2>&1 | tail  # prior fix preserved
```

**Reproduction:** before — snapshot has only centroid, which moves with members;
after — `formation_x/y` is constant across ticks while members move (the new
harness asserts this).

**Visual:** `--full` Visual Verify launches Godot with the rebuilt cdylib; the
settlement marker should sit STILL at the formation tile and be visually distinct
(gathering_marker) from the campfire buildings.

**★ dylib rebuild REQUIRED** (sim-core + sim-systems + sim-bridge changed):
`cargo build -p sim-bridge` before any windowed run.

**Notion:** update the V7 progress log with the fixed-marker entry.
