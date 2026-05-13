# T7.10.B — Light channel shadowcast wiring (second-channel Phase 2 escape)

> Lane: `--quick` (sim-systems + sim-test, no FFI / scene changes)
> Scope: SINGLE channel (Light only). Warmth already wired (T7.10.A).
> Other 6 channels remain dispatch-shell.
> Governance: v3.3.16. Visual: octagonal/circular light disc at the
> stamped building's center using recursive symmetric shadowcasting.

---

## Section 1 — Implementation Intent

T7.10.A escaped the dispatch shell for the Warmth channel via `propagate_bfs`.
T7.10.B is the **second channel to escape** — Light shadowcast propagation
wired end-to-end via `propagate_shadowcast` (Adam Milazzo / Björn Bergström
recursive symmetric variant, already exported from `sim_core::influence`).

After this commit, `on_building_placed(32, 32, 12)` produces both a Warmth
disc (T7.10.A regression) **and** a Light disc (octagonal shadowcast shape,
radius 15) in their respective `current[]` buffers after one tick.

Renderer change is **deferred**: `world_renderer.gd` keeps `CHANNEL_WARMTH=0`
so the visible sprite continues to show Warmth. Light visualisation is a
later UI-only task (no harness coverage required here).

Other 6 channels stay dispatch-shell — they will be wired in T7.10.C..F.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Light enum index | `channel.rs` | `InfluenceChannel::Light as usize == 1` |
| Light aggregation | `channel.rs` | `AggKind::Max` |
| Light tier | `channel.rs` | Warm (staggered scheduling deferred) |
| Light max_radius (this task) | LOCKED design D2 | **15** |
| Light initial intensity | LOCKED design D1 | **200** |
| Shadowcast falloff | `propagate.rs:249` | `intensity / (1.0 + 0.1 * d)` |
| Shadowcast cutoff | `propagate.rs:247` | `dist_sq <= radius*radius` (Euclidean) |
| `propagate_shadowcast` signature | `propagate.rs` | `(tile_grid, &mut [u8], (u32,u32), u8, u32)` — NO blocking_cache, NO decay closure, NO channel param |
| Wall blocking | `propagate.rs` | binary opaque via `TileGrid::is_wall` |
| BSS stamps Light | `building_stamp.rs:25` STAMPED_CHANNELS | already includes Light — NO BSS change needed |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs:90` | 100 |

**Source center invariant** (EC-2/EC-11 from `propagate.rs`): `propagate_shadowcast`
applies `apply_agg(Max)` at the start of propagation so the source tile always
equals `initial_intensity` (200) regardless of walls — shadowcast cannot
"black out" its own seed.

---

## Section 3 — What to build

### 3.1 Modify `rust/crates/sim-systems/src/runtime/influence/update.rs`

Mirror the T7.10.A Warmth branch pattern for Light. New flow:

```text
tick():
  // Warmth branch (T7.10.A — unchanged)
  1. warmth_dirty = drain(influence_grid.dirty_regions[Warmth])
  2. if warmth_dirty non-empty:
       a. clear_pending(Warmth)
       b. for each region: propagate_bfs(... center, 200, decay, Warmth, 12)
     else:
       pending[Warmth].copy_from_slice(&current[Warmth])  // Cold-tier persistence

  // Light branch (T7.10.B — NEW)
  3. light_dirty = drain(influence_grid.dirty_regions[Light])
  4. if light_dirty non-empty:
       a. clear_pending(Light)
       b. for each region:
            cx = (region.min_x + region.max_x) / 2
            cy = (region.min_y + region.max_y) / 2
            propagate_shadowcast(
              &tile_grid,
              pending_buf_mut(Light),
              (cx, cy),
              200,   // LIGHT_INITIAL_INTENSITY
              15,    // LIGHT_MAX_RADIUS
            )
     else:
       pending[Light].copy_from_slice(&current[Light])  // Warm-tier persistence

  // Dispatch-shell baseline for remaining 6 channels (T7.10.C..F not wired)
  5. For each channel ≠ Warmth AND ≠ Light: clear_pending(ch)
  6. swap()  // all 8 channels swap together
```

**Concrete additions** (added to `update.rs`):

```rust
/// Initial intensity at the Light source tile.
const LIGHT_INITIAL_INTENSITY: u8 = 200;

/// Light shadowcast radius in tiles.
const LIGHT_MAX_RADIUS: u32 = 15;
```

Import update:

```rust
use sim_core::influence::{propagate_bfs, propagate_shadowcast, InfluenceChannel};
```

Light branch inside `tick()` (placed after the Warmth branch, before the
"other 6 channels" loop):

```rust
let light_idx = InfluenceChannel::Light as usize;
let light_dirty =
    std::mem::take(&mut resources.influence_grid.dirty_regions[light_idx]);

if !light_dirty.is_empty() {
    resources.influence_grid.clear_pending(InfluenceChannel::Light);
    for region in &light_dirty {
        let cx = (region.min_x + region.max_x) / 2;
        let cy = (region.min_y + region.max_y) / 2;
        let tile_grid = &resources.tile_grid;
        let pending =
            resources.influence_grid.pending_buf_mut(InfluenceChannel::Light);
        propagate_shadowcast(
            tile_grid,
            pending,
            (cx, cy),
            LIGHT_INITIAL_INTENSITY,
            LIGHT_MAX_RADIUS,
        );
    }
} else {
    // Warm-tier persistence: copy current → pending so swap is a no-op.
    let light_snapshot = resources.influence_grid.current[light_idx].clone();
    resources.influence_grid.pending[light_idx]
        .copy_from_slice(&light_snapshot);
}
```

Final loop change (skip both Warmth and Light):

```rust
for ch in InfluenceChannel::all() {
    if *ch != InfluenceChannel::Warmth && *ch != InfluenceChannel::Light {
        resources.influence_grid.clear_pending(*ch);
    }
}
```

### 3.2 Update IUS module docstring

Extend the `//!` header to mention T7.10.B land alongside T7.10.A, document
Light's shadowcast specifics (falloff, max_radius, aggregation Max, wall
blocking via `TileGrid::is_wall`).

### 3.3 Add `rust/crates/sim-test/tests/harness_t7_10_b_light_shadowcast_wiring.rs`

New harness file (9-10 assertions) mirroring the T7.10.A structure but
asserting Light shadowcast behaviour:

- **B1 source_center_lit**: `sample(SX, SY, Light) == 200` after 1 tick.
- **B2 falloff_one_step**: `sample(SX+1, SY, Light) ∈ [178, 184]` — expected
  `200 / (1 + 0.1*1) = 181` ± rounding.
- **B3 gradient_monotone**: pairwise non-increasing along cardinal axis +
  strictly decreasing endpoints (v0 > v15).
- **B4 boundary_at_max_radius**: `sample(SX+15, SY, Light) ∈ [76, 84]` —
  expected `200 / (1 + 0.1*15) = 80`.
- **B5 max_radius_cutoff**: `sample(SX+16, SY, Light) == 0` — beyond radius.
- **B6 persistence_across_ticks**: source still 200 after 10 event-less ticks
  (Warm-tier persistence branch).
- **B7+B8 other_channels_behavior**: T7.10.A regression (Warmth==200) +
  Spiritual/Beauty dispatch-shell zero + Noise/FoodAroma/Danger/Social zero.
- **B9 no_event_no_light**: zero events → all 3 sample positions zero after
  5 ticks.
- **B10 dirty_regions_drained**: `dirty_regions[Light].len() == 0` after IUS.

### 3.4 Update legacy harness assertions

Two existing tests in `harness_phase2_substantial.rs` assert Light is still
dispatch-shell. Update them to recognise T7.10.B:

- `harness_substantial_all_4_stamped_channels_dirty_1_non_stamped_0_full_pipeline`:
  change Light from `len == 1` to `len == 0` (drained).
- `harness_substantial_four_corner_stamps_clamp_no_oob_dirty`: change Light
  from `len == 4` to `len == 0` and add a corner-Light shadowcast value
  assertion (200 at each corner BFS center).

Also update `harness_t7_10_a_other_channels_remain_zero` to recognise Light
now propagates (== 200) while keeping the Spiritual/Beauty/Noise/FoodAroma/
Danger/Social zero assertions intact.

Update the `harness_phase2_substantial.rs` module docstring T7.10.A note to
include T7.10.B's Light drain + propagate behaviour.

---

## Section 4 — Locale

No new locale keys. UI is unchanged; world_renderer.gd still binds
`CHANNEL_WARMTH=0` (D3 decision — Light visualisation deferred).

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted harness
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture

# 3. T7.10.A regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture

# 4. Phase 2 regression
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
```

Expected: 9 new T7.10.B tests pass + 9 T7.10.A tests still pass +
15 substantial tests still pass + 0 clippy warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/update.rs` (single file edit)
  + `sim-test/tests/harness_t7_10_b_light_shadowcast_wiring.rs` (new harness)
  + 2 substantial test adjustments + 1 T7.10.A test adjustment.
- No FFI surface change (T7.7.B contract intact).
- No GDScript / scene / shader changes.
- Phase 0 design is already locked; planning debate skipped per --quick.
- Visual Verify: renderer keeps `CHANNEL_WARMTH=0`, so the visible sprite
  is unchanged from T7.10.A baseline (Warmth disc). Light disc exists in
  the buffer but is not rendered yet — visualisation is a later UI task.

---

## Section 7 — In-game verification (post-merge)

After `cargo build -p sim-bridge` AND `cargo build -p sim-bridge --release`
(to refresh BOTH debug AND release dylibs and avoid Case E stale dylib),
restart Godot 4.6 editor and press F6 on `scenes/main.tscn`:

**Expected console output** (unchanged from T7.10.A):
```
Initialize godot-rust (API v4.5.stable.official, runtime v4.6.stable.official)
WorldRenderer ready (T7.9.B render mechanism)
```

**Expected visual** (unchanged from T7.10.A):
- 1024×1024 sprite shows the **Warmth disc** (renderer still bound to
  channel 0).
- No visible Light disc — visualisation deferred (D3 decision).
- Warmth disc persists across frames (T7.10.A regression check).

**Background invariant** (not user-visible, asserted by harness only):
- `current[Light]` now contains the Light shadowcast field after the stamp,
  ready for a future renderer switch in a follow-up UI task.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.B is **single-channel** scope (Light only). Honest limitations:

1. **No renderer switch**: D3 keeps `CHANNEL_WARMTH=0` in world_renderer.gd.
   Light propagates in the buffer but is not visualised. A subsequent
   UI-only task will add a renderer channel switch / toggle UI.
2. **Single source only per region**: `clear_pending(Light)` before every
   shadowcast pass means a second building's stamp overwrites the first in
   pending before swap. The `Max` aggregation inside `propagate_shadowcast`
   handles overlap correctly **within one tick** (multi-region accumulation
   onto a cleared pending buffer); cross-tick multi-source is identical to
   single source for now.
3. **No demolition handling**: If a building is removed, the Light disc
   persists indefinitely (no negative dirty_regions / removal events). Out
   of scope for T7.10.B.
4. **Other 6 channels remain dispatch-shell**: Noise linear decay, Danger
   sight-radius cap, Social LOD-aware stamp, Spiritual/Beauty/FoodAroma
   exponential are all still zero buffers. T7.10.C..F will wire each.
5. **Shadowcast uses dirty_region center as source**: For single-tile
   building sources (BSS uses `event.position` as Chebyshev box center),
   region center = building center exactly. Larger structural footprints
   would need refinement in a later phase.

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any GDScript / scene / shader change (including world_renderer.gd channel
  binding — D3 deferred)
- Renderer channel switch / Light visualisation
- Multi-source Light persistence (T7.10.C+)
- Building demolition / negative events (T7.10.C+)
- Any non-Warmth, non-Light channel propagation (T7.10.C..F)
- Performance optimization beyond what `propagate_shadowcast` already provides
