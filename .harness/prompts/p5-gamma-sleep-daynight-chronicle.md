# P5-γ — Sleep + day/night cycle + chronicle harness (V7 Phase 5 closure milestone)

> Lane: `--full` (sim-core + sim-engine + sim-systems + sim-test; CausalEvent::DecisionReason enum extension + SimResources schema extension force full lane)
> Scope: Third and final Phase 5 sub-stage. Adds the third need (`Sleep`),
> the simulation-wide day/night clock (`time_of_day`, `ticks_per_day = 1440`),
> the priority-132 `SleepDecaySystem`, the `FatigueThresholdBreach`
> decision reason, the `TargetKind::Sleep` variant **(Path b symmetry —
> matches Plan §2.3 line 286-289 explicit preference, NOT a new
> `AgentState::Sleeping` variant)**, and a per-tick chronicle harness
> that walks one agent through one full simulated day.
> Governance: v3.3.17. Visual: backend-leaning (no new HUD surface) —
> Pipeline VLM no-godot-scope auto credit expected (no `.gd`/`.gdshader`/
> `.tscn`/`.tres` + no scripts/scenes path).

---

## Section 1 — Implementation Intent

Phase 5-α delivered `AgentId`, `Hunger`, and the priority-130
`HungerDecaySystem`. Phase 5-β added `Thirst`, the `AgentState` FSM
(`Idle/Seeking/Consuming`), `AgentDecisionSystem` at priority 125,
`CausalEvent::AgentDecision`, and the sparse food/water tile substrate.

Phase 5-γ **closes V7 Phase 5 (First Daily Routine)** by:

1. Adding the **third need** (`Sleep { fatigue, growth_rate }`, mirroring
   the f64 Thirst contract exactly) and its priority-132 decay system.
2. Introducing the **simulation-wide day/night clock** via two new
   `SimResources` fields: `time_of_day: f64` in `[0.0, 24.0)` and
   `ticks_per_day: u64` (default 1440 = 24 × 60, "one tick per minute").
3. Extending `TargetKind` with `Sleep` (Path b — Plan §2.3 line 286-289
   preferred symmetry path; AgentState variants are NOT modified).
4. Extending `DecisionReason` with `FatigueThresholdBreach`.
5. Adding a sparse `sleep_tiles` substrate to `SimResources` (mirrors
   `food_tiles`/`water_tiles` exactly).
6. Wiring `AgentDecisionSystem` to handle the third need: Hunger → Thirst
   → Fatigue priority ordering, `Consuming { Sleep }` decrements
   `Sleep.fatigue` and the `sleep_tiles` counter.
7. Adding a **chronicle harness** that spawns one agent on a 16×16 grid
   with placed Food/Water/Sleep tiles, runs the engine for one full day
   (1440 ticks), and asserts the full FSM trajectory closes with all
   three needs visited.

After P5-γ:
- `sim_core::components::Sleep` lives alongside `Hunger`/`Thirst`.
- `sim_core::components::TargetKind` has three variants: `Food`, `Water`,
  `Sleep`. `AgentState` is unchanged.
- `sim_core::causal::DecisionReason` has three variants:
  `HungerThresholdBreach`, `ThirstThresholdBreach`,
  `FatigueThresholdBreach`.
- `sim_engine::SimResources` has `time_of_day: f64`, `ticks_per_day: u64`,
  `sleep_tiles: HashMap<(u32, u32), u8>` plus `set_sleep_tile` helper.
- `SimEngine::tick` advances `time_of_day = ((current_tick %
  ticks_per_day) / ticks_per_day) * 24.0` before systems run.
- `sim_systems::runtime::needs::SleepDecaySystem` at priority 132 ticks
  every entity carrying `Sleep`.
- `AgentDecisionSystem` extended to handle `Sleep` need + `TargetKind::Sleep`.
- A 13-assertion harness `harness_p5_gamma_sleep_daynight_chronicle.rs`
  in `sim-test/tests/` covers component basics, system metadata,
  day/night clock determinism, FSM transitions, full-day chronicle
  trajectory, and Phase 4 + 5-α/β regression.

---

## Section 2 — Locked facts (from pre-grep — must match implementation)

| Fact | Source | Value |
|------|--------|-------|
| P5γ-1: Sleep contract | Plan §2.3 + Thirst mirror | `Sleep { fatigue: f64, growth_rate: f64 }`, `SATURATION = 100.0_f64`, `new(initial, growth_rate)` clamps `[0.0, SATURATION]`, `tick()` adds growth then min-saturates then floors at 0.0 |
| P5γ-2: time_of_day model | Plan §2.3 line 296 | `SimEngine::tick` computes `resources.time_of_day = ((resources.current_tick % resources.ticks_per_day) as f64 / resources.ticks_per_day as f64) * 24.0`. Wraps cleanly at boundary. |
| P5γ-3: ticks_per_day | Plan §2.3 line 294-295 | `pub ticks_per_day: u64`, defaults to `1440` in `SimResources::new` (24 × 60). Public field — harness may override before running. |
| P5γ-4: SleepDecaySystem | Plan §2.3 line 275-277 | `priority = 132`, `tick_interval = 1`, `name = "SleepDecaySystem"`. Walks `&mut Sleep` and calls `Sleep::tick()`. NOTE: day/night-aware rate variation is achieved via the FSM (Consuming-during-sleep decrements via the decision system's consume path); the decay system itself stays uniform — keeping it symmetric with HungerDecaySystem and ThirstDecaySystem. |
| P5γ-5: Sleeping FSM path | Plan §2.3 line 286-289 (Path b) | Extend `TargetKind` with **`Sleep`** variant. NO new `AgentState` variant. `Consuming { target: TargetKind::Sleep }` is conceptually identical to Consuming-food / Consuming-water. |
| P5γ-6: DecisionReason | Plan §2.3 line 290-291 | Add **only** `FatigueThresholdBreach` variant. `as_str()` returns `"fatigue_threshold_breach"`. |
| P5γ-7: Tile substrate | Plan §2.3 + β mirror | Add `pub sleep_tiles: HashMap<(u32, u32), u8>` to `SimResources` (initialized empty in `SimResources::new`). Add `pub fn set_sleep_tile(&mut self, x: u32, y: u32, amount: u8)` helper (mirrors `set_food_tile` exactly — `0` removes, non-zero inserts/overwrites). |
| P5γ-8: Constants | Plan §2.3 + β mirror | `FATIGUE_THRESHOLD: f64 = 50.0`, `FATIGUE_CONSUME_AMOUNT: f64 = 30.0` exported as `pub const` from `agent_decision.rs`. |
| Priority slot | Plan §2.3 + β confirmed | `90 BSS → 100 IUS → 110 sample → 120 move → 125 decide → 130 hunger_decay → 131 thirst_decay → 132 sleep_decay → 1000 viz` |
| FSM transition rule | Plan §2.3 + β extension | If `state == Idle && hunger.value > 50` → `Seeking { Food }` (Hunger wins). Else if `state == Idle && thirst.value > 50` → `Seeking { Water }` (Thirst second). Else if `state == Idle && sleep.fatigue > 50` → `Seeking { Sleep }` (Fatigue third). Deterministic priority order. |
| Seeking → Consuming | β scope extended | Triggered when agent's `Position` matches a `food_tiles` / `water_tiles` / `sleep_tiles` entry (counter > 0). Same-tick transition (no pathing). |
| Consuming → Idle | β scope extended | Section 3.10 conditional-mutation rule preserved: tile mutation conditional on `Some`, need decrement UNCONDITIONAL, state→Idle UNCONDITIONAL. Sleep variant: `sleep_tiles[(x,y)]` decremented (removed at 0), `Sleep.fatigue` decremented by `FATIGUE_CONSUME_AMOUNT` (saturating at 0.0), state → `Idle`. |
| Chronicle harness | Plan §2.3 line 303-316 | One agent on a 16×16 grid spawned at (8, 8). One Food tile at (4, 4), one Water tile at (12, 4), one Sleep tile at (8, 12). Hunger growth 1.0, Thirst growth 0.7, Fatigue growth 0.5 (so Hunger breaches first, then Thirst, then Fatigue — deterministic). Run 1440 ticks (one full simulated day). |
| Harness assertion count | Axiom #1 floor + Plan §2.3 line 320-331 | **13 assertions** (12 mandatory + 1 bonus full-day chronicle trajectory). |

**Determinism rationale** (axiom #2 — verified before assuming):
1. `time_of_day` is a pure modulo / division on `u64` tick counter — no
   wall-clock, no RNG, no floating-point accumulator drift.
2. `SleepDecaySystem::tick` iterates `world.query::<&mut Sleep>()` — same
   stable hecs archetype order as `ThirstDecaySystem`.
3. FSM priority order (Hunger → Thirst → Fatigue) is unconditional
   `if/else if` — deterministic ordering, no RNG consumed.
4. Causal parent linkage reuses the β linear scan over `world.causal_log`
   — already deterministic.
5. `sleep_tiles` is `HashMap<(u32,u32), u8>` — same deterministic point
   lookups as `food_tiles`/`water_tiles`.

---

## Section 3 — What to build (10-file scope)

### 3.1 `rust/crates/sim-core/src/components/sleep.rs` (NEW)

Exact mirror of `thirst.rs` with renamed surface. Module-doc references
P5γ-1 and the Phase 5 sub-stage progression. Use this skeleton:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sleep {
    pub fatigue: f64,
    pub growth_rate: f64,
}

impl Sleep {
    pub const SATURATION: f64 = 100.0;

    pub fn new(initial: f64, growth_rate: f64) -> Self {
        Self {
            fatigue: initial.clamp(0.0, Self::SATURATION),
            growth_rate,
        }
    }

    pub fn tick(&mut self) {
        self.fatigue = (self.fatigue + self.growth_rate).min(Self::SATURATION);
        if self.fatigue < 0.0 {
            self.fatigue = 0.0;
        }
    }
}
```

6 inline unit tests mirroring `thirst.rs`: `SATURATION` const sanity,
`new` clamps negative input, `new` clamps over-saturation input,
`tick` adds growth, `tick` clamps at saturation, serde round-trip.

### 3.2 `rust/crates/sim-core/src/components/mod.rs` (MODIFY)

Add:
```rust
pub mod sleep;
pub use sleep::Sleep;
```

Insert the `pub mod sleep;` line alphabetically (between `position` and
`thirst`) and the `pub use sleep::Sleep;` re-export alphabetically
(between `position::Position` and `thirst::Thirst`). Update the
module-doc comment to mention "Phase 5-γ adds the third need
([`Sleep`])" after the existing β paragraph.

### 3.3 `rust/crates/sim-core/src/components/agent_state.rs` (MODIFY)

Extend `TargetKind`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    Food,
    Water,
    Sleep,  // V7 Phase 5-γ / P5γ-5 — Path b symmetry (Plan §2.3 line 286-289)
}
```

The two existing tests `target_helper_returns_inner_kind`,
`only_seeking_suppresses_movement`, `serde_round_trip_each_variant`,
`target_kind_serde_round_trip` MUST be extended to include the Sleep
variant:
- `target_kind_serde_round_trip`: iterate over `[Food, Water, Sleep]`.
- `serde_round_trip_each_variant`: add `Seeking { target: Sleep }` and
  `Consuming { target: Sleep }` to the cases array.
- `only_seeking_suppresses_movement`: add
  `assert!(AgentState::Seeking { target: TargetKind::Sleep }.suppresses_movement());`
  and `assert!(!AgentState::Consuming { target: TargetKind::Sleep }.suppresses_movement());`.

`AgentState` itself is NOT modified. The module-doc comment β-era
sentence "`Sleep` enters in γ once the day/night clock lands" can be
updated to past tense ("`Sleep` landed in γ alongside the day/night
clock — see `Sleep` component and `SleepDecaySystem`").

### 3.4 `rust/crates/sim-core/src/causal/event.rs` (MODIFY)

Extend `DecisionReason`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecisionReason {
    HungerThresholdBreach,
    ThirstThresholdBreach,
    FatigueThresholdBreach,  // V7 Phase 5-γ / P5γ-6
}

impl DecisionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionReason::HungerThresholdBreach => "hunger_threshold_breach",
            DecisionReason::ThirstThresholdBreach => "thirst_threshold_breach",
            DecisionReason::FatigueThresholdBreach => "fatigue_threshold_breach",
        }
    }
}
```

The β module-doc note that "Mood/morale/social reasons land in γ/δ" can
be tightened to "Mood/morale/social reasons land in δ" (γ scope was
need-driven Fatigue, NOT mood).

Existing tests `agent_decision_accessors_and_reasons` MUST be extended
to also assert the new discriminator:
```rust
assert_eq!(
    DecisionReason::FatigueThresholdBreach.as_str(),
    "fatigue_threshold_breach"
);
```

### 3.5 `rust/crates/sim-engine/src/lib.rs` (MODIFY)

On `SimResources`:
- Add `pub time_of_day: f64` (doc: "V7 Phase 5-γ / P5γ-2. Current
  simulated time-of-day in `[0.0, 24.0)`. Refreshed by
  `SimEngine::tick` before systems run, derived deterministically from
  `current_tick % ticks_per_day`.")
- Add `pub ticks_per_day: u64` (doc: "V7 Phase 5-γ / P5γ-3. Number of
  ticks in one simulated day. Defaults to 1440 (24 × 60 = one
  tick-per-minute). Public so harness scenarios can override.")
- Add `pub sleep_tiles: HashMap<(u32, u32), u8>` (doc identical in
  shape to `food_tiles`, referencing P5γ-7).

In `SimResources::new` (or wherever the struct is instantiated):
- `time_of_day: 0.0`,
- `ticks_per_day: 1440`,
- `sleep_tiles: HashMap::new()`.

Add helper:
```rust
pub fn set_sleep_tile(&mut self, x: u32, y: u32, amount: u8) {
    if amount == 0 {
        self.sleep_tiles.remove(&(x, y));
    } else {
        self.sleep_tiles.insert((x, y), amount);
    }
}
```

In `SimEngine::tick`, **before** dispatching systems (immediately after
the existing `self.resources.current_tick = self.current_tick;` line),
advance the clock:
```rust
self.resources.time_of_day = if self.resources.ticks_per_day == 0 {
    0.0
} else {
    ((self.resources.current_tick % self.resources.ticks_per_day) as f64
        / self.resources.ticks_per_day as f64)
        * 24.0
};
```

The zero-guard prevents a division-by-zero panic if a future scenario
sets `ticks_per_day = 0` (e.g., to stop the clock). With `1440`
default, the path is never zero.

Add inline unit tests (or extend the existing nearby tests) covering:
- `time_of_day` is 0.0 at tick 0.
- `time_of_day` advances linearly across the first day (e.g., at
  `tick = 360`, `time_of_day == 6.0`).
- `time_of_day` wraps at `ticks_per_day` (at `tick = 1440`,
  `time_of_day == 0.0`).

### 3.6 `rust/crates/sim-systems/src/runtime/needs/sleep_decay.rs` (NEW)

Exact mirror of `thirst_decay.rs`. Priority **132**, tick_interval 1,
name `"SleepDecaySystem"`. Walks `&mut Sleep` and calls
`Sleep::tick()`. Module-doc updates the priority diagram to add the
132 SleepDecaySystem row.

3 inline unit tests mirroring `thirst_decay.rs`:
- `priority_is_132_interval_is_1`
- `tick_advances_fatigue`
- `tick_respects_saturation_clamp`

### 3.7 `rust/crates/sim-systems/src/runtime/needs/mod.rs` (MODIFY)

Add:
```rust
pub mod sleep_decay;
pub use sleep_decay::SleepDecaySystem;
```

(Insert alphabetically after `hunger_decay` / `thirst_decay`.)

### 3.8 `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` (MODIFY)

Extend the constant set:
```rust
pub const FATIGUE_THRESHOLD: f64 = 50.0;
pub const FATIGUE_CONSUME_AMOUNT: f64 = 30.0;
```

In `AgentDecisionSystem::tick`, modify the query to also pull
`Option<&mut Sleep>`:
```rust
let mut query = world.query::<(
    &Position,
    &Agent,
    &mut AgentState,
    Option<&mut Hunger>,
    Option<&mut Thirst>,
    Option<&mut Sleep>,
)>();
for (_entity, (pos, agent, state, hunger_opt, thirst_opt, sleep_opt)) in query.iter() {
    /* ... */
}
```

In the `Idle` arm, extend the breach chain (Fatigue is the third branch,
AFTER Thirst):
```rust
let breached = if hunger_opt
    .as_ref()
    .is_some_and(|h| h.value > HUNGER_THRESHOLD)
{
    Some((TargetKind::Food, DecisionReason::HungerThresholdBreach))
} else if thirst_opt
    .as_ref()
    .is_some_and(|t| t.value > THIRST_THRESHOLD)
{
    Some((TargetKind::Water, DecisionReason::ThirstThresholdBreach))
} else if sleep_opt
    .as_ref()
    .is_some_and(|s| s.fatigue > FATIGUE_THRESHOLD)
{
    Some((TargetKind::Sleep, DecisionReason::FatigueThresholdBreach))
} else {
    None
};
```

In the `Seeking { target }` arm, extend the resource lookup match:
```rust
let has_resource = match target {
    TargetKind::Food => resources.food_tiles.get(&key).copied().is_some_and(|v| v > 0),
    TargetKind::Water => resources.water_tiles.get(&key).copied().is_some_and(|v| v > 0),
    TargetKind::Sleep => resources.sleep_tiles.get(&key).copied().is_some_and(|v| v > 0),
};
```

In the `Consuming { target }` arm, add the Sleep variant (preserving
Section 3.10's conditional-mutation rule from β):
```rust
TargetKind::Sleep => {
    if let Some(counter) = resources.sleep_tiles.get_mut(&key) {
        *counter = counter.saturating_sub(1);
        if *counter == 0 {
            resources.sleep_tiles.remove(&key);
        }
    }
    if let Some(s) = sleep_opt {
        s.fatigue = (s.fatigue - FATIGUE_CONSUME_AMOUNT).max(0.0);
    }
    *state = AgentState::Idle;
}
```

Update the `use sim_core::components::{Agent, AgentState, Hunger,
Position, TargetKind, Thirst};` import to include `Sleep`.

Update the module-doc to:
- Add a `132  SleepDecaySystem` row to the priority diagram.
- Extend the FSM-rules block to include the Fatigue branch.
- Note that γ landed `TargetKind::Sleep` symmetry (Path b — Plan §2.3).

Extend the existing inline tests with one new test covering Fatigue
breach (mirrors `idle_to_seeking_on_thirst_when_hunger_below`):
- `idle_to_seeking_on_fatigue_when_hunger_thirst_below`: spawn agent
  with `Hunger 10, Thirst 10, Sleep::new(60.0, 0.0)`, tick once, assert
  `state == Seeking { target: Sleep }`.

The remaining β tests (`metadata`, `idle_to_seeking_on_hunger_breach`,
`idle_to_seeking_on_thirst_when_hunger_below`,
`idle_stays_idle_when_both_below`,
`seeking_transitions_to_consuming_on_food_tile`,
`consuming_decrements_food_and_hunger_then_idles`,
`consuming_removes_tile_when_counter_hits_zero`,
`breach_emits_agent_decision_event`) must stay PASSING — γ is
strictly additive on the FSM surface.

### 3.9 `rust/crates/sim-systems/src/lib.rs` (MODIFY)

- Update the priority-table doc to add "132 SleepDecaySystem" row.
- `register_needs_systems` MUST now register THREE need-decay systems
  (HungerDecaySystem + ThirstDecaySystem + **SleepDecaySystem**), single
  call. Keep the existing β registration site — γ only appends the
  third register call.

### 3.10 `rust/crates/sim-test/tests/harness_p5_gamma_sleep_daynight_chronicle.rs` (NEW, 13 assertions)

This is the new harness. Header per project convention:

```rust
//! V7 Phase 5-γ — Sleep + day/night cycle + chronicle harness
//! (closure milestone for Phase 5: First Daily Routine).
//!
//! plan_attempt: 1
//! assertions: 13
//! lane: --full
```

Test functions and what each asserts:

| # | Test name | Asserts | Type |
|---|-----------|---------|:----:|
| 1 | `harness_p5_gamma_sleep_clamps_and_ticks` | `Sleep::new` clamps `[0.0, SATURATION]`, `tick` adds growth, `tick` saturates at SATURATION, `tick` floors at 0.0 | A |
| 2 | `harness_p5_gamma_sleep_serde_round_trip` | `Sleep` serializes / deserializes losslessly via RON | A |
| 3 | `harness_p5_gamma_target_kind_sleep_variant_exists` | `TargetKind::Sleep` discriminant exists; `Seeking { target: Sleep }` and `Consuming { target: Sleep }` are constructible and serde-round-trip; `Seeking { Sleep }.suppresses_movement() == true` and `Consuming { Sleep }.suppresses_movement() == false` | A |
| 4 | `harness_p5_gamma_decision_reason_fatigue_discriminator` | `DecisionReason::FatigueThresholdBreach.as_str() == "fatigue_threshold_breach"` | A |
| 5 | `harness_p5_gamma_sleep_decay_system_metadata` | `SleepDecaySystem::priority() == 132`, `tick_interval() == 1`, `name() == "SleepDecaySystem"` | A |
| 6 | `harness_p5_gamma_time_of_day_starts_at_zero` | Fresh `SimEngine` reports `resources.time_of_day == 0.0` and `resources.ticks_per_day == 1440` | A |
| 7 | `harness_p5_gamma_time_of_day_advances_linearly` | After 360 ticks (default `ticks_per_day == 1440`), `time_of_day == 6.0` (within 1e-9). After 720 ticks, `time_of_day == 12.0`. | A |
| 8 | `harness_p5_gamma_time_of_day_wraps_at_boundary` | After exactly 1440 ticks, `time_of_day == 0.0` (within 1e-9). After 1441 ticks, `time_of_day` is back to one-tick-into-day. | A |
| 9 | `harness_p5_gamma_idle_to_seeking_sleep_on_fatigue_breach` | Idle agent with `Hunger 10`, `Thirst 10`, `Sleep::new(60.0, 0.0)` transitions to `Seeking { target: Sleep }` after one decision tick; an `AgentDecision { reason: FatigueThresholdBreach }` is pushed to the agent's tile log | A |
| 10 | `harness_p5_gamma_consuming_sleep_decrements_both_then_idles` | Agent in `Consuming { Sleep }` standing on a `sleep_tiles[(x,y)] = 2` tile, after one decision tick: tile counter becomes 1, `Sleep.fatigue` decreases by 30.0 (clamped at 0), state returns to `Idle`. Tile is removed when counter hits 0. | A |
| 11 | `harness_p5_gamma_full_day_chronicle_visits_all_three_needs` | **CHRONICLE LANDMARK.** Spawn one agent at (8, 8) on a 16×16 grid; insert one Food tile at (8, 8), one Water tile at (8, 8), one Sleep tile at (8, 8) with counter 5 each; spawn agent with `Hunger(0, 1.0)`, `Thirst(0, 0.7)`, `Sleep(0, 0.5)`. Register `AgentMovementSystem`, `AgentDecisionSystem`, `HungerDecaySystem`, `ThirstDecaySystem`, `SleepDecaySystem`. Run `engine.tick()` 1440 times. Assert: at least one `AgentDecision { reason: HungerThresholdBreach }`, at least one with `ThirstThresholdBreach`, and at least one with `FatigueThresholdBreach` appear in the agent-tile causal log over the full day. (All three tiles share `(8, 8)` so the agent doesn't need to walk — keeps the harness deterministic without pathing logic.) | D |
| 12 | `harness_p5_gamma_phase4_phase5alpha_phase5beta_regression_clean` | Re-run β surface in-place: spawn Idle agent with `Hunger 60` (no Thirst/Sleep), tick AgentDecisionSystem once, assert it transitions to `Seeking { Food }` and emits `HungerThresholdBreach`. Spawn another with `Hunger 10, Thirst 60`, tick once, assert `Seeking { Water }` + `ThirstThresholdBreach`. Phase 4 movement: spawn agent with no `AgentState`, tick `AgentMovementSystem` 16 times, assert position can change (Brownian still works). | D |
| 13 | `harness_p5_gamma_chronicle_emits_chained_agent_decisions` | **BONUS** — over the 1440-tick full-day run from test 11, assert the agent-tile causal log accumulates at least 3 distinct `AgentDecision` events (one per need), each with a unique `id`. Provides the chained-decision evidence the chronicle harness was designed for. | D |

Each test should construct a fresh `SimEngine` via `SimEngine::new(W, H, MaterialRegistry::new())`. Tests that register systems should use:
```rust
use sim_systems::{register_decision_systems, register_movement_systems, register_needs_systems};
register_movement_systems(&mut e);
register_decision_systems(&mut e);
register_needs_systems(&mut e);
```

(or equivalent — check the β harness for the canonical wiring).

Spawning a fully-needed agent for tests:
```rust
let entity = e.spawn_agent(x, y);
e.world.insert(entity, (
    AgentState::Idle,
    Hunger::new(0.0, 1.0),
    Thirst::new(0.0, 0.7),
    Sleep::new(0.0, 0.5),
)).unwrap();
```

For test 11, use 16×16 grid (`SimEngine::new(16, 16, ...)`), spawn at
`(8, 8)`, insert all three tile types at `(8, 8)` with counter 5 each:
```rust
e.resources.set_food_tile(8, 8, 5);
e.resources.set_water_tile(8, 8, 5);
e.resources.set_sleep_tile(8, 8, 5);
```

Run for 1440 ticks. With growth rates 1.0 / 0.7 / 0.5, Hunger crosses
50 first (~tick 50), Thirst second (~tick 72), Fatigue third
(~tick 100). After each consume, the need drops below threshold and
the system re-arms.

After the run, inspect `e.resources.causal_log.get(tile_idx)` where
`tile_idx = 8 * 16 + 8 = 136`. Walk the slice and collect distinct
`DecisionReason` discriminants observed.

### 3.11 (No FFI changes)

`CausalEventView` already exposes `reason: Option<&'static str>` from
β. The new `FatigueThresholdBreach` discriminator is picked up
automatically by `reason.as_str()`. No `sim-bridge` modification
required.

### 3.12 (No GDScript changes)

γ is backend-only. No new HUD surface, no new locale keys, no scene
edits. The CausalPanel UI (Phase 3-γ-2-β substrate) will pick up the
new `"fatigue_threshold_breach"` discriminator via the existing
`reason` field on `CausalEventView`. The Locale-keyed display label for
the new reason (e.g., `causal.reason.fatigue_threshold_breach`) is
intentionally deferred — wire it in a follow-up UX-polish pass after
Phase 5-γ ships and the chronicle evidence is in.

---

## Section 4 — Locale

No new locale keys in γ. The `causal.reason.fatigue_threshold_breach`
Locale key (matching the `as_str()` discriminator pattern from β) is
deferred to a follow-up UI polish pass after Phase 5-γ ships.

---

## Section 5 — Verification

```bash
# 1. Workspace tests + clippy (gate)
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail

# 2. Targeted P5-γ harness
cd rust && cargo test -p sim-test --test harness_p5_gamma_sleep_daynight_chronicle -- --nocapture

# 3. Phase 5-α regression
cd rust && cargo test -p sim-test --test harness_p5_alpha_agent_id_hunger -- --nocapture

# 4. Phase 5-β regression
cd rust && cargo test -p sim-test --test harness_p5_beta_decision -- --nocapture

# 5. Phase 4 regression (α/β/γ contracts intact)
cd rust && cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture
cd rust && cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture

# 6. Phase 3-α causal-log regression (exhaustive match site migration check)
cd rust && cargo test -p sim-test --test harness_p3_alpha_event_recording -- --nocapture
```

Expected: 13 new P5-γ tests pass + all P5-β / P5-α / P4-α/β / P3-α
regressions green + 0 clippy warnings + 0 build failures.

---

## Section 6 — Lane

`--full`. Rationale:
- Public `DecisionReason` enum extended (new variant) — every
  exhaustive `match` site (FFI mapping in `world_node.rs`, `as_str`,
  any UI-side mapping) must add the new arm or the build fails.
- `TargetKind` enum extended — same exhaustive-match cascade.
- `SimResources` schema gains three new fields (`time_of_day`,
  `ticks_per_day`, `sleep_tiles`) — every `SimResources { .. }`
  literal in tests must be updated.
- `SimEngine::tick` gains the clock-advance step before system
  dispatch — semantic change to the tick loop.
- New `Sleep` component + new `SleepDecaySystem` register through the
  workspace.
- Backend-leaning: no shader, no new HUD panel, no scene edit, no
  locale key.

---

## Section 7 — In-game verification (post-merge)

P5-γ is **backend-only by visual scope**. Agents will visibly cycle
through Hunger / Thirst / Fatigue breaches and pause-on-tile during
each Consuming step (same Brownian-suppressed behavior as β Seeking).
No new HUD readout, no causal-panel new label (the existing β
substrate already renders the `reason` discriminator string).

VLM verification: no Godot launch required for γ (no `.gd`/`.gdshader`/
`.tscn`/`.tres` edits + no scripts/scenes path edits) — Pipeline VLM
no-godot-scope auto credit expected (per v3.3.7 §2).

---

## Section 8 — Phase 5-γ scope honesty (axiom #1)

P5-γ closes V7 Phase 5 (First Daily Routine). Honest scope limits:

1. **No pathing** — `Seeking { Sleep }` still suppresses Brownian
   motion without walking toward a remote sleep tile. The chronicle
   harness co-locates all three tiles at the agent spawn so the
   trajectory closes without pathing logic. Goal-directed pathing
   lands in Phase 6 (or later) with influence-gradient sampling.
2. **No day/night-modulated decay rates** — `SleepDecaySystem` ticks
   uniformly (same rate every tick); the day/night clock is exposed as
   `time_of_day` but no system varies its behavior by time-of-day in
   γ. The clock is the substrate; rate modulation lands later when
   the simulation has a richer behavioral repertoire (Phase 6 / Phase
   7).
3. **No bed-quality / sleep-depth model** — `sleep_tiles[(x,y)]` is a
   single `u8` counter (mirrors `food_tiles`/`water_tiles`). No
   variant tile types, no per-bed fatigue-restore rate.
4. **No CausalPanel UI label** — the new
   `"fatigue_threshold_breach"` discriminator is exposed via FFI but
   no new Locale key wires the human-readable label. The β-era
   `causal.reason.*` Locale keys remain a deferred UI polish task.
5. **No save/load coverage** — `Sleep`, `TargetKind::Sleep`,
   `time_of_day`, `ticks_per_day`, and `sleep_tiles` all derive
   `Serialize`/`Deserialize`, but no save-path harness is added in γ.
6. **No PRS / temperament-modulated thresholds** —
   `FATIGUE_THRESHOLD = 50.0` is a flat constant. Per-agent thresholds
   land later (Phase 6).
7. **No sleep-tile spawning mechanic** — γ only adds the substrate +
   `set_sleep_tile` helper; test scenarios populate tiles manually.

---

## Section 9 — Out of scope

- Goal-directed pathing toward food/water/sleep tiles (Phase 6+)
- Day/night-modulated decay rates (Phase 6+)
- CausalPanel UI Locale label for `fatigue_threshold_breach` (UI polish)
- PRS / temperament-modulated thresholds (Phase 6)
- LodTier throttling for SleepDecaySystem / AgentDecisionSystem
- Save/load harness for `Sleep` / `time_of_day` / `sleep_tiles`
- Bridge FFI: agent-side reads of `Sleep.fatigue` (deferred until HUD wires it)
- Performance benchmarks for SleepDecaySystem throughput
- Sleep-tile spawning mechanic (γ only adds the substrate + helper;
  test scenarios populate tiles manually)
- Bed-quality / sleep-depth model (deferred to richer survival pass)
- Dream / nightmare events (deferred to mood/morale δ scope)

---

## Section 10 — Phase 5 closure marker (axiom #3)

This sub-stage is the **closure of V7 Phase 5 (First Daily Routine)**.
After this commit lands, the sub-stage chain α + β + γ delivers the
first observable behavioral milestone: an agent can be born, age
through hunger / thirst / fatigue, consume to satisfy each need, and
emit a causal trace `AgentDecision → AgentDecision → AgentDecision`
that the "왜?" UI can walk for any tile in the world.

Next phase (per `.harness/plans/v7_progress.md`): Phase 6 — Building
System Deep (Week 11-12). γ does NOT begin Phase 6 work; the closure
report attached to this commit's pipeline run is the canonical
transition signal.
