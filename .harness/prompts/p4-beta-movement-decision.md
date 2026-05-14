# P4-β — Agent Movement System + AgentDecision plumbing (V7 Phase 4 sub-stage)

> Lane: `--quick` (sim-core + sim-engine + sim-systems + sim-test; no GDScript, no FFI surface change)
> Scope: Second Phase 4 sub-stage. Land `AgentMovementSystem` (priority 120)
> with deterministic per-agent Brownian motion via inline splitmix64.
> AgentDecision plumbing is deferred — see Section 8 honesty notes.
> Governance: v3.3.16. Visual: out of scope (backend-only; VLM auto-credit).

---

## Section 1 — Implementation Intent

P4-α (commit 81fa6428 area, T-prefixed agent core) landed
`sim_core::components::{Position, Agent}` and `SimEngine::spawn_agent`.
P4-β layers on top: agents now **move** each tick. The motion model is
the simplest deterministic walk that the planning fact-base sanctions —
per-agent Brownian step in `{-1, 0, +1}^2`, seeded from a per-agent
`MovementRng` so replays are byte-identical.

Planning §2.2 (`.harness/plans/phase4.md` lines 120-153) prescribes:
- AgentMovementSystem priority ~120 (after AIS priority 110)
- "Brownian step or simple goal-seek" — Brownian chosen (minimal scope)
- "Deterministic seeded RNG" — splitmix64 inline (no `rand` crate dep)
- AgentDecision deferral acceptable (β default — γ/δ may revisit)
- ≥8 assertions — this prompt targets **11** assertions

After P4-β:
- `sim_systems::runtime::agent::{AgentMovementSystem, MovementRng}` is the
  canonical movement subsystem; priority 120 keeps it strictly **after**
  AIS (110) so each agent reads influence at its pre-move tile.
- `register_agent_systems(&mut SimEngine)` is the registration helper
  (parallel to `register_phase2_systems`).
- AgentDecision is **not** introduced — neither as a component nor as a
  `CausalEvent` variant. β scope locks at deterministic motion only.
- `causal::event::CausalEvent` is **unchanged** (3 variants — see Out of
  scope §9).

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P4β-1: motion model | Planning §2.2 + axiom #2 | **Brownian** step `{-1, 0, +1}^2`, **inline splitmix64** (no `rand` crate dep) |
| P4β-2: AgentDecision scope | Planning §2.2 ("deferral acceptable") | **Deferred to γ/δ** — neither component nor enum variant lands in β |
| P4β-3: CausalEvent extension | `causal/event.rs` doc | **Untouched** — 3 variants remain (BuildingPlaced / StampDirty / InfluenceChanged) |
| P4β-4: System priority / cadence | Planning §2.2 | **priority = 120, tick_interval = 1** (after AIS 110, before Viz 1000) |
| P4β-5: Harness assertion count | Planning §2.2 ≥8 + axiom #1 | **11 assertions** (1 more than P4-α floor) |
| Position field types | P4-α LOCKED | u32 (tile coordinates) — clamp via i64 arithmetic |
| Step value reduction | LOCKED design | `((u64 % 3) as i32) - 1` — bias on order of 2^-63, well below β tolerance |
| Boundary handling | LOCKED design | `.clamp(0, max)` on i64, then cast back to u32 — never underflow |
| Tick gating | `sim-engine/src/lib.rs:~170` | `current_tick.is_multiple_of(tick_interval)` — interval=1 ⇒ every tick |
| Re-export contract | Mirrors P4-α | `pub use movement::{AgentMovementSystem, MovementRng};` on `runtime::agent` |
| ECS query | LOCKED design | `world.query::<(&mut Position, &mut MovementRng)>()` |

**Determinism rationale** (axiom #2 — verify before assuming):
1. splitmix64 is a 64-bit mixing function used as a `SplittableRandom`
   (Java) successor; output stream passes PractRand at the step counts
   any single sim will realistically consume.
2. State `seed == 0` is a valid seed: splitmix64 escapes zero on its
   first call because the additive constant (0x9E3779B97F4A7C15) is
   non-zero.
3. Per-agent state ⇒ trajectories are independent (no cross-agent
   coupling through a shared RNG).
4. Same seed ⇒ same `next_u64()` stream ⇒ same `(dx, dy)` sequence ⇒
   byte-identical trajectory replay.

All four properties are exercised by the harness (tests #4, #6, #7, #8).

---

## Section 3 — What to build

### 3.1 `rust/crates/sim-systems/src/runtime/agent/mod.rs` (NEW)

```rust
//! V7 Phase 4-β — Agent runtime systems.
//!
//! Owns the per-tick motion system for canonical
//! [`sim_core::components::Agent`] entities. Phase 4-α landed the canonical
//! components and `SimEngine::spawn_agent`; this module adds the priority-120
//! [`AgentMovementSystem`] so agents actually move on the tile grid.

pub mod movement;

pub use movement::{AgentMovementSystem, MovementRng};
```

### 3.2 `rust/crates/sim-systems/src/runtime/agent/movement.rs` (NEW)

- `MovementRng { state: u64 }` — `new(seed)`, `next_u64()` (splitmix64),
  `next_step() -> i32` (`((u64 % 3) as i32) - 1`).
- `AgentMovementSystem` — zero-sized, `Default + new()`, `RuntimeSystem`
  impl: `name = "AgentMovementSystem"`, `priority = 120`,
  `tick_interval = 1`, `tick()` queries `(&mut Position, &mut MovementRng)`
  and applies clamped `(dx, dy) ∈ {-1, 0, +1}^2`.
- 5 inline unit tests: metadata / splitmix-escapes-zero /
  same-seed-same-stream / agent-moves-after-one-tick / boundary-clamp /
  distinct-seeds-diverge.

### 3.3 `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod agent;` alongside existing `pub mod influence;`. Update the
file-level doc to record that β-stage agent systems live here.

### 3.4 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

Add `register_agent_systems(&mut SimEngine)` that registers
`AgentMovementSystem` only. Update the crate doc to reference the new
helper. Do **not** call it from `register_phase2_systems` — registration
ordering is the harness/test responsibility (`register_phase2_systems`
then `register_agent_systems`).

### 3.5 `rust/crates/sim-test/tests/harness_p4_beta_movement.rs` (NEW, 11 assertions)

| # | Test name | Asserts | Type |
|---|-----------|---------|:----:|
| 1 | `harness_p4_beta_movement_system_export_resolves` | `AgentMovementSystem::new()` resolves through `sim_systems::runtime::agent::*` | A |
| 2 | `harness_p4_beta_system_metadata` | `name() == "AgentMovementSystem"`, `priority() == 120`, `tick_interval() == 1` | A |
| 3 | `harness_p4_beta_rng_export_resolves` | `MovementRng::new(seed)` resolves through `runtime::agent` | A |
| 4 | `harness_p4_beta_rng_escapes_zero_seed` | `MovementRng::new(0).next_u64() != 0` (splitmix64 zero-seed escape) | A |
| 5 | `harness_p4_beta_one_tick_brownian_step` | spawn at (10,10), 1 tick → `abs_diff(new, 10) ≤ 1` on each axis | A |
| 6 | `harness_p4_beta_determinism_full_trajectory` | two engines, same seed, 16 ticks → byte-identical position sequence | A |
| 7 | `harness_p4_beta_distinct_seeds_diverge` | seeds (1,2), 32 ticks → at least one tick where positions differ | A |
| 8 | `harness_p4_beta_multi_agent_independence` | solo trajectory (seed 7, 16 ticks) == multi-tenant trajectory of same seed alongside seed 13 | D |
| 9 | `harness_p4_beta_boundary_clamp` | 200 ticks at (0,0) and (W-1,H-1) — coords stay in `[0, W) × [0, H)` | A |
| 10 | `harness_p4_beta_register_and_progress` | `register_agent_systems(&mut engine)` + 1 spawn + 8 advance ticks → at least one position changed | D |
| 11 | `harness_p4_beta_influence_sampler_still_reads_canonical_position` | stamp Warmth at agent's pre-move tile, register Phase 2 + agent stacks, 1 tick → `InfluenceSample::warmth > 0` (IUS@110 still upstream of AMS@120) | D |

Test 6 is the **determinism landmark**: byte-identical replay across
fresh engines proves the splitmix64 contract.
Test 11 is the **integration regression**: ensures the Phase 2 sampler
(priority 110) still observes the agent's pre-move tile before AMS (120)
relocates it.

---

## Section 4 — Locale

No new locale keys. No user-visible UI surface in P4-β.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted P4-β harness
cd rust && cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture

# 3. P4-α regression (canonical components contract intact)
cd rust && cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture

# 4. Phase 2 / T7.10 regression (IUS chain priority unaffected by AMS@120)
cd rust && cargo test -p sim-test --test harness_phase2 -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_f_beauty_bfs_wiring -- --nocapture
```

Expected: 11 new P4-β tests pass + all P4-α / Phase 2 / T7.10
regressions green + 0 clippy warnings + 0 build failures.

---

## Section 6 — Lane

`--quick`. Rationale:
- Two new files under `sim-systems/src/runtime/agent/` (sibling to
  `influence/`).
- One new method-free constant in `sim-systems/src/lib.rs`
  (`register_agent_systems`).
- One new harness file (11 assertions).
- No FFI surface change (T7.7.B contract locked).
- No scene/shader/locale change.
- No new external dependency (inline splitmix64; `rand` not pulled in).
- Planning debate skipped — planning §2.2 fact-base prescribes Brownian
  + seeded RNG + priority 120 explicitly.

---

## Section 7 — In-game verification (post-merge)

P4-β is **backend-only**. Agents now move tick-by-tick in the simulation
but do not render in the Godot view yet (sprite rendering arrives in γ).
Verification is via harness tests only; VLM auto-credit applies (no
Godot scope).

After `cargo build -p sim-bridge` + `cargo build -p sim-bridge --release`,
Godot continues to display the same influence buffers as before. Agent
motion is observable only through the harness assertions and (if a
developer manually calls `world.query::<(&Position, &Agent)>()`) the
Rust-side debug tooling.

---

## Section 8 — Phase 4 dispatch (axiom #1 honesty)

P4-β is **second of four sub-stages**. Honest scope limits:

1. **Brownian only** — motion is undirected. γ/δ may layer goal-seek
   (e.g. gradient-ascent on Warmth influence) but β does not.
2. **No AgentDecision** — neither as a component nor as a `CausalEvent`
   variant. The plumbing is deferred per planning §2.2 explicit
   sanction. A future sub-stage adding decision events will:
   (a) add `AgentDecision { tick, entity, choice }` to
   `causal::event::CausalEvent`,
   (b) extend the ring buffer recorders,
   (c) wire β motion to publish a step record.
3. **No personality / body / needs / BodyHealth** — Agent remains a
   zero-sized marker (β-stage decision; carry-over from α-stage scope).
4. **No sprite rendering / FFI extension** — γ adds Godot sprite layer
   and sim-bridge agent snapshot.
5. **No removal of `agent_sample::Position` `pub use` alias** — alias
   remains for downstream compatibility (post-γ cleanup at earliest).
6. **No movement budget tuning** — every agent gets one step per tick.
   LodTier-based throttling is not in β.

---

## Section 9 — Out of scope

- AgentDecision component (β/γ choice; deferred)
- `causal::event::CausalEvent::AgentDecision` variant (deferred — see §8)
- Causal ring-buffer recording of movement (deferred)
- Goal-seek / influence-gradient motion (γ/δ)
- Sprite rendering / `sim-bridge` agent snapshot FFI extension (γ)
- Personality / Body / Needs / BodyHealth components (β/γ/δ already
  carved up in α scope)
- Movement budget / LodTier-based tick throttling
- Removal of `agent_sample::Position` `pub use` alias
- Any scene / shader / locale change
- Performance benchmarks for movement throughput (Phase 1-target ~500
  agents; architecture headroom per CLAUDE.md is well above β cost)
