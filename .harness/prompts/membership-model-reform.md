# Settlement Membership Model Reform — proximity-recompute → belonging model

## Section 1: Implementation Intent

**Why:** `SettlementSystem::sync_membership` rebuilds `member_agents` from scratch every
tick — `member_agents = {all agents within SETTLEMENT_PROXIMITY_RADIUS of formation_tile}`.
A member who steps outside the radius is dropped the very next tick. This makes "membership"
equal to "currently standing in the bubble", which breaks any behavior that takes a member
AWAY from the settlement (measured: the 2-2 Gather feature pulled members to far food tiles
→ roster collapsed 75→0 → settlements went member-empty → snapshot empty → A2/A6 regression).

**Reform (belonging model):**
- **Join**: an agent that ENTERS the radius becomes a member (add-only; existing members are
  NOT dropped for leaving the radius).
- **Leave**: ONLY on death (already handled by `despawn_agent`'s `remove_member_agent`) or
  migration to a different settlement (an agent belongs to at most ONE settlement).
- **Dissolve**: when a settlement has zero LIVE members (all dead) AND no buildings.
- **Determinism**: add/remove with deterministic resolution (no iteration-order dependence),
  not a wholesale HashSet replace.

Scope: membership model ONLY. The 2-2 Gather feature is a SEPARATE follow-up (stashed) that
resumes on top of this reform; do NOT add Gather/Inventory here.

**Tradeoff:** `population_stats.current` now counts live members regardless of distance (a
forager far afield still counts as population). This is the intended semantics — it is what
lets carry/store behavior (Direction-2) work without dissolving the home settlement.

## Section 2: What to Build

**File:** `rust/crates/sim-systems/src/runtime/settlement/settlement_system.rs` (the reform);
plus the new harness. No sim-core/bridge changes expected (membership is settlement-internal).

1. **`sync_membership` (≈ line 243-262) — wholesale recompute → retain-live + add-in-radius:**
   - REMOVE `let mut new_members = HashSet::new(); … settlement.member_agents = new_members;`.
   - INSTEAD, per settlement:
     - DROP only DEAD members: `member_agents.retain(|id| agent_positions.contains_key(id))`
       (the `agent_positions` map is built from live `(Agent, Position)` queries, so a member
       absent from it has despawned). Far-but-LIVE members are kept.
     - ADD in-radius live agents: for each `(agent_id, pos)` in `agent_positions` with
       `chebyshev(formation_tile, *pos) <= SETTLEMENT_PROXIMITY_RADIUS`, `member_agents.insert(id)`.
   - Keep the existing `member_buildings` retain/insert and
     `population_stats.current = member_agents.len() as u32` (now = live-member count).
2. **Migration exclusivity (one settlement per agent) — deterministic:**
   - After memberships are updated, ensure NO agent is a member of two settlements. Resolve
     deterministically: an agent in multiple rosters is kept ONLY in the settlement whose
     `formation_tile` is Manhattan-nearest to the agent's current position, tie-broken by the
     lower `SettlementId` (NOT by HashMap iteration order). Remove it from the others.
   - An agent currently in NO settlement's radius keeps its single existing membership (it does
     not get reassigned just for wandering) — exclusivity only resolves genuine double-membership.
3. **Dissolution (`run_dissolutions`, ≈ line 655):** the existing condition
   `population_stats.current == 0 && member_buildings.is_empty()` is now CORRECT as-is, because
   after the retain-live step `current == 0` ⟺ all members dead. Verify (and comment) that the
   condition now means "all live members gone", not "none currently nearby". Do NOT dissolve a
   settlement that still has a live far-away member.
4. **Founding members (`run_formation_scan`, ≈ line 310):** unchanged (founding still adds
   in-radius agents) — but a founding add must also respect exclusivity (a founder already a
   member of another settlement is moved, not duplicated).

**Scope boundary — DO NOT:** add Gather/Inventory/2-2 anything; change birth/combat/snapshot
logic (they consume `member_agents`/`current` and keep working under the new semantics); change
`SETTLEMENT_PROXIMITY_RADIUS`/`MAX_POP`; change `despawn_agent` (its `remove_member_agent` is the
death-leave path and stays); change FFI/dylib (membership is internal — but DO rebuild the dylib
if any sim-systems signature the bridge calls changes, which it should not).

## Section 3: How to Implement

1. In `sync_membership`, replace the `new_members` block with `retain(live)` + in-radius `insert`.
2. Add the exclusivity pass: build a map `agent → Vec<(settlement_id, dist)>` for double-members
   only; for each, compute the nearest by `(manhattan(formation_tile, agent_pos), settlement_id)`
   via `min_by_key`, and `remove_member_agent` from the rest. Iterate settlements/agents in a
   SORTED order (collect ids, sort) so the resolution is iteration-order-independent.
3. Recompute `population_stats.current = member_agents.len()` AFTER exclusivity resolution.
4. Confirm `run_dissolutions` unchanged; add a clarifying comment on the new meaning.
5. Confirm founding respects exclusivity (move a cross-settlement founder).
6. `cargo build`; fix any exhaustive-match / borrow issues.

**MANDATORY harness** `rust/crates/sim-test/tests/harness_membership_belonging.rs`:
- A1 (★ Type A — persistence): a member that MOVES outside `SETTLEMENT_PROXIMITY_RADIUS`
  (teleport/relocate its Position beyond the bubble, run a tick) REMAINS in `member_agents`
  (proximity-leave ≠ membership-leave). Under the OLD model it would be dropped.
- A2 (Type A — death leaves): a member that despawns is removed from `member_agents` within a tick.
- A3 (Type A — dissolution = all live members gone): a settlement with one far-but-LIVE member is
  NOT dissolved; once that member despawns (and no buildings), it IS dissolved.
- A4 (Type A — join on entry): a non-member that enters the radius becomes a member.
- A5 (★ Type A — exclusivity): an agent positioned in two settlements' radii ends up a member of
  EXACTLY ONE (the nearer formation_tile; tie → lower SettlementId), deterministically.
- A6 (★ Type A — lockstep determinism): a settlement-forming scene run twice from seed 42 to N
  ticks yields byte-identical per-tick fingerprints (mirror `harness_resource_scarcity` FNV).
- A7 (★ Type A — settlements_zero invariant preserved): replicate `production_scene` (bootstrap
  64 + 3 startup buildings, RUN_TICKS 300) and assert ≥1 settlement with a resolvable member at
  the end (the exact invariant `harness_settlements_zero_regression` A1/A2 check — must still hold).

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | `sync_membership` retain-live + add-in-radius | 🔴 DIRECT | — (core, determinism-critical) |
| T2 | exclusivity resolution (deterministic) | 🔴 DIRECT | T1 |
| T3 | dissolution comment + founding exclusivity | 🔴 DIRECT | T1 |
| T4 | `harness_membership_belonging.rs` | 🟢 DISPATCH | T1–T3 |

DIRECT for T1–T3: a single coherent change to the determinism-critical membership hot path.

## Section 5: Localization Checklist

No new localization keys. (Simulation logic only.)

## Section 6: Verification & Notion

```bash
cd rust && cargo test --workspace 2>&1 | grep "test result" | tail
cd rust && cargo test -p sim-test --test harness_settlements_zero_regression --test harness_p10_beta_settlement_system --test harness_p10_gamma_settlement_migration --test harness_settlement_migration_unfreeze 2>&1 | grep "test result"
cd rust && cargo test -p sim-test --test harness_membership_belonging -- --nocapture
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail
```

Expected: workspace green; ALL existing settlement/freeze/determinism harnesses still pass
(settlements_zero A1/A2/A6 in particular — the reform must PRESERVE formation+snapshot, not just
membership persistence); new `harness_membership_belonging` A1–A7 pass; clippy clean. ★ With
count-guard fixed (`c65be4eb`) the gate is fast; if the Generator still stalls on this sim-systems
change, eval-only recovery applies, but verify settlements_zero + lockstep BEFORE committing.

Notion: settlement membership reform (belonging model) complete; Direction-2 slice 2-2 (Gather,
stashed) resumes on top — members may now forage outside the bubble without dissolving the home.
