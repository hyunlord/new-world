# a15 determinism fix — HashSet-ordered structural insert reshuffled hecs query order

## Section 1: Implementation Intent

`harness_scarcity_a15_determinism_of_scarcity_run` reproducibly diverged at tick 2251/5000 between
two `SimEngine`s identical except for independent HashMap `RandomState` seeds. Determinism is a
project CARDINAL, so this was the sole entry in `.harness/baseline/known_failures.txt`.

Measure-first localization (temporary instrumentation, since reverted) found the divergence chain:
- **tick 9** — the RAW hecs query iteration order of agents first diverges (same membership, a
  permutation only);
- **tick 489** — agent 14's `Inventory` Food count diverges (2 vs 3) while all resource TILES stay
  byte-identical → the gather pickup distributes the same total food among contending agents in a
  different per-agent split because the processing (query) order differs;
- **tick 2251** — the accumulated inventory difference changes a famine `StockpileConsumeSystem`
  relief outcome, so one Idle agent's Hunger (its `needs` value) diverges — the first
  fingerprint-visible difference (position/state/hp/tiles/death all identical).

Root cause: `AgentDecisionSystem` iterated the `settlement_migrant_tag` **HashSet** to apply a
**structural** `world.insert_one(SettlementMigrant)`. A component insert moves the entity into the
`SettlementMigrant` archetype, whose dense-array order IS the insertion order — so iterating the
HashSet applied the marker in `RandomState`-seed order, permuting hecs query iteration order from
that tick onward. The pre-existing comment even claimed "per-entity insert/remove is
order-independent" — true for the per-entity VALUE, false for archetype LAYOUT.

This is the one site; all other structural mutations in the runtime are already applied in sorted
or query order (deaths in query order, settlement births sorted by SettlementId, stockpile
systems sorted by AgentId + BTreeMap, regen reroute in query order). `nearest_resource_tile` /
`nearest_food_tile_in_radius` were already deterministic and are untouched.

## Section 2: What to Build

Exactly three files change. No others are authorized.

- `rust/crates/sim-systems/src/runtime/decision/agent_decision.rs`
  — In `AgentDecisionSystem::tick`'s deferred `SettlementMigrant` insert pass (~L1582), replace the
    `for e in &settlement_migrant_tag { world.insert_one(*e, SettlementMigrant) }` HashSet iteration
    with a deterministic one: collect `(AgentId, Entity)` for the tagged entities, `sort_by_key` on
    the stable `AgentId`, then `insert_one` in that order. `migrant_clear` (a `Vec` in query order)
    and the SeekTarget passes are unchanged — they become deterministic once query order is
    restored. No behavioural change to WHICH agents are tagged; only the structural-insert ORDER.
- `.harness/baseline/known_failures.txt` — remove the now-fixed
  `harness_scarcity_a15_determinism_of_scarcity_run` entry; the list is now EMPTY (0 known
  failures); header updated to reflect 0 + document the fix.
- `.harness/baseline_test_failures.txt` — count `1` → `0`.

**Locked scope boundary:** NO other sim-systems/sim-core/sim-engine/sim-bridge change; NO GDScript;
NO new component; NO change to the agent decision logic, target selection, gather/eat amounts, or
the migrant tagging conditions. Only the iteration order of the deferred SettlementMigrant insert.

## Section 3: How to Implement

```rust
let mut migrant_tag_sorted: Vec<(AgentId, hecs::Entity)> = settlement_migrant_tag
    .iter()
    .filter_map(|&e| world.get::<&Agent>(e).ok().map(|a| (a.id, e)))
    .collect();
migrant_tag_sorted.sort_by_key(|(id, _)| *id);
for (_, e) in migrant_tag_sorted {
    let _ = world.insert_one(e, SettlementMigrant);
}
```
`AgentId` is unique per agent → a total deterministic order. The immutable `world.get` borrows are
released by `collect()` before the `&mut world` `insert_one`. This mirrors the project's established
determinism pattern (the stockpile systems sort `member_agents` by AgentId).

## Section 4: Dispatch Plan

| Ticket | File/Concern | Mode | Depends on |
|--------|--------------|------|------------|
| T1 | agent_decision.rs deterministic migrant-insert | 🔴 DIRECT | — |
| T2 | clear baseline (known_failures.txt + count) | 🔴 DIRECT | T1 (a15 must pass first) |

DIRECT: a one-site determinism fix + baseline cleanup. No feature logic to dispatch.

## Section 5: Localization Checklist

**No new localization keys.**

## Section 6: Verification & Notion

```bash
# a15 passes the full 5000-tick per-tick lockstep across independently-seeded engines,
# run several times (HashMap seeds are random per process — different orderings each run):
cd rust && for i in 1 2 3; do cargo test -p sim-test --test harness_resource_scarcity \
    harness_scarcity_a15_determinism -- --nocapture; done   # ALL: "5000-tick per-tick lockstep IDENTICAL"
cd rust && cargo test --workspace      # 0 failures (a15 fixed; baseline now empty)
cd rust && cargo clippy --workspace --all-targets -- -D warnings   # clean
```
Regression: the Step-2.7 deterministic guard set-differences the gate result against the now-EMPTY
`known_failures.txt` → any failure is a NEW regression. Expect `regression_status: CLEAN` (0
failures). Settlement/membership/gather/deposit/consume harnesses (`settlements_zero`,
`membership_belonging`, the stockpile + scarcity suites) must stay green — verified by the full
gate, proving the migrant-insert ordering change is behaviour-preserving.

Scope: `git diff` touches only the three files in Section 2. No Notion page.
