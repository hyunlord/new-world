# Stage 1.5 — newborns get MovementRng + nonzero need rates (settlement births stop freezing)

> Governance: `9dce85e1` (Stage-1 migration unfreeze) → this. Lane: `--full` (sim-systems `.rs`). No ENV-BYPASS.
> Fixes a bug surfaced by the Stage-1 verification (it was masked by the migration freeze).

---

## Section 1: Implementation Intent

**Problem.** `SettlementSystem::run_births` (`settlement_system.rs`) spawns each newborn with this component set:
```rust
world.insert(entity, (
    AgentState::Idle,
    Hunger::new(0.0, 0.0), Thirst::new(0.0, 0.0),
    Sleep::new(0.0, 0.0),  Social::new(0.0, 0.0),
    Memory::new(),
));   // ← no MovementRng
```
Two defects make every newborn a permanently frozen agent:
1. **No `MovementRng`.** `AgentMovementSystem::tick` queries `(&mut Position, &mut MovementRng, Option<&AgentState>, Option<&SeekTarget>)` — `MovementRng` is a *required* (non-`Option`) term, so an entity lacking it is **never iterated** by the movement system → it can never take a Brownian step nor a directed one.
2. **All need growth rates are 0.0.** Even if it could move, its Hunger/Thirst/Sleep/Social never rise, so it never breaches a threshold, never enters `Seeking`, never participates in the gathering loop.

This was invisible until Stage-1 (`9dce85e1`) removed the settlement-migration freeze; the Stage-1 verification probe then measured **3/6 newborns frozen**. It accumulates slowly (one birth per settlement per `BIRTH_COOLDOWN_TICKS`=200), so over a long run the settled population degrades to stationary.

**Fix.** Give newborns the **same component set bootstrap gives every agent** (`bootstrap_spawn_agents` in `world_node.rs` attaches exactly `MovementRng, AgentState, Hunger, Thirst, Sleep, Social, Memory` — no Personality/Body/Temperament, so this set is complete): add a `MovementRng` seeded deterministically from the unique `new_agent_id`, and nonzero growth rates mirroring the bootstrap values (Hunger 0.05 / Thirst 0.08 / Sleep 0.03 / Social 0.04). Initial need values stay 0.0 (a newborn isn't hungry at birth; births are already time-staggered by the cooldown, so no initial-value stagger is needed).

**Prototype-verified (this exact change):** in the real windowed scenario (bootstrap + 3 startup buildings, 1200 ticks) — **15/15 newborns** carry a `MovementRng`, have nonzero rates on all four needs, and **all 15 moved** from their birth tile (was 3/6 frozen). `cargo test --workspace` fully green, p10 birth tests 35/35, clippy clean.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. The change is confined to the newborn spawn in `run_births`. The birth event/membership/cooldown/formation logic is NOT touched. No new components beyond the ones bootstrap already uses; no FFI/renderer/locale.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | (a) `use crate::runtime::agent::MovementRng;`. (b) Add 4 local `const` birth rates + 1 RNG-salt const (bootstrap rates live in sim-bridge, which depends on sim-systems → cannot be imported; local consts are correct). (c) In `run_births`, add `MovementRng::new(birth_seed)` to the newborn's `world.insert` tuple and change the four `Need::new(0.0, 0.0)` to `Need::new(0.0, <rate>)`. `birth_seed` derived deterministically from `new_agent_id`. No other line changes. |
| 2 | `rust/crates/sim-test/tests/harness_newborn_mobile.rs` | **New** harness, ≥8 assertions (see Section 3 T2). |

**Scope boundary — NOT in this ticket:** bootstrap (already correct); Stage 2 / P10-γ settlement migration pathing; parental genetics/trait inheritance; Personality/Body/Temperament/Skills (bootstrap doesn't attach them either — newborns match bootstrap, no more); birth event/membership/cooldown logic; initial-need stagger.

---

## Section 3: How to Implement

### T1 — settlement_system.rs

**(a) Import** (after `use sim_engine::{RuntimeSystem, SimResources};`):
```rust
use crate::runtime::agent::MovementRng;
```

**(b) Constants** (after `pub const BIRTH_COOLDOWN_TICKS`):
```rust
// V7 Stage 1.5 — newborn need growth rates + movement seed. Mirror the
// bootstrap rates (BOOTSTRAP_HUNGER_RATE etc. in sim-bridge world_node.rs);
// duplicated as local consts because that crate depends on sim-systems and
// cannot be imported here. Keep in sync with the bootstrap rates.
const BIRTH_HUNGER_RATE: f32 = 0.05;
const BIRTH_THIRST_RATE: f64 = 0.08;
const BIRTH_SLEEP_RATE: f64 = 0.03;
const BIRTH_SOCIAL_RATE: f64 = 0.04;
/// Salt so a newborn's Brownian stream does not alias a bootstrap agent's.
const BIRTH_RNG_SALT: u64 = 0xB117_0000_5EED_0001;
```
(Types matter: `Hunger::new(f32, f32)`; `Thirst`/`Sleep`/`Social::new(f64, f64)`.)

**(c) Spawn block** — locate the `world.insert(entity, ( AgentState::Idle, Hunger::new(0.0,0.0), … Memory::new() ))` in `run_births` (right after `let entity = world.spawn((Position…, Agent{…}))` and `new_agent_id`). Replace with:
```rust
// V7 Stage 1.5 — match bootstrap's component set so the newborn can move
// (MovementRng → AgentMovementSystem iterates it) and its needs rise so it
// joins the Seeking→Consuming loop. Seed derived deterministically from the
// unique new_agent_id (splitmix64 multiply + birth salt) → reproducible, and
// distinct from bootstrap streams. Initial need values stay 0.0 (newborn not
// hungry at birth; births are time-staggered by the cooldown).
let birth_seed = (new_agent_id)
    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
    .wrapping_add(BIRTH_RNG_SALT);
let _ = world.insert(
    entity,
    (
        MovementRng::new(birth_seed),
        AgentState::Idle,
        Hunger::new(0.0, BIRTH_HUNGER_RATE),
        Thirst::new(0.0, BIRTH_THIRST_RATE),
        Sleep::new(0.0, BIRTH_SLEEP_RATE),
        Social::new(0.0, BIRTH_SOCIAL_RATE),
        Memory::new(),
    ),
);
```
`new_agent_id` is `AgentId` (= `u64`), so it multiplies directly. Everything after the insert (AgentBorn emit, membership add, history routing) is UNCHANGED.

### T2 — harness_newborn_mobile.rs (≥8 assertions)

Imports + helpers mirror `harness_settlement_migration_unfreeze.rs` / `harness_s16_zeta_social_freeze_fix.rs` (`bootstrap_spawn_agents`, `enqueue_building_placed`, `register_default_runtime_systems`, `MovementRng`, the need components with their `growth_rate` fields). To force a birth use the real path: bootstrap + 3 buildings at (32,32),(24,32),(40,32) r=8 (or a `spawn_cluster`+`place_buildings` settlement like p10_beta), then run past `BIRTH_COOLDOWN_TICKS`. Newborns are the agents with `Agent.id >= 64` (bootstrap makes ids 0..63) — or capture the pre-birth id set and diff.

1. **(A) birth occurs** — after bootstrap + buildings + `BIRTH_COOLDOWN_TICKS+` ticks, ≥1 new agent exists (id ≥ 64 / not in the pre-birth set).
2. **(A) newborn has MovementRng** — every newborn entity has a `MovementRng` component.
3. **(A) newborn nonzero rates** — every newborn's `Hunger.growth_rate == 0.05` (f32), `Thirst == 0.08`, `Sleep == 0.03`, `Social.growth_rate == 0.04` (hand-compared to the constants).
4. **(A/D) newborn MOVES** — track newborn positions; after enough ticks every newborn (except possibly one born in the final ticks) has changed position from its birth tile.
5. **(A) deterministic seed** — two independent bootstrap+buildings engines run the same tick count ⇒ identical {newborn id → position} maps (the birth_seed derivation is a pure function of new_agent_id).
6. **(D) birth logic preserved** — `AgentBorn` event still emitted per birth; the newborn is added to the settlement's `member_agents` (membership unaffected by the component change).
7. **(D) bootstrap agents unaffected** — the 64 bootstrap agents still behave as before (a quick movement/҂state sanity check).
8. **(D) newborn eventually Seeks** — run a newborn long enough (its needs rise at the new rates) that it enters a `Seeking{Food/Water/Sleep}` state at least once, i.e. it joins the gathering loop (confirms the rates actually drive behavior, not just movement).

Each prints `[newborn Ak] … ✓`.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | run_births: MovementRng + nonzero rates | 🟢 DISPATCH | — |
| T2 | new harness_newborn_mobile | 🟢 DISPATCH | T1 |

Dispatch 100%. Single-block change + its harness.

---

## Section 5: Localization Checklist

**No new localization keys.** Pure simulation-logic change; no user-facing text.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`

**MANDATORY regression (births change the newborn's components — these must stay green):**
```bash
cd rust && cargo test -p sim-test \
  --test harness_p10_beta_settlement_system \
  --test harness_p10_alpha_settlement \
  --test harness_p10_gamma_settlement_chronicle \
  --test harness_settlement_migration_unfreeze \
  --test harness_s16_zeta_social_freeze_fix \
  --test harness_newborn_mobile -- --nocapture
```
p10-β **a13/a15/a21/a23** (birth-fires / birth-spawns-inside-radius / community-history / parent-chain) all stay green — the change adds components to the spawned agent, it does not touch the birth trigger, position, event, or membership.

**Pipeline:** `bash tools/harness/harness_pipeline.sh fix-newborn-movementrng .harness/prompts/fix-newborn-movementrng.md --full`. Set `GENERATOR_TIMEOUT_SECONDS=1800`.

**Honest disclosure (include in report):**
- This bug was surfaced by the Stage-1 verification (3/6 newborns frozen) and is the second half of the "agents stop moving" story: Stage-1 unfroze the 64 bootstrap agents (migration trap); Stage-1.5 unfreezes their offspring.
- Fix was prototype-verified (15/15 newborns move; workspace + p10 + clippy green), then reverted for the Generator.
- The birth rates are duplicated from sim-bridge's bootstrap constants because sim-bridge depends on sim-systems (not the reverse). A shared-constants refactor (move the rates into sim-core) is a possible later cleanup — out of scope here; the doc comment flags the sync requirement.
- Possible pipeline note: the Generator's `gen_result.md` evidence may be stale (a known test-discovery bug that looks in `sim-test/src/main.rs` instead of `tests/` — seen in ζ and Stage-1); if so, the verdict rests on independent execution. The operator re-verifies the working tree directly before commit.
- VLM Visual Verify won't show this (long-run movement property at sub-resolution); treat any VLM WARNING as environmental (Rule 7 / +8). Harness assertions #4/#8 are the authoritative proof.

**Governance chain:** `9dce85e1` (Stage-1) → this (Stage-1.5). After APPROVE + commit + push-verify + dylib rebuild, a **windowed run** (`3` key = 1× normal speed) should show Stage-1 + Stage-1.5 together: no mass freeze, and newborns also moving. NOT auto-proceeded. Then Stage 2 (P10-γ proper settlement migration). Pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt remains open (out of scope).
