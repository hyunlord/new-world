# T7.10.C — Noise channel linear-decay wiring (third-channel Phase 2 escape)

> Lane: `--quick` (sim-systems + sim-test, no FFI / scene changes)
> Scope: SINGLE channel (Noise only). Warmth (T7.10.A) and Light (T7.10.B)
> already wired. Other 5 channels remain dispatch-shell.
> Governance: v3.3.16. Visual: octagonal/square Noise field at the stamped
> building's center via linear-decay BFS (Songs of Syx 2-tile ISSUE 2 v0.1.1
> fix: alpha=15 per step + density-derived wall blocking).

---

## Section 1 — Implementation Intent

T7.10.A/B escaped the dispatch shell for Warmth (BFS exponential) and Light
(shadowcast Euclidean). T7.10.C is the **third channel to escape** — Noise
linear-decay BFS wired end-to-end via `propagate_noise` (which wraps
`propagate_linear` with alpha=15 + `u32::MAX` max_radius so the natural
`intensity < 5` cutoff terminates the BFS frontier).

After this commit, `on_building_placed(32, 32, 12)` produces a Warmth disc
(T7.10.A regression), a Light disc (T7.10.B regression), **and** a Noise
field (linear-decay, natural radius ≈ 13 tiles) in `current[]` after one tick.

Renderer change is **part of T7.10.B1's earlier 3-state SPACE cycle**
(Warmth → Light → Noise → Warmth) implemented in `world_renderer.gd`. No
additional UI change required in T7.10.C — the visualisation toggle was
finalised in B1 and exercises this new buffer.

Other 5 channels stay dispatch-shell — they will be wired in T7.10.D..F.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Noise enum index | `channel.rs` | `InfluenceChannel::Noise as usize == 2` |
| Noise aggregation | `channel.rs` | `AggKind::Max` |
| Noise tier | `channel.rs` | Hot (every tick, parity with Danger) |
| Noise max_radius (this task) | `propagate_noise` wrapper | `u32::MAX` (natural cutoff) |
| Noise initial intensity | LOCKED design (Warmth/Light parity) | **200** |
| Linear decay alpha | `propagate.rs` `propagate_noise` | **15** per BFS step |
| Linear cutoff | `propagate.rs:363` `propagate_linear` | `intensity < 5` exits |
| Natural radius | derived | 200 - 13×15 = 5 (queued); d=14 → saturating_sub=0 (not written) |
| `propagate_noise` signature | `propagate.rs` | `(tile_grid, blocking_cache, &mut [u8], (u32,u32), u8)` |
| `propagate_linear` signature | `propagate.rs` | `(tile_grid, blocking_cache, &mut [u8], (u32,u32), u8, u32, u8)` — alpha + max_radius last two |
| Wall blocking | `propagate.rs` | density-derived via `MaterialBlockingCache` (4-neighbor BFS) |
| BSS STAMPED_CHANNELS | `building_stamp.rs` | now 5 channels: Warmth, Light, Noise, Spiritual, Beauty |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs:90` | 100 |

**Source center invariant** (parallels Warmth/Light): `propagate_linear`
applies `apply_agg(Max)` at the start of propagation (propagate.rs:358) so
the source tile always equals `NOISE_INITIAL_INTENSITY` (200) regardless of
neighbouring walls — the source cannot decay below its seed within a single
pass.

**BFS distance invariant**: Linear decay uses 4-neighbor frontier expansion
(Manhattan-like). Diagonal tile `(cx+1, cy+1)` is reached in BFS d=2, not
Euclidean d=√2. Expected value: 200 - 2*15 = **170** exact (no f32, integer
arithmetic only). This is a clean discriminator vs Light's Euclidean falloff
(which would give 200 / (1 + 0.1*√2) ≈ 175 at the same tile).

---

## Section 3 — What to build

### 3.1 Modify `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs`

Append `Noise` to `STAMPED_CHANNELS` so BSS marks `dirty_regions[Noise]`
on every BuildingPlacedEvent. Old length was 4 (Warmth, Light, Spiritual,
Beauty); new length is 5.

Update the BSS internal unit test name + assertion:
`single_event_marks_4_channels_dirty` → `single_event_marks_5_channels_dirty`.

### 3.2 Modify `rust/crates/sim-systems/src/runtime/influence/update.rs`

Mirror the T7.10.B Light branch pattern for Noise (third branch). New flow:

```text
tick():
  1. Warmth branch (T7.10.A) — unchanged
  2. Light branch  (T7.10.B) — unchanged
  3. Noise branch  (T7.10.C) — NEW:
       noise_dirty = drain(influence_grid.dirty_regions[Noise])
       if noise_dirty non-empty:
         clear_pending(Noise)
         for each region:
           cx = (region.min_x + region.max_x) / 2
           cy = (region.min_y + region.max_y) / 2
           propagate_noise(&tile_grid, &blocking_cache, pending_buf_mut(Noise),
                           (cx, cy), 200 /* NOISE_INITIAL_INTENSITY */)
       else:
         pending[Noise].copy_from_slice(&current[Noise])  // Hot-tier persistence
  4. For each channel ∉ {Warmth, Light, Noise}: clear_pending(ch)
  5. swap()
```

**Concrete additions** (added to `update.rs`):

```rust
/// Initial intensity at the Noise source tile.
const NOISE_INITIAL_INTENSITY: u8 = 200;
```

Import update:

```rust
use sim_core::influence::{
    propagate_bfs, propagate_noise, propagate_shadowcast, InfluenceChannel,
};
```

Noise branch inside `tick()` (placed after the Light branch, before the
"other 5 channels" loop):

```rust
let noise_idx = InfluenceChannel::Noise as usize;
let noise_dirty =
    std::mem::take(&mut resources.influence_grid.dirty_regions[noise_idx]);

if !noise_dirty.is_empty() {
    resources.influence_grid.clear_pending(InfluenceChannel::Noise);
    for region in &noise_dirty {
        let cx = (region.min_x + region.max_x) / 2;
        let cy = (region.min_y + region.max_y) / 2;
        let tile_grid = &resources.tile_grid;
        let blocking_cache = &resources.material_blocking_cache;
        let pending =
            resources.influence_grid.pending_buf_mut(InfluenceChannel::Noise);
        propagate_noise(
            tile_grid,
            blocking_cache,
            pending,
            (cx, cy),
            NOISE_INITIAL_INTENSITY,
        );
    }
} else {
    // Hot-tier persistence (V7 Phase 2 no-agents): copy current → pending so
    // the swap is a no-op for Noise on event-less ticks. Replace with
    // empty pending once agent acoustic events arrive (Phase 3+).
    let noise_snapshot = resources.influence_grid.current[noise_idx].clone();
    resources.influence_grid.pending[noise_idx]
        .copy_from_slice(&noise_snapshot);
}
```

Final loop change (skip Warmth, Light, AND Noise):

```rust
for ch in InfluenceChannel::all() {
    if *ch != InfluenceChannel::Warmth
        && *ch != InfluenceChannel::Light
        && *ch != InfluenceChannel::Noise
    {
        resources.influence_grid.clear_pending(*ch);
    }
}
```

### 3.3 Update IUS module docstring

Extend the `//!` header to mention T7.10.C alongside T7.10.A/B. Document
Noise's linear-decay specifics (alpha=15, natural radius via `intensity<5`
cutoff, aggregation Max, wall blocking via `MaterialBlockingCache`).

### 3.4 Add `rust/crates/sim-test/tests/harness_t7_10_c_noise_linear_wiring.rs`

New harness file (10 assertions) mirroring the T7.10.B structure but
asserting Noise linear-decay behaviour:

- **C1 source_center_lit**: `sample(SX, SY, Noise) == 200` after 1 tick.
- **C2 linear_one_step_alpha_discriminator**: `sample(SX+1, SY, Noise) == 185`
  exact — `200 - 15 = 185`. Discriminator: shadowcast would give 181, BFS
  exponential (Warmth) would give ~172. Only linear-decay alpha=15 yields 185.
- **C3 bfs_distance_manhattan_discriminator**: `sample(SX+1, SY+1, Noise) == 170`
  exact — `200 - 2*15 = 170` at BFS d=2. Discriminator: Euclidean would give
  ≈179. Only 4-neighbor BFS Manhattan-like expansion yields 170.
- **C4 gradient_monotone**: strict `>` pairwise along cardinal axis for
  d=0..=11 (15-unit drops, no truncation ties possible).
- **C5 natural_radius_boundary**: `sample(SX+13, SY, Noise) == 5` (d=13:
  200-13*15=5, still queued because `5 < 5` is false) AND
  `sample(SX+14, SY, Noise) == 0` (d=14: saturating_sub(5,15)=0, after_block>0
  fails — not written).
- **C6 persistence_ten_ticks**: source still 200 after 1 stamp + 9 event-less
  ticks (Hot-tier persistence branch in V7 Phase 2).
- **C7 no_event_no_noise**: zero events → 3 sample positions all zero after
  5 ticks.
- **C8 warmth_and_light_regression_guard**: `sample(SX, SY, Warmth) == 200`
  AND `sample(SX, SY, Light) == 200` (T7.10.A/B regression).
- **C9 dirty_regions_drained**: `dirty_regions[Noise].len() == 0` after IUS
  via `std::mem::take`.
- **C10 other_channels_remain_zero**: Spiritual, Beauty (still dispatch-shell)
  + FoodAroma, Danger, Social (not stamped) all sample to 0 at source.

### 3.5 Update legacy harness assertions

Five existing harness tests assert Noise is non-stamped or zero. Update
each to recognise T7.10.C:

- `harness_t7_10_a_warmth_wiring.rs` `_other_channels_remain_zero`: remove
  Noise from the non-Warmth/non-Light zero list; add Noise=200 regression
  guard at source center.
- `harness_t7_10_b_light_shadowcast_wiring.rs` `_other_channels_behavior`:
  remove Noise from unstamped zero list (keep FoodAroma/Danger/Social);
  add Noise=200 regression guard at source center.
- `harness_phase2_ffi.rs` A11/A16/A20: move Noise from non-stamped set
  (3 channels: Danger/FoodAroma/Social) to stamped set (5 channels: Warmth/
  Light/Noise/Spiritual/Beauty). Update doc comments to "5 channels".
- `harness_phase2_substantial.rs` A5: add `noise_len == 0` drain assertion
  (parallel to Warmth/Light); remove Noise from non-stamped list.
- `harness_phase2_substantial.rs` A15/S12: add `noise_regs.len() == 0`
  drain assertion; add Noise=200 corner BFS regression guard.

Also update the `harness_phase2_substantial.rs` module docstring T7.10.B
note to include T7.10.C's Noise drain + propagate behaviour.

### 3.6 Update SPACE 3-state cycle test (already done in T7.10.B1)

`harness_t7_10_b1_space_toggle.rs::B1.S4/S5` already verifies the
`CHANNEL_WARMTH → CHANNEL_LIGHT → CHANNEL_NOISE → CHANNEL_WARMTH` cycle
in `world_renderer.gd`. No further GDScript change is required for T7.10.C —
the renderer toggle was finalised in B1 and now exercises the new buffer.

---

## Section 4 — Locale

No new locale keys. UI is unchanged; world_renderer.gd retains the B1
SPACE 3-state cycle from T7.10.B1.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted T7.10.C harness
cd rust && cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture

# 3. T7.10.A/B regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b1_light_viz_toggle -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b1_space_toggle -- --nocapture

# 4. Phase 2 regression
cd rust && cargo test -p sim-test --test harness_phase2_ffi -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
```

Expected: 10 new T7.10.C tests pass + 10 T7.10.A pass + 10 T7.10.B pass +
all B1 toggle tests pass + 15 substantial tests pass + 21 FFI tests pass +
0 clippy warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/{building_stamp.rs, update.rs}`
  (two file edits) + `sim-test/tests/harness_t7_10_c_noise_linear_wiring.rs`
  (new harness) + 5 legacy harness adjustments.
- No FFI surface change (T7.7.B contract intact).
- No GDScript / scene / shader changes (B1 toggle already includes Noise).
- Phase 0 design is already locked (alpha=15 from propagate_noise wrapper);
  planning debate skipped per --quick.
- Visual Verify: renderer's SPACE cycle from B1 already exposes the new
  channel; visual harness can confirm Noise field by pressing SPACE twice
  from default state.

---

## Section 7 — In-game verification (post-merge)

After `cargo build -p sim-bridge` AND `cargo build -p sim-bridge --release`
(refresh BOTH debug AND release dylibs — Case E protection), restart Godot
4.6 editor and press F6 on `scenes/main.tscn`:

**Expected console output**:
```
Initialize godot-rust (API v4.5.stable.official, runtime v4.6.stable.official)
WorldRenderer ready (T7.9.B render mechanism)
```

**Expected interaction**:
- Initial: 1024×1024 sprite shows **Warmth disc** (channel 0, default).
- Press SPACE once → shows **Light disc** (octagonal/circular shadowcast).
- Press SPACE again → shows **Noise field** (square-ish linear-decay,
  natural radius ≈ 13 tiles, 15-unit step gradient).
- Press SPACE a third time → cycles back to Warmth disc.
- Console prints `channel_name = "Warmth" | "Light" | "Noise"` on each toggle.

**Background invariant** (not user-visible, asserted by harness only):
- `current[Noise]` contains the linear-decay field after the stamp.
- Source tile = 200; d=1 neighbour = 185; d=13 boundary = 5; d=14 = 0.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.C is **single-channel** scope (Noise only). Honest limitations:

1. **No agent acoustic events yet**: Phase 2 has no agents, so the only
   Noise source is the static BuildingPlacedEvent stamp. Phase 3+ will add
   per-agent transient noise (footsteps, speech, combat) which require Hot-
   tier semantics (no persistence — empty pending each tick). The current
   persistence branch matches V7 Phase 2's "single static source" reality.
2. **Linear decay only**: Alpha=15 is the Songs of Syx 2-tile ISSUE 2
   v0.1.1 fix. No frequency-dependent attenuation, no doppler, no echo.
   Wall blocking is density-derived (per-edge MaterialBlockingCache lookup).
3. **No demolition handling**: If a building is removed, the Noise field
   persists indefinitely (no negative dirty_regions / removal events). Out
   of scope for T7.10.C.
4. **Other 5 channels remain dispatch-shell**: Spiritual/Beauty (stamped,
   propagation not wired), FoodAroma/Danger/Social (not stamped). T7.10.D..F
   will wire each.
5. **Single-tile source assumption**: BSS uses `event.position` as
   Chebyshev box center; region center = building center exactly. Larger
   structural footprints would need refinement in a later phase.

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any GDScript / scene / shader change (B1 SPACE cycle already includes Noise)
- Agent-driven transient noise events (Phase 3+)
- Building demolition / negative events (T7.10.D+)
- Any non-Warmth, non-Light, non-Noise channel propagation (T7.10.D..F)
- Frequency-dependent attenuation, doppler, echo, or other realistic acoustic
  physics (Songs of Syx 2-tile ISSUE 2 v0.1.1 fix is the locked model)
- Performance optimization beyond what `propagate_noise` / `propagate_linear`
  already provide
