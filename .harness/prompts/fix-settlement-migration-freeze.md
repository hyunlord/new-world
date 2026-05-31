# Settlement-migration unfreeze (Stage 1) — agents stop freezing en masse

> Governance: Stage 68 `037799ca` (ζ) + harness `c51a6aec`/`3d07cda8` → this. Lane: `--full` (sim-systems `.rs`). No ENV-BYPASS.
> Root cause confirmed by **direct headless-Godot execution + cargo reproduction** (prior turn). Fix prototype-verified before this prompt.

---

## Section 1: Implementation Intent

**Problem (confirmed by running the real game, not just headless cargo).** In the windowed build, almost all agents freeze within ~200 ticks. Direct instrumentation of `WorldSimNode::process()` showed: `tick` advances normally, `sim_speed=1`, but by tick 27 **60 of 64 agents are in `Seeking{Agent}`** and by tick 201 **all 64**, every one frozen in place. Loneliness is ~1 at tick 27 (far below the 50 social threshold), so this is **NOT** social — it is **settlement migration**.

**Mechanism.** The scene's `world_renderer.gd::_ready()` places 3 startup buildings at (24,32),(32,32),(40,32). `SettlementSystem` forms a settlement from those buildings + nearby agents. Thereafter every **non-member** agent, when no need has breached (early game, needs low), hits the lowest-priority **settlement-migration cascade arm** (`agent_decision.rs`, the `else` branch of the `Idle` arm, ~line 887-963) and transitions to `Seeking { target: TargetKind::Agent(member) }`. That state carries **no `SeekTarget`** — `movement.rs` suppresses Brownian motion for every `Seeking` state and takes a directed step only toward an attached `SeekTarget` (Section 16-β); the settlement arm attaches none (P10-γ migration pathing is unimplemented). So the migrant **freezes**, never reaches the settlement, never becomes a member, and stays frozen forever. With a settlement present, this fires for the whole non-member population → total freeze.

**Why this is the user's bug and ζ didn't fix it.** ζ fixed the *Social* `Seeking{Agent}` freeze but **deliberately excluded** settlement migration ("P10-γ owns settlement pathing"). The settlement-migration freeze is the exact twin, left in place. The earlier "backend healthy / stale dylib" investigations missed it because they ran `bootstrap` with **no buildings** → no settlement → no migration. Adding the 3 buildings to a cargo run reproduces the Godot freeze byte-for-byte (tick 27: Idle=4, SeekAgent=60).

**Fix (Stage 1 — immediate unfreeze).** In the migration arm, **keep the `SettlementReason` intent event** (preserves p10-β A16 + community-history routing) but **remove the `Seeking{Agent}` FSM transition**. The non-member stays `Idle` → keeps Brownian motion → no freeze. The proper migration (walk to the settlement, then join) is **Stage 2 / P10-γ**, which will restore the transition together with a `SeekTarget(member tile)` + arrival→join.

**Prototype-verified (this exact change):** with bootstrap + the 3 buildings over 500 ticks — `SeekAgent` stays **0**, all 64 bootstrap agents keep moving (worst frozen streak **1** tick, was permanent), `cargo test --workspace` fully green, clippy clean.

**Out of scope (documented, NOT fixed here — CLAUDE.md rule 3):** verification surfaced a *separate* bug — `settlement_system.rs:564-578` spawns newborns with no `MovementRng`, which `movement.rs` requires, so **newborns never move** (3/6 frozen in the probe). It was masked by the migration freeze and accumulates slowly over a long run. It is NOT the mass freeze and is a birth-completeness issue (user excluded 출산 로직). Recommend as a 1-line Stage-1.5 follow-up (add `MovementRng` to the birth spawn). Not touched here.

---

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly three files. The change is confined to the settlement-migration arm of `AgentDecisionSystem`. `movement.rs`, `agent_state.rs`, `seek_target.rs`, `settlement_system.rs`, and the Social/Combat/needs cascade arms are NOT touched. No new components, config constants, FFI, renderer, or locale keys.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` | In the settlement-migration arm only (the `else` branch of the `AgentState::Idle` cascade, the block that currently does `if let Some(target_agent) = target_agent_opt { … SettlementReason event …; *state = AgentState::Seeking { target: TargetKind::Agent(target_agent) }; }`): KEEP the `SettlementReason` event push, REMOVE the `*state = Seeking{…}` transition. To avoid an unused-variable clippy error, change the guard from `if let Some(target_agent) = target_agent_opt {` to `if target_agent_opt.is_some() {` (the event body does not use `target_agent`). No other line of the file changes. |
| 2 | `rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs` | Update **A4** and **A14** — both currently assert the settlement migrant transitions to `Seeking{Agent}`. The Stage-1 fix changes that behavior, so update both to assert the migrant **stays `Idle`** (with no `SeekTarget`). Update their `// comment` + `println!` text accordingly. Do NOT touch A1–A3, A5–A13 (social/resource/determinism guards — must stay green). |
| 3 | `rust/crates/sim-test/tests/harness_settlement_migration_unfreeze.rs` | **New** harness, ≥7 assertions (see Section 3 T3). |

**Scope boundary — NOT in this ticket:** the newborn-`MovementRng` birth bug (`settlement_system.rs` — Stage 1.5); P10-γ migration pathing (Stage 2); the Social `Seeking{Agent}` fix (ζ, already done); settlement formation/birth logic; the 3 startup buildings (`world_renderer.gd`); `movement.rs` / `agent_state.rs`.

---

## Section 3: How to Implement

### T1 — agent_decision.rs (remove the migration Seeking transition)

Locate the settlement-migration arm (the `} else {` after the `if let Some((natural_target, natural_reason)) = breached` block, inside the `AgentState::Idle =>` match arm; comment "8th cascade arm: settlement migration pull", ~line 887). At its core is:
```rust
if let Some(target_agent) = target_agent_opt {
    let id = resources.issue_event_id();
    resources.causal_log.push(tile_idx, CausalEvent::AgentDecision {
        id, parent: None, agent: agent.id, position: (pos.x, pos.y),
        reason: DecisionReason::SettlementReason, tick,
    });
    *state = AgentState::Seeking { target: TargetKind::Agent(target_agent) };
}
```
Change to (keep event, drop transition; `is_some()` so `target_agent` isn't an unused binding):
```rust
// V7 Settlement-migration unfreeze (Stage 1): P10-γ migration pathing is
// unimplemented, so a Seeking{Agent(member)} transition here carries NO
// SeekTarget and freezes the migrant forever (movement.rs suppresses
// Brownian for every Seeking state; no target → no directed step). With the
// scene's 3 startup buildings a settlement forms early, so the whole
// non-member population froze (~tick 200; confirmed by headless-Godot +
// cargo reproduction). Record the migration INTENT (SettlementReason event —
// preserves p10-β A16 + community-history routing) but DO NOT transition the
// FSM; the non-member stays Idle and keeps Brownian motion. P10-γ restores
// the transition with a SeekTarget(member tile) + arrival→join.
if target_agent_opt.is_some() {
    let id = resources.issue_event_id();
    resources.causal_log.push(tile_idx, CausalEvent::AgentDecision {
        id, parent: None, agent: agent.id, position: (pos.x, pos.y),
        reason: DecisionReason::SettlementReason, tick,
    });
}
```
`candidate_ids` / `settlement_id` / `target_agent_opt` remain used (the `.is_some()` gate keeps the "a settlement-to-join exists" condition, so A16 still fires exactly when before). The `*state` is left untouched → the agent stays `Idle` (this is the Idle arm). Nothing else in the file changes — the Social tag-insert (`social_seek_entities`), resource arms, and the post-decision `SeekTarget` pass are all untouched.

### T2 — harness_s16_zeta_social_freeze_fix.rs (A4 + A14 → Idle)

**A4** (`..._a4_settlement_migrant_no_target_regression_guard`): the migrant (loneliness < threshold) currently asserts `Seeking{Agent}`. Change to:
```rust
e.tick(); // Stage-1 unfreeze: SettlementReason intent fires, NO Seeking transition
assert_eq!(agent_state(&e, migrant), AgentState::Idle,
    "A4: settlement migrant stays Idle — Stage-1 unfreeze removed the Seeking{{Agent}} \
     transition (P10-γ pathing unimplemented), so the migrant keeps Brownian motion");
assert!(seek_tile(&e, migrant).is_none(), "A4: settlement migrant must have NO SeekTarget");
```
Update the `println!` to say "stays Idle (unfrozen)".

**A14** (`..._a14_lonely_settlement_migrant_no_target`): the lonely-but-isolated migrant (loneliness > threshold, alone at (50,50) so the social arm finds no co-located peer and falls through to settlement) currently asserts `Seeking{Agent}`. Change to assert `AgentState::Idle` (same pattern), and update its header comment (which says "stays frozen by design") + `println!` to reflect the new Idle/unfrozen behavior.

Leave A1–A3, A5–A13 untouched.

### T3 — harness_settlement_migration_unfreeze.rs (new, ≥7 assertions)

Reproduces the REAL windowed scenario (bootstrap + the 3 startup buildings). Imports + helpers mirror `harness_s16_zeta_social_freeze_fix.rs` (`bootstrap_spawn_agents`, `enqueue_building_placed`, `register_default_runtime_systems`, `spawn_cluster`, `place_buildings`, `count_agent_born`, `SETTLEMENT_*`). Helper: place the 3 buildings via `enqueue_building_placed(&mut e.resources, x, y, 8)` at (32,32),(24,32),(40,32) then tick.

1. **(A) settlement forms** — bootstrap + 3 buildings, run a few ticks ⇒ `resources.settlements.len() >= 1`.
2. **(A) migrant stays Idle** — a non-member outsider (needs 0, loneliness < threshold) at a tile far from any peer, with a settlement present: after a decision tick its state is `Idle` (NOT `Seeking{Agent}`).
3. **(A) SettlementReason intent still emitted** — same setup: a `CausalEvent::AgentDecision { reason: SettlementReason, agent: outsider }` exists in the causal log (A16 preserved).
4. **(A) member emits no migration** — a settlement member does not emit `SettlementReason` (A17-equivalent).
5. **(D — THE unfreeze guard) long-run no mass freeze** — bootstrap + 3 buildings, run **300** `engine.tick()`s tracking per-agent frozen streaks (position unchanged tick-over-tick). Assert: **(a)** zero agents in `Seeking{Agent}` at the end (migration freeze gone), AND **(b)** every **bootstrap agent (id < 64)** has a max frozen streak **< 100** (the 64 originals keep moving). Newborns (id ≥ 64) are excluded from (b) — document inline that births lacking `MovementRng` is a separate out-of-scope bug.
6. **(D) gathering loop preserved** — a forced-hungry agent co-located with a food tile still completes `Idle→Seeking{Food}→Consuming{Food}`.
7. **(A) Social ζ preserved** — two co-located lonely agents mutually `Seeking{Agent}` each other still receive partner `SeekTarget`s (ζ social path unaffected by the settlement-arm change).
8. **(A) determinism** — two independent bootstrap+buildings engines, 200 ticks ⇒ identical {id→(pos,state)} maps.

Each assertion prints a `[mig-unfreeze Ak] … ✓` line.

---

## Section 4: Dispatch Plan

| Ticket | Concern | Routing | Depends on |
|--------|---------|---------|-----------|
| T1 | migration arm: drop Seeking transition, keep event | 🟢 DISPATCH | — |
| T2 | ζ A4/A14 → Idle (behavior-change guard update) | 🟢 DISPATCH | T1 |
| T3 | new unfreeze harness | 🟢 DISPATCH | T1 |

Dispatch 100%. Single-arm logic change + test updates.

---

## Section 5: Localization Checklist

**No new localization keys.** Pure simulation-logic change; no user-facing text.

---

## Section 6: Verification & Notion

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`

**MANDATORY regression (the fix changes settlement-migration behavior — these must all stay green):**
```bash
cd rust && cargo test -p sim-test \
  --test harness_p10_beta_settlement_system \
  --test harness_p10_alpha_settlement \
  --test harness_p10_gamma_settlement_chronicle \
  --test harness_s16_zeta_social_freeze_fix \
  --test harness_s16_alpha_seektarget \
  --test harness_settlement_migration_unfreeze -- --nocapture
```
Specifically p10-β **A16** (outsider emits SettlementReason — event kept ✓), **A17** (member doesn't), **A18** (no Seeking at MAX_POP + event fires when capacity opens). All three are event/negative checks — none asserts the migrant *enters* Seeking, so removing the transition keeps them green (verified in prototype).

**Pipeline:** `bash tools/harness/harness_pipeline.sh fix-settlement-migration-freeze .harness/prompts/fix-settlement-migration-freeze.md --full`. Set `GENERATOR_TIMEOUT_SECONDS=1800`.

**Honest disclosure (include in report):**
- Root cause was confirmed by **directly running headless Godot** (not just cargo) + a cargo reproduction with the 3 startup buildings (byte-for-byte match). The fix was prototype-verified (SeekAgent 60→0, bootstrap agents' worst frozen streak permanent→1, workspace + clippy clean), then reverted for the Generator.
- This corrects the prior "stale dylib was the cause" report: the stale-dylib (fixed by G Phase B) was a real but *separate*/past issue; the freeze persists with fresh ζ code because ζ excluded settlement migration. This Stage-1 fix addresses the actual mass freeze.
- **Separate bug surfaced, NOT fixed here (out of scope):** settlement births spawn newborns without `MovementRng` → newborns can't move (3/6 frozen in the probe; accumulates slowly). It is not the mass freeze. Recommend Stage 1.5 (add `MovementRng` to `settlement_system.rs` birth spawn).
- Stage-1 is a deliberate *partial* implementation: it disables migration *movement* (which doesn't exist yet) while keeping migration *intent* (the event). Full settlement migration is **Stage 2 / P10-γ** (walk to settlement + join), which restores the `Seeking` transition with a proper `SeekTarget`.
- VLM Visual Verify won't show this in a single screenshot (it's a long-run movement property); treat any VLM WARNING as environmental (Rule 7 / +8). The harness long-run assertion (T3 #5) is the authoritative proof.

**Governance chain:** `037799ca` (ζ) → `c51a6aec`/`3d07cda8` (harness G Phase A/B) → this (Stage-1 unfreeze). After APPROVE + commit + push-verify, a **windowed run** (`3` key = 1× normal speed) should show agents no longer freezing en masse past tick 200. NOT auto-proceeded. Then Stage 1.5 (newborn MovementRng) + Stage 2 (P10-γ migration pathing). Pre-existing `shelter-ring-center-fix-v1` ENV-BYPASS debt remains open (out of scope).
