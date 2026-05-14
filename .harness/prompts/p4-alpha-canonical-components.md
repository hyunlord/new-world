# P4-α — Canonical Position + Agent Components (V7 Phase 4 entry)

> Lane: `--quick` (sim-core + sim-engine + sim-systems + sim-test; no GDScript, no FFI surface change)
> Scope: First Phase 4 sub-stage. Establish `sim_core::components::{Position, Agent}`
> as canonical ECS components, replacing the Phase 2 placeholder at
> `sim_systems::runtime::influence::agent_sample::Position` per the
> self-documenting landmark (`agent_sample.rs:9-15`). Add
> `SimEngine::spawn_agent(x: u32, y: u32) -> Entity` wrapper API.
> Governance: v3.3.16. Visual: out of scope (backend-only; VLM auto-credit).

---

## Section 1 — Implementation Intent

Phase 3-γ closed the causal-chain rendering loop (commit d8545fa6,
T7.10.A-F closed all 6 stamped influence channels). Phase 4 opens the
**Agent layer** — autonomous entities with Position + AgentDecision +
movement systems + sprite rendering. P4-α is the first of four
sub-stages (α/β/γ/δ); it lands the canonical components and spawn API
so β (movement systems + AgentDecision plumbing) can layer on top.

The Phase 2 IUS sampler already references a placeholder
`Position { x: u32, y: u32 }` at `agent_sample.rs:9-15` (with a comment
explicitly flagging the migration target: *"land Position in
sim-core::components when Phase 4 begins; this file rewires via
`pub use` in a single line"*). P4-α executes that single-line rewire
exactly as the landmark prescribes.

After P4-α:
- `sim_core::components::Position { x: u32, y: u32 }` is the canonical
  ECS coordinate component.
- `sim_core::components::Agent` is a zero-sized marker identifying
  autonomous entities (Personality/Body/Needs/BodyHealth deferred to
  β/γ/δ — minimal scope per axiom #2).
- `SimEngine::spawn_agent(x: u32, y: u32) -> Entity` is the canonical
  spawn API; bridge / harness / future β systems call this instead of
  raw `world.spawn(...)`.
- `agent_sample.rs` retains a single-line `pub use sim_core::components::Position;`
  for downstream import compatibility (Phase 2 imports continue to
  resolve through the legacy path during transition).
- The 4 known migration touchpoints (Phase 2 harness + integration +
  bench imports) point at the canonical `sim_core::components::Position`
  directly.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| Position field types | `agent_sample.rs:23-29` placeholder | **u32** (tile coordinates, NOT pixels) |
| Architecture invariant | CLAUDE.md "Tile coords, not pixels" | u32 simulation, f32 pixel conversion = GDScript renderer's job |
| Position consumer check | `agent_sample.rs:70-74` | `pos.x >= w as u32` — already u32, no cast required |
| Phase 2 IUS sampler priority | `agent_sample.rs` | **110** (unchanged) |
| Spawn API signature | LOCKED design | `pub fn spawn_agent(&mut self, x: u32, y: u32) -> hecs::Entity` |
| Agent marker layout | LOCKED design | **zero-sized** unit struct, derives `Copy + Default + PartialEq + Eq + Debug` |
| Migration touchpoints | grep `agent_sample::Position` | 4 files: `sim-systems/tests/integration.rs:14`, `sim-test/tests/harness_phase2.rs:19`, `sim-test/tests/harness_phase2_substantial.rs:34`, `sim-test/benches/phase2_benchmarks.rs:29` |
| ECS backend | `Cargo.toml` workspace | `hecs 0.10` (already in `sim-core` deps; no new dependency) |
| Re-export contract | LOCKED design | `pub use sim_core::components::Position;` on `agent_sample` keeps existing imports working |

**u32 alignment rationale** (axiom #2 — verify before assuming):
1. Phase 4 design doc §2 (lines 84-85) specifies u32 tile coordinates.
2. The existing Phase 2 placeholder at `agent_sample.rs:23-29` is u32.
3. The Phase 2 consumer (`agent_sample.rs:70-74`) compares with `pos.x >= w as u32`.
4. CLAUDE.md architecture invariant: tile coords are u32, pixel conversion is GDScript's job.
All four sources align on u32. A pre-emptive f32 hypothesis was
explicitly rejected during Step 0 sweep — that decision is recorded here
so future readers do not re-litigate.

---

## Section 3 — What to build

### 3.1 `rust/crates/sim-core/src/components/mod.rs` (NEW)

```rust
//! Canonical ECS components (V7 Phase 4 entry).
//!
//! Phase 4-α lands the minimal set: `Position` (tile coordinates) and
//! `Agent` (marker). Further components — Personality, Body, Needs,
//! BodyHealth — arrive in β/γ/δ sub-stages.

pub mod agent;
pub mod position;

pub use agent::Agent;
pub use position::Position;
```

### 3.2 `rust/crates/sim-core/src/components/position.rs` (NEW)

```rust
//! Canonical `Position` component (V7 Phase 4-α first deliverable).
//!
//! Tile coordinates in u32. Pixel conversion lives in the GDScript
//! renderer (architecture invariant). Replaces the Phase 2 placeholder
//! at `sim_systems::runtime::influence::agent_sample::Position` per the
//! self-documenting landmark.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_coordinates() {
        let p = Position::new(7, 11);
        assert_eq!(p.x, 7);
        assert_eq!(p.y, 11);
    }

    #[test]
    fn equality_holds_for_same_coords() {
        assert_eq!(Position::new(3, 4), Position::new(3, 4));
        assert_ne!(Position::new(3, 4), Position::new(4, 3));
    }

    #[test]
    fn copy_semantics_preserve_value() {
        let a = Position::new(5, 9);
        let b = a;
        assert_eq!(a, b);
    }
}
```

### 3.3 `rust/crates/sim-core/src/components/agent.rs` (NEW)

Zero-sized marker, `Copy + Default + PartialEq + Eq + Debug`.
Personality/Body/Needs/BodyHealth are out of scope (β/γ/δ).

### 3.4 `rust/crates/sim-core/src/lib.rs` (MODIFY)

Add `pub mod components;` and `pub use components::{Agent, Position};`
alongside existing module declarations. No other change.

### 3.5 `rust/crates/sim-engine/src/lib.rs` (MODIFY)

Add `spawn_agent` method on `SimEngine`:

```rust
use hecs::Entity;
use sim_core::components::{Agent, Position};

impl SimEngine {
    pub fn spawn_agent(&mut self, x: u32, y: u32) -> Entity {
        self.world.spawn((Position::new(x, y), Agent))
    }
}
```

### 3.6 `rust/crates/sim-systems/src/runtime/influence/agent_sample.rs` (MIGRATE)

- Remove the local placeholder `pub struct Position { pub x: u32, pub y: u32 }` (lines 23-29).
- Add `pub use sim_core::components::Position;` so existing downstream
  imports (`sim_systems::runtime::influence::agent_sample::Position`)
  continue resolving through this module during transition.
- Update the file-level `//!` header to record that the Phase 4-α
  migration has landed.
- No change to `InfluenceSample`, the sampler system, or any tests in
  the same file.

### 3.7 Migration touchpoints (4 files)

Each swaps `use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};`
→ `use sim_core::components::Position;` + `use sim_systems::runtime::influence::agent_sample::InfluenceSample;`.

- `rust/crates/sim-systems/tests/integration.rs:14`
- `rust/crates/sim-test/tests/harness_phase2.rs:19`
- `rust/crates/sim-test/tests/harness_phase2_substantial.rs:34`
- `rust/crates/sim-test/benches/phase2_benchmarks.rs:29`

No semantic change — the same underlying type, imported through the
canonical path.

### 3.8 `rust/crates/sim-test/tests/harness_p4_alpha_agent_core.rs` (NEW, 11 assertions)

| # | Test name | Asserts |
|---|-----------|---------|
| 1 | `harness_p4_alpha_components_mod_exports` | `sim_core::components::{Position, Agent}` re-exports resolve (compile-time) |
| 2 | `harness_p4_alpha_position_struct_fields_u32` | `Position { x: u32, y: u32 }` field types locked |
| 3 | `harness_p4_alpha_position_new_stores_fields` | `Position::new(5, 9)` → `x==5, y==9` |
| 4 | `harness_p4_alpha_agent_marker_zero_sized` | `size_of::<Agent>() == 0`; `Default + Copy` contract |
| 5 | `harness_p4_alpha_spawn_agent_returns_entity` | `engine.spawn_agent(3,4)` → entity present in `world` |
| 6 | `harness_p4_alpha_spawned_agent_has_position` | spawned entity has `Position` matching args |
| 7 | `harness_p4_alpha_spawned_agent_has_marker` | spawned entity satisfies `(&Position, &Agent)` query tuple |
| 8 | `harness_p4_alpha_multiple_spawns_unique_entities` | 3 spawns → 3 distinct `Entity` handles; `Agent` query count == 3 |
| 9 | `harness_p4_alpha_agent_sample_re_exports_canonical` | `agent_sample::Position == sim_core::components::Position` (compile-time same type) |
| 10 | `harness_p4_alpha_agent_sample_runtime_integration` | stamp `Warmth=99` at (5,5) → swap → spawn agent at (5,5) → IUS priority 110 → tick → `InfluenceSample::warmth == 99` |
| 11 | `harness_p4_alpha_position_equality` | `Position::new(8,8) == Position::new(8,8)`; inequality holds |

Test 9 is the **landmark proof**: a compile-time `assert_same<T>(_:&T,_:&T)` ensures the two import paths yield literally the same `T`.
Test 10 is the **runtime integration proof**: ensures the migrated `agent_sample` sampler still reads correctly through the canonical type.

---

## Section 4 — Locale

No new locale keys. No user-visible UI surface in P4-α.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted P4-α harness
cd rust && cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture

# 3. Phase 2 regression (must remain green — touchpoints refactored)
cd rust && cargo test -p sim-test --test harness_phase2 -- --nocapture
cd rust && cargo test -p sim-test --test harness_phase2_substantial -- --nocapture
cd rust && cargo test -p sim-systems --test integration -- --nocapture

# 4. T7.10.A-F regression (BSS / IUS chain unaffected)
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_f_beauty_bfs_wiring -- --nocapture
```

Expected: 11 new P4-α tests pass + all Phase 2 / T7.10 regressions
green + 0 clippy warnings + 0 build failures.

---

## Section 6 — Lane

`--quick`. Rationale:
- Single-line rewire in `agent_sample.rs` (no semantic change to IUS).
- Two new component files in `sim-core/src/components/` (sibling modules).
- One method on `SimEngine`.
- 4 touchpoint imports refactored mechanically (same type, canonical path).
- One new harness file (11 assertions).
- No FFI surface change (T7.7.B contract locked).
- No scene/shader/locale change.
- No new external dependency (hecs already in sim-core deps).
- Planning debate skipped — the design is fully prescribed by the
  Phase 2 landmark and the Phase 4 plan §2.

---

## Section 7 — In-game verification (post-merge)

P4-α is **backend-only**. Agents are spawned via `SimEngine::spawn_agent`
but do not render in the Godot view yet (sprite rendering arrives in
γ). Verification is via harness tests only; VLM auto-credit applies
(no Godot scope).

After `cargo build -p sim-bridge` + `cargo build -p sim-bridge --release`,
Godot continues to display the same influence buffers as before T7.10.F
(no behavior change). The presence/absence of spawned agents is not yet
visible in the Godot renderer.

---

## Section 8 — Phase 4 dispatch (axiom #1 honesty)

P4-α is **first of four sub-stages**. Honest scope limits:

1. **No movement** — Position is static after spawn. β adds
   `MovementSystem` + `AgentDecision` plumbing.
2. **No personality / body / needs / BodyHealth** — Agent is a marker
   only. β/γ/δ add the cognition + physiology surface.
3. **No sprite rendering** — γ adds Godot sprite layer + FFI snapshot
   extension.
4. **No bridge surface change** — sim-bridge does not yet expose agent
   query. β extends FFI when movement state needs to surface.
5. **No removal of legacy `agent_sample::Position` path** — the
   `pub use` keeps it alive during transition. A later sub-stage will
   delete the alias after all downstream code is migrated to the
   canonical path.

---

## Section 9 — Out of scope

- Movement / pathfinding / AgentDecision (β)
- Personality / Body / Needs / BodyHealth (β/γ/δ)
- Sprite rendering / sim-bridge agent snapshot (γ)
- Removal of the `agent_sample::Position` `pub use` alias (post-β)
- Any FFI surface change (T7.7.B contract locked)
- Any scene / shader / locale change
- Performance benchmarks for spawn throughput (not yet a budget
  concern — Phase 1-target ~500 agents, architecture for 10,000+ per
  CLAUDE.md)
