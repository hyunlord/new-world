# T7.10.E — Spiritual channel BFS exponential-decay wiring (fifth-channel Phase 2 escape)

> Lane: `--quick` (sim-systems + sim-test + minimal GDScript renderer toggle)
> Scope: SINGLE channel (Spiritual only). Warmth (T7.10.A), Light (T7.10.B),
> Noise (T7.10.C), and Danger (T7.10.D) already wired. Remaining 3 channels
> (Beauty, FoodAroma, Social) stay dispatch-shell.
> Governance: v3.3.16. Visual: gentler-falloff Spiritual disc at the stamped
> building's center via exponential-decay BFS with max_radius=15 tiles,
> k=0.10 per step, wall-aware (density-derived blocking — ritual influence
> respects physical barriers like Warmth does).

---

## Section 1 — Implementation Intent

T7.10.A/B/C/D escaped the dispatch shell for Warmth (BFS exponential k=0.15),
Light (shadowcast Euclidean), Noise (linear-decay α=15 + density wall
blocking), and Danger (linear-decay α=5, sight-radius cap=15, no walls).
T7.10.E is the **fifth channel to escape** — Spiritual exponential BFS
wired end-to-end via the existing `propagate_bfs` primitive at
`propagate.rs:75` (same primitive Warmth uses), with k=0.10
(gentler than Warmth's 0.15 — ritual influence carries further than thermal
heat) and a longer max_radius=15 (matching Light's FOV reach).

After this commit, `on_building_placed(32, 32, 12)` produces a Warmth disc
(T7.10.A regression), a Light disc (T7.10.B regression), a Noise field
(T7.10.C regression), a Danger field (T7.10.D regression), **and** a
Spiritual disc (BFS exponential k=0.10, max_radius=15) in `current[Spiritual]`
after one tick.

Renderer change: extend the SPACE-key channel cycle in `world_renderer.gd`
from 4-state (Warmth → Light → Noise → Danger) to 5-state (Warmth → Light →
Noise → Danger → Spiritual → Warmth) so the operator can visually confirm
the new buffer.

Remaining 3 channels (Beauty, FoodAroma, Social) stay dispatch-shell — Beauty
will be wired in T7.10.F; FoodAroma/Social are deferred until agents arrive.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Spiritual enum index | `channel.rs` | `InfluenceChannel::Spiritual as usize == 6` |
| Spiritual aggregation | `channel.rs` | `AggKind::Max` (parity with Warmth/Light/Noise/Danger) |
| Spiritual tier | `channel.rs` | Cold (parity with Warmth — ritual sources are persistent structural placements, not transient agent events) |
| Spiritual max_radius | LOCKED design | **15 tiles** (gentler reach than Warmth=12; parity with Light=15) |
| Spiritual initial intensity | LOCKED design | **200** (Warmth/Light/Noise/Danger parity) |
| Exponential decay constant k | LOCKED design | **0.10** (gentler than Warmth's 0.15) |
| Pre-computed decay multiplier | `exp(-0.10)` | **0.904_837** (Rust does not const-eval `f32::exp` in stable) |
| Wall blocking | `propagate_bfs` blocking_cache | density-derived (same as Warmth — ritual influence respects walls) |
| `propagate_bfs` signature | `propagate.rs:75` | `(tile_grid, blocking_cache, &mut [u8], (u32,u32), u8, decay_fn, channel, max_radius)` |
| BSS STAMPED_CHANNELS (unchanged post-E) | `building_stamp.rs` | still **6 channels**: Warmth, Light, Noise, Danger, Spiritual, Beauty (Spiritual was already in the list since T7.10.A; only the propagation branch was missing) |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs` | 100 |

**Source center invariant** (parallels Warmth): `propagate_bfs` applies
`apply_agg(Max)` at the source so `sample(SX, SY, Spiritual) == 200`
regardless of neighbouring geometry — the source cannot decay below its
seed within a single pass.

**BFS distance invariant**: Exponential decay along 4-neighbor frontier
expansion. One-step neighbour value = `200 * 0.904_837 ≈ 181`
(integer arithmetic via `(i as f32 * SPIRITUAL_DECAY_PER_STEP) as u8`
truncating).
- d=1: `200 * 0.904_837 ≈ 180.967 → 180 or 181 depending on f32 rounding`
- d=2: `181 * 0.904_837 ≈ 163.776 → 163` (chain via intermediate i)
- d=15: `200 * 0.904_837^15 ≈ 200 * 0.2231 ≈ 44.6 → 44 or 45`
- d=16: 0 (frontier never expands past max_radius=15)

**Discriminator vs Warmth**: Warmth's k=0.15 yields d=1 ≈ 172 (multiplier
0.8607); Spiritual's k=0.10 yields d=1 ≈ 181 (multiplier 0.9048). The
~9-unit gap at d=1 is the cleanest single-step discriminator. At
max_radius (d=15), Warmth would already be at 0 (radius=12 cuts it off);
Spiritual at d=15 ≈ 44 (radius=15 still active).

---

## Section 3 — What to build

### 3.1 `rust/crates/sim-systems/src/runtime/influence/update.rs`

Mirror the T7.10.A Warmth branch pattern for Spiritual (fifth branch). New
flow:

```text
tick():
  1. Warmth branch   (T7.10.A) — unchanged
  2. Light branch    (T7.10.B) — unchanged
  3. Noise branch    (T7.10.C) — unchanged
  4. Danger branch   (T7.10.D) — unchanged
  5. Spiritual branch (T7.10.E) — NEW:
       spiritual_dirty = drain(influence_grid.dirty_regions[Spiritual])
       if spiritual_dirty non-empty:
         clear_pending(Spiritual)
         for each region:
           cx = (region.min_x + region.max_x) / 2
           cy = (region.min_y + region.max_y) / 2
           propagate_bfs(&tile_grid, &blocking_cache, pending_buf_mut(Spiritual),
                         (cx, cy), 200, |i, _| i * 0.904_837,
                         Spiritual, 15)
       else:
         pending[Spiritual].copy_from_slice(&current[Spiritual])  // Cold-tier persistence
  6. For each channel ∉ {Warmth, Light, Noise, Danger, Spiritual}: clear_pending(ch)
  7. swap()
```

**Concrete additions** (added to `update.rs`):

```rust
/// `exp(-0.10)` — spiritual exponential decay per BFS step.
///
/// Pre-computed: Rust does not const-eval `f32::exp` in stable.
/// k = 0.10 (gentler than Warmth's 0.15 — ritual influence carries further
/// than thermal heat).
const SPIRITUAL_DECAY_PER_STEP: f32 = 0.904_837;

/// Initial intensity at the Spiritual source tile.
const SPIRITUAL_INITIAL_INTENSITY: u8 = 200;

/// Spiritual propagation radius in tiles.
///
/// 15 = longer reach than Warmth's 12, consistent with a transcendent-source
/// archetype (parity with Light's 15-tile FOV reach).
const SPIRITUAL_MAX_RADIUS: u32 = 15;
```

Spiritual branch inside `tick()` (placed after the Danger branch, before
the "remaining channels" loop):

```rust
let spiritual_idx = InfluenceChannel::Spiritual as usize;
let spiritual_dirty =
    std::mem::take(&mut resources.influence_grid.dirty_regions[spiritual_idx]);

if !spiritual_dirty.is_empty() {
    resources.influence_grid.clear_pending(InfluenceChannel::Spiritual);
    for region in &spiritual_dirty {
        let cx = (region.min_x + region.max_x) / 2;
        let cy = (region.min_y + region.max_y) / 2;
        let tile_grid = &resources.tile_grid;
        let blocking_cache = &resources.material_blocking_cache;
        let pending = resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Spiritual);
        propagate_bfs(
            tile_grid,
            blocking_cache,
            pending,
            (cx, cy),
            SPIRITUAL_INITIAL_INTENSITY,
            |i, _| i * SPIRITUAL_DECAY_PER_STEP,
            InfluenceChannel::Spiritual,
            SPIRITUAL_MAX_RADIUS,
        );
    }
} else {
    // Cold-tier persistence (parity with Warmth): copy current → pending so
    // the swap is a no-op for Spiritual on event-less ticks.
    let spiritual_snapshot =
        resources.influence_grid.current[spiritual_idx].clone();
    resources.influence_grid.pending[spiritual_idx]
        .copy_from_slice(&spiritual_snapshot);
}
```

Final loop change (skip Warmth, Light, Noise, Danger, AND Spiritual):

```rust
for ch in InfluenceChannel::all() {
    if *ch != InfluenceChannel::Warmth
        && *ch != InfluenceChannel::Light
        && *ch != InfluenceChannel::Noise
        && *ch != InfluenceChannel::Danger
        && *ch != InfluenceChannel::Spiritual
    {
        resources.influence_grid.clear_pending(*ch);
    }
}
```

### 3.2 Update IUS module docstring

Extend the `//!` header to mention T7.10.E alongside T7.10.A/B/C/D. Document
Spiritual's exponential-decay specifics (k=0.10, max_radius=15, aggregation
Max, density-derived wall blocking — parity with Warmth).

### 3.3 Add `rust/crates/sim-test/tests/harness_t7_10_e_spiritual_bfs_wiring.rs`

New harness file (11 assertions) mirroring T7.10.A's BFS structure:

- **E1 source_center_lit**: `sample(SX, SY, Spiritual) == 200` after 1 tick.
- **E2 exp_decay_one_step_discriminator**: `sample(SX+1, SY, Spiritual) ∈ [179, 183]`
  — `200 * 0.904_837 ≈ 180.97` with f32 rounding tolerance. Discriminator:
  Warmth k=0.15 would give ~172; Noise α=15 would give 185; Danger α=5 would
  give 195. Only k=0.10 exp falls in this window.
- **E3 bfs_distance_manhattan_discriminator**: `sample(SX+2, SY, Spiritual) ∈ [161, 165]`
  — two-step exp chain ≈163. Discriminator: Euclidean would interpolate
  differently; linear decays give different numerics.
- **E4 gradient_monotone**: strict `>` pairwise along cardinal axis for
  d=0..=14 (exponential decay is strictly decreasing within max_radius).
- **E5 boundary_at_max_radius**: `sample(SX+15, SY, Spiritual) ∈ [40, 50]`
  (d=15: 200 * 0.904_837^15 ≈ 44.6, still inside max_radius).
- **E6 max_radius_cutoff**: `sample(SX+16, SY, Spiritual) == 0` (d=16
  outside max_radius=15, never written). This is the cap discriminator
  vs Warmth (radius=12) and Light (radius=15 same but different shape).
- **E7 persistence_ten_ticks**: source still 200 after 1 stamp + 10
  event-less ticks (Cold-tier persistence branch).
- **E8 no_event_no_spiritual**: zero events → 3 sample positions all zero
  after 5 ticks.
- **E9 prior_channels_regression_guard**: `sample(SX, SY, Warmth) == 200`
  AND `sample(SX, SY, Light) == 200` AND `sample(SX, SY, Noise) == 200`
  AND `sample(SX, SY, Danger) == 200` (T7.10.A/B/C/D regression).
- **E10 dirty_regions_drained**: `dirty_regions[Spiritual].len() == 0`
  after IUS via `std::mem::take`.
- **E11 other_channels_remain_zero**: Beauty (still dispatch-shell) +
  FoodAroma, Social (not stamped) all sample to 0 at source.

### 3.4 Update legacy harness assertions

Five existing harness tests assert Spiritual is dispatch-shell or zero.
Update each to recognise T7.10.E:

- `harness_t7_10_a_warmth_wiring.rs` `_other_channels_remain_zero`: remove
  Spiritual from unstamped zero list; add `Spiritual == 200` regression
  guard at source center.
- `harness_t7_10_b_light_shadowcast_wiring.rs` `_other_channels_behavior`:
  replace `for ch in [Spiritual, Beauty]` with separate `Spiritual == 200`
  guard + `Beauty == 0` zero-list entry.
- `harness_t7_10_c_noise_linear_wiring.rs` `_other_channels_remain_zero`:
  add `Spiritual == 200` regression guard; remove Spiritual from
  unstamped/zero loop (Beauty stays).
- `harness_t7_10_d_danger_linear_wiring.rs` D10: replace `Spiritual/Beauty`
  loop with `Spiritual == 200` + `Beauty == 0` separate checks.
- `harness_phase2_substantial.rs` Assertion 5 + Assertion 15:
  add Spiritual drain (`len == 0`) AND Spiritual propagation discriminator
  (`sample(SX, SY) == 200`); remove Spiritual from the "remaining stamped"
  loop (Beauty stays).
- `harness_next_a_postverify.rs` Assertions 4/5/8/9/10/A6: rotate the
  "non-Warmth dispatch-shell representative" from Spiritual to Beauty
  (Spiritual now drained + propagating; Beauty is the only remaining
  dispatch-shell channel).
- `harness_phase2.rs` `harness_influence_update_clear_before_swap`: rotate
  Spiritual → Beauty in the test body (Spiritual now persists via Cold-tier
  branch, no longer a clean clear-before-swap discriminator).
- `sim-systems/tests/integration.rs` `harness_influence_update_clear_before_swap`:
  same rotation Spiritual → Beauty (mirrors the sim-test sibling).

Also update relevant module docstrings to reflect "5 channels wired,
Beauty remains the lone dispatch-shell channel".

### 3.5 Update SPACE channel cycle in `scripts/ui/world_renderer.gd`

Extend the SPACE-key channel cycle from 4-state (Warmth → Light → Noise →
Danger → Warmth) to 5-state (Warmth → Light → Noise → Danger → Spiritual →
Warmth):

- Add `const CHANNEL_SPIRITUAL := 6` (matches
  `InfluenceChannel::Spiritual as usize`).
- Extend the modulo cycle to use `% 5` with the next index lookup:
  `Warmth (0) → Light (3) → Noise (2) → Danger (4) → Spiritual (6) → Warmth (0)`.
- Console log emits `channel_name = "Spiritual"` on the Danger → Spiritual
  toggle and `channel_name = "Warmth"` on the Spiritual → Warmth toggle.

This is a minimal renderer change (one constant + one branch in the cycle
function). No new visualisation logic — the existing uniform/texture upload
path already handles arbitrary channel indices.

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

# 2. Targeted T7.10.E harness
cd rust && cargo test -p sim-test --test harness_t7_10_e_spiritual_bfs_wiring -- --nocapture

# 3. T7.10.A/B/C/D regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_d_danger_linear_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b1_space_toggle -- --nocapture

# 4. Phase 2 regression
cd rust && cargo test -p sim-test --test harness_phase2_ffi -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-test --test harness_next_a_postverify -- --nocapture
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
cd rust && cargo test -p sim-systems --test integration -- --nocapture
```

Expected: 11 new T7.10.E tests pass + all T7.10.A/B/C/D pass + all B1
toggle tests pass + all FFI/substantial/post-verify tests pass + 0 clippy
warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/update.rs` (one file edit)
  + `sim-test/tests/harness_t7_10_e_spiritual_bfs_wiring.rs` (new harness)
  + 8 legacy harness adjustments + 1 GDScript renderer constant + cycle
  extension.
- No FFI surface change (T7.7.B contract intact).
- No BSS change (Spiritual was already in `STAMPED_CHANNELS` since
  T7.10.A — only IUS propagation was missing).
- GDScript change is a minimal SPACE-cycle extension (one constant + cycle
  update), no scene/shader rewrite.
- Phase 0 design is already locked (k=0.10 exp, max_radius=15, density wall
  blocking); planning debate skipped per --quick.
- Visual Verify: renderer's SPACE cycle now exposes the Spiritual buffer;
  visual harness can confirm the Spiritual disc by pressing SPACE four
  times from default state.

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
- Press SPACE again → shows **Noise field** (square-ish linear-decay).
- Press SPACE a third time → shows **Danger field** (square,
  sight-radius cap=15, alpha=5 step gradient, pierces walls).
- Press SPACE a fourth time → shows **Spiritual disc** (gentler exponential
  falloff than Warmth, k=0.10, max_radius=15, wall-respecting).
- Press SPACE a fifth time → cycles back to Warmth disc.
- Console prints `channel_name = "Warmth" | "Light" | "Noise" | "Danger" | "Spiritual"`
  on each toggle.

**Background invariant** (not user-visible, asserted by harness only):
- `current[Spiritual]` contains the exponential-decay BFS field after the stamp.
- Source tile = 200; d=1 neighbour ≈ 181; d=15 boundary ≈ 44; d=16 = 0.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.E is **single-channel** scope (Spiritual only). Honest limitations:

1. **No ritual / shrine events yet**: Phase 2 has no rituals or shrines, so
   the only Spiritual source is the static BuildingPlacedEvent stamp.
   Phase 3+ will add per-shrine-class intensities, ritual amplification,
   and per-event transient peaks. The current Cold-tier persistence branch
   matches V7 Phase 2's "single static source" reality.
2. **Exponential decay k=0.10, max_radius=15, density wall blocking**: locked
   model. No occlusion refinement, no per-faith-tradition intensity
   modulation, no temporal modulation (ritual timing).
3. **No structure-removal handling**: If a shrine building is removed, the
   Spiritual field persists indefinitely (no negative dirty_regions /
   removal events). Out of scope for T7.10.E.
4. **Remaining channels stay dispatch-shell**: Beauty (stamped, propagation
   not wired — T7.10.F target), FoodAroma/Social (not stamped, deferred
   until agent events arrive).
5. **All shrines have identical reach**: BSS uses `event.position` as
   Chebyshev box center; region center = building center exactly; intensity
   200 is hardcoded. Per-structure-class intensity differentiation would
   need refinement in a later phase.

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any scene / shader change (only minimal world_renderer.gd cycle extension)
- Ritual / shrine-class events (Phase 3+)
- Building demolition / negative events (T7.10.F+)
- Beauty / FoodAroma / Social propagation (T7.10.F+)
- Per-faith-tradition intensity differentiation, ritual temporal modulation,
  or occlusion refinement (Phase 0 LOCKED: k=0.10 exp, max_radius=15,
  density wall blocking — Warmth-parity)
- Performance optimization beyond what `propagate_bfs` already provides
