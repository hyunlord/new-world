# T7.10.D — Danger channel linear-decay wiring (fourth-channel Phase 2 escape)

> Lane: `--quick` (sim-systems + sim-test + minimal GDScript renderer toggle)
> Scope: SINGLE channel (Danger only). Warmth (T7.10.A), Light (T7.10.B), and
> Noise (T7.10.C) already wired. Remaining 4 channels (Spiritual, Beauty,
> FoodAroma, Social) stay dispatch-shell.
> Governance: v3.3.16. Visual: square-ish Danger field at the stamped
> building's center via linear-decay BFS with sight-radius cap=15 tiles,
> alpha=5 per step, NO wall blocking (Phase 0 ISSUE 3 spec — Danger pierces
> walls because hostiles see through openings, around corners, hear screams).

---

## Section 1 — Implementation Intent

T7.10.A/B/C escaped the dispatch shell for Warmth (BFS exponential), Light
(shadowcast Euclidean), and Noise (linear-decay alpha=15 + density-derived
wall blocking). T7.10.D is the **fourth channel to escape** — Danger
linear-decay BFS wired end-to-end via the existing `propagate_danger`
primitive (which wraps `propagate_linear` with alpha=5, max_radius=15, and
a no-op `BlockingNone` cache so walls do not attenuate).

After this commit, `on_building_placed(32, 32, 12)` produces a Warmth disc
(T7.10.A regression), a Light disc (T7.10.B regression), a Noise field
(T7.10.C regression), **and** a Danger field (square, sight-radius=15, alpha=5
per step) in `current[Danger]` after one tick.

Renderer change: extend the SPACE-key channel cycle in `world_renderer.gd`
from 3-state (Warmth → Light → Noise) to 4-state (Warmth → Light → Noise →
Danger → Warmth) so the operator can visually confirm the new buffer.

Remaining 4 channels (Spiritual, Beauty, FoodAroma, Social) stay
dispatch-shell — they will be wired in T7.10.E..F.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Danger enum index | `channel.rs` | `InfluenceChannel::Danger as usize == 4` |
| Danger aggregation | `channel.rs:109` | `AggKind::Max` |
| Danger tier | `channel.rs` | Hot (every tick, parity with Noise) |
| Danger max_radius | `propagate_danger` wrapper (propagate.rs:318) | **15 tiles** (sight-radius cap) |
| Danger initial intensity | LOCKED Phase 0 ISSUE 3 spec | **200** (Warmth/Light/Noise parity) |
| Linear decay alpha | `propagate_danger` wrapper | **5** per BFS step |
| Linear cutoff | `propagate.rs` `propagate_linear` | `intensity < 5` exits |
| Wall blocking | `propagate_danger` wrapper | **NONE** (`BlockingNone`) — pierces walls |
| `propagate_danger` signature | `propagate.rs:318` | `(tile_grid, &mut [u8], (u32,u32), u8)` (no blocking_cache arg) |
| BSS STAMPED_CHANNELS (post-D) | `building_stamp.rs` | now **6 channels**: Warmth, Light, Noise, Danger, Spiritual, Beauty |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs:90` | 100 |

**Source center invariant** (parallels Warmth/Light/Noise): `propagate_linear`
applies `apply_agg(Max)` at the start of propagation so the source tile
always equals `DANGER_INITIAL_INTENSITY` (200) regardless of neighbouring
geometry — the source cannot decay below its seed within a single pass.

**BFS distance invariant**: Linear decay uses 4-neighbor frontier expansion
(Manhattan-like). One-step neighbour `(cx+1, cy)` is reached in BFS d=1.
Expected value: `200 - 1*5 = 195` exact (integer arithmetic only).
Diagonal tile `(cx+1, cy+1)` is reached in BFS d=2: `200 - 2*5 = 190`.

**Cap discriminator**: at d=15 (the max_radius), value = `200 - 15*5 = 125`.
At d=16 (one beyond cap), value = 0 (frontier never expands past cap).
This is a clean discriminator vs Noise's natural radius cutoff (Noise:
d=13→5, d=14→0) and Warmth's max_radius=12.

---

## Section 3 — What to build

### 3.1 Modify `rust/crates/sim-systems/src/runtime/influence/building_stamp.rs`

Append `Danger` to `STAMPED_CHANNELS` so BSS marks `dirty_regions[Danger]`
on every BuildingPlacedEvent. Old length was 5 (Warmth, Light, Noise,
Spiritual, Beauty); new length is 6.

Update the BSS internal unit test name + assertion:
`single_event_marks_5_channels_dirty` → `single_event_marks_stamped_channels_dirty`.

### 3.2 Modify `rust/crates/sim-systems/src/runtime/influence/update.rs`

Mirror the T7.10.C Noise branch pattern for Danger (fourth branch). New flow:

```text
tick():
  1. Warmth branch (T7.10.A) — unchanged
  2. Light branch  (T7.10.B) — unchanged
  3. Noise branch  (T7.10.C) — unchanged
  4. Danger branch (T7.10.D) — NEW:
       danger_dirty = drain(influence_grid.dirty_regions[Danger])
       if danger_dirty non-empty:
         clear_pending(Danger)
         for each region:
           cx = (region.min_x + region.max_x) / 2
           cy = (region.min_y + region.max_y) / 2
           propagate_danger(&tile_grid, pending_buf_mut(Danger),
                            (cx, cy), 200 /* DANGER_INITIAL_INTENSITY */)
       else:
         pending[Danger].copy_from_slice(&current[Danger])  // Hot-tier persistence
  5. For each channel ∉ {Warmth, Light, Noise, Danger}: clear_pending(ch)
  6. swap()
```

**Concrete additions** (added to `update.rs`):

```rust
/// Initial intensity at the Danger source tile.
const DANGER_INITIAL_INTENSITY: u8 = 200;
```

Import update:

```rust
use sim_core::influence::{
    propagate_bfs, propagate_danger, propagate_noise, propagate_shadowcast,
    InfluenceChannel,
};
```

Danger branch inside `tick()` (placed after the Noise branch, before the
"remaining channels" loop):

```rust
let danger_idx = InfluenceChannel::Danger as usize;
let danger_dirty =
    std::mem::take(&mut resources.influence_grid.dirty_regions[danger_idx]);

if !danger_dirty.is_empty() {
    resources.influence_grid.clear_pending(InfluenceChannel::Danger);
    for region in &danger_dirty {
        let cx = (region.min_x + region.max_x) / 2;
        let cy = (region.min_y + region.max_y) / 2;
        let tile_grid = &resources.tile_grid;
        let pending =
            resources.influence_grid.pending_buf_mut(InfluenceChannel::Danger);
        propagate_danger(
            tile_grid,
            pending,
            (cx, cy),
            DANGER_INITIAL_INTENSITY,
        );
    }
} else {
    // Hot-tier persistence (V7 Phase 2 no-agents): copy current → pending so
    // the swap is a no-op for Danger on event-less ticks. Replace with
    // empty pending once agent threat events arrive (Phase 3+).
    let danger_snapshot = resources.influence_grid.current[danger_idx].clone();
    resources.influence_grid.pending[danger_idx]
        .copy_from_slice(&danger_snapshot);
}
```

Final loop change (skip Warmth, Light, Noise, AND Danger):

```rust
for ch in InfluenceChannel::all() {
    if *ch != InfluenceChannel::Warmth
        && *ch != InfluenceChannel::Light
        && *ch != InfluenceChannel::Noise
        && *ch != InfluenceChannel::Danger
    {
        resources.influence_grid.clear_pending(*ch);
    }
}
```

### 3.3 Update IUS module docstring

Extend the `//!` header to mention T7.10.D alongside T7.10.A/B/C. Document
Danger's linear-decay specifics (alpha=5, sight-radius cap=15 via
`propagate_danger`'s explicit `max_radius`, aggregation Max, NO wall
blocking — pierces walls per Phase 0 ISSUE 3 spec).

### 3.4 Add `rust/crates/sim-test/tests/harness_t7_10_d_danger_linear_wiring.rs`

New harness file (10 assertions) mirroring the T7.10.C structure but
asserting Danger linear-decay behaviour:

- **D1 source_center_lit**: `sample(SX, SY, Danger) == 200` after 1 tick.
- **D2 linear_one_step_alpha_discriminator**: `sample(SX+1, SY, Danger) == 195`
  exact — `200 - 5 = 195`. Discriminator: Noise alpha=15 would give 185,
  Warmth exponential would give ~172. Only alpha=5 yields 195.
- **D3 bfs_distance_manhattan_discriminator**: `sample(SX+1, SY+1, Danger) == 190`
  exact — `200 - 2*5 = 190` at BFS d=2. Discriminator: Euclidean would
  give ≈193, Noise BFS would give 170. Only 4-neighbor BFS Manhattan +
  alpha=5 yields 190.
- **D4 gradient_monotone**: strict `>` pairwise along cardinal axis for
  d=0..=14 (5-unit drops, no truncation ties possible within max_radius).
- **D5 sight_radius_cap_boundary**: `sample(SX+15, SY, Danger) == 125`
  (d=15: 200-15*5=125, still inside max_radius) AND
  `sample(SX+16, SY, Danger) == 0` (d=16: outside max_radius, never
  written). This is the cap discriminator — Noise (natural cutoff)
  terminates at d=13, Warmth at d=12.
- **D6 persistence_ten_ticks**: source still 200 after 1 stamp + 10
  event-less ticks (Hot-tier persistence branch in V7 Phase 2).
- **D7 no_event_no_danger**: zero events → 3 sample positions all zero
  after 5 ticks.
- **D8 warmth_light_noise_regression_guard**: `sample(SX, SY, Warmth) == 200`
  AND `sample(SX, SY, Light) == 200` AND `sample(SX, SY, Noise) == 200`
  (T7.10.A/B/C regression).
- **D9 dirty_regions_drained**: `dirty_regions[Danger].len() == 0` after IUS
  via `std::mem::take`.
- **D10 other_channels_remain_zero**: Spiritual, Beauty (still dispatch-shell)
  + FoodAroma, Social (not stamped) all sample to 0 at source.

### 3.5 Update legacy harness assertions

Four existing harness tests assert Danger is non-stamped or zero. Update
each to recognise T7.10.D:

- `harness_t7_10_a_warmth_wiring.rs` `_other_channels_remain_zero`: remove
  Danger from the unstamped zero list; add Danger=200 regression guard at
  source center.
- `harness_t7_10_b_light_shadowcast_wiring.rs` `_other_channels_behavior`:
  remove Danger from unstamped zero list (keep FoodAroma/Social);
  add Danger=200 regression guard at source center.
- `harness_t7_10_c_noise_linear_wiring.rs` `_other_channels_remain_zero`:
  remove Danger from unstamped zero list; add Danger=200 regression guard
  at source center.
- `harness_phase2_ffi.rs` A11/A16/A20/radius_zero: move Danger from
  non-stamped set (3 channels: Danger/FoodAroma/Social) to stamped set
  (6 channels: Warmth/Light/Noise/Danger/Spiritual/Beauty). Update doc
  comments to "6 channels". Non-stamped set becomes 2 channels:
  FoodAroma/Social.
- `harness_phase2_substantial.rs` A5: add `danger_len == 0` drain assertion
  (parallel to Warmth/Light/Noise); add Danger=200 propagation
  discriminator; remove Danger from non-stamped list.

Also update the `harness_phase2_substantial.rs` module docstring T7.10.C
note to include T7.10.D's Danger drain + propagate behaviour.

### 3.6 Update SPACE channel cycle in `scripts/ui/world_renderer.gd`

Extend the SPACE-key channel cycle from 3-state (Warmth → Light → Noise →
Warmth) to 4-state (Warmth → Light → Noise → Danger → Warmth):

- Add `const CHANNEL_DANGER := 4` (matches `InfluenceChannel::Danger as usize`).
- Extend the modulo cycle to use `% 4` with the next index lookup:
  `Warmth (0) → Light (3) → Noise (2) → Danger (4) → Warmth (0)`.
- Console log emits `channel_name = "Danger"` on the Noise → Danger toggle
  and `channel_name = "Warmth"` on the Danger → Warmth toggle.

This is a minimal renderer change (one constant + one branch in the cycle
function). No new visualisation logic — the existing uniform/texture
upload path already handles arbitrary channel indices.

---

## Section 4 — Locale

No new locale keys. UI is unchanged except for the SPACE cycle extension
which only adds a debug console string (not user-visible).

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted T7.10.D harness
cd rust && cargo test -p sim-test --test harness_t7_10_d_danger_linear_wiring -- --nocapture

# 3. T7.10.A/B/C regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b1_space_toggle -- --nocapture

# 4. Phase 2 regression
cd rust && cargo test -p sim-test --test harness_phase2_ffi -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
```

Expected: 10 new T7.10.D tests pass + 10 T7.10.A pass + 10 T7.10.B pass +
10 T7.10.C pass + all B1 toggle tests pass + 15 substantial tests pass +
all FFI tests pass + 0 clippy warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/{building_stamp.rs, update.rs}`
  (two file edits) + `sim-test/tests/harness_t7_10_d_danger_linear_wiring.rs`
  (new harness) + 4 legacy harness adjustments + 1 GDScript renderer
  constant + cycle extension.
- No FFI surface change (T7.7.B contract intact).
- GDScript change is a minimal SPACE-cycle extension (one constant + cycle
  update), no scene/shader rewrite.
- Phase 0 design is already locked (alpha=5, cap=15, no walls — ISSUE 3 spec);
  planning debate skipped per --quick.
- Visual Verify: renderer's SPACE cycle now exposes the Danger buffer;
  visual harness can confirm Danger field by pressing SPACE three times
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
- Press SPACE a third time → shows **Danger field** (square, sight-radius
  cap = 15 tiles, 5-unit step gradient, pierces walls).
- Press SPACE a fourth time → cycles back to Warmth disc.
- Console prints `channel_name = "Warmth" | "Light" | "Noise" | "Danger"`
  on each toggle.

**Background invariant** (not user-visible, asserted by harness only):
- `current[Danger]` contains the linear-decay field after the stamp.
- Source tile = 200; d=1 neighbour = 195; d=15 boundary = 125; d=16 = 0.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.D is **single-channel** scope (Danger only). Honest limitations:

1. **No agent threat events yet**: Phase 2 has no agents, so the only
   Danger source is the static BuildingPlacedEvent stamp. Phase 3+ will
   add per-agent transient threat sources (hostile entities, weapon
   strikes, fire) which require Hot-tier semantics (no persistence —
   empty pending each tick). The current persistence branch matches V7
   Phase 2's "single static source" reality.
2. **Linear decay, alpha=5, cap=15, no walls**: Per Phase 0 ISSUE 3
   spec — Danger pierces walls because hostiles see through openings,
   around corners, and screams travel through doors. This is the
   locked model; no line-of-sight refinement, no occlusion, no
   intensity per threat-type.
3. **No threat-source removal handling**: If a building is removed,
   the Danger field persists indefinitely (no negative dirty_regions /
   removal events). Out of scope for T7.10.D.
4. **Remaining 4 channels stay dispatch-shell**: Spiritual/Beauty
   (stamped, propagation not wired), FoodAroma/Social (not stamped).
   T7.10.E..F will wire each.
5. **Single-tile source assumption**: BSS uses `event.position` as
   Chebyshev box center; region center = building center exactly.
   Larger structural footprints would need refinement in a later phase.

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any scene / shader change (only minimal world_renderer.gd cycle extension)
- Agent-driven transient threat events (Phase 3+)
- Building demolition / negative events (T7.10.E+)
- Any non-{Warmth,Light,Noise,Danger} channel propagation (T7.10.E..F)
- Line-of-sight occlusion, threat-type intensity differentiation, or
  per-agent fear modulation (Phase 0 ISSUE 3 v0.1.1 fix is the locked model:
  alpha=5 linear, cap=15, no walls)
- Performance optimization beyond what `propagate_danger` / `propagate_linear`
  already provide
