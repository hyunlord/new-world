# T7.10.A — Warmth channel BFS wiring (single-channel Phase 2 escape)

> Lane: `--quick` (sim-systems + sim-test, no FFI / scene changes)
> Scope: SINGLE channel (Warmth only). Other 7 channels remain dispatch-shell.
> Governance: v3.3.16. First visual milestone — radial warmth disc at the
> stamped building's center.

---

## Section 1 — Implementation Intent

T7.7.B locked the FFI surface; T7.9.A wired the renderer scaffold; T7.9.B
uploads `get_influence_overlay(Warmth)` to a 1024×1024 Sprite2D every
frame. Result so far: uniformly black square — Phase 2 IUS is still the
**dispatch shell** documented in `update.rs:6-9`:

```text
Phase 2 land (T7.6) is a *dispatch shell*: it clears every pending buffer
and swaps double-buffers each tick. Actual source iteration (BFS /
shadowcast / linear propagation) lands together with BuildingStampSystem
plumbing in later phases — the shell guarantees a deterministic
zero-state baseline regardless of registration order.
```

T7.10.A is the **first channel to escape the shell** — Warmth BFS
propagation wired end-to-end. After this commit, `on_building_placed(32,
32, 12)` produces a visible radial warmth disc on the rendered sprite
(centered bright pixels fading to black at ~12 tiles).

Other 7 channels stay dispatch-shell — they will be wired in T7.10.B…F.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Warmth aggregation | `channel.rs:103` | `AggKind::Additive` |
| Warmth decay kind | `channel.rs:32` comment + `channel.rs:118` tier | Exponential, k=0.15, Cold tier |
| Warmth max_radius | Phase 0 + `channel.rs:343` test fixture | 12 |
| BSS priority | `building_stamp.rs:50` | 90 (runs BEFORE IUS) |
| IUS priority | `update.rs:41` | 100 |
| BSS dirty_region | `building_stamp.rs:77-86` | Chebyshev box clamped to grid |
| `propagate_bfs` signature | `propagate.rs:75-86` | `(tile_grid, blocking_cache, &mut [u8], (u32,u32), u8, decay_fn, channel, max_radius)` |
| SimResources fields | `engine/src/lib.rs:85,94,122,126,129` | `tile_grid`, `influence_grid`, `material_blocking_cache` |
| `InfluenceGrid::pending` | `grid.rs:49` | `pub pending: [Vec<u8>; 8]` — direct array access allowed |
| `InfluenceGrid::current` | `grid.rs:47` | `pub current: [Vec<u8>; 8]` — direct array access allowed |
| `DirtyRegion` shape | `grid.rs:13-22` | `(min_x, min_y, max_x, max_y)` inclusive |

---

## Section 3 — What to build

### 3.1 Modify `rust/crates/sim-systems/src/runtime/influence/update.rs`

Replace the dispatch-shell `tick()` body. New flow:

```text
tick():
  1. warmth_dirty = drain(influence_grid.dirty_regions[Warmth])
  2. if warmth_dirty non-empty:
       a. clear_pending(Warmth)
       b. for each region in warmth_dirty:
            cx = (region.min_x + region.max_x) / 2
            cy = (region.min_y + region.max_y) / 2
            propagate_bfs(
              &tile_grid,
              &material_blocking_cache,
              pending_buf_mut(Warmth),
              (cx, cy),
              200,                              // initial_intensity
              |i, _| i * WARMTH_DECAY_PER_STEP, // exp(-0.15) ≈ 0.8607
              InfluenceChannel::Warmth,
              12,                               // max_radius
            )
     else:
       # Persistence — Cold-tier semantics. Without this, the swap below
       # would zero current[Warmth] on every event-less tick, causing the
       # rendered disc to flicker.
       pending[Warmth].copy_from_slice(&current[Warmth])
  3. For each channel ≠ Warmth: clear_pending(ch)
       (other 7 channels remain dispatch-shell zero baseline)
  4. swap()  // all 8 channels swap together
```

Concrete syntax:

```rust
//! `InfluenceUpdateSystem` — priority 100, every tick.
//!
//! T7.10.A land: Warmth channel escapes the Phase 2 dispatch shell.
//! On any tick where BuildingStampSystem (priority 90) populated
//! `dirty_regions[Warmth]`, IUS runs `propagate_bfs` from each region's
//! center into `pending[Warmth]` before the swap. Persistence across
//! event-less ticks is preserved by copying `current[Warmth]` into
//! `pending[Warmth]` so the swap is a no-op for Warmth (Cold-tier
//! event-driven semantics, per Phase 0 Section 2.6).
//!
//! Other 7 channels remain dispatch-shell (clear pending + swap) and
//! will be wired in T7.10.B..F.
//!
//! Decay: exponential k = 0.15 (Phase 0 Section 2.3.1, channel.rs:32).
//! Max radius: 12 (Phase 0 Section 2.3.1 + ChannelDef fixture).
//! Initial intensity: 200 (matches existing propagate.rs test conventions).
//! Aggregation: Additive (InfluenceChannel::Warmth.aggregation()).

use hecs::World;
use sim_core::influence::{propagate_bfs, InfluenceChannel};
use sim_engine::{RuntimeSystem, SimResources};

/// exp(-0.15) — pre-computed; Rust does not const-eval `f32::exp`.
const WARMTH_DECAY_PER_STEP: f32 = 0.860_708;

/// Initial intensity stamped at the building's center.
const WARMTH_INITIAL_INTENSITY: u8 = 200;

/// Channel propagation radius (Phase 0 Section 2.3.1).
const WARMTH_MAX_RADIUS: u32 = 12;

pub struct InfluenceUpdateSystem;

impl InfluenceUpdateSystem {
    pub fn new() -> Self { Self }
}

impl Default for InfluenceUpdateSystem {
    fn default() -> Self { Self::new() }
}

impl RuntimeSystem for InfluenceUpdateSystem {
    fn name(&self) -> &str { "InfluenceUpdateSystem" }
    fn priority(&self) -> u32 { 100 }
    fn tick_interval(&self) -> u64 { 1 }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let warmth_idx = InfluenceChannel::Warmth as usize;

        // Drain Warmth dirty regions (BSS at priority 90 already wrote them).
        let warmth_dirty = std::mem::take(
            &mut resources.influence_grid.dirty_regions[warmth_idx]
        );

        if !warmth_dirty.is_empty() {
            // Fresh pass: clear pending then propagate from each source.
            resources.influence_grid.clear_pending(InfluenceChannel::Warmth);
            for region in &warmth_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;
                propagate_bfs(
                    &resources.tile_grid,
                    &resources.material_blocking_cache,
                    resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth),
                    (cx, cy),
                    WARMTH_INITIAL_INTENSITY,
                    |i, _| i * WARMTH_DECAY_PER_STEP,
                    InfluenceChannel::Warmth,
                    WARMTH_MAX_RADIUS,
                );
            }
        } else {
            // Cold-tier persistence: preserve current[Warmth] across the swap.
            let (warmth_ch_current, warmth_ch_pending) = {
                let cur_ptr = resources.influence_grid.current[warmth_idx].as_ptr();
                let len = resources.influence_grid.current[warmth_idx].len();
                let snapshot = unsafe { std::slice::from_raw_parts(cur_ptr, len) }.to_vec();
                (snapshot, resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth))
            };
            warmth_ch_pending.copy_from_slice(&warmth_ch_current);
        }

        // Other 7 channels: dispatch-shell baseline (clear + swap → zero).
        for ch in InfluenceChannel::all() {
            if *ch != InfluenceChannel::Warmth {
                resources.influence_grid.clear_pending(*ch);
            }
        }

        resources.influence_grid.swap();
    }
}
```

**Borrow-checker note**: the `pending_buf_mut(Warmth)` call mutably
borrows `resources.influence_grid` while `&resources.tile_grid` /
`&resources.material_blocking_cache` borrow disjoint SimResources
fields. Rust 2021 NLL handles disjoint-field borrows correctly. If the
borrow checker complains in practice, split the borrow with a local
destructure or sequence the copy via a temporary `Vec<u8>` snapshot.

The `unsafe` slice block in the persistence path is one acceptable way
to avoid simultaneous `&` and `&mut` of `influence_grid.current[idx]`
and `pending[idx]`. Equivalent safe alternative: `to_vec()` the current
buffer first, then write via `pending_buf_mut`. Pick whichever the
borrow checker accepts. Both produce identical observable behavior.

### 3.2 Update existing IUS unit tests

`update.rs` tests at lines 63-122:

- **`metadata`** (line 73): unchanged — still expects name/priority/interval.
- **`tick_does_not_panic_on_empty_world`** (line 81): unchanged — empty
  engine, no dirty regions, hits the persistence (else) branch which
  copies zero→zero.
- **`baseline_remains_zero_after_ticks`** (line 92): unchanged — same
  empty-engine path, all channels stay zero.
- **`pending_write_cleared_before_swap`** (line 106): **must be updated**
  to match T7.10.A semantics:
  - The test manually writes `pending[Warmth][i] = 200` and expects
    `current[Warmth][...] == 0` after one tick.
  - Under T7.10.A: manual pending write does NOT add a dirty_region.
    → `warmth_dirty` is empty → persistence branch copies current
    (zero) → pending. Then swap. So `current[Warmth][...] == 0`. ✅
    Test still passes as written. Add a comment explaining why.

### 3.2.5 Relax dispatch-shell asserts in `rust/crates/sim-test/tests/harness_phase2_substantial.rs`

The 15-test `harness_phase2_substantial.rs` suite was written under the pre-T7.10
dispatch-shell invariant: **`current[Warmth] == 0 everywhere, dirty_regions
preserved after IUS tick`**. T7.10.A escapes that invariant for Warmth only:

- `current[Warmth]` is now non-zero inside the propagation radius after a tick.
- `dirty_regions[Warmth]` is drained by IUS (via `std::mem::take`) before propagation.
- Other 7 channels (Spiritual / Beauty / Light / Danger / Knowledge / Social / Fear) **still observe full dispatch-shell semantics**.

**Required relaxation pattern (apply per failing test):**

1. **Tests asserting `current[Warmth] == 0`** → assert on a non-Warmth stamped
   channel instead (Spiritual / Beauty / Light), OR remove the Warmth-specific
   assert with a `// T7.10.A: Warmth propagates; see harness_t7_10_a_*` comment.
2. **Tests asserting `dirty_regions[Warmth].len() == N` after IUS tick** →
   change to assert **before** the IUS tick (i.e. after BSS only, by registering
   only BSS), OR switch the asserted channel to a non-Warmth stamped channel
   (Spiritual stays dispatch-shell, dirty_regions preserved).
3. **Tests asserting `pending[Warmth] == 0`** → same as (1) — switch channel
   or remove the Warmth-specific check.
4. **Counting tests** (3-event count, 100-event burst, corner stamps, OOB guard):
   verify the count on a non-Warmth stamped channel (Spiritual) — the count
   semantic is identical for shell channels, just verified on a different
   channel index.
5. **`1k_agents_*` and `idle_4380_*`**: these are baseline performance smoke
   tests — relax the "all channels zero" to "non-Warmth channels zero" with
   a comment.

**Critical**: do **not** delete tests. Relax them surgically — the goal is to
preserve coverage of the dispatch-shell invariant for the 7 channels that
**still observe it**, while letting Warmth's new propagation behavior pass.
Every relaxed assert must have a one-line `// T7.10.A: ...` comment naming
the reason.

**Verification**: after relaxation, `cargo test -p sim-test --test
harness_phase2_substantial` must show **15/15 pass** with the same test names
(no test renames, no deletions).

### 3.3 Add `rust/crates/sim-test/tests/harness_t7_10_a_warmth_wiring.rs`

New harness test file (4-6 assertions):

```rust
//! T7.10.A — Warmth channel BFS wiring (single-channel Phase 2 escape).
//!
//! After T7.6 dispatch shell + T7.9.A scaffold + T7.9.B render, the
//! Warmth channel is the first to escape the shell. A BuildingPlacedEvent
//! at (32, 32) with radius 12 must produce a radial warmth disc in
//! `current[Warmth]` centered at (32, 32) after one tick.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture`

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;
const SX: u32 = 32;
const SY: u32 = 32;

fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

fn place_warmth_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── A1: source center reaches initial intensity ──────────────────────────────

/// Type A: current[Warmth][(SX,SY)] == 200 after first tick post-event.
#[test]
fn harness_t7_10_a_source_center_lit() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(v, 200, "source center must equal WARMTH_INITIAL_INTENSITY");
}

// ── A2: radial decay (1 step) ────────────────────────────────────────────────

/// Type A: current[Warmth][(SX, SY+1)] ≈ 200 * 0.8607 ≈ 172 (±2).
#[test]
fn harness_t7_10_a_radial_decay_one_step() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    let v = e.resources.influence_grid.sample(SX, SY + 1, InfluenceChannel::Warmth);
    assert!(
        (170..=174).contains(&v),
        "1-step neighbor must decay to ~172 (200*exp(-0.15)); got {v}"
    );
}

// ── A3: max_radius cutoff ────────────────────────────────────────────────────

/// Type A: current[Warmth][(SX+13, SY)] == 0 (beyond radius 12).
#[test]
fn harness_t7_10_a_max_radius_cutoff() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    let v = e.resources.influence_grid.sample(SX + 13, SY, InfluenceChannel::Warmth);
    assert_eq!(v, 0, "tile beyond max_radius=12 must be 0");
}

// ── A4: persistence across event-less ticks ──────────────────────────────────

/// Type A: after one stamp + 10 event-less ticks, source still lit.
/// Verifies Cold-tier persistence branch in IUS.
#[test]
fn harness_t7_10_a_persistence_across_ticks() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick(); // first tick — propagates
    for _ in 0..10 { e.tick(); } // event-less ticks — persistence branch
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(v, 200, "source must persist across 10 event-less ticks");
}

// ── A5: dispatch-shell preserved for non-Warmth channels ─────────────────────

/// Type A: Spiritual/Beauty/Light still emit dirty_regions from BSS but
/// remain zero in current (no propagation wired yet).
#[test]
fn harness_t7_10_a_other_channels_remain_zero() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    for ch in [
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero at T7.10.A (only Warmth wired)");
    }
}

// ── A6: source-only stamp survives one tick on empty engine ──────────────────

/// Type A: zero events → no warmth visible anywhere.
#[test]
fn harness_t7_10_a_no_event_no_warmth() {
    let mut e = fresh_engine();
    for _ in 0..5 { e.tick(); }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Warmth);
        assert_eq!(v, 0, "no events ⇒ Warmth stays zero at ({x},{y})");
    }
}
```

---

## Section 4 — Locale

No new locale keys. UI is unchanged; visual change is sprite pixel
content only.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted harness
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture

# 3. Phase 2 regression
cd rust && cargo test -p sim-systems runtime::influence -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2 -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
```

Expected: 6 new tests pass + existing 277 workspace tests still pass + 0
clippy warnings.

---

## Section 6 — Lane

`--quick`. Rationale:
- Sub-area: `sim-systems/src/runtime/influence/update.rs` (single file edit)
  + `sim-test/tests/harness_t7_10_a_warmth_wiring.rs` (new harness)
- No FFI surface change (T7.7.B contract intact)
- No GDScript / scene / shader changes
- Phase 0 design is already locked; planning debate skipped per --quick
- Visual Verify: the T7.9.B render pipeline already uploads
  `get_influence_overlay(Warmth)` every frame — once Warmth propagates,
  the disc appears automatically. No GDScript edits needed.

---

## Section 7 — In-game verification (post-merge)

After `cargo build -p sim-bridge --release` + Godot 4.6 editor restart,
press F6 on `scenes/main.tscn`:

**Expected console output** (unchanged from T7.9.B):
```
Initialize godot-rust (API v4.5.stable.official, runtime v4.6.stable.official)
WorldRenderer ready (T7.9.B render mechanism)
```

**Expected visual** (changed from T7.9.B's uniform black):
- 1024×1024 sprite (same as T7.9.B)
- Bright spot near pixel (512, 512) — sprite center corresponds to tile
  (32, 32) when the sprite mapping is 1024 pixels / 64 tiles = 16
  pixels per tile.
- Radial gradient: bright center → dim edges → black beyond ~12 tile
  radius (~192 pixel radius from center)
- Disc PERSISTS across frames (does not flicker) — Cold-tier
  event-driven semantics verified.

The seed building is placed by `on_building_placed` in
`world_node.rs:on_ready` (T7.7.B existing scaffold) — no new FFI calls
needed.

---

## Section 8 — Phase 2 disclosure (axiom #1 honesty)

T7.10.A is **single-channel** scope. Honest current limitations:

1. **Single source only**: Multi-building Warmth fields are NOT yet
   correct. The current IUS clears pending[Warmth] before every BFS
   pass, so a second building's stamp overwrites the first instead of
   accumulating. Multi-source persistence + Additive aggregation lands
   in T7.10.B (separate prompt).
2. **No demolition handling**: If a building is removed, the Warmth
   disc persists indefinitely (no negative dirty_regions / removal
   events). Out of scope for T7.10.A.
3. **Other 7 channels remain dispatch-shell**: Light shadowcast, Noise
   linear decay, Danger sight-radius cap, Social LOD-aware stamp,
   Spiritual/Beauty/FoodAroma exponential are all still zero buffers.
   T7.10.B..F will wire each.
4. **BFS uses dirty_region center as source**: A more accurate model
   would track exact source positions in a dedicated event queue
   (T7.10.B). For T7.10.A, region center = building center (BSS uses
   `event.position` as Chebyshev box center), so the approximation is
   exact for single-tile sources.

These limitations are intentional T7.10.A scope reduction per v3.3.15
`validate_scope.sh` (single channel per dispatch).

---

## Section 9 — Out of scope

- Any FFI surface change (T7.7.B contract locked)
- Any GDScript / scene / shader change
- Multi-source Warmth persistence (T7.10.B)
- Building demolition / negative events (T7.10.B+)
- Any non-Warmth channel propagation (T7.10.B..F)
- Performance optimization beyond what `propagate_bfs` already provides
