# Section 16-ζ — Social Seeking{Agent} freeze-trap fix (agents no longer stop moving over time)

> Governance: Stage 67 (`944c6eaf`) → Stage 68. Lane: `--full` (sim-systems `.rs`). No ENV-BYPASS.
> Root cause reproduced AND fix prototype-verified BEFORE this prompt (see Section 1).
> A naive "fix ALL Seeking{Agent}" attempt broke 4 p10 settlement-birth tests — the
> corrected approach below distinguishes Social from Settlement at the decision source.

---

## Section 1: Implementation Intent

**Problem.** Over a long run, agents progressively stop moving. The cause is a **permanent freeze trap** in the `Seeking{Agent}` state. Three decision paths set `AgentState::Seeking { target: TargetKind::Agent(_) }`:
- **Social** — natural `SocialReason` (`agent_decision.rs:510-515` → state set at `:883`) and memory bias-flip to an Agent target (`:847`).
- **Settlement migration** — `SettlementReason` (`:941`, inside the decision `else` branch).
- (**Combat** transitions `Idle→Consuming{Agent}` directly at `:622` — it never lingers in `Seeking{Agent}`, so it is not a freeze trap.)

`AgentMovementSystem` (`movement.rs:176`) suppresses Brownian motion for **all** `Seeking{..}` states and only takes a directed step when a `SeekTarget` is attached. The α post-pass attaches a `SeekTarget` for `Food/Water/Sleep` **only** — so a `Seeking{Agent}` agent **freezes in place**, can never walk to its partner, and the mutual-handshake exit (`:979-986`) almost never fires → it stays frozen forever (a one-way ratchet).

**Reproduction (measured on HEAD `944c6eaf`, headless bootstrap = windowed-identical, 8000 ticks):** worst single-agent frozen streak **5678 ticks**; **5/64 agents permanently frozen** by tick 8000, count rising monotonically (~1 trapped per ~1100 ticks). Bootstrap creates NO settlements, so every `Seeking{Agent}` there is **Social** — confirming the user's symptom is the Social freeze.

**The trap for the fix (discovered by prototyping + the harness Evaluator).** A first attempt attached a partner `SeekTarget` to **every** `Seeking{Agent}`. That **broke 4 p10 settlement-birth tests** (`harness_p10_beta_a13/a15/a21/a23`). Mechanism (root-caused with an a15 replica): founders form a settlement, Brownian-drift, the settlement loses members; drifted non-members enter **Settlement-migration** `Seeking{Agent}`. On HEAD these **freeze** (stay near the cluster → settlement persists → birth fires). The naive fix made them **walk toward (and chase) each other**, so the cluster dispersed → settlement dissolved → **no birth**. A membership-based exclusion also failed (once the settlement fully dissolves, the member set is empty, so the exclusion stops applying mid-run).

**Fix (source-based, robust).** Distinguish Social from Settlement **at the decision, where the reason is known** — not by a fragile post-hoc heuristic. In the decision loop, tag every entity that enters `Seeking{Agent}` via the **social/needs/combat** branch (`if let Some(breached)`); the **settlement** branch (the `else`) is deliberately left untagged. The post-decision `SeekTarget` pass attaches/re-resolves a partner `SeekTarget` **only** for tagged seeks (or seeks that already carry a `SeekTarget` from a prior tick — the persistent tag, since only social seeks ever receive one). Settlement migrants never get a `SeekTarget` → stay frozen (their pathing is P10-γ's concern; p10 birth tests lock that behavior). `movement.rs` already steps toward any attached `SeekTarget` (Section 16-β, generic over all `Seeking` variants) → **no movement.rs change**.

**Prototype-verified (this exact implementation, before writing this prompt):**
- ζ freeze probe (6000 ticks, real metric = "in `Seeking{Agent}`, ≥2 tiles from partner, not moving"): worst stuck streak **1 tick** (was 5678), **0** agents stuck ≥50 ticks, **0** still-stuck at end; movers steady 53–61/64 with no decline.
- `harness_p10_beta_settlement_system`: **35/35 pass** (the regression is gone; births fire).
- `cargo clippy -p sim-systems`: clean.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly two files. `movement.rs`, `agent_state.rs` (the locked `suppresses_movement()` truth table), and `seek_target.rs` are NOT touched. No new components, no new config constants, no FFI change, no renderer change, no locale keys.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` | (a) Tag social `Seeking{Agent}` transitions in the decision loop's `if let Some(breached)` arm. (b) Extend the existing α "SeekTarget post-decision pass" with a `Seeking{Agent}` branch that attaches/re-resolves the partner's CURRENT tile for tagged (social) seeks only. Resource (`Food/Water/Sleep`) attach logic stays behaviorally identical. No other part of the file changes. |
| 2 | `rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs` | **New** harness, ≥10 assertions (see Section 3 T2). |

**Scope boundary — explicitly NOT in this ticket:** `movement.rs` (β directed-step reused as-is); `agent_state.rs` `suppresses_movement()` (locked by 5 external truth-table harnesses — `p5_beta`/`p5_gamma`/`p6_alpha`/`p7_alpha`/`p7_gamma`); the Social/Settlement/Combat *decision* logic (how agents ENTER `Seeking{Agent}` is unchanged — we only give SOCIAL ones a goal to walk to); the mutual-handshake rule; settlement-migrant pathing (P10-γ); the mutual-oscillation edge (two social agents 1 tile apart on an axis can step past each other — they MOVE, so the freeze is resolved; convergence tuning is out of scope); FFI/renderer/visuals; the stale module-doc at `agent_decision.rs:38-42`.

---

## Section 3: How to Implement

### T1 — agent_decision.rs

Four edits. All needed imports become available with edit (a).

**(a) Import `HashSet`** (line 55):
```rust
use std::collections::{HashMap, HashSet};
```

**(b) Declare the tag set + bind the loop entity handle.** Immediately before `let mut query = world.query::<(` (~line 452), add the set; and change the loop binding `_entity` → `entity`:
```rust
        // V7 Section 16-ζ — entities that enter Seeking{Agent} via the SOCIAL
        // path (natural SocialReason or a memory bias-flip to an Agent target)
        // are tagged here, the ONE place the decision reason is known. The
        // Settlement-migration proxy (the `else` branch below) sets the same
        // Seeking{Agent} state but is deliberately NOT tagged, so the post pass
        // leaves settlement migrants frozen (P10-γ owns their pathing; p10
        // birth tests lock that). Robust to fluctuating settlement membership
        // and independent of loneliness magnitude.
        let mut social_seek_entities: HashSet<hecs::Entity> = HashSet::new();
        let mut query = world.query::<(
            // ... unchanged ...
        )>();
        for (entity, (pos, agent, state, hunger_opt, thirst_opt, sleep_opt, social_opt, memory_opt)) in
            query.iter()
        {
```

**(c) Tag the social transition.** At the END of the `if let Some((natural_target, natural_reason)) = breached { ... }` block — i.e. immediately after the inner bias/natural `}` that closes the `*state = AgentState::Seeking { target: natural_target };` else-arm (~line 886), and BEFORE the `} else {` that opens the settlement branch (~line 887):
```rust
                        // V7 Section 16-ζ — tag a SOCIAL Seeking{Agent}
                        // transition. This `if let Some(breached)` arm is the
                        // needs/construction/social/combat path; the `else`
                        // below is settlement migration, deliberately left
                        // untagged so its migrant stays frozen (P10-γ scope).
                        // Only Agent targets are tagged; Food/Water/Sleep/
                        // ConstructionSite states do not match the pattern.
                        if let AgentState::Seeking { target: TargetKind::Agent(_) } = *state {
                            social_seek_entities.insert(entity);
                        }
```
(Combat does `continue` before reaching here; the settlement `else` is outside this block — so only social Agent-seeks are tagged.)

**(d) Post-decision SeekTarget pass.** Replace the existing pass's build+loop (the `let mut seek_set …` through the close of its `for` loop — NOT the trailing `for (e, tile) in seek_set { insert_one }` / `for e in seek_clear { remove_one }` apply blocks, which stay as-is) with:
```rust
        // V7 Section 16-ζ — partner-position map (Agent.id → current tile) for
        // the Social Seeking{Agent} branch below. Lookups only → deterministic.
        let agent_pos: HashMap<AgentId, (u32, u32)> = world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(_, (a, p))| (a.id, (p.x, p.y)))
            .collect();

        let mut seek_set: Vec<(hecs::Entity, (u32, u32))> = Vec::new();
        let mut seek_clear: Vec<hecs::Entity> = Vec::new();
        for (e, (agent, pos, state, seek_opt)) in world
            .query::<(&Agent, &Position, &AgentState, Option<&SeekTarget>)>()
            .iter()
        {
            match state {
                // Resource seeks: set ONCE, keep stable (α behavior unchanged —
                // resource tiles are fixed, so never re-target while seeking).
                AgentState::Seeking { target: TargetKind::Food }
                | AgentState::Seeking { target: TargetKind::Water }
                | AgentState::Seeking { target: TargetKind::Sleep } => {
                    if seek_opt.is_none() {
                        let t = match state {
                            AgentState::Seeking { target: TargetKind::Food } => &resources.food_tiles,
                            AgentState::Seeking { target: TargetKind::Water } => &resources.water_tiles,
                            _ => &resources.sleep_tiles,
                        };
                        if let Some(tile) = nearest_resource_tile(pos, t) {
                            seek_set.push((e, tile));
                        }
                        // None ⇒ empty resource map → skip the attach (no panic).
                    }
                    // else keep existing (stability).
                }
                // Section 16-ζ — SOCIAL Seeking{Agent} heads toward the partner's
                // CURRENT tile, RE-RESOLVED every tick (the partner moves, unlike
                // a fixed resource tile). A seek is SOCIAL iff it was tagged at
                // its decision THIS tick (`social_seek_entities`) OR it already
                // carries a SeekTarget from a prior tick — the persistent tag,
                // since only social seeks ever receive one. The Settlement proxy
                // is never tagged and never gets a SeekTarget, so it stays frozen
                // (P10-γ owns its pathing; p10 birth tests lock that). Robust to
                // fluctuating membership; independent of loneliness magnitude.
                // Combat never lingers in Seeking{Agent} (direct Idle→Consuming).
                // Defensive: never chase self / a missing partner.
                AgentState::Seeking { target: TargetKind::Agent(pid) } => {
                    let is_social = social_seek_entities.contains(&e) || seek_opt.is_some();
                    let partner_tile = if is_social && *pid != agent.id {
                        agent_pos.get(pid).copied()
                    } else {
                        None
                    };
                    match partner_tile {
                        Some(tile) => {
                            if seek_opt.map(|s| s.tile) != Some(tile) {
                                seek_set.push((e, tile));
                            }
                        }
                        None => {
                            if seek_opt.is_some() {
                                seek_clear.push(e);
                            }
                        }
                    }
                }
                // ConstructionSite seek (co-located) / Idle / Consuming → no
                // goal; drop any stale SeekTarget.
                _ => {
                    if seek_opt.is_some() {
                        seek_clear.push(e);
                    }
                }
            }
        }
```

**Determinism:** `agent_pos`/`social_seek_entities` are used by membership lookup only (never iterated for ordering), so results are independent of `HashMap`/`HashSet` order. `insert_one` overwrites, so per-tick re-targeting is correct. movement@120 runs before decision@125 → the directed step consumes the prior tick's `SeekTarget` (identical to how β consumes resource targets).

### T2 — harness_s16_zeta_social_freeze_fix.rs (≥10 assertions, non-circular)

Imports mirror `harness_s16_alpha_seektarget.rs` + `harness_s16_delta_need_stagger.rs` (`bootstrap_spawn_agents`, `register_default_runtime_systems`, `SimEngine`, components, `AgentDecisionSystem`, `AgentMovementSystem`, `Social`, `Settlement` API). Helper `bootstrapped()` = `SimEngine::new(64,64,…) → register_default_runtime_systems → bootstrap_spawn_agents`.

1. **(A) social tag → SeekTarget attach** — two co-located Idle agents A@(5,5), B@(5,5), BOTH with `Social` loneliness > 50; one `AgentDecisionSystem` tick ⇒ each ends `Seeking{Agent}` with a `SeekTarget` == the partner's tile (hand-checked).
2. **(A) directed step (movement reuse)** — a social `Seeking{Agent}` agent at (5,5) whose partner is at (10,5); after attach + one `AgentMovementSystem` tick it is at (6,5) (dx=+1,dy=0, hand-verified). No movement.rs change.
3. **(A) RE-TARGET each tick** — after the partner moves (manually) from (10,5) to (10,8), the next decision tick updates the seeker's `SeekTarget` to (10,8). (A resource seek would NOT update — the defining difference.)
4. **(A/D — THE regression guard) settlement migrant gets NO SeekTarget** — build a settlement with ≥1 member, plus a NON-member agent positioned so it takes the Settlement-migration arm (`Seeking{Agent(member)}` via `SettlementReason`); after a decision tick the migrant is `Seeking{Agent}` but has **no** `SeekTarget` (it was not tagged social). Use the `Settlement`/`resources.settlements` API as `harness_p10_beta` does.
5. **(A) manually-placed (untagged) Seeking{Agent} gets no target** — an agent whose state is set to `Seeking{Agent(x)}` WITHOUT going through a social decision, then a decision tick: no `SeekTarget` attached (proves the tag, not the bare state, gates the attach).
6. **(A) cleared on exit** — a social `Seeking{Agent}` carrying a `SeekTarget`, forced to `Idle` (and separately to `Consuming{Agent}`): the `SeekTarget` is removed.
7. **(A) resource set-once UNCHANGED** — a hungry agent → `Seeking{Food}` + nearest-food `SeekTarget`; move it one tile manually; next decision tick keeps the SAME original tile (not re-targeted). Guards α stability.
8. **(D, long-run — the actual bug)** — `bootstrapped()`, run **3000** `engine.tick()`s; track, per agent, the max consecutive ticks it is in `Seeking{Agent}`, ≥2 Manhattan tiles from its partner, AND not moving. Assert **worst such streak < 100**. (BEFORE fix this is in the thousands; AFTER, ~1. Co-located handshake-waiting, dist<2, is correctly excluded.)
9. **(D) settlement births still fire (cross-feature)** — replicate `harness_p10_beta_a15`'s shape (3-founder cluster + buildings, run past `BIRTH_COOLDOWN`), assert ≥1 `AgentBorn` — proving the fix does not regress settlement dynamics at the unit level. (The full `harness_p10_beta` suite is also a Section-6 gate.)
10. **(A) determinism** — two independent `bootstrapped()` engines, 1500 ticks each ⇒ identical {Agent.id → (position, SeekTarget)} maps.
11. **(A) gathering loop preserved** — a forced-hungry agent co-located with a food tile still completes `Idle→Seeking{Food}→Consuming{Food}`; α0 substrate (12 source tiles) intact after bootstrap.

Each assertion prints a `[S16-ζ Ak] … ✓` line on pass.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | decision-loop social tag + post-pass SeekTarget attach | 🟢 DISPATCH | — |
| T2 | harness `harness_s16_zeta_social_freeze_fix.rs` | 🟢 DISPATCH | T1 |

Dispatch 100%. Single-file logic change + its harness; no shared-interface or wiring work.

---

## Section 5: Localization Checklist

**No new localization keys.** Pure simulation-logic change; no user-facing text.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`

**MANDATORY regression (the naive fix broke these — they MUST stay green):**
```bash
cd rust && cargo test -p sim-test \
  --test harness_p10_beta_settlement_system \
  --test harness_s16_alpha_seektarget \
  --test harness_s16_beta_movement \
  --test harness_s16_delta_need_stagger \
  --test harness_s16_epsilon_state_viz \
  --test harness_p5_beta_decision \
  --test harness_p7_beta_social_system \
  --test harness_s16_zeta_social_freeze_fix -- --nocapture
# suppresses_movement() truth table (agent_state.rs untouched):
cd rust && cargo test -p sim-core agent_state -- --nocapture
```

**Pipeline:** `bash tools/harness/harness_pipeline.sh s16-zeta-social-freeze-fix .harness/prompts/s16-zeta-social-freeze-fix.md --full`. Set `GENERATOR_TIMEOUT_SECONDS=1800`.

**Honest disclosure (include in report):**
- Root cause **reproduced before coding** (worst frozen streak 5678 ticks, 5/64 trapped at tick 8000) and the exact fix **prototype-verified** (worst far-from-partner stuck = 1 tick; 0 trapped; `harness_p10_beta` 35/35; clippy clean) via a temporary headless probe, since deleted.
- The original "fix ALL `Seeking{Agent}`" approach **broke 4 p10 settlement-birth tests**; root-caused (settlement dissolution + chase dynamics) and corrected with the **source-based** Social/Settlement distinction. A membership-based exclusion was tried and rejected (fails once a settlement fully dissolves mid-run).
- VLM Visual Verify will NOT show this in a single screenshot (visible only over a long run; agents render sub-resolution) → treat any VLM WARNING as environmental (Rule 7 / +8), not a code defect. The harness long-run far-stuck assertion (T2 #8) + the p10-birth assertion (#9) are the authoritative proofs.
- **Known limitation (out of scope):** two social agents 1 tile apart on an axis can step past each other (oscillate) without co-locating — they MOVE (freeze symptom resolved) but that pair may not resolve Social; convergence tuning is deferred. Settlement-migrant pathing remains frozen by design (P10-γ).

**Governance chain:** Stage 67 `944c6eaf` (ε) → Stage 68 (this ζ). Also commits the G Phase A harness fix (`c51a6aec`, already landed: `validate_plan_scope_semantic` `grep` substitutions now guarded with `|| true` — they silently killed the pipeline under `set -euo pipefail` when a plan listed no file paths). After APPROVE + commit + push-verify, a **windowed run** confirms agents keep moving past tick 3000+ (the lonely ones now walk to a partner). NOT auto-proceeded. Pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt remains open (flag to user; out of scope here).
