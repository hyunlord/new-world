# Add starvation/dehydration death — needs → frailty → death (survival stakes + clear causality)

HEAD: 610a50cd. sim-core + sim-systems + sim-bridge + harness (`--full` lane).

## Section 1: Implementation Intent

**Why this exists.** Right now unmet needs have no consequence: `Hunger`/`Thirst`
saturate at 100 and nothing happens, so "why gather?" is weak. Adding death gives
survival meaning and clear causality (starved / died of thirst). Death currently
exists ONLY via combat (`combat_system.rs:~213`: `BodyHealth.is_dead()` →
`world.despawn` + 4 resource-map retains). Two gaps: (a) no needs-driven death;
(b) combat death does NOT clean `settlement.member_agents` (a latent leak).

**Approach — Option A (BodyHealth-unified), chosen at Step 0.** `BodyHealth`
(`sim-core`, `{hp, max_hp}`, `new()`/`apply_damage()`/`heal()`/`is_dead()`) is
trivial to attach (a 1-liner) and already drives combat death. So:
1. Attach `BodyHealth::new()` at bootstrap + birth (currently neither does —
   combat treats missing BodyHealth as instantly-dead via `unwrap_or(true)`).
2. New `StarvationSystem` (priority 139, after needs-decay 130-132, combat 137,
   settlement 138): when `Hunger.value >= SATURATION` damage hp; when
   `Thirst.value >= SATURATION` damage hp faster (thirst kills sooner — realistic);
   when BOTH needs are low and hp < max, heal slowly (recovery after eating, so a
   survived starvation spell isn't a permanent death sentence → population
   stability). hp ≤ 0 → death.
3. A shared `despawn_agent(...)` helper that BOTH StarvationSystem and
   CombatSystem call — unifies the death path (single cleanup site) AND fixes the
   combat `member_agents` leak in one place. It despawns, retains the 4 resource
   maps, removes the agent from every settlement's `member_agents` (incrementing
   `population_stats.total_deaths` / decrementing `current`), and emits a new
   `CausalEvent::AgentDied { reason }` for the chronicle ("why did they die?").

**Tradeoffs.** Option A unifies death on hp (vs Option B's separate
counter→despawn), enabling future injury/disease, at the cost of attaching
BodyHealth everywhere (trivial). Conservative damage (death ≈ 1000-1500 ticks of
CONTINUOUS saturation ≈ 33-50s @ 30 TPS) keeps survival pressure real without
population collapse; healing-on-recovery prevents a death spiral. Balancing is
verified by a population-stability harness (Section 3 T-balance), not asserted by
fiat.

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-core/src/causal/event.rs` | Add `pub enum DeathReason { Starvation, Dehydration, Combat }` (derive `Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize`; an `as_str()` like `DecisionReason`). Add `CausalEvent::AgentDied { id: EventId, parent: Option<EventId>, agent: AgentId, position: (u32,u32), reason: DeathReason, tick: u64 }` (mirror `AgentBorn`). Add the `AgentDied` arm to ALL FOUR match methods (`id()`, `parent()`, `tick()`, `channel()` → `None`) — there is no catch-all, so the compiler enforces this. Add a `#[cfg(test)]` round-trip + `DeathReason::as_str` test. |
| 2 | `rust/crates/sim-systems/src/runtime/survival/mod.rs` + register in `runtime/mod.rs` | **New module.** (a) `pub fn despawn_agent(world: &mut World, resources: &mut SimResources, entity: Entity, agent_id: AgentId, position: (u32,u32), reason: DeathReason, tick: u64)`: `world.despawn(entity)`; retain `relationships`/`interaction_progress`/`combat_pairs`/`combat_progress` (drop any key referencing `agent_id`); for each settlement, if `remove_member_agent(agent_id)` returns true set `population_stats.current = member_agents.len() as u32` and `total_deaths += 1`; push `CausalEvent::AgentDied{reason,...}` to the causal log at the death tile. (b) `pub struct StarvationSystem` (priority **139**, interval 1) implementing `RuntimeSystem`: query `(&Agent, &Position, &Hunger, &Thirst, &mut BodyHealth)`; per agent compute damage (`hunger.value >= Hunger::SATURATION` → +`STARVATION_DMG_PER_TICK`; `thirst.value >= Thirst::SATURATION` → +`DEHYDRATION_DMG_PER_TICK`), else if `hunger.value < SAFE_NEED_CEILING && thirst.value < SAFE_NEED_CEILING && hp < max_hp` → `heal(STARVATION_HEAL_PER_TICK)`; collect `(entity, agent_id, pos, reason)` for those reaching `is_dead()` (reason = Dehydration if thirst-saturated else Starvation; thirst takes precedence as the faster killer) into a deferred Vec (no despawn mid-query), then call `despawn_agent` for each after the query drops. Consts: `STARVATION_DMG_PER_TICK = 0.08`, `DEHYDRATION_DMG_PER_TICK = 0.12`, `SAFE_NEED_CEILING = 50.0`, `STARVATION_HEAL_PER_TICK = 0.05` (all f64; doc each). |
| 3 | `rust/crates/sim-systems/src/runtime/combat/combat_system.rs` | Replace the inline death-cleanup block (despawn + 4 retains) with a call to `survival::despawn_agent(world, resources, defender_entity, dead_id, defender_pos, DeathReason::Combat, tick)` — this ALSO fixes the missing `member_agents` cleanup for combat deaths. Keep the attacker→Idle reset. (defender position: read its `Position` before despawn.) |
| 4 | `rust/crates/sim-systems/src/lib.rs` | Register `StarvationSystem` (new `register_survival_systems` or fold into the existing needs registration) so it lands at priority 139. Update the module doc-comment list. |
| 5 | `rust/crates/sim-bridge/src/ffi/world_node.rs` | `bootstrap_spawn_agents`: add `BodyHealth::new()` to the agent's `world.insert` tuple (import `BodyHealth`). |
| 6 | `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` | `run_births`: add `BodyHealth::new()` to the newborn `world.insert` tuple (import `BodyHealth`). |
| 7 | `rust/crates/sim-test/tests/harness_starvation_death.rs` | **New** harness, ≥10 assertions (Section 3 T7) incl. the population-stability balance check. |
| 8 | Existing harnesses that break under starvation | **Conditional, authorized:** if the workspace gate shows an existing harness failing because an agent it relies on now starves to death, re-point it MINIMALLY — keep its assertion intent, just keep the relevant agent fed (reset Hunger/Thirst) or assert-around the death. Do NOT weaken what the test proves. List every such file in gen_result. (Likely candidates: long-run harnesses that pin a need at SATURATION — verify via the gate, do not guess.) |
| 9 | `rust/crates/sim-core/src/causal/mod.rs` | **Enum-collateral (authorized):** re-export the new `DeathReason` from `event` (`pub use event::{… DeathReason …}`). |
| 10 | `rust/crates/sim-systems/src/runtime/memory/memory_system.rs` | **Enum-collateral (authorized):** add the `CausalEvent::AgentDied { .. }` arm to its exhaustive `CausalEvent` classifier (a dead agent has no Memory to encode → non-actor leaf, same `=> None` group as SettlementDissolved). Mechanical; no behavior change. |
| 11 | `rust/crates/sim-test/tests/harness_p6_alpha_construction_components.rs`, `harness_p6_beta_construction_system.rs`, `harness_p3_alpha_event_recording.rs` | **Enum-collateral (authorized, NOT weakening):** these contain exhaustive `CausalEvent` matches with NO wildcard (intentional tripwires guarding against undocumented variant additions). Adding `CausalEvent::AgentDied` forces a one-arm addition to each (p6 classifiers → `"agent_died"`; p3 tick-extractors → `=> *tick`). The tripwire intent is preserved — the new variant is now explicitly listed, not wildcarded away. |

**Scope boundary — NOT in this ticket:** Age / natural-causes death (no Age
component — separate); injury/disease (future, BodyHealth enables it); death visual
(corpse sprite — future); the gathering/migration/freeze logic; need decay rates
(unchanged); the `is_dead`/`apply_damage`/`heal` BodyHealth API (reused as-is).

## Section 3: How to Implement

**Death timing (balance).** hp 100, continuous saturation: starving-only =
100/0.08 ≈ 1250 ticks (~42s @ 30TPS); thirsty-only = 100/0.12 ≈ 833 ticks (~28s);
both = 100/0.20 = 500 ticks (~17s). An agent that reaches a resource resets its
need below `SAFE_NEED_CEILING` and then HEALS, so only agents that genuinely
cannot reach food/water die — survival pressure, not collapse.

**Determinism.** StarvationSystem is pure arithmetic over a query + a deferred
despawn list (no RNG); `despawn_agent`'s settlement loop iterates a HashMap but
only mutates per-agent membership (order-independent). The causal-log push uses
the existing `issue_event_id()`.

**Ordering.** 139 runs after settlement (138) so a death's `member_agents` removal
is the last word that tick; the settlement proximity-sync next tick recomputes
membership from survivors. Combat (137) → settlement (138) → starvation (139).

**T7 harness `harness_starvation_death.rs` (≥10):**
1. Hunger held at SATURATION → BodyHealth.hp strictly decreases over ticks.
2. hp reaching 0 → agent despawned (gone from the world query).
3. ★ on death, the agent is removed from its settlement's `member_agents` AND
   `total_deaths` incremented (the combat-leak fix, exercised via starvation).
4. `CausalEvent::AgentDied` emitted with the correct `DeathReason` (Starvation
   when hunger-saturated; Dehydration when thirst-saturated).
5. Thirst-driven death works and is FASTER than hunger-only (fewer ticks to die).
6. A non-starving agent (needs low) never loses hp / never dies.
7. ★ Recovery: an agent damaged then fed (needs reset below SAFE_NEED_CEILING)
   HEALS (hp rises) and does NOT die.
8. Combat death now also cleans `member_agents` (call the shared helper; assert a
   combat-killed member leaves its settlement roster).
9. Bootstrap + newborn agents both carry BodyHealth (the attach landed).
10. Gathering loop preserved (a forced-hungry agent with a reachable source still
    reaches it and survives — death only when unreachable).
11. ★ Population stability + mechanism-wired (the balance gate): production
    scene (bootstrap 64 + 3 buildings), run ~5000 ticks. Assert (a)
    `StarvationSystem` is registered in the default runtime (wired), and (b) the
    live agent count is STABLE — `> 0` (no collapse), `>= 20` (no near-collapse),
    and `<= POP_CEILING` (no birth explosion). Do NOT assert `total_deaths >= 1`:
    this scene is resource-rich (12 non-depleting source tiles + the efficient
    gathering loop), so every agent reaches food/water and resets — ZERO
    starvation deaths here is the CORRECT emergent outcome. The death CONSEQUENCE
    is proven by the controlled assertions (pin a need at SATURATION → hp decays
    → death); forcing deaths in a fed scene would conflate "the mechanism kills
    when food is unreachable" with "this scene is resource-stressed". Real
    starvation tension is the follow-up resource-scarcity ticket. Record final
    live count + total deaths + total births for observability.
12. Determinism (same seed → identical death set + final population across two runs).

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | event.rs AgentDied + DeathReason | 🔴 DIRECT | — |
| T2 | survival module (despawn_agent + StarvationSystem) | 🔴 DIRECT | T1 |
| T3 | combat_system uses shared helper | 🔴 DIRECT | T2 |
| T4 | lib.rs register | 🔴 DIRECT | T2 |
| T5 | bootstrap BodyHealth | 🟢 DISPATCH | — |
| T6 | birth BodyHealth | 🟢 DISPATCH | — |
| T7 | new harness | 🟢 DISPATCH | T1-T6 |
| T8 | existing-harness re-points (if any) | 🔴 DIRECT | gate result |

T1-T4 DIRECT (shared types/death path). 3/8 dispatchable.

## Section 5: Localization Checklist

No new localization keys. (DeathReason is an internal enum; any GDScript-facing
death label is a future UI ticket. `as_str()` returns debug snake_case, not
user-visible text.)

## Section 6: Verification & Notion

**Gate (no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cargo test -p sim-test harness_p10 -- --nocapture 2>&1 | tail   # births/settlement preserved
cargo test -p sim-test harness_p9 -- --nocapture 2>&1 | tail    # combat/BodyHealth preserved
cargo test -p sim-test harness_starvation_death -- --nocapture 2>&1 | tail
```

**★ Balance check (the key risk).** The new T7.11 population-stability assertion
IS the balance gate — it fails if starvation collapses the population to 0 or if
no deaths ever occur. If it shows collapse, lower `STARVATION_DMG_PER_TICK` /
raise `STARVATION_HEAL_PER_TICK` and report the tuned values. Record the
5000-tick death/birth/final-population numbers in gen_result.

**Reproduction:** before — an agent pinned at Hunger=100 lives forever; after —
its hp decays and it dies (AgentDied/Starvation), leaving its settlement roster.

**Visual:** `--full` Visual Verify; the agent count may now decrease over a long
run (deaths) while births replenish — VLM confirms the scene still renders.

**★ dylib rebuild REQUIRED** (sim-core + sim-systems + sim-bridge): `cargo build
-p sim-bridge` before any windowed run.

**Notion:** update the V7 progress log with the starvation/death entry.
