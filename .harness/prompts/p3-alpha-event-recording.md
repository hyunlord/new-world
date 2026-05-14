# P3-α — Cause-Effect Event Recording (V7 Week 5-6 entry)

## Implementation Intent

Adapt the A-4 V6 32-event-per-entity ring buffer to a **tile-level**
8-event sparse log. This is the first land of V7 Phase 3-α
("Cause-Effect Tracking") and powers the upcoming "왜?" UI in Week 6.

Concretely: every `BuildingPlacedEvent` that BSS accepts must leave a
causal trail on the centre tile (BuildingPlaced + 6×StampDirty), and
every dirty-region drain that IUS performs must record an
InfluenceChanged event at the region centre (6×InfluenceChanged, one
per wired channel). Memory budget is bounded by sparse storage
(only event-bearing tiles allocate a log) plus a per-tile FIFO ring of
8 events.

## Locked facts (do NOT change)

These are the four P3α-* sub-decisions ratified before implementation:

- **P3α-1**: Three event variants only — `BuildingPlaced`, `StampDirty`,
  `InfluenceChanged`. `CausalEvent` derives `Debug + Clone + PartialEq`
  (NOT `Copy` — `DirtyRegion` is not `Copy`, so the enum inherits the
  constraint).
- **P3α-2**: `TILE_CAUSAL_RING_SIZE = 8`. FIFO eviction
  (`events.remove(0)` then `events.push(event)` when full). Backed by
  `arrayvec::ArrayVec<CausalEvent, 8>` (arrayvec already in sim-core
  deps — no Cargo.toml change).
- **P3α-3**: Sparse storage via `HashMap<u32, TileCausalLog>`. The key
  matches `InfluenceGrid::idx` (`y * width + x`). Tiles that never
  observe a causal event do NOT allocate.
- **P3α-4**: Recording surface is the two influence-pipeline junctions:
  BSS pushes `BuildingPlaced` + 6×`StampDirty`; IUS pushes one
  `InfluenceChanged` per drained dirty region per wired channel, at
  the region centre. **No agent / combat / decision variants in this
  land** — Phase 3-β extends the enum without touching the ring size.

Other locked facts:

- `causal_log: CausalLogStorage` lives on `SimResources` (sibling of
  `influence_grid`, `material_registry`, etc.). Constructed via
  `CausalLogStorage::new()` inside `SimEngine::new`.
- `tile_idx = y * width + x` (`InfluenceGrid::idx`).
- `tick` is captured ONCE at the start of each system's `tick` method
  (`let tick = resources.current_tick;`) so all records emitted by
  that system on that tick share the same stamp.
- BSS records the BuildingPlaced + StampDirty entries on the **building
  centre tile** `idx(cx, cy)`, NOT on every tile in the dirty region.
- IUS records one InfluenceChanged per drained region per channel at
  the region centre — same sample point the propagation primitive uses
  as its source. **NOT per-cell** (would explode the ring).

## What to build

1. **New module** `rust/crates/sim-core/src/causal/`:
   - `mod.rs` — re-exports `CausalEvent`, `TileCausalLog`,
     `CausalLogStorage`, `TILE_CAUSAL_RING_SIZE`.
   - `event.rs` — `CausalEvent` enum (3 variants per P3α-1).
   - `ring_buffer.rs` — `TILE_CAUSAL_RING_SIZE = 8`, `TileCausalLog`
     with FIFO `push`, `len`, `is_empty`, `iter`, `as_slice`.
   - `storage.rs` — `CausalLogStorage` with `HashMap<u32, TileCausalLog>`,
     methods `new`, `push`, `get`, `get_mut`, `active_tile_count`,
     `is_empty`, `iter`, `clear`.
2. **sim-core lib re-exports**: add `pub mod causal;` plus a `pub use`
   re-export so downstream crates can `use sim_core::CausalEvent;` etc.
3. **sim-engine integration**: add `causal_log: CausalLogStorage` to
   `SimResources`; initialise in `SimEngine::new`.
4. **BSS wiring**
   (`rust/crates/sim-systems/src/runtime/influence/building_stamp.rs`):
   - Capture `let tick = resources.current_tick;` before the drain loop.
   - For every accepted (in-bounds, non-zero radius) event:
     - Compute `let centre_idx = resources.influence_grid.idx(cx, cy) as u32;`.
     - Push `CausalEvent::BuildingPlaced { position, radius, tick }`
       onto `centre_idx`.
     - For each of the 6 `STAMPED_CHANNELS`, clone the `DirtyRegion`
       into `mark_dirty` and push
       `CausalEvent::StampDirty { channel, region, tick }` onto
       `centre_idx`.
   - OOB events MUST short-circuit BEFORE any causal push.
5. **IUS wiring**
   (`rust/crates/sim-systems/src/runtime/influence/update.rs`):
   - Capture `let tick = resources.current_tick;` once at the start of
     `tick`.
   - Inside each of the six `if !X_dirty.is_empty()` branches
     (Warmth / Light / Noise / Danger / Spiritual / Beauty), after the
     `propagate_*` call returns, read
     `current[X_idx][centre_idx]` and `pending[X_idx][centre_idx]` and
     push `CausalEvent::InfluenceChanged { channel, position, old,
     new, tick }`.
   - Persistence branches (no events this tick) MUST NOT push records.
6. **Harness test** at
   `rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs`
   covering at minimum 10 assertions (see Verification below).

## Locale

No new locale keys. This land is a pure simulation-core surface — the
"왜?" UI in Week 6 will introduce its own locale keys for the inspector.

## Verification

`cargo test -p sim-test --test harness_p3_alpha_event_recording -- --nocapture`
must run **10 tests** (all Type A — threshold assertions):

1. `fresh_log_is_empty` — `active_tile_count() == 0` on a fresh engine.
2. `centre_tile_log_after_one_tick` — after one BSS+IUS tick the centre
   tile log holds exactly `TILE_CAUSAL_RING_SIZE = 8` events.
3. `fifo_eviction_retains_recent_eight` — verifies eviction order: the
   oldest retained slot is `StampDirty { Noise }`, slot 1 is
   `StampDirty { Danger }`, slots 2..=7 are six InfluenceChanged in
   IUS branch order (Warmth, Light, Noise, Danger, Spiritual, Beauty).
4. `building_placed_fields_round_trip` — every recorded event on the
   centre tile reports `tick == 0` (the tick when BSS/IUS ran).
5. `sparse_storage_single_active_tile` — exactly one tile is active in
   `causal_log`; corners and a non-centre tile report `None`.
6. `no_event_no_records` — 5 idle ticks with no events leave the log
   empty (persistence branches do NOT push).
7. `tick_stamp_matches_current_tick` — events recorded during tick 5
   carry `tick == 5`.
8. `oob_event_does_not_record` — BuildingPlacedEvent at (999, 999) on a
   64×64 grid produces zero causal records (OOB guard short-circuits
   BEFORE the causal push).
9. `multi_event_isolation` — two events at different positions in the
   same tick produce exactly two active tiles.
10. `per_tile_fifo_eviction_across_ticks` — five repeated events at the
    same centre cap the ring at 8 and all retained slots carry the
    LATEST tick (4).

In addition, full gate must pass:

- `cargo test --workspace` — 0 failures.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.

## Lane

`--quick` (sim-core/sim-engine/sim-systems changes, but the surface is
non-behavioural in the influence-grid sense: this is bookkeeping that
runs alongside the existing dispatch chain and does not perturb the
public InfluenceChannel state observed by the renderer or the existing
T7.10.A-F harness). All pre-existing harness tests (Phase 2 dispatch
shell, T7.10.A-F wiring) MUST continue to pass — regression guard is
intrinsic to `cargo test --workspace`.

VLM visual verify is informational only; no rendering changes.

## In-game verification

None for this land — Week 6 introduces the "왜?" UI which consumes
`causal_log`. This land lays the substrate.

## Phase disclosure

V7 Phase 3-α (Week 5-6 entry). Adapts the A-4 V6 32-event-per-entity
ring buffer (sim-core/A-4 v6 era) to a tile-level 8-event sparse log,
per the master direction Section 7 ("Week 5-6 Cause-Effect Tracking +
'왜?' UI"). The 6/6 stamped-channel wiring milestone (T7.10.A-F)
completed in V7 Phase 2; P3-α opens Phase 3.

## Out of scope (do NOT touch in this land)

- Agent-level causal events (Phase 3-β / Week 7-8 Agent Core).
- "왜?" UI panel / locale keys (Week 6).
- FFI exposure of `causal_log` to GDScript (Week 6 — `SimBridge`
  getter will land alongside the inspector).
- Combat / decision / death / trauma variants on `CausalEvent` (later
  phases extend the enum without changing the ring size or storage
  layout).
- Performance optimisation (HashMap → FxHashMap, intrusive ring, etc.)
  — premature; current sparse footprint is well under budget.
