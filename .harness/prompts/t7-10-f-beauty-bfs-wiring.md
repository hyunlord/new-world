# T7.10.F — Beauty channel BFS exponential-decay wiring (sixth-channel Phase 2 escape, FINAL stamped channel)

> Lane: `--quick` (sim-systems + sim-test + minimal GDScript renderer toggle)
> Scope: SINGLE channel (Beauty only). Warmth (T7.10.A), Light (T7.10.B),
> Noise (T7.10.C), Danger (T7.10.D), and Spiritual (T7.10.E) already wired.
> Remaining 2 unstamped channels (FoodAroma, Social) stay dispatch-shell.
> Governance: v3.3.16. Visual: gentler-falloff Beauty disc at the stamped
> building's center via exponential-decay BFS with max_radius=15 tiles,
> k=0.12 per step, wall-aware (density-derived blocking — aesthetic
> influence respects physical barriers like Warmth/Spiritual do).

---

## Section 1 — Implementation Intent

T7.10.A/B/C/D/E escaped the dispatch shell for Warmth (BFS exponential
k=0.15), Light (shadowcast Euclidean), Noise (linear-decay α=15 + density
wall blocking), Danger (linear-decay α=5, sight-radius cap=15, no walls),
and Spiritual (BFS exponential k=0.10). T7.10.F is the **sixth and FINAL
stamped channel to escape** — Beauty exponential BFS wired end-to-end via
the existing `propagate_bfs` primitive at `propagate.rs:75` (same primitive
Warmth/Spiritual use), with k=0.12 (between Warmth's 0.15 and Spiritual's
0.10 — aesthetic influence carries slightly further than thermal heat but
less than ritual) and max_radius=15 (Spiritual parity).

After this commit, `on_building_placed(32, 32, 12)` produces a Warmth disc
(T7.10.A regression), a Light disc (T7.10.B regression), a Noise field
(T7.10.C regression), a Danger field (T7.10.D regression), a Spiritual
disc (T7.10.E regression), **and** a Beauty disc (BFS exponential k=0.12,
max_radius=15) in `current[Beauty]` after one tick.

Renderer change: extend the SPACE-key channel cycle in `world_renderer.gd`
from 5-state (Warmth → Light → Noise → Danger → Spiritual → Warmth) to
6-state (Warmth → Light → Noise → Danger → Spiritual → Beauty → Warmth)
so the operator can visually confirm the new buffer.

Remaining 2 unstamped channels (FoodAroma, Social) stay dispatch-shell —
deferred until agent-driven sources arrive in later V7 phases.

**Milestone**: With T7.10.F all 6 stamped channels (Warmth, Light, Noise,
Danger, Spiritual, Beauty) escape the dispatch shell. The dispatch-shell
architecture remains only for the 2 unstamped channels (FoodAroma, Social)
that lack BSS sources at the Phase 2 stage.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Beauty enum index | `channel.rs` | `InfluenceChannel::Beauty as usize == 7` |
| Beauty aggregation | `channel.rs` | `AggKind::Max` (parity with Warmth/Light/Noise/Danger/Spiritual) |
| Beauty tier | `channel.rs` | Cold (parity with Warmth/Spiritual — aesthetic sources are persistent structural placements) |
| Beauty max_radius | LOCKED design | **15 tiles** (Spiritual parity — gentler reach than Warmth=12) |
| Beauty initial intensity | LOCKED design | **200** (all-channel parity) |
| Exponential decay constant k | `channel.rs:74` Phase 0 spec | **0.12** (between Warmth's 0.15 and Spiritual's 0.10) |
| Pre-computed decay multiplier | `exp(-0.12)` | **0.886_920** (Rust does not const-eval `f32::exp` in stable) |
| Wall blocking | `propagate_bfs` blocking_cache | density-derived (same as Warmth/Spiritual — aesthetic influence respects walls) |
| `propagate_bfs` signature | `propagate.rs:75` | `(tile_grid, blocking_cache, &mut [u8], (u32,u32), u8, decay_fn, channel, max_radius)` |
| BSS STAMPED_CHANNELS (unchanged post-F) | `building_stamp.rs` | still **6 channels**: Warmth, Light, Noise, Danger, Spiritual, Beauty (Beauty was already in the list since T7.10.A; only the propagation branch was missing) |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs` | 100 |

**Source center invariant** (parallels Warmth/Spiritual): `propagate_bfs`
applies `apply_agg(Max)` at the source so `sample(SX, SY, Beauty) == 200`
regardless of neighbouring geometry — the source cannot decay below its
seed within a single pass.

**BFS distance invariant**: Exponential decay along 4-neighbor frontier
expansion. One-step neighbour value = `200 * 0.886_920 ≈ 177.4`
(integer arithmetic via `(i as f32 * BEAUTY_DECAY_PER_STEP) as u8`
truncating).
- d=1: `200 * 0.886_920 ≈ 177.384 → 177` (f32→u8 truncation)
- d=2: `177 * 0.886_920 ≈ 156.985 → 156` (chain via intermediate i)
- d=15: `200 * 0.886_920^15 ≈ 200 * 0.1656 ≈ 33.1 → 33` (window [28, 38])
- d=16: 0 (frontier never expands past max_radius=15)

**Discriminator vs Warmth/Spiritual**: Warmth's k=0.15 yields d=1 ≈ 172
(multiplier 0.8607); Spiritual's k=0.10 yields d=1 ≈ 181 (multiplier
0.9048); Beauty's k=0.12 yields d=1 ≈ 177 (multiplier 0.8869). The
~4-5 unit gaps at d=1 are clean single-step discriminators. At
max_radius (d=15), Warmth would already be at 0 (radius=12 cuts it off);
Spiritual at d=15 ≈ 44 (radius=15 still active); Beauty at d=15 ≈ 33.

---

## Section 3 — What to build

### 3.1 `rust/crates/sim-systems/src/runtime/influence/update.rs`

Mirror the T7.10.A/E Warmth/Spiritual branch pattern for Beauty (sixth
branch). New flow:

```text
tick():
  1. Warmth branch    (T7.10.A) — unchanged
  2. Light branch     (T7.10.B) — unchanged
  3. Noise branch     (T7.10.C) — unchanged
  4. Danger branch    (T7.10.D) — unchanged
  5. Spiritual branch (T7.10.E) — unchanged
  6. Beauty branch    (T7.10.F) — NEW:
       beauty_dirty = drain(influence_grid.dirty_regions[Beauty])
       if beauty_dirty non-empty:
         clear_pending(Beauty)
         for each region:
           cx = (region.min_x + region.max_x) / 2
           cy = (region.min_y + region.max_y) / 2
           propagate_bfs(&tile_grid, &blocking_cache, pending_buf_mut(Beauty),
                         (cx, cy), 200, |i, _| i * 0.886_920,
                         Beauty, 15)
       else:
         pending[Beauty].copy_from_slice(&current[Beauty])  // Cold-tier persistence
  7. For each channel ∉ {Warmth, Light, Noise, Danger, Spiritual, Beauty}: clear_pending(ch)
  8. swap()
```

**Concrete additions** (added to `update.rs`):

```rust
/// `exp(-0.12)` — beauty exponential decay per BFS step.
///
/// Pre-computed: Rust does not const-eval `f32::exp` in stable.
/// k = 0.12 (channel.rs:74 Phase 0 spec — between Warmth's 0.15 and
/// Spiritual's 0.10).
const BEAUTY_DECAY_PER_STEP: f32 = 0.886_920;

/// Initial intensity at the Beauty source tile.
const BEAUTY_INITIAL_INTENSITY: u8 = 200;

/// Beauty propagation radius in tiles.
///
/// 15 = Spiritual parity (Cold-tier reach archetype, longer than Warmth's 12).
const BEAUTY_MAX_RADIUS: u32 = 15;
```

Beauty branch inside `tick()` (placed after the Spiritual branch, before
the "remaining channels" loop):

```rust
let beauty_idx = InfluenceChannel::Beauty as usize;
let beauty_dirty =
    std::mem::take(&mut resources.influence_grid.dirty_regions[beauty_idx]);

if !beauty_dirty.is_empty() {
    resources.influence_grid.clear_pending(InfluenceChannel::Beauty);
    for region in &beauty_dirty {
        let cx = (region.min_x + region.max_x) / 2;
        let cy = (region.min_y + region.max_y) / 2;
        let tile_grid = &resources.tile_grid;
        let blocking_cache = &resources.material_blocking_cache;
        let pending = resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Beauty);
        propagate_bfs(
            tile_grid,
            blocking_cache,
            pending,
            (cx, cy),
            BEAUTY_INITIAL_INTENSITY,
            |i, _| i * BEAUTY_DECAY_PER_STEP,
            InfluenceChannel::Beauty,
            BEAUTY_MAX_RADIUS,
        );
    }
} else {
    // Cold-tier persistence (parity with Warmth/Spiritual): copy current → pending so
    // the swap is a no-op for Beauty on event-less ticks.
    let beauty_snapshot =
        resources.influence_grid.current[beauty_idx].clone();
    resources.influence_grid.pending[beauty_idx]
        .copy_from_slice(&beauty_snapshot);
}
```

Final loop change (skip Warmth, Light, Noise, Danger, Spiritual, AND Beauty):

```rust
for ch in InfluenceChannel::all() {
    if *ch != InfluenceChannel::Warmth
        && *ch != InfluenceChannel::Light
        && *ch != InfluenceChannel::Noise
        && *ch != InfluenceChannel::Danger
        && *ch != InfluenceChannel::Spiritual
        && *ch != InfluenceChannel::Beauty
    {
        resources.influence_grid.clear_pending(*ch);
    }
}
```

### 3.2 Update IUS module docstring

Extend the `//!` header to mention T7.10.F alongside T7.10.A/B/C/D/E.
Document Beauty's exponential-decay specifics (k=0.12, max_radius=15,
aggregation Max, density-derived wall blocking — parity with
Warmth/Spiritual).

### 3.3 Add `rust/crates/sim-test/tests/harness_t7_10_f_beauty_bfs_wiring.rs`

New harness file (11 assertions) mirroring T7.10.E's BFS structure:

- **F1 source_center_lit**: `sample(SX, SY, Beauty) == 200` after 1 tick.
- **F2 exp_decay_one_step_discriminator**: `sample(SX+1, SY, Beauty) ∈ [175, 180]`
  — `200 * 0.886_920 ≈ 177.4` with f32 rounding tolerance. Discriminator:
  Warmth k=0.15 would give ~172; Spiritual k=0.10 would give ~181; Noise
  α=15 would give 185; Danger α=5 would give 195. Only k=0.12 exp falls
  in this window.
- **F3 bfs_distance_manhattan_discriminator**: `sample(SX+2, SY, Beauty) ∈ [155, 160]`
  — two-step exp chain ≈157. Discriminator vs other decays.
- **F4 gradient_monotone**: strict `>` pairwise along cardinal axis for
  d=0..=14 (exponential decay is strictly decreasing within max_radius).
- **F5 boundary_at_max_radius**: `sample(SX+15, SY, Beauty) ∈ [28, 38]`
  (d=15: 200 * 0.886_920^15 ≈ 33, still inside max_radius).
- **F6 max_radius_cutoff**: `sample(SX+16, SY, Beauty) == 0` (d=16
  outside max_radius=15, never written).
- **F7 persistence_ten_ticks**: source still 200 after 1 stamp + 10
  event-less ticks (Cold-tier persistence branch).
- **F8 no_event_no_beauty**: zero events → 3 sample positions all zero
  after 5 ticks.
- **F9 prior_channels_regression_guard**: `sample(SX, SY, Warmth) == 200`
  AND `sample(SX, SY, Light) == 200` AND `sample(SX, SY, Noise) == 200`
  AND `sample(SX, SY, Danger) == 200` AND `sample(SX, SY, Spiritual) == 200`
  (T7.10.A/B/C/D/E regression).
- **F10 dirty_regions_drained**: `dirty_regions[Beauty].len() == 0`
  after IUS via `std::mem::take`.
- **F11 unstamped_channels_remain_zero**: FoodAroma + Social (not stamped)
  all sample to 0 at source.

### 3.4 Update legacy harness assertions

Existing harness tests assert Beauty is dispatch-shell or zero. Update
each to recognise T7.10.F:

- `harness_t7_10_e_spiritual_bfs_wiring.rs` E11 `_other_channels_remain_zero`:
  replace `Beauty == 0` with `Beauty == 200` regression guard at source center.
- `harness_phase2_substantial.rs` Assertion 5 + Assertion 15:
  add Beauty drain (`len == 0`) AND Beauty propagation discriminator
  (`sample(SX, SY) == 200`); remove Beauty from the "remaining stamped"
  loop. Post-T7.10.F all 6 stamped channels are drained.
- `harness_next_a_postverify.rs` Assertions 4/5/6/8/9/10: rewrite tests
  that previously checked `dirty_regions[Beauty].len() == N` to check
  `current[Beauty] == 200` at source positions (proves FFI→BSS→IUS→swap
  pipeline). Tests that checked `pending[Beauty] cleared each tick` rotate
  to `pending[FoodAroma]` (only remaining unstamped dispatch-shell channel).
- `harness_phase2.rs` `harness_influence_update_clear_before_swap`: rotate
  Beauty → FoodAroma in the test body (Beauty now persists via Cold-tier
  branch, no longer a clean clear-before-swap discriminator).
- `sim-systems/tests/integration.rs` `harness_influence_update_clear_before_swap`:
  same rotation Beauty → FoodAroma (mirrors the sim-test sibling).

Note: `harness_phase2_ffi.rs` Beauty assertions use `bss_tick` (BSS-only,
no IUS), so dirty_regions checks remain valid — BSS still marks all 6
stamped channels including Beauty.

Also update relevant module docstrings to reflect "6 channels wired,
only FoodAroma/Social remain dispatch-shell (deferred until agent events)".

### 3.5 Update SPACE channel cycle in `scripts/ui/world_renderer.gd`

Extend the SPACE-key channel cycle from 5-state (Warmth → Light → Noise →
Danger → Spiritual → Warmth) to 6-state (Warmth → Light → Noise → Danger
→ Spiritual → Beauty → Warmth):

- Add `const CHANNEL_BEAUTY := 7` (matches
  `InfluenceChannel::Beauty as usize`).
- Extend the modulo cycle to use `% 6` with the next index lookup:
  `Warmth (0) → Light (3) → Noise (2) → Danger (4) → Spiritual (6) →
  Beauty (7) → Warmth (0)`.
- Console log emits `channel_name = "Beauty"` on the Spiritual → Beauty
  toggle and `channel_name = "Warmth"` on the Beauty → Warmth toggle.

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

# 2. Targeted T7.10.F harness
cd rust && cargo test -p sim-test --test harness_t7_10_f_beauty_bfs_wiring -- --nocapture

# 3. T7.10.A/B/C/D/E regression
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_d_danger_linear_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_e_spiritual_bfs_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_b1_space_toggle -- --nocapture

# 4. Phase 2 regression
cd rust && cargo test -p sim-test --test harness_phase2_ffi -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-test --test harness_next_a_postverify -- --nocapture
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
cd rust && cargo test -p sim-systems --test integration -- --nocapture
```

Expected: 11 new T7.10.F tests pass + all T7.10.A/B/C/D/E pass + all B1
toggle tests pass + all FFI/substantial/post-verify tests pass + 0 clippy
warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/update.rs` (one file edit)
  + `sim-test/tests/harness_t7_10_f_beauty_bfs_wiring.rs` (new harness)
  + legacy harness adjustments + 1 GDScript renderer constant + cycle
  extension.
- No FFI surface change (T7.7.B contract intact).
- No BSS change (Beauty was already in `STAMPED_CHANNELS` since
  T7.10.A — only IUS propagation was missing).
- GDScript change is a minimal SPACE-cycle extension (one constant + cycle
  update), no scene/shader rewrite.
- Phase 0 design is already locked (k=0.12 exp from channel.rs:74,
  max_radius=15, density wall blocking); planning debate skipped per
  --quick.
- Visual Verify: renderer's SPACE cycle now exposes the Beauty buffer;
  visual harness can confirm the Beauty disc by pressing SPACE five
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
- Press SPACE a fourth time → shows **Spiritual disc** (gentler
  exponential falloff, k=0.10, max_radius=15, wall-respecting).
- Press SPACE a fifth time → shows **Beauty disc** (intermediate
  exponential falloff, k=0.12, max_radius=15, wall-respecting).
- Press SPACE a sixth time → cycles back to Warmth disc.
- Console prints `channel_name = "Warmth" | "Light" | "Noise" | "Danger" | "Spiritual" | "Beauty"`
  on each toggle.

**Background invariant** (not user-visible, asserted by harness only):
- `current[Beauty]` contains the exponential-decay BFS field after the stamp.
- Source tile = 200; d=1 neighbour ≈ 177; d=15 boundary ≈ 33; d=16 = 0.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.F is **single-channel** scope (Beauty only). Honest limitations:

1. **No aesthetic-source variation yet**: Phase 2 has no per-structure
   beauty differentiation, so the only Beauty source is the static
   BuildingPlacedEvent stamp at uniform intensity 200.
   Phase 3+ will add per-structure-class intensities (e.g., monument vs hovel),
   beauty amplification via décor entities, and per-event transient peaks.
   The current Cold-tier persistence branch matches V7 Phase 2's
   "single static source" reality.
2. **Exponential decay k=0.12, max_radius=15, density wall blocking**:
   locked model. No occlusion refinement, no per-aesthetic-style intensity
   modulation, no temporal modulation (e.g., seasonal beauty).
3. **No structure-removal handling**: If a beautiful building is removed,
   the Beauty field persists indefinitely (no negative dirty_regions /
   removal events). Out of scope for T7.10.F.
4. **Remaining unstamped channels stay dispatch-shell**: FoodAroma and
   Social (not stamped — no BSS sources at Phase 2 stage, deferred until
   agent events arrive in later V7 phases).
5. **All buildings have identical beauty intensity**: BSS uses
   `event.position` as Chebyshev box center; region center = building
   center exactly; intensity 200 is hardcoded. Per-structure-class beauty
   differentiation would need refinement in a later phase.

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any scene / shader change (only minimal world_renderer.gd cycle extension)
- Per-aesthetic-class events or décor entities (Phase 3+)
- Building demolition / negative events (later V7 phase)
- FoodAroma / Social propagation (deferred until agent events arrive)
- Per-aesthetic-style intensity differentiation, beauty temporal modulation,
  or occlusion refinement (Phase 0 LOCKED: k=0.12 exp from channel.rs:74,
  max_radius=15, density wall blocking — Spiritual-parity reach)
- Performance optimization beyond what `propagate_bfs` already provides
- Resolving T7.10.E Spiritual drift (Phase 0 doc k=0.08 vs IUS k=0.10
  vs channel.rs spec) — deferred to post-T7.10.F governance review
