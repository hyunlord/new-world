# Stage 2 / P10-γ — settlement migration pathing (non-members walk to a settlement and join)

HEAD: 5f4a6947 (Stage 1 migration-unfreeze + Stage 1.5 newborn mobility landed).

## Section 1: Implementation Intent

**Why this exists.** Stage 1 (`9dce85e1`) stopped the windowed mass-freeze by
making the settlement-migration cascade arm a NO-OP: a non-member that wants to
migrate records the `SettlementReason` intent event but stays `Idle` (keeps
Brownian motion) instead of transitioning to `Seeking{Agent(member)}` with no
`SeekTarget` (which `movement.rs` freezes — Seeking suppresses Brownian, and a
target-less Seek never takes a directed step). Stage 1's own comment scopes the
real fix to P10-γ: "restores the transition with a SeekTarget(member tile) +
arrival→join." This is that fix: a non-member now walks toward a settlement
member, the existing per-tick proximity refresh auto-admits it as a member, and
it then returns to normal behavior.

**Approach + the two non-obvious findings from the Step-0 read of the live FSM
(HEAD 5f4a6947) — these correct the naïve plan:**

1. **The FSM exit is NOT automatic.** `agent_decision.rs:475` matches on
   `*state`. The needs cascade + migration arm live ONLY in the `Idle` arm
   (475–980). A `Seeking{Agent}` agent runs the `Seeking` arm (981–1093),
   which never re-enters the cascade. The `Seeking{Agent}` arm's ONLY exit is a
   **mutual handshake** (a co-located peer that is seeking THIS agent back —
   `seeking_partners_by_pos` snapshot, line ~1011). A settlement member does
   NOT seek the migrant back, so a migrant that reached `Seeking{Agent(member)}`
   would be **stuck in Seeking forever** even after the proximity join makes it
   a member. Therefore Stage 2 must add an **explicit exit**: a migrant that has
   become a member (or whose target vanished) is reset to `Idle`.

2. **Discrimination needs a PERSISTENT marker, not a per-tick HashSet.** The
   social branch persists its "this Seeking{Agent} is social" classification via
   `seek_opt.is_some()` (post-pass line 1245) — which works ONLY because social
   was historically the sole owner of a `SeekTarget`. Once settlement migrants
   also carry a `SeekTarget`, that test is ambiguous. A per-tick
   `settlement_seek_entities` HashSet does NOT survive across ticks because
   Seeking agents skip the cascade where tagging happens. So we add a persistent
   zero-size marker component `SettlementMigrant` (inserted on the migration
   transition, removed on join/abort). The marker is needed ONLY for the
   exit-on-join decision; the post-pass *routing* is identical for social and
   settlement (both head to `agent_pos[pid]`).

**Tradeoffs.** Adding a sim-core marker component is the principled persistent
discriminator (travels with the entity through serde/save-load; idiomatic hecs;
no FFI snapshot exposure). The alternative per-tick HashSet cannot persist; a
`SeekTarget.kind` field would perturb the ε FFI surface. The marker is the
lowest-risk choice for an internal decision-only flag.

## Section 2: What to Build  ⚠️ LOCKED SCOPE — DO NOT EXPAND ⚠️

**Exactly four files. The change restores the migration FSM transition, adds a
persistent migrant marker, routes the migrant to the member tile (reusing the ζ
post-pass), and adds an explicit exit on join/abort. Settlement formation,
birth orchestration, the proximity-join refresh, the Social/resource loops, and
all FFI/renderer/locale are NOT touched.**

| # | Authorized file | Change |
|---|-----------------|--------|
| 1 | `rust/crates/sim-core/src/components/settlement_migrant.rs` | **New.** Zero-size marker component `SettlementMigrant` (unit struct). Derives mirror `SeekTarget`: `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]`. Doc comment + `#[cfg(test)]` serde round-trip (mirror `seek_target.rs`). |
| 2 | `rust/crates/sim-core/src/components/mod.rs` | Add `pub mod settlement_migrant;` (alpha order, after `seek_target`) and `pub use settlement_migrant::SettlementMigrant;` (after the `seek_target` use). |
| 3 | `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs` | (a) Migration arm (`else` block ~916–973): change `if target_agent_opt.is_some()` to `if let Some(target_agent) = target_agent_opt`, keep the `SettlementReason` event emit unchanged, then ADD `*state = AgentState::Seeking { target: TargetKind::Agent(target_agent) };` and tag the entity into a new per-tick `settlement_migrant_tag: HashSet<Entity>` (declare alongside `social_seek_entities` at ~460). (b) Main decision query tuple: add `Option<&SettlementMigrant>` so the `Seeking{Agent}` arm can read the marker. (c) `Seeking{Agent(pid)}` decision arm (~1011, BEFORE the mutual-handshake `has_resource` logic): if the entity HAS the `SettlementMigrant` marker, compute `is_member` (`resources.settlements.values().any(|s| s.member_agents.contains(&agent.id))`) and `target_alive` (`resources.settlements.values().any(|s| s.member_agents.contains(pid))`); if `is_member || !target_alive` → `*state = AgentState::Idle;` and push `e` to a new `migrant_clear: Vec<Entity>` (the marker is removed in the post-pass). Skip the handshake logic for that entity this tick. (d) Post-pass query (~1210): add `Option<&SettlementMigrant>` to the tuple. (e) Post-pass `Seeking{Agent(pid)}` branch (~1244): replace the `is_social` test with an explicit two-way classification — `let is_migrant = settlement_migrant_tag.contains(&e) || migrant_marker.is_some();` `let is_social = !is_migrant && (social_seek_entities.contains(&e) || seek_opt.is_some());` — and route BOTH (`(is_migrant || is_social) && *pid != agent.id`) to `agent_pos.get(pid)` exactly as today (re-resolve every tick; `None` → clear SeekTarget). (f) After the post-pass, insert `SettlementMigrant` for every entity in `settlement_migrant_tag` that lacks it (deferred `world.insert_one`), and `world.remove_one::<SettlementMigrant>(e)` for every `e` in `migrant_clear`. |
| 4 | `rust/crates/sim-test/tests/harness_p10_gamma_settlement_migration.rs` | **New** harness, ≥8 assertions (see Section 3 / Step 5). |
| 5 | `rust/crates/sim-test/tests/harness_s16_zeta_social_freeze_fix.rs` | **Behavior-spec UPDATE (allowed).** Its A4/A14 lock the pre-Stage-2 invariant "settlement migrant stays Idle / no SeekTarget" (Stage 1 unfreeze). Stage 2 restores the Seeking{Agent}+SeekTarget transition, so those assertions must be re-pointed to the new behavior ("migrant transitions to Seeking{Agent} WITH SettlementMigrant marker AND a SeekTarget" — same conjunct rigor, anti-freeze now = has-a-goal). Social-freeze assertions (A1-A3, A5-A13) MUST stay green unchanged. This mirrors how Stage 1 itself updated ζ A4/A14. |
| 6 | `rust/crates/sim-test/tests/harness_settlement_migration_unfreeze.rs` | **Behavior-spec UPDATE (allowed).** This Stage-1 harness locks "non-member stays Idle after the migration arm". Stage 2 changes that to "transitions to Seeking{Agent} with a goal". Re-point the affected assertions to the new behavior; keep every assertion that still holds (e.g. no-freeze, determinism, births). |
| 7 | `rust/crates/sim-test/tests/harness_p10_beta_settlement_system.rs` | **Stale-assumption REPAIR (allowed, test-only, assertion UNCHANGED).** `a21` stages a combat with a born agent as attacker, assuming it is "no MovementRng → pinned, always member". V7 Stage 1.5 (`5f4a6947`) gave settlement births a MovementRng, so the born agent now drifts out of proximity over the ~200 cooldown ticks and Stage 2's directed migration exposes it → `CombatCompleted.settlement_link` resolves `None` → the "member-involved combat routes" assertion fails. Fix = re-assert the attacker's membership (`settlements[sid].member_agents.insert(born_agent_id)`) immediately before the combat tick; CombatSystem (137) reads `member_agents` live before SettlementSystem (138) recomputes, so this restores the test's original pinned-member precondition. The assertion itself is NOT weakened. Do NOT touch any other p10_beta assertion. |

**Scope boundary — NOT in this ticket:** settlement formation scan; birth
orchestration (AgentBorn/membership/cooldown/formation/two-parent chain — Stage
1.5 owns the spawn, P10-β owns births); the proximity-join refresh
(`settlement_system.rs:243-309` — ALREADY auto-admits, do not modify); the
Social mutual-handshake loop; the resource gather loop; movement.rs (β step is
reused unchanged); FFI snapshot / renderers / goal-line viz / locale. The
marker must NOT be added to any FFI snapshot.

## Section 3: How to Implement

**Tick ordering context.** `AgentDecisionSystem` runs at its existing priority;
`SettlementSystem` does the proximity member refresh in its own tick. A 1-tick
lag between "migrant enters proximity" and "decision arm sees `is_member`" is
acceptable (the migrant simply resets to Idle one tick later). Determinism is
preserved: the new HashSet/Vec are used only for `.contains()` / membership
(order-independent), and marker insert/remove is per-entity.

**Step 1 — sim-core marker (files 1+2).** Create `settlement_migrant.rs`
mirroring `seek_target.rs` structure (module doc explaining it flags a
non-member currently pathing to a settlement via `Seeking{Agent(member)}`, set
on the migration transition and cleared on join/abort; never in the FFI
snapshot). Register in `mod.rs`.

**Step 2 — restore the transition + tag (file 3a/3b).** In the migration `else`
arm, after the unchanged `SettlementReason` push, transition to
`Seeking{Agent(target_agent)}` and `settlement_migrant_tag.insert(entity)`.
`target_agent` = `member_agents.iter().min()` of the lowest-id capacity
settlement (already computed as `target_agent_opt`). The candidate filter
already requires `!member_agents.is_empty()` (dissolve guard) — keep it.

**Step 3 — explicit exit (file 3c).** This is the new safety the naïve plan
missed. In the `Seeking{Agent(pid)}` decision arm, gate on the marker:
- `is_member` true → migration succeeded → `Idle` + `migrant_clear.push(e)`.
- `!target_alive` → abort → `Idle` + `migrant_clear.push(e)` (prevents a
  no-target mini-freeze and lets the agent re-evaluate next tick).
  **★ `target_alive` must check TWO things (a roster entry alone is not enough):**
  (1) the `pid` is still in some settlement's `member_agents`, AND (2) an entity
  with `Agent.id == pid` still EXISTS in the world (`agent_pos.contains_key(pid)`
  or equivalent). A target entity can despawn while its id lingers in a stale
  roster entry; if only the roster is checked the migrant would chase a ghost
  (`agent_pos[pid]` is None → post-pass clears SeekTarget → the exact target-less
  Seeking{Agent} freeze this stage prevents). Abort on EITHER condition failing.
- Otherwise (still migrating) → fall through to the existing handshake logic
  (which is a no-op for a non-reciprocated member target → stays Seeking, keeps
  walking via the post-pass SeekTarget).

Non-migrant `Seeking{Agent}` (social) entities are untouched by this gate
(marker absent) → social handshake path preserved exactly.

**Step 4 — post-pass routing + marker lifecycle (file 3d/3e/3f).** Add
`Option<&SettlementMigrant>` to the post-pass query. Classify migrant vs social
explicitly (migrant takes precedence via the marker / this-tick tag). Both route
to `agent_pos[pid]`, re-resolved every tick, defensive `*pid != agent.id` and
`None`-clears (unchanged ζ behavior). After the `seek_set`/`seek_clear` apply
loops, add two deferred loops: insert the marker for `settlement_migrant_tag`
entities lacking it, remove it for `migrant_clear` entities.

**★ β movement model (LOCKED — harness assertions MUST match this, do NOT
assume single-axis movement).** The reused β directed step (`movement.rs`
~179-182) is `dx = signum(target.x − pos.x); dy = signum(target.y − pos.y)`,
applied to BOTH axes every tick. Consequences the migration harness MUST encode
(β is forbidden from modification, so the test bends to β, never the reverse):
- **On-axis** migrant (shares exactly one coordinate with the member tile): one
  signum is 0 → single-axis step → Manhattan distance decreases by exactly 1.
- **Off-axis** migrant (differs on BOTH coordinates): both signums are ±1 → a
  DIAGONAL step → Manhattan distance decreases by exactly 2, and the agent moves
  on BOTH axes (each in the correct signum direction). It is NOT a single-axis
  −1 step. Any "directed step" assertion must therefore check **Chebyshev
  distance decreases by exactly 1** (true for both on- and off-axis), or branch
  explicitly on on-axis (−1 Manhattan) vs off-axis (−2 Manhattan, diagonal).
  Do NOT write a "single-axis, Manhattan −1" assertion for the off-axis case —
  that behavior does not exist in the locked β and would force a RE-PLAN.

**Crate/module:** all logic in `sim-systems` `runtime/decision/agent_decision.rs`
+ the `sim-core` marker. f64 sim math unaffected. No `unwrap()` in production.
No Godot types in sim-core/sim-systems.

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|-----------|
| T1 | sim-core `SettlementMigrant` marker + mod export | 🟢 DISPATCH | — |
| T2 | agent_decision.rs transition+tag+exit+post-pass | 🔴 DIRECT | T1 (shared component) |
| T3 | harness `harness_p10_gamma_settlement_migration.rs` | 🟢 DISPATCH | T2 |

T2 is DIRECT: it is the FSM wiring across one hot-path file with shared-state
ordering concerns (<120 lines). Dispatch ratio 2/3.

## Section 5: Localization Checklist

No new localization keys. (Internal simulation FSM change; no user-visible text.)

## Section 6: Verification & Notion

**Gate (all must pass, no ENV-BYPASS):**
```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
cargo test -p sim-test harness_p10 -- --nocapture 2>&1 | tail   # birth tests a13/a15/a21/a23 MUST stay green
cargo test -p sim-test harness_s16_zeta -- --nocapture 2>&1 | tail  # social freeze fix preserved
```

**Reproduction smoke (Step 7).** Bootstrap 64 + 3 startup buildings
(32,32)/(24,32)/(40,32); run to settlement formation; assert non-members
transition to `Seeking{Agent}` with the marker, receive a `SeekTarget` at the
member tile, move toward it, and the settlement's `member_agents` grows as
migrants arrive (proximity join). Pre-fix (Stage 1): non-members only wander
Idle. Post-fix: they converge on the settlement.

**Harness assertions (≥8), Type per evaluation_criteria.md:**
1. Non-member → `Seeking{Agent}` transition + `SettlementMigrant` marker attached
   + `SettlementReason` intent event (positive arm-entry proof, anti-vacuity).
2. Migrant receives `SeekTarget` equal to the target member's CURRENT tile (and
   not the migrant's own tile).
3. Migrant takes a correct directed step per the LOCKED β model (see Section 3
   β note): **Chebyshev distance to the member tile decreases by exactly 1** each
   movement tick (covers on-axis −1-Manhattan AND off-axis diagonal −2-Manhattan).
   Do NOT assert single-axis-only movement for an off-axis migrant.
4. Directed convergence: over a multi-tick run the migrant's min distance to the
   member reaches ≤ the join/proximity radius (it actually arrives, not just
   "moves once").
5. Proximity arrival → auto-join (`member_agents` contains the migrant).
6. After join, migrant exits migration: state is NOT
   `Seeking{Agent(former_target)}` and the `SettlementMigrant` marker is absent
   (assert the negative invariant, not strict `Idle` — a need may legitimately
   re-route it the same tick).
7. Social `Seeking{Agent}` is NOT misclassified (a tagged social seeker keeps its
   social routing; no marker leaks onto social seekers).
8. Settlement dissolve / target-member-leaves-roster → abort to non-migration
   (marker removed), no chase, no target-less freeze.
9. **Target ENTITY despawn while its id lingers in `member_agents`** → abort
   (marker removed), no target-less Seeking{Agent} freeze, motion resumes within
   a few ticks (the #7 production-liveness guard).
10. Long-run no-freeze: across a multi-thousand-tick run with the 3-building
    settlement, zero agents are in a target-less `Seeking{Agent}` at any sample,
    and the worst motile-agent frozen streak stays < 100 ticks.
11. On/adjacent-to-member edge case: a migrant already on the member tile still
    enters the migration FSM (transition + marker + SeekTarget), then joins
    (≤ a few ticks) and exits — the Stage-1 idle/no-transition path must NOT
    satisfy this.
12. Concurrent migrants: ≥2 simultaneous migrants each get live per-tick
    SeekTarget re-resolution after the target member moves.
13. p10 birth tests preserved (AgentBorn count + membership invariants hold);
    resource gather loop preserved (a forced-hungry agent still reaches food);
    determinism (same seed → identical migration trajectories across two runs).

**Notion:** update the V7 progress log with the P10-γ migration-pathing entry.
