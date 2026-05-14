# P3-γ (γ-1) — FFI Surface for Causal Chain Queries (V7 Phase 3-γ)

## Implementation Intent

P3-α (commit bb925bd1) recorded *what* happened on each influence tile;
P3-β (commit fa6652a6) linked those events into walkable lineage chains
via `EventId` + `Option<EventId>` parents. P3-γ now exposes that
substrate to the upcoming "왜?" UI through two read-only sim-bridge
`#[func]` methods. γ-1 is the **FFI surface only** — γ-2 will build the
Godot panel that consumes it.

The two new methods:

```rust
WorldSimNode::get_tile_causal_history(x: i32, y: i32) -> Array<Dictionary>
WorldSimNode::get_event_chain(x: i32, y: i32, event_id: i64) -> Array<Dictionary>
```

Each `Dictionary` entry mirrors a single `CausalEvent` flattened into a
tagged record. The `kind` field discriminates between the three variants
(`"building_placed"`, `"stamp_dirty"`, `"influence_changed"`); variant-
specific keys are present only for their owning variant. `parent: None`
is encoded as the sentinel `-1` so GDScript can detect chain roots and
evicted ancestors uniformly.

## Locked facts (do NOT change)

- **P3γ-S1**: Single-scope dispatch — γ-1 covers the sim-bridge FFI
  surface and harness coverage only. γ-2 (panel UI consumption) is a
  separate decision cycle, NOT in this land.
- **P3γ-F1**: Dictionary schema (Variant-side):
  - `kind: String` — one of `"building_placed"`, `"stamp_dirty"`,
    `"influence_changed"`.
  - `id: i64` — monotonic `EventId` cast to `i64`.
  - `parent: i64` — `-1` for `None`, otherwise the parent `EventId`.
  - `tick: i64` — simulation tick.
  - `channel: i32` — present for `stamp_dirty` and `influence_changed`
    only. Index matches `InfluenceChannel` enum ordering.
  - `position: Vector2i` — `(x, y)` for `building_placed` and
    `influence_changed`; absent for `stamp_dirty`.
  - `radius: i32` — present for `building_placed` only.
  - `region: Vector4i` — `(min_x, min_y, max_x, max_y)` for
    `stamp_dirty` only.
  - `old: f32`, `new: f32` — present for `influence_changed` only.
- **P3γ-F2**: Boundary semantics — out-of-bounds `(x, y)` returns an
  empty `Array`. Negative `event_id` returns an empty `Array`. Unknown
  `event_id` on an in-bounds tile returns an empty `Array`. None of
  these conditions throw or panic.
- **P3γ-U1**: Bridge Identity Contract — every `#[func]` body consists
  of a thin forwarding call followed by `Variant` marshalling. The
  canonical Rust-only implementations live in the pure-Rust pub fns
  `collect_tile_causal_history` and `collect_event_chain` (signature:
  `&SimResources, tile_idx: u32 [, event_id: EventId] -> Vec<CausalEventView>`).
  Sim-test imports both directly and exercises every assertion in this
  land without a Godot runtime.
- **P3γ-U2**: `CausalEventView` is the pure-Rust mirror of the dictionary
  schema. Fields use `Option<…>` for variant-specific values; the
  marshalling layer (`event_view_to_dict`) renders `Option::None` as a
  missing dictionary key or — for `parent` — as the `-1` sentinel.

Other locked facts:

- `EventId` remains `u64`. The FFI boundary uses `i64` because Godot's
  `Variant` integer is signed 64-bit; the conversion is checked at the
  marshalling layer (negative inputs rejected; non-negative inputs
  cast through `as EventId`).
- The ring size (`TILE_CAUSAL_RING_SIZE = 8`) and sparse storage are
  unchanged — γ-1 reads through `CausalLogStorage::get` and
  `CausalLogStorage::trace_parents`, neither of which is modified.
- `WorldSimNode` keeps its existing 3 `#[func]` methods unchanged
  (`get_influence_overlay`, `get_tile_detail`, `on_building_placed`).
  γ-1 adds 2 new methods adjacent to them in the same `#[godot_api]`
  block.

## What to build

1. **`rust/crates/sim-bridge/src/ffi/world_node.rs`** — extend the
   `#[godot_api]` block on `WorldSimNode` with two new `#[func]` methods
   (`get_tile_causal_history`, `get_event_chain`). Add the pure-Rust
   collectors `collect_tile_causal_history`, `collect_event_chain`, the
   view type `CausalEventView`, and the marshalling helpers
   (`event_view_to_dict`, `event_views_to_variant_array`,
   `tile_idx_from_coords`, `dirty_region_bounds`).
2. **`rust/crates/sim-bridge/src/ffi/mod.rs`** — re-export
   `CausalEventView`, `collect_tile_causal_history`, `collect_event_chain`
   alongside the existing `enqueue_building_placed` / `WorldSimNode`
   re-exports.
3. **`rust/crates/sim-bridge/src/lib.rs`** — update the crate doc-comment
   to mention 5 FFI methods (3 T7.7.B + 2 γ-1) and extend the Bridge
   Identity Contract paragraph for the two new collectors.
4. **Harness test** at
   `rust/crates/sim-test/tests/harness_p3_gamma_1_ffi.rs` covering the
   11 assertions listed under Verification.

## Locale

No new locale keys. P3-γ-1 is a pure backend FFI surface — γ-2 (the
"왜?" panel) will introduce its own locale keys when it consumes the
chain.

## Verification

`cargo test -p sim-test --test harness_p3_gamma_1_ffi -- --nocapture`
must run **11 tests** (all Type A — threshold assertions):

1. `harness_p3_gamma_1_enumerator_empty_on_unstamped_tile` — an
   untouched tile returns an empty `Vec<CausalEventView>`.
2. `harness_p3_gamma_1_enumerator_returns_all_events_after_building` —
   one full Phase 2 tick produces 13 pushes; the ring caps retention
   at 8; the collector returns exactly that count with strictly
   monotonic ids.
3. `harness_p3_gamma_1_enumerator_dict_schema_building_placed` —
   BSS-only engine; the `building_placed` view exposes `kind`,
   `parent = None`, `position`, `radius`, and `None` for channel /
   region / old / new.
4. `harness_p3_gamma_1_enumerator_dict_schema_stamp_dirty` — BSS-only
   engine; a `stamp_dirty` view exposes `kind`, a channel index, a
   region matching the Chebyshev box, `parent = Some(building_id)`,
   `None` for position / radius / old / new.
5. `harness_p3_gamma_1_enumerator_dict_schema_influence_changed` —
   full Phase 2 engine; an `influence_changed` view exposes `kind`,
   a channel index, the centre `position`, `Some(old)`, `Some(new)`,
   `Some(parent)`, and `None` for radius / region.
6. `harness_p3_gamma_1_chain_returns_three_events` — hand-built 3-event
   chain on a single tile; `collect_event_chain` returns
   `[InfluenceChanged, StampDirty, BuildingPlaced]` with the root
   reporting `parent == None`.
7. `harness_p3_gamma_1_chain_root_event_returns_single` — walking from
   a root `BuildingPlaced` yields a 1-element chain.
8. `harness_p3_gamma_1_chain_invalid_event_id_returns_empty` —
   walking from an unknown id returns an empty `Vec`.
9. `harness_p3_gamma_1_cross_channel_chain_isolation` — full Phase 2
   engine; the Noise `InfluenceChanged`'s chain hops to a Noise
   `StampDirty`; the Danger chain hops to a Danger `StampDirty`; the
   two parent ids differ.
10. `harness_p3_gamma_1_parent_none_serialized_as_minus_one` — a root
    `BuildingPlaced` carries `parent: None` through both
    `collect_tile_causal_history` and `collect_event_chain`. Mirroring
    the dictionary encoding (`parent.map(|p| p as i64).unwrap_or(-1)`)
    yields `-1`; a present parent of `7u64` mirrors to `7i64`.
11. `harness_p3_gamma_1_collectors_are_publicly_reexported` — compile-
    time evidence that `collect_tile_causal_history`,
    `collect_event_chain`, and `CausalEventView` are reachable through
    `sim_bridge::ffi::…`.

In addition, the full gate must pass:

- `cargo test --workspace` — 0 failures.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo build -p sim-bridge` — sim-bridge links cleanly with the new
  `Array<Dictionary>` return types via the godot 0.5 crate.

All pre-existing P3-α (10 assertions), P3-β (8 assertions), and
T7.10.A-F wiring tests must remain green; regression is intrinsic to
`cargo test --workspace`.

## Lane

`--quick` (sim-bridge + sim-test edits only — no sim-core, sim-systems,
or sim-engine changes; the influence-grid surface observed by the
renderer is unchanged; sim-bridge is the cold-tier FFI layer per
governance v3.3.8 §1).

VLM visual verify is informational only — γ-1 introduces no rendering
changes (no Godot runtime invoked in this land).

## In-game verification

None for this land. γ-2 will introduce the "왜?" panel that consumes
`get_tile_causal_history` and `get_event_chain` via GDScript. γ-1
supplies the FFI substrate only; the public Godot surface gains two new
methods but no UI surfaces them yet. VLM `no-godot-scope` auto-credit
applies — confirmed by the absence of `.tscn`, `.gd`, or `.gdshader`
edits in this land.

## Phase disclosure

V7 Phase 3-γ (γ-1). Extends P3-β (commit fa6652a6) by exposing
`CausalLogStorage::get` and `CausalLogStorage::trace_parents` over the
FFI boundary. γ-1 is the read-only substrate; γ-2 (separate decision
cycle) will build the Godot panel UI that walks the chain and renders
`[child → parent → root]` lineage with "cause evicted" gracefully
handled at the array terminator. No changes to the simulation hot path,
no new ECS components, no new locale keys.

## Out of scope (do NOT touch in this land)

- The "왜?" panel UI (γ-2 — separate land).
- New locale keys (γ-2 will own them).
- `Array<…>` mutation getters (γ-1 is read-only; γ-2 may add filter /
  cursor helpers if needed).
- Agent-level causal events (Week 7-8 Agent Core).
- Forward traversal (child lookup) — backward walk only; γ-1 mirrors
  P3-β's contract.
- Combat / decision / death / trauma variants on `CausalEvent` (later
  phases extend the enum without changing FFI semantics).
- Performance optimisation (FxHashMap, intrusive ring, etc.) —
  premature; γ-1 inherits P3-α's sparse footprint.
