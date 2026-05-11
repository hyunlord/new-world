# T7.8 — Substantial Harness Tests + Phase 2 Criterion Benchmarks + T7.7.B Formal Re-run

**Lane**: `--full` (Planning debate + Visual Verify + Evaluator)
**Branch**: `lead/main`
**Prereq**: T7.7.B ENV-BYPASS commit `6032b0a3` (sim-bridge FFI scaffold + 21 mechanism tests)
**Hook governance**: v3.3.12 (Regression Guard SKIP_V7_RESET → CLEAN binding)

---

## 1. Implementation Intent

T7.7.B landed under ENV-BYPASS (Claude API rate limit) with **mechanism-level** harness coverage:
- 21 `harness_phase2_ffi.rs` tests verify FFI surface invariants (queue init, channel zeros,
  enqueue inbounds/OOB, dirty_regions, radius clamp, isolation).
- Coverage proves *the wiring works*; it does **not** prove *the wiring scales*.

T7.8 adds the missing dimensions:

1. **Substantial harness scenarios** (sim-test): long-running multi-tick simulations with
   1K agents, multi-building dirty-region integration, edge cases that exercise the full
   `building_event_queue → BuildingStampSystem → InfluenceUpdateSystem → AgentInfluenceSampleSystem`
   pipeline end-to-end.
2. **Phase 2 criterion benchmarks** (sim-core/benches/): per-tier latency measurement against
   Hard Gate 6 budget (Hot 0.5ms / Warm 2ms / Cold 5ms @ 1K agents).
3. **T7.7.B formal re-run**: replay the full pipeline now that rate-limit recovered, satisfy the
   §7.1 mandatory follow-up, and append `verified-post-bypass-6032b0a3` to
   `.harness/audit/env_bypass.log` (Step 2, separate commit).

> **Why now**: Phase 2 system list is frozen (priority 90/100/110/1000), Phase 0 design v0.1.1
> §G2 specifies Hard Gate 6, and the FFI surface is locked. Performance budget validation must
> happen before Phase 2 system content (T7.9+) starts compounding cost.

---

## 2. Decisions Locked

| # | Decision | Value | Source |
|---|----------|-------|--------|
| D1 | Substantial test count | 10–15 new tests | User spec (Step 1b directive) |
| D2 | Substantial test location | `rust/crates/sim-test/tests/harness_phase2_substantial.rs` (new file) | Mirror existing `harness_phase2.rs` / `harness_phase2_ffi.rs` |
| D3 | Benchmark location | `rust/crates/sim-test/benches/phase2_benchmarks.rs` (new file) | **Path B2** — only sim-test has all 4 crate deps (cycle-safe + Hard Gate 6 본질) |
| D3a | Benchmark host crate rationale | sim-engine cannot dev-dep sim-systems (cycle: sim-systems→sim-engine already exists, line 11). sim-test = 모든 deps 보유 + identity = "test harness — long-running simulation tests + integration scenarios" | `sim-systems/Cargo.toml:11`, `sim-test/Cargo.toml:6,11-14` |
| D4 | Benchmark framework | `criterion` 0.5 (workspace dep, **NOT yet wired for sim-test** — must add) | Cargo.toml workspace.dependencies:27 |
| D5 | Hard Gate 6 budget | Hot 0.5ms / Warm 2ms / Cold 5ms @ 1K agents | sim-engine/src/lib.rs:22-25, Phase 0 v0.1.1 §G2 |
| D6 | Long-run tick count | 4380 ticks (= 1 sim-year @ 12 ticks/day × 365) | Phase 0 design canonical year |
| D7 | Agent population | 1K (matches Hard Gate 6 reference scale) | Hard Gate 6 |
| D8 | Grid size for substantial tests | 64×64 (matches T7.7.B FFI tests `fresh_64()`) | Existing convention |
| D9 | T7.7.B FFI tests | **Untouched** (regression guard) | 21 tests must continue to pass byte-identical |
| D10 | New localization keys | **None** (Rust-only, no GDScript surface) | Lane `--full` no-locale path |
| D11 | sim-bridge changes | **None** in T7.8 (T7.9 reserved for sim-bridge integration tests) | Phase 2 Phase 0 §sequencing |
| D12 | Benchmark group naming | `phase2_hot`, `phase2_warm`, `phase2_cold` | Tier-aligned with Hard Gate 6 |

---

## 3. Files

| Action | Path | Reason |
|--------|------|--------|
| **NEW** | `rust/crates/sim-test/tests/harness_phase2_substantial.rs` | 10–15 long-running scenario tests |
| **NEW** | `rust/crates/sim-test/benches/phase2_benchmarks.rs` | Criterion benchmarks for Hot/Warm/Cold tiers (Path B2) |
| **EDIT** | `rust/crates/sim-test/Cargo.toml` | Add `[dev-dependencies] criterion = { workspace = true }` + `[[bench]] name = "phase2_benchmarks", harness = false` |
| **READ-ONLY** | `rust/crates/sim-test/tests/harness_phase2.rs` | Pattern reference (FireCounter, make_engine) |
| **READ-ONLY** | `rust/crates/sim-test/tests/harness_phase2_ffi.rs` | Pattern reference (fresh_64, bss_tick); regression guard |
| **READ-ONLY** | `rust/crates/sim-bridge/src/ffi/world_node.rs` | `enqueue_building_placed` import target |
| **READ-ONLY** | `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs` | `Position` + `InfluenceSample` type source (NOT re-exported at module level — import via `agent_sample::`) |
| **READ-ONLY** | `rust/crates/sim-core/benches/material_benchmarks.rs` | T6.7 precedent (criterion mechanism: `criterion_group!` + `criterion_main!` + `c.bench_function`) |

**Deliberately excluded** (out of scope, will be touched in T7.9):
- `rust/crates/sim-bridge/src/**` — no FFI changes
- `rust/crates/sim-engine/Cargo.toml` — BENCH-B 원안 폐기 (cycle 차단)
- `rust/crates/sim-core/Cargo.toml` — T6.7 `material_benchmarks` 그대로 보존
- `scripts/**` (GDScript) — no UI surface
- `localization/**` — no new keys
- `.harness/audit/env_bypass.log` — Step 2 (separate commit)

---

## 4. How to Implement

### 4.1 `rust/crates/sim-test/tests/harness_phase2_substantial.rs` (NEW)

> **D4 SCOPE — FACT-BASE PRECISION**: Scenarios below are **gap-focused** against the 39
> existing tests (18 `harness_phase2.rs` T7.6 + 21 `harness_phase2_ffi.rs` T7.7.B), mapped via
> direct grep. Each S<n> targets a documented coverage gap — NOT a hypothesis. Thresholds
> cross-verified against:
>
> - `sim-core/src/influence/grid.rs:51,89-118` (InfluenceGrid API: `sample`/`current_buf`/`swap`/`clear_dirty`/`dirty_regions` field)
> - `sim-systems/src/lib.rs:30-43` (`register_phase2_systems` registers exactly 4 systems)
> - `sim-systems/src/runtime/influence/agent_sample.rs:33-38` (`InfluenceSample` has 2 fields: `warmth` + `danger`)
> - `sim-engine/src/lib.rs:44,102,139,157` (`BuildingPlacedEvent`, `building_event_queue`, `register_system`, `tick`)
>
> **Existing coverage saturation** (do NOT duplicate):
> - Metadata (name/priority/tick_interval × 4 systems) ✅ T7.6 A1–A4
> - BSS/AIS isolation ✅ T7.6 A7, T7.7.B A18
> - Per-tile sample correctness ✅ T7.6 A9 (3 agents)
> - Viz digest fields (warmth_total/danger_peak/tick/fire_count) ✅ T7.6 A11/A14, 20 ticks
> - Register count==4 + priority order ✅ T7.6 A12/A15
> - FFI surface invariants (queue init, 8-channel zeros, enqueue inbounds/OOB, 4-channel stamp dirty, large-radius clamp, radius-0 single pixel, mixed OOB+valid continue, empty-queue tick) ✅ T7.7.B A1–A20

**Helper module** (top of file):

```rust
//! T7.8 — Substantial Phase 2 harness: long-running, multi-building, edge-case scenarios.
//!
//! Complements T7.7.B mechanism tests (`harness_phase2_ffi.rs`) by exercising the
//! `building_event_queue → BuildingStampSystem → InfluenceUpdateSystem →
//! AgentInfluenceSampleSystem` pipeline at 1K-agent / 4380-tick scale.

use sim_bridge::ffi::enqueue_building_placed;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;
const TICKS_PER_YEAR: u32 = 4380;

fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}
```

**Test list — 15 substantial, gap-focused (fact-base)**:

| # | Test name | Gap covered | Setup | Tick budget | Precise assertions |
|---|-----------|-------------|-------|-------------|---------------------|
| S1 | `harness_substantial_idle_4380_no_systems_all_channels_zero` | Long-run grid stability w/o systems (existing max=20 ticks) | 64×64, **no** `register_phase2_systems`, 0 agents, 0 events | 4380 ticks via raw `engine.tick()` | For each `ch in InfluenceChannel::all()`: `current_buf(ch).iter().all(|&v| v == 0)`. `building_event_queue.is_empty()`. `dirty_regions[ch as usize].is_empty()` for all 8. |
| S2 | `harness_substantial_idle_4380_phase2_active_all_channels_zero` | Long-run system idle stability (existing baseline only 0 ticks) | 64×64 + `register_phase2_systems`, 0 agents, 0 events | 4380 ticks | Same channel-zero + queue-empty + dirty-empty asserts as S1, but with 4 systems firing every tick (proves no spurious side-effects). |
| S3 | `harness_substantial_single_stamp_steady_state_4380_persist` | Long-run stamp persistence (existing only 20 ticks) | 64×64 + Phase 2; `enqueue_building_placed(&mut r, 32, 32, 5)` at tick 0 | 4380 ticks | Record `sample(32, 32, Warmth)` at ticks 1, 100, 1000, 4380 — **all four readings identical** (T7.7.B BSS has no decay; persistence invariant). Same channel readings for Spiritual/Beauty/Light. |
| S4 | `harness_substantial_1k_agents_sample_matches_grid_post_tick` | 1K-agent AIS correctness (existing max=3 agents) | 64×64 + Phase 2; spawn 1000 agents at `Position { x: (i*7)%64, y: ((i*13)/64)%64 }` + `InfluenceSample::default()`; stamp (32,32) r=10 at tick 0 | 30 ticks (warmup) | For every agent entity `(pos, sample)`: `sample.warmth == grid.sample(pos.x, pos.y, Warmth)` AND `sample.danger == grid.sample(pos.x, pos.y, Danger)`. Count agents with `sample.warmth > 0` ≥ 1 (footprint covers ≥1 agent given 1K density). |
| S5 | `harness_substantial_sequential_events_10_ticks_cumulative` | Time-evolution accumulation (existing A14 = 3 events SAME tick; this is across-ticks) | 64×64 + Phase 2; at each tick `t in 0..10` enqueue `(5+t, 5+t, r=1)` | 11 ticks | After tick 11: for each `t in 0..10`, `sample(5+t, 5+t, Warmth) > 0`. `building_event_queue.is_empty()`. `dirty_regions[Warmth].is_empty()` (IUS consumed all). |
| S6 | `harness_substantial_burst_100_events_single_tick_drain` | High-throughput BSS drain (existing max = 3 events/tick) | 64×64 + Phase 2; enqueue 100 events at distinct `(i % 8 * 8, i / 8 * 8)` for `i in 0..100` at tick 0 | 1 tick | `building_event_queue.is_empty()` after tick. Count of `(x,y)` tiles where `sample(x,y, Warmth) > 0` ≥ 100. No panic. |
| S7 | `harness_substantial_channel_saturation_50_repeat_stamps_no_panic` | u8 saturation behavior (existing tests never re-stamp) | 64×64 + Phase 2; for `t in 0..50` enqueue `(32, 32, r=0)` at tick `t` | 51 ticks | `sample(32, 32, Warmth) <= u8::MAX`. No panic. `building_event_queue.is_empty()`. (Documents whichever saturation/clamp policy BSS implements — invariant lock.) |
| S8 | `harness_substantial_dirty_region_lifecycle_clear_after_ius` | dirty_regions lifecycle (existing FFI tests use BSS-only ticks, never run IUS) | 64×64 + Phase 2; enqueue 1 event at tick 0 | 1 tick (full engine.tick → BSS+IUS+AIS+viz) | After tick 1: `dirty_regions[Warmth as usize].is_empty()` AND `dirty_regions[Spiritual as usize].is_empty()` AND `dirty_regions[Beauty as usize].is_empty()` AND `dirty_regions[Light as usize].is_empty()` (IUS consumed via `clear_dirty`). |
| S9 | `harness_substantial_hot_cold_tier_co_execution_no_interference` | Hot 90/100/110 + Cold 1000 same tick interleave (existing tests isolate tiers) | 64×64 + Phase 2; enqueue event at tick 5 | Advance until `current_tick == 6` (viz fires since `6 % 6 == 0`) | At tick 6: `building_event_queue.is_empty()` (BSS drained). `sample(x,y, Warmth) > 0` at stamped tile (IUS swapped). Viz `digest.tick == 6` (cold tier fired). No panic. |
| S10 | `harness_substantial_buffer_swap_consistency_4380` | Double-buffer integrity across 4380 swaps | 64×64 + Phase 2; stamp (32, 32, r=2) at tick 0; spawn 1 agent at (32, 32) | 4380 ticks; sample `(agent.InfluenceSample.warmth, grid.sample(32,32,Warmth))` at ticks 30, 100, 1000, 4380 | All 4 sampled pairs equal across all 4 ticks (steady-state preserved). |
| S11 | `harness_substantial_register_phase2_twice_doubles_count_no_panic` | Re-registration behavior unspecified by current contract | 64×64; call `register_phase2_systems(&mut engine)` twice | 1 tick | No panic on second registration. No panic on subsequent tick (priority sort stable with duplicates). (Locks current behavior as invariant — Phase 0 §G2 contract documentation.) |
| S12 | `harness_substantial_four_corner_stamps_clamp_no_oob_dirty` | Boundary clamping at all 4 corners (existing FFI A16 tests one large radius only) | 64×64 + Phase 2; enqueue 4 events: (0,0,r=3), (63,0,r=3), (0,63,r=3), (63,63,r=3) at tick 0 | 1 tick | For each of 4 stamped channels: `dirty_regions[ch as usize].iter().all(\|r\| r.x0 < 64 && r.y0 < 64 && r.x1 <= 64 && r.y1 <= 64)`. (No coordinate exceeds grid bounds.) No panic. |
| S13 | `harness_substantial_composite_edge_types_single_tick` | 4-way edge composition (existing FFI A17 = 2-type mix) | 64×64 + Phase 2; enqueue in order: (5,5,r=2)/(-1,5,r=2)/(62,62,r=10)/(5,5,r=0) | 1 tick | `enqueue_building_placed` returns: `[true, false, true, true]`. After tick 1: queue empty; (5,5)/(62,62)/(5,5)+r=0 footprints stamped; (-1,5) NOT enqueued (and thus not stamped). All `dirty_regions` in-bounds. |
| S14 | `harness_substantial_viz_24_fires_in_144_ticks_aggregate` | Viz frequency invariant at scale (existing = 4 fires in 20 ticks) | 64×64 + Phase 2 + `FireCounter` (T7.6 helper pattern at priority 1001, interval 6); stamp (32,32,r=5) at tick 0 | 144 ticks | `FireCounter` count == 24 (fires at `current_tick % 6 == 0` for ticks 0,6,12,…,138 = 24 fires). Viz `digest.tick == 138`. Viz `digest.warmth_total > 0` (persistent stamp). |
| S15 | `harness_substantial_throughput_1000_events_4380_ticks_walltime` | Year-scale throughput smoke (NOT a criterion substitute — release-build sanity bound) | 64×64 + Phase 2; spread 1000 events at `tick % 4 == 0 && tick < 4000` deterministic positions | 4380 ticks | `building_event_queue.is_empty()`. Wall-clock < 30 s in `--release` (test gated by `#[cfg(not(debug_assertions))]` or unconditional with generous bound). ≥1000 distinct `(x,y)` tiles with `sample(x,y, Warmth) > 0`. |

**Integration with existing 39 tests** — separate file `harness_phase2_substantial.rs` (NOT extension of `harness_phase2.rs` / `harness_phase2_ffi.rs`):
- T7.7.B `harness_phase2_ffi.rs` uses **BSS-only ticks** via `bss_tick(e)` — preserves dirty_regions for direct assertion. Regression guard demands these stay byte-identical.
- T7.6 `harness_phase2.rs` uses full `engine.tick()` for Phase 2 metadata + isolation.
- T7.8 substantial **uses full `engine.tick()`** (real pipeline integration) with `register_phase2_systems` — different model than T7.7.B's BSS-only.
- Separating files keeps the contracts non-overlapping; importing `FireCounter` (T7.6 line 33) is allowed via `mod` reuse or duplication (decided at implementation: duplicate is acceptable, mod-export not currently set up).

> **User review focus** (post-D4 precision):
> - **S7 saturation policy**: BSS `stamp_*` impl decides clamp-to-max vs saturating_add vs wrapping_add. Test locks whichever is current. Confirm acceptable to lock current behavior as invariant.
> - **S11 re-registration**: current `engine.register_system(Box::new(...))` is additive (no dedup). Test asserts this. If contract should be idempotent instead → S11 inverts (assert count stays 4).
> - **S15 walltime threshold**: 30 s is generous. Lower bound (e.g. 10 s) if release-mode 1K events / 4380 ticks measures sub-5-s on local hardware.

### 4.2 `rust/crates/sim-test/benches/phase2_benchmarks.rs` (NEW — Path B2)

**Path B2 본질**: sim-test가 `sim-core + sim-engine + sim-systems + sim-bridge` 모두 의존
→ `SimEngine.tick` + `register_phase2_systems` + 1K agents 정합 측정 가능 (Hard Gate 6 본질).

> **D4 fact-base — tier mapping locked**:
> - **1K agents spawn** (D3 locked): deterministic seed `x = (i*7) % 64, y = ((i*13)/64) % 64`,
>   1 InfluenceSample component per agent, 30-tick warmup before measurement.
> - **Hot tier** = priority 90/100/110, all `tick_interval == 1` (every tick): BSS + IUS + AIS.
>   Benches B1 (idle) + B2 (1 event/tick) measure this.
> - **Cold tier** = priority 1000 InfluenceVisualizationSystem, `tick_interval == 6`. Benches
>   B3 (burst stress) + B4 (viz-fire tick alignment) measure this.
> - **Warm tier** = **N/A (D2 locked)**. No Phase 2 system fits the Phase 0 §G2 "stagger 1/3 ticks"
>   profile yet. Adding a placeholder bench measures nothing → omitted. Will reintroduce in T7.9+
>   when first Warm-tier system lands.

**Skeleton** (T6.7 mechanism + sim-test imports):

```rust
//! T7.8 — Phase 2 criterion benchmarks: SimEngine.tick @ 1K agents vs Hard Gate 6.
//!
//! Hard Gate 6 budget (sim-engine/src/lib.rs:22-25, Phase 0 v0.1.1 §G2):
//!   Hot   tier: 0.5 ms / tick @ 1K agents
//!   Warm  tier: 2.0 ms / tick @ 1K agents
//!   Cold  tier: 5.0 ms / tick @ 1K agents
//!
//! Path B2: sim-test hosts the bench because it's the only crate with
//! deps on all 4 sim-* crates (sim-systems → sim-engine cycle blocks
//! sim-engine/benches/ from importing register_phase2_systems).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hecs::Entity;

use sim_bridge::ffi::enqueue_building_placed;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};

const W: u32 = 64;
const H: u32 = 64;
const AGENTS: usize = 1_000;
const WARMUP_TICKS: u32 = 30;

/// Build a Hard-Gate-6 reference engine: 64×64 grid, Phase 2 systems registered,
/// 1K agents spawned at deterministic Position grid coverage, 30-tick warmup.
fn make_1k_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);

    // 1K agents distributed across 64×64 (4096 tiles → 1 per ~4 tiles).
    for i in 0..AGENTS {
        let x = (i as u32 * 7) % W;          // pseudo-random spread
        let y = ((i as u32 * 13) / W) % H;   // (deterministic seed)
        engine
            .world
            .spawn((Position { x, y }, InfluenceSample::default()));
    }

    // Warmup: stabilize buffers + dirty region invariants.
    for _ in 0..WARMUP_TICKS {
        engine.tick();
    }
    engine
}
```

**Benchmark functions** (B1–B5 — user reviews & confirms):

| # | Function | Tier | Measures |
|---|----------|------|----------|
| B1 | `bench_engine_tick_idle_1k_agents` | Hot | Steady-state SimEngine.tick (no events injected) — baseline cost of 1K agent sample + buffer swap + dirty_region empty |
| B2 | `bench_engine_tick_single_event_per_tick` | Hot | 1 BuildingPlacedEvent / tick → BuildingStampSystem (90) + InfluenceUpdateSystem (100) + AgentInfluenceSampleSystem (110) full path |
| B3 | `bench_engine_tick_burst_10_events` | Cold | 10 BuildingPlacedEvent same tick → max dirty_region union scenario |
| B4 | `bench_engine_tick_visualization_tick` | Cold | Tick aligned to InfluenceVisualizationSystem (priority 1000, tick_interval=6) |
| B5 | `bench_register_phase2_systems_construction` | (cold-init) | One-shot: SimEngine::new + register_phase2_systems + 1K spawn (excludes warmup) — sanity bound for engine bring-up |

**Tier mapping rationale** (D2 + D3 locked, fact-base):
- BSS (90) + IUS (100) + AIS (110) all `tick_interval == 1` → **Hot tier**, B1 (idle) + B2 (1 event/tick) cover this with the full chain firing per tick.
- InfluenceVisualizationSystem (1000) is `tick_interval == 6` → **Cold tier** (every 6th tick). B4 measures it via tick alignment; B3 separately measures dirty-region union pressure under burst stress.
- **Warm tier N/A** (D2): no Phase 2 system on the stagger-1/3 profile. Placeholder omitted; revisited in T7.9+.

**criterion_group structure**:

```rust
criterion_group!(
    phase2_hot,
    bench_engine_tick_idle_1k_agents,
    bench_engine_tick_single_event_per_tick,
);
criterion_group!(
    phase2_cold,
    bench_engine_tick_burst_10_events,
    bench_engine_tick_visualization_tick,
);
criterion_group!(
    phase2_init,
    bench_register_phase2_systems_construction,
);
criterion_main!(phase2_hot, phase2_cold, phase2_init);
```

> **본질 측정 원칙** (사용자 axiom #1 정합):
> - 모든 bench는 `SimEngine::tick()` 호출 중심 (primitives 측정 X)
> - 1K agents 실제 spawn (Position + InfluenceSample component 동반)
> - `register_phase2_systems(&mut engine)` 통과 (실제 4 systems 동작)
> - 30-tick warmup 후 측정 (dirty_region + buffer steady-state)

### 4.3 `rust/crates/sim-test/Cargo.toml` (EDIT)

```toml
# (existing dependencies unchanged: hecs, serde, sim-core, sim-engine, sim-systems, sim-bridge)

[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "phase2_benchmarks"
harness = false
```

**No changes** to:
- `rust/crates/sim-engine/Cargo.toml` (BENCH-B 원안 폐기, cycle 차단)
- `rust/crates/sim-core/Cargo.toml` (T6.7 `material_benchmarks` 보존)
- workspace `Cargo.toml` (`criterion` 이미 workspace dep)

---

## 5. Verification

### 5.1 Mechanical Gate (must pass)

```bash
cd rust
cargo test --workspace --no-fail-fast              # all tests pass (T7.7.B 21 + T7.6 18 + T7.8 new 10–15)
cargo build --workspace                            # all crates build
cargo clippy --workspace --all-targets -- -D warnings  # clean
cargo bench -p sim-test --bench phase2_benchmarks -- --quick  # benchmarks compile + run smoke (Path B2)
```

### 5.2 Hard Gate 6 Budget Verification

After `cargo bench --bench phase2_benchmarks`:

| Tier | Budget @ 1K agents | Pass criterion | Bench coverage |
|------|--------------------|----------------|----------------|
| Hot  | 0.5 ms / tick      | All `phase2_hot` benches median < 500 µs   | B1 (idle 1K agents), B2 (single event / tick — BSS→IUS→AIS full chain) |
| Warm | 2.0 ms / tick      | **N/A (D2 lock)** — no Phase 2 system in this tier | (no bench scheduled — see rationale) |
| Cold | 5.0 ms / tick      | All `phase2_cold` benches median < 5000 µs | B3 (burst 10 events — max dirty-region union), B4 (viz tick alignment, `current_tick % 6 == 0`) |

**Warm-tier N/A rationale (D2 — user-locked)**:
Phase 2 currently has **no separate Warm-tier system**. All 4 RuntimeSystems are either Hot
(`tick_interval == 1`: BSS pri 90, IUS pri 100, AIS pri 110) or Cold (InfluenceVisualizationSystem
`tick_interval == 6`). The Phase 0 §G2 "Warm tier stagger" target is reserved for systems landing
in **T7.9+** (Population, Need, Cognition tiers staggered 1/3 of ticks). Reintroducing a Warm
benchmark before then would be measuring nothing.

→ Hard Gate 6 Warm row marked N/A with this rationale. Will be revisited when first Warm-tier
system lands.

If any non-N/A benchmark exceeds its budget → **STOP**, report to user before commit (Phase 2
design must be revisited per Phase 0 §G2).

### 5.3 Regression Guard

- T7.6 `harness_phase2.rs` 18 tests: continue to pass byte-identical
- T7.7.B `harness_phase2_ffi.rs` 21 tests: continue to pass byte-identical
- No changes to `register_phase2_systems` system list
- No changes to `sim-bridge/src/ffi/world_node.rs` (Bridge Identity Contract preserved)

### 5.4 Localization

**No new keys.** Pure Rust; no `.ftl` / `.json` touched.

---

## 6. Lane / Pipeline

`--full` → Planning debate (Drafter → Challenger → Quality Checker) → Generator → Visual Verify →
Evaluator (Codex) → Integrator.

**Expected score**: ≥ 90 / 100
- Code Quality: 15/15 (attempt 1, no rework expected)
- Visual Verify: 20/20 (no GDScript surface; verify-bypass headless OK)
- Tests: 20/20 (**15 new substantial + 21 FFI + 18 Phase 2 = 54 tests**)
- Regression: 15/15 (T7.6 + T7.7.B intact; classifier v3.3.12)
- Evaluator: 15/15 (Hard Gate 6 Hot+Cold budget + criterion evidence; Warm N/A documented per D2)
- Gate: 10/10
- **Total: ≥ 95**

**Hot-tier classifier**: this commit hits hot tier (sim-test + sim-core both governed). No
STRUCTURAL admission needed — formal pipeline applies.

---

## 7. 인게임 확인사항

**없음.** sim-bridge / GDScript surface 변경 없음 — T7.9 (sim-bridge integration tests) 에서
WorldSimNode 상에서의 in-game smoke test 수행 예정.

---

## 8. Commit Message (template)

```
feat(phase2)[T7.8]: Substantial harness + Phase 2 criterion benchmarks (Path B2)

- sim-test/tests: + harness_phase2_substantial.rs (N tests, 4380-tick scale, 1K agents)
- sim-test/benches: + phase2_benchmarks.rs (SimEngine.tick @ 1K agents, Hard Gate 6 verify)
- sim-test/Cargo.toml: + criterion dev-dep + [[bench]] phase2_benchmarks

Path B2 rationale: sim-engine cannot dev-dep sim-systems (cycle —
sim-systems already depends on sim-engine). sim-test = only crate
with all 4 sim-* deps; identity = "test harness — long-running +
integration scenarios" (Cargo.toml:6) → bench host 정합.

Hard Gate 6 budget verified @ 1K agents (SimEngine.tick + register_phase2_systems):
  Hot   tier: <result> ms / 0.5 ms budget
  Cold  tier: <result> ms / 5.0 ms budget
  (Warm tier benchmark deferred — Phase 2 has no Warm-tier system yet)

Regression: T7.6 18 tests + T7.7.B 21 FFI tests pass byte-identical.
No sim-bridge changes (T7.9 reserved). No sim-engine/sim-core Cargo.toml
changes. No localization keys.
```

---

## 9. Step 2 (separate commit) — T7.7.B Postverify Audit

After T7.8 lands and pipeline verdict = APPROVE, append to `.harness/audit/env_bypass.log`:

```
2026-05-11T<UTC>Z | t7-7-b-sim-bridge-ffi | verified-post-bypass-6032b0a3 | T7.8 formal pipeline APPROVE confirms T7.7.B FFI surface; mandatory §7.1 follow-up satisfied within 7-day window
```

This is a `.harness/audit/` file → auto-exempt from pre-commit hook → normal commit, no
HARNESS_SKIP needed.

---

## Annotations for User Review (Path B2 Final)

> **★ Cycle verified — BENCH-B 원안 폐기**:
> - `sim-systems/Cargo.toml:11`: `sim-engine = { path = "../sim-engine" }` ← **이미 의존**
> - 따라서 sim-engine → sim-systems dev-dep = workspace cycle
> - **Final location**: `sim-test/benches/` (sim-test = 모든 deps 보유 + bench host identity)
>
> **★ Type name correction (사용자 spec → actual)**:
> - 사용자 spec: `AgentInfluenceSnapshot` → **실제 코드**: `InfluenceSample`
>   (`sim-systems/src/runtime/influence/agent_sample.rs:33`)
> - 사용자 spec: `sim-systems::runtime::influence::Position` → **실제 import 경로**:
>   `sim_systems::runtime::influence::agent_sample::Position` (모듈 레벨 re-export X,
>   `mod.rs:9-17` 확인)
> - 사용자 spec: `register_phase2_systems(&mut engine, ...)` → **실제 시그니처**:
>   `register_phase2_systems(engine: &mut SimEngine)` 단일 인자 (`sim-systems/src/lib.rs:30`)
>
> **★ T6.7 precedent verified**: `rust/crates/sim-core/benches/material_benchmarks.rs`
> read in full — `criterion_group!` + `criterion_main!` + `c.bench_function(..., |b| b.iter(...))`
> + `black_box(...)` mechanism은 location 무관 정합. Path B2에서도 그대로 적용.
>
> **★ D4 (S-C) resolved — fact-base S1-S15**:
> - 39 existing tests grep-mapped (18 T7.6 + 21 T7.7.B); each S<n> targets a documented gap.
> - Assertion thresholds derived from API source files (grid.rs / lib.rs / agent_sample.rs),
>   not hypothesized.
> - Long-run scale (4380 ticks / 1K agents) is **new dimension** vs existing (max 20 ticks / 3 agents).
> - 15 tests total (D1 spec lower-bound of 10–15) — all 15 cover distinct gaps.
> - Separate file `harness_phase2_substantial.rs` chosen because T7.7.B uses BSS-only ticks
>   (regression-guard sensitive); T7.8 uses full `engine.tick()` integration.
>
> **★ D2 resolved — Warm tier N/A locked**:
> - §4.2 tier mapping + §5.2 Hard Gate 6 row both state "Warm N/A — no Phase 2 system on this
>   profile". Reintroduced in T7.9+.
>
> **★ D3 resolved — Grid 64×64 locked**:
> - matches T7.7.B `fresh_64()` convention; 1K agents = 25% density.
>
> **★ Remaining user review focus**:
> - **S7 saturation policy lock** — confirm OK to encode "whatever BSS does today" as invariant.
> - **S11 register idempotence direction** — current behavior is additive (count doubles); test
>   asserts that. If contract should be idempotent → flip S11 assertion to "count stays 4".
> - **S15 walltime threshold** — current 30 s in `--release`; lower if local hardware shows headroom.
>
> **★ User action path**:
> - Approve as-is → I move file to `.harness/prompts/t7-8-substantial-harness.md` → Step 1c
>   `--full` pipeline (~1.5–2 h, expect Score ≥ 90, auto-commit).
> - Modify S7/S11/S15 only → edit §4.1 row directly, then approve.
> - Step 2 (post-pipeline APPROVE) → `.harness/audit/env_bypass.log` append + separate commit.
