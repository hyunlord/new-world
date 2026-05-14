# P3-β — Causal Chain Link Tracking (V7 Phase 3-β)

## Implementation Intent

Extend P3-α's tile-level causal log so every recorded event carries a
monotonically allocated `EventId` and an `Option<EventId>` parent link.
P3-α answered "what happened on this tile"; P3-β answers "why was each
event emitted" by chaining `BuildingPlaced → StampDirty → InfluenceChanged`
through explicit parent ids that the upcoming "왜?" UI (Week 6) can walk
backward to reconstruct the lineage of every influence change.

The chain semantic (P3β-3):

```text
BuildingPlaced  { id, parent: None }
  → StampDirty  { id, parent: Some(building_id) }          (× 6 channels)
    → InfluenceChanged { id, parent: Some(stamp_id) }       (per channel,
                                                             matched by
                                                             channel)
```

Per-channel parent disambiguation in IUS resolves the matching
`StampDirty` via `CausalLogStorage::find_recent_stamp_dirty(tile, ch)`,
so each `InfluenceChanged` points at the *same-channel* stamp — not the
most recent stamp of any channel.

## Locked facts (do NOT change)

- **P3β-1**: EventId mechanism — `pub type EventId = u64`. Global counter
  via `AtomicU64` field `next_event_id` on `SimResources`. Allocation
  through `SimResources::issue_event_id(&self) -> EventId` using
  `fetch_add(1, Ordering::Relaxed)`. Relaxed ordering is sufficient
  because uniqueness is the only invariant and per-tick ordering is
  preserved by the priority-sorted system schedule. `next_event_id`
  initialises to `AtomicU64::new(0)` in `SimEngine::new`.
- **P3β-2**: Parent reference — every `CausalEvent` variant carries
  `id: EventId` and `parent: Option<EventId>`. `None` denotes a root
  event (only `BuildingPlaced` is a root in P3-β). The enum keeps its
  P3-α derives (`Debug + Clone + PartialEq`, NOT `Copy` — `DirtyRegion`
  blocks `Copy`).
- **P3β-3**: Chain semantic — exactly one `BuildingPlaced` per accepted
  FFI event becomes the parent of all 6 `StampDirty` records emitted by
  BSS on the same tile/tick. Each `InfluenceChanged` then references the
  same-channel `StampDirty` resolved at push time via
  `find_recent_stamp_dirty(centre_idx, channel)`. When the matching
  stamp has been evicted from the ring (FIFO 8), `parent` is `None`.
- **P3β-4**: Traversal API — `CausalLogStorage` gains two methods:
  - `find_recent_stamp_dirty(tile_idx, channel) -> Option<EventId>` —
    scans the tile's ring newest → oldest, returns the first
    `StampDirty` id whose channel matches.
  - `trace_parents(tile_idx, event_id) -> Vec<&CausalEvent>` — backward
    walk only. Returns `[child, parent, grand-parent, …]`. Terminates
    when the current event has `parent == None` (root) or when the
    referenced parent id is not present on the same tile (evicted). The
    returned chain MAY be shorter than three entries — that is the
    expected graceful-termination behaviour.

Other locked facts:

- BSS allocates `building_id` BEFORE the `BuildingPlaced` push so the
  6 subsequent stamps can reference it. Each stamp allocates its own
  id, and `parent: Some(building_id)` is stored.
- IUS allocates `influence_id` AFTER resolving the parent via
  `find_recent_stamp_dirty(centre_idx, channel)` so the id sequence
  reflects the actual push order on the tile.
- `id` accessors: `CausalEvent::id() -> EventId`,
  `CausalEvent::parent() -> Option<EventId>`,
  `CausalEvent::tick() -> u64`,
  `CausalEvent::channel() -> Option<InfluenceChannel>`
  (`None` for `BuildingPlaced`). All four are pub.
- Ring size (`TILE_CAUSAL_RING_SIZE = 8`) and sparse storage
  (`HashMap<u32, TileCausalLog>`) are UNCHANGED — P3-β adds fields, not
  storage rules.

## What to build

1. **`rust/crates/sim-core/src/causal/event.rs`** — extend each
   `CausalEvent` variant with `id: EventId` and `parent: Option<EventId>`
   fields. Add `pub type EventId = u64` at the top. Implement
   `id()`, `parent()`, `tick()`, `channel()` accessor methods.
2. **`rust/crates/sim-core/src/causal/mod.rs`** — re-export `EventId`
   alongside `CausalEvent`.
3. **`rust/crates/sim-core/src/causal/storage.rs`** — add
   `find_recent_stamp_dirty` and `trace_parents` (using `as_slice()` to
   reach `DoubleEndedIterator`, since `TileCausalLog::iter()` hides
   that capability).
4. **`rust/crates/sim-engine/src/lib.rs`** — add
   `next_event_id: AtomicU64` to `SimResources`, initialise in
   `SimEngine::new`. Provide `impl SimResources { pub fn
   issue_event_id(&self) -> EventId { ... } }`.
5. **BSS wiring**
   (`rust/crates/sim-systems/src/runtime/influence/building_stamp.rs`) —
   allocate `building_id` before the `BuildingPlaced` push, then per
   stamped channel allocate a `stamp_id` and store `parent: Some(building_id)`.
6. **IUS wiring**
   (`rust/crates/sim-systems/src/runtime/influence/update.rs`) — in
   each of the 6 channel branches (Warmth / Light / Noise / Danger /
   Spiritual / Beauty) resolve `parent` via `find_recent_stamp_dirty`,
   then allocate `influence_id` and push the record.
7. **Harness test** at
   `rust/crates/sim-test/tests/harness_p3_beta_causal_chain.rs` covering
   the 8 assertions listed under Verification.

## Locale

No new locale keys. P3-β is a pure simulation-core surface; the "왜?"
UI in Week 6 will introduce its own locale keys for the inspector when
it consumes the chain.

## Verification

`cargo test -p sim-test --test harness_p3_beta_causal_chain -- --nocapture`
must run **8 tests** (all Type A — threshold assertions):

1. `event_id_monotonic` — across a full Phase 2 tick the recorded
   slice has strictly increasing `id()` values (one full chain produces
   13 pushes; 8 retained after FIFO eviction maintain the monotonic
   order).
2. `building_placed_has_no_parent` — uses a BSS-only engine so
   `BuildingPlaced` survives in the ring. The recorded
   `BuildingPlaced` reports `parent() == None`.
3. `stamp_dirty_parent_is_building` — uses the BSS-only engine
   (7 events fit the ring of 8). Every `StampDirty.parent` matches the
   `BuildingPlaced.id` on the same tile.
4. `influence_changed_parent_is_stamp_dirty` — full Phase 2 engine,
   verifies the Noise `InfluenceChanged.parent` points at the Noise
   `StampDirty.id` (Noise stamp survives the ring under the 13-push
   pattern).
5. `chain_depth_three` — manual `CausalLogStorage::push` of a 3-event
   chain (BuildingPlaced → StampDirty → InfluenceChanged). `trace_parents`
   returns 3 entries in `[child, parent, root]` order.
6. `cross_channel_isolation` — Noise and Danger
   `InfluenceChanged.parent` values resolve to channel-matching stamps
   (NOT each other).
7. `multi_tick_chains_independent` — two BSS events at different tile
   positions yield distinct `BuildingPlaced.id` values; each tile's
   chain is self-contained.
8. `no_parent_after_eviction` — the full Phase 2 engine evicts the
   Warmth `StampDirty` (slot 1 of 13 pushes), but the Warmth
   `InfluenceChanged.parent` field still carries the original id.
   `trace_parents` returns a chain of length 1 (the influence event
   itself), proving graceful termination on evicted ancestors.

In addition, full gate must pass:

- `cargo test --workspace` — 0 failures.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.

## Lane

`--quick` (sim-core / sim-engine / sim-systems edits, but bookkeeping
only — the public influence-grid surface observed by the renderer is
unchanged). All pre-existing P3-α harness tests (10 assertions) and
T7.10.A-F wiring tests MUST continue to pass — regression guard is
intrinsic to `cargo test --workspace`.

VLM visual verify is informational only; no rendering changes.

## In-game verification

None for this land — Week 6 introduces the "왜?" UI which consumes the
P3-β chain via `trace_parents`. P3-β supplies the substrate.

## Phase disclosure

V7 Phase 3-β. Extends P3-α (commit bb925bd1) by adding parent_event_id
chain links. The 6/6 stamped-channel wiring milestone (T7.10.A-F)
completed Phase 2; P3-α opened Phase 3 with tile-level event recording;
P3-β now links those events into walkable lineage chains in preparation
for the Week 6 "왜?" UI.

## Out of scope (do NOT touch in this land)

- Agent-level causal events (Week 7-8 Agent Core).
- "왜?" UI panel / locale keys (Week 6).
- FFI exposure of `causal_log` to GDScript (Week 6 — `SimBridge` getter
  will land alongside the inspector).
- Forward traversal (child lookup) — backward walk only in P3-β.
- Combat / decision / death / trauma variants on `CausalEvent` (later
  phases extend the enum without changing chain semantics).
- Performance optimisation (FxHashMap, intrusive ring, etc.) —
  premature; sparse footprint remains well under budget.
