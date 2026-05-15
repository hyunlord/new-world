# P5-α — AgentId infrastructure + Hunger component + HungerDecaySystem (V7 Phase 5 sub-stage)

> Lane: `--full` (sim-core + sim-engine + sim-systems + sim-test; no GDScript, no FFI surface change)
> Scope: First Phase 5 sub-stage. Promote `Agent` from a ZST marker to a stable-id
> carrier, introduce the first need component `Hunger`, and land the priority-130
> `HungerDecaySystem`. Eat-action behavior is deferred — see Section 8.
> Governance: v3.3.16. Visual: γ sprite regression check only (backend-leaning).

---

## Section 1 — Implementation Intent

Phase 4 closed with `Agent` as a ZST marker (α), Brownian motion (β), and
sprite rendering (γ). Phase 5 opens "First Daily Routine" by giving each
agent (a) a **stable identity** and (b) its **first need**.

Planning v7 §Phase 5 prescribes:
- `AgentId = u64` — opaque public identifier, monotonically minted.
- `Agent { id: AgentId }` — struct upgrade from ZST; `Default` dropped to
  avoid a zero-id default colliding with the first issued id.
- `next_agent_id: AtomicU64` on `SimResources` — minted via
  `issue_agent_id()` helper (mirrors Phase 3-β `next_event_id`).
- `Hunger { value: f32, growth_rate: f32 }` with `SATURATION = 100.0`.
  Value clamped to `[0.0, 100.0]`; `tick()` adds `growth_rate` and
  saturates.
- `HungerDecaySystem` priority 130, every tick — sits between
  `AgentMovementSystem` (120) and `InfluenceVisualizationSystem` (1000).

After P5-α:
- `sim_core::components::{Agent, AgentId, Hunger}` exposes the new
  identity + need primitives.
- `SimEngine::spawn_agent` mints a fresh id via `issue_agent_id` and
  attaches `Agent { id }`. β determinism is preserved (id minting does
  not touch RNG state).
- `sim_systems::runtime::needs::HungerDecaySystem` is the canonical
  hunger driver; `register_needs_systems(&mut SimEngine)` is the
  registration helper (parallel to `register_agent_systems`).
- No eat-action exists. Hunger climbs unbounded toward saturation; γ
  sprite rendering is unchanged.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P5α-1: AgentId type | Planning §Phase 5 + axiom #2 | `pub type AgentId = u64;` re-export from `components::agent` |
| P5α-2: Agent struct | Planning §Phase 5 | `pub struct Agent { pub id: AgentId }` — `Copy + Eq + Serialize + Deserialize`; **`Default` dropped** |
| P5α-3: Counter | Planning §Phase 5 (mirrors `next_event_id`) | `pub next_agent_id: AtomicU64` on `SimResources`; `issue_agent_id(&self) -> AgentId` uses `fetch_add(1, Ordering::Relaxed)` |
| P5α-4: Hunger semantics | Planning §Phase 5 | `Hunger { value: f32, growth_rate: f32 }`, `const SATURATION: f32 = 100.0`, `new(initial, rate)` clamps initial to `[0, SATURATION]`, `tick()` saturates |
| P5α-5: HungerDecaySystem | Planning §Phase 5 | `priority = 130`, `tick_interval = 1`, `name = "HungerDecaySystem"`, queries `&mut Hunger`, calls `Hunger::tick()` |
| Harness assertion count | Axiom #1 (≥11) | **12 assertions** (1 above floor) |
| spawn_agent contract | P4-α LOCKED + P5α-2 | mints id via `resources.issue_agent_id()`, spawns `(Position, Agent { id })` — Hunger NOT auto-attached |
| Counter ordering | Mirrors `next_event_id` | `Ordering::Relaxed` (single-writer per tick, no cross-thread contention) |
| Module path | LOCKED design | `sim_systems::runtime::needs::HungerDecaySystem` + `runtime/needs/mod.rs` |
| Register helper | Mirrors `register_agent_systems` | `pub fn register_needs_systems(engine: &mut SimEngine)` |
| Priority ordering | β doc-comment table | 90 → 100 → 110 → 120 → **130 (new)** → 1000 |
| Cascade comment fix | β doc-comment | `agent/movement.rs` priority table updates "130 DEFERRED" → "130 LANDED in V7 Phase 5-α" |
| α harness migration | P4-α tests | `Agent` literal → `Agent { id: 0 }` everywhere; `harness_p4_alpha_agent_marker_zero_sized` asserts `size_of::<Agent>() == size_of::<AgentId>()` |
| β harness migration | P4-β tests | All `world.spawn((..., Agent, ...))` patterns → `Agent { id: N }` (literal per spawn, no collision concern in β) |

**Determinism rationale** (axiom #2 — verify before assuming):
1. `AtomicU64::fetch_add(1, Relaxed)` is deterministic under single-writer
   per-tick patterns. Sim tick scheduling is sequential by design — no
   inter-system parallelism inside one tick — so the issued id sequence
   replays byte-identically.
2. Id minting consumes no movement RNG state. β determinism (per-agent
   `splitmix64`) is preserved without modification.
3. `Hunger::tick()` is a pure arithmetic + clamp — no RNG, no I/O.

---

## Section 3 — What to build

### 3.1 `rust/crates/sim-core/src/components/agent.rs` (REWRITE)

ZST → struct upgrade:

```rust
pub type AgentId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
}
```

`Default` derive intentionally dropped (a zero-id default would collide
with the first id minted by `SimResources::issue_agent_id`).

Internal unit tests: `agent_carries_public_id`, `agent_is_copy_and_eq`,
`serde_round_trip_preserves_id`, `agent_size_equals_agent_id`.

### 3.2 `rust/crates/sim-core/src/components/hunger.rs` (NEW)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hunger {
    pub value: f32,
    pub growth_rate: f32,
}

impl Hunger {
    pub const SATURATION: f32 = 100.0;
    pub fn new(initial: f32, growth_rate: f32) -> Self { /* clamp */ }
    pub fn tick(&mut self) { /* add growth_rate, saturate at SATURATION, floor at 0 */ }
}
```

6 inline unit tests covering construction clamp, saturation cap, floor at
zero, multi-tick monotonic growth, serde round-trip, and `SATURATION`
constant.

### 3.3 `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

Add `pub mod hunger;`, `pub use agent::{Agent, AgentId};`,
`pub use hunger::Hunger;`. Crate `lib.rs` re-export already covers the
`components` module path.

### 3.4 `rust/crates/sim-engine/src/lib.rs` (MODIFY, 5 edits)

- Add `use sim_core::components::{Agent, AgentId, Position};` (Agent now
  used by spawn_agent body).
- Add `pub next_agent_id: AtomicU64` field on `SimResources`.
- Initialize `next_agent_id: AtomicU64::new(0)` in `SimResources::new`.
- Add `pub fn issue_agent_id(&self) -> AgentId { self.next_agent_id.fetch_add(1, Ordering::Relaxed) }`.
- Rewrite `SimEngine::spawn_agent`:
  ```rust
  pub fn spawn_agent(&mut self, x: u32, y: u32) -> Entity {
      let id = self.resources.issue_agent_id();
      self.world.spawn((Position::new(x, y), Agent { id }))
  }
  ```

### 3.5 `rust/crates/sim-systems/src/runtime/needs/mod.rs` (NEW)

```rust
//! V7 Phase 5-α — Agent needs systems.
//!
//! First need: [`HungerDecaySystem`] (priority 130). Future Phase 5
//! sub-stages will add Thirst, Sleep, Energy.

pub mod hunger_decay;

pub use hunger_decay::HungerDecaySystem;
```

### 3.6 `rust/crates/sim-systems/src/runtime/needs/hunger_decay.rs` (NEW)

- `HungerDecaySystem` — zero-sized, `Default + new()`.
- `RuntimeSystem`: `name = "HungerDecaySystem"`, `priority = 130`,
  `tick_interval = 1`, `tick()` queries `&mut Hunger` and calls
  `Hunger::tick()` per entity.
- 3 inline unit tests: metadata / single-agent tick advances hunger /
  saturation cap respected after many ticks.

### 3.7 `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod needs;` alongside existing `pub mod agent; pub mod influence;`.
Update file-level doc to record "Phase 5-α land".

### 3.8 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

Add `register_needs_systems(&mut SimEngine)` that registers
`HungerDecaySystem` only. Update crate doc to reference the new helper.
Do **not** call it from `register_phase2_systems` or
`register_agent_systems` — composition is the test/runtime responsibility.

### 3.9 `rust/crates/sim-systems/src/runtime/agent/movement.rs` (MODIFY, doc-comment only)

Update the priority-table comment:

```text
130  HungerDecaySystem           ← LANDED in V7 Phase 5-α
```

(Was previously "DEFERRED" or absent — now reflects reality.)

### 3.10 `rust/crates/sim-test/tests/harness_p4_alpha_agent_core.rs` (MIGRATE)

Two surgical edits:
- The `_ = Agent;` runtime witness → `_ = Agent { id: 0 };`.
- `harness_p4_alpha_agent_marker_zero_sized` test body: assertion changes
  from `size_of::<Agent>() == 0` to
  `size_of::<Agent>() == size_of::<AgentId>()`; remove
  `Agent::default()` call (Default derive dropped). Doc comment updated
  to describe the new contract.

### 3.11 `rust/crates/sim-test/tests/harness_p4_beta_movement.rs` (MIGRATE)

All `world.spawn((..., Agent, ...))` tuple-spawn sites → `Agent { id: N }`
with a literal id per spawn. No β test queries by id, so collisions are
irrelevant — only the type must satisfy the struct shape.

### 3.12 `rust/crates/sim-test/tests/harness_p5_alpha_agent_id_hunger.rs` (NEW, 12 assertions)

| # | Test name | Asserts | Type |
|---|-----------|---------|:----:|
| 1 | `harness_p5_alpha_agent_id_type_resolves` | `let _: AgentId = 0u64;` compile-time alias check | A |
| 2 | `harness_p5_alpha_agent_struct_carries_id` | `Agent { id: 42 }.id == 42` | A |
| 3 | `harness_p5_alpha_spawn_agent_stamps_unique_ids` | two `spawn_agent` calls → distinct `Agent::id`, monotonic +1 | A |
| 4 | `harness_p5_alpha_issue_agent_id_advances_counter` | `predict = issue_agent_id()`; `spawn = spawn_agent()` → `spawn.id == predict + 1` | A |
| 5 | `harness_p5_alpha_hunger_new_clamps_initial` | `Hunger::new(-5.0, ...).value == 0.0` and `Hunger::new(999.0, ...).value == SATURATION` | A |
| 6 | `harness_p5_alpha_hunger_tick_saturates` | `Hunger::new(95.0, 10.0)` after 1 tick → `value == SATURATION` | A |
| 7 | `harness_p5_alpha_hunger_tick_floor_at_zero` | negative growth_rate cannot drive value below 0 | A |
| 8 | `harness_p5_alpha_hunger_decay_system_metadata` | name/priority/tick_interval | A |
| 9 | `harness_p5_alpha_hunger_decay_advances_value` | 1 agent with `Hunger::new(0.0, 1.0)`, 5 ticks via system → `value == 5.0` | A |
| 10 | `harness_p5_alpha_register_needs_systems_registers` | `register_needs_systems(&mut engine)` + 1 spawn + Hunger attach + 3 advance ticks → hunger value > 0 | D |
| 11 | `harness_p5_alpha_full_stack_phase2_agent_needs` | register all 3 stacks (phase2 + agent + needs), 1 agent at (16,16) with Hunger and MovementRng, 8 advance ticks → id stable + position moved + hunger == 8.0 | D |
| 12 | `harness_p5_alpha_agent_size_equals_agent_id` | `size_of::<Agent>() == size_of::<AgentId>()` (no padding; ZST → 8 bytes) | A |

Test 4 is the **counter contract landmark**: proves
`issue_agent_id` and `spawn_agent` share the same `AtomicU64` counter.
Test 11 is the **integration regression**: ensures the full system stack
co-exists (Phase 2 sampler@110 + AMS@120 + HungerDecay@130) without
priority collisions or state corruption.

---

## Section 4 — Locale

No new locale keys. No user-visible UI surface in P5-α — Hunger has no
HUD display yet.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted P5-α harness
cd rust && cargo test -p sim-test --test harness_p5_alpha_agent_id_hunger -- --nocapture

# 3. Phase 4 regression (α/β/γ contracts intact under struct migration)
cd rust && cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture
cd rust && cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture
cd rust && cargo test -p sim-test --test harness_p4_gamma_sprite_rendering -- --nocapture

# 4. Phase 2 / T7.10 regression (priority 130 doesn't collide with 110/1000)
cd rust && cargo test -p sim-test --test harness_phase2 -- --nocapture
cd rust && cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture
```

Expected: 12 new P5-α tests pass + all P4-α / P4-β / P4-γ / Phase 2 /
T7.10 regressions green + 0 clippy warnings + 0 build failures.

---

## Section 6 — Lane

`--full`. Rationale:
- Three crates modified (sim-core components + sim-engine resources +
  sim-systems runtime).
- Public API change: `Agent` shape (ZST → struct).
- New `SimResources` field (`next_agent_id`) — affects every consumer of
  resources, even read-only ones.
- New top-level component (`Hunger`) and new runtime module (`needs`).
- α/β harness migration touches existing assertion contracts.
- Planning debate is short (planning §Phase 5 prescribes types
  explicitly), but evaluator review is warranted because the public
  API change cascades.

---

## Section 7 — In-game verification (post-merge)

P5-α is **backend-only** by design. Hunger climbs in the simulation but
has no HUD readout yet. After `cargo build -p sim-bridge`:

- γ sprite regression: agents must still render at their `Position` —
  the `Agent` struct migration does NOT break the Bridge FFI snapshot
  format (Bridge does not currently serialize `Agent.id`).
- No new visual surface. VLM verifies that γ rendering is unchanged
  (agents visible, no crash, no regression).

---

## Section 8 — Phase 5 dispatch (axiom #1 honesty)

P5-α is **first of multiple Phase 5 sub-stages**. Honest scope limits:

1. **No eat-action behavior** — Hunger climbs but no agent consumes
   food. β will add `EatAction` decision + food-source seeking.
2. **No PRS-driven growth_rate** — `growth_rate` is a user-provided
   constant per agent. Temperament-modulated rates land in a later
   sub-stage (planning §Phase 6).
3. **No SatietyDecaySystem** — only growth (hunger increase). The
   counterpart (satiety after eating) lands with the eat-action.
4. **No Bridge FFI surface for Hunger** — `sim-bridge` snapshot is
   unchanged; HUD displays nothing yet. γ visual scope unchanged.
5. **No causal-event recording of hunger transitions** — `CausalEvent`
   stays at its current variant set (3 from Phase 4 area; α did not
   extend it).
6. **No LodTier-based tick throttling for HungerDecaySystem** — every
   agent ticks every tick at priority 130. Headroom is ample at the
   Phase 1 target of ~500 agents.
7. **No save/load coverage** — `Hunger` derives `Serialize`/`Deserialize`
   but no save-path harness is added in α.

---

## Section 9 — Out of scope

- EatAction component / behavior / FFI (P5-β)
- Food-source seeking, food entity, food influence channel (P5-β)
- Temperament-modulated growth_rate (Phase 6)
- SatietyDecaySystem / post-eat recovery curve (P5-β)
- Bridge FFI snapshot extension for Hunger (P5-β or γ)
- Causal-event recording for hunger transitions
- LodTier throttling for needs systems
- Save/load harness for Agent.id and Hunger persistence
- Thirst / Sleep / Energy needs (later Phase 5 sub-stages)
- Any scene / shader / locale change
- Performance benchmarks for needs system throughput
