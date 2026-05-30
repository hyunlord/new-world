# Section 16-β — directed movement (Seeking → goal), gathering loop complete

> Governance: Stage 63 (`335ca145`) → Stage 64. Lane: `--full` (sim-systems movement behavior change). No ENV-BYPASS.
> Section 16 gathering loop: α0 substrate ✓ + α targeting ✓ → **β movement (this, final step)**. After β, hungry/thirsty agents visibly walk to resources and consume — the "freezing" symptom resolves.

---

## Section 1: Implementation Intent

α0 (`53075aff`) laid the resource substrate; α (`335ca145`) added the `SeekTarget` component (nearest resource tile, attached by `AgentDecisionSystem`'s post-decision pass to any agent in `Seeking{Food/Water/Sleep}`). But `movement.rs` still **freezes all `Seeking` agents** (verified at `335ca145`, `movement.rs:154-158`), so they never walk to `SeekTarget` — agents starve in place.

**β = directed movement.** In `AgentMovementSystem`, a `Seeking{Food/Water/Sleep}` agent that has a `SeekTarget` takes a one-tile **directed step** toward `SeekTarget.tile` (instead of being frozen). This closes the loop: breach → `Seeking` + `SeekTarget` (α) → walk to tile (β) → `Seeking→Consuming` (decision, on arrival) → need satisfied → `Idle`.

### ★ Locked constraint — the `suppresses_movement()` truth table is NOT changed
Verified at `335ca145`: `AgentState::suppresses_movement()` returns `true` for all `Seeking{..}` (and `Consuming{Agent(_)}`). **Five external harnesses assert this truth table** by calling `suppresses_movement()` directly: `harness_p5_beta_decision` (b), `harness_p5_gamma_sleep_daynight_chronicle` (c), `harness_p6_alpha_construction_components` (A16), `harness_p7_alpha_social_components`, `harness_p7_gamma_social_chronicle`. These check the **pure predicate**, not movement behavior.

**Resolution (H-Phase-A precedent):** keep `suppresses_movement()` meaning "suppresses *Brownian random walk*" → `Seeking → true` stays. A **directed step is not Brownian motion**, so doing a directed step for `Seeking{resource}+SeekTarget` does not contradict the predicate's meaning. The truth table in `agent_state.rs` is **untouched**; only `AgentMovementSystem`'s internal branching changes. The 5 external harnesses stay green because the predicate is unchanged.

### Behavioral tests also survive (verified, honest note)
`harness_p5_beta_decision` additionally has *behavioral* movement tests (beyond the truth table): `harness_p5_beta_seeking_position_frozen_idle_baseline_moves` (β-11, asserts a `Seeking` probe's position is frozen) and consume-cycle tests asserting the agent stays on the resource tile. These **still pass** under β, because:
- β-11's `make_stage1_engine` seeds **no** `food_tiles` and the test seeds none → its `Seeking{Food}` probe never receives a `SeekTarget` → β's *no-target → freeze* branch keeps it frozen. ✓
- Consume-cycle agents sit **on** the food tile → `Seeking→Consuming` the same tick → β's *Consuming → freeze* branch keeps them in place. ✓
- `Seeking{Agent(_)}` / `Seeking{ConstructionSite}` agents never get a `SeekTarget` (α only targets Food/Water/Sleep) → freeze → social/construction co-location behavior preserved. ✓

### Consuming freeze is preserved
`movement.rs:138-148` documents why `Consuming` must freeze: the decision system's 2-tick consume reads `pos.x/pos.y` to locate the tile; a step between `Seeking→Consuming` and `Consuming→Idle` would commit on the wrong tile. β keeps the `Consuming → continue` (freeze) branch.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. The `suppresses_movement()` truth table in `agent_state.rs` is NOT changed. The 5 external harnesses are NOT touched (they pass unchanged). No FFI, no renderer, no sim-engine, no new component, no new system.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-systems/src/runtime/agent/movement.rs` | Add `Option<&SeekTarget>` to the movement query; replace the freeze block so `Seeking{..}+SeekTarget` does a directed step, `Consuming{..}` and `Seeking{..}` without target freeze, `Idle` does Brownian. Update the header/branch comments. Update this file's own `#[cfg(test)]` tests (rename the seeking-freeze test + add a seeking-with-target test; preserve idle/consuming/determinism/RNG/metadata tests). |
| 2 | `rust/crates/sim-test/tests/harness_s16_beta_movement.rs` | **New** harness, ≥10 assertions incl. the full end-to-end gathering loop. |

**Movement branch logic (locked):**
```rust
// query gains Option<&SeekTarget>:
for (_, (pos, rng, state, seek)) in world
    .query::<(&mut Position, &mut MovementRng, Option<&AgentState>, Option<&SeekTarget>)>()
    .iter()
{
    if let Some(s) = state {
        // Consuming: freeze (2-tick consume commit accuracy — unchanged).
        if matches!(s, AgentState::Consuming { .. }) {
            continue;
        }
        // Seeking: directed step toward SeekTarget (β). suppresses_movement()
        // still reports true (Brownian suppressed) — a directed step is not
        // Brownian. Without a SeekTarget, freeze (defensive; α attaches one
        // for Food/Water/Sleep, none for ConstructionSite/Agent).
        if matches!(s, AgentState::Seeking { .. }) {
            if let Some(target) = seek {
                let dx = (target.tile.0 as i64 - pos.x as i64).signum();
                let dy = (target.tile.1 as i64 - pos.y as i64).signum();
                pos.x = (pos.x as i64 + dx).clamp(0, max_x) as u32;
                pos.y = (pos.y as i64 + dy).clamp(0, max_y) as u32;
            }
            continue; // no Brownian for Seeking either way
        }
    }
    // Idle (or no AgentState): Brownian (unchanged).
    let dx = rng.next_step() as i64;
    let dy = rng.next_step() as i64;
    pos.x = (pos.x as i64 + dx).clamp(0, max_x) as u32;
    pos.y = (pos.y as i64 + dy).clamp(0, max_y) as u32;
}
```
`SeekTarget` import: add to the existing `use sim_core::components::{...}` line in `movement.rs`. `max_x`/`max_y` already exist (lines 125-126). The directed step is **pure coordinate math — no RNG → inherently deterministic**.

**Scope boundary — NOT in this ticket:** any change to `agent_state.rs` / `suppresses_movement()` truth table, any change to the 5 external harnesses, A*/pathfinding (a single signum step suffices — no obstacles), resource-depletion re-targeting (sources infinite), `ConstructionSite`/`Agent` movement (no SeekTarget → freeze), starvation/death, resource carrying, any FFI/renderer/sim-engine/sim-core change.

---

## Section 3: How to Implement

### T1 — movement.rs
1. Add `SeekTarget` to the `use sim_core::components::{AgentState, Position};` import.
2. Add `Option<&SeekTarget>` as the 4th query element; bind it (`seek`).
3. Replace the single `if s.suppresses_movement() || matches!(Consuming) { continue; }` block with the Consuming-freeze / Seeking-directed / Idle-Brownian branching above. **Do NOT call `suppresses_movement()` in the new branching** — branch on `matches!(s, AgentState::Seeking{..})` / `Consuming{..}` directly (the predicate stays defined in `agent_state.rs` for the external truth-table harnesses; movement decides its own behavior). The directed step must NOT consume `rng` (keep RNG stream identical for Idle agents → determinism preserved).
4. Update the header comment block (`movement.rs:131-153`) to describe the three branches and that `suppresses_movement()` now means "suppresses Brownian" (directed steps are separate).

### T1b — movement.rs own tests (`#[cfg(test)]`)
- Rename `seeking_state_suppresses_movement` → `seeking_without_target_freezes`: insert `AgentState::Seeking{Food}` + Position + RNG but **no** `SeekTarget`, run the system, assert position unchanged.
- Add `seeking_with_target_moves_toward`: insert `Seeking{Food}` + `SeekTarget { tile: (15, 10) }` at `(10,10)`, run one tick → assert `pos.x == 11` (one step toward goal, `y` unchanged); run more ticks → assert it reaches `(15,10)` and then stays (signum 0).
- Preserve `metadata`, `splitmix_escapes_zero_seed`, `same_seed_produces_same_stream`, `agent_moves_after_one_tick`, `idle_state_still_moves`, `consuming_state_freezes_movement_in_runtime`, and the `suppresses_movement` truth-table test unchanged.

### T2 — harness (`harness_s16_beta_movement.rs`)
Build a `SimEngine`, register systems, seed resource tiles explicitly (`engine.resources.set_food_tile(x,y,amount)` — `make_stage1_engine` does not seed). ≥10 assertions:

1. **Directed step:** `Seeking{Food}` + `SeekTarget{(15,10)}` at `(10,10)`, one `AgentMovementSystem` tick → `pos.x == 11`, `pos.y == 10` (toward goal, hand-computed).
2. **Determinism:** two engines, same setup → identical positions after K ticks (directed path is RNG-free).
3. **★ Full gathering loop (end-to-end):** seed food at a known tile (e.g. `(20,16)`), spawn an agent at `(16,16)` with `Hunger.value = HUNGER_THRESHOLD + 1.0`, growth 0; run N (~20) full `engine.tick()`s → assert the agent's `Hunger.value` dropped below `HUNGER_THRESHOLD` (proves breach→Seeking→walk→arrive→Consuming→satisfied fired) and it ended `Idle`.
4. **No-target freeze:** `Seeking{Food}` + no `SeekTarget` + empty `food_tiles` → position frozen after ticks.
5. **Consuming freeze preserved:** `Consuming{Food}` agent → position frozen.
6. **Idle Brownian preserved:** `Idle` agent moves within ±1/axis over ticks.
7. **Truth table invariant:** assert `AgentState::Seeking{Food}.suppresses_movement() == true` (and `Idle == false`) — proves the predicate is unchanged.
8. **Water + Sleep** directed movement analogous to (1).
9. **α0 source preserved:** seeded source tiles at `u8::MAX` remain (not depleted by movement).
10. **Co-located target (signum 0):** agent already ON its `SeekTarget` tile → does not move (dx=dy=0).

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | movement.rs directed-step branching + comments | 🟢 DISPATCH | — |
| T1b | movement.rs own tests | 🟢 DISPATCH | T1 |
| T2 | end-to-end gathering-loop harness | 🟢 DISPATCH | T1 |

Dispatch 100%. No 🔴 DIRECT (single-file behavior change + its tests + one new harness).

---

## Section 5: Localization Checklist

**No new localization keys.** Simulation-only change.

---

## Section 6: Verification & Notion

**Gate:**
```bash
cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```
**★ External truth-table harnesses MUST stay green** (the change preserves `suppresses_movement()` so they should pass unchanged):
```bash
cd rust && cargo test -p sim-test --test harness_p5_beta_decision \
  --test harness_p5_gamma_sleep_daynight_chronicle \
  --test harness_p6_alpha_construction_components \
  --test harness_p7_alpha_social_components \
  --test harness_p7_gamma_social_chronicle -- --nocapture
```
**β harness:**
```bash
cd rust && cargo test -p sim-test --test harness_s16_beta_movement -- --nocapture
```
**Pipeline:** `bash tools/harness/harness_pipeline.sh s16-beta-movement .harness/prompts/s16-beta-movement.md --full`. Rust guards retired in Stage 61, so the sim-systems change is not blocked.

**Visual:** β is the first Section-16 step with a real visible effect — hungry/thirsty agents now walk toward resource tiles and consume. (The VLM whole-scene grader may still be sub-resolution for individual agent paths; windowed confirmation is the authoritative visual check — see next step.)

**Honest disclosure (include in report):** β closes the α0+α+β gathering loop — agents now seek, walk to, and consume resources; the "agents freeze and starve" symptom is resolved. Still out of scope (later sections): starvation/death, resource carrying/stockpiling, multi-resource prioritization beyond the existing Hunger>Thirst>Fatigue cascade.

**Governance chain:** Stage 63 `335ca145` → Stage 64 (this β). After APPROVE + commit, the next step is a **windowed Godot run to visually confirm agents walking to resources** (authoritative check the VLM can't do) — NOT auto-proceeded. The pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt also remains open.
