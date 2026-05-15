# P5-β — Thirst + AgentState FSM + AgentDecisionSystem + CausalEvent::AgentDecision + food/water tile substrate (V7 Phase 5 sub-stage)

> Lane: `--full` (sim-core + sim-engine + sim-systems + sim-bridge + sim-test; CausalEvent enum extension forces full lane)
> Scope: First agent-originated causal event lands. Agents transition
> `Idle → Seeking { target } → Consuming { target } → Idle` driven by
> Hunger/Thirst threshold breach. The chain
> `HungerThresholdBreach → Seeking → Consuming` is observable in the
> causal log via the existing CausalPanel ("왜?" UI) substrate.
> Governance: v3.3.16. Visual: γ sprite regression check only (backend-leaning, no new HUD surface).

---

## Section 1 — Implementation Intent

Phase 5-α delivered `AgentId`, `Hunger`, and the priority-130
`HungerDecaySystem`. β opens the **first agent-originated causal chain**
by adding:

1. The second need (`Thirst`, mirroring Hunger contract) and its
   priority-131 decay system.
2. An explicit per-agent FSM (`AgentState { Idle | Seeking | Consuming }`)
   that drives goal-directed behavior; β scope is **threshold-based
   transition only — no pathing** (Seeking suppresses Brownian motion,
   Consuming triggers when the agent already stands on a matching tile).
3. A new `AgentDecisionSystem` at priority **125** (between movement@120
   and hunger decay@130) — the canonical FSM tick.
4. A new `CausalEvent::AgentDecision` variant carrying
   `{ id, parent, agent: AgentId, position, reason: DecisionReason, tick }`
   so every threshold breach becomes a traceable causal node.
5. A sparse food/water tile substrate on `SimResources`
   (`food_tiles: HashMap<(u32,u32), u8>` and `water_tiles`) — the minimum
   resource layer needed to close the consume loop.
6. FFI exposure: extend `CausalEventView` with `agent_id` and `reason`
   fields so γ HUD work can read the new variant without an FFI revision.

After P5-β:
- `sim_core::components::{Thirst, AgentState, TargetKind}` adds the
  need + FSM primitives.
- `sim_core::causal::CausalEvent::AgentDecision` + `DecisionReason {
  HungerThresholdBreach, ThirstThresholdBreach }` enter the causal log
  type set.
- `sim_systems::runtime::decision::AgentDecisionSystem` (priority 125)
  is the canonical FSM driver; `sim_systems::register_decision_systems`
  is the registration helper (parallel to `register_needs_systems`).
- `AgentMovementSystem` queries the optional `AgentState` and skips
  Brownian steps when the agent is `Seeking { .. }` (movement
  suppression). Idle and Consuming agents still wander.
- `CausalEventView` (FFI surface) gains two `Option<...>` fields
  (`agent_id: Option<u64>`, `reason: Option<&'static str>`) so existing
  consumers stay source-compatible.

---

## Section 2 — Locked facts from pre-grep (must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P5β-1: Thirst contract | Plan §Phase 5-β | `Thirst { value: f32, growth_rate: f32 }`, `SATURATION = 100.0`, mirrors Hunger semantics exactly (clamp + saturate + floor) |
| P5β-2: AgentState FSM | Plan §Phase 5-β | `enum AgentState { Idle, Seeking { target: TargetKind }, Consuming { target: TargetKind } }`, `Default = Idle`, derives Clone/Copy/PartialEq/Eq/Serde |
| P5β-3: TargetKind | Plan §Phase 5-β | `enum TargetKind { Food, Water }` — exactly two variants in β |
| P5β-4: DecisionReason | Plan §Phase 5-β | `enum DecisionReason { HungerThresholdBreach, ThirstThresholdBreach }` (Sleep deferred to γ); `fn as_str(&self) -> &'static str` discriminator returns `"hunger_threshold_breach"` / `"thirst_threshold_breach"` |
| P5β-5: AgentDecisionSystem | Plan §Phase 5-β | `priority = 125`, `tick_interval = 1`, `name = "AgentDecisionSystem"`; queries `(&mut AgentState, &Hunger, &Thirst, &Position)` |
| P5β-6: ThirstDecaySystem | Plan §Phase 5-β | `priority = 131`, `tick_interval = 1`, `name = "ThirstDecaySystem"` (mirrors HungerDecaySystem) |
| P5β-7: Tile substrate | Plan §Phase 5-β ("α-implementer chooses") | Sparse `HashMap<(u32,u32), u8>` on `SimResources`: `food_tiles` + `water_tiles`. Insert via `set_food_tile(x, y, amount)` / `set_water_tile(x, y, amount)`. Decrement on Consume, remove entry when `0`. |
| P5β-8: Thresholds + amounts | Plan §Phase 5-β + locked dispatch | `HUNGER_THRESHOLD = 50.0`, `THIRST_THRESHOLD = 50.0`, `HUNGER_CONSUME_AMOUNT = 30.0`, `THIRST_CONSUME_AMOUNT = 30.0` |
| Priority slot | Plan §Phase 5-β | `90 BSS → 100 IUS → 110 sample → 120 move → 125 decide → 130 hunger_decay → 131 thirst_decay → 1000 viz` |
| FSM transition rule | Plan §Phase 5-β | If `state == Idle && hunger.value > 50` → `Seeking { Food }`. Else if `state == Idle && thirst.value > 50` → `Seeking { Water }`. Hunger wins on ties (deterministic). |
| Seeking → Consuming | β scope | Triggered when agent's `Position` matches a `food_tiles` / `water_tiles` entry. Same-tick transition (no pathing). |
| Consuming → Idle | β scope | One-tick consume: decrements `Hunger`/`Thirst` by amount, decrements tile counter (and removes if 0), state → `Idle`. |
| Movement suppression | Plan §Phase 5-β | `AgentMovementSystem::tick` queries `Option<&AgentState>`; if `state.suppresses_movement()` returns true (Seeking only), agent skips Brownian step. Idle and Consuming still move. |
| Causal parent linkage | Plan §Phase 5-β causal-chain block | `parent = Some(<latest same-tile InfluenceChanged id>)` if any in log, else `None`. β-scope simple linear scan of `world.causal_log` — no per-tile index. |
| CausalEventView FFI | Plan §Phase 5-β + FFI minimality | Add `pub agent_id: Option<u64>` + `pub reason: Option<&'static str>`; existing 3 variant arms keep `agent_id: None, reason: None`. `event_view_to_dict` emits both keys when present. |
| Harness assertion count | Axiom #1 (≥10) | **15 assertions** (5 above floor) |

**Determinism rationale** (axiom #2 — verify before assuming):
1. `AgentDecisionSystem::tick` iterates a `world.query` and only mutates
   `AgentState`; the iteration order is stable per hecs's archetype
   layout and tick scheduling is sequential.
2. The "Hunger wins thirst on ties" rule is unconditional — no RNG
   consumed; FSM transitions are pure arithmetic + comparison.
3. Causal parent linkage is a deterministic linear scan over the
   existing `world.causal_log` ring buffer; no allocation, no RNG.
4. Tile substrate uses `HashMap<(u32,u32), u8>` — lookups/insertions
   are deterministic; iteration is **not** required (Consuming uses
   point lookup on `Position`).

---

## Section 3 — What to build (14-file scope)

### 3.1 `rust/crates/sim-core/src/components/thirst.rs` (NEW)

Mirror of `hunger.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Thirst {
    pub value: f32,
    pub growth_rate: f32,
}

impl Thirst {
    pub const SATURATION: f32 = 100.0;
    pub fn new(initial: f32, growth_rate: f32) -> Self { /* clamp [0, SATURATION] */ }
    pub fn tick(&mut self) { /* add growth_rate, saturate, floor */ }
}
```

6 inline unit tests covering construct clamp (low + high), tick
saturate cap, tick floor at zero, multi-tick monotonic growth, serde
round-trip, `SATURATION` const sanity.

### 3.2 `rust/crates/sim-core/src/components/agent_state.rs` (NEW)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind { Food, Water }

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    #[default]
    Idle,
    Seeking { target: TargetKind },
    Consuming { target: TargetKind },
}

impl AgentState {
    pub fn target(&self) -> Option<TargetKind> { /* Some for Seeking/Consuming, None for Idle */ }
    pub fn suppresses_movement(&self) -> bool { matches!(self, AgentState::Seeking { .. }) }
}
```

4 inline unit tests: default == Idle, target() helper coverage,
suppresses_movement() truth table, serde round-trip for all variants
+ TargetKind round-trip.

### 3.3 `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

Add:
```rust
pub mod agent_state;
pub mod thirst;
pub use agent_state::{AgentState, TargetKind};
pub use thirst::Thirst;
```

### 3.4 `rust/crates/sim-core/src/causal/event.rs` (MODIFY)

Extend `CausalEvent` enum with:
```rust
AgentDecision {
    id: EventId,
    parent: Option<EventId>,
    agent: AgentId,
    position: (u32, u32),
    reason: DecisionReason,
    tick: u64,
},
```

Add new public enum:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionReason {
    HungerThresholdBreach,
    ThirstThresholdBreach,
}

impl DecisionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionReason::HungerThresholdBreach => "hunger_threshold_breach",
            DecisionReason::ThirstThresholdBreach => "thirst_threshold_breach",
        }
    }
}
```

Extend existing accessor methods (`id()`, `parent()`, `tick()`,
`channel()`) to handle the new variant: `channel()` returns `None`
for AgentDecision; others extract the matching field.

### 3.5 `rust/crates/sim-core/src/causal/mod.rs` (MODIFY)

Re-export `DecisionReason` alongside existing `CausalEvent`/`EventId`.

### 3.6 `rust/crates/sim-engine/src/lib.rs` (MODIFY)

- Add `use std::collections::HashMap;`
- On `SimResources`: add
  ```rust
  pub food_tiles: HashMap<(u32, u32), u8>,
  pub water_tiles: HashMap<(u32, u32), u8>,
  ```
- Initialize both as empty `HashMap::new()` in `SimResources::new`.
- Add helpers:
  ```rust
  pub fn set_food_tile(&mut self, x: u32, y: u32, amount: u8) { ... }
  pub fn set_water_tile(&mut self, x: u32, y: u32, amount: u8) { ... }
  ```
  (amount=0 → remove entry; otherwise insert/overwrite.)

### 3.7 `rust/crates/sim-systems/src/runtime/needs/thirst_decay.rs` (NEW)

Mirror of `hunger_decay.rs`:
- `ThirstDecaySystem` ZST, `Default + new()`.
- `RuntimeSystem`: name="ThirstDecaySystem", priority=131,
  tick_interval=1, queries `&mut Thirst` and calls `Thirst::tick()`.
- 3 inline unit tests (metadata / advances value / saturation cap).

### 3.8 `rust/crates/sim-systems/src/runtime/needs/mod.rs` (MODIFY)

Add `pub mod thirst_decay; pub use thirst_decay::ThirstDecaySystem;`.

### 3.9 `rust/crates/sim-systems/src/runtime/decision/mod.rs` (NEW)

```rust
//! V7 Phase 5-β — Agent decision FSM driver.
pub mod agent_decision;
pub use agent_decision::AgentDecisionSystem;
```

### 3.10 `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (NEW)

- `AgentDecisionSystem` ZST, `Default + new()`.
- `RuntimeSystem`: name="AgentDecisionSystem", priority=125,
  tick_interval=1.
- `tick()` logic (per entity with `&mut AgentState`, `&Hunger`,
  `&Thirst`, `&Position`):
  - **Idle**:
    - If `hunger.value > HUNGER_THRESHOLD` → `Seeking { Food }`,
      emit `CausalEvent::AgentDecision { reason: HungerThresholdBreach, ... }`.
    - Else if `thirst.value > THIRST_THRESHOLD` → `Seeking { Water }`,
      emit `CausalEvent::AgentDecision { reason: ThirstThresholdBreach, ... }`.
  - **Seeking { Food }**:
    - If `resources.food_tiles.contains_key(&(pos.x, pos.y))` →
      `Consuming { Food }`. (No event emit — purely state transition.)
  - **Seeking { Water }**: symmetric on `water_tiles`.
  - **Consuming { Food }**:
    - Decrement `food_tiles[(x,y)]` by 1 (remove on 0).
    - Subtract `HUNGER_CONSUME_AMOUNT` from `Hunger.value` (clamp at 0).
    - `state → Idle`.
  - **Consuming { Water }**: symmetric on `water_tiles` + Thirst.
- Causal parent linkage: linear scan of `world.causal_log` for most
  recent `InfluenceChanged` at the same `(x,y)`, take its `id` (or
  `None` if not present); use as `parent`.
- Constants exported as `pub const` at module scope:
  `HUNGER_THRESHOLD = 50.0`, `THIRST_THRESHOLD = 50.0`,
  `HUNGER_CONSUME_AMOUNT = 30.0`, `THIRST_CONSUME_AMOUNT = 30.0`.
- 3 inline unit tests: metadata, Idle→Seeking on hunger breach,
  Seeking→Consuming when on food tile.

### 3.11 `rust/crates/sim-systems/src/runtime/mod.rs` (MODIFY)

Add `pub mod decision;` alongside existing modules. Doc-comment update
noting Phase 5-β landing.

### 3.12 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

- Doc updated to mention ThirstDecaySystem (131) and
  AgentDecisionSystem (125).
- `register_needs_systems` now registers BOTH HungerDecaySystem AND
  ThirstDecaySystem (single call, both needs).
- New top-level helper:
  ```rust
  pub fn register_decision_systems(engine: &mut SimEngine) {
      engine.register_system(Box::new(runtime::decision::AgentDecisionSystem::new()));
  }
  ```

### 3.13 `rust/crates/sim-systems/src/runtime/agent/movement.rs` (MODIFY)

- Import `AgentState`.
- Extend query to `(&mut Position, &mut MovementRng, Option<&AgentState>)`.
- Skip the step (`continue`) when
  `state.is_some_and(|s| s.suppresses_movement())`.
- Update priority-table doc-comment to add "125 AgentDecisionSystem"
  row and note "Phase 5-β" landing.
- Add 3 new tests: `seeking_state_suppresses_movement`,
  `idle_state_still_moves`, `consuming_state_still_moves`.

### 3.14 `rust/crates/sim-bridge/src/ffi/world_node.rs` (MODIFY)

- Extend `CausalEventView` struct with two `Option<...>` fields
  (additive, source-compatible):
  ```rust
  pub agent_id: Option<u64>,
  pub reason: Option<&'static str>,
  ```
- All 3 existing match arms in `CausalEventView::from_event` extended
  with `agent_id: None, reason: None`.
- New arm for `CausalEvent::AgentDecision`:
  ```rust
  CausalEvent::AgentDecision { id, parent, agent, position, reason, tick } => Self {
      kind: "agent_decision",
      id: *id,
      parent: *parent,
      tick: *tick,
      channel: None,
      position: Some(*position),
      radius: None, region: None, old_value: None, new_value: None,
      agent_id: Some(*agent),
      reason: Some(reason.as_str()),
  },
  ```
- `event_view_to_dict` extended to emit `agent_id` (as i64) and
  `reason` (string) keys when their `Option` is `Some`.

### 3.15 `rust/crates/sim-test/tests/harness_p5_beta_decision.rs` (NEW, 15 assertions)

| # | Test name | Asserts | Type |
|---|-----------|---------|:----:|
| 1 | `harness_p5_beta_thirst_clamps_and_ticks` | Thirst::new clamp + tick saturates + tick floor | A |
| 2 | `harness_p5_beta_agent_state_default_is_idle` | `AgentState::default() == Idle` | A |
| 3 | `harness_p5_beta_suppresses_movement_truth_table` | Only Seeking suppresses | A |
| 4 | `harness_p5_beta_causal_event_view_maps_agent_decision` | FFI mapping: kind, agent_id, reason, channel=None | A |
| 5 | `harness_p5_beta_decision_reason_discriminator` | as_str() returns expected strings | A |
| 6 | `harness_p5_beta_causal_event_agent_decision_accessors` | id/parent/tick/channel=None + root parent=None | A |
| 7 | `harness_p5_beta_agent_decision_system_metadata` | name/priority=125/interval=1 | A |
| 8 | `harness_p5_beta_thirst_decay_system_metadata` | priority=131/interval=1 | A |
| 9 | `harness_p5_beta_idle_to_seeking_food_on_hunger_breach` | Hunger > threshold → Seeking { Food } | A |
| 10 | `harness_p5_beta_idle_to_seeking_water_when_only_thirst_breaches` | Thirst > threshold + low Hunger → Seeking { Water } | A |
| 11 | `harness_p5_beta_consuming_food_decrements_both` | Tile counter and Hunger both decrease; state → Idle | A |
| 12 | `harness_p5_beta_movement_skipped_for_seeking_agent` | Position unchanged after 32 ticks while Seeking | A |
| 13 | `harness_p5_beta_decision_event_recorded` | CausalEvent::AgentDecision in log at agent's tile, parent=None | D |
| 14 | `harness_p5_beta_hunger_decay_regression` | HungerDecaySystem still works post-β changes | D |
| 15 | `harness_p5_beta_consuming_water_decrements_both` | Water tile (1→0 removal) and Thirst both decrease | A |

Test 13 is the **causal chain landmark**. Tests 14 is the **regression**
guard for Phase 5-α work.

### 3.16 `rust/crates/sim-test/tests/harness_p3_alpha_event_recording.rs` (MIGRATE, 3 sites)

Three exhaustive `match ev` blocks must add the new variant. Use
`replace_all: true` on:
```rust
| CausalEvent::AgentDecision { tick, .. } => *tick,
```

---

## Section 4 — Locale

No new locale keys in β. Causal-panel locale keys
(`causal.event.agent_decision`, `causal.reason.hunger_threshold_breach`,
etc.) land with γ when the HUD wires the new variant. β leaves the
existing CausalPanel UI unchanged — the new event simply exists in the
log substrate.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted P5-β harness
cd rust && cargo test -p sim-test --test harness_p5_beta_decision -- --nocapture

# 3. Phase 5-α regression
cd rust && cargo test -p sim-test --test harness_p5_alpha_agent_id_hunger -- --nocapture

# 4. Phase 4 regression (α/β/γ contracts intact)
cd rust && cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture
cd rust && cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture
cd rust && cargo test -p sim-test --test harness_p4_gamma_sprite_rendering -- --nocapture

# 5. Phase 3-α causal-log regression (exhaustive match site migration)
cd rust && cargo test -p sim-test --test harness_p3_alpha_event_recording -- --nocapture
```

Expected: 15 new P5-β tests pass + all P5-α / P4-α/β/γ / P3-α
regressions green + 0 clippy warnings + 0 build failures.

---

## Section 6 — Lane

`--full`. Rationale:
- Public CausalEvent enum extended — every `match` site must add the
  new arm or the build fails. This cascades through `sim-core`,
  `sim-bridge`, and `sim-test`.
- Four crates modified (sim-core components + causal + sim-engine
  resources + sim-systems runtime + sim-bridge FFI).
- New components (`Thirst`, `AgentState`, `TargetKind`) and new system
  module (`decision`).
- FFI surface gains two `Option<...>` fields on `CausalEventView`.
- Backend-leaning: no new shader, no new HUD panel. γ sprite
  rendering should regression-pass unchanged.

---

## Section 7 — In-game verification (post-merge)

P5-β is **backend-only by visual scope**. Agents will visibly stop
moving when they enter `Seeking` (because Brownian motion is
suppressed), then resume motion after `Consuming → Idle` completes.
No new HUD readout, no causal-panel variant icon (γ work).

VLM verification: γ sprite rendering unchanged, no rendering crash,
no regression in tile draw / building draw / overlay draw.

---

## Section 8 — Phase 5-β scope honesty (axiom #1)

P5-β is the **second of multiple Phase 5 sub-stages**. Honest scope
limits:

1. **No pathing** — Seeking suppresses Brownian motion but the agent
   does not walk toward a target. Consuming triggers only when the
   agent's `Position` already matches a food/water tile. Goal-directed
   pathing lands in γ (or later) with the day/night clock and influence
   gradient sampling.
2. **No PRS-driven thresholds** — `HUNGER_THRESHOLD = 50.0` and
   `THIRST_THRESHOLD = 50.0` are hard constants. Temperament-modulated
   thresholds land later (planning §Phase 6).
3. **No Sleep target** — γ adds `TargetKind::Sleep` and a day/night
   clock. β stays at exactly two targets.
4. **No CausalPanel UI surface for AgentDecision** — the event exists
   in the log and is exposed via `CausalEventView` (FFI), but the HUD
   does not render a new icon/label yet. γ wires the UI.
5. **No locale keys** — γ adds `causal.event.agent_decision` and
   `causal.reason.*` keys when wiring the UI.
6. **No save/load coverage** — `AgentState`, `Thirst`, and the food/water
   tile maps all derive Serialize/Deserialize, but no save-path harness
   is added in β.
7. **No multi-tile consume** — one Consume tick decrements by exactly
   one (tile) and the configured amount (need). Multi-tick consumption
   (e.g., gnawing on a stockpile) is deferred.

---

## Section 9 — Out of scope

- Goal-directed pathing toward food/water tiles (Phase 5-γ)
- TargetKind::Sleep + day/night clock (Phase 5-γ)
- CausalPanel UI rendering of AgentDecision variant (Phase 5-γ)
- Locale keys for new causal events (Phase 5-γ)
- PRS / temperament-modulated thresholds (Phase 6)
- LodTier throttling for AgentDecisionSystem
- Save/load harness for AgentState / Thirst / tile maps
- Bridge FFI: agent-side reads of `AgentState` (γ scope when HUD needs it)
- Performance benchmarks for decision system throughput
- Food/water spawning mechanic (β only adds the substrate + helpers;
  test scenarios populate tiles manually)
