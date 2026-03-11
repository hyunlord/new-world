# WS-FEAT-003 Repo Scan

## 1. How Rooms Are Detected
- Room detection lives in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/room.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/room.rs).
- `detect_rooms(grid: &TileGrid)` flood-fills orthogonally connected floor tiles and marks whether the region is enclosed.
- `assign_room_ids(grid, rooms)` writes `RoomId` values back into the shared [`TileGrid`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/tile_grid.rs).
- Runtime refresh happens in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs), where structural context changes trigger `detect_rooms` + `assign_room_ids`.

## 2. How Warmth Currently Propagates
- Warmth propagation is owned by the shared [`InfluenceGrid`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/influence_grid.rs).
- Runtime emission refresh happens in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs).
- Existing warmth emitters already include:
  - completed `campfire`
  - completed `shelter`
  - registry-backed furniture/structure influence emissions when present
- Needs consumption of Warmth influence happens in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/needs.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/needs.rs), where `ChannelId::Warmth` is sampled and converted into `NeedType::Warmth` recovery.

## 3. Whether Walls Attenuate Influence
- Yes. Wall attenuation already exists in [`InfluenceGrid`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/influence_grid.rs) through `WallBlockingMask` and path-based blocking.
- Existing runtime tests already prove blocked Warmth is lower than open Warmth.
- Structural wall state is rebuilt from shelter structures in [`runtime/influence.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs), then fed into `InfluenceGrid`.

## 4. Where Warmth Emitters Already Exist
- Fallback campfire warmth emitter: [`runtime/influence.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs)
- Fallback shelter warmth emitter: [`runtime/influence.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs)
- Registry-backed furniture/structure emissions: same runtime file, via `append_registry_emissions(...)`
- Existing direct Warmth consumer: [`runtime/needs.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/needs.rs)

## 5. Best Insertion Point For Shelter Bias Logic
- The least invasive correct insertion point is [`runtime/steering.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/steering.rs).
- Reason:
  - Food and Danger biases are already combined there.
  - The system already samples `InfluenceGrid` and turns gradients into movement velocity.
  - It can reuse existing Warmth sampling without adding a new generic framework.
  - It can add a room-aware preference layer using `TileGrid.room_id` while still remaining spatial and local.
- This keeps responsibilities aligned:
  - `runtime/influence.rs` owns emitter + room refresh
  - `runtime/needs.rs` owns need recovery/decay
  - `runtime/steering.rs` owns movement bias from local influence sampling
